"""HLTV data for the feature groups, joined to the Match table, point in time.

- `pre` (per Match, from that Match's own page): oriented so side 0 is PandaScore team A. Rows a
  page lists that are not strictly earlier are dropped here as a guard (the audit found none):
  a past-matches row that is the Match itself or a linked Match whose cutoff is not earlier, and
  a head-to-head row dated at or after the cutoff. The counts are in `meta["guards"]`.
- HLTV rank (the badge's weekly ranking, `backtest/rank_history.rs`): known from the end of the
  ranking's UTC day, read strictly before the cutoff, at most 35 days old; any linked page may tell
  a team's rank. Columns `hltv_rank_a/b` (now), `hltv_rank28_a/b` and `hltv_rank84_a/b` (as of
  28/84 days before the cutoff, for momentum).
- `records` (per linked Match): everything the Match produced (maps, veto, player stats overall,
  per map and per side, CT/T half scores) and its lineups, for later Matches only: the timeline
  releases a record once the Match's known finish is strictly before the cutoff asked for.
"""
from __future__ import annotations

import gzip
import json
import pickle
from concurrent.futures import ProcessPoolExecutor
from datetime import datetime, timezone
from pathlib import Path

import numpy as np
import pandas as pd

from . import timeutil as T
from .hltv_html import parse_file, parse_ranking
from .teams import team_key

CACHE_VERSION = "kbml-hltv/2"
MAX_AGE = 35 * T.DAY


def _parse(path: str):
    try:
        return path, parse_file(Path(path))
    except Exception as e:  # a page that does not parse is coverage, not an error
        return path, {"error": repr(e)}


def parse_pages(paths: list[str], cache: Path, log=print) -> dict[str, dict]:
    if cache.exists():
        stored = pickle.loads(cache.read_bytes())
        if stored.get("version") == CACHE_VERSION and set(paths) <= set(stored["pages"]):
            return stored["pages"]
    log(f"parsing {len(paths)} HLTV pages ...")
    with ProcessPoolExecutor() as ex:
        pages = dict(ex.map(_parse, paths, chunksize=16))
    cache.parent.mkdir(parents=True, exist_ok=True)
    cache.write_bytes(pickle.dumps({"version": CACHE_VERSION, "pages": pages}))
    return pages


def end_of_day(date: str) -> int:
    return T.parse(date + "T23:59:59.999999Z")


class RankHistory:
    def __init__(self):
        self.rows: dict[str, list[tuple[int, int]]] = {}

    def add(self, key: str, observed: int, rank: int):
        self.rows.setdefault(key, []).append((observed, rank))

    def finish(self):
        for v in self.rows.values():
            v.sort()  # by observed; the first saved page wins among equals (stable)

    def at(self, key: str, cutoff: int) -> int | None:
        best = None
        for obs, rank in self.rows.get(key, []):
            if obs < cutoff and (best is None or obs > best[0]):
                best = (obs, rank)
        if best is None or best[0] < cutoff - MAX_AGE:
            return None
        return best[1]


def _swap_pre(pre: dict) -> dict:
    out = dict(pre)
    for k in ("teams", "ranks", "lineups"):
        out[k] = list(reversed(pre[k])) if len(pre.get(k) or []) == 2 else pre.get(k)
    out["past"] = {k: list(reversed(v)) if len(v) == 2 else v for k, v in pre["past"].items()}
    out["streak"] = {k: list(reversed(v)) if len(v) == 2 else v for k, v in pre["streak"].items()}
    if pre.get("h2h") and pre["h2h"].get("summary"):
        s = pre["h2h"]["summary"]
        out["h2h"] = dict(pre["h2h"], summary=[s[2], s[1], s[0]])
    return out


RANK_MAX_AGE = 35 * T.DAY


def weekly_rankings(archive_dir: Path) -> list[dict]:
    """Every saved weekly HLTV world ranking, oldest first, each known from the end of its UTC day."""
    folder = Path(archive_dir) / "hltv" / "ranking"
    pages = []
    for f in sorted(folder.glob("hltv-ranking-*.html.gz")) if folder.is_dir() else []:
        page = parse_ranking(gzip.decompress(f.read_bytes()).decode("utf-8"))
        page["known"] = end_of_day(page["date"])
        pages.append(page)
    pages.sort(key=lambda p: p["known"])
    return pages


def weekly_columns(m: pd.DataFrame, pre_by_match: dict, pages: list[dict]) -> pd.DataFrame:
    """`rw_*` columns: each team's rank and points on the latest weekly ranking known strictly
    before the cutoff (≤ 35 days old; a missing week carries the previous one). The team is found
    by the HLTV id on the Match's own page (pre-match), else by name key. Unranked: NA, with
    `rw_size` (teams on that page) so a feature can impute."""
    known = np.array([p["known"] for p in pages], dtype=np.int64)
    by_id = [{t["team_id"]: t for t in p["teams"]} for p in pages]
    by_key = [{team_key(t["name"]): t for t in p["teams"]} for p in pages]
    cols = {c: [] for c in ("rw_rank_a", "rw_rank_b", "rw_points_a", "rw_points_b", "rw_size", "rw_known")}
    for mid, ka, kb, cut in zip(m.match_id, m.team_a_key, m.team_b_key, m.cutoff):
        i = int(np.searchsorted(known, int(cut), side="left")) - 1  # latest known strictly before
        if i < 0 or known[i] < int(cut) - RANK_MAX_AGE:
            for c in cols:
                cols[c].append(None)
            continue
        pre = pre_by_match.get(int(mid))
        ids = pre.get("page_team_ids") if pre else None
        found = []
        for side, key in ((0, ka), (1, kb)):
            t = by_id[i].get(ids[side]) if ids and ids[side] is not None else None
            found.append(t or by_key[i].get(key))
        cols["rw_rank_a"].append(found[0]["rank"] if found[0] else None)
        cols["rw_rank_b"].append(found[1]["rank"] if found[1] else None)
        cols["rw_points_a"].append(found[0]["points"] if found[0] else None)
        cols["rw_points_b"].append(found[1]["points"] if found[1] else None)
        cols["rw_size"].append(len(pages[i]["teams"]))
        cols["rw_known"].append(int(known[i]))
    for c, v in cols.items():
        m[c] = pd.array(v, dtype="Int64")
    return m


def build(matches: pd.DataFrame, cache_dir: Path, log=print, archive_dir: Path | None = None):
    """(matches with rank columns, pre by match_id, records frame, meta)."""
    linked = matches[matches["hltv_file"].notna()]
    html_paths = [f[: -len(".json")] + ".html.gz" for f in linked["hltv_file"]]
    pages = parse_pages([p for p in html_paths if Path(p).exists()], cache_dir / "hltv_pages.pkl", log)
    cutoff_of_hltv = dict(zip(linked["hltv_match_id"].astype(int), linked["cutoff"].astype("int64")))
    ranks = RankHistory()
    pre_by_match, records = {}, []
    guards = {"past_rows_dropped": 0, "h2h_rows_dropped": 0, "unoriented": 0, "unparsed": 0}
    for row, hpath in zip(linked.itertuples(index=False), html_paths):
        page = pages.get(hpath)
        rec = json.loads(Path(row.hltv_file).read_text(encoding="utf-8"))
        hid = int(row.hltv_match_id)
        keys = [team_key(rec["team_a"]["name"]), team_key(rec["team_b"]["name"])]
        if page is None or "error" in page:
            guards["unparsed"] += 1
            page = {"pre": None, "post": {"halves": [], "sides": {}}}
        pre = page["pre"]
        if pre is not None and len(pre["teams"]) == 2:
            # Rank badges (HLTV only; the VRS badge is never read).
            for side, badge in enumerate(pre["ranks"]):
                if badge:
                    ranks.add(keys[side], end_of_day(badge["date"]), badge["rank"])
            if keys[0] == row.team_a_key:
                oriented = pre
            elif keys[1] == row.team_a_key or keys[0] == row.team_b_key:
                oriented = _swap_pre(pre)
            else:
                guards["unoriented"] += 1
                oriented = None
            if oriented is not None:
                cut = int(row.cutoff)
                for kind, sides in oriented["past"].items():
                    for i, rows in enumerate(sides):
                        keep = [r for r in rows if r["mid"] != hid and
                                not (r["mid"] in cutoff_of_hltv and cutoff_of_hltv[r["mid"]] >= cut)]
                        guards["past_rows_dropped"] += len(rows) - len(keep)
                        sides[i] = keep
                if oriented.get("h2h"):
                    rows = oriented["h2h"]["rows"]
                    keep = [r for r in rows if r["unix_ms"] * 1000 < cut and r["mid"] != hid]
                    guards["h2h_rows_dropped"] += len(rows) - len(keep)
                    oriented["h2h"]["rows"] = keep
                oriented["page_team_ids"] = [t["id"] for t in oriented["teams"]]
                pre_by_match[int(row.match_id)] = oriented
        records.append({
            "match_id": int(row.match_id),
            "known_finish": row.known_finish,
            "rec": {
                "hltv_match_id": hid,
                "keys": keys,
                "team_ids": [rec["team_a"].get("id"), rec["team_b"].get("id")],
                "started_at": T.parse(rec["started_at"]),
                "format": rec.get("format"),
                "maps": rec.get("maps") or [],
                "veto": rec.get("veto") or [],
                "overall": rec.get("overall_stats") or [],
                "per_map": rec.get("per_map_stats") or [],
                "halves": page["post"]["halves"],
                "sides": page["post"]["sides"],
                "lineups": [[p["id"] for p in side] for side in pre["lineups"]] if pre and len(pre["lineups"]) == 2 else None,
                "igl": [[p["id"] for p in side if p["igl"]] for side in pre["lineups"]] if pre and len(pre["lineups"]) == 2 else None,
            },
        })
    ranks.finish()
    m = matches.copy()
    for col, back in (("hltv_rank", 0), ("hltv_rank28", 28 * T.DAY), ("hltv_rank84", 84 * T.DAY)):
        m[col + "_a"] = pd.array([ranks.at(k, int(c) - back) for k, c in zip(m.team_a_key, m.cutoff)], dtype="Int64")
        m[col + "_b"] = pd.array([ranks.at(k, int(c) - back) for k, c in zip(m.team_b_key, m.cutoff)], dtype="Int64")
    m = weekly_columns(m, pre_by_match, weekly_rankings(archive_dir) if archive_dir else [])
    rec_frame = pd.DataFrame(records)
    if len(rec_frame):
        rec_frame["known_finish"] = rec_frame["known_finish"].astype("Int64")
    meta = {"pages": len(pages), "linked": len(linked), "pre": len(pre_by_match), "guards": guards}
    return m, pre_by_match, rec_frame, meta


def load(matches: pd.DataFrame, cache_dir: Path, rebuild: bool = False, log=print, archive_dir: Path | None = None):
    """`build`, cached as one pickle next to the Match table."""
    f = cache_dir / "hltv_build.pkl"
    if not rebuild and f.exists():
        stored = pickle.loads(f.read_bytes())
        if stored.get("version") == CACHE_VERSION and stored["n"] == len(matches):
            return stored["matches"], stored["pre"], stored["records"], stored["meta"]
    m, pre, records, meta = build(matches, cache_dir, log, archive_dir)
    f.write_bytes(pickle.dumps({"version": CACHE_VERSION, "n": len(matches), "matches": m, "pre": pre,
                                "records": records, "meta": meta}))
    return m, pre, records, meta
