"""Collect the historical CS2 research archive. See docs/research-archive.md."""

import argparse
import csv
import gzip
import hashlib
import json
import re
import sqlite3
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from datetime import UTC, date, datetime, timedelta
from pathlib import Path

from collect_backtest import LIQUIPEDIA, ROOT, USER_AGENT, iso, parse_time

DEFAULT_CAPS = {"panda": 100, "valve": 60, "liquipedia": 100}
PANDA = "https://api.pandascore.co"
VALVE = "https://api.github.com/repos/ValveSoftware/counter-strike_regional_standings"
FIRST = date(2024, 8, 1)


class Budget(Exception):
    pass


def url(base, **params):
    return base + "?" + urllib.parse.urlencode(params)


class Archive:
    """Charge attempts before transport. Publish a file only after recording provenance."""

    def __init__(self, root, caps, fetch, secrets=()):
        self.root, self.caps, self.fetch = root, caps, fetch
        self.secrets = [s.encode() for s in secrets if s]
        self.root.mkdir(parents=True, exist_ok=True)
        self.db = sqlite3.connect(root / "collector.db")
        self.db.executescript("""
            CREATE TABLE IF NOT EXISTS spent(day TEXT, source TEXT, n INTEGER, PRIMARY KEY(day,source));
            CREATE TABLE IF NOT EXISTS files(path TEXT PRIMARY KEY, request TEXT, fetched TEXT, hash TEXT);
        """)
        self.manifest()

    def manifest(self):
        path = self.root / "sources.tsv"
        tmp = path.with_suffix(".tmp")
        with tmp.open("w", encoding="utf8", newline="") as f:
            rows = csv.writer(f, delimiter="\t", lineterminator="\n")
            rows.writerow(("file", "request", "fetched_at", "sha256"))
            for row in self.db.execute("SELECT * FROM files ORDER BY path"):
                if (self.root / row[0]).is_file():
                    rows.writerow(row)
        tmp.replace(path)

    def spent(self, source):
        day = datetime.now(UTC).date().isoformat()
        row = self.db.execute("SELECT n FROM spent WHERE day=? AND source=?", (day, source)).fetchone()
        return row[0] if row else 0

    def charge(self, source):
        day = datetime.now(UTC).date().isoformat()
        with self.db:
            self.db.execute("BEGIN IMMEDIATE")
            if self.spent(source) >= self.caps[source]:
                raise Budget(f"{source} daily cap reached")
            self.db.execute("INSERT INTO spent VALUES(?,?,1) ON CONFLICT(day,source) DO UPDATE SET n=n+1",
                            (day, source))

    def read(self, source, request, name):
        if Path(name).is_absolute() or ".." in Path(name).parts:
            raise ValueError("invalid archive path")
        relative = name + ".gz"
        path = self.root / relative
        row = self.db.execute("SELECT request,hash FROM files WHERE path=?", (relative,)).fetchone()
        if row and row[0] != request:
            raise ValueError(f"conflicting request for {relative}")
        if row and path.exists():
            body = gzip.decompress(path.read_bytes())
            if hashlib.sha256(body).hexdigest() != row[1]:
                raise ValueError(f"digest mismatch for {relative}")
        else:
            self.charge(source)
            body = self.fetch(source, request)
            if any(s in body or s in request.encode() for s in self.secrets):
                raise ValueError("credential in source response or provenance")
            if name.endswith(".json"):
                json.loads(body)
            path.parent.mkdir(parents=True, exist_ok=True)
            tmp = path.with_suffix(".partial")
            tmp.write_bytes(gzip.compress(body, mtime=0))
            with self.db:
                self.db.execute("INSERT OR REPLACE INTO files VALUES(?,?,?,?)",
                                (relative, request, iso(datetime.now(UTC)), hashlib.sha256(body).hexdigest()))
            tmp.replace(path)
            with (self.root / "sources.tsv").open("a", encoding="utf8", newline="") as f:
                record = self.db.execute("SELECT * FROM files WHERE path=?", (relative,)).fetchone()
                csv.writer(f, delimiter="\t", lineterminator="\n").writerow(record)
        return json.loads(body) if name.endswith(".json") else body


def month_ranges(until):
    start = FIRST
    while start <= until:
        following = (start.replace(day=28) + timedelta(days=4)).replace(day=1)
        yield start, min(until, following - timedelta(days=1))
        start = following


def collect_panda(a, until):
    for start, end in month_ranges(until):
        window = f"{start}T00:00:00Z,{end}T23:59:59Z"
        for page in range(1, sys.maxsize):
            query = url(PANDA + "/csgo/tournaments/past", **{
                "range[end_at]": window, "filter[tier]": "s,a", "sort": "id", "page": page, "per_page": 100})
            entries = a.read("panda", query, f"catalog/tournaments-{start}-{end}-{page}.json")
            for entry in entries:
                if entry.get("tier") not in ("s", "a"):
                    continue
                tid = int(entry["id"])
                for part in range(1, sys.maxsize):
                    query = url(f"{PANDA}/tournaments/{tid}/matches", sort="id", page=part, per_page=100)
                    rows = a.read("panda", query, f"panda-{tid}-{part}.json")
                    if len(rows) < 100:
                        break
            if len(entries) < 100:
                break


def collect_valve(a, until):
    # A dated commit pins both the catalog and each raw download across resumes.
    commits = a.read("valve", url(VALVE + "/commits", until=f"{until}T23:59:59Z", per_page=1),
                     f"catalog/valve-commit-{until}.json")
    sha = commits[0]["sha"]
    tree = a.read("valve", url(f"{VALVE}/git/trees/{sha}", recursive=1), f"catalog/valve-tree-{sha}.json")
    if tree.get("truncated"):
        raise ValueError("Valve returned an incomplete file tree")
    for entry in tree["tree"]:
        match = re.fullmatch(r"live/\d{4}/(standings_global_(\d{4}_\d{2}_\d{2})\.md)", entry["path"])
        if match and FIRST <= date.fromisoformat(match[2].replace("_", "-")) <= until:
            # Blob URL is immutable across later commit-tree refreshes.
            a.read("valve", entry["url"], match[1])


def known_titles():
    """Bootstrap verified Event aliases from the small benchmark's saved stage revisions."""
    from collect_backtest import ARCHIVE, load
    titles = {}
    for path in ARCHIVE.glob("lp-*.json.gz"):
        matches = path.with_name(path.name.replace("lp-", "panda-", 1))
        if matches.exists():
            page = load(path)["query"]["pages"][0]["title"]
            root, _, tail = page.rpartition("/")
            event = root if "Stage" in tail or tail == "Playoffs" else page
            titles[str(load(matches)[0]["serie_id"])] = event
    return titles


def collect_rosters(a, until, titles):
    events = {}
    missing = []
    for path in sorted(a.root.glob("panda-*.json.gz")):
        for row in json.loads(gzip.decompress(path.read_bytes())):
            serie = row["serie"]
            eid = str(row["serie_id"])
            if not serie.get("begin_at") or not serie.get("end_at"):
                continue
            events[eid] = (row["league"]["name"] + " " + serie["full_name"],
                           parse_time(serie["begin_at"]).date(), min(until, parse_time(serie["end_at"]).date()))
    for eid, (label, first, last) in events.items():
        title = titles.get(eid)
        if not title:
            query = url(LIQUIPEDIA, action="query", list="search", srsearch=label, srnamespace=0, format="json")
            hits = a.read("liquipedia", query, f"catalog/wiki-search-{eid}.json")["query"]["search"]
            words = lambda s: sorted(re.findall(r"[a-z0-9]+", s.lower()))
            exact = [h["title"] for h in hits if words(h["title"]) == words(label)]
            if len(exact) != 1:
                print(f"No verified Liquipedia title for Event {eid} ({label})", flush=True)
                missing.append(eid)
                continue
            title = exact[0]
        day = first
        while day <= last:
            before = datetime.combine(day, datetime.min.time(), UTC) - timedelta(seconds=1)
            query = url(LIQUIPEDIA, action="query", prop="revisions", rvprop="ids|timestamp|content",
                        rvslots="main", rvlimit=1, rvstart=iso(before), rvdir="older", formatversion=2,
                        format="json", titles=title)
            a.read("liquipedia", query, f"roster-{eid}-{day}.json")
            day += timedelta(days=1)
    return missing


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--archive", type=Path, default=ROOT / "target/research-archive")
    parser.add_argument("--until", type=date.fromisoformat, default=datetime.now(UTC).date() - timedelta(days=1))
    parser.add_argument("--env", type=Path, default=ROOT / ".env")
    parser.add_argument("--titles", type=Path, help="JSON object mapping PandaScore serie IDs to verified wiki titles")
    for source, cap in DEFAULT_CAPS.items():
        parser.add_argument("--" + source + "-cap", type=int, default=cap)
    args = parser.parse_args()
    if args.archive.resolve().is_relative_to((ROOT / "sidecar/tests/fixtures").resolve()):
        parser.error("research archives must be outside the test fixtures")
    if not FIRST <= args.until < datetime.now(UTC).date():
        parser.error("--until must be a completed UTC day since August 2024")
    import os
    from collect_backtest import SECRETS, load_secrets
    if args.env.exists():
        load_secrets(args.env)
    token = os.environ.get("PANDASCORE_TOKEN") or SECRETS.get("PANDASCORE_TOKEN")
    caps = {source: getattr(args, source + "_cap") for source in DEFAULT_CAPS}
    if caps["panda"] and not token:
        parser.error("PANDASCORE_TOKEN is required for PandaScore")
    last_lp = 0.0
    def fetch(source, request):
        nonlocal last_lp
        headers = {"User-Agent": USER_AGENT, "Accept-Encoding": "gzip"}
        if source == "panda":
            headers["Authorization"] = "Bearer " + token
        if source == "valve" and "/git/blobs/" in request:
            headers["Accept"] = "application/vnd.github.raw+json"
        if source == "liquipedia":
            time.sleep(max(0, last_lp + 2.1 - time.monotonic()))
            last_lp = time.monotonic()
        with urllib.request.urlopen(urllib.request.Request(request, headers=headers), timeout=45) as response:
            body = response.read()
            return gzip.decompress(body) if body.startswith(b"\x1f\x8b") else body
    args.archive.mkdir(parents=True, exist_ok=True)
    # An exclusive OS-released SQLite transaction prevents concurrent manifest writers.
    lock = sqlite3.connect(args.archive / "collector-lock.db", timeout=0)
    lock.execute("BEGIN EXCLUSIVE")
    archive = Archive(args.archive, caps, fetch, [token, *SECRETS.values()])
    titles = known_titles()
    if args.titles:
        titles.update(json.loads(args.titles.read_text(encoding="utf8")))
    partial = False
    for step in (lambda: collect_panda(archive, args.until), lambda: collect_valve(archive, args.until),
                 lambda: collect_rosters(archive, args.until, titles)):
        try:
            if step():
                partial = True
        except (Budget, urllib.error.URLError) as exc:
            print(exc, file=sys.stderr)
            partial = True
    for source, cap in caps.items():
        print(f"{source}: {archive.spent(source)}/{cap} requests today")
    archive.db.close()
    lock.close()
    return 2 if partial else 0


if __name__ == "__main__":
    sys.exit(main())
