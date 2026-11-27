"""The Signal Lab anchor's two inputs, as the Rust replay ports compute them.

- Valve rating (`vrs.rs` + `backtest/feeds.rs::ReplaySignal`): the latest global standings
  published strictly before the cutoff (a date-only snapshot counts from the end of its UTC day),
  Glicko-style map probability, series probability by format.
- Market (`backtest/feeds.rs::ReplayOdds`): OddsPapi Pinnacle, then Polymarket, then Kalshi;
  each the last de-vigged price strictly before the cutoff, for a listing of the same two teams
  within 6 h of the Scheduled Start.
- Anchor (`debate::policy::base`): 0.80 market + 0.20 rating in logit space; whichever exists alone.
"""
from __future__ import annotations

import json
import math
from dataclasses import dataclass
from datetime import datetime, timedelta

from . import timeutil as T
from .archive import Archive, ReplayMatch
from .teams import team_key


# ---------------------------------------------------------------- Valve rating

@dataclass
class Snapshot:
    observed: int
    date: str
    points: dict[str, int]


def _keyed_standings(doc: str) -> str:
    lines = doc.split("\n")
    out = [lines[0].replace("-", "_")] if lines else [""]
    for line in lines[1:]:
        cells = line.split("|")
        is_row = len(cells) > 4 and cells[1].strip().isdigit()
        if is_row:
            cells[3] = f" {team_key(cells[3])} "
            out.append("|".join(cells))
        else:
            out.append(line)
    return "\n".join(out)


def _parse_standings(doc: str) -> Snapshot:
    lines = doc.splitlines()
    header = lines[0]
    prefix = "### Standings as of "
    if not header.startswith(prefix):
        raise ValueError("missing Valve header")
    date_s = header[len(prefix):].split("<")[0]
    d = datetime.strptime(date_s, "%Y_%m_%d")
    # 23:59:59.999999999 of that UTC day; microseconds are enough against second-granular cutoffs.
    observed = T.parse(d.strftime("%Y-%m-%dT23:59:59.999999Z"))
    points: dict[str, int] = {}
    for row in lines[1:]:
        if not row.startswith("|"):
            continue
        cells = [c.strip() for c in row.strip("|").split("|")]
        rank = cells[0]
        if rank in ("Standing", ":-"):
            continue
        int(rank)
        rating = int(cells[1])
        team = cells[2].lower()
        points.setdefault(team, rating)
    if not points:
        raise ValueError("empty standings")
    return Snapshot(observed, d.strftime("%Y-%m-%d"), points)


def series_probability(p: float, fmt: str) -> float:
    if fmt == "bo1":
        return p
    if fmt == "bo3":
        return p * p * (3.0 - 2.0 * p)
    if fmt == "bo5":
        return p ** 3 * (10.0 - 15.0 * p + 6.0 * p * p)
    raise ValueError(fmt)


_RD = 75.0 * math.log(10.0) / (400.0 * math.pi)
_G = 1.0 / math.sqrt(1.0 + 3.0 * _RD ** 2)


def map_probability(points_a: float, points_b: float) -> float:
    return 1.0 / (1.0 + 10.0 ** (_G * (points_b - points_a) / 400.0))


class Standings:
    def __init__(self, documents: list[str]):
        snaps = [_parse_standings(_keyed_standings(d)) for d in documents]
        snaps.sort(key=lambda s: s.observed)  # stable, like Rust's sort_by_key
        self.snaps = snaps

    def snapshot_before(self, cutoff: int) -> Snapshot | None:
        for s in reversed(self.snaps):
            if s.observed < cutoff:
                return s
        return None

    def estimate(self, name_a: str, name_b: str, fmt: str, cutoff: int):
        """(map p, series p, observed, points a, points b, rank a, rank b) or None."""
        s = self.snapshot_before(cutoff)
        if s is None:
            return None
        a, b = team_key(name_a).lower(), team_key(name_b).lower()
        if a not in s.points or b not in s.points:
            return None
        p = map_probability(s.points[a], s.points[b])
        return p, series_probability(p, fmt), s.observed, s.points[a], s.points[b]


# ---------------------------------------------------------------- markets

def _normalize(a: float, b: float):
    if not (0.0 < a < 1.0 and 0.0 < b < 1.0):
        return None
    p = a / (a + b)
    return p, 1.0 - p


def _last(items):
    """Rust's `max_by_key(at)`: the last of the latest."""
    best = None
    for at, v in items:
        if best is None or at >= best[0]:
            best = (at, v)
    return best


@dataclass
class Quote:
    p: float  # team A's fair series probability
    observed: int
    source: str


def _same_match(m: ReplayMatch, keys, start: int | None):
    """`same_match` on names already keyed: which side team A is, or None."""
    if len(keys) != 2 or m.scheduled_start is None or start is None:
        return None
    if abs(start - m.scheduled_start) > 6 * T.HOUR:
        return None
    a, b = team_key(m.team_a.name), team_key(m.team_b.name)
    x, y = keys
    if a == x and b == y:
        return 0
    if a == y and b == x:
        return 1
    return None


def kalshi_start(event_ticker: str) -> int | None:
    parts = event_ticker.split("-")
    if len(parts) < 2 or len(parts[1]) < 11:
        return None
    try:
        local = datetime.strptime(parts[1][:11], "%y%b%d%H%M")
    except ValueError:
        return None

    def sunday(month, nth):
        first = datetime(local.year, month, 1)
        to_sunday = (6 - first.weekday()) % 7  # Monday=0 ... Sunday=6
        return first + timedelta(days=to_sunday + 7 * (nth - 1), hours=2)

    daylight = sunday(3, 2) <= local < sunday(11, 1)
    utc = local + timedelta(hours=4 if daylight else 5)
    return T.parse(utc.strftime("%Y-%m-%dT%H:%M:%SZ"))


PINNACLE, FREE_PINNACLE, WINNER, PARTICIPANTS = "pinnacle", "pinnacle+30", "171", ("171", "172")


def _pair(keys) -> frozenset | None:
    return frozenset(keys) if len(keys) == 2 else None


def _index(items, keys_of):
    """Items grouped by their unordered team-key pair, original order kept: a Match only ever
    matches a listing of its own two teams, so scanning the group is the same as scanning all."""
    out: dict = {}
    for item in items:
        k = _pair(keys_of(item))
        if k is not None:
            out.setdefault(k, []).append(item)
    return out


class Markets:
    def __init__(self, archive: Archive):
        self.lists = archive.fixture_lists
        self.histories = archive.histories
        self.polymarket = archive.polymarket_events
        self.polymarket_histories = archive.polymarket_histories
        self.kalshi_candles = archive.kalshi_candles
        events: dict[str, list] = {}
        for mk in archive.kalshi_markets:
            e = mk.get("event_ticker")
            if isinstance(e, str):
                events.setdefault(e, []).append(mk)
        self.kalshi_events = [
            (e, mks, [team_key(mk.get("yes_sub_title") or "") for mk in mks], kalshi_start(e))
            for e, mks in sorted(events.items())  # BTreeMap order
        ]
        self._poly_lines = []
        for event in self.polymarket:
            line = next((mk for mk in event.get("markets") or [] if mk.get("sportsMarketType") == "moneyline"), None)
            if line is None:
                continue
            try:
                outcomes = json.loads(line.get("outcomes")) if isinstance(line.get("outcomes"), str) else []
            except ValueError:
                outcomes = []
            start = None
            gst = line.get("gameStartTime")
            if isinstance(gst, str):
                try:
                    dt = datetime.strptime(gst + "00", "%Y-%m-%d %H:%M:%S%z")
                    start = T.to_us(dt)
                except ValueError:
                    start = None
            self._poly_lines.append((event, [team_key(o) for o in outcomes], start))
        self._poly_index = _index(self._poly_lines, lambda x: x[1])
        self._kalshi_index = _index(self.kalshi_events, lambda x: x[2])
        self._fixture_index = [
            _index(fx, lambda f: [team_key(f.get("participant1Name") or ""), team_key(f.get("participant2Name") or "")]
                   if isinstance(f.get("participant1Name"), str) and isinstance(f.get("participant2Name"), str) else [])
            for fx in self.lists]

    # OddsPapi -----------------------------------------------------------
    def _settled(self, fixtures, m: ReplayMatch):
        if m.scheduled_start is None:
            return None
        a, b = team_key(m.team_a.name), team_key(m.team_b.name)
        if not a or not b or a == b:
            return None
        nearest, distance = None, None
        for f in fixtures:
            if f.get("statusId") != 2:
                continue
            fid = f.get("fixtureId")
            n1, n2 = f.get("participant1Name"), f.get("participant2Name")
            st, tst = T.parse(f.get("startTime")), T.parse(f.get("trueStartTime"))
            if not isinstance(fid, str) or not isinstance(n1, str) or not isinstance(n2, str) or st is None or tst is None:
                continue
            k1, k2 = team_key(n1), team_key(n2)
            if a == k1 and b == k2:
                side = 0
            elif a == k2 and b == k1:
                side = 1
            else:
                continue
            delta = abs(m.scheduled_start - st)
            if delta > 6 * T.HOUR:
                continue
            if distance is None or delta < distance:
                distance, nearest = delta, (fid, side, min(m.scheduled_start - T.HARD_LOCK_LEAD, tst))
            elif delta == distance:
                nearest = None
        return nearest

    def oddspapi(self, m: ReplayMatch, cutoff: int) -> Quote | None:
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

            def latest(book, outcome):
                ticks = (((((history.get("bookmakers") or {}).get(book) or {}).get("markets") or {}).get(WINNER) or {})
                         .get("outcomes") or {}).get(outcome) or {}
                ticks = (ticks.get("players") or {}).get("0")
                if not isinstance(ticks, list):
                    return None
                best = _last((t, tick) for tick in ticks if (t := T.parse(tick.get("createdAt"))) is not None and t < cut)
                if best is None or best[1].get("active") is not True:
                    return None
                return best

            for book in (PINNACLE, FREE_PINNACLE):
                la, lb = latest(book, PARTICIPANTS[0]), latest(book, PARTICIPANTS[1])
                if la is None or lb is None:
                    continue
                pa, pb = la[1].get("price"), lb[1].get("price")
                if not isinstance(pa, (int, float)) or not isinstance(pb, (int, float)):
                    continue
                fair = _normalize(1.0 / pa, 1.0 / pb)
                if fair is None:
                    continue
                p = fair[0] if side == 0 else 1.0 - fair[0]
                return Quote(p, max(la[0], lb[0]), "oddspapi")
            return None
        return None

    # Polymarket ---------------------------------------------------------
    def polymarket_quote(self, m: ReplayMatch, cutoff: int) -> Quote | None:
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
                if isinstance(t, int) and isinstance(p, (int, float)):
                    at = T.from_seconds(t)
                    if at < cutoff:
                        pts.append((at, float(p)))
            last = _last(pts)
            if last is not None and 0.0 < last[1] < 1.0:
                return Quote(last[1] if side == 0 else 1.0 - last[1], last[0], "polymarket")
            return None
        return None

    # Kalshi -------------------------------------------------------------
    def kalshi(self, m: ReplayMatch, cutoff: int) -> Quote | None:
        pair = frozenset((team_key(m.team_a.name), team_key(m.team_b.name)))
        for event, markets, keys, start in self._kalshi_index.get(pair, []):
            if len(markets) != 2:
                continue
            side = _same_match(m, keys, start)
            if side is None:
                continue

            def mid(mk):
                candles = self.kalshi_candles.get(mk.get("ticker"))
                if candles is None:
                    return None
                pts = []
                for c in candles.get("candlesticks") or []:
                    ts = c.get("end_period_ts")
                    if not isinstance(ts, int):
                        continue
                    try:
                        bid = float(c["yes_bid"]["close_dollars"])
                        ask = float(c["yes_ask"]["close_dollars"])
                    except (KeyError, TypeError, ValueError):
                        continue
                    at = T.from_seconds(ts)
                    if at < cutoff:
                        pts.append((at, (bid + ask) / 2.0))
                return _last(pts)

            ma, mb = mid(markets[0]), mid(markets[1])
            if ma is None or mb is None:
                return None
            a, b = ma[1], mb[1]
            if not (a > 0.0 and b > 0.0):
                return None
            p_first = a / (a + b)
            return Quote(p_first if side == 0 else 1.0 - p_first, max(ma[0], mb[0]), "kalshi")
        return None

    def quote(self, m: ReplayMatch, cutoff: int) -> Quote | None:
        for fn in (self.oddspapi, self.polymarket_quote, self.kalshi):
            q = fn(m, cutoff)
            if q is not None:
                assert q.observed < cutoff, "market datum at or after the cutoff"
                return q
        return None


# ---------------------------------------------------------------- the anchor

MARKET_WEIGHT, STAT_WEIGHT = 0.80, 0.20


def logit(p: float) -> float:
    p = min(max(p, 1e-9), 1.0 - 1e-9)
    return math.log(p / (1.0 - p))


def anchor(market: float | None, rating: float | None) -> float | None:
    if market is not None and rating is not None:
        z = MARKET_WEIGHT * logit(market) + STAT_WEIGHT * logit(rating)
        return 1.0 / (1.0 + math.exp(-z))
    if market is not None:
        return market
    return rating
