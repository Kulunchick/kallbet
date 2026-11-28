"""W6 veto evaluation: point-in-time rules, exact integration against brute force, scoring."""
import itertools
import random

import numpy as np

from kbml import veto_eval as ve

DAY = 86_400_000_000
NAMES = [ve.MAPS[i] for i in range(10)]


def synthetic(n=60, seed=1):
    """A chronological list of Bo3 records over three teams, seven-map pool, one record per day."""
    rng = random.Random(seed)
    pool = list(range(7))
    R = []
    for i in range(n):
        order = pool[:]
        rng.shuffle(order)
        first = rng.randrange(2)
        sides = [first, 1 - first, first, 1 - first, first, 1 - first]
        kinds = [0, 0, 1, 1, 0, 0]
        ordn, seen = [], {}
        for s, k in zip(sides, kinds):
            ordn.append(seen.get((s, k), 0))
            seen[(s, k)] = seen.get((s, k), 0) + 1
        a, b = rng.sample(["t1", "t2", "t3"], 2)
        R.append({"fmt": 1, "keys": [a, b], "sides": sides, "kinds": kinds, "ordn": ordn, "maps": order,
                  "names": [NAMES[m] for m in order], "started_at": 1_000_000 + i * DAY,
                  "match_id": i, "cutoff": 1_000_000 + i * DAY, "known_finish": 1_000_000 + i * DAY + DAY // 4,
                  "event_id": i // 5, "tier": "s", "block": "scope", "in_scope": True, "scored": True,
                  "ev_begins": 0, "ev_ended": 1})
    return R


def test_own_and_later_vetoes_never_change_a_forecast():
    R = synthetic()
    base = ve.run_pass(R, "detected", log=lambda *_: None)
    j = 40
    R2 = [dict(r) for r in R]
    R2[j]["maps"] = list(reversed(R[j]["maps"]))
    R2[j]["names"] = [NAMES[m] for m in R2[j]["maps"]]
    flipped = ve.run_pass(R2, "detected", log=lambda *_: None)
    # next-step forecasts are conditional on the Match's own realised earlier steps (by design), so only
    # Matches before it are compared in full; for the Match itself the first step and the pre-veto
    # inputs (nothing realised) must be identical
    for key in ("D_eb", "D_lab", "D_freq", "D_global", "D_shrunk", "D_freqx"):
        assert np.array_equal(base[key][:j], flipped[key][:j]), key
        assert np.array_equal(base[key][j, ..., 0, :], flipped[key][j, ..., 0, :]), key
    for key in ("M_freq", "M_global", "W_eb", "W_lab", "W_freq", "nprior", "pool_eb"):
        assert np.array_equal(base[key][: j + 1], flipped[key][: j + 1]), key
    assert not np.array_equal(base["W_eb"][j + 1:], flipped["W_eb"][j + 1:])


def test_a_veto_finishing_after_the_cutoff_is_not_released():
    R = synthetic()
    # Match 10 finishes after the cutoff of Match 11 (they overlap): Match 11 must not see it
    R[10]["known_finish"] = R[11]["cutoff"] + 1
    R2 = [dict(r) for r in R]
    R2[10]["maps"] = list(reversed(R[10]["maps"]))
    R2[10]["names"] = [NAMES[m] for m in R2[10]["maps"]]
    a = ve.run_pass(R, "detected", log=lambda *_: None)
    b = ve.run_pass(R2, "detected", log=lambda *_: None)
    assert np.array_equal(a["D_eb"][11], b["D_eb"][11])
    assert np.array_equal(a["D_freq"][11], b["D_freq"][11])
    assert not np.array_equal(a["D_eb"][12:], b["D_eb"][12:])  # released from Match 12 on


def test_first_forecast_is_uniform_without_history():
    R = synthetic()
    R[-1]["keys"] = ["new1", "new2"]
    out = ve.run_pass(R, "detected", log=lambda *_: None)
    d = out["D_eb"][-1, 0]
    assert np.allclose(d[d > 0], 1 / 7, atol=1e-6) and (d > 0).sum() == 7


def test_next_step_distributions_sum_to_one_and_exclude_used_maps():
    R = synthetic()
    out = ve.run_pass(R, "detected", log=lambda *_: None)
    v = np.where(out["valid"])[0]
    for key in ("D_eb", "D_lab", "D_uniform"):
        assert np.allclose(out[key][v].sum(-1), 1, atol=1e-5), key
    for i in v[:10]:
        for k in range(6):
            used = R[i]["maps"][:k]
            assert (out["D_eb"][i, k][used] == 0).all()


def brute_marginals(W, sides, kinds):
    steps = np.zeros((6, 7))
    left = np.zeros(7)
    for perm in itertools.permutations(range(7), 6):
        p, rem = 1.0, set(range(7))
        for k, m in enumerate(perm):
            w = W[sides[k]][kinds[k]]
            p *= w[m] / sum(w[x] for x in rem)
            rem.remove(m)
        for k, m in enumerate(perm):
            steps[k, m] += p
        left[rem.pop()] += p
    return steps, left


def test_integration_matches_brute_force_and_runs_on_torch_cpu():
    rng = np.random.default_rng(3)
    W = rng.uniform(0.2, 3.0, (1, 2, 2, 7))
    sides = np.array([[0, 1, 0, 1, 0, 1]])
    kinds = np.array([[0, 0, 1, 1, 0, 0]])
    r = ve.integrate(W, sides, kinds, "cpu")
    steps, left = brute_marginals(W[0], sides[0], kinds[0])
    assert np.allclose(r["steps"][0], steps) and np.allclose(r["left"][0], left)
    assert np.isclose(r["total"][0], 1) and np.isclose(r["sets"][0].sum(), 1) and np.isclose(r["triple"][0].sum(), 1)
    # step 1's marginal is just the first actor's normalised weights
    assert np.allclose(r["steps"][0, 0], W[0, 0, 0] / W[0, 0, 0].sum())


def test_uniform_integration_gives_equal_sets():
    W = np.ones((1, 2, 2, 7))
    r = ve.integrate(W, np.array([[1, 0, 1, 0, 1, 0]]), np.array([[0, 0, 1, 1, 0, 0]]), "cpu")
    assert np.allclose(r["sets"][0], 1 / 35) and np.allclose(r["steps"][0], 1 / 7)


def test_score_rank_floor_and_tie_break():
    D = np.zeros((1, 1, ve.V))
    D[0, 0, [0, 1, 2]] = [0.5, 0.3, 0.2]
    cand = np.zeros_like(D, bool)
    cand[0, 0, [0, 1, 2]] = True
    tie = np.zeros_like(D)
    s = ve.score(D, cand, tie, np.array([[1]]))
    assert s["rank"][0, 0] == 1 and s["hit1"][0, 0] == 0 and s["hit2"][0, 0] == 1
    assert np.isclose(s["p"][0, 0], 0.3) and np.isclose(s["bs"][0, 0], 0.25 + 0.49 + 0.04)
    off = ve.score(D, cand, tie, np.array([[5]]))  # chosen map outside the candidates
    assert off["p"][0, 0] == ve.FLOOR and off["hit3"][0, 0] == 0
    D2 = np.zeros((1, 1, ve.V))
    D2[0, 0, [3, 4]] = 0.5
    c2 = np.zeros_like(D2, bool)
    c2[0, 0, [3, 4]] = True
    t2 = np.zeros_like(D2)
    t2[0, 0, 4] = 5.0  # equal probability: the globally more frequent map ranks first
    assert ve.score(D2, c2, t2, np.array([[4]]))["hit1"][0, 0] == 1
    assert ve.score(D2, c2, t2, np.array([[3]]))["hit1"][0, 0] == 0


def test_parse_record_requires_a_full_veto_in_the_format_pattern():
    base = {"format": "bo3", "keys": ["aa", "bb"], "started_at": 1,
            "overall": [{"team_name": "AA"}, {"team_name": "BB"}]}
    acts = ["removed", "removed", "picked", "picked", "removed", "removed"]
    maps = ["Mirage", "Nuke", "Inferno", "Ancient", "Dust2", "Anubis", "Train"]
    veto = [{"step": i + 1, "team": "AA" if i % 2 == 0 else "BB", "action": a, "map_name": m}
            for i, (a, m) in enumerate(zip(acts, maps))]
    veto.append({"step": 7, "team": None, "action": "left_over", "map_name": "Train"})
    veto[6]["map_name"] = "Anubis"
    veto = veto[:6] + [{"step": 7, "team": None, "action": "left_over", "map_name": "Train"}]
    p = ve.parse_record({**base, "veto": veto})
    assert p and p["sides"] == [0, 1, 0, 1, 0, 1] and p["ordn"] == [0, 0, 0, 0, 1, 1]
    assert ve.parse_record({**base, "veto": veto[:6]}) is None
    assert ve.parse_record({**base, "format": "bo1", "veto": veto}) is None  # wrong pattern for Bo1
