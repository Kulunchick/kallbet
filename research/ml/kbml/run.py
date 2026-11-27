"""`uv run python -m kbml.run --model logreg|lgbm|mlp[,..] --features baseline`

Prints, per model, the walk-forward result against the anchor on the same Matches.
"""
from __future__ import annotations

import argparse
import json
import sys
import time
from pathlib import Path

from . import evaluate, features, paths
from .table import cache_dir, load_table


def with_hltv(matches, archive, rebuild=False):
    """The Match table with the HLTV rank columns, and (pre, records) for the HLTV groups."""
    from . import hltv_data
    m, pre, records, _ = hltv_data.load(matches, cache_dir(archive), rebuild=rebuild, archive_dir=archive)
    return m, (pre, records)


def main(argv=None):
    ap = argparse.ArgumentParser(prog="kbml.run")
    ap.add_argument("--model", default="logreg", help="logreg, lgbm, mlp, or a comma list")
    ap.add_argument("--features", default="baseline",
                    help=f"comma list of groups or sets; groups {sorted(features.REGISTRY)}, sets {sorted(features.SETS)}")
    ap.add_argument("--archive", type=Path, default=paths.ARCHIVE)
    ap.add_argument("--rebuild", action="store_true", help="re-read the archive instead of the parquet cache")
    ap.add_argument("--no-train-extra", action="store_true", help="fit only on in-scope rows")
    ap.add_argument("--min-train", type=int, default=150)
    ap.add_argument("--seed", type=int, default=33)
    ap.add_argument("--resamples", type=int, default=2000)
    ap.add_argument("--save", action="store_true", help="write per-Match predictions and the summary to the cache")
    args = ap.parse_args(argv)
    try:
        sys.stdout.reconfigure(encoding="utf-8")
    except AttributeError:
        pass

    t0 = time.time()
    matches, maps, meta = load_table(args.archive, rebuild=args.rebuild)
    groups = features.resolve(args.features)
    matches, hltv = with_hltv(matches, args.archive, args.rebuild)
    X, kinds = features.compute(matches, maps, groups, hltv)
    print(f"{len(matches)} Matches ({int(matches.in_scope.sum())} in scope), {len(kinds)} features from {groups} "
          f"[{time.time() - t0:.0f}s]")
    results = {}
    for name in args.model.split(","):
        t1 = time.time()
        pred = evaluate.walk_forward(matches, X, kinds, name, train_extra=not args.no_train_extra,
                                     min_train=args.min_train, seed=args.seed)
        results[name] = evaluate.summarize(pred, args.resamples, 0.9, args.seed)
        print(f"{name}: {len(pred)} Matches, {pred.event_id.nunique()} Events, "
              f"{int(pred.fallback.sum())} on the anchor (too little training) [{time.time() - t1:.0f}s]")
        if args.save:
            out = cache_dir(args.archive) / "runs"
            out.mkdir(parents=True, exist_ok=True)
            stem = f"{name}-{args.features.replace(',', '+')}"
            pred.to_parquet(out / f"{stem}.parquet", index=False)
            (out / f"{stem}.json").write_text(json.dumps(results[name], indent=1))
    print()
    print(evaluate.table(results))
    print(f"\nΔ = model − anchor on the same Matches; 90% Event-bootstrap intervals, {args.resamples} resamples, "
          f"seed {args.seed}. Total {time.time() - t0:.0f}s.")


if __name__ == "__main__":
    main()
