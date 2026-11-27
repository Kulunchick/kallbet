"""Feature groups: one class per group, registered by name.

A group learns only from what `Timeline.advance_to(cutoff)` releases to `update` (results and
played maps whose known finish is strictly before the Match being described) and describes a
Match from its `MatchInfo` (pre-match columns only). So no group can read a later result.

Every column is declared `anti` (antisymmetric: it changes sign when team A and team B are
swapped, e.g. a rating difference) or `sym` (unchanged by the swap, e.g. the format). The models
use this to train on both orientations and to give exactly p(A) = 1 − p(B).

To add a group: subclass `Group`, set `name` and `columns`, implement `update` (optional) and
`features`, decorate with `@register`. Then `--features baseline,<name>` picks it up.
"""
from __future__ import annotations

import math
from collections import defaultdict, deque

import numpy as np
import pandas as pd

from .anchor import logit, series_probability
from .history import History, MatchInfo, Timeline, match_infos

REGISTRY: dict[str, type["Group"]] = {}

#: Named sets for `--features`.
SETS = {
    "anchor": ["anchor"],
    "baseline": ["anchor", "elo", "map_elo", "form", "h2h", "rest", "lan", "format"],
    "no_anchor": ["elo", "map_elo", "form", "h2h", "rest", "lan", "format"],
    "hltv": ["rank", "opp_form", "past_form", "hltv_h2h", "players", "map_pool", "sides", "standin", "roster",
             "stakes"],
}
SETS["full"] = SETS["baseline"] + SETS["hltv"]


def register(cls):
    REGISTRY[cls.name] = cls
    return cls


class Group:
    name = ""
    columns: dict[str, str] = {}

    def update(self, new) -> None:
        """`new`: a `history.Release` (results, maps, records) known before the next cutoff."""
        pass

    def features(self, m: MatchInfo) -> dict:
        raise NotImplementedError


def _num(v):
    if v is None:
        return None
    try:
        if pd.isna(v):
            return None
    except TypeError:
        pass
    return float(v)


# ------------------------------------------------------------------ anchor inputs

@register
class Anchor(Group):
    """The anchor's own inputs: market line, Valve rating, the blend."""
    name = "anchor"
    columns = {"logit_anchor": "anti", "logit_market": "anti", "has_market": "sym",
               "logit_vrs": "anti", "has_vrs": "sym", "vrs_points_diff": "anti"}

    def features(self, m):
        mk, vr, an = _num(m["market_p"]), _num(m["vrs_series_p"]), _num(m["anchor_p"])
        pa, pb = _num(m["vrs_points_a"]), _num(m["vrs_points_b"])
        return {
            "logit_anchor": logit(an) if an is not None else 0.0,
            "logit_market": logit(mk) if mk is not None else 0.0,
            "has_market": float(mk is not None),
            "logit_vrs": logit(vr) if vr is not None else 0.0,
            "has_vrs": float(vr is not None),
            "vrs_points_diff": (pa - pb) / 100.0 if pa is not None and pb is not None else 0.0,
        }


# ------------------------------------------------------------------ Elo on series results

@register
class Elo(Group):
    """Series Elo per team key (K grows with the map margin)."""
    name = "elo"
    columns = {"elo_diff": "anti", "elo_logit": "anti", "elo_n_min": "sym"}
    K, START = 32.0, 1500.0

    def __init__(self):
        self.r = defaultdict(lambda: self.START)
        self.n = defaultdict(int)

    def update(self, new):
        results, maps = new.results, new.maps
        for a, b, won, ma, mb in zip(results.team_a_key, results.team_b_key, results.team_a_won,
                                     results.maps_a, results.maps_b):
            ea = 1.0 / (1.0 + 10 ** ((self.r[b] - self.r[a]) / 400.0))
            margin = 1.0
            if not pd.isna(ma) and not pd.isna(mb):
                margin = 1.0 + 0.25 * max(abs(int(ma) - int(mb)) - 1, 0)
            delta = self.K * margin * ((1.0 if won else 0.0) - ea)
            self.r[a] += delta
            self.r[b] -= delta
            self.n[a] += 1
            self.n[b] += 1

    def features(self, m):
        a, b = m["team_a_key"], m["team_b_key"]
        d = self.r[a] - self.r[b]
        return {"elo_diff": d / 100.0, "elo_logit": d * math.log(10) / 400.0,
                "elo_n_min": math.log1p(min(self.n[a], self.n[b]))}


# ------------------------------------------------------------------ map-level Elo (HLTV maps)

@register
class MapElo(Group):
    """Elo on single maps: overall per team, and per (team, map); the pool feature averages the
    per-map difference over maps both teams have played."""
    name = "map_elo"
    columns = {"map_elo_diff": "anti", "map_elo_series_logit": "anti", "map_pool_diff": "anti",
               "map_pool_n": "sym"}
    K, KM = 20.0, 24.0

    def __init__(self):
        self.r = defaultdict(lambda: 1500.0)
        self.rm = defaultdict(lambda: 1500.0)
        self.nm = defaultdict(int)
        self.pool = defaultdict(set)  # team -> maps it has played

    def update(self, new):
        results, maps = new.results, new.maps
        if maps is None or not len(maps):
            return
        for a, b, mp, won in zip(maps.team_a_key, maps.team_b_key, maps.map_name, maps.a_won):
            s = 1.0 if won else 0.0
            ea = 1.0 / (1.0 + 10 ** ((self.r[b] - self.r[a]) / 400.0))
            self.r[a] += self.K * (s - ea)
            self.r[b] -= self.K * (s - ea)
            ka, kb = (a, mp), (b, mp)
            # per-map rating as an offset over the overall rating
            em = 1.0 / (1.0 + 10 ** ((self.rm[kb] - self.rm[ka]) / 400.0))
            self.rm[ka] += self.KM * (s - em)
            self.rm[kb] -= self.KM * (s - em)
            self.nm[ka] += 1
            self.nm[kb] += 1
            self.pool[a].add(mp)
            self.pool[b].add(mp)

    def features(self, m):
        a, b = m["team_a_key"], m["team_b_key"]
        d = self.r[a] - self.r[b]
        p = 1.0 / (1.0 + 10 ** (-d / 400.0))
        maps = sorted(self.pool[a] & self.pool[b])
        pool = [self.rm[(a, mp)] - self.rm[(b, mp)] for mp in maps]
        return {"map_elo_diff": d / 100.0,
                "map_elo_series_logit": logit(series_probability(p, m["format"])),
                "map_pool_diff": float(np.mean(pool)) / 100.0 if pool else 0.0,
                "map_pool_n": float(len(pool))}


# ------------------------------------------------------------------ recent form

@register
class Form(Group):
    """Smoothed win share of the last 10 Matches and mean round difference of the last 15 maps."""
    name = "form"
    columns = {"form10_diff": "anti", "round_diff15": "anti"}

    def __init__(self):
        self.res = defaultdict(lambda: deque(maxlen=10))
        self.rounds = defaultdict(lambda: deque(maxlen=15))

    def update(self, new):
        results, maps = new.results, new.maps
        for a, b, won in zip(results.team_a_key, results.team_b_key, results.team_a_won):
            self.res[a].append(1.0 if won else 0.0)
            self.res[b].append(0.0 if won else 1.0)
        if maps is not None and len(maps):
            for a, b, ra, rb in zip(maps.team_a_key, maps.team_b_key, maps.rounds_a, maps.rounds_b):
                if pd.isna(ra) or pd.isna(rb):
                    continue
                self.rounds[a].append(float(ra) - float(rb))
                self.rounds[b].append(float(rb) - float(ra))

    def _share(self, t):
        r = self.res[t]
        return (sum(r) + 2.5) / (len(r) + 5.0)

    def _rd(self, t):
        r = self.rounds[t]
        return sum(r) / (len(r) + 3.0)

    def features(self, m):
        a, b = m["team_a_key"], m["team_b_key"]
        return {"form10_diff": self._share(a) - self._share(b), "round_diff15": (self._rd(a) - self._rd(b)) / 5.0}


# ------------------------------------------------------------------ head to head

@register
class HeadToHead(Group):
    name = "h2h"
    columns = {"h2h_share": "anti", "h2h_n": "sym"}

    def __init__(self):
        self.w = defaultdict(float)  # (x, y) -> wins of x over y

    def update(self, new):
        results, maps = new.results, new.maps
        for a, b, won in zip(results.team_a_key, results.team_b_key, results.team_a_won):
            if won:
                self.w[(a, b)] += 1
            else:
                self.w[(b, a)] += 1

    def features(self, m):
        a, b = m["team_a_key"], m["team_b_key"]
        wa, wb = self.w[(a, b)], self.w[(b, a)]
        return {"h2h_share": (wa + 1.0) / (wa + wb + 2.0) - 0.5, "h2h_n": math.log1p(wa + wb)}


# ------------------------------------------------------------------ rest and load

@register
class Rest(Group):
    """Days since each team's last known result (capped at 30) and results in the last 7 days."""
    name = "rest"
    columns = {"rest_diff": "anti", "load7_diff": "anti"}
    DAY = 86_400_000_000

    def __init__(self):
        self.finishes = defaultdict(list)

    def update(self, new):
        results, maps = new.results, new.maps
        for a, b, f in zip(results.team_a_key, results.team_b_key, results.known_finish):
            self.finishes[a].append(int(f))
            self.finishes[b].append(int(f))

    def _rest(self, t, cutoff):
        f = self.finishes[t]
        return min((cutoff - f[-1]) / self.DAY, 30.0) if f else 30.0

    def _load(self, t, cutoff):
        return float(sum(1 for x in reversed(self.finishes[t][-20:]) if cutoff - x <= 7 * self.DAY))

    def features(self, m):
        a, b, c = m["team_a_key"], m["team_b_key"], m.cutoff
        return {"rest_diff": (self._rest(a, c) - self._rest(b, c)) / 10.0,
                "load7_diff": self._load(a, c) - self._load(b, c)}


# ------------------------------------------------------------------ venue

@register
class Lan(Group):
    """Whether the Match is on LAN (HLTV page; unknown → 0.5) and each team's smoothed win share
    in earlier Matches on the same kind of venue."""
    name = "lan"
    columns = {"is_lan": "sym", "venue_form_diff": "anti"}

    def __init__(self):
        self.rec = defaultdict(lambda: [0.0, 0.0])  # (team, lan) -> [wins, n]

    def update(self, new):
        results, maps = new.results, new.maps
        for a, b, won, lan in zip(results.team_a_key, results.team_b_key, results.team_a_won, results.hltv_is_lan):
            if lan is None or pd.isna(lan):
                continue
            for t, w in ((a, won), (b, not won)):
                r = self.rec[(t, bool(lan))]
                r[0] += 1.0 if w else 0.0
                r[1] += 1.0

    def features(self, m):
        lan = m["hltv_is_lan"]
        if lan is None or pd.isna(lan):
            return {"is_lan": 0.5, "venue_form_diff": 0.0}
        a, b = self.rec[(m["team_a_key"], bool(lan))], self.rec[(m["team_b_key"], bool(lan))]
        sa, sb = (a[0] + 2.5) / (a[1] + 5.0), (b[0] + 2.5) / (b[1] + 5.0)
        return {"is_lan": float(bool(lan)), "venue_form_diff": sa - sb}


@register
class FormatGroup(Group):
    name = "format"
    columns = {"bo1": "sym", "bo3": "sym", "bo5": "sym"}

    def features(self, m):
        f = m["format"]
        return {"bo1": float(f == "bo1"), "bo3": float(f == "bo3"), "bo5": float(f == "bo5")}


# ------------------------------------------------------------------ driver

def resolve(spec: str) -> list[str]:
    names: list[str] = []
    for part in spec.split(","):
        part = part.strip()
        for n in SETS.get(part, [part]):
            if n not in REGISTRY:
                raise SystemExit(f"unknown feature group {n!r}; known: {sorted(REGISTRY)} and sets {sorted(SETS)}")
            if n not in names:
                names.append(n)
    return names


def compute(matches: pd.DataFrame, maps: pd.DataFrame, groups: list[str], hltv=None
            ) -> tuple[pd.DataFrame, dict[str, str]]:
    """One feature row per Match (index = match_id), each computed at that Match's cutoff from
    what the timeline released. `hltv`: `(pre by match_id, records frame)` from `hltv_data.build`,
    for the HLTV groups (their rank columns must already be on `matches`). Returns the frame and
    {column: anti|sym}."""
    inst = [REGISTRY[g]() for g in groups]
    kinds = {c: k for g in inst for c, k in g.columns.items()}
    pre, records = hltv if hltv is not None else ({}, None)
    timeline = Timeline(History(matches, maps, records))
    infos = match_infos(matches)
    for info in infos:
        info.row["hltv_pre"] = pre.get(info.row["match_id"])
    infos.sort(key=lambda i: (i.cutoff, i.row["match_id"]))
    rows, ids = [], []
    for info in infos:
        new = timeline.advance_to(info.cutoff)
        if len(new.results) or len(new.maps) or len(new.records):
            for g in inst:
                g.update(new)
        row = {}
        for g in inst:
            row.update(g.features(info))
        rows.append(row)
        ids.append(info.row["match_id"])
    frame = pd.DataFrame(rows, index=pd.Index(ids, name="match_id"))
    return frame[list(kinds)], kinds


from . import features_hltv  # noqa: E402,F401  (registers the HLTV groups)
