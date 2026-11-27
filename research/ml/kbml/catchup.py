"""Catch-up in KallBet-style Leagues: `sidecar/src/adapters/lab/leagues.rs` ported.

Each trial draws one archived Event (SeededRng, the lab's draw) and replays its scored Matches in
cutoff order. Counterfactual bots (one per policy) face the same opponents and never each other;
one Match Point per correct pick; standings show only results published strictly before a
cutoff; a missing input is a missed pick. Catch-up reverses a favourite of 50-60% (0.40 ≤ p ≤ 0.60,
p ≠ 0.5) whenever the bot trails the leader among the opponents.

`parity()` reproduces `docs/lab/research/catch-up.md` (anchor bots); `compare()` runs bots that use
another probability (the ext forecaster) next to the anchor bots, on the same draws.
"""
from __future__ import annotations

import heapq
import math

import numpy as np

from .stats import SeededRng


def next_f64(rng: SeededRng) -> float:
    return (rng.next_u64() >> 11) * (1.0 / (1 << 53))


def catch_up(p: float, deficit: int) -> bool:
    fav = p >= 0.5
    if deficit > 0 and p != 0.5 and 0.40 <= p <= 0.60:
        return not fav
    return fav


def simulate(games, size: int, noise: float | None, rng: SeededRng, bots):
    """`games`: list of (cutoff, known or None, anchor or None, market or None, won, {name: p}).
    `bots`: [(name of p, catch-up?)]. Opponents: anchor follower with `noise`, or market follower
    (`noise is None`). Returns each bot's place."""
    k = len(bots)
    final = [0] * (k + size - 1)
    visible = [0] * (k + size - 1)
    pending: list = []
    for index, (cut, known, anc, mkt, won, ps) in enumerate(games):
        while pending and pending[0][0] < cut:
            for member in heapq.heappop(pending)[2]:
                visible[member] += 1
        leader = max(visible[k:])
        picks = []
        for name, cu in bots:
            p = ps.get(name) if name != "anchor" else anc
            if p is None or (isinstance(p, float) and math.isnan(p)):
                picks.append(None)
                continue
            deficit = max(leader - visible[len(picks)], 0)
            picks.append(catch_up(p, deficit) if cu else p >= 0.5)
        for _ in range(1, size):
            if noise is None:
                picks.append(None if mkt is None else mkt >= 0.5)
            else:
                picks.append(None if anc is None else (anc >= 0.5) ^ (next_f64(rng) < noise))
        correct = [i for i, pick in enumerate(picks) if pick is not None and pick == won]
        for i in correct:
            final[i] += 1
        if known is not None:
            heapq.heappush(pending, (known, index, correct))
    others = final[k:]
    return [1 + sum(o > final[i] for o in others) for i in range(k)]


def events_from(matches, probs: dict | None = None, scope_only=False):
    """Games per Event as the Rust report builds them (Event id order; stable sort by cutoff)."""
    probs = probs or {}
    out = {}
    rows = matches[matches["in_scope"]] if scope_only else matches
    for r in rows.itertuples(index=False):
        ps = {name: p.get(r.match_id) for name, p in probs.items()}
        out.setdefault(int(r.event_id), []).append((
            int(r.cutoff), None if r.known_finish is None or r.known_finish != r.known_finish else int(r.known_finish),
            None if r.anchor_p != r.anchor_p or r.anchor_p is None else float(r.anchor_p),
            None if r.market_p != r.market_p or r.market_p is None else float(r.market_p),
            bool(r.team_a_won), ps))
    events = []
    for eid in sorted(out):
        events.append(sorted(out[eid], key=lambda g: g[0]))
    return events


def run(events, bots, size, noise, trials=2000, seed=33):
    rng = SeededRng(seed)
    places = np.zeros((trials, len(bots)), dtype=int)
    for t in range(trials):
        e = int(next_f64(rng) * len(events))
        places[t] = simulate(events[e], size, noise, rng, bots)
    return places


def hoeffding(values, low, high):
    n = len(values)
    mean = float(np.mean(values))
    r = (high - low) * math.sqrt(math.log(20.0) / (2 * n))
    return mean, max(mean - r, low), min(mean + r, high)


def parity(matches, trials=2000):
    """(members, noise label) -> (plain first, catch-up first, plain place, catch-up place)."""
    events = events_from(matches)
    res = {}
    for label, noise in (("10%", 0.1), ("25%", 0.25), ("market", None)):
        for size in (4, 12):
            p = run(events, [("anchor", False), ("anchor", True)], size, noise, trials)
            res[(size, label)] = ((p[:, 0] == 1).mean(), (p[:, 1] == 1).mean(), p[:, 0].mean(), p[:, 1].mean())
    return res, len(events)


BOTS = [("anchor", False), ("anchor", True), ("lgbm ext", False), ("lgbm ext", True)]


def compare(matches, probs: dict, trials=2000):
    """Anchor bots and forecaster bots (plain and catch-up) on the same draws of the in-scope
    Events. Rows: (opponents, members) -> per bot (first-place share, expected place) and the paired
    difference forecaster catch-up − anchor catch-up in first place (90% Hoeffding, Monte Carlo)."""
    events = events_from(matches, probs, scope_only=True)
    out = {}
    for label, noise in (("favourite follower, 10% noise", 0.1), ("favourite follower, 25% noise", 0.25),
                         ("market follower", None)):
        for size in (4, 12):
            p = run(events, BOTS, size, noise, trials)
            first = (p == 1).astype(float)
            out[(label, size)] = {
                "bots": [(hoeffding(first[:, i], 0, 1), hoeffding(p[:, i], 1, size)) for i in range(len(BOTS))],
                "diff_catchup": hoeffding(first[:, 3] - first[:, 1], -1, 1),
                "diff_plain": hoeffding(first[:, 2] - first[:, 0], -1, 1),
            }
    return out, len(events)


def markdown(res, n_events) -> str:
    lines = [f"{n_events} in-scope Events, 2000 paired trials, seed 33. Brackets: 90% Hoeffding (Monte Carlo "
             "only, conditional on these Events and opponents).", "",
             "| Opponents | Members | Anchor plain | Anchor catch-up | lgbm-ext plain | lgbm-ext catch-up "
             "| Δ first, lgbm-ext − anchor (plain) | Δ first, lgbm-ext − anchor (catch-up) |",
             "|---|---:|---|---|---|---|---|---|"]
    for (label, size), r in res.items():
        cells = [f"{f[0]:.3f} (place {pl[0]:.2f})" for f, pl in r["bots"]]
        dp, dc = r["diff_plain"], r["diff_catchup"]
        lines.append(f"| {label} | {size} | " + " | ".join(cells) +
                     f" | {dp[0]:+.3f} [{dp[1]:+.3f}, {dp[2]:+.3f}] | {dc[0]:+.3f} [{dc[1]:+.3f}, {dc[2]:+.3f}] |")
    return "\n".join(lines)
