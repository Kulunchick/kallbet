"""Offline Singapore 2026 Stage 1 Pick'em replay. Run with python -m kbml.pickem_check."""
import argparse
from itertools import combinations

import numpy as np
import pandas as pd
from scipy.optimize import linear_sum_assignment

from . import archive, evaluate, features, paths, timeutil
from .anchor import Markets, Standings, anchor
from .table import load_table
from .teams import team_key

# November 16 Liquipedia revision 3471273, lp-22378.json.gz, SwissStandings tiebreaker order.
SEEDS = ("NIP", "PVISION", "Heroic", "FaZe", "Liquid", "Eternal Fire", "100 Thieves", "9z",
         "M80", "DENDELE CS", "Imperial", "Wildcard", "GOSH", "FlyQuest", "Chinggis Warriors", "Lynn Vision")

# PandaScore tournaments/22378/matches, final nine matches by ascending ID, winner first.
LAST_RESULTS = (("Heroic", "FaZe"), ("Eternal Fire", "M80"), ("PVISION", "Lynn Vision"),
                ("100 Thieves", "9z"), ("Liquid", "Imperial"), ("Wildcard", "Chinggis Warriors"),
                ("Lynn Vision", "Heroic"), ("Imperial", "Wildcard"), ("100 Thieves", "M80"))


def forecast(data, stage, opening):
    matches, maps, _ = load_table()
    cutoff = stage.cutoff
    assert stage.event_id not in set(matches.event_id)
    standings, markets = Standings(data.standings), Markets(data)
    lines = {}
    for m in opening:
        q = markets.quote(m, cutoff)
        if q:
            assert q.observed < cutoff
            lines[m.team_a.name, m.team_b.name] = q.p
            lines[m.team_b.name, m.team_a.name] = 1 - q.p
    rows = []
    for fmt in ("bo1", "bo3"):
        for a, b in combinations(SEEDS, 2):
            rating = standings.estimate(a, b, fmt, cutoff)
            vr = rating[1] if rating else None
            market = lines.get((a, b)) if fmt == "bo1" else None
            p = anchor(market, vr)
            rows.append(dict(match_id=-len(rows)-1, event_id=stage.event_id, event=stage.event,
                             event_begins=cutoff, event_ended=None, in_scope=True, tier="s",
                             is_qualifier=False, cutoff=cutoff, known_finish=None, team_a_won=False,
                             team_a_key=team_key(a), team_b_key=team_key(b), format=fmt, hltv_is_lan=True,
                             market_p=market, vrs_series_p=vr, anchor_p=p if p is not None else 0.5,
                             vrs_points_a=rating[3] if rating else None, vrs_points_b=rating[4] if rating else None))
    future = pd.DataFrame(rows)
    frame = pd.concat([matches, future], ignore_index=True)
    x, kinds = features.compute(frame, maps, features.resolve("baseline"))
    print("Stage cutoff", timeutil.iso(cutoff), "opening lines", len(lines)//2, "of 8")
    probabilities = {"anchor": future.anchor_p, "VRS": future.vrs_series_p.fillna(0.5)}
    for model in ("logreg", "lgbm", "mlp"):
        # Synthetic evaluation rows have no known finish and an open event. The dummy label
        # merely satisfies walk_forward's output schema and never enters a fit or score.
        pred = evaluate.walk_forward(frame, x, kinds, model)
        target = pred.set_index("match_id").loc[future.match_id]
        print(model, "training matches", target.train_matches.iloc[0], "params", target.params.iloc[0])
        probabilities[model] = target.variant
    return probabilities


def matchings(pool):
    if not pool:
        yield ()
        return
    a = pool[0]
    for b in reversed(pool[1:]):
        rest = [t for t in pool[1:] if t != b]
        for tail in matchings(rest):
            yield ((a, b),) + tail


def simulate(values, opening, runs):
    rng = np.random.default_rng(33)
    tables = {n: list(matchings(list(range(n)))) for n in (4, 6, 8)}
    # Published Valve six-team priority order as encoded by sidecar's swiss.rs.
    tables[6] = [tuple((int(s[i])-1, int(s[i+1])-1) for i in (0, 2, 4)) for s in
                 "162534 162435 152634 152436 142635 142536 162345 152346 132645 132546 142356 132456 123645 123546 123456".split()]
    p = np.zeros((2, 16, 16))
    a, b = np.triu_indices(16, 1)
    p[:, a, b] = np.asarray(values).reshape(2, -1)
    p[:, b, a] = 1 - p[:, a, b]
    wins = np.zeros((runs, 16), dtype=int)
    losses = np.zeros_like(wins)
    played = np.zeros((runs, 16, 16), dtype=bool)
    row = np.arange(runs)

    def play(a, b, bo3):
        win = rng.random(runs) < p[int(bo3), a, b]
        wins[row, np.where(win, a, b)] += 1
        losses[row, np.where(win, b, a)] += 1
        played[row, a, b] = played[row, b, a] = True

    for m in opening:
        play(SEEDS.index(m.team_a.name), SEEDS.index(m.team_b.name), False)
    for r in range(1, 5):
        buchholz = (played * (wins - losses)[:, None, :]).sum(axis=2)
        # Freeze all records and pairings at the start of the round.
        pending = []
        for w in range(max(0, r-2), min(2, r)+1):
            l = r-w
            mask = (wins == w) & (losses == l)
            n = int(mask[0].sum())
            order = np.argsort(np.where(mask, -buchholz*16 + np.arange(16), 10000), axis=1)[:, :n]
            choices = np.zeros((runs, n//2, 2), dtype=int)
            unresolved = row
            for pairing in tables[n]:
                if not len(unresolved):
                    break
                pairs = order[unresolved][:, pairing]
                bad = played[unresolved[:, None], pairs[:, :, 0], pairs[:, :, 1]].any(axis=1)
                choices[unresolved[~bad]] = pairs[~bad]
                unresolved = unresolved[bad]
            if len(unresolved):
                choices[unresolved] = order[unresolved][:, tables[n][0]]
            pending.append((choices, max(w, l) == 2))
        for choices, bo3 in pending:
            for a, b in choices.transpose(1, 2, 0):
                play(a, b, bo3)
    return np.array([((wins == 3) & (losses == 0)).mean(axis=0),
                     ((wins == 0) & (losses == 3)).mean(axis=0),
                     ((wins == 3) & (losses > 0)).mean(axis=0)])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--runs", type=int, default=20000)
    parser.add_argument("--pairs", action="store_true", help="also print every stage-start series probability")
    args = parser.parse_args()
    data = archive.load(paths.REPO / "sidecar/tests/fixtures/backtest", with_hltv=False)
    stage = next(s for s in data.stages if s.id == 22378)
    opening = [m for m in stage.matches if m.name.startswith("Round 1:")]
    assert len(opening) == 8 and len(stage.matches) == 24
    forecasts = forecast(data, stage, opening)
    if args.pairs:
        labels = [(fmt, a, b) for fmt in ("bo1", "bo3") for a, b in combinations(SEEDS, 2)]
        pairs = pd.DataFrame(labels, columns=["format", "team_a", "team_b"])
        for model, values in forecasts.items():
            pairs[model] = np.asarray(values)
        print(pairs.to_csv(index=False, float_format="%.6f"))
    wins, losses = dict.fromkeys(SEEDS, 0), dict.fromkeys(SEEDS, 0)
    for m in stage.matches:
        a, b = m.team_a, m.team_b
        winner, loser = (a, b) if a.id == m.winner else (b, a)
        wins[winner.name] += 1
        losses[loser.name] += 1
    for winner, loser in LAST_RESULTS:
        wins[winner] += 1
        losses[loser] += 1
    truth = [{t for t in SEEDS if wins[t] == 3 and losses[t] == 0},
             {t for t in SEEDS if losses[t] == 3 and wins[t] == 0},
             {t for t in SEEDS if wins[t] == 3 and losses[t] > 0}]
    assert list(map(len, truth)) == [2, 2, 6]
    print("Actual (3-0, 0-3, other advance):", [sorted(g) for g in truth])
    for model, values in forecasts.items():
        prob = simulate(values, opening, args.runs)
        slots = np.repeat(np.arange(3), [2, 2, 6])
        # Same maximum expected correct, disjoint 2/2/6 objective as swiss.rs.
        _, teams = linear_sum_assignment(prob[slots], maximize=True)
        picks = [[SEEDS[t] for t in teams[slots == k]] for k in range(3)]
        score = sum(len(set(p) & g) for p, g in zip(picks, truth))
        print(model, *(", ".join(p) for p in picks), score, score >= 5,
              f"{prob[slots, teams].sum():.3f}", sep=" | ")


if __name__ == "__main__":
    main()
