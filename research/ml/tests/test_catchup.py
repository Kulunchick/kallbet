"""The catch-up port reproduces docs/lab/research/catch-up.md (Rust lab, seed 33, 2000 trials)."""
import pytest

from kbml import catchup, paths
from kbml.stats import SeededRng


def test_catch_up_rule():
    assert catchup.catch_up(0.55, 1) is False and catchup.catch_up(0.55, 0) is True
    assert catchup.catch_up(0.61, 3) is True and catchup.catch_up(0.40, 1) is True
    assert catchup.catch_up(0.5, 2) is True


def test_strict_reveal_boundary():
    # the bot trails only once a result is public strictly before the next cutoff
    games = [(0, 10, 0.8, 0.4, False, {}), (10, 20, 0.55, 0.4, False, {}), (11, 30, 0.55, 0.4, False, {})]
    rng = SeededRng(1)
    places = catchup.simulate(games, 2, None, rng, [("anchor", False), ("anchor", True)])
    # market follower picks B every time (3 points); plain bot 0; catch-up flips only the third
    assert places == [2, 2]


@pytest.mark.skipif(not paths.ARCHIVE.is_dir(), reason="research archive not present")
def test_parity_with_the_rust_catch_up_report():
    from kbml.table import load_table
    m, _, _ = load_table(log=lambda *a: None)
    res, n = catchup.parity(m)
    assert n == 62
    want = {(4, "10%"): (0.5765, 0.650, 1.527, 1.562), (12, "10%"): (0.192, 0.394, 2.867, 3.4265),
            (4, "25%"): (0.691, 0.6985, 1.372, 1.4225), (12, "25%"): (0.3945, 0.4495, 2.305, 2.637),
            (4, "market"): (0.963, 0.9835, 1.111, 1.0495), (12, "market"): (0.963, 0.9835, 1.407, 1.1815)}
    for k, v in want.items():
        assert all(abs(a - b) < 1e-9 for a, b in zip(res[k], v)), k
