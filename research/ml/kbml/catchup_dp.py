"""Catch-up by backward induction (Bellman DP) in KallBet-style Leagues. See CATCHUP.md.

The replay is `kbml.catchup.simulate` generalised to policies that see the bot's published
standing, the opponents' published Points and how many results are still unpublished. Random draws
(Event, opponents' noise) are consumed exactly as in `catchup.run`, so every policy faces the same
opponents as the plain and 40-60 bots there.

DP state before a Match: r = results not yet published (this Match included), d = bot − leader
(published Points, clipped to ±D), c0 = opponents tied for the lead, c1 = opponents one behind
(the rest count as two behind). V_r(d, c0, c1) = P(final Points ≥ every opponent's), ties shared as
in `scoring::leaderboard`. Each opponent independently takes the consensus side and reverses it
with probability `eps` (0 for the market follower); no consensus means every opponent misses.
"""
from __future__ import annotations

import heapq
import math
from dataclasses import dataclass

import numpy as np

from .catchup import next_f64
from .stats import SeededRng

D = 10  # |bot − leader| beyond this is clipped


# ---------------------------------------------------------------- opponent transitions

def states(n_opp: int) -> list[tuple[int, int]]:
    return [(c0, c1) for c0 in range(1, n_opp + 1) for c1 in range(0, n_opp - c0 + 1)]


def _binom(n: int, p: float) -> np.ndarray:
    return np.array([math.comb(n, k) * p ** k * (1 - p) ** (n - k) for k in range(n + 1)])


def transitions(n_opp: int, eps: float):
    """{outcome: (P_inc0, P_inc1)}, outcome 1 = consensus side wins, 0 = it loses; P_inc[s, s'] =
    P(leader gains inc and the field moves from state s to s')."""
    st = states(n_opp)
    index = {s: i for i, s in enumerate(st)}
    out = {}
    for outcome in (1, 0):
        pc = 1 - eps if outcome else eps
        P = np.zeros((2, len(st), len(st)))
        for i, (c0, c1) in enumerate(st):
            c2 = n_opp - c0 - c1
            b0, b1, b2 = _binom(c0, pc), _binom(c1, pc), _binom(c2, pc)
            for x0 in range(c0 + 1):
                for x1 in range(c1 + 1):
                    if x0 > 0:  # leader +1; the rest of the old top and the risers from L−1 are one behind
                        P[1, i, index[(x0, c0 - x0 + x1)]] += b0[x0] * b1[x1]
                        continue
                    for x2 in range(c2 + 1):
                        if x1 > 0:
                            ns = (c0 + x1, c1 - x1 + x2)
                        else:
                            ns = (c0, c1 + x2)
                        P[0, i, index[ns]] += b0[x0] * b1[x1] * b2[x2]
        out[outcome] = P
    return out, index


def _shift(V: np.ndarray, k: int) -> np.ndarray:
    """V with d → clip(d + k)."""
    if k == 0:
        return V
    idx = np.clip(np.arange(V.shape[0]) + k, 0, V.shape[0] - 1)
    return V[idx]


@dataclass
class Tables:
    V: list[np.ndarray]          # V[r]: (2D+1, S)
    T: dict
    index: dict

    def after(self, r: int, d: int, s: int, outcome: int, ours: int) -> float:
        """E[V_{r-1} | this Match's outcome and our correctness] from state (d, s)."""
        V = self.V[min(r - 1, len(self.V) - 1)]
        P = self.T[outcome]
        total = 0.0
        for inc in (0, 1):
            di = int(np.clip(d + ours - inc + D, 0, 2 * D))
            total += float(P[inc, s] @ V[di])
        return total

    def values(self, r: int, d: int, c0: int, c1: int, q: float) -> tuple[float, float]:
        """(V if we take the consensus side, V if we take the other side), q = P(consensus wins)."""
        s = self.index[(c0, c1)]
        cons = q * self.after(r, d, s, 1, 1) + (1 - q) * self.after(r, d, s, 0, 0)
        other = q * self.after(r, d, s, 1, 0) + (1 - q) * self.after(r, d, s, 0, 1)
        return cons, other


def build(n_opp: int, eps: float, qs: np.ndarray, ws: np.ndarray, w_none: float, f_none: float,
          rmax: int) -> Tables:
    """Backward recursion. `qs`, `ws`: Match types (P(consensus side wins) under our p) and weights;
    `w_none`: share of Matches without a consensus, where our favourite wins with mean prob f_none."""
    T, index = transitions(n_opp, eps)
    S = len(index)
    V0 = np.zeros((2 * D + 1, S))
    V0[D:, :] = 1.0
    Vs = [V0]
    qs = np.asarray(qs, float)[:, None, None]
    ws = np.asarray(ws, float)[:, None, None]
    for _ in range(rmax):
        V = Vs[-1]
        U = {}
        for outcome in (1, 0):
            for ours in (0, 1):
                U[outcome, ours] = sum(_shift(V, ours - inc) @ T[outcome][inc].T for inc in (0, 1))
        cons = qs * U[1, 1] + (1 - qs) * U[0, 0]
        other = qs * U[1, 0] + (1 - qs) * U[0, 1]
        Vn = (ws * np.maximum(cons, other)).sum(axis=0)
        Vn = Vn + w_none * (f_none * _shift(V, 1) + (1 - f_none) * V)
        Vs.append(Vn / (ws.sum() + w_none))
    return Tables(Vs, T, index)


def match_types(games, src: str, follower: str, bins: int = 40):
    """Empirical Match-type distribution from games (tuples as in `catchup.events_from`)."""
    qs, none_f = [], []
    for _, _, anc, mkt, _, ps in games:
        p = anc if src == "anchor" else ps.get(src)
        if p is None or (isinstance(p, float) and math.isnan(p)):
            continue
        cons = anc if follower == "favourite" else mkt
        if cons is None:
            none_f.append(max(p, 1 - p))
            continue
        qs.append(p if cons >= 0.5 else 1 - p)
    edges = np.linspace(0, 1, bins + 1)
    hist, _ = np.histogram(qs, bins=edges)
    mids = (edges[:-1] + edges[1:]) / 2
    # bin means, not midpoints, where a bin has Matches
    sums, _ = np.histogram(qs, bins=edges, weights=qs)
    q = np.where(hist > 0, sums / np.maximum(hist, 1), mids)
    keep = hist > 0
    return q[keep], hist[keep].astype(float), float(len(none_f)), float(np.mean(none_f)) if none_f else 0.75


PRIOR = (np.linspace(0.1, 0.9, 17), np.ones(17), 0.0, 0.75)  # uniform 50-90% favourite, either side


# ---------------------------------------------------------------- policies

class Plain:
    def __init__(self, src):
        self.src = src

    def setup(self, event):
        pass

    def pick(self, p, ctx):
        return p >= 0.5


class Window:
    """Reverse a favourite of at most `hi` whenever trailing (hi = 0.60 is catch-up.md's rule)."""

    def __init__(self, src, hi=0.60, per_event=None):
        self.src, self.hi, self.per_event, self.cur = src, hi, per_event, hi

    def setup(self, event):
        self.cur = self.per_event.get(event, self.hi) if self.per_event is not None else self.hi

    def pick(self, p, ctx):
        fav = p >= 0.5
        if ctx.deficit > 0 and p != 0.5 and 1 - self.cur <= p <= self.cur:
            return not fav
        return fav


class DP:
    def __init__(self, src, tables: dict):
        self.src, self.tables, self.t = src, tables, None
        self.flips = 0

    def setup(self, event):
        self.t = self.tables[event]

    def pick(self, p, ctx):
        fav = p >= 0.5
        if ctx.consensus is None or ctx.r < 1:
            return fav
        q = p if ctx.consensus else 1 - p
        d = max(min(ctx.own - ctx.leader, D), -D)
        cons, other = self.t.values(ctx.r, d, ctx.c0, ctx.c1, q)
        cons_is_fav = ctx.consensus == fav
        v_fav, v_dog = (cons, other) if cons_is_fav else (other, cons)
        if v_dog > v_fav + 1e-12:
            self.flips += 1
            return not fav
        return fav


@dataclass
class Ctx:
    own: int
    leader: int
    c0: int
    c1: int
    deficit: int
    r: int
    consensus: bool | None


# ---------------------------------------------------------------- replay

def simulate(games, size, noise, rng, bots, event=None):
    """`catchup.simulate` with policy objects; same random draws. Returns each bot's place."""
    for b in bots:
        b.setup(event)
    k = len(bots)
    final = [0] * (k + size - 1)
    visible = [0] * (k + size - 1)
    pending: list = []
    published = 0
    n = len(games)
    for index, (cut, known, anc, mkt, won, ps) in enumerate(games):
        while pending and pending[0][0] < cut:
            for member in heapq.heappop(pending)[2]:
                visible[member] += 1
            published += 1
        opp = visible[k:]
        leader = max(opp)
        c0 = sum(o == leader for o in opp)
        c1 = sum(o == leader - 1 for o in opp)
        cons_p = anc if noise is not None else mkt
        consensus = None if cons_p is None else cons_p >= 0.5
        picks = []
        for i, b in enumerate(bots):
            p = anc if b.src == "anchor" else ps.get(b.src)
            if p is None or (isinstance(p, float) and math.isnan(p)):
                picks.append(None)
                continue
            ctx = Ctx(visible[i], leader, c0, c1, max(leader - visible[i], 0), n - published, consensus)
            picks.append(b.pick(p, ctx))
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
        else:
            pass  # a missing end time is never published: it stays among the r unpublished results
    others = final[k:]
    return [1 + sum(o > final[i] for o in others) for i in range(k)]


def run(events, ids, bots, size, noise, trials=2000, seed=33):
    """Returns (places (trials, bots), drawn Event index per trial)."""
    rng = SeededRng(seed)
    places = np.zeros((trials, len(bots)), dtype=int)
    drawn = np.zeros(trials, dtype=int)
    for t in range(trials):
        e = int(next_f64(rng) * len(events))
        drawn[t] = e
        places[t] = simulate(events[e], size, noise, rng, bots, ids[e])
    return places, drawn
