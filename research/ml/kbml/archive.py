"""Reads the research archive as `sidecar/src/adapters/backtest/archive.rs` does.

Only what the Match table and the anchor need: PandaScore Match lists (the ground truth),
Valve global standings, the three market sources (OddsPapi, Polymarket, Kalshi), Liquipedia
Event revisions (rosters) and parsed HLTV match records. What a file is, is read from its
content, in the same order of tests as `ReplayArchive::load`; files are visited in name order.
"""
from __future__ import annotations

import gzip
import json
from dataclasses import dataclass, field
from pathlib import Path

from . import timeutil as T


def read_bytes(path: Path) -> bytes:
    raw = path.read_bytes()
    return gzip.decompress(raw) if path.name.endswith(".gz") else raw


@dataclass
class Team:
    id: int
    name: str


@dataclass
class ReplayMatch:
    id: int
    event_id: int
    stage_id: int
    team_a: Team
    team_b: Team
    format: str  # bo1 | bo3 | bo5
    scheduled_start: int | None
    actual_start: int
    finished_at: int
    reported_finish: int | None
    winner: int
    maps_score: tuple[int, int] | None
    name: str
    round: int | None

    def cutoff(self) -> int:
        """`match_cutoff`: the earlier of Scheduled Start − 15 min and the actual start."""
        sched = self.scheduled_start if self.scheduled_start is not None else self.actual_start
        return min(sched - T.HARD_LOCK_LEAD, self.actual_start)

    def known_finish(self) -> int | None:
        """The reported finish if it is after the start; else unknown (never visible)."""
        if self.reported_finish is not None and self.reported_finish > self.actual_start:
            return self.reported_finish
        return None


@dataclass
class ReplayStage:
    id: int
    event_id: int
    event: str
    tier: str | None
    name: str
    kind: str
    teams: dict[int, Team]
    matches: list[ReplayMatch]
    unscored: int
    complete: bool
    unscored_end: int | None
    cutoff: int

    def known_end(self) -> int | None:
        if not self.complete:
            return None
        ends = []
        for m in self.matches:
            f = m.known_finish()
            if f is None:
                return None
            ends.append(f)
        if self.unscored_end is not None:
            ends.append(self.unscored_end)
        return max(ends) if ends else None


@dataclass
class WikiRevision:
    title: str
    timestamp: int
    file: str


@dataclass
class Archive:
    stages: list[ReplayStage] = field(default_factory=list)
    standings: list[str] = field(default_factory=list)
    fixture_lists: list[list] = field(default_factory=list)
    histories: dict[str, dict] = field(default_factory=dict)
    polymarket_events: list[dict] = field(default_factory=list)
    polymarket_histories: dict[str, dict] = field(default_factory=dict)
    kalshi_markets: list[dict] = field(default_factory=list)
    kalshi_candles: dict[str, dict] = field(default_factory=dict)
    event_pages: dict[str, list[WikiRevision]] = field(default_factory=dict)
    event_titles: dict[int, str] = field(default_factory=dict)
    hltv: list[dict] = field(default_factory=list)
    unknown: list[str] = field(default_factory=list)
    files: int = 0


def _first(value):
    if isinstance(value, list):
        return value[0] if value else None
    return None


def _get(obj, key):
    return obj.get(key) if isinstance(obj, dict) else None


def load(directory: Path, with_hltv: bool = True) -> Archive:
    directory = Path(directory)
    archive = Archive()
    names = sorted(p for p in directory.iterdir() if p.is_file())
    matches: dict[int, list] = {}
    for path in names:
        stem = path.name[:-3] if path.name.endswith(".gz") else path.name
        if not (stem.endswith(".json") or stem.endswith(".md")):
            continue
        data = read_bytes(path)
        archive.files += 1
        if stem.endswith(".md"):
            if stem.startswith("standings_global_"):
                archive.standings.append(data.decode("utf-8"))
            continue
        value = json.loads(data)
        first = _first(value)
        if stem.startswith("panda-") and value == []:
            continue
        if stem.startswith("maps-"):
            continue  # post-Match Liquipedia map pages: not needed here
        if isinstance(_get(first, "slug"), str) and isinstance(_get(first, "markets"), list):
            archive.polymarket_events.extend(value)
        elif isinstance(_get(value, "history"), list):
            slug = stem[: -len(".json")]
            if not slug.startswith("polymarket-history-"):
                raise ValueError(f"{path.name}: a price history is named by its event slug")
            archive.polymarket_histories[slug[len("polymarket-history-"):]] = value
        elif isinstance(_get(value, "markets"), list) and "cursor" in value:
            archive.kalshi_markets.extend(value["markets"])
        elif isinstance(_get(value, "candlesticks"), list):
            archive.kalshi_candles[value["ticker"]] = value
        elif isinstance(_get(first, "fixtureId"), str):
            archive.fixture_lists.append(value)
        elif isinstance(_get(value, "fixtureId"), str) and isinstance(_get(value, "bookmakers"), dict):
            archive.histories[value["fixtureId"]] = value
        elif isinstance(_get(first, "tournament_id"), int) and isinstance(_get(first, "opponents"), list):
            matches.setdefault(first["tournament_id"], []).extend(value)
        elif isinstance(_first(_get(_get(value, "query"), "pages")), dict):
            page = value["query"]["pages"][0]
            roster_event = None
            if stem.startswith("roster-"):
                head = stem[len("roster-"):].split("-", 1)[0]
                roster_event = int(head) if head.isdigit() else None
            revs = page.get("revisions")
            if roster_event is not None and not revs:
                continue
            rev = revs[0]
            ts = T.parse(rev.get("timestamp"))
            title = page.get("title")
            if title is None or ts is None:
                raise ValueError(f"{path.name}: no revision")
            if roster_event is not None:
                archive.event_titles[roster_event] = title
            if stem.startswith("lp-"):
                continue  # Stage opening views (Pick'em): not needed here
            archive.event_pages.setdefault(title, []).append(WikiRevision(title, ts, path.name))
        elif isinstance(value, dict) and "match_id" in value and "overall_stats" in value:
            archive.hltv.append(value)
        else:
            archive.unknown.append(path.name)
    if with_hltv:
        _load_hltv_dir(directory, archive)
    for revs in archive.event_pages.values():
        revs.sort(key=lambda r: r.timestamp)
    for tid in sorted(matches):
        archive.stages.append(stage_from_pandascore(tid, matches[tid]))
    archive.stages.sort(key=lambda s: (s.cutoff, s.id))
    return archive


def _load_hltv_dir(directory: Path, archive: Archive) -> None:
    """Parsed `HltvMatchRecord` JSONs under `hltv/` and `hltv/matches/` (raw pages are not parsed here)."""
    paths = []
    for sub in (directory / "hltv", directory / "hltv" / "matches"):
        if sub.is_dir():
            paths.extend(p for p in sub.iterdir() if p.is_file())
    for path in sorted(paths):
        if "-blocked." in path.name:
            continue
        stem = path.name[:-3] if path.name.endswith(".gz") else path.name
        if not stem.endswith(".json"):
            continue
        value = json.loads(read_bytes(path))
        if isinstance(value, dict) and "match_id" in value and "overall_stats" in value:
            value["_file"] = str(path)
            archive.hltv.append(value)
    # One record per HLTV match id: the earliest fetch kept.
    archive.hltv.sort(key=lambda r: (r["match_id"], T.parse(r.get("fetched_at")) or 0))
    unique = {}
    for r in archive.hltv:
        unique.setdefault(r["match_id"], r)
    archive.hltv = sorted(unique.values(), key=lambda r: (T.parse(r["started_at"]), r["match_id"]))


FORMATS = {1: "bo1", 3: "bo3", 5: "bo5"}


def _stage_kind(name: str, event: str) -> str:
    n = name.lower()
    if "playoff" in n:
        return "playoffs"
    if "group" in n:
        return "groups"
    if "swiss" in n or ("major" in event.lower() and name in
                        ("Elimination Stage", "Opening Stage", "Stage 1", "Stage 2", "Stage 3")):
        return "swiss"
    return "other"


def stage_from_pandascore(tid: int, rows: list) -> ReplayStage:
    first = rows[0]
    name = (first.get("tournament") or {}).get("name") or ""
    event = f"{(first.get('league') or {}).get('name') or ''} {(first.get('serie') or {}).get('full_name') or ''}".strip()
    event_id = first["serie_id"]
    teams: dict[int, Team] = {}
    for row in rows:
        for o in row.get("opponents") or []:
            t = o.get("opponent") or {}
            if isinstance(t.get("id"), int) and isinstance(t.get("name"), str):
                teams[t["id"]] = Team(t["id"], t["name"])
    cutoff = None
    matches: list[ReplayMatch] = []
    unscored = 0
    complete = True
    unscored_end = None
    for row in rows:
        status = row.get("status") or ""
        complete &= status in ("finished", "canceled")
        scheduled = T.parse(row.get("scheduled_at"))
        actual = T.parse(row.get("begin_at"))
        known = [t for t in (scheduled, actual) if t is not None]
        if known and status != "canceled":
            earliest = min(known)
            cutoff = earliest if cutoff is None else min(cutoff, earliest)
        fmt = FORMATS.get(row.get("number_of_games"))
        pair = [teams[o["opponent"]["id"]] for o in (row.get("opponents") or [])
                if isinstance((o.get("opponent") or {}).get("id"), int) and o["opponent"]["id"] in teams]
        rname = row.get("name") or ""
        rnd = None
        if rname.startswith("Round "):
            head = rname[len("Round "):].split(":")[0].strip()
            rnd = int(head) if head.isdigit() else None
        if fmt is None or len(pair) != 2:
            unscored += 1
            continue
        a, b = pair
        winner = row.get("winner_id")
        maps_score = None
        res = row.get("results")
        if isinstance(res, list):
            def score_of(tid_):
                for r in res:
                    if r.get("team_id") == tid_ and isinstance(r.get("score"), int) and r["score"] >= 0:
                        return r["score"]
                return None
            sa, sb = score_of(a.id), score_of(b.id)
            if sa is not None and sb is not None:
                maps_score = (sa, sb)
        if (status == "finished" and row.get("forfeit") is False and isinstance(winner, int)
                and scheduled is not None and actual is not None and winner in (a.id, b.id)):
            end = T.parse(row.get("end_at"))
            matches.append(ReplayMatch(
                id=row["id"], event_id=event_id, stage_id=tid, team_a=a, team_b=b, format=fmt,
                scheduled_start=scheduled, actual_start=actual,
                finished_at=max(end if end is not None else actual, actual),
                reported_finish=end, winner=winner, maps_score=maps_score, name=rname, round=rnd))
        else:
            unscored += 1
            if status == "finished":
                end = T.parse(row.get("end_at"))
                if end is not None:
                    unscored_end = end if unscored_end is None else max(unscored_end, end)
    matches.sort(key=lambda m: (m.cutoff(), m.id))
    if cutoff is None:
        raise ValueError(f"stage {tid}: no Match has a start time")
    tier = (first.get("tournament") or {}).get("tier")
    tier = tier.strip().lower() if isinstance(tier, str) and tier.strip() else None
    return ReplayStage(id=tid, event_id=event_id, event=event, tier=tier, name=name,
                       kind=_stage_kind(name, event), teams=teams, matches=matches,
                       unscored=unscored, complete=complete, unscored_end=unscored_end, cutoff=cutoff)
