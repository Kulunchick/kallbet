"""The DP catch-up policy: transitions, tables against brute force, and replay parity with catchup.py."""
import itertools
import math

import numpy as np

from kbml import catchup, catchup_dp as dp
from kbml.stats import SeededRng


def test_transition_rows_are_distributions():
    for n, eps in ((3, 0.1), (11, 0.25), (11, 0.0)):
        T, index = dp.transitions(n, eps)
        for outcome in (0, 1):
            assert np.allclose(T[outcome].sum(axis=(0, 2)), 1.0)
    T, index = dp.transitions(11, 0.0)  # market follower: the field moves as one
    s = index[(11, 0)]
    assert T[1][1, s, s] == 1.0 and T[0][0, s, s] == 1.0


def brute(n_opp, eps, q, r, own, opps):
    """Exact P(first) by enumeration for one Match type repeated r times, optimal picks."""
    if r == 0:
        return float(own >= max(opps))
    best = 0.0
    for take_cons in (True, False):
        v = 0.0
        for outcome in (1, 0):
            po = q if outcome else 1 - q
            pc = 1 - eps if outcome else eps
            ours = own + (take_cons == bool(outcome))
            for hits in itertools.product((0, 1), repeat=n_opp):
                ph = math.prod(pc if h else 1 - pc for h in hits)
                v += po * ph * brute(n_opp, eps, q, r - 1, ours, [o + h for o, h in zip(opps, hits)])
        best = max(best, v)
    return best


def test_tables_match_brute_force_with_two_opponents():
    # with two opponents the (c0, c1, rest two behind) state is exact while nobody falls 2+ behind
    for eps, q in ((0.1, 0.62), (0.25, 0.55)):
        t = dp.build(2, eps, np.array([q]), np.array([1.0]), 0.0, 0.75, 3)
        for own, opps in ((0, [0, 0]), (-1, [0, -1]), (0, [0, -1])):
            leader = max(opps)
            c0, c1 = opps.count(leader), opps.count(leader - 1)
            got = t.V[3][own - leader + dp.D, t.index[(c0, c1)]]
            assert abs(got - brute(2, eps, q, 3, own, opps)) < 1e-12


def test_dp_takes_the_favourite_when_leading_on_the_last_match():
    t = dp.build(11, 0.1, np.array([0.6]), np.array([1.0]), 0.0, 0.75, 5)
    cons, other = t.values(1, 1, 3, 2, 0.55)  # one ahead with one Match left: any pick keeps first
    assert cons == other == 1.0
    cons, other = t.values(1, -1, 3, 2, 0.55)  # one behind, last Match: only the underdog can tie
    assert other > cons


def test_plain_and_window_replay_equal_catchup_simulate():
    rng = np.random.default_rng(5)
    games = []
    for i in range(40):
        a = float(rng.uniform(0.2, 0.9))
        games.append((i * 10, i * 10 + int(rng.integers(5, 40)), a, None if i % 3 else float(rng.uniform(0.2, 0.9)),
                      bool(rng.random() < a), {"m": float(rng.uniform(0.3, 0.8))}))
    for size, noise in ((4, 0.1), (12, 0.25), (12, None)):
        want = catchup.simulate(games, size, noise, SeededRng(9),
                                [("anchor", False), ("anchor", True), ("m", False), ("m", True)])
        got = dp.simulate(games, size, noise, SeededRng(9),
                          [dp.Plain("anchor"), dp.Window("anchor"), dp.Plain("m"), dp.Window("m")])
        assert got == want
