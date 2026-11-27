"""Pre-stage Pick'em forecast for PGL Major Singapore 2026 Stage 2: python -m kbml.pickem_forecast.

Trains on every archived result plus all 33 Singapore Stage 1 results (each strictly before the
Stage 2 cutoff), forecasts all 120 pairings in BO1 and BO3, simulates the Swiss stage with the
replay's simulator and picks slips with strategy A (expected points) and B (chance of five or more).
Refuses to run once any Stage 2 match has started or has a result.
"""

import argparse
import gzip
import json
import subprocess
import sys
import time
from datetime import datetime, timezone
from itertools import combinations
from pathlib import Path

import numpy as np
import pandas as pd

from . import archive, evaluate, features, paths
from . import timeutil as T
from .anchor import Markets, Standings, anchor
from .pickem_check import expected_picks, scores, search, simulate
from .pickem_data import FIXTURES, Stage, rank_history
from .run import with_hltv
from .table import load_table
from .teams import team_key

ROOT = Path(__file__).resolve().parents[1]
DATA = ROOT / "forecasts" / "data"
OUT = ROOT / "forecasts" / "singapore-2026-stage2.md"
STAGE1, STAGE2 = 22378, 22473
# Major Swiss formats by (wins, losses) before the match: BO3 when a team can advance or be eliminated.
FORMATS = {(w, l): 3 if 2 in (w, l) else 1 for w in range(3) for l in range(3)}
CHUNK = 5000
ROLE_NAMES = ("3–0", "0–3", "advance")
# THE ONE is the 1win roster renamed after the 2026-11-02 Valve standings (same five players:
# ArtFr0st, Ax1Le, BELCHONOKK, fame, nafany). Without this the rating lookup finds an unrelated
# "the ONE" ranked 318th and the archive history of 1WIN is lost.
RENAMED = {"theone": "1win"}


def key(name):
    k = team_key(name)
    return RENAMED.get(k, k)


def rating_name(name):
    return "1win" if team_key(name) in RENAMED else name


def read(name):
    return json.loads(gzip.decompress((DATA / name).read_bytes()))


def seed_order(teams):
    """Initial seeds 1–16 from the Liquipedia Stage 2 revision (highest tiebreaker = seed 1)."""
    import re

    revision = read("lp-22473.json.gz")["query"]["pages"][0]["revisions"][0]
    entries = re.findall(r"TeamOpponent\|([^|{}]+)\|tiebreaker=(\d+)", revision["slots"]["main"]["content"])
    by_key = {team_key(t): t for t in teams}
    names = [by_key.get(team_key(name.strip())) for name, _ in sorted(entries, key=lambda e: -int(e[1]))]
    # THE ONE is "1w" on Liquipedia.
    names = [n if n else next(t for t in teams if t not in names and team_key(t) == "theone") for n in names]
    assert len(names) == 16 and set(names) == set(teams), names
    return names, revision["timestamp"]


def history_row(m, stage, standings, markets, ranks, lan):
    cutoff = m.cutoff()
    rating = standings.estimate(m.team_a.name, m.team_b.name, m.format, cutoff)
    quote = markets.quote(m, cutoff)
    vr = rating[1] if rating else None
    market = quote.p if quote else None
    row = {
        "match_id": m.id, "duplicate": False, "event_id": stage.event_id, "event": stage.event,
        "tier": stage.tier, "is_qualifier": False, "in_scope": True, "event_begins": stage.cutoff,
        "event_ended": None, "stage_id": stage.id, "stage": stage.name, "stage_kind": stage.kind,
        "stage_cutoff": stage.cutoff, "name": m.name, "round": m.round,
        "team_a_id": m.team_a.id, "team_a": m.team_a.name, "team_a_key": key(m.team_a.name),
        "team_b_id": m.team_b.id, "team_b": m.team_b.name, "team_b_key": key(m.team_b.name),
        "format": m.format, "scheduled_start": m.scheduled_start, "actual_start": m.actual_start,
        "cutoff": cutoff, "known_finish": m.known_finish(), "finished_at": m.finished_at,
        "winner_id": m.winner, "team_a_won": m.winner == m.team_a.id,
        "maps_a": m.maps_score[0] if m.maps_score else None, "maps_b": m.maps_score[1] if m.maps_score else None,
        "market_p": market, "market_source": quote.source if quote else None,
        "market_observed": quote.observed if quote else None,
        "vrs_map_p": rating[0] if rating else None, "vrs_series_p": vr,
        "vrs_observed": rating[2] if rating else None,
        "vrs_points_a": rating[3] if rating else None, "vrs_points_b": rating[4] if rating else None,
        "anchor_p": anchor(market, vr), "hltv_match_id": None, "hltv_is_lan": lan,
    }
    for suffix, t in (("a", m.team_a.name), ("b", m.team_b.name)):
        for col, days in (("hltv_rank", 0), ("hltv_rank28", 28), ("hltv_rank84", 84)):
            row[f"{col}_{suffix}"] = ranks.at(key(t), cutoff - days * T.DAY)
    return row


def build():
    """(stage, forecasts, info) with forecasts {model: 240 probabilities, BO1 pairs then BO3 pairs}."""
    s1_rows, s2_rows = read("panda-22378-matches.json.gz"), read("panda-22473-matches.json.gz")
    stage1 = archive.stage_from_pandascore(STAGE1, s1_rows)
    stage2 = archive.stage_from_pandascore(STAGE2, s2_rows)
    assert len(stage1.matches) == 33 and stage1.complete, "Stage 1 must be complete"
    started = [r["id"] for r in s2_rows if r["status"] != "not_started" or r["winner_id"] or r["begin_at"] is None
               or T.parse(r["begin_at"]) <= T.parse(r["modified_at"])]
    assert not started, f"Stage 2 matches already started: {started}"
    assert not stage2.matches, "Stage 2 has a result"
    opening_rows = sorted([r for r in s2_rows if r["name"].startswith("Round 1:")], key=lambda r: r["id"])
    pairs = [tuple(o["opponent"]["name"] for o in r["opponents"]) for r in opening_rows]
    assert len(pairs) == 8 and all(r["number_of_games"] == 1 for r in opening_rows)
    names, wiki_time = seed_order(sorted({t for p in pairs for t in p}))
    cutoff = stage2.cutoff
    lan = {"online": False, "offline": True}.get(s2_rows[0]["tournament"]["type"])

    history, maps, _ = load_table()
    history, hltv = with_hltv(history, paths.ARCHIVE)
    ranks = rank_history(history, hltv)
    fixture = archive.load(FIXTURES, with_hltv=False)
    standings, markets = Standings(fixture.standings), Markets(fixture)
    lan1 = {"online": False, "offline": True}.get(s1_rows[0]["tournament"]["type"])
    stage1_frame = pd.DataFrame([history_row(m, stage1, standings, markets, ranks, lan1) for m in stage1.matches])
    assert not set(stage1_frame.match_id) & set(history.match_id)
    results = pd.concat([history, stage1_frame], ignore_index=True)
    for c in ("round", "maps_a", "maps_b", "vrs_points_a", "vrs_points_b", "hltv_match_id",
              "event_ended", "known_finish", "market_observed", "vrs_observed"):
        results[c] = results[c].astype("Int64")
    assert (results.known_finish.dropna() < cutoff).all(), "a training result finishes after the Stage 2 cutoff"

    synthetic = []
    for fmt in ("bo1", "bo3"):
        for a, b in combinations(names, 2):
            rating = standings.estimate(rating_name(a), rating_name(b), fmt, cutoff)
            vr = rating[1] if rating else None
            blended = anchor(None, vr)
            row = {
                "match_id": -len(synthetic) - 1, "stage_id": STAGE2, "cutoff": cutoff, "known_finish": None,
                "team_a_key": key(a), "team_b_key": key(b), "format": fmt, "hltv_is_lan": lan,
                "market_p": None, "vrs_series_p": vr, "vrs_map_p": rating[0] if rating else None,
                "anchor_p": 0.5 if blended is None else blended,
                "vrs_points_a": rating[3] if rating else None, "vrs_points_b": rating[4] if rating else None,
            }
            for suffix, t in (("a", a), ("b", b)):
                for col, days in (("hltv_rank", 0), ("hltv_rank28", 28), ("hltv_rank84", 84)):
                    row[f"{col}_{suffix}"] = ranks.at(key(t), cutoff - days * T.DAY)
            synthetic.append(row)
    future = pd.DataFrame(synthetic)
    frame = pd.concat([results, future], ignore_index=True)
    x, kinds = features.compute(frame, maps, features.resolve("full"), hltv)

    cols = list(kinds)
    anti = np.array([kinds[c] == "anti" for c in cols])
    train = x.loc[results.match_id, cols].to_numpy(float)
    targets = results.team_a_won.to_numpy(float)
    tr = ((results.known_finish < cutoff) & ~results.is_qualifier).fillna(False).to_numpy(bool)
    events = evaluate.event_infos(results)
    completed = [e for e in events.values() if e.ended is not None and e.ended < cutoff]
    cls = evaluate.MODELS["lgbm"]
    params = evaluate.select(cls, completed, results.event_id.to_numpy(), tr, train, targets, anti, 150, 33)
    model = cls(anti, 33).fit(train[tr], targets[tr], params)
    forecasts = {
        "Anchor": future.anchor_p.to_numpy(),
        "lgbm all": model.predict(x.loc[future.match_id, cols].to_numpy(float)),
    }
    stage = Stage(
        STAGE2, "PGL Major Singapore 2026 / Stage 2", cutoff, names,
        [(names.index(a), names.index(b)) for a, b in pairs], FORMATS, None, True, lan, [],
    )
    s1_lines = int(stage1_frame.market_p.notna().sum())
    info = {
        "cutoff": cutoff, "wiki_revision": wiki_time, "training": int(tr.sum()),
        "stage1_rows": len(stage1_frame), "stage1_lines": s1_lines,
        "latest_archive_result": int(history.known_finish.max()), "params": params,
        "vrs_pairs": int(future.vrs_series_p.notna().sum()),
        "ranks": [ranks.at(key(t), cutoff) for t in names], "features": len(cols),
        "pandascore_modified": max(r["modified_at"] for r in s2_rows),
    }
    return stage, forecasts, info


def simulate_runs(stage, values, runs):
    rng = np.random.default_rng(33)
    return np.concatenate([simulate(stage, values, min(CHUNK, runs - i), rng) for i in range(0, runs, CHUNK)])


def slip(stage, roles):
    return [", ".join(t for i, t in enumerate(stage.teams) if roles[i] == k) for k in range(3)]


def check_gpu(stage, values, runs, dev, log, batch=1_000_000):
    """numpy vs torch on `runs` simulations each: wall times and z-scores of every team's outcome shares."""
    from . import swiss_gpu

    t0 = time.perf_counter()
    cpu = simulate_runs(stage, values, runs)
    cpu_seconds = time.perf_counter() - t0
    t0 = time.perf_counter()
    gpu = swiss_gpu.simulate_many(stage, values, runs, dev, batch, seed=34)
    gpu_seconds = time.perf_counter() - t0
    p = np.array([(cpu == k).mean(axis=0) for k in range(4)])
    q = np.array([(gpu == k).mean(axis=0) for k in range(4)])
    se = np.sqrt((p * (1 - p) + q * (1 - q)) / runs)
    z = np.where(se > 0, (q - p) / np.where(se > 0, se, 1), 0.0)
    log(f"  check at {runs:,}: numpy {cpu_seconds:.1f} s, torch {gpu_seconds:.1f} s, max |z| {np.abs(z).max():.2f}")
    return {"runs": runs, "cpu_seconds": cpu_seconds, "gpu_seconds": gpu_seconds, "z": z,
            "max_diff": float(np.abs(q - p).max())}


def analyse(stage, values, runs, checkpoints, log, dev=None, random_starts=20, batch=1_000_000):
    t0 = time.perf_counter()
    if dev is None:
        outcomes = simulate_runs(stage, values, runs)
    else:
        from . import swiss_gpu

        outcomes = swiss_gpu.simulate_many(stage, values, runs, dev, batch)
    sim_seconds = time.perf_counter() - t0
    log(f"  simulated {runs:,} stages in {sim_seconds:.1f} s")
    gen = np.random.default_rng(33)
    randoms = [gen.permutation(np.repeat(np.arange(4), [2, 2, 6, 6])) for _ in range(random_starts)]
    steps = []
    previous_b = None
    for n in checkpoints:
        sub = outcomes[:n]
        a = expected_picks(sub)
        starts = [a] + ([previous_b] if previous_b is not None else []) + randoms[:2]
        t1 = time.perf_counter()
        b = search(sub, starts)
        steps.append((n, a, b, time.perf_counter() - t1))
        previous_b = b
        log(f"  {n:>9,}: searched B in {steps[-1][3]:.1f} s")
    # Final B: the largest run, warm-started from every checkpoint's slips and 20 random slips.
    t1 = time.perf_counter()
    final_a = steps[-1][1]
    final_b = search(outcomes, [steps[-1][2], final_a, *[s[2] for s in steps], *[s[1] for s in steps], *randoms])
    search_seconds = time.perf_counter() - t1
    log(f"  final B search in {search_seconds:.1f} s")
    marginals = np.array([(outcomes == k).mean(axis=0) for k in range(3)])
    return {
        "outcomes": outcomes, "steps": steps, "A": final_a, "B": final_b, "marginals": marginals,
        "sim_seconds": sim_seconds, "search_seconds": search_seconds, "randoms": randoms,
    }


def summary(outcomes, roles):
    s = scores(outcomes, roles)
    return float(s.mean()), float((s >= 5).mean()), np.bincount(s, minlength=11)[:11] / len(s)


def git_hash():
    return subprocess.run(["git", "rev-parse", "HEAD"], capture_output=True, text=True, cwd=ROOT).stdout.strip()


def render(stage, forecasts, info, results, runs, made, head):
    n = runs
    lines = [
        "# PGL Major Singapore 2026 Stage 2 — Pick'em forecast",
        "",
        f"Made **{made}** at git `{head}` (the commit this file is added on top of), before Stage 2 begins "
        f"({T.iso(info['cutoff'])}, PandaScore tournament {STAGE2}). PandaScore listed every Stage 2 match as "
        f"`not_started` with no winner (latest listing change {info['pandascore_modified']}). No Stage 2 result is "
        f"used anywhere. Reproduce with `{info['command']}` from `research/ml/` (see README for the GPU "
        "environment); the script refuses to run if its saved Stage 2 listing shows a started match.",
        "",
        "Primary slip: **LightGBM on all features, strategy B** (highest simulated chance of at least five correct), "
        "the best row of the 28-stage replay in `PICKEM.md` (mean 4.57, 14/28 stages ≥5; the intervals of all "
        "models overlap, so this ranking is weak evidence).",
        "",
    ]
    order = (("lgbm all", "B"), ("lgbm all", "A"), ("Anchor", "A"), ("Anchor", "B"))
    lines += [
        "## Picks",
        "",
        "| Model | Strategy | 3–0 | 0–3 | Advance (3–1 / 3–2) | Expected correct | P(≥5) |",
        "|---|---|---|---|---|---:|---:|",
    ]
    for model, strategy in order:
        r = results[model]
        roles = r[strategy]
        mean, p5, _ = summary(r["outcomes"], roles)
        label = "**LightGBM all**" if (model, strategy) == order[0] else ("LightGBM all" if model == "lgbm all" else model)
        lines.append(f"| {label} | {strategy} | {' | '.join(slip(stage, roles))} | {mean:.3f} | {p5:.1%} |")
    lines += [
        "",
        f"Expected correct and P(≥5) are in-sample on the {n:,} simulated stages of that model "
        f"(Monte Carlo standard error of P(≥5) ≤ {0.5 / np.sqrt(n):.2%}). Each model is scored on its own "
        "simulation, so the numbers measure how confident the model is, not which slip is better.",
        "",
        "Cross-model view: each slip's expected correct / P(≥5) under the other model's simulation.",
        "",
        "| Slip | under Anchor | under LightGBM all |",
        "|---|---:|---:|",
    ]
    for model, strategy in order:
        roles = results[model][strategy]
        cells = []
        for judge in ("Anchor", "lgbm all"):
            mean, p5, _ = summary(results[judge]["outcomes"], roles)
            cells.append(f"{mean:.3f} / {p5:.1%}")
        lines.append(f"| {'LightGBM all' if model == 'lgbm all' else model} {strategy} | {' | '.join(cells)} |")

    lines += ["", "## Seeds and round 1", ""]
    lines.append(
        f"Seeds from the Liquipedia Stage 2 page (revision {info['wiki_revision']}): invited teams 1–8 by Valve "
        "global standing, Stage 1 qualifiers 9–16 by Stage 1 record and Buchholz. Round 1 pairs seed k with k+8, "
        "matching the PandaScore listing. Round 1, round 2 and the 1–1 round-3 matches are BO1; every match that "
        "can advance or eliminate a team is BO3."
    )
    lines += ["", "| Round 1 (BO1) | Anchor P(first wins) | LightGBM all P(first wins) |", "|---|---:|---:|"]
    a, b = np.triu_indices(16, 1)
    index = {(int(i), int(j)): k for k, (i, j) in enumerate(zip(a, b))}
    for i, j in stage.opening:
        k = index[(min(i, j), max(i, j))]
        cells = []
        for model in ("Anchor", "lgbm all"):
            p = forecasts[model][k]
            cells.append(f"{(p if i < j else 1 - p):.1%}")
        lines.append(f"| {i + 1}. {stage.teams[i]} vs {j + 1}. {stage.teams[j]} | {' | '.join(cells)} |")

    for model in ("lgbm all", "Anchor"):
        r = results[model]
        m = r["marginals"]
        label = "LightGBM all" if model == "lgbm all" else "Anchor"
        lines += [
            "",
            f"## Team probabilities — {label}",
            "",
            f"From {n:,} simulated stages. Advance = any 3–x finish (3–0, 3–1 or 3–2); the advance pick scores only on 3–1 or 3–2.",
            "",
            "| Seed | Team | P(3–0) | P(0–3) | P(advance) | P(3–1/3–2) | B pick | A pick |",
            "|---:|---|---:|---:|---:|---:|---|---|",
        ]
        rows = sorted(range(16), key=lambda t: -(m[0, t] + m[2, t]))
        for t in rows:
            picks = [ROLE_NAMES[x[t]] if x[t] != 3 else "—" for x in (r["B"], r["A"])]
            lines.append(
                f"| {t + 1} | {stage.teams[t]} | {m[0, t]:.1%} | {m[1, t]:.1%} | {m[0, t] + m[2, t]:.1%} | "
                f"{m[2, t]:.1%} | {picks[0]} | {picks[1]} |"
            )

    lines += [
        "",
        "## Convergence",
        "",
        f"Simulator: {info['simulator']}. Each checkpoint is a prefix of the same run "
        + ("(numpy, seed 33, chunks of 5,000)" if info["simulator"] == "numpy" else
           f"(torch, batches of {info['batch']:,}, batch b seeded 33·1,000,003 + b)")
        + ": A by exact assignment on the prefix, B by the replay's best-improving swap search on the prefix "
        "(started from A, the previous checkpoint's B and two seeded random slips). The final B search on the "
        f"full run restarts from every checkpoint's A and B and {len(results['Anchor']['randoms'])} seeded random "
        "slips. Changed = team/category selections that differ from the final slip of that strategy. Expected "
        "and P(≥5) are re-scored on the full run.",
        "",
    ]
    for model in ("lgbm all", "Anchor"):
        r = results[model]
        label = "LightGBM all" if model == "lgbm all" else "Anchor"
        lines += [
            f"### {label}",
            "",
            f"Simulation of {n:,} stages took {r['sim_seconds']:.1f} s; the final B search {r['search_seconds']:.0f} s.",
            "",
            "| Runs | Strategy | 3–0 | 0–3 | Advance | Changed vs final | Expected (full run) | P(≥5) (full run) | P(≥5) (prefix) |",
            "|---:|---|---|---|---|---:|---:|---:|---:|",
        ]
        for count, a_roles, b_roles, _ in r["steps"]:
            for strategy, roles in (("A", a_roles), ("B", b_roles)):
                final = r[strategy]
                changed = int(np.count_nonzero((final != 3) & (final != roles)))
                mean, p5, _ = summary(r["outcomes"], roles)
                _, p5_prefix, _ = summary(r["outcomes"][:count], roles)
                lines.append(
                    f"| {count:,} | {strategy} | {' | '.join(slip(stage, roles))} | {changed} | {mean:.4f} | "
                    f"{p5:.3%} | {p5_prefix:.3%} |"
                )
        lines.append(
            f"| {n:,} | B (final search) | {' | '.join(slip(stage, r['B']))} | 0 | "
            f"{summary(r['outcomes'], r['B'])[0]:.4f} | {summary(r['outcomes'], r['B'])[1]:.3%} | — |"
        )
        lines.append("")

    checks = [(m, results[m]["check"]) for m in ("lgbm all", "Anchor") if results[m]["check"]]
    if checks:
        lines += [
            "## GPU simulator check",
            "",
            "`kbml/swiss_gpu.py` runs the same Swiss procedure as the replay's numpy simulator with every run "
            "advancing round by round in tensors (Buchholz pairing with the rematch tables, BO1/BO3 by bucket). "
            "Fed the same uniform draws it returns identical outcomes (`tests/test_swiss_gpu.py`, CPU and CUDA). "
            "Here both simulate each model independently (numpy seed 33, torch seed 34) and every team's 3–0, "
            "0–3, 3–1/3–2 and other shares are compared (16 × 4 z-scores per model).",
            "",
            "| Model | Runs each | numpy | torch | Largest share gap | max abs z | abs z > 3 |",
            "|---|---:|---:|---:|---:|---:|---:|",
        ]
        for m, c in checks:
            lines.append(
                f"| {'LightGBM all' if m == 'lgbm all' else m} | {c['runs']:,} | {c['cpu_seconds']:.1f} s | "
                f"{c['gpu_seconds']:.1f} s | {c['max_diff']:.3%} | {np.abs(c['z']).max():.2f} | "
                f"{int((np.abs(c['z']) > 3).sum())}/64 |"
            )
        lines.append("")
    lines += ["## Score distribution", "", "| Slip | " + " | ".join(str(k) for k in range(11)) + " |",
              "|---|" + "---:|" * 11]
    for model, strategy in order:
        _, _, dist = summary(results[model]["outcomes"], results[model][strategy])
        lines.append(f"| {'LightGBM all' if model == 'lgbm all' else model} {strategy} | "
                     + " | ".join(f"{d:.1%}" for d in dist) + " |")

    ranks = info["ranks"]
    lines += [
        "",
        "## Inputs",
        "",
        f"- Training: {info['training']:,} non-qualifier results with a known finish before the cutoff: the research "
        f"archive (latest result {T.iso(info['latest_archive_result'])}) plus all {info['stage1_rows']} Singapore "
        "Stage 1 results from PandaScore tournament 22378 (finish times, winners and map scores). The archive "
        "snapshot holds nothing between its latest result and Stage 1, so any other matches played in that gap "
        "are missing.",
        f"- Stage 1 rows carry the saved pre-match market lines where the replay fixtures have them "
        f"({info['stage1_lines']}/33) and Valve ratings otherwise. They have no HLTV match pages, so their map-level "
        "and page-derived features are missing; they still feed series Elo, form, head-to-head, rest and venue "
        "history and the training set.",
        f"- LightGBM hyperparameters from the inner walk-forward over completed events: `{info['params']}`; "
        f"{info['features']} features (the `full` set).",
        "- Round 1 market lines: none. The saved OddsPapi fixture lists stop at Stage 1 and a new fixtures listing "
        "is a billable request, so the anchor is the Valve-rating probability alone (the 80/20 blend with no "
        f"market). Valve ratings cover {info['vrs_pairs']}/240 pair-format rows (standings of 2026-11-02); "
        "a pair without both ratings gets 50%.",
        "- THE ONE is the 1win roster renamed after the 2026-11-02 Valve standings (same five players; seed 4 "
        "on Liquipedia matches 1win's 1,598 points). Its rating, archive history (as 1WIN) and rank use 1win.",
        f"- HLTV rank badges at the cutoff known for {sum(r is not None for r in ranks)}/16 teams.",
        "- Stage 2 pages, lineups and match-specific panels do not exist before the stage and take the feature "
        "groups' missing-data behaviour, as in the replay.",
        "- Pairings after round 1 follow the sidecar Buchholz rule with Liquipedia's initial seeds, the procedure "
        "that reproduced all 12 Major stages with published seeds in the replay.",
        "- B is a local search (best-improving two-team role swaps) from A, every checkpoint's A and B and "
        f"{len(results['Anchor']['randoms'])} seeded random slips; a better slip it did not reach can exist.",
        "- Raw inputs saved under `forecasts/data/`: PandaScore `GET /tournaments/22378/matches?per_page=100` and "
        "`/tournaments/22473/matches?per_page=100`, and the Liquipedia revision of `PGL/2026/Singapore/Stage 2`, "
        "all fetched 2026-11-27.",
        "",
        "## Result",
        "",
        "To be scored after Stage 2 ends (2026-12-01).",
        "",
    ]
    return "\n".join(lines)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--runs", type=int, default=1_000_000)
    parser.add_argument("--checkpoints", default="100000,200000,500000,1000000")
    parser.add_argument("--out", type=Path, default=OUT)
    parser.add_argument("--device", default="auto",
                        help="numpy (the replay's simulator), cuda or cpu (torch), auto = cuda if available else numpy")
    parser.add_argument("--batch", type=int, default=1_000_000, help="torch runs per batch (1M fits a 6 GB GPU)")
    parser.add_argument("--check-runs", type=int, default=1_000_000,
                        help="runs of the numpy-vs-torch check per model (0 skips it)")
    parser.add_argument("--random-starts", type=int, default=20)
    args = parser.parse_args()
    checkpoints = sorted({int(c) for c in args.checkpoints.split(",") if int(c) <= args.runs} | {args.runs})
    made = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    head = git_hash()
    t0 = time.perf_counter()
    stage, forecasts, info = build()
    print(f"forecasts built in {time.perf_counter() - t0:.0f} s; training {info['training']:,}; params {info['params']}",
          flush=True)
    dev = None
    if args.device != "numpy":
        try:
            from . import swiss_gpu

            dev = swiss_gpu.device(args.device)
            if args.device == "auto" and dev.type != "cuda":
                dev = None
        except ImportError:
            if args.device != "auto":
                raise
    log = lambda s: print(s, flush=True)  # noqa: E731
    print(f"simulator: {'numpy' if dev is None else f'torch on {dev}'}", flush=True)
    results = {}
    for model in ("lgbm all", "Anchor"):
        print(model, flush=True)
        check = check_gpu(stage, forecasts[model], args.check_runs, dev, log, args.batch) if dev is not None and args.check_runs else None
        results[model] = analyse(stage, forecasts[model], args.runs, checkpoints, log, dev, args.random_starts, args.batch)
        results[model]["check"] = check
    info["simulator"] = "numpy" if dev is None else f"torch {__import__('torch').__version__} on {dev}" + (
        f" ({__import__('torch').cuda.get_device_name(dev)})" if dev.type == "cuda" else "")
    info["batch"] = args.batch
    info["command"] = "uv run python -m kbml.pickem_forecast " + " ".join(sys.argv[1:])
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_bytes(render(stage, forecasts, info, results, args.runs, made, head).encode("utf-8") + b"\n")
    print(f"wrote {args.out} ({time.perf_counter() - t0:.0f} s total)")


if __name__ == "__main__":
    main()
