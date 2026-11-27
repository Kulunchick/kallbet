from itertools import combinations

import numpy as np

from kbml.pickem_check import SLOTS, expected_picks, scores, search, simulate
from kbml.pickem_data import Stage
from kbml.pickem_report import BASE_ROLES, random_reference


def test_joint_search_has_no_improving_one_swap_and_preserves_slots():
    rng = np.random.default_rng(12)
    # Deliberately not divisible by eight, to test the bit-packed margins.
    outcomes = np.stack([rng.permutation(BASE_ROLES) for _ in range(257)])
    a = expected_picks(outcomes)
    b = search(outcomes, [a, rng.permutation(a)])
    assert np.bincount(b).tolist() == [2, 2, 6, 6]
    passes = np.count_nonzero(scores(outcomes, b) >= 5)
    assert passes >= np.count_nonzero(scores(outcomes, a) >= 5)
    for i, j in combinations(range(16), 2):
        candidate = b.copy()
        candidate[[i, j]] = b[[j, i]]
        assert passes >= np.count_nonzero(scores(outcomes, candidate) >= 5)


def test_simulation_uses_actual_bucket_format():
    # Low index always wins BO1 and always loses BO3.
    stage = Stage(
        1,
        "test",
        0,
        list(map(str, range(16))),
        [(i, i + 8) for i in range(8)],
        {(w, l): 3 for w in range(3) for l in range(3)},
        BASE_ROLES,
        True,
        True,
        list(range(16)),
    )
    values = np.repeat([1.0, 0.0], 120)
    for fmt, strongest, weakest in ((3, 15, 0), (1, 0, 15)):
        stage.formats = dict.fromkeys(stage.formats, fmt)
        outcomes = simulate(stage, values, 31, np.random.default_rng(33))
        assert np.all(outcomes[:, strongest] == 0) and np.all(outcomes[:, weakest] == 1)
        assert np.bincount(outcomes[0]).tolist() == [2, 2, 6, 6]
        slip = expected_picks(outcomes)
        assert np.all(scores(outcomes, slip) == len(SLOTS))


def test_uniform_random_reference_counts_every_legal_slip():
    from math import comb

    passes, total = random_reference()
    assert total == comb(16, 2) * comb(14, 2) * comb(12, 6)
    assert 0 < passes < total


def test_published_singapore_seeds_replay_all_five_rounds():
    from kbml import archive
    from kbml.pickem_data import FIXTURES, LAST_RESULTS, manifest

    source = archive.load(FIXTURES, with_hltv=False)
    replay = next(s for s in source.stages if s.id == 22378)
    stage = manifest(replay, FIXTURES)
    assert stage.published
    matrix = np.full((2, 16, 16), np.nan)
    results = [
        (m.team_a.name, m.team_b.name, m.winner == m.team_a.id, m.format == "bo3") for m in replay.matches
    ]
    results += [(winner, loser, True, True) for winner, loser in LAST_RESULTS]
    for a, b, a_won, bo3 in results:
        i, j = stage.teams.index(a), stage.teams.index(b)
        matrix[int(bo3), i, j] = float(a_won)
        matrix[int(bo3), j, i] = float(not a_won)

    class RecordedMatches:
        def random(self, n):
            return self

        def __lt__(self, probabilities):
            # The pairing algorithm must request a real match in the correct format.
            assert np.isfinite(probabilities).all()
            return probabilities > 0.5

    a, b = np.triu_indices(16, 1)
    outcomes = simulate(stage, matrix[:, a, b], 1, RecordedMatches())
    np.testing.assert_array_equal(outcomes[0], stage.truth)
