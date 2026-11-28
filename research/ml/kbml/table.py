"""One Match table (plus an HLTV map table) built from the archive, cached as parquet.

Rules follow the Rust lab (`lab/runner.rs`, `lab/holdout.rs`):
- Event = PandaScore serie; title = "<league> <serie full name>"; tier = the first Stage (in
  cutoff order) that names one; qualifier = title contains `RMR` or `Qualifier`.
- Event end = the latest known finish of all its Stages; `None` if any Stage is incomplete or a
  scored result has no reported finish.
- Holdout = a Stage that begins at or after 2026-11-25 23:46:46 UTC, or whose Event has a result
  at or after it or is not complete, except PGL Singapore 2026 Stage 1 (PandaScore 22378).
  Holdout Matches are dropped here: no column of theirs (winner, market, rating) is computed or
  cached, and nothing downstream can read them. Only their count is kept.
"""
from __future__ import annotations

import json
import time
from pathlib import Path

import pandas as pd

from . import paths
from . import timeutil as T
from .anchor import Markets, Standings, anchor
from .archive import Archive, load
from .teams import team_key

SINGAPORE_STAGE_1 = 22378
CACHE_VERSION = "kbml-table/3"

#: Evaluation scope (coordinator, 2026-11-27): tier S/A, not a qualifier, from the start of the
#: research archive. Its first Event (BLAST Premier Groups Fall 2024) begins 2024-07-29 and is
#: scored by signal-lab/4, so the scope starts that day rather than 2024-08-01, keeping parity.
#: Rows outside the scope (tier B, older Events, qualifiers) only feed training (never a
#: qualifier) and point-in-time history.
SCOPE_TIERS = ("s", "a")
SCOPE_FROM = T.parse("2024-07-29T00:00:00Z")


def in_scope(tier, qualifier: bool, event_begins: int) -> bool:
    return tier in SCOPE_TIERS and not qualifier and event_begins >= SCOPE_FROM


def is_qualifier(event: str) -> bool:
    name = event.lower()
    return "rmr" in name or "qualifier" in name


def event_ends(archive: Archive) -> dict[int, int | None]:
    ends: dict[int, int | None] = {}
    seen: set[int] = set()
    for stage in archive.stages:
        end = stage.known_end()
        if stage.event_id not in seen:
            seen.add(stage.event_id)
            ends[stage.event_id] = end
        else:
            prev = ends[stage.event_id]
            ends[stage.event_id] = None if prev is None or end is None else max(prev, end)
    return ends


def holdout_stage_ids(archive: Archive, ends=None, boundary: int = T.SPEC_PUBLISHED) -> set[int]:
    ends = event_ends(archive) if ends is None else ends
    out = set()
    for s in archive.stages:
        if s.id == SINGAPORE_STAGE_1:
            continue
        last = ends[s.event_id]
        if s.cutoff >= boundary or last is None or last >= boundary:
            out.add(s.id)
    return out


def _hltv_links(archive: Archive):
    """`player_history::Linker`: same canonical team pair, nearest actual start within ±6 h."""
    by_pair: dict[tuple[str, str], list] = {}
    for stage in archive.stages:
        for m in stage.matches:
            ka, kb = team_key(m.team_a.name), team_key(m.team_b.name)
            pair = (ka, kb) if ka <= kb else (kb, ka)
            by_pair.setdefault(pair, []).append((m.actual_start, m.id))
    links = {}
    for rec in archive.hltv:
        ka, kb = team_key(rec["team_a"]["name"]), team_key(rec["team_b"]["name"])
        pair = (ka, kb) if ka <= kb else (kb, ka)
        started = T.parse(rec["started_at"])
        best = None
        for actual, mid in by_pair.get(pair, []):
            d = abs(actual - started)
            if d <= 6 * T.HOUR and (best is None or d < best[0]):  # min_by_key keeps the first
                best = (d, mid)
        if best is not None:
            links[rec["match_id"]] = best[1]
    return links


def reference_stages(reference_dir: Path) -> set[int]:
    """PandaScore tournament ids whose Match lists the reference archive holds."""
    out = set()
    for p in Path(reference_dir).glob("panda-*.json*"):
        head = p.name[len("panda-"):].split("-")[0]
        if head.isdigit():
            out.add(int(head))
    return out


def separate_new_stages(archive: Archive, reference: set[int]) -> int:
    """A Stage the reference archive lacks never changes an Event of the reference: if it shares a
    PandaScore serie with reference Stages, it forms its own Event (id = −serie). Returns the count."""
    ref_events = {s.event_id for s in archive.stages if s.id in reference}
    moved = 0
    for s in archive.stages:
        if s.id not in reference and s.event_id in ref_events:
            s.event_id = -s.event_id
            for m in s.matches:
                m.event_id = s.event_id
            moved += 1
    return moved


def block_of(tier, qualifier: bool, scope: bool) -> str:
    """Training block of a row: scope (evaluated), sa_old (tier S/A before the scope), b, c,
    qualifier (never fitted), other (no tier)."""
    if scope:
        return "scope"
    if qualifier:
        return "qualifier"
    if tier in ("s", "a"):
        return "sa_old"
    return tier if tier in ("b", "c") else "other"


def build(archive_dir: Path = paths.ARCHIVE, log=print, reference_dir: Path | None = None
          ) -> tuple[pd.DataFrame, pd.DataFrame, dict]:
    t0 = time.time()
    archive = load(archive_dir)
    moved = 0
    if reference_dir is not None and Path(reference_dir).resolve() != Path(archive_dir).resolve():
        moved = separate_new_stages(archive, reference_stages(reference_dir))
    log(f"archive read: {archive.files} files, {len(archive.stages)} Stages, {len(archive.hltv)} HLTV records "
        f"({time.time() - t0:.0f}s)")
    standings = Standings(archive.standings)
    markets = Markets(archive)
    ends = event_ends(archive)
    holdout = holdout_stage_ids(archive, ends)
    holdout_matches = {m.id for s in archive.stages if s.id in holdout for m in s.matches}

    # Event title/tier/begins as the lab's `side()` builds them (training side only).
    events: dict[int, dict] = {}
    for s in archive.stages:
        if s.id in holdout:
            continue
        e = events.setdefault(s.event_id, {"event": s.event, "tier": None, "begins": s.cutoff})
        e["begins"] = min(e["begins"], s.cutoff)
        if e["tier"] is None:
            e["tier"] = s.tier

    hltv_by_match: dict[int, dict] = {}
    if paths.LINKS:
        # explicit Match -> page list (one page per Match, one Match per page)
        links = pd.read_csv(paths.LINKS, sep="	", usecols=["panda_id", "hltv_match_id"])
        by_id = {rec["match_id"]: rec for rec in archive.hltv}
        for pid, hid in zip(links["panda_id"], links["hltv_match_id"]):
            rec = by_id.get(int(hid))
            if rec is not None and int(pid) not in holdout_matches:
                hltv_by_match.setdefault(int(pid), rec)
        log(f"links file {paths.LINKS}: {len(hltv_by_match)} of {len(links)} links have a parsed page")
    else:
        hltv_links = _hltv_links(archive)
        for rec in archive.hltv:
            mid = hltv_links.get(rec["match_id"])
            if mid is not None and mid not in holdout_matches:
                hltv_by_match.setdefault(mid, rec)

    rows = []
    seen = set()
    for s in archive.stages:
        if s.id in holdout:
            continue
        e = events[s.event_id]
        lp_title = archive.event_titles.get(s.event_id)
        revisions = archive.event_pages.get(lp_title, []) if lp_title else []
        for m in s.matches:
            cutoff = m.cutoff()
            vrs = standings.estimate(m.team_a.name, m.team_b.name, m.format, cutoff)
            q = markets.quote(m, cutoff)
            rating = vrs[1] if vrs else None
            market = q.p if q else None
            roster = next((r for r in reversed(revisions) if r.timestamp < cutoff), None)
            rec = hltv_by_match.get(m.id)
            rows.append({
                "match_id": m.id,
                "duplicate": m.id in seen,
                "event_id": s.event_id,
                "event": e["event"],
                "tier": e["tier"],
                "is_qualifier": is_qualifier(s.event),
                "in_scope": in_scope(e["tier"], is_qualifier(s.event), e["begins"]),
                "block": block_of(e["tier"], is_qualifier(s.event), in_scope(e["tier"], is_qualifier(s.event), e["begins"])),
                "event_begins": e["begins"],
                "event_ended": ends[s.event_id],
                "stage_id": s.id,
                "stage": s.name,
                "stage_kind": s.kind,
                "stage_cutoff": s.cutoff,
                "name": m.name,
                "round": m.round,
                "team_a_id": m.team_a.id,
                "team_a": m.team_a.name,
                "team_a_key": team_key(m.team_a.name),
                "team_b_id": m.team_b.id,
                "team_b": m.team_b.name,
                "team_b_key": team_key(m.team_b.name),
                "format": m.format,
                "scheduled_start": m.scheduled_start,
                "actual_start": m.actual_start,
                "cutoff": cutoff,
                "known_finish": m.known_finish(),
                "finished_at": m.finished_at,
                "winner_id": m.winner,
                "team_a_won": m.winner == m.team_a.id,
                "maps_a": m.maps_score[0] if m.maps_score else None,
                "maps_b": m.maps_score[1] if m.maps_score else None,
                "market_p": market,
                "market_source": q.source if q else None,
                "market_observed": q.observed if q else None,
                "vrs_map_p": vrs[0] if vrs else None,
                "vrs_series_p": rating,
                "vrs_observed": vrs[2] if vrs else None,
                "vrs_points_a": vrs[3] if vrs else None,
                "vrs_points_b": vrs[4] if vrs else None,
                "anchor_p": anchor(market, rating),
                "hltv_match_id": rec["match_id"] if rec else None,
                "hltv_file": rec.get("_file") if rec else None,
                "hltv_is_lan": rec.get("is_lan") if rec else None,
                "lp_event_title": lp_title,
                "lp_roster_file": roster.file if roster else None,
                "lp_roster_revision": roster.timestamp if roster else None,
            })
            seen.add(m.id)
    matches = pd.DataFrame(rows)
    matches["date"] = pd.to_datetime(matches["cutoff"], unit="us", utc=True)
    for c in ("round", "maps_a", "maps_b", "vrs_points_a", "vrs_points_b", "hltv_match_id"):
        matches[c] = matches[c].astype("Int64")
    for c in ("event_ended", "known_finish", "market_observed", "vrs_observed", "lp_roster_revision"):
        matches[c] = matches[c].astype("Int64")

    # Played maps of linked HLTV records, visible from the linked Match's known finish.
    finish = dict(zip(matches.match_id, matches.known_finish))
    map_rows = []
    for mid, rec in hltv_by_match.items():
        ka, kb = team_key(rec["team_a"]["name"]), team_key(rec["team_b"]["name"])
        for mp in rec.get("maps") or []:
            if not mp.get("played") or mp.get("winner") not in ("a", "b"):
                continue
            map_rows.append({
                "match_id": mid,
                "hltv_match_id": rec["match_id"],
                "known_finish": finish.get(mid),
                "started_at": T.parse(rec["started_at"]),
                "is_lan": rec.get("is_lan"),
                "position": mp.get("position"),
                "map_name": mp.get("map_name"),
                "team_a_key": ka,
                "team_b_key": kb,
                "rounds_a": mp.get("team_a_score"),
                "rounds_b": mp.get("team_b_score"),
                "a_won": mp.get("winner") == "a",
            })
    maps = pd.DataFrame(map_rows)
    if len(maps):
        maps["known_finish"] = maps["known_finish"].astype("Int64")
        maps["rounds_a"] = maps["rounds_a"].astype("Int64")
        maps["rounds_b"] = maps["rounds_b"].astype("Int64")
    meta = {
        "version": CACHE_VERSION,
        "archive": str(archive_dir),
        "files": archive.files,
        "stages": len(archive.stages),
        "holdout_stages": sorted(holdout),
        "holdout_matches_dropped": len(holdout_matches),
        "unknown_files": archive.unknown[:50],
        "reference": str(reference_dir) if reference_dir else None,
        "links": paths.LINKS,
        "stages_separated_from_reference_events": moved,
        "build_seconds": round(time.time() - t0, 1),
    }
    return matches, maps, meta


def cache_dir(archive_dir: Path = paths.ARCHIVE) -> Path:
    return paths.CACHE / Path(archive_dir).name


def load_table(archive_dir: Path = paths.ARCHIVE, rebuild: bool = False, log=print,
               reference_dir: Path | None = paths.ARCHIVE):
    """(matches, maps, meta), from the parquet cache when present."""
    d = cache_dir(archive_dir)
    mf, pf, jf = d / "matches.parquet", d / "maps.parquet", d / "meta.json"
    if not rebuild and mf.exists() and jf.exists():
        meta = json.loads(jf.read_text())
        if meta.get("version") == CACHE_VERSION and meta.get("links") == paths.LINKS:
            return pd.read_parquet(mf), pd.read_parquet(pf), meta
    matches, maps, meta = build(archive_dir, log=log, reference_dir=reference_dir)
    d.mkdir(parents=True, exist_ok=True)
    matches.to_parquet(mf, index=False)
    maps.to_parquet(pf, index=False)
    jf.write_text(json.dumps(meta, indent=1))
    log(f"cached under {d}")
    return matches, maps, meta


if __name__ == "__main__":
    m, mp, meta = load_table(rebuild=True)
    print(meta)
    print(m.head())
