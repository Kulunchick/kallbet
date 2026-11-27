"""The pairwise forecaster sees only results known strictly before the cutoff it is asked for."""
import numpy as np
import pandas as pd

from kbml.forecaster import PairwiseForecaster
from kbml import timeutil as T

CONFIGS = {"logreg-ext": ("logreg", "p3_rw"), "lgbm-ext": ("lgbm", "p3_all")}


def _frames(m, mp):
    m = m.copy()
    m["block"] = "scope"
    for c in ("hltv_rank_a", "hltv_rank_b", "hltv_rank28_a", "hltv_rank28_b", "hltv_rank84_a", "hltv_rank84_b"):
        m[c] = pd.array([None] * len(m), dtype="Int64")
    return m, mp, ({}, None), []


def _requests(cutoff):
    teams = ["alpha", "bravo", "charlie", "delta"]
    rows = [{"team_a_key": a, "team_b_key": b, "format": f, "cutoff": cutoff, "tier": "s", "anchor_p": 0.5}
            for f in ("bo1", "bo3") for a in teams for b in teams if a != b]
    return pd.DataFrame(rows)


def test_results_after_the_cutoff_change_no_forecast(table):
    m, mp = table
    cutoff = int(m["cutoff"].quantile(0.7))
    req = _requests(cutoff)
    fc = PairwiseForecaster(None, log=lambda *a: None, frames=_frames(m, mp))
    base = fc.predict(req, CONFIGS)
    later = m["known_finish"].astype("float") >= cutoff
    m2 = m.copy()
    m2.loc[later, "team_a_won"] = ~m2.loc[later, "team_a_won"].astype(bool)
    mp2 = mp.copy()
    late_ids = set(m.loc[later, "match_id"])
    mp2.loc[mp2.match_id.isin(late_ids), "a_won"] = ~mp2.loc[mp2.match_id.isin(late_ids), "a_won"]
    after = PairwiseForecaster(None, log=lambda *a: None, frames=_frames(m2, mp2)).predict(req, CONFIGS)
    for k in base:
        assert np.array_equal(base[k], after[k]), k
    # and an earlier result does move them
    m3 = m.copy()
    early = (m["known_finish"].astype("float") < cutoff).to_numpy()
    m3.loc[early, "team_a_won"] = ~m3.loc[early, "team_a_won"].astype(bool)
    moved = PairwiseForecaster(None, log=lambda *a: None, frames=_frames(m3, mp)).predict(req, CONFIGS)
    assert not np.allclose(base["logreg-ext"], moved["logreg-ext"])


def test_forecasts_are_symmetric_in_the_two_teams(table):
    m, mp = table
    req = _requests(int(m["cutoff"].quantile(0.8)))
    p = PairwiseForecaster(None, log=lambda *a: None, frames=_frames(m, mp)).predict(req, CONFIGS)
    key = {(r.team_a_key, r.team_b_key, r.format): i for i, r in enumerate(req.itertuples())}
    for (a, b, f), i in key.items():
        for k in p:
            assert abs(p[k][i] + p[k][key[b, a, f]] - 1) < 1e-9
