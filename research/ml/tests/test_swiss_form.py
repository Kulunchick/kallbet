import numpy as np
import pytest

torch = pytest.importorskip("torch")

from kbml import pickem_check, swiss_form, swiss_gpu  # noqa: E402
from kbml.pickem_data import Stage  # noqa: E402

MAJOR = {(w, l): 3 if 2 in (w, l) else 1 for w in range(3) for l in range(3)}


def stage():
    return Stage(1, "test", 0, list(map(str, range(16))), [(i, i + 8) for i in range(8)], MAJOR, None, True, True,
                 list(range(16)))


def test_zero_shock_is_the_independent_simulation():
    values = np.random.default_rng(1).uniform(0.1, 0.9, 240)
    dev = torch.device("cpu")
    plain = swiss_gpu.simulate_many(stage(), values, 4000, dev, 2000, 33)
    zero = swiss_gpu.simulate_many(stage(), values, 4000, dev, 2000, 33, sd=np.zeros(16))
    np.testing.assert_array_equal(plain, zero)


def test_shocks_widen_the_spread_of_a_dominant_team():
    p = np.full(240, 0.5)
    a, b = np.triu_indices(16, 1)
    p.reshape(2, -1)[:, a == 0] = 0.9  # team 0 beats everyone 90% of the time
    dev = torch.device("cpu")
    wins = lambda o: (o[:, 0] == 0).mean()
    base = wins(swiss_gpu.simulate_many(stage(), p, 20000, dev, 20000, 33))
    shocked = wins(swiss_gpu.simulate_many(stage(), p, 20000, dev, 20000, 33, sd=np.full(16, 1.0)))
    assert shocked < base  # a shock on a 90% favourite lowers its expected strength in logit-mean terms


def test_regularised_search_with_lambda_zero_is_the_original():
    values = np.random.default_rng(2).uniform(0.1, 0.9, 240)
    out = swiss_gpu.simulate_many(stage(), values, 6000, torch.device("cpu"), 6000, 33)
    a = pickem_check.expected_picks(out)
    starts = [a, np.random.default_rng(3).permutation(a)]
    np.testing.assert_array_equal(pickem_check.search(out, starts), swiss_form.search_reg(out, starts, 0.0))
    # a large lambda turns B into the expected-points optimum
    big = swiss_form.search_reg(out, starts, 50.0)
    marg = np.array([(out == k).mean(axis=0) for k in range(3)])
    assert pickem_check.scores(out, big).mean() >= pickem_check.scores(out, a).mean() - 1e-9
