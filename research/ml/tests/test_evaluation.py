"""Walk-forward rules, the bootstrap port and the ported helpers."""
import numpy as np
import pandas as pd
import pytest

from kbml import evaluate, stats
from kbml import features as F
from kbml import timeutil as T
from kbml.anchor import anchor, kalshi_start, series_probability
from kbml.models import MODELS
from kbml.table import holdout_stage_ids, in_scope, is_qualifier
from kbml.teams import team_key


def test_team_keys_meet_like_the_rust_table():
    for a, b in [("Natus Vincere", "NAVI"), ("Ninjas in Pyjamas", "NIP"), ("Team Vitality", "Vitality"),
                 ("PARIVISION", "PVISION"), ("BC.Game Esports", "GOSH"), ("Dendele CS", "Dendele"),
                 ("BetBoom Team", "BB"), ("The MongolZ", "MongolZ"), ("Team Liquid", "TL")]:
        assert team_key(a) == team_key(b), (a, b)
    assert team_key("FaZe") != team_key("FURIA")


def test_kalshi_ticker_times_follow_us_eastern():
    assert T.iso(kalshi_start("KXCS2GAME-26NOV250530CWGOSH")) == "2026-11-25T10:30:00Z"  # EST
    assert T.iso(kalshi_start("KXCS2GAME-26JUL250530CWGOSH")) == "2026-07-25T09:30:00Z"  # EDT


def test_anchor_blend_and_series():
    assert anchor(None, None) is None
    assert anchor(0.6, None) == 0.6 and anchor(None, 0.7) == 0.7
    assert abs(anchor(0.6, 0.7) - 1 / (1 + np.exp(-(0.8 * np.log(1.5) + 0.2 * np.log(0.7 / 0.3))))) < 1e-12
    assert abs(series_probability(0.6, "bo3") - 0.648) < 1e-12


def test_qualifier_rule_and_scope():
    assert is_qualifier("Perfect World Major Shanghai: European RMR A 2024")
    assert is_qualifier("BLAST Premier Bounty Closed Qualifier 2025")
    assert not is_qualifier("IEM Cologne Major 2026")
    begins = T.parse("2025-01-01T00:00:00Z")
    assert in_scope("s", False, begins) and in_scope("a", False, begins)
    assert not in_scope("b", False, begins) and not in_scope("s", True, begins)
    assert not in_scope("s", False, T.parse("2024-03-01T00:00:00Z"))


def test_bootstrap_port_matches_the_rust_tests():
    b = stats.event_bootstrap([1, 2], [-1.0, 1.0], 4000, 0.9, 7)
    assert b["mean"] == 0.0 and b["low"] == -1.0 and b["high"] == 1.0
    assert abs(b["std_error"] - 0.5 ** 0.5) < 0.03
    b = stats.event_bootstrap([1, 1, 1, 2], [-0.1, -0.1, -0.1, 0.1], 4000, 0.9, 7)
    assert abs(b["mean"] + 0.05) < 1e-12 and abs(b["low"] + 0.1) < 1e-12 and abs(b["high"] - 0.1) < 1e-12
    one = stats.event_bootstrap([1, 1], [0.2, 0.4], 500, 0.9, 1)
    assert abs(one["low"] - 0.3) < 1e-12 and abs(one["high"] - 0.3) < 1e-12 and one["std_error"] == 0.0
    assert stats.event_bootstrap([], [], 500, 0.9, 1) is None


def test_seeded_rng_is_xoshiro256pp_seeded_by_splitmix64():
    # Reference: xoshiro256++ with SplitMix64(0) seeding.
    r = stats.SeededRng(0)
    assert r.s[0] == 0xE220A8397B1DCDAF
    first = r.next_u64()
    assert 0 <= first < 2 ** 64
    assert stats.SeededRng(33).next_u64() == stats.SeededRng(33).next_u64()


def _walk(m, mp, model="logreg"):
    X, kinds = F.compute(m, mp, F.resolve("baseline"))
    return evaluate.walk_forward(m, X, kinds, model, min_train=20).set_index("match_id")


def test_training_only_on_events_that_ended_before(table):
    m, mp = table
    events = evaluate.event_infos(m)
    for e in events.values():
        for t in events.values():
            if t.trains(e):
                assert t.ended < e.begins and t.id != e.id
    pred = _walk(m, mp)
    first = min(events.values(), key=lambda e: e.begins)
    assert pred.loc[pred.event_id == first.id, "fallback"].all()  # nothing to fit on: the anchor


@pytest.mark.parametrize("model", ["logreg", "lgbm"])
def test_reversing_a_later_event_moves_no_earlier_prediction(table, model):
    m, mp = table
    base = _walk(m, mp, model)
    rev_event = 4
    changed = m.copy()
    sel = changed["event_id"] == rev_event
    changed.loc[sel, "team_a_won"] = ~changed.loc[sel, "team_a_won"].astype(bool)
    after = _walk(changed, mp, model)
    first_finish = int(m.loc[sel, "known_finish"].min())
    earlier = m.loc[m["cutoff"] <= first_finish, "match_id"]
    assert np.allclose(base.loc[earlier, "variant"], after.loc[earlier, "variant"], atol=0, rtol=0)
    later = m.loc[m["event_id"] > rev_event, "match_id"]
    assert not np.allclose(base.loc[later, "variant"], after.loc[later, "variant"])


def test_qualifiers_neither_scored_nor_fitted(table):
    m, mp = table
    m = m.copy()
    q = m["event_id"] == 2
    m.loc[q, "is_qualifier"] = True
    m.loc[q, "in_scope"] = False
    pred = _walk(m, mp)
    assert 2 not in set(pred["event_id"])
    # flipping the qualifier's results moves no fitted model, only features of later Matches
    X, kinds = F.compute(m, mp, F.resolve("anchor,format"))  # features that do not read results
    a = evaluate.walk_forward(m, X, kinds, "logreg", min_train=20)
    m2 = m.copy()
    m2.loc[q, "team_a_won"] = ~m2.loc[q, "team_a_won"].astype(bool)
    b = evaluate.walk_forward(m2, X, kinds, "logreg", min_train=20)
    assert np.array_equal(a["variant"].to_numpy(), b["variant"].to_numpy())


def test_models_are_antisymmetric(table):
    m, mp = table
    X, kinds = F.compute(m, mp, F.resolve("baseline"))
    cols = list(kinds)
    anti = np.array([kinds[c] == "anti" for c in cols])
    x = X[cols].to_numpy(float)
    y = m["team_a_won"].to_numpy(float)
    for name, cls in MODELS.items():
        model = cls(anti).fit(x, y, cls.grid[0])
        p = model.predict(x)
        xs = x.copy()
        xs[:, anti] = -xs[:, anti]
        assert np.allclose(p, 1 - model.predict(xs), atol=1e-9), name


def test_holdout_rule():
    from kbml.archive import Archive, ReplayStage
    boundary = T.SPEC_PUBLISHED

    def stage(i, ev, cutoff):
        return ReplayStage(i, ev, "E", "s", "S", "other", {}, [], 0, True, None, cutoff)

    arch = Archive(stages=[stage(1, 10, boundary - T.DAY), stage(2, 11, boundary + 1), stage(22378, 12, boundary + 1)])
    ends = {10: boundary - 1, 11: boundary + T.DAY, 12: boundary + T.DAY}
    assert holdout_stage_ids(arch, ends) == {2}
    assert holdout_stage_ids(arch, {10: None, 11: boundary - 1, 12: None}) == {1, 2}
