"""`uv run python -m kbml.spa_inputs [--phases 2,3]`: per-Match predictions of every configuration
of phases 2 and 3 (phase 4's Match-level forecasters are phase 3 configurations), for the Hansen
SPA test of phase 5 (`kbml.spa`).

Re-runs each walk-forward exactly as `kbml.report` (phase 2, snapshot archive: every model on
`full`, `baseline`, `baseline+<hltv group>`, and the `map_model` rows) and `kbml.report3` (phase 3,
extended archive: model × set × training rows) define it, and stores one frame per phase under the
cache's `spa/` folder: match_id, event_id, team_a_won, anchor, has_market, and one column per
configuration. Nothing is scored here.
"""
from __future__ import annotations

import argparse
import sys
import time
from pathlib import Path

import pandas as pd

from . import evaluate, features, paths
from .report3 import DATA, EXT, SETS
from .run import with_hltv
from .table import cache_dir, load_table


def _frame(rows: dict[str, pd.DataFrame]) -> pd.DataFrame:
    first = next(iter(rows.values()))
    out = first[["match_id", "event_id", "team_a_won", "anchor", "has_market"]].copy()
    for name, pred in rows.items():
        assert (pred["match_id"].to_numpy() == out["match_id"].to_numpy()).all()
        out[name] = pred["variant"].to_numpy()
    return out


def phase2(models, log=print) -> pd.DataFrame:
    archive = paths.ARCHIVE
    m, maps, _ = load_table(archive, log=log)
    m, hltv = with_hltv(m, archive)
    groups = features.resolve("full,map_model")
    X, kinds = features.compute(m, maps, groups, hltv)
    by_group = {g: list(features.REGISTRY[g].columns) for g in groups}
    configs = []
    for model in models:
        configs.append((model, "full", features.SETS["full"]))
        configs.append((model, "baseline", features.SETS["baseline"]))
        for g in features.SETS["hltv"]:
            configs.append((model, f"baseline+{g}", features.SETS["baseline"] + [g]))
        if model != "mlp":
            configs.append((model, "baseline+map_model", features.SETS["baseline"] + ["map_model"]))
            configs.append((model, "full+map_model", features.SETS["full"] + ["map_model"]))
    rows = {}
    for model, label, gs in configs:
        t = time.time()
        cols = {c: kinds[c] for g in gs for c in by_group[g]}
        rows[f"p2 {model} {label}"] = evaluate.walk_forward(m, X, cols, model, log=lambda *_: None)
        log(f"p2 {model} {label} [{time.time() - t:.0f}s]")
    return _frame(rows)


def phase3(models, log=print) -> pd.DataFrame:
    m, maps, _ = load_table(EXT, log=log)
    m, hltv = with_hltv(m, EXT)
    groups = features.resolve("p3_all")
    X, kinds = features.compute(m, maps, groups, hltv)
    X.to_parquet(cache_dir(EXT) / "spa" / "p3_all_X.parquet")
    by_group = {g: list(features.REGISTRY[g].columns) for g in groups}
    data = {label: (blocks, w) for label, blocks, w in DATA}
    rows = {}
    for model in models:
        for fs in SETS:
            for label in data:
                t = time.time()
                cols = {c: kinds[c] for g in features.SETS[fs] for c in by_group[g]}
                blocks, weights = data[label]
                rows[f"p3 {model} {fs} / {label}"] = evaluate.walk_forward(
                    m, X, cols, model, train_blocks=blocks, block_weights=weights, log=lambda *_: None)
                log(f"p3 {model} {fs} / {label} [{time.time() - t:.0f}s]")
    return _frame(rows)


def main(argv=None):
    ap = argparse.ArgumentParser(prog="kbml.spa_inputs")
    ap.add_argument("--phases", default="2,3")
    ap.add_argument("--models", default="logreg,lgbm,mlp")
    args = ap.parse_args(argv)
    try:
        sys.stdout.reconfigure(encoding="utf-8", line_buffering=True)
    except AttributeError:
        pass
    out = cache_dir(EXT) / "spa"
    out.mkdir(parents=True, exist_ok=True)
    models = args.models.split(",")
    for ph in args.phases.split(","):
        frame = phase2(models) if ph == "2" else phase3(models)
        frame.to_parquet(out / f"phase{ph}.parquet", index=False)
        print(f"phase {ph}: {frame.shape[1] - 5} configurations, {len(frame)} Matches -> {out}")


if __name__ == "__main__":
    main()
