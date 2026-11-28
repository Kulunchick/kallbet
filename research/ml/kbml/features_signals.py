"""Phase 5 feature groups (exploratory signals B, C, D of SIGNALS.md). Point in time as every group:
they learn only from what the timeline releases (results, maps and HLTV records whose known finish
is strictly before the cutoff) and describe a Match from its pre-match columns and its own page's
pre-match lineups.

- `veto_eb` (B): Bo3 veto simulation over per-map strengths shrunk to team strength by empirical
  Bayes; per-map evidence is reset when a map (re)enters the active pool, the pool and its change
  dates being read from earlier vetoes.
- `openskill` (C): Weng-Lin (OpenSkill, Plackett-Luce) player ratings updated on every released map;
  the current five (own page's lineup, else the team's last known five) summed, max, uncertainty, and
  core continuity (maps the five played together).
- `travel` (D): time zones crossed since the previous Match, eastward flag, days since the team last
  played in another region, home region; from the Stage's country (PandaScore, else Liquipedia
  `|country=`) and each team's PandaScore location.
"""
from __future__ import annotations

import math
from collections import Counter, defaultdict, deque

import numpy as np

from .anchor import logit
from .features import Group, register
from .teams import team_key

DAY = 86_400_000_000
LN10_400 = math.log(10) / 400.0


def _sig(z):
    return 1.0 / (1.0 + math.exp(-z))


def _veto_sides(rec):
    """(per-team {'removed', 'picked'} sets, step order of team sides [0/1 per step]) of a record."""
    keys = rec["keys"]
    names = {}
    for side, team in enumerate(rec["overall"][:2]):
        names[team_key(team.get("team_name") or "")] = keys[side]
    per = {k: {"removed": set(), "picked": set()} for k in keys}
    order = []
    for step in rec["veto"]:
        k = team_key(step.get("team") or "")
        k = k if k in per else names.get(k)
        if k in per and step.get("action") in ("removed", "picked"):
            per[k][step["action"]].add(step["map_name"])
            order.append(keys.index(k))
    return per, order


# ------------------------------------------------------------------ B: Bo3 veto, empirical Bayes

class ActivePool:
    """The active seven-map pool, read from released 7-step vetoes: the seven maps that appear most
    often in the vetoes of the last `window` days (by start). A map's entry time is when it first
    joins the pool after being out of it."""

    def __init__(self, window_days=14):
        self.window = window_days * DAY
        self.recent = deque()  # (t, maps)
        self.pool: frozenset = frozenset()
        self.entered: dict[str, int] = {}
        self.changes: list[tuple[int, list, list]] = []

    def add(self, t, maps):
        self.recent.append((t, frozenset(maps)))

    def refresh(self, now):
        if not self.recent:
            return
        latest = max(t for t, _ in self.recent)
        while self.recent and self.recent[0][0] < latest - self.window:
            self.recent.popleft()
        c = Counter(m for _, ms in self.recent for m in ms)
        top = sorted(c, key=lambda m: (-c[m], m))[:7]
        if len(top) < 7:
            return
        new = frozenset(top)
        if new != self.pool:
            added, dropped = sorted(new - self.pool), sorted(self.pool - new)
            if self.pool:
                self.changes.append((now, added, dropped))
            for mp in added:
                self.entered[mp] = now
            self.pool = new


@register
class VetoEB(Group):
    name = "veto_eb"
    columns = {"veb_series_logit": "anti", "veb_map_logit": "anti", "veb_offset_edge": "anti", "veb_has": "sym"}
    K = 20.0  # overall map Elo
    HALF_LIFE = 180.0
    inline = False   # True: integrate each veto while computing (slow, CPU); else `fill_series`
    reset = True     # variance reset on pool change (exploratory switch)
    shrink = True    # empirical-Bayes shrinkage (False: tau² = 25, almost no shrinkage)

    def __init__(self):
        self.r = defaultdict(lambda: 1500.0)
        self.res = defaultdict(list)  # (team, map) -> [(t, residual, info)]
        self.vetoes = defaultdict(lambda: deque(maxlen=25))
        self.pool = ActivePool()
        self.tau2 = 0.1
        self.tau_at = None
        self.now = 0
        self.pending: dict[int, np.ndarray] = {}
        VetoEB.last = self

    def pending_one(self, mid):
        return self.pending[mid][None]

    def update(self, new):
        if len(new.records):
            for rec in sorted(new.records["rec"], key=lambda r: r["started_at"]):
                if len(rec["veto"]) == 7:
                    self.pool.add(rec["started_at"], [s["map_name"] for s in rec["veto"] if s.get("map_name")])
                if rec["veto"]:
                    per, _ = _veto_sides(rec)
                    for k in rec["keys"]:
                        self.vetoes[k].append(per[k])
        if new.maps is not None and len(new.maps):
            for a, b, mp, won, t in zip(new.maps.team_a_key, new.maps.team_b_key, new.maps.map_name,
                                        new.maps.a_won, new.maps.known_finish):
                s = 1.0 if won else 0.0
                e = 1.0 / (1.0 + 10 ** ((self.r[b] - self.r[a]) / 400.0))
                self.r[a] += self.K * (s - e)
                self.r[b] -= self.K * (s - e)
                info = e * (1 - e)
                self.res[(a, mp)].append((int(t), s - e, info))
                self.res[(b, mp)].append((int(t), e - s, info))

    def _stats(self, team, mp, cutoff):
        rows = self.res.get((team, mp), ())
        start = self.pool.entered.get(mp, 0) if self.reset else 0
        sr = si = 0.0
        for t, r, i in rows:
            if t < start:
                continue
            w = 0.5 ** ((cutoff - t) / (self.HALF_LIFE * DAY))
            sr += w * r
            si += w * i
        return sr, si

    def _estimate_tau2(self, cutoff):
        """Method of moments over (team, map) pairs with information ≥ 2: var(raw) − mean(sigma²)."""
        raws, sig = [], []
        for (team, mp) in list(self.res.keys()):
            if mp not in self.pool.pool:
                continue
            sr, si = self._stats(team, mp, cutoff)
            if si >= 2.0:
                raws.append(sr / si)
                sig.append(1.0 / si)
        if len(raws) < 30:
            return 0.1
        return float(max(np.var(raws) - np.mean(sig), 0.01))

    def offset(self, team, mp, cutoff):
        sr, si = self._stats(team, mp, cutoff)
        if si <= 0:
            return 0.0
        tau2 = self.tau2 if self.shrink else 25.0
        return sr / (si + 1.0 / tau2)  # = w · raw, w = tau²/(tau² + 1/si)

    def _weights(self, team, pool, action):
        v = self.vetoes[team]
        return np.array([(sum(mp in x[action] for x in v) + 0.3) / (len(v) + 2.0) for mp in pool])

    def features(self, m):
        c = m.cutoff
        if c - self.now > 7 * DAY or self.tau_at is None:
            self.pool.refresh(c)
            self.now = c
        if self.tau_at is None or c - self.tau_at > 30 * DAY:
            self.tau2 = self._estimate_tau2(c)
            self.tau_at = c
        out = {k: 0.0 for k in self.columns}
        pool = sorted(self.pool.pool)
        if m["format"] != "bo3" or len(pool) != 7:
            return out
        a, b = m["team_a_key"], m["team_b_key"]
        zt = (self.r[a] - self.r[b]) * LN10_400
        off = np.array([self.offset(a, mp, c) - self.offset(b, mp, c) for mp in pool])
        pm = 1.0 / (1.0 + np.exp(-(zt + off)))
        ban = [self._weights(a, pool, "removed"), self._weights(b, pool, "removed")]
        pick = [self._weights(a, pool, "picked"), self._weights(b, pool, "picked")]
        # the series probability needs the exact veto integration: batched later (`veto_series`)
        self.pending[int(m["match_id"])] = np.stack([pm, ban[0], ban[1], pick[0], pick[1]])
        ps = float(np.clip(veto_series(self.pending_one(int(m["match_id"])), "numpy")[0], 1e-4, 1 - 1e-4))             if self.inline else 0.5
        out.update({"veb_series_logit": logit(ps), "veb_map_logit": float(np.mean(zt + off)),
                    "veb_offset_edge": float(off.mean()), "veb_has": 1.0})
        return out


_PERMS = None


def _perms():
    """The 5040 ordered choices of the six veto steps among seven maps (the 7th is the decider)."""
    global _PERMS
    if _PERMS is None:
        import itertools
        _PERMS = np.array(list(itertools.permutations(range(7), 6)), dtype=np.int64)
    return _PERMS


def veto_series(batch: np.ndarray, device: str = "numpy", chunk: int = 512) -> np.ndarray:
    """Exact Bo3 veto integration. `batch`: (n, 5, 7) = per-map P(A wins), A's and B's ban weights,
    A's and B's pick weights, over the active pool. Order Ban-Ban-Pick-Pick-Ban-Ban-Decider, each
    team drawing among the remaining maps in proportion to its weights, averaged over which team
    starts. Returns P(A wins the series) = E[p1 p2 + p1 (1 − p2) p3 + (1 − p1) p2 p3].
    `device`: "numpy" (CPU) or a torch device ("cuda"); the same arithmetic in float64."""
    perms = _perms()
    acts = ["b", "b", "p", "p", "b", "b"]
    out = np.empty(len(batch))
    if device == "numpy":
        xp, P = np, perms
    else:
        import torch
        xp = None
        P = torch.as_tensor(perms, device=device)
    for i in range(0, len(batch), chunk):
        part = batch[i:i + chunk]
        if device == "numpy":
            B = part
        else:
            import torch
            B = torch.as_tensor(part, dtype=torch.float64, device=device)
        pm = B[:, 0]
        total = 0.0
        for first in (0, 1):
            seq = [first, 1 - first] * 3
            prob = 1.0
            for s_, (side, act) in enumerate(zip(seq, acts)):
                W = B[:, 1 + side] if act == "b" else B[:, 3 + side]  # (n, 7)
                chosen = W[:, P[:, s_]]  # (n, 5040)
                used = W[:, P[:, :s_]].sum(-1) if s_ else 0.0
                denom = W.sum(-1, keepdims=True) - used
                prob = prob * chosen / denom
            p1, p2 = pm[:, P[:, 2]], pm[:, P[:, 3]]
            p3 = pm[:, 21 - P.sum(-1)]
            series = p1 * p2 + p1 * (1 - p2) * p3 + (1 - p1) * p2 * p3
            total = total + (prob * series).sum(-1) / 2
        out[i:i + chunk] = total if device == "numpy" else total.cpu().numpy()
        if device != "numpy":
            del B, pm, total, prob, chosen, series, p1, p2, p3
    if device != "numpy":
        import torch
        torch.cuda.empty_cache()
    return out


def fill_series(X, pending: dict[int, np.ndarray], device: str = "numpy", column: str = "veb_series_logit"):
    """Write the exact series logit for every Match with a simulated veto into `X[column]`."""
    ids = list(pending)
    if not ids:
        return X
    p = np.clip(veto_series(np.stack([pending[i] for i in ids]), device), 1e-4, 1 - 1e-4)
    X.loc[ids, column] = np.log(p / (1 - p))
    return X


# ------------------------------------------------------------------ C: OpenSkill player ratings

@register
class OpenSkillGroup(Group):
    name = "openskill"
    columns = {"os_mu_sum_diff": "anti", "os_mu_max_diff": "anti", "os_sigma_sum_diff": "anti",
               "os_win_logit": "anti", "os_core_diff": "anti", "os_has": "sym"}

    def __init__(self):
        from openskill.models import PlackettLuce
        self.model = PlackettLuce()
        self.rating = {}
        self.last5 = {}  # team key -> last known five ids
        self.core = defaultdict(Counter)  # team key -> frozenset(five) -> maps played

    def _r(self, pid):
        r = self.rating.get(pid)
        if r is None:
            r = self.model.rating(name=str(pid))
            self.rating[pid] = r
        return r

    def update(self, new):
        if not len(new.records):
            return
        for rec in sorted(new.records["rec"], key=lambda r: r["started_at"]):
            winners = {mp["map_name"]: mp["winner"] for mp in rec["maps"] if mp.get("played") and mp.get("winner") in ("a", "b")}
            ids = rec["team_ids"]
            for pm in rec["per_map"]:
                w = winners.get(pm.get("map_name"))
                teams = pm.get("teams") or []
                if w is None or len(teams) != 2:
                    continue
                side = {t.get("team_id"): t for t in teams}
                ta, tb = side.get(ids[0]), side.get(ids[1])
                if ta is None or tb is None:
                    continue
                pa = [p["player_id"] for p in ta.get("players") or []]
                pb = [p["player_id"] for p in tb.get("players") or []]
                if len(pa) != 5 or len(pb) != 5:
                    continue
                ra, rb = [self._r(p) for p in pa], [self._r(p) for p in pb]
                ranks = [0, 1] if w == "a" else [1, 0]
                na, nb = self.model.rate([ra, rb], ranks=ranks)
                for p, r in zip(pa + pb, list(na) + list(nb)):
                    self.rating[p] = r
                for k, five in ((rec["keys"][0], pa), (rec["keys"][1], pb)):
                    self.core[k][frozenset(five)] += 1
                    self.last5[k] = five

    def _five(self, m, side):
        pre = m["hltv_pre"]
        if pre and len(pre.get("lineups") or []) == 2 and len(pre["lineups"][side]) == 5:
            return [p["id"] for p in pre["lineups"][side]]
        return self.last5.get(m["team_a_key"] if side == 0 else m["team_b_key"])

    def features(self, m):
        fa, fb = self._five(m, 0), self._five(m, 1)
        if not fa or not fb:
            return {k: 0.0 for k in self.columns}
        ra, rb = [self._r(p) for p in fa], [self._r(p) for p in fb]
        pw = self.model.predict_win([ra, rb])[0]
        ca = self.core[m["team_a_key"]][frozenset(fa)]
        cb = self.core[m["team_b_key"]][frozenset(fb)]
        mu = lambda rs: [r.mu for r in rs]  # noqa: E731
        sg = lambda rs: [r.sigma for r in rs]  # noqa: E731
        return {"os_mu_sum_diff": (sum(mu(ra)) - sum(mu(rb))) / 10.0,
                "os_mu_max_diff": (max(mu(ra)) - max(mu(rb))) / 5.0,
                "os_sigma_sum_diff": (sum(sg(ra)) - sum(sg(rb))) / 10.0,
                "os_win_logit": logit(min(max(pw, 1e-4), 1 - 1e-4)),
                "os_core_diff": math.log1p(ca) - math.log1p(cb), "os_has": 1.0}


# ------------------------------------------------------------------ D: travel

#: Standard-time UTC offset (hours) of a country's main competitive region (a multi-zone country
#: takes its most populous zone: US -6, BR -3, RU +3, CA -5, AU +10, CN +8, KZ +5, MX -6).
TZ = {
    "US": -6, "CA": -5, "MX": -6, "BR": -3, "AR": -3, "CL": -4, "UY": -3, "PE": -5, "CO": -5, "VE": -4,
    "PY": -4, "BO": -4, "EC": -5, "GT": -6, "CR": -6, "PA": -5, "DO": -4, "PR": -4, "SV": -6, "HN": -6,
    "GB": 0, "IE": 0, "PT": 0, "IS": 0, "MA": 0, "FR": 1, "DE": 1, "ES": 1, "IT": 1, "NL": 1, "BE": 1,
    "DK": 1, "SE": 1, "NO": 1, "PL": 1, "CZ": 1, "SK": 1, "AT": 1, "CH": 1, "HU": 1, "HR": 1, "SI": 1,
    "RS": 1, "BA": 1, "ME": 1, "MK": 1, "AL": 1, "MT": 1, "LU": 1, "XK": 1, "TN": 1, "DZ": 1,
    "FI": 2, "EE": 2, "LV": 2, "LT": 2, "UA": 2, "RO": 2, "BG": 2, "GR": 2, "MD": 2, "CY": 2, "IL": 2,
    "EG": 2, "ZA": 2, "LB": 2, "JO": 2, "RU": 3, "BY": 3, "TR": 3, "SA": 3, "KW": 3, "QA": 3, "BH": 3,
    "IQ": 3, "AE": 4, "AM": 4, "GE": 4, "AZ": 4, "OM": 4, "IR": 3.5, "KZ": 5, "UZ": 5, "PK": 5,
    "KG": 6, "TJ": 5, "TM": 5, "IN": 5.5, "BD": 6, "MN": 8, "CN": 8, "HK": 8, "MO": 8, "TW": 8,
    "SG": 8, "MY": 8, "PH": 8, "ID": 7, "TH": 7, "VN": 7, "KR": 9, "JP": 9, "AU": 10, "NZ": 12,
}
COUNTRY_NAMES = {"united states": "US", "usa": "US", "canada": "CA", "mexico": "MX", "brazil": "BR",
                 "argentina": "AR", "chile": "CL", "united kingdom": "GB", "england": "GB", "ireland": "IE",
                 "portugal": "PT", "france": "FR", "germany": "DE", "spain": "ES", "italy": "IT",
                 "netherlands": "NL", "belgium": "BE", "denmark": "DK", "sweden": "SE", "norway": "NO",
                 "poland": "PL", "czech republic": "CZ", "czechia": "CZ", "slovakia": "SK", "austria": "AT",
                 "switzerland": "CH", "hungary": "HU", "croatia": "HR", "serbia": "RS", "malta": "MT",
                 "finland": "FI", "estonia": "EE", "latvia": "LV", "lithuania": "LT", "ukraine": "UA",
                 "romania": "RO", "bulgaria": "BG", "greece": "GR", "israel": "IL", "russia": "RU",
                 "turkey": "TR", "türkiye": "TR", "saudi arabia": "SA", "qatar": "QA", "united arab emirates": "AE",
                 "kazakhstan": "KZ", "india": "IN", "china": "CN", "hong kong": "HK", "macau": "MO",
                 "taiwan": "TW", "singapore": "SG", "malaysia": "MY", "philippines": "PH", "indonesia": "ID",
                 "thailand": "TH", "vietnam": "VN", "south korea": "KR", "korea": "KR", "japan": "JP",
                 "australia": "AU", "new zealand": "NZ", "mongolia": "MN", "uzbekistan": "UZ",
                 "azerbaijan": "AZ", "georgia": "GE", "armenia": "AM", "belarus": "BY", "slovenia": "SI",
                 "bosnia and herzegovina": "BA", "montenegro": "ME", "north macedonia": "MK", "cyprus": "CY",
                 "egypt": "EG", "south africa": "ZA", "jordan": "JO", "kuwait": "KW", "bahrain": "BH",
                 "uruguay": "UY", "peru": "PE", "colombia": "CO"}


def region_of(country):
    tz = TZ.get(country)
    if tz is None:
        return None
    if tz <= -2:
        return "SA" if tz >= -4 and country not in ("US", "CA", "PR", "DO") else "NA"
    if tz <= 2.5:
        return "EU"
    if tz <= 4.5:
        return "ME/EEU"
    return "ASIA" if country not in ("AU", "NZ") else "OCE"


@register
class Travel(Group):
    """Set `Travel.stages = {stage_id: (country or None, offline bool or None)}` and
    `Travel.homes = {team_id: country}` before computing."""
    name = "travel"
    columns = {"tz_crossed_diff": "anti", "east_diff": "anti", "jet_recent_diff": "anti",
               "days_other_region_diff": "anti", "home_diff": "anti", "travel_has": "sym"}
    stages: dict = {}
    homes: dict = {}

    def __init__(self):
        self.last = {}  # team id -> (finish, country)
        self.other = {}  # team id -> list of (finish, region)

    def _where(self, stage_id, team_id):
        country, offline = self.stages.get(stage_id, (None, None))
        if offline and country in TZ:
            return country
        home = self.homes.get(team_id)
        if offline is False and home in TZ:
            return home
        return None

    def update(self, new):
        r = new.results
        for sid, a, b, f in zip(r.stage_id, r.team_a_id, r.team_b_id, r.known_finish):
            for t in (a, b):
                c = self._where(sid, t)
                if c is not None:
                    self.last[t] = (int(f), c)
                    self.other.setdefault(t, deque(maxlen=50)).append((int(f), region_of(c)))

    def _one(self, sid, t, cut):
        here = self._where(sid, t)
        if here is None:
            return None
        prev = self.last.get(t)
        shift = TZ[here] - TZ[prev[1]] if prev else 0.0
        days_prev = (cut - prev[0]) / DAY if prev else 30.0
        reg = region_of(here)
        away = [f for f, rg in self.other.get(t, ()) if rg != reg]
        d_other = min((cut - away[-1]) / DAY, 60.0) if away else 60.0
        home = self.homes.get(t)
        return {"cross": abs(shift), "east": float(shift > 0),
                "jet": float(abs(shift) >= 3 and days_prev <= 4), "other": d_other / 10.0,
                "home": float(home is not None and region_of(home) == reg)}

    def features(self, m):
        sid, cut = m["stage_id"], m.cutoff
        a, b = self._one(sid, m["team_a_id"], cut), self._one(sid, m["team_b_id"], cut)
        if a is None or b is None:
            return {k: 0.0 for k in self.columns}
        return {"tz_crossed_diff": a["cross"] - b["cross"], "east_diff": a["east"] - b["east"],
                "jet_recent_diff": a["jet"] - b["jet"], "days_other_region_diff": a["other"] - b["other"],
                "home_diff": a["home"] - b["home"], "travel_has": 1.0}


# ------------------------------------------------------------------ F: rolling player history (W5 table)

PH_STATS = ("rating", "swing", "ekd", "eadr", "ekast")
PH_SCALE = {"rating": 10.0, "swing": 1.0, "ekd": 10.0, "eadr": 0.1, "ekast": 0.1}
RATING3_FROM = "2023-10-16"


def load_player_maps(path) -> dict[int, list]:
    """W5's `hltv_player_match_stats.parquet` → {HLTV match id: [(map scope, team_id, player_id,
    rating3, swing, eK-eD per round, eADR, eKAST)]}: per-map rows of both sides together (`side ==
    all`, scope ≠ `all`). Rating 3.0, Swing and the eco-adjusted columns only from Rating 3.0 pages
    (Matches from 2023-10-16; earlier pages are Rating 2.0 and stay NaN). A Match's rows reach a
    group only when its HLTV record is released (its linked Match finished before the cutoff)."""
    import pandas as pd
    d = pd.read_parquet(path, columns=["match_id", "started_at", "scope", "side", "team_id", "player_id",
                                       "rating_version", "map_rounds", "rating3", "swing_pct", "ekills",
                                       "edeaths", "eadr", "ekast_pct"])
    d = d[(d["side"] == "all") & (d["scope"] != "all")]
    v3 = (d["rating_version"] == "3.0") & (d["started_at"].str[:10] >= RATING3_FROM)
    rounds = d["map_rounds"].where(d["map_rounds"] > 0).astype(float)
    ekd = (d["ekills"].astype(float) - d["edeaths"].astype(float)) / rounds
    cols = {"rating": d["rating3"], "swing": d["swing_pct"], "ekd": ekd, "eadr": d["eadr"], "ekast": d["ekast_pct"]}
    vals = np.column_stack([np.where(v3, pd.to_numeric(c, errors="coerce").to_numpy(float), np.nan) for c in cols.values()])
    out: dict[int, list] = defaultdict(list)
    for mid, scope, tid, pid, row in zip(d["match_id"].to_numpy(), d["scope"].to_numpy(), d["team_id"].to_numpy(),
                                         d["player_id"].to_numpy(), vals):
        out[int(mid)].append((scope, int(tid), int(pid), row))
    return dict(out)


@register
class PlayerHist(Group):
    """Set `PlayerHist.maps = load_player_maps(...)` before computing. Per player the last `N` maps,
    shrunk to the team's recent mean with `K` pseudo-maps; per lineup (own page's five, else the
    team's last five) the mean and the max of each stat; core-five continuity (maps the five played
    together for this team); coverage (share of the five with ≥ 5 rated maps)."""
    name = "player_hist"
    columns = {**{f"ph_{s}_{a}_diff": "anti" for s in PH_STATS for a in ("mean", "max")},
               "ph_core_diff": "anti", "ph_cover_min": "sym", "ph_has": "sym"}
    maps: dict = {}
    N, K, TEAM_N = 20, 5.0, 100

    def __init__(self):
        self.p = defaultdict(lambda: deque(maxlen=self.N))
        self.team = defaultdict(lambda: deque(maxlen=self.TEAM_N))
        self.glob = np.zeros(len(PH_STATS))
        self.glob_n = np.zeros(len(PH_STATS))
        self.last5 = {}
        self.core = defaultdict(Counter)

    def update(self, new):
        if not len(new.records):
            return
        for rec in sorted(new.records["rec"], key=lambda r: r["started_at"]):
            rows = self.maps.get(rec.get("hltv_match_id"))
            if not rows:
                continue
            key_of = {tid: k for tid, k in zip(rec["team_ids"], rec["keys"]) if tid is not None}
            by_map = defaultdict(lambda: defaultdict(list))
            for scope, tid, pid, v in rows:
                k = key_of.get(tid)
                if k is None:
                    continue
                self.p[pid].append(v)
                self.team[k].append(v)
                ok = ~np.isnan(v)
                self.glob[ok] += v[ok]
                self.glob_n[ok] += 1
                by_map[scope][k].append(pid)
            for scope in sorted(by_map):
                for k, five in by_map[scope].items():
                    if len(five) == 5:
                        self.core[k][frozenset(five)] += 1
                        self.last5[k] = five

    def _five(self, m, side):
        pre = m["hltv_pre"]
        if pre and len(pre.get("lineups") or []) == 2 and len(pre["lineups"][side]) == 5:
            return [p["id"] for p in pre["lineups"][side]]
        return self.last5.get(m["team_a_key"] if side == 0 else m["team_b_key"])

    def _team(self, key):
        prior = np.where(self.glob_n > 0, self.glob / np.maximum(self.glob_n, 1), 0.0)
        t = self.team.get(key)
        if not t:
            return prior
        a = np.array(t)
        n = (~np.isnan(a)).sum(axis=0)
        s = np.nansum(a, axis=0)
        return (s + self.K * prior) / (n + self.K)  # the team mean itself shrunk to the global mean

    def _lineup(self, five, key):
        tm = self._team(key)
        vals, cover = [], 0
        for pid in five:
            h = self.p.get(pid)
            if h:
                a = np.array(h)
                n = (~np.isnan(a)).sum(axis=0)
                s = np.nansum(a, axis=0)
            else:
                n, s = np.zeros(len(PH_STATS)), np.zeros(len(PH_STATS))
            vals.append((s + self.K * tm) / (n + self.K))
            cover += n[0] >= 5
        v = np.array(vals)
        return v.mean(axis=0), v.max(axis=0), cover / 5.0

    def features(self, m):
        fa, fb = self._five(m, 0), self._five(m, 1)
        if not fa or not fb or self.glob_n[0] == 0:
            return {k: 0.0 for k in self.columns}
        ka, kb = m["team_a_key"], m["team_b_key"]
        ma, xa, ca = self._lineup(fa, ka)
        mb, xb, cb = self._lineup(fb, kb)
        out = {}
        for i, s in enumerate(PH_STATS):
            out[f"ph_{s}_mean_diff"] = float(ma[i] - mb[i]) * PH_SCALE[s]
            out[f"ph_{s}_max_diff"] = float(xa[i] - xb[i]) * PH_SCALE[s]
        out["ph_core_diff"] = math.log1p(self.core[ka][frozenset(fa)]) - math.log1p(self.core[kb][frozenset(fb)])
        out["ph_cover_min"] = float(min(ca, cb))
        out["ph_has"] = 1.0
        return out
