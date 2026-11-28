"""Every market source's prices of a Match, point in time (phase 5, signal A).

The anchor's chain (`anchor.Markets.quote`) keeps one de-vigged price: the first source that has
one. Here each source is read on its own with the same listing match (same two teams within 6 h of
the Scheduled Start) and the same rule: only ticks strictly before the cutoff (OddsPapi also
before its hard lock). Per Match and source:

- Pinnacle (OddsPapi `pinnacle`, else `pinnacle+30`): the two decimal prices and stake limits of
  the last active ticks, oriented to team A, so any de-vig method can be applied;
- Polymarket: the last price-history point (team A's side);
- Kalshi: the two markets' bid/ask closes (normalised mid, mean spread), volume of the last 24 h of
  candles and open interest, all from candles that ended before the cutoff;
- the same at an *early* time: the last price strictly before cutoff − 12 h; if the listing has no
  price then, its first price if that is at least 2 h before the cutoff; else none.

Polymarket's event `volume` is the total at fetch time (after the Match), so it is not used.
"""
from __future__ import annotations

import bisect
import math
import pickle
from pathlib import Path

import numpy as np
import pandas as pd

from . import timeutil as T
from .anchor import FREE_PINNACLE, PARTICIPANTS, PINNACLE, WINNER, Markets, _same_match
from .archive import load
from .teams import team_key

EARLY = 12 * T.HOUR
MIN_LEAD = 2 * T.HOUR


# ---------------------------------------------------------------- de-vig (two outcomes)

def devig_proportional(oa: float, ob: float) -> float:
    a, b = 1.0 / oa, 1.0 / ob
    return a / (a + b)


def devig_power(oa: float, ob: float) -> float:
    """p_i = (1/o_i)^k with k such that the two sum to 1."""
    a, b = 1.0 / oa, 1.0 / ob
    lo, hi = 1e-3, 50.0
    for _ in range(100):
        k = 0.5 * (lo + hi)
        if a ** k + b ** k > 1.0:
            lo = k
        else:
            hi = k
    k = 0.5 * (lo + hi)
    return a ** k / (a ** k + b ** k)


def devig_shin(oa: float, ob: float) -> float:
    """Shin (1993): p_i = (sqrt(z² + 4(1−z) π_i²/B) − z) / (2(1−z)), z chosen so they sum to 1."""
    pi = np.array([1.0 / oa, 1.0 / ob])
    B = pi.sum()
    if B <= 1.0:
        return float(pi[0] / B)

    def total(z):
        return float(((np.sqrt(z * z + 4 * (1 - z) * pi ** 2 / B) - z) / (2 * (1 - z))).sum())

    lo, hi = 0.0, 0.999
    for _ in range(100):
        z = 0.5 * (lo + hi)
        if total(z) > 1.0:
            lo = z
        else:
            hi = z
    z = 0.5 * (lo + hi)
    p = (np.sqrt(z * z + 4 * (1 - z) * pi ** 2 / B) - z) / (2 * (1 - z))
    return float(p[0] / p.sum())


DEVIG = {"proportional": devig_proportional, "power": devig_power, "shin": devig_shin}


# ---------------------------------------------------------------- extraction

def _at(series, t):
    """Last (time, value) strictly before t in a time-sorted list, or None."""
    i = bisect.bisect_left([x[0] for x in series], t) if not hasattr(series, "times") else bisect.bisect_left(series.times, t)
    return series[i - 1] if i > 0 else None


class _Timed(list):
    """A time-sorted list of (time, value) with its times cached for bisection."""
    def __init__(self, rows):
        super().__init__(sorted(rows, key=lambda x: x[0]))
        self.times = [x[0] for x in self]


def _early(series, cutoff):
    e = _at(series, cutoff - EARLY)
    if e is not None:
        return e
    if series and series[0][0] < cutoff - MIN_LEAD:
        return series[0]
    return None


class Detail(Markets):
    def pinnacle_series(self, m, cutoff):
        """[(t, (odds_a, odds_b, limit_a, limit_b))] of the first book with a price, oriented to A."""
        pair = frozenset((team_key(m.team_a.name), team_key(m.team_b.name)))
        for index in self._fixture_index:
            settled = self._settled(index.get(pair, []), m)
            if settled is None:
                continue
            fid, side, hard_lock = settled
            history = self.histories.get(fid)
            if history is None:
                continue
            cut = min(cutoff, hard_lock)
            for book in (PINNACLE, FREE_PINNACLE):
                outs = (((((history.get("bookmakers") or {}).get(book) or {}).get("markets") or {}).get(WINNER) or {})
                        .get("outcomes") or {})
                sides = []
                for o in PARTICIPANTS:
                    ticks = ((outs.get(o) or {}).get("players") or {}).get("0")
                    ticks = ticks if isinstance(ticks, list) else []
                    sides.append(_Timed((t, tk) for tk in ticks if (t := T.parse(tk.get("createdAt"))) is not None and t < cut))
                if not sides[0] or not sides[1]:
                    continue
                if sides[0][-1][1].get("active") is not True or sides[1][-1][1].get("active") is not True:
                    continue  # as the chain: the latest ticks must be active, else the next book
                times = sorted({t for s in sides for t, _ in s})
                series = []
                for t in times:
                    la, lb = _at(sides[0], t + 1), _at(sides[1], t + 1)
                    if la is None or lb is None or la[1].get("active") is not True or lb[1].get("active") is not True:
                        continue
                    pa, pb = la[1].get("price"), lb[1].get("price")
                    if not isinstance(pa, (int, float)) or not isinstance(pb, (int, float)) or pa <= 1 or pb <= 1:
                        continue
                    lim = (la[1].get("limit"), lb[1].get("limit"))
                    v = (pa, pb, lim[0], lim[1]) if side == 0 else (pb, pa, lim[1], lim[0])
                    series.append((t, v))
                if series:
                    return _Timed(series)
                # the chain stops at the first book with both latest ticks; keep the same order
            return None
        return None

    def polymarket_series(self, m, cutoff):
        pair = frozenset((team_key(m.team_a.name), team_key(m.team_b.name)))
        for event, outcomes, start in self._poly_index.get(pair, []):
            side = _same_match(m, outcomes, start)
            if side is None:
                continue
            history = self.polymarket_histories.get(event.get("slug"))
            if history is None:
                return None
            pts = []
            for pt in history.get("history") or []:
                t, p = pt.get("t"), pt.get("p")
                if isinstance(t, int) and isinstance(p, (int, float)) and 0.0 < p < 1.0:
                    at = T.from_seconds(t)
                    if at < cutoff:
                        pts.append((at, float(p) if side == 0 else 1.0 - float(p)))
            pts = _Timed(pts)
            return pts or None
        return None

    def kalshi_series(self, m, cutoff):
        pair = frozenset((team_key(m.team_a.name), team_key(m.team_b.name)))
        for event, markets, keys, start in self._kalshi_index.get(pair, []):
            if len(markets) != 2:
                continue
            side = _same_match(m, keys, start)
            if side is None:
                continue
            per = []
            for mk in markets:
                c = self.kalshi_candles.get(mk.get("ticker"))
                rows = []
                for cd in (c or {}).get("candlesticks") or []:
                    ts = cd.get("end_period_ts")
                    if not isinstance(ts, int):
                        continue
                    try:
                        bid = float(cd["yes_bid"]["close_dollars"])
                        ask = float(cd["yes_ask"]["close_dollars"])
                    except (KeyError, TypeError, ValueError):
                        continue
                    at = T.from_seconds(ts)
                    if at < cutoff:
                        rows.append((at, (bid, ask, float(cd.get("volume_fp") or 0.0), float(cd.get("open_interest_fp") or 0.0))))
                per.append(_Timed(rows))
            if not per[0] or not per[1]:
                return None
            times = sorted({t for s in per for t, _ in s})
            series = []
            for t in times:
                x, y = _at(per[0], t + 1), _at(per[1], t + 1)
                if x is None or y is None:
                    continue
                ma, mb = (x[1][0] + x[1][1]) / 2, (y[1][0] + y[1][1]) / 2
                if not (ma > 0 and mb > 0):
                    continue
                p_first = ma / (ma + mb)
                spread = ((x[1][1] - x[1][0]) + (y[1][1] - y[1][0])) / 2
                series.append((t, (p_first if side == 0 else 1.0 - p_first, spread)))
            if not series:
                return None
            series = _Timed(series)
            vol = sum(v[2] for s in per for t, v in s if t >= cutoff - 24 * T.HOUR)
            oi = sum(s[-1][1][3] for s in per)
            return series, vol, oi
        return None


def build(archive_dir: Path, match_ids: set[int] | None = None, log=print) -> pd.DataFrame:
    archive = load(archive_dir, with_hltv=False)
    d = Detail(archive)
    rows = []
    for s in archive.stages:
        for m in s.matches:
            if match_ids is not None and m.id not in match_ids:
                continue
            cut = m.cutoff()
            r = {"match_id": m.id}
            pin = d.pinnacle_series(m, cut)
            if pin:
                t, (oa, ob, la, lb) = pin[-1]
                r.update(pin_oa=oa, pin_ob=ob, pin_lim_a=la, pin_lim_b=lb, pin_t=t, pin_n=len(pin))
                e = _early(pin, cut)
                if e is not None:
                    r.update(pin_oa_early=e[1][0], pin_ob_early=e[1][1], pin_t_early=e[0])
            poly = d.polymarket_series(m, cut)
            if poly:
                r.update(poly_p=poly[-1][1], poly_t=poly[-1][0], poly_n=len(poly))
                e = _early(poly, cut)
                if e is not None:
                    r.update(poly_p_early=e[1], poly_t_early=e[0])
            kal = d.kalshi_series(m, cut)
            if kal:
                series, vol, oi = kal
                t, (p, spread) = series[-1]
                r.update(kal_p=p, kal_spread=spread, kal_t=t, kal_vol24=vol, kal_oi=oi, kal_n=len(series))
                e = _early(series, cut)
                if e is not None:
                    r.update(kal_p_early=e[1][0], kal_t_early=e[0])
            rows.append(r)
    out = pd.DataFrame(rows).drop_duplicates("match_id").set_index("match_id")
    log(f"market detail: {len(out)} Matches, Pinnacle {out['pin_oa'].notna().sum()}, "
        f"Polymarket {out['poly_p'].notna().sum()}, Kalshi {out['kal_p'].notna().sum()}")
    return out


def load_detail(archive_dir: Path, cache: Path, log=print) -> pd.DataFrame:
    f = cache / "market_detail.pkl"
    if f.exists():
        return pickle.loads(f.read_bytes())
    out = build(archive_dir, log=log)
    f.write_bytes(pickle.dumps(out))
    return out


def logit(p):
    p = np.clip(np.asarray(p, float), 1e-6, 1 - 1e-6)
    return np.log(p / (1 - p))


def source_logits(det: pd.DataFrame, devig: str = "power", early: bool = False) -> pd.DataFrame:
    """z_pin, z_poly, z_kal (NaN where missing), de-vigged as asked; `early` = the early price."""
    sfx = "_early" if early else ""
    fn = DEVIG[devig]
    pin = [fn(a, b) if not (pd.isna(a) or pd.isna(b)) else math.nan
           for a, b in zip(det.get("pin_oa" + sfx, pd.Series(dtype=float)), det.get("pin_ob" + sfx, pd.Series(dtype=float)))]
    out = pd.DataFrame(index=det.index)
    out["z_pin"] = np.where(np.isnan(pin), np.nan, logit(np.nan_to_num(np.array(pin, float), nan=0.5)))
    for src, col in (("poly", "poly_p"), ("kal", "kal_p")):
        v = det[col + sfx].to_numpy(float)
        out[f"z_{src}"] = np.where(np.isnan(v), np.nan, logit(np.nan_to_num(v, nan=0.5)))
    return out
