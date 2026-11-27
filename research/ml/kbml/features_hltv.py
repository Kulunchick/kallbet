"""Feature groups from the HLTV pages (ml-notes/hltv-features.md), one class per group.

Pre-match data of the Match itself comes from `m["hltv_pre"]` (its own page, oriented so side 0
is team A, rows not strictly earlier already dropped) and the rank columns. Everything a Match
produced (stats, maps, vetoes, half scores, lineups) reaches a group only through `update`, i.e.
for later Matches, once the Match's known finish is before their cutoff. No group reads the
Match's own veto, map picks or any post-veto datum.
"""
from __future__ import annotations

import math
from collections import defaultdict, deque

import numpy as np

from .anchor import logit, series_probability
from .features import Group, register
from .teams import team_key

DAY = 86_400_000_000
UNRANKED = 150


def _s(x, default=0.0):
    try:
        if x is None or (isinstance(x, float) and math.isnan(x)):
            return default
        import pandas as pd
        if pd.isna(x):
            return default
    except (TypeError, ValueError):
        pass
    return float(x)


def _rank(x):
    v = _s(x, None) if x is not None else None
    return v


def _lineup(m, side):
    pre = m["hltv_pre"]
    if not pre or len(pre.get("lineups") or []) != 2:
        return None
    return pre["lineups"][side]


def _logit_anchor(m):
    a = m["anchor_p"]
    return logit(float(a)) if a is not None and not (isinstance(a, float) and math.isnan(a)) else 0.0


# ------------------------------------------------------------------ HLTV rank

@register
class Rank(Group):
    """HLTV world rank (weekly ranking known before the cutoff): difference, log ratio, momentum."""
    name = "rank"
    columns = {"rank_diff": "anti", "rank_log_ratio": "anti", "rank_mom28_diff": "anti",
               "rank_mom84_diff": "anti", "has_rank": "sym"}

    def features(self, m):
        ra, rb = _rank(m["hltv_rank_a"]), _rank(m["hltv_rank_b"])
        has = ra is not None and rb is not None
        a, b = min(ra or UNRANKED, UNRANKED), min(rb or UNRANKED, UNRANKED)

        def mom(now, back):
            now, back = _rank(m[now]), _rank(m[back])
            return (min(back, UNRANKED) - min(now, UNRANKED)) if now is not None and back is not None else 0.0

        return {"rank_diff": (b - a) / 10.0, "rank_log_ratio": math.log(b) - math.log(a),
                "rank_mom28_diff": (mom("hltv_rank_a", "hltv_rank28_a") - mom("hltv_rank_b", "hltv_rank28_b")) / 10.0,
                "rank_mom84_diff": (mom("hltv_rank_a", "hltv_rank84_a") - mom("hltv_rank_b", "hltv_rank84_b")) / 10.0,
                "has_rank": float(has)}


# ------------------------------------------------------------------ opponent-weighted form

@register
class OppForm(Group):
    """Last 15 results weighted by the opponent's HLTV rank at that time (1/log2(1+rank)), and the
    share won against top-20 opponents."""
    name = "opp_form"
    columns = {"owf_diff": "anti", "top20_diff": "anti"}

    def __init__(self):
        self.res = defaultdict(lambda: deque(maxlen=15))
        self.top = defaultdict(lambda: deque(maxlen=10))

    def update(self, new):
        r = new.results
        if not len(r) or "hltv_rank_a" not in r:
            return
        for a, b, won, ra, rb in zip(r.team_a_key, r.team_b_key, r.team_a_won, r.hltv_rank_a, r.hltv_rank_b):
            for t, w, opp in ((a, won, rb), (b, not won, ra)):
                rank = min(_s(opp, UNRANKED), UNRANKED)
                self.res[t].append((1.0 if w else 0.0, 1.0 / math.log2(1.0 + rank)))
                if rank <= 20:
                    self.top[t].append(1.0 if w else 0.0)

    def _owf(self, t):
        d = self.res[t]
        return (sum(w * x for x, w in d) + 0.5) / (sum(w for _, w in d) + 1.0)

    def _top(self, t):
        d = self.top[t]
        return (sum(d) + 1.0) / (len(d) + 2.0)

    def features(self, m):
        a, b = m["team_a_key"], m["team_b_key"]
        return {"owf_diff": self._owf(a) - self._owf(b), "top20_diff": self._top(a) - self._top(b)}


# ------------------------------------------------------------------ past-matches widget (team / core)

@register
class PastForm(Group):
    """The page's "past 3 months" lists (team and core): win share, last 5, streak, map margin."""
    name = "past_form"
    columns = {f"pm_{k}_{f}": "anti" for k in ("team", "core") for f in ("share", "last5", "streak", "margin")}
    columns.update({"pm_n_min": "sym", "has_past": "sym"})

    @staticmethod
    def stats(rows):
        n = len(rows)
        w = sum(r["won"] for r in rows)
        last5 = rows[:5]
        streak = 0
        for r in rows:
            if not r["won"]:
                break
            streak += 1
        margins = []
        for r in rows:
            if r["for"] is None:
                continue
            if r["token"] in ("bo3", "bo5"):
                margins.append(r["for"] - r["against"])
            else:
                margins.append(1.0 if r["won"] else -1.0)
        return {"share": (w + 2.0) / (n + 4.0), "last5": (sum(r["won"] for r in last5) + 1.0) / (len(last5) + 2.0),
                "streak": min(streak, 10) / 5.0, "margin": sum(margins) / (len(margins) + 2.0), "n": n}

    def features(self, m):
        pre = m["hltv_pre"]
        out = {c: 0.0 for c in self.columns}
        if not pre or len(pre["past"].get("team") or []) != 2:
            return out
        nmin = 1e9
        for kind in ("team", "core"):
            sides = pre["past"].get(kind) or []
            if len(sides) != 2:
                continue
            a, b = self.stats(sides[0]), self.stats(sides[1])
            for f in ("share", "last5", "streak", "margin"):
                out[f"pm_{kind}_{f}"] = a[f] - b[f]
            nmin = min(nmin, a["n"], b["n"])
        out["pm_n_min"] = math.log1p(nmin if nmin < 1e9 else 0)
        out["has_past"] = 1.0
        return out


# ------------------------------------------------------------------ head to head (page listing)

@register
class HltvH2H(Group):
    """Earlier maps between the two (page listing, strictly before the cutoff): share of maps won,
    and a recency- and lineup-overlap-weighted share."""
    name = "hltv_h2h"
    columns = {"hh_share": "anti", "hh_weighted": "anti", "hh_n": "sym"}

    def features(self, m):
        pre = m["hltv_pre"]
        out = {"hh_share": 0.0, "hh_weighted": 0.0, "hh_n": 0.0}
        if not pre or not pre.get("h2h") or len(pre.get("lineups") or []) != 2:
            return out
        ida = pre["page_team_ids"][0]
        nick = [{p["nick"].lower() for p in side} for side in pre["lineups"]]
        won = n = 0
        sw = swin = 0.0
        for r in pre["h2h"]["rows"]:
            p1 = {x.lower() for x in r["players1"]}
            p2 = {x.lower() for x in r["players2"]}
            if r["team1"] == ida:
                a_is_1 = True
            elif r["team2"] == ida:
                a_is_1 = False
            else:
                o1, o2 = len(p1 & nick[0]), len(p2 & nick[0])
                if o1 == o2:
                    continue
                a_is_1 = o1 > o2
            a_won = r["winner1"] if a_is_1 else r["winner2"]
            if not (r["winner1"] or r["winner2"]):
                continue
            n += 1
            won += a_won
            pa, pb = (p1, p2) if a_is_1 else (p2, p1)
            age = max(m.cutoff - r["unix_ms"] * 1000, 0) / DAY
            w = 0.5 ** (age / 180.0) * (0.2 + len(pa & nick[0]) / 5.0 * len(pb & nick[1]) / 5.0)
            sw += w
            swin += w * (1.0 if a_won else 0.0)
        if n:
            out = {"hh_share": (won + 1.0) / (n + 2.0) - 0.5, "hh_weighted": (swin + 0.5) / (sw + 1.0) - 0.5,
                   "hh_n": math.log1p(n)}
        return out


# ------------------------------------------------------------------ player ratings of the lineup

def _decayed(entries, cutoff, prior, k=2.0, half_life=120.0):
    sw = sx = 0.0
    for t, x in entries:
        if x is None:
            continue
        w = 0.5 ** (max(cutoff - t, 0) / DAY / half_life)
        sw += w
        sx += w * x
    return (sx + k * prior) / (sw + k), sw


@register
class Players(Group):
    """Each lineup player's earlier HLTV stats (Rating 3.0, swing, eADR; time-decayed, shrunk to a
    prior), aggregated over the five: mean, floor and star spread."""
    name = "players"
    columns = {"pl_rating_diff": "anti", "pl_swing_diff": "anti", "pl_eadr_diff": "anti",
               "pl_min_diff": "anti", "pl_spread_diff": "anti", "pl_cover_min": "sym"}

    def __init__(self):
        self.p = defaultdict(lambda: deque(maxlen=40))

    def update(self, new):
        for rec in new.records.get("rec", []) if len(new.records) else []:
            t = rec["started_at"]
            for team in rec["overall"]:
                for pl in team.get("players") or []:
                    self.p[pl["player_id"]].append((t, pl.get("rating"), pl.get("swing_pct"), pl.get("eadr")))

    def _agg(self, lineup, cutoff):
        if not lineup:
            return None
        rs, ss, es, cov = [], [], [], 0
        for pl in lineup:
            h = self.p.get(pl["id"], ())
            rs.append(_decayed([(t, r) for t, r, _, _ in h], cutoff, 1.0)[0])
            ss.append(_decayed([(t, s) for t, _, s, _ in h], cutoff, 0.0)[0])
            es.append(_decayed([(t, e) for t, _, _, e in h], cutoff, 75.0)[0])
            cov += bool(h)
        return {"r": np.mean(rs), "s": np.mean(ss), "e": np.mean(es), "min": min(rs),
                "spread": max(rs) - np.mean(rs), "cov": cov / max(len(lineup), 1)}

    def features(self, m):
        a, b = self._agg(_lineup(m, 0), m.cutoff), self._agg(_lineup(m, 1), m.cutoff)
        if a is None or b is None:
            return {c: 0.0 for c in self.columns}
        return {"pl_rating_diff": (a["r"] - b["r"]) * 10, "pl_swing_diff": a["s"] - b["s"],
                "pl_eadr_diff": (a["e"] - b["e"]) / 10.0, "pl_min_diff": (a["min"] - b["min"]) * 10,
                "pl_spread_diff": (a["spread"] - b["spread"]) * 10, "pl_cover_min": min(a["cov"], b["cov"])}


# ------------------------------------------------------------------ map pool and pre-veto edge

@register
class MapPool(Group):
    """Expected pre-veto map edge. From each team's earlier vetoes (ban and pick shares per map) a
    map's chance to be played; from earlier maps each team's decayed win share per map and the
    lineup's per-map player rating. Edge = sum over the active pool of p(map) × difference; the
    log5 map probability averaged over the pool goes through the series conversion. Uses only
    earlier Matches' vetoes, never this Match's."""
    name = "map_pool"
    columns = {"mp_edge": "anti", "mp_series_logit": "anti", "mp_depth_diff": "anti",
               "mp_player_edge": "anti", "mp_vetoes_min": "sym"}

    def __init__(self):
        self.vetoes = defaultdict(lambda: deque(maxlen=25))
        self.res = defaultdict(lambda: deque(maxlen=25))  # (team, map) -> (t, won)
        self.seen: dict[str, int] = {}
        self.pm = defaultdict(lambda: deque(maxlen=15))  # (player, map) -> (t, rating)

    def update(self, new):
        if not len(new.records):
            return
        for rec in new.records["rec"]:
            t = rec["started_at"]
            keys = rec["keys"]
            names = {}
            for side, team in enumerate(rec["overall"][:2]):
                names[team_key(team.get("team_name") or "")] = keys[side] if side < 2 else None
            per_team = {k: {"removed": set(), "picked": set()} for k in keys}
            for step in rec["veto"]:
                k = team_key(step.get("team") or "")
                k = k if k in per_team else names.get(k)
                if step.get("map_name"):
                    self.seen[step["map_name"]] = max(self.seen.get(step["map_name"], 0), t)
                if k in per_team and step.get("action") in ("removed", "picked"):
                    per_team[k][step["action"]].add(step["map_name"])
            if rec["veto"]:
                for k in keys:
                    self.vetoes[k].append(per_team[k])
            for mp in rec["maps"]:
                if not mp.get("played") or mp.get("winner") not in ("a", "b"):
                    continue
                name = mp["map_name"]
                self.seen[name] = max(self.seen.get(name, 0), t)
                self.res[(keys[0], name)].append((t, 1.0 if mp["winner"] == "a" else 0.0))
                self.res[(keys[1], name)].append((t, 0.0 if mp["winner"] == "a" else 1.0))
            for pm in rec["per_map"]:
                for team in pm.get("teams") or []:
                    for pl in team.get("players") or []:
                        if pl.get("rating") is not None:
                            self.pm[(pl["player_id"], pm.get("map_name"))].append((t, pl["rating"]))

    def _share(self, team, mp, action):
        v = self.vetoes[team]
        return (sum(mp in x[action] for x in v) + 0.3) / (len(v) + 2.0)

    def _wr(self, team, mp, cutoff):
        return _decayed(self.res[(team, mp)], cutoff, 0.5, k=2.0, half_life=180.0)

    def _lineup_map(self, lineup, mp, cutoff):
        return float(np.mean([_decayed(self.pm.get((p["id"], mp), ()), cutoff, 1.0, k=2.0)[0] for p in lineup]))

    def features(self, m):
        a, b, c = m["team_a_key"], m["team_b_key"], m.cutoff
        pool = sorted(mp for mp, t in self.seen.items() if c - t <= 120 * DAY)
        out = {k: 0.0 for k in self.columns}
        if not pool:
            return out
        w = np.array([(1 - self._share(a, mp, "removed")) * (1 - self._share(b, mp, "removed")) *
                      (1 + self._share(a, mp, "picked") + self._share(b, mp, "picked")) for mp in pool])
        p = w / w.sum()
        wa = np.array([self._wr(a, mp, c)[0] for mp in pool])
        wb = np.array([self._wr(b, mp, c)[0] for mp in pool])
        na = np.array([self._wr(a, mp, c)[1] for mp in pool])
        nb = np.array([self._wr(b, mp, c)[1] for mp in pool])
        q = wa * (1 - wb) / (wa * (1 - wb) + wb * (1 - wa))
        pbar = float(np.clip((p * q).sum(), 1e-4, 1 - 1e-4))
        out.update({
            "mp_edge": float((p * (wa - wb)).sum()) * 10,
            "mp_series_logit": logit(series_probability(pbar, m["format"])),
            "mp_depth_diff": float(((wa > 0.55) & (na >= 2)).sum() - ((wb > 0.55) & (nb >= 2)).sum()),
            "mp_vetoes_min": math.log1p(min(len(self.vetoes[a]), len(self.vetoes[b]))),
        })
        la, lb = _lineup(m, 0), _lineup(m, 1)
        if la and lb:
            out["mp_player_edge"] = float(sum(pi * (self._lineup_map(la, mp, c) - self._lineup_map(lb, mp, c))
                                              for pi, mp in zip(p, pool))) * 10
        return out


# ------------------------------------------------------------------ CT / T sides

@register
class Sides(Group):
    """Round win rates on CT and on T from earlier maps' half scores (HTML side tags), and the
    lineup's earlier CT-only and T-only player ratings."""
    name = "sides"
    columns = {"ct_rate_diff": "anti", "t_rate_diff": "anti", "ct_rating_diff": "anti", "t_rating_diff": "anti"}

    def __init__(self):
        self.h = defaultdict(lambda: deque(maxlen=40))  # team -> (ct_won, ct_n, t_won, t_n)
        self.ps = defaultdict(lambda: deque(maxlen=30))  # player -> (t, ct rating, t rating)

    def update(self, new):
        if not len(new.records):
            return
        for rec in new.records["rec"]:
            ka, kb = rec["keys"]
            for h in rec["halves"]:
                b_ct_n = h["a_t_n"]  # B is CT when A is T
                b_t_n = h["a_ct_n"]
                self.h[ka].append((h["a_ct"], h["a_ct_n"], h["a_t"], h["a_t_n"]))
                self.h[kb].append((h["b_ct"], b_ct_n, h["b_t"], b_t_n))
            for pid, s in rec["sides"].items():
                self.ps[pid].append((rec["started_at"], (s.get("ct") or {}).get("rating"), (s.get("t") or {}).get("rating")))

    def _rates(self, t):
        d = self.h[t]
        ct = (sum(x[0] for x in d) + 5.0) / (sum(x[1] for x in d) + 10.0)
        tt = (sum(x[2] for x in d) + 5.0) / (sum(x[3] for x in d) + 10.0)
        return ct, tt

    def _lineup(self, lineup, cutoff):
        ct = [_decayed([(t, c) for t, c, _ in self.ps.get(p["id"], ())], cutoff, 1.0)[0] for p in lineup]
        tt = [_decayed([(t, x) for t, _, x in self.ps.get(p["id"], ())], cutoff, 1.0)[0] for p in lineup]
        return float(np.mean(ct)), float(np.mean(tt))

    def features(self, m):
        a, b = self._rates(m["team_a_key"]), self._rates(m["team_b_key"])
        out = {"ct_rate_diff": (a[0] - b[0]) * 10, "t_rate_diff": (a[1] - b[1]) * 10,
               "ct_rating_diff": 0.0, "t_rating_diff": 0.0}
        la, lb = _lineup(m, 0), _lineup(m, 1)
        if la and lb:
            x, y = self._lineup(la, m.cutoff), self._lineup(lb, m.cutoff)
            out["ct_rating_diff"], out["t_rating_diff"] = (x[0] - y[0]) * 10, (x[1] - y[1]) * 10
        return out


# ------------------------------------------------------------------ stand-ins and coaches

@register
class Standin(Group):
    """Stand-in markers in the lineup and coaches substituting (the match note), per team."""
    name = "standin"
    columns = {"standin_diff": "anti", "coach_sub_diff": "anti", "any_standin": "sym"}

    def features(self, m):
        la, lb = _lineup(m, 0), _lineup(m, 1)
        if la is None or lb is None:
            return {c: 0.0 for c in self.columns}
        sa, sb = sum(p["standin"] for p in la), sum(p["standin"] for p in lb)
        ca = cb = 0
        for sub in (m["hltv_pre"].get("notes") or {}).get("coach_subs", []):
            coach = sub["coach"].lower()
            ca += any(p["nick"].lower() == coach for p in la)
            cb += any(p["nick"].lower() == coach for p in lb)
        return {"standin_diff": float(sa - sb), "coach_sub_diff": float(ca - cb), "any_standin": float(sa + sb > 0)}


# ------------------------------------------------------------------ roster continuity

@register
class Roster(Group):
    """Lineup continuity against the team's earlier HLTV lineups: players shared with the last
    one, Matches in 180 days with at least four of today's five, IGL present and unchanged."""
    name = "roster"
    columns = {"shared_diff": "anti", "tenure_diff": "anti", "igl_diff": "anti", "igl_same_diff": "anti"}

    def __init__(self):
        self.l = defaultdict(lambda: deque(maxlen=60))  # team -> (t, lineup ids, igl ids)

    def update(self, new):
        if not len(new.records):
            return
        for rec in new.records["rec"]:
            if not rec["lineups"]:
                continue
            for side in (0, 1):
                self.l[rec["keys"][side]].append((rec["started_at"], set(rec["lineups"][side]), set(rec["igl"][side])))

    def _one(self, team, lineup, cutoff):
        cur = {p["id"] for p in lineup}
        igl = {p["id"] for p in lineup if p["igl"]}
        hist = self.l[team]
        if not hist:
            return 0.0, 0.0, float(bool(igl)), 0.0
        _, last, last_igl = hist[-1]
        tenure = sum(1 for t, ids, _ in hist if cutoff - t <= 180 * DAY and len(cur & ids) >= 4)
        return float(len(cur & last)), math.log1p(tenure), float(bool(igl)), float(bool(igl & last_igl))

    def features(self, m):
        la, lb = _lineup(m, 0), _lineup(m, 1)
        if la is None or lb is None:
            return {c: 0.0 for c in self.columns}
        a, b = self._one(m["team_a_key"], la, m.cutoff), self._one(m["team_b_key"], lb, m.cutoff)
        return {"shared_diff": a[0] - b[0], "tenure_diff": a[1] - b[1], "igl_diff": a[2] - b[2],
                "igl_same_diff": a[3] - b[3]}


# ------------------------------------------------------------------ format x stakes

@register
class Stakes(Group):
    """Stage and stakes from the match note's structured `*` lines (Swiss record, elimination,
    decider, playoffs, final), and their interaction with the format and the anchor's logit: a
    symmetric context can only move a probability through such interactions in a linear model."""
    name = "stakes"
    columns = {"st_elim": "sym", "st_adv": "sym", "st_decider": "sym", "st_playoff": "sym", "st_final": "sym",
               "has_stage": "sym", "elim_x_anchor": "anti", "playoff_x_anchor": "anti",
               "bo1_x_anchor": "anti", "bo5_x_anchor": "anti"}

    def features(self, m):
        pre = m["hltv_pre"]
        text = " ".join((pre.get("notes") or {}).get("stage", [])).lower() if pre else ""
        import re
        rec = re.search(r"(\d)\s*-\s*(\d)\s+record", text) or re.search(r"\((\d)\s*-\s*(\d)\)", text)
        wins, losses = (int(rec.group(1)), int(rec.group(2))) if rec else (None, None)
        elim = (losses == 2) or "elim" in text or "lower bracket" in text
        adv = wins == 2 or "advances" in text or "qualifies" in text
        decider = (wins == 2 and losses == 2) or "decider" in text
        playoff = any(w in text for w in ("final", "semi", "quarter", "playoff"))
        final = "grand final" in text
        la = _logit_anchor(m)
        return {"st_elim": float(elim), "st_adv": float(adv), "st_decider": float(decider),
                "st_playoff": float(playoff), "st_final": float(final), "has_stage": float(bool(text)),
                "elim_x_anchor": float(elim) * la, "playoff_x_anchor": float(playoff) * la,
                "bo1_x_anchor": float(m["format"] == "bo1") * la, "bo5_x_anchor": float(m["format"] == "bo5") * la}


# ------------------------------------------------------------------ map-level model (online)

@register
class MapModel(MapPool):
    """A map-level model, point in time by construction: an online logistic regression (no
    intercept, antisymmetric inputs) on single maps. When a Match is described, the per-map inputs
    of every pool map are stored; once its record is released (known finish before a later
    cutoff), each map it actually played is one SGD step with that map's stored inputs. The
    prediction combines the per-map probabilities over the expected pre-veto pool (`MapPool`'s
    veto shares) and goes through the series conversion. Inputs per map: the teams' decayed win
    share on it (logit difference), the lineups' per-map player rating difference, the overall map
    Elo difference and the Valve map probability's logit."""
    name = "map_model"
    columns = {"mm_series_logit": "anti", "mm_map_logit": "anti", "mm_trained": "sym"}
    LR, L2 = 0.03, 1e-3

    def __init__(self):
        super().__init__()
        self.w = np.zeros(4)
        self.n = 0
        self.stored: dict[int, tuple[str, dict]] = {}
        self.elo = defaultdict(lambda: 1500.0)

    def update(self, new):
        if len(new.records):
            for mid, rec in zip(new.records["match_id"], new.records["rec"]):
                got = self.stored.pop(int(mid), None)
                if got is None:
                    continue
                a_key, xs = got
                flip = rec["keys"][0] != a_key
                for mp in rec["maps"]:
                    if not mp.get("played") or mp.get("winner") not in ("a", "b") or mp["map_name"] not in xs:
                        continue
                    y = 1.0 if (mp["winner"] == "a") != flip else 0.0
                    x = xs[mp["map_name"]]
                    p = 1.0 / (1.0 + math.exp(-float(self.w @ x)))
                    self.w += self.LR * ((y - p) * x - self.L2 * self.w)
                    self.n += 1
        if len(new.maps):
            for a, b, won in zip(new.maps.team_a_key, new.maps.team_b_key, new.maps.a_won):
                ea = 1.0 / (1.0 + 10 ** ((self.elo[b] - self.elo[a]) / 400.0))
                d = 20.0 * ((1.0 if won else 0.0) - ea)
                self.elo[a] += d
                self.elo[b] -= d
        super().update(new)

    def features(self, m):
        a, b, c = m["team_a_key"], m["team_b_key"], m.cutoff
        pool = sorted(mp for mp, t in self.seen.items() if c - t <= 120 * DAY)
        if not pool:
            return {"mm_series_logit": 0.0, "mm_map_logit": 0.0, "mm_trained": 0.0}
        w = np.array([(1 - self._share(a, mp, "removed")) * (1 - self._share(b, mp, "removed")) *
                      (1 + self._share(a, mp, "picked") + self._share(b, mp, "picked")) for mp in pool])
        p = w / w.sum()
        vrs = m["vrs_map_p"]
        vrs_logit = logit(float(vrs)) / 4.0 if vrs is not None and not (isinstance(vrs, float) and math.isnan(vrs)) else 0.0
        la, lb = _lineup(m, 0), _lineup(m, 1)
        elo = (self.elo[a] - self.elo[b]) / 100.0
        xs = {}
        for mp in pool:
            wa, wb = self._wr(a, mp, c)[0], self._wr(b, mp, c)[0]
            pl = (self._lineup_map(la, mp, c) - self._lineup_map(lb, mp, c)) * 10 if la and lb else 0.0
            xs[mp] = np.array([logit(wa) - logit(wb), pl, elo, vrs_logit])
        self.stored[int(m["match_id"])] = (a, xs)
        q = np.array([1.0 / (1.0 + math.exp(-float(self.w @ xs[mp]))) for mp in pool])
        pbar = float(np.clip((p * q).sum(), 1e-4, 1 - 1e-4))
        return {"mm_series_logit": logit(series_probability(pbar, m["format"])), "mm_map_logit": logit(pbar),
                "mm_trained": math.log1p(self.n)}
