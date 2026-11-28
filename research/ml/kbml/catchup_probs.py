"""`uv run python -m kbml.catchup_probs`: walk-forward lgbm-ext probabilities of the 2087 scored
Matches (phase 3 protocol, as `forecaster_report.reliability`), cached for the catch-up study."""
from __future__ import annotations

import pandas as pd

from . import evaluate, features, paths
from .forecaster import EXT, TRAIN_BLOCKS

CACHE = paths.CACHE / "catchup-probs.parquet"


def compute(log=print) -> pd.DataFrame:
    from .run import with_hltv
    from .table import load_table
    m, mp, _ = load_table(EXT, log=lambda *a: None)
    m, h = with_hltv(m, EXT)
    X, kinds = features.compute(m, mp, features.resolve("p3_all"), h)
    cols = {c: kinds[c] for g in features.SETS["p3_all"] for c in features.REGISTRY[g].columns}
    pr = evaluate.walk_forward(m, X, cols, "lgbm", train_blocks=TRAIN_BLOCKS, log=log)
    return pr


def load(log=print) -> dict:
    """{match_id: lgbm-ext p(team A wins)}."""
    if not CACHE.exists():
        pr = compute(log)
        pr[["match_id", "event_id", "variant", "anchor", "team_a_won"]].to_parquet(CACHE)
    pr = pd.read_parquet(CACHE)
    return dict(zip(pr["match_id"].astype(int), pr["variant"].astype(float)))


if __name__ == "__main__":
    p = load()
    print(len(p))
