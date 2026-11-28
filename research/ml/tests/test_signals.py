"""Phase 5: de-vig methods, Murphy decomposition, SPA, the fusion's training rule, and point in time
for the new feature groups (veto_eb, openskill, travel)."""
import numpy as np
import pandas as pd
import pytest

from kbml import features as F
from kbml import features_signals as fs
from kbml import fusion, spa
from kbml import timeutil as T
from kbml.market_detail import devig_power, devig_proportional, devig_shin
from kbml.signal_eval import murphy

POOL = ["Ancient", "Dust2", "Inferno", "Mirage", "Nuke", "Overpass", "Train"]


def test_devig_methods():
    for oa, ob in ((1.526, 2.55), (1.1, 7.0), (2.4, 1.6)):
        prop, pw, sh = devig_proportional(oa, ob), devig_power(oa, ob), devig_shin(oa, ob)
        assert 0 < pw < 1 and 0 < sh < 1
        fav_is_a = oa < ob
        # both methods move probability toward the favourite (favourite-longshot correction)
        assert (pw > prop) == fav_is_a and (sh > prop) == fav_is_a
        # antisymmetric in the two sides
        assert abs(devig_power(ob, oa) - (1 - pw)) < 1e-9
        assert abs(devig_shin(ob, oa) - (1 - sh)) < 1e-9
    # no overround: every method returns the implied probability
    assert abs(devig_power(2.0, 2.0) - 0.5) < 1e-9
    assert abs(devig_shin(1.25, 5.0) - 0.8) < 1e-6


def test_murphy_identity_on_bin_centred_forecasts():
    rng = np.random.default_rng(3)
    p = (rng.integers(0, 10, 4000) + 0.5) / 10
    y = rng.random(4000) < p
    d = murphy(p, y)
    P, Y = np.concatenate([p, 1 - p]), np.concatenate([y, ~y])
    brier = np.mean((P - Y) ** 2)
    assert abs(brier - (d["reliability"] - d["resolution"] + d["uncertainty"])) < 1e-12
    assert abs(d["uncertainty"] - 0.25) < 1e-12


def test_spa_detects_a_better_model_and_not_a_copy():
    rng = np.random.default_rng(0)
    n, k = 1200, 40
    ev = np.repeat(np.arange(k), n // k)
    bench = rng.normal(1.0, 0.3, n)
    good = bench - 0.15 + rng.normal(0, 0.05, n)
    noise = {f"n{i}": bench + rng.normal(0, 0.05, n) for i in range(5)}
    r = spa.spa(ev, bench, {"good": good, **noise})
    assert r["p_consistent"] < 0.05 and r["survivors"] == ["good"]
    r2 = spa.spa(ev, bench, noise)
    assert r2["p_consistent"] > 0.1 and r2["survivors"] == []


def test_fusion_stage2_trains_only_on_earlier_priced_events(table):
    m, _ = table
    m = m.copy()
    m["block"] = "scope"
    Z = pd.DataFrame({"z_fund": np.log(m["market_p"] / (1 - m["market_p"])), "z_pin": 0.0}, index=m.index)
    base = pd.Series(0.5, index=m.index)
    p, info = fusion.stage2(m, Z, base, min_rows=10)
    ev = m["event_id"].to_numpy()
    # the first Event has no earlier Event: it keeps the base
    assert info["rows"][1] == 0 and 1 in info["fallback_events"]
    assert np.allclose(p[ev == 1], 0.5)
    # later Events train on exactly the priced Matches of Events that ended before they began
    for e, rows in info["rows"].items():
        assert rows == int((ev < e).sum())
    # a future result cannot move an earlier Event's fusion
    m2 = m.copy()
    m2.loc[ev == ev.max(), "team_a_won"] = ~m2.loc[ev == ev.max(), "team_a_won"]
    p2, _ = fusion.stage2(m2, Z, base, min_rows=10)
    early = ev < ev.max()
    assert np.allclose(p[early], p2[early])


def test_fundamental_columns_drop_every_market_input():
    kinds = {c: k for g in F.resolve("p3_all") for c, k in F.REGISTRY[g].columns.items()}
    cols = fusion.fundamental_columns("p3_all", kinds)
    assert not set(cols) & fusion.MARKET_COLUMNS
    assert "logit_vrs" in cols and "vrs_points_diff" in cols


def _records(m, seed=2):
    """Synthetic HLTV records (7-step veto, per-map players) for every Match of the table."""
    rng = np.random.default_rng(seed)
    players = {k: [hash((k, i)) % 100000 for i in range(5)] for k in set(m.team_a_key) | set(m.team_b_key)}
    out = []
    for r in m.itertuples():
        order = list(rng.permutation(POOL))
        veto = [{"step": i + 1, "team": (r.team_a if i % 2 == 0 else r.team_b),
                 "action": "removed" if i in (0, 1, 4, 5) else "picked", "map_name": order[i]} for i in range(6)]
        veto.append({"step": 7, "team": None, "action": "left_over", "map_name": order[6]})
        played = [order[2], order[3], order[6]]
        wins = ["a" if rng.random() < 0.5 else "b" for _ in played]
        maps = [{"position": i + 1, "map_name": mp, "winner": w, "played": True} for i, (mp, w) in enumerate(zip(played, wins))]
        per_map = [{"map_name": mp, "teams": [
            {"team_id": r.team_a_id, "team_name": r.team_a, "players": [{"player_id": p} for p in players[r.team_a_key]]},
            {"team_id": r.team_b_id, "team_name": r.team_b, "players": [{"player_id": p} for p in players[r.team_b_key]]}]}
            for mp in played]
        out.append({"match_id": r.match_id, "known_finish": r.known_finish, "rec": {
            "keys": [r.team_a_key, r.team_b_key], "team_ids": [r.team_a_id, r.team_b_id],
            "started_at": int(r.actual_start), "format": "bo3", "maps": maps, "veto": veto,
            "overall": [{"team_name": r.team_a}, {"team_name": r.team_b}], "per_map": per_map}})
    rec = pd.DataFrame(out)
    rec["known_finish"] = rec["known_finish"].astype("Int64")
    return rec


def _maps_from(m, rec):
    rows = []
    for r, x in zip(m.itertuples(), rec["rec"]):
        for mp in x["maps"]:
            rows.append(dict(match_id=r.match_id, hltv_match_id=r.match_id, known_finish=r.known_finish,
                             started_at=r.actual_start, is_lan=True, position=mp["position"], map_name=mp["map_name"],
                             team_a_key=r.team_a_key, team_b_key=r.team_b_key, rounds_a=13, rounds_b=8,
                             a_won=mp["winner"] == "a"))
    out = pd.DataFrame(rows)
    out["known_finish"] = out["known_finish"].astype("Int64")
    return out


@pytest.fixture
def with_records(table):
    m, _ = table
    rec = _records(m)
    fs.Travel.stages = {s: ("DE" if s % 2 else "US", True) for s in m["stage_id"].unique()}
    fs.Travel.homes = {t: "SE" for t in set(m.team_a_id) | set(m.team_b_id)}
    yield m, _maps_from(m, rec), rec
    fs.Travel.stages, fs.Travel.homes = {}, {}


def test_new_groups_cover_and_stay_point_in_time(with_records):
    m, mp, rec = with_records
    groups = ["veto_eb", "openskill", "travel"]
    X, _ = F.compute(m, mp, groups, ({}, rec))
    assert X["os_has"].iloc[0] == 0  # nobody has played yet
    assert X["os_has"].iloc[-len(m) // 3:].eq(1).all()
    assert (X["travel_has"] == 1).all()
    assert (X["veb_has"] == 1).sum() > len(m) // 2  # once a 14-day window of vetoes is released
    # flip one Match's maps: nothing with a cutoff at or before its finish moves
    i = len(m) // 2
    rec2 = rec.copy()
    r = dict(rec2.at[i, "rec"])
    r["maps"] = [dict(x, winner="b" if x["winner"] == "a" else "a") for x in r["maps"]]
    rec2.at[i, "rec"] = r
    mp2 = _maps_from(m, rec2)
    m2 = m.copy()
    m2.loc[i, "team_a_won"] = not m2.loc[i, "team_a_won"]
    X2, _ = F.compute(m2, mp2, groups, ({}, rec2))
    finish = int(m.loc[i, "known_finish"])
    early = m.loc[m["cutoff"] <= finish, "match_id"]
    pd.testing.assert_frame_equal(X.loc[early], X2.loc[early])
    late = m.loc[m["cutoff"] > finish, "match_id"]
    assert not X.loc[late].equals(X2.loc[late])


def test_active_pool_detects_a_map_swap():
    p = fs.ActivePool(window_days=14)
    day = fs.DAY
    for d in range(20):
        p.add(d * day, POOL)
    p.refresh(20 * day)
    assert p.pool == frozenset(POOL) and p.changes == []
    new = POOL[:-1] + ["Cache"]
    for d in range(40, 60):
        p.add(d * day, new)
    p.refresh(60 * day)
    assert p.pool == frozenset(new)
    assert p.changes == [(60 * day, ["Cache"], ["Train"])]
    assert p.entered["Cache"] == 60 * day


def _brute(pm, ban, pick):
    """Recursive exact veto integration (reference for `veto_series`)."""
    acts = ["b", "b", "p", "p", "b", "b"]
    total = 0.0
    for first in (0, 1):
        seq = [first, 1 - first] * 3

        def rec(step, avail, picks, prob):
            if step == 6:
                d = avail[0]
                p1, p2, p3 = pm[picks[0]], pm[picks[1]], pm[d]
                return prob * (p1 * p2 + p1 * (1 - p2) * p3 + (1 - p1) * p2 * p3)
            w = (ban if acts[step] == "b" else pick)[seq[step]]
            tot = sum(w[i] for i in avail)
            return sum(rec(step + 1, [j for j in avail if j != i], picks + [i] if acts[step] == "p" else picks,
                           prob * w[i] / tot) for i in avail)
        total += rec(0, list(range(7)), [], 1.0) / 2
    return total


def test_exact_veto_integration_matches_brute_force_and_gpu():
    rng = np.random.default_rng(5)
    batch = np.stack([np.stack([rng.uniform(0.2, 0.8, 7), *rng.uniform(0.05, 1, (4, 7))]) for _ in range(3)])
    got = fs.veto_series(batch, "numpy")
    for b, g in zip(batch, got):
        assert abs(_brute(b[0], [b[1], b[2]], [b[3], b[4]]) - g) < 1e-12
    # equal maps → the series is the Bo3 conversion of that map probability
    flat = np.stack([np.full(7, 0.6), *np.ones((4, 7))])[None]
    assert abs(fs.veto_series(flat, "numpy")[0] - 0.6 ** 2 * (3 - 2 * 0.6)) < 1e-12
    torch = pytest.importorskip("torch")
    if torch.cuda.is_available():
        assert np.allclose(fs.veto_series(batch, "cuda"), got, atol=1e-12)
