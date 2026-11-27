"""Reproduce the archived Swiss Pick'em report with python -m kbml.pickem_check."""

import argparse
from itertools import combinations
from pathlib import Path

import numpy as np
from scipy.optimize import linear_sum_assignment

from .pickem_data import forecast_all

SLOTS = np.repeat(np.arange(3), [2, 2, 6])


def matchings(pool):
    if not pool:
        yield ()
        return
    a = pool[0]
    for b in reversed(pool[1:]):
        for tail in matchings([t for t in pool[1:] if t != b]):
            yield ((a, b),) + tail


TABLES = {n: list(matchings(list(range(n)))) for n in (4, 8)}
# Same six-team priority table as sidecar/src/adapters/pickem/swiss.rs.
TABLES[6] = [
    tuple((int(s[i]) - 1, int(s[i + 1]) - 1) for i in (0, 2, 4))
    for s in [
        "162534",
        "162435",
        "152634",
        "152436",
        "142635",
        "142536",
        "162345",
        "152346",
        "132645",
        "132546",
        "142356",
        "132456",
        "123645",
        "123546",
        "123456",
    ]
]


def simulate(stage, values, runs, rng):
    p = np.zeros((2, 16, 16))
    a, b = np.triu_indices(16, 1)
    p[:, a, b] = np.asarray(values).reshape(2, -1)
    p[:, b, a] = 1 - p[:, a, b]
    seed = np.arange(16) if stage.published else stage.seed
    wins = np.zeros((runs, 16), dtype=np.int8)
    losses = np.zeros_like(wins)
    played = np.zeros((runs, 16), dtype=np.uint16)
    opponents = np.zeros((4, runs, 16), dtype=np.uint8)
    row = np.arange(runs)
    bits = (1 << np.arange(16)).astype(np.uint16)

    def play(a, b, best_of, rnd):
        win = rng.random(runs) < p[int(best_of == 3), a, b]
        wins[row, np.where(win, a, b)] += 1
        losses[row, np.where(win, b, a)] += 1
        played[row, a] |= bits[b]
        played[row, b] |= bits[a]
        if rnd < 4:
            opponents[rnd, row, a] = b
            opponents[rnd, row, b] = a

    for a, b in stage.opening:
        play(a, b, stage.formats[0, 0], 0)
    for r in range(1, 5):
        diff = wins - losses
        buchholz = sum(np.take_along_axis(diff, opponent, axis=1) for opponent in opponents[:r])
        priority = seed - 16 * buchholz.astype(np.int16)
        pending = []
        for w in range(max(0, r - 2), min(2, r) + 1):
            l = r - w
            mask = (wins == w) & (losses == l)
            n = int(mask[0].sum())
            pool = np.argsort(np.where(mask, priority, 10000), axis=1)[:, :n]
            choices = np.empty((runs, n // 2, 2), dtype=np.uint8)
            unresolved = row
            for pairing in TABLES[n]:
                if not len(unresolved):
                    break
                pairs = pool[unresolved][:, pairing]
                bad = (played[unresolved[:, None], pairs[:, :, 0]] & bits[pairs[:, :, 1]]).any(axis=1)
                choices[unresolved[~bad]] = pairs[~bad]
                unresolved = unresolved[bad]
            if len(unresolved):
                choices[unresolved] = pool[unresolved][:, TABLES[n][0]]
            pending.append((choices, stage.formats[w, l]))
        for choices, fmt in pending:
            for a, b in choices.transpose(1, 2, 0):
                play(a, b, fmt, r)
    return np.select(
        [(wins == 3) & (losses == 0), (losses == 3) & (wins == 0), (wins == 3) & (losses > 0)],
        [0, 1, 2],
        default=3,
    ).astype(np.uint8)


def expected_picks(outcomes):
    marginals = np.array([(outcomes == role).mean(axis=0) for role in range(3)])
    _, teams = linear_sum_assignment(marginals[SLOTS], maximize=True)
    roles = np.full(16, 3)
    roles[teams] = SLOTS
    return roles


def scores(outcomes, roles):
    return ((outcomes == roles) & (roles != 3)).sum(axis=-1)


def packed(bits):
    return np.packbits(bits)


def search(outcomes, starts):
    # The change in score from swapping two team roles is in [-2,2]. Only
    # simulations currently scoring 3,4,5,6 can change the threshold count.
    hits = [
        [packed(outcomes[:, t] == k) if k != 3 else packed(np.zeros(len(outcomes), bool)) for k in range(4)]
        for t in range(16)
    ]
    visited = {}
    best = None
    for initial in starts:
        roles = initial.copy()
        while roles.tobytes() not in visited:
            old = scores(outcomes, roles)
            value = int((old >= 5).sum())
            visited[roles.tobytes()] = value
            if best is None or value > best[0]:
                best = (value, roles.copy())
            s3, s4, s5, s6 = (packed(old == k) for k in (3, 4, 5, 6))
            gain, move = 0, None
            for a, b in combinations(range(16), 2):
                i, j = roles[a], roles[b]
                if i == j:
                    continue
                add_a, add_b = hits[a][j], hits[b][i]
                rem_a, rem_b = hits[a][i], hits[b][j]
                up = (add_a & ~rem_b) | (add_b & ~rem_a)
                down = (rem_a & ~add_b) | (rem_b & ~add_a)
                better = (s3 & add_a & add_b) | (s4 & up)
                worse = (s6 & rem_a & rem_b) | (s5 & down)
                delta = int(np.bitwise_count(better).sum()) - int(np.bitwise_count(worse).sum())
                if delta > gain:
                    gain, move = delta, (a, b)
            if move is None:
                break
            a, b = move
            roles[a], roles[b] = roles[b], roles[a]
    return best[1]


def evaluate_strategy(task):
    stage, model, values, runs = task
    rng = np.random.default_rng(33)
    outcomes = np.concatenate(
        [simulate(stage, values, min(5000, runs - i), rng) for i in range(0, runs, 5000)]
    )
    small = outcomes[: runs // 2]
    a_small, a = expected_picks(small), expected_picks(outcomes)
    # Multistart best-improvement exchange search; warm-start the larger search.
    gen = np.random.default_rng(33)
    starts = [gen.permutation(a_small) for _ in range(2)]
    b_small = search(small, [a_small, *starts])
    b = search(outcomes, [b_small, a, *starts])
    result = []
    for strategy, roles, half in (("A", a, a_small), ("B", b, b_small)):
        sim = scores(outcomes, roles)
        changed = np.count_nonzero((roles != 3) & (roles != half))
        result.append(
            (
                model,
                strategy,
                roles,
                int(scores(stage.truth, roles)),
                changed,
                float(sim.mean()),
                float((sim >= 5).mean()),
            )
        )
    return stage.id, result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--runs", type=int, default=200000)
    parser.add_argument("--jobs", type=int, default=3)
    parser.add_argument(
        "--pairs", action="store_true", help="also print both formats for all stage-start pairs"
    )
    parser.add_argument("--report", type=Path, default=Path(__file__).resolve().parents[1] / "PICKEM.md")
    args = parser.parse_args()
    from .pickem_report import run_report

    rows = list(forecast_all())
    if args.pairs:
        import csv
        import sys

        writer = csv.writer(sys.stdout)
        writer.writerow(["stage_id", "format", "team_a", "team_b", *rows[0][1]])
        for stage, forecasts, *_ in rows:
            pairs = [(fmt, a, b) for fmt in ("bo1", "bo3") for a, b in combinations(stage.teams, 2)]
            for position, pair in enumerate(pairs):
                writer.writerow([stage.id, *pair, *(v[position] for v in forecasts.values())])
    run_report(rows, args.runs, args.jobs, args.report)


if __name__ == "__main__":
    main()
