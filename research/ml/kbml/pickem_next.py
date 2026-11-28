"""Live: model picks for the next paired, not-yet-finished Stage 2 Matches (live tracking only;
never part of the Holdout evaluation, which scores only the pre-registered forecast).

`<gpu or cpu python> -m kbml.pickem_next` appends a section to forecasts/singapore-2026-stage2-live.md.

(a) frozen: the pre-Stage forecasts (cutoff 2026-11-28T04:00Z), exactly the arrays of the forecast files.
(b) live: the same model configurations refitted on every result known at each Match's cutoff
    (Scheduled Start − 15 min), Stage 1 and the finished Stage 2 Matches of the saved live listing
    included; no HLTV page of a Stage 2 Match is in the archive, so their page features are missing
    and they enter through results (Elo, form, head-to-head, rest).
Models: lgbm-ext, logreg-ext, their equal-weight logit ensemble, lgbm all, anchor (Valve rating as of
the cutoff; no market line is saved for Stage 2).
"""
from __future__ import annotations

import gzip
import json
import math
import sys
from datetime import datetime, timezone
from itertools import combinations

import numpy as np
import pandas as pd

from . import archive, hltv_data, paths
from . import timeutil as T
from .anchor import Markets, Standings, anchor, logit
from .forecaster import EXT, PairwiseForecaster
from .pickem_data import FIXTURES, rank_history
from .pickem_forecast import STAGE1, STAGE2, build, git_hash, history_row, key, rating_name, read
from .pickem_forecast_ext import ext_forecasts
from .pickem_live import OUT, latest_live, results_from, simulator_pairing
from .run import with_hltv
from .table import load_table

INT_COLS = ("round", "maps_a", "maps_b", "vrs_points_a", "vrs_points_b", "hltv_match_id", "event_ended",
            "known_finish", "market_observed", "vrs_observed")


def sigmoid(z):
    return 1.0 / (1.0 + math.exp(-z))


def ensemble(p, q):
    return sigmoid(0.5 * logit(p) + 0.5 * logit(q))


def stage_rows(rows, stage_id, standings, markets, ranks):
    st = archive.stage_from_pandascore(stage_id, rows)
    lan = {"online": False, "offline": True}.get(rows[0]["tournament"]["type"])
    return pd.DataFrame([history_row(m, st, standings, markets, ranks, lan) for m in st.matches])


def live_history(base, s_rows, live_rows, standings, markets, ranks, rankings):
    extra = pd.concat([stage_rows(s_rows, STAGE1, standings, markets, ranks),
                       stage_rows(live_rows, STAGE2, standings, markets, ranks)], ignore_index=True)
    assert not set(extra.match_id) & set(base.match_id)
    extra["block"] = "scope"
    extra = hltv_data.weekly_columns(extra, {}, rankings)
    h = pd.concat([base, extra], ignore_index=True)
    for c in INT_COLS + tuple(c for c in h.columns if c.startswith("rw_")):
        if c in h:
            h[c] = h[c].astype("Int64")
    return h


def requests(pairs, standings, ranks):
    out = []
    for p in pairs:
        rating = standings.estimate(rating_name(p["a"]), rating_name(p["b"]), p["format"], p["cutoff"])
        vr = rating[1] if rating else None
        blended = anchor(None, vr)
        row = {"team_a_key": key(p["a"]), "team_b_key": key(p["b"]), "format": p["format"], "cutoff": p["cutoff"],
               "tier": "s", "hltv_is_lan": True, "market_p": None, "vrs_series_p": vr,
               "vrs_map_p": rating[0] if rating else None, "anchor_p": 0.5 if blended is None else blended,
               "vrs_points_a": rating[3] if rating else None, "vrs_points_b": rating[4] if rating else None}
        for suffix, t in (("a", p["a"]), ("b", p["b"])):
            for col, days in (("hltv_rank", 0), ("hltv_rank28", 28), ("hltv_rank84", 84)):
                row[f"{col}_{suffix}"] = ranks.at(key(t), p["cutoff"] - days * T.DAY)
        out.append(row)
    return pd.DataFrame(out)


def main():
    try:
        sys.stdout.reconfigure(encoding="utf-8", line_buffering=True)
    except AttributeError:
        pass
    live = latest_live()
    live_rows = json.loads(gzip.decompress(live.read_bytes()))
    stage, forecasts, _ = build()
    frozen_ext, _ = ext_forecasts(stage, log=lambda *a: None)
    names = stage.teams
    res = results_from(live_rows, names)
    # The next Matches: paired, not finished.
    pending = [x for x in res if x[3] is None]
    by_id = {r["id"]: r for r in live_rows}
    nxt = min(x[0] for x in pending)
    note = ""
    listed = [x for x in pending if x[0] == nxt]
    sim = simulator_pairing(stage, res, nxt)
    assert {frozenset((a, b)) for _, a, b, *_ in listed} == sim, "listed pairing differs from the simulator's"
    note = f"Round {nxt} is listed by PandaScore; its pairing equals the simulator's Buchholz pairing from the actual results."
    pairs = []
    for rnd, a, b, _, status, mid in listed:
        r = by_id[mid]
        fmt = f"bo{r['number_of_games']}"
        sched = T.parse(r["scheduled_at"])
        pairs.append({"round": rnd, "a": names[a], "b": names[b], "ia": a, "ib": b, "format": fmt, "scheduled": sched,
                      "cutoff": min(sched - T.HARD_LOCK_LEAD, T.parse(r["begin_at"]) or sched), "status": status})
    # (a) frozen arrays: BO1 pairs then BO3 pairs, triu order of `names`
    tri = {pair: k for k, pair in enumerate(combinations(range(16), 2))}

    def frozen(values, p):
        i, j = p["ia"], p["ib"]
        k = tri[(min(i, j), max(i, j))] + (0 if p["format"] == "bo1" else 120)
        q = float(values[k])
        return q if i < j else 1.0 - q

    fz = {"lgbm-ext": [frozen(frozen_ext["lgbm-ext"], p) for p in pairs],
          "logreg-ext": [frozen(frozen_ext["logreg-ext"], p) for p in pairs],
          "lgbm all": [frozen(forecasts["lgbm all"], p) for p in pairs],
          "Anchor": [frozen(forecasts["Anchor"], p) for p in pairs]}
    fz["ensemble"] = [ensemble(x, y) for x, y in zip(fz["lgbm-ext"], fz["logreg-ext"])]
    # (b) live refits at each Match's cutoff
    fixture = archive.load(FIXTURES, with_hltv=False)
    standings, markets = Standings(fixture.standings), Markets(fixture)
    s1_rows = read("panda-22378-matches.json.gz")
    ext_base, ext_maps, _ = load_table(EXT, log=lambda *a: None)
    ext_base, ext_hltv = with_hltv(ext_base, EXT)
    ext_ranks = rank_history(ext_base, ext_hltv)
    rankings = hltv_data.weekly_rankings(EXT)
    eh = live_history(ext_base, s1_rows, live_rows, standings, markets, ext_ranks, rankings)
    req = requests(pairs, standings, ext_ranks)
    lv = PairwiseForecaster(None, log=print, frames=(eh, ext_maps, ext_hltv, rankings)).predict(req)
    snap, snap_maps, _ = load_table(paths.ARCHIVE, log=lambda *a: None)
    snap, snap_hltv = with_hltv(snap, paths.ARCHIVE)
    snap_ranks = rank_history(snap, snap_hltv)
    sh = live_history(snap, s1_rows, live_rows, standings, markets, snap_ranks, [])
    lv_all = PairwiseForecaster(None, log=print, frames=(sh, snap_maps, snap_hltv, [])).predict(
        requests(pairs, standings, snap_ranks), {"lgbm all": ("lgbm", "full")})
    live_p = {"lgbm-ext": list(lv["lgbm-ext"]), "logreg-ext": list(lv["logreg-ext"]), "lgbm all": list(lv_all["lgbm all"]),
              "Anchor": list(req["anchor_p"].astype(float))}
    live_p["ensemble"] = [ensemble(x, y) for x, y in zip(live_p["lgbm-ext"], live_p["logreg-ext"])]
    trained = int(((eh.known_finish.astype("float") < min(p["cutoff"] for p in pairs)) & ~eh.is_qualifier).sum())
    text = render(pairs, fz, live_p, note, live, trained)
    cur = OUT.read_text(encoding="utf-8")
    marker = "\n## Next Matches"
    cur = cur.split(marker)[0].rstrip() + "\n"
    OUT.write_text(cur + text, encoding="utf-8")
    print(text)


ORDER = ("lgbm-ext", "logreg-ext", "ensemble", "lgbm all", "Anchor")


def render(pairs, fz, lv, note, live, trained):
    made = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    lines = ["", f"## Next Matches: model picks (live, made {made})", "",
             f"Listing `forecasts/data/{live.name}`. {note} P = probability that the first-named team wins the "
             "series. Pick = the team with P ≥ 50%. Frozen = the pre-Stage models of the forecast (cutoff "
             "2026-11-28T04:00Z); live = the same configurations refitted on every result known at the Match's cutoff "
             f"(Scheduled Start − 15 min), Stage 1 and Stage 2 rounds so far included ({trained:,} training Matches for "
             "the ext models). Ensemble = equal-weight average of lgbm-ext and logreg-ext in logit space. The anchor is "
             "the Valve rating alone (no Stage 2 market line saved). Live tracking only, not the Holdout evaluation.", ""]
    for label, src in (("Frozen pre-Stage models", fz), ("Live models refitted with Stage 2 results so far", lv)):
        lines += [f"### {label}", "",
                  "| Time (UTC) | Fmt | Match | " + " | ".join(ORDER) + " | Picks | Note |",
                  "|---|---|---|" + "---:|" * len(ORDER) + "---|---|"]
        for k, p in enumerate(pairs):
            ps = [src[m][k] for m in ORDER]
            picks = [p["a"] if x >= 0.5 else p["b"] for x in ps]
            models_pick = set(picks[:4])
            notes = []
            if len(models_pick) > 1:
                notes.append("models disagree")
            if any(x != picks[4] for x in picks[:4]):
                notes.append("differs from anchor")
            consensus = picks[2]
            lines.append(f"| {T.iso(p['scheduled'])[11:16]} | {p['format'].upper()} | {p['a']} vs {p['b']} | "
                         + " | ".join(f"{x:.0%}" for x in ps) + f" | ensemble: **{consensus}** | {'; '.join(notes) or '—'} |")
        lines.append("")
    return "\n".join(lines)


if __name__ == "__main__":
    main()
