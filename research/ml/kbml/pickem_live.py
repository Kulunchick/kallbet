"""Live tracking of Singapore 2026 Stage 2 Pick'em slips (post-start; NOT the Holdout evaluation).

`<gpu python> -m kbml.pickem_live [--live forecasts/data/panda-22473-matches-live-<ts>.json.gz]`
→ forecasts/singapore-2026-stage2-live.md

The pre-Stage models are used unchanged (inputs as of the cutoff 2026-11-28T04:00Z, no refit on any
Stage 2 result): `lgbm all` (the pre-registered slip's own model), `lgbm-ext`, `logreg-ext`, anchor.
The Swiss simulation is conditioned on every finished Match of the saved live listing by fixing
its winner (probability 1/0 for that pair; Swiss never rematches a pair); running Matches count as
unplayed. Pairings of later rounds follow from the actual results through the simulator's Buchholz
rule; the round-2 pairing the simulator derives is checked against the listing, and every listed
pairing of a later round too.
"""
from __future__ import annotations

import argparse
import gzip
import json
import re
import sys
from datetime import datetime, timezone
from itertools import combinations
from pathlib import Path

import numpy as np

from .pickem_check import TABLES, scores
from .pickem_forecast import ROOT, build, git_hash, key, slip
from .pickem_forecast_ext import ext_forecasts

DATA = ROOT / "forecasts" / "data"
OUT = ROOT / "forecasts" / "singapore-2026-stage2-live.md"
SLIPS = {  # name -> (3-0, 0-3, advance), from the pre-Stage forecast files
    "lgbm all B (pre-registered)": (["FUT Esports", "TheMongolz"], ["FlyQuest", "Lynn Vision"],
                                    ["Nemiga", "G2", "THE ONE", "Natus Vincere", "Liquid", "PARIVISION"]),
    "Anchor A (pre-registered file)": (["FUT Esports", "Nemiga"], ["FlyQuest", "Lynn Vision"],
                                       ["G2", "THE ONE", "Natus Vincere", "Astralis", "TheMongolz", "NIP"]),
    "logreg-ext B (not pre-registered)": (["FUT Esports", "TheMongolz"], ["FlyQuest", "Lynn Vision"],
                                          ["Nemiga", "Natus Vincere", "NIP", "Liquid", "PARIVISION", "100 Thieves"]),
}
ROLE = ("3–0", "0–3", "advance")


def latest_live() -> Path:
    files = sorted(DATA.glob("panda-22473-matches-live-*.json.gz"))
    if not files:
        raise SystemExit("no live listing saved")
    return files[-1]


def results_from(rows, names):
    """[(round, a index, b index, winner index or None, status, id)] by round then id."""
    idx = {key(n): i for i, n in enumerate(names)}
    out = []
    for r in rows:
        m = re.match(r"Round (\d):", r["name"])
        if not m or len(r["opponents"]) != 2:
            continue
        a, b = (idx[key(o["opponent"]["name"])] for o in r["opponents"])
        w = None
        if r["status"] == "finished" and r["winner_id"]:
            wname = next(o["opponent"]["name"] for o in r["opponents"] if o["opponent"]["id"] == r["winner_id"])
            w = idx[key(wname)]
        out.append((int(m[1]), a, b, w, r["status"], r["id"]))
    return sorted(out, key=lambda x: (x[0], x[5]))


def records(results, n=16):
    w, l = np.zeros(n, int), np.zeros(n, int)
    for _, a, b, win, _, _ in results:
        if win is not None:
            w[win] += 1
            l[b if win == a else a] += 1
    return w, l


def simulator_pairing(stage, results, rnd):
    """The pairs the replay simulator forms for round `rnd` (≥2) from the actual results of the earlier
    rounds (single run, the code of `pickem_check.simulate`)."""
    seed = np.arange(16) if stage.published else stage.seed
    wins, losses = np.zeros(16, int), np.zeros(16, int)
    played = np.zeros(16, np.int64)
    opponents = []
    for r in range(1, rnd):
        opp = np.zeros(16, int)
        for rr, a, b, win, _, _ in results:
            if rr != r:
                continue
            assert win is not None, "an earlier round is not finished"
            wins[win] += 1
            losses[b if win == a else a] += 1
            played[a] |= 1 << b
            played[b] |= 1 << a
            opp[a], opp[b] = b, a
        opponents.append(opp)
    diff = wins - losses
    buchholz = sum(diff[o] for o in opponents)
    priority = seed - 16 * buchholz
    pairs = set()
    r = rnd - 1
    for w in range(max(0, r - 2), min(2, r) + 1):
        l = r - w
        mask = (wins == w) & (losses == l)
        n = int(mask.sum())
        pool = np.argsort(np.where(mask, priority, 10000), kind="stable")[:n]
        chosen = None
        for pairing in TABLES[n]:
            cand = [(pool[i], pool[j]) for i, j in pairing]
            if not any(played[a] & (1 << b) for a, b in cand):
                chosen = cand
                break
        chosen = chosen or [(pool[i], pool[j]) for i, j in TABLES[n][0]]
        pairs |= {frozenset(map(int, p)) for p in chosen}
    return pairs


def conditioned(values, results):
    """Model probabilities (BO1 pairs then BO3 pairs) with every finished Match's winner fixed."""
    v = np.array(values, dtype=float).reshape(2, -1).copy()
    a_idx, b_idx = np.triu_indices(16, 1)
    pos = {(int(a), int(b)): k for k, (a, b) in enumerate(zip(a_idx, b_idx))}
    for _, a, b, win, _, _ in results:
        if win is None:
            continue
        i, j = min(a, b), max(a, b)
        v[:, pos[i, j]] = 1.0 if win == i else 0.0
    return v.reshape(-1)


def roles_of(names, s):
    roles = np.full(16, 3)
    for k, group in enumerate(s):
        for t in group:
            roles[names.index(t)] = k
    return roles


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--live", type=Path, default=None)
    ap.add_argument("--runs", type=int, default=5_000_000)
    ap.add_argument("--out", type=Path, default=OUT)
    args = ap.parse_args()
    try:
        sys.stdout.reconfigure(encoding="utf-8", line_buffering=True)
    except AttributeError:
        pass
    live = args.live or latest_live()
    rows = json.loads(gzip.decompress(live.read_bytes()))
    stage, forecasts, info = build()  # pre-Stage inputs only; refuses if its saved listing had started
    preds, _ = ext_forecasts(stage)
    models = {"lgbm all": forecasts["lgbm all"], "lgbm-ext": preds["lgbm-ext"], "logreg-ext": preds["logreg-ext"],
              "Anchor": forecasts["Anchor"]}
    names = stage.teams
    res = results_from(rows, names)
    # Checks: round 1 is the opening; each listed later round is the simulator's pairing.
    r1 = {frozenset(p) for p in stage.opening}
    assert {frozenset((a, b)) for r, a, b, *_ in res if r == 1} == r1, "round 1 differs from the opening"
    checks = []
    for rnd in range(2, 6):
        listed = {frozenset((a, b)) for r, a, b, *_ in res if r == rnd}
        if not listed:
            continue
        if any(w is None for r, a, b, w, *_ in res if r < rnd):
            checks.append((rnd, "earlier round unfinished: not checked"))
            continue
        sim = simulator_pairing(stage, res, rnd)
        assert listed == sim, f"round {rnd}: listed pairing differs from the simulator's"
        checks.append((rnd, "listed pairing = simulator's pairing"))
    finished = [x for x in res if x[3] is not None]
    w, l = records(res)
    from . import swiss_gpu
    dev = swiss_gpu.device("auto")
    out = {}
    for name, values in models.items():
        cv = conditioned(values, res)
        sims = swiss_gpu.simulate_many(stage, cv, args.runs, dev, 1_000_000, seed=33)
        # no run may contradict a known result
        assert not ((sims == 0) & (l > 0)).any() and not ((sims == 1) & (w > 0)).any(), name
        marg = np.array([(sims == k).mean(axis=0) for k in range(4)])
        out[name] = {"sims": sims, "marg": marg}
        print(name, "simulated", flush=True)
    made = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    text = render(stage, rows, res, finished, w, l, out, live, made, git_hash(), checks, args.runs)
    if args.out.name in ("singapore-2026-stage2.md", "singapore-2026-stage2-ext.md"):
        raise SystemExit("refusing to overwrite a forecast file")
    args.out.write_bytes(text.encode("utf-8") + b"\n")
    print(text)


def render(stage, rows, res, finished, w, l, out, live, made, head, checks, runs):
    names = stage.teams
    listed_at = max(r["modified_at"] for r in rows)
    lines = [
        "# PGL Major Singapore 2026 Stage 2 — LIVE tracking (post-start)",
        "",
        f"**Live, made {made} during Stage 2** at git `{head}` from the PandaScore listing saved as "
        f"`forecasts/data/{live.name}` (latest listing change {listed_at}). This is live tracking, separate from the "
        "Holdout evaluation, which scores only the pre-registered forecast (`singapore-2026-stage2.md`, unchanged). "
        "Probabilities of the remaining Matches come from the pre-Stage models only (inputs as of the cutoff "
        "2026-11-28T04:00:00Z, no refit on any Stage 2 result); finished Matches are fixed, running ones count as unplayed. "
        f"{runs:,} GPU runs per model, seed 33.",
        "",
        "## Results so far",
        "",
        "| Round | Match | Winner | Status |",
        "|---:|---|---|---|",
    ]
    for rnd, a, b, win, status, _ in res:
        lines.append(f"| {rnd} | {names[a]} vs {names[b]} | {names[win] if win is not None else '—'} | {status} |")
    lines += ["", "Pairing checks: " + "; ".join(f"round {r}: {c}" for r, c in checks) + ".", "",
              "## Current records", "", "| Team | Record |", "|---|---|"]
    for i in sorted(range(16), key=lambda i: (-(w[i] - l[i]), -w[i], names[i])):
        lines.append(f"| {names[i]} | {w[i]}–{l[i]} |")
    for sname, s in SLIPS.items():
        roles = roles_of(names, s)
        lines += ["", f"## {sname}", "",
                  f"3–0: {', '.join(s[0])}; 0–3: {', '.join(s[1])}; advance: {', '.join(s[2])}.", "",
                  "| Model | P(≥5) | P(≥6) | P(≥7) | Expected correct | Score distribution 0…10 |",
                  "|---|---:|---:|---:|---:|---|"]
        for model, o in out.items():
            sc = scores(o["sims"], roles)
            dist = np.bincount(sc, minlength=11)[:11] / len(sc)
            cells = " · ".join(f"{k}: {p:.1%}" for k, p in enumerate(dist) if p >= 0.0005)
            lines.append(f"| {model} | {(sc >= 5).mean():.1%} | {(sc >= 6).mean():.1%} | {(sc >= 7).mean():.1%} "
                         f"| {sc.mean():.2f} | {cells} |")
        lines += ["", "Per pick (probability it comes true; settled = 0% or 100% under every model):", "",
                  "| Pick | Team | Record | " + " | ".join(out) + " | State |", "|---|---|---|" + "---:|" * len(out) + "---|"]
        for k in range(3):
            for t in s[k]:
                i = names.index(t)
                ps = [o["marg"][k, i] for o in out.values()]
                state = "correct" if all(p == 1 for p in ps) else "wrong" if all(p == 0 for p in ps) else "open"
                lines.append(f"| {ROLE[k]} | {t} | {w[i]}–{l[i]} | " + " | ".join(f"{p:.1%}" for p in ps) + f" | {state} |")
    lines += ["", "## Method", "",
              "- Models and inputs: as in `singapore-2026-stage2.md` (`lgbm all`, anchor) and `singapore-2026-stage2-ext.md` "
              "(`lgbm-ext`, `logreg-ext`), rebuilt from the same saved pre-Stage files; the code refuses to run if those show a "
              "started Match.",
              "- Conditioning: each finished Match's pair gets probability 1 (or 0) in both formats; the Swiss simulator "
              "(`kbml/swiss_gpu.py`) then forms every later pairing from the results by its Buchholz rule. Round 1 is "
              "checked against the opening, each listed later round against the simulator's own pairing, and no simulated "
              "run contradicts a known result (asserted).",
              ""]
    return "\n".join(lines)


if __name__ == "__main__":
    main()
