"""Singapore 2026 Stage 2 Pick'em from the ext forecasters (logreg-ext, lgbm-ext), from pre-Stage
inputs only. NOT pre-registered (made after the pre-registered forecast and the phase 4 replay).

`<gpu python> -m kbml.pickem_forecast_ext --runs 5000000 --random-starts 8` → forecasts/singapore-2026-stage2-ext.md

Inputs: exactly those of `kbml.pickem_forecast` (the saved PandaScore listings of Stage 1 and
Stage 2 and the Liquipedia Stage 2 revision under `forecasts/data/`, all fetched 2026-11-27, and the
archives), nothing fetched. The Stage, its seeds, round-1 pairs and formats and the anchor come from
`pickem_forecast.build()`, which refuses to run if the saved Stage 2 listing shows a started Match.
The ext history is the extended archive (max known finish 2026-11-22) plus Singapore Stage 1 from
the saved listing; every training row must finish strictly before the Stage 2 cutoff (asserted).
"""
from __future__ import annotations

import argparse
import sys
import time
from datetime import datetime, timezone
from itertools import combinations
from pathlib import Path

import numpy as np
import pandas as pd

from . import archive, hltv_data
from . import timeutil as T
from .anchor import Markets, Standings, anchor
from .forecaster import EXT, PairwiseForecaster
from .pickem_data import FIXTURES, rank_history
from .pickem_forecast import (ROOT, STAGE1, analyse, build, git_hash, history_row, key, rating_name, read,
                              slip, summary)
from .run import with_hltv
from .table import load_table

OUT = ROOT / "forecasts" / "singapore-2026-stage2-ext.md"
MODELS = ("logreg-ext", "lgbm-ext", "Anchor")


def ext_forecasts(stage, log=print):
    """240 probabilities per ext forecaster (BO1 pairs then BO3 pairs, in `stage.teams` order)."""
    cutoff = stage.cutoff
    history, maps, _ = load_table(EXT, log=lambda *a: None)
    history, hltv = with_hltv(history, EXT)
    ranks = rank_history(history, hltv)
    fixture = archive.load(FIXTURES, with_hltv=False)
    standings, markets = Standings(fixture.standings), Markets(fixture)
    s1_rows = read("panda-22378-matches.json.gz")
    stage1 = archive.stage_from_pandascore(STAGE1, s1_rows)
    lan1 = {"online": False, "offline": True}.get(s1_rows[0]["tournament"]["type"])
    s1 = pd.DataFrame([history_row(m, stage1, standings, markets, ranks, lan1) for m in stage1.matches])
    assert not set(s1.match_id) & set(history.match_id)
    s1["block"] = "scope"
    rankings = hltv_data.weekly_rankings(EXT)
    s1 = hltv_data.weekly_columns(s1, {}, rankings)
    hist = pd.concat([history, s1], ignore_index=True)
    for c in ("round", "maps_a", "maps_b", "vrs_points_a", "vrs_points_b", "hltv_match_id", "event_ended",
              "known_finish", "market_observed", "vrs_observed", "rw_rank_a", "rw_rank_b", "rw_points_a",
              "rw_points_b", "rw_size", "rw_known"):
        hist[c] = hist[c].astype("Int64")
    assert (hist.known_finish.dropna() < cutoff).all(), "a training result finishes after the Stage 2 cutoff"
    assert all(p["known"] < cutoff for p in rankings), "a weekly ranking is not known before the cutoff"
    req = []
    for fmt in ("bo1", "bo3"):
        for a, b in combinations(stage.teams, 2):
            rating = standings.estimate(rating_name(a), rating_name(b), fmt, cutoff)
            vr = rating[1] if rating else None
            blended = anchor(None, vr)
            row = {"team_a_key": key(a), "team_b_key": key(b), "format": fmt, "cutoff": cutoff, "tier": "s",
                   "hltv_is_lan": stage.lan, "market_p": None, "vrs_series_p": vr,
                   "vrs_map_p": rating[0] if rating else None, "anchor_p": 0.5 if blended is None else blended,
                   "vrs_points_a": rating[3] if rating else None, "vrs_points_b": rating[4] if rating else None}
            for suffix, t in (("a", a), ("b", b)):
                for col, days in (("hltv_rank", 0), ("hltv_rank28", 28), ("hltv_rank84", 84)):
                    row[f"{col}_{suffix}"] = ranks.at(key(t), cutoff - days * T.DAY)
            req.append(row)
    fc = PairwiseForecaster(None, log=log, frames=(hist, maps, hltv, rankings))
    preds = fc.predict(pd.DataFrame(req))
    trained = int(((hist.known_finish.astype("float") < cutoff) & ~hist.is_qualifier
                   & hist.block.isin({"scope", "sa_old", "b", "c"})).sum())
    return preds, {"training": trained, "stage1_rows": len(s1), "latest_ext_result": int(history.known_finish.max()),
                   "ranking_page": max(p["date"] for p in rankings)}


def render(stage, results, info, runs, made, head, command):
    names = stage.teams
    lines = [
        "# PGL Major Singapore 2026 Stage 2 — ext forecasters' slips (NOT pre-registered)",
        "",
        f"**Made {made}** (machine clock, UTC; Stage 2 starts 2026-11-28T04:00:00Z) at git `{head}`, from **pre-Stage "
        "inputs only**: the PandaScore listings and the Liquipedia revision saved under `forecasts/data/` on "
        "2026-11-27 (every Stage 2 Match `not_started`), the snapshot and the extended archive (latest result "
        f"{T.iso(info['latest_ext_result'])}; latest weekly HLTV ranking {info['ranking_page']}). Nothing was fetched "
        "and no Stage 2 result enters. It is **not a pre-registered forecast**: it was made after the pre-registered "
        "forecast was fixed and after the 28-Stage replay results of these forecasters (FORECASTER.md) were seen, on "
        "request, so it must not be scored as one. The pre-registered forecast is `singapore-2026-stage2.md` (unchanged).",
        "",
        f"Reproduce: `{command}` from `research/ml/` (GPU environment, see README).",
        "",
        "## Slips",
        "",
        "| Model | Strategy | 3–0 | 0–3 | Advance (3–1 / 3–2) | Expected correct | P(≥5) |",
        "|---|---|---|---|---|---:|---:|",
    ]
    for model in MODELS:
        for strat in ("B", "A"):
            roles = results[model][strat]
            e, p5, _ = summary(results[model]["outcomes"], roles)
            bold = "**" if strat == "B" and model != "Anchor" else ""
            lines.append(f"| {bold}{model}{bold} | {strat} | " + " | ".join(slip(stage, roles)) + f" | {e:.3f} | {p5:.1%} |")
    lines += ["", f"Expected correct and P(≥5) are under the slip's own model, in-sample on its {runs:,} simulated "
              "Stages.", "", "## Cross table: each slip under each model", "",
              "| Slip | " + " | ".join(f"under {m}" for m in MODELS) + " |", "|---|" + "---:|" * len(MODELS)]
    for model in MODELS:
        for strat in ("B", "A"):
            roles = results[model][strat]
            cells = []
            for other in MODELS:
                e, p5, _ = summary(results[other]["outcomes"], roles)
                cells.append(f"{e:.3f} / {p5:.1%}")
            lines.append(f"| {model} {strat} | " + " | ".join(cells) + " |")
    lines += ["", "Cells: expected correct / P(≥5).", "", "## Per team under logreg-ext", "",
              "| Seed | Team | P(3–0) | P(0–3) | P(3–1 or 3–2) | lgbm-ext P(3–0) | lgbm-ext P(0–3) | lgbm-ext P(3–1/3–2) |",
              "|---:|---|---:|---:|---:|---:|---:|---:|"]
    ml, mg = results["logreg-ext"]["marginals"], results["lgbm-ext"]["marginals"]
    for i, t in enumerate(names):
        lines.append(f"| {i + 1} | {t} | {ml[0, i]:.1%} | {ml[1, i]:.1%} | {ml[2, i]:.1%} | {mg[0, i]:.1%} | "
                     f"{mg[1, i]:.1%} | {mg[2, i]:.1%} |")
    lines += ["", "## Method", "",
              "- Forecasters: `kbml/forecaster.py` (phase 4, FORECASTER.md). `logreg-ext`: logreg on `p3_rw` (baseline + "
              "weekly HLTV ranking + tier). `lgbm-ext`: LightGBM on `p3_all`. Each fitted once on every non-qualifier "
              f"Match known before the cutoff: {info['training']:,} (in-scope, S/A 2023-24, tier B, tier C, Singapore "
              f"Stage 1's {info['stage1_rows']} from the saved listing); hyperparameters by inner walk-forward on "
              "completed earlier Events. The 120 pairings in BO1 and BO3 are hypothetical: match-page features take the "
              "missing-data path. No Stage 2 market line exists in the inputs; the anchor is the Valve rating alone.",
              "- Simulation and search: `pickem_forecast.analyse` (GPU Swiss simulator, seed 33; strategy A = exact "
              "expected-points optimum; B = local search for P(≥5) from A, the checkpoints' slips and 8 seeded random "
              "slips), the same code as the pre-registered forecast.", "",
              "## Result", "", "To be scored after Stage 2 ends.", ""]
    return "\n".join(lines)


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--runs", type=int, default=5_000_000)
    ap.add_argument("--checkpoints", default="200000,1000000")
    ap.add_argument("--random-starts", type=int, default=8)
    ap.add_argument("--batch", type=int, default=1_000_000)
    ap.add_argument("--out", type=Path, default=OUT)
    args = ap.parse_args()
    try:
        sys.stdout.reconfigure(encoding="utf-8", line_buffering=True)
    except AttributeError:
        pass
    made = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    head = git_hash()
    stage, forecasts, info = build()
    preds, extra = ext_forecasts(stage)
    forecasts = {"Anchor": forecasts["Anchor"], **preds}
    from . import swiss_gpu
    dev = swiss_gpu.device("auto")
    print(f"simulator on {dev}", flush=True)
    checkpoints = sorted({int(c) for c in args.checkpoints.split(",") if int(c) <= args.runs} | {args.runs})
    results = {}
    for model in MODELS:
        print(model, flush=True)
        results[model] = analyse(stage, forecasts[model], args.runs, checkpoints, print, dev, args.random_starts, args.batch)
    command = "<gpu python> -m kbml.pickem_forecast_ext " + " ".join(sys.argv[1:])
    if args.out.resolve() == (ROOT / "forecasts" / "singapore-2026-stage2.md").resolve():
        raise SystemExit("refusing to overwrite the pre-registered forecast")
    args.out.write_bytes(render(stage, results, extra, args.runs, made, head, command).encode("utf-8") + b"\n")
    print(f"wrote {args.out}")


if __name__ == "__main__":
    main()
