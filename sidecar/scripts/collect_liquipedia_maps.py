"""Collect post-Match Liquipedia Event + Stage pages into `<archive>/hltv/liquipedia/`
for HLTV match ID discovery (`|hltv=<id>`) and point-in-time per-map results (#44).

Strictly isolated from `collect_research.py` (never touches `collector.db` or root
`sources.tsv`). Follows Liquipedia API Terms of Use (<= 1 req / 2 s, custom User-Agent,
gzip, never calls `action=parse`, resumable cache).
"""

import argparse
import csv
import gzip
import hashlib
import json
import pathlib
import re
import time
import urllib.parse
import urllib.request
from datetime import datetime, timezone

LIQUIPEDIA = "https://liquipedia.net/counterstrike/api.php"
USER_AGENT = "KallBet/0.1 (https://kallbet.kulunchick.tech)"
SKIP_SUBPAGES = (
    "qualif",
    "invite",
    "showmatch",
    "statistic",
    "prize",
    "participants",
    "europe",
    "asia",
    "north america",
    "south america",
    "americas",
    "china",
    "oceania",
    "closed",
    "open",
)

# Curated verified Liquipedia root titles for PandaScore series whose naming
# differs from Liquipedia's path convention (e.g. "IEM" -> "Intel Extreme Masters").
KNOWN_SERIES_TITLES = {
    "7754": "BLAST/Premier/2024/Fall/Groups",
    "7790": "Intel Extreme Masters/2024/Cologne",
    "7856": "BLAST/Premier/2024/Fall/Showdown",
    "7871": "BetBoom Dacha/2024/Belgrade/2",
    "7904": "ESL/Pro League/Season 20",
    "7940": "BLAST/Premier/2024/Fall",
    "7986": "Intel Extreme Masters/2024/Rio",
    "8062": "Thunderpick/World Championship/2024",
    "8063": "Perfect World/Major/2024/Shanghai/Europe A",
    "8064": "Perfect World/Major/2024/Shanghai/Europe B",
    "8065": "Perfect World/Major/2024/Shanghai/Americas",
    "8071": "BLAST/Premier/2024/World Final",
    "8751": "Perfect World/Major/2024/Shanghai",
    "8888": "BLAST/Bounty/2025/Spring/Closed Qualifier",
    "8942": "Intel Extreme Masters/2025/Katowice",
    "8943": "BLAST/Bounty/2025/Spring",
    "9006": "PGL/2025/Cluj-Napoca",
    "9063": "ESL/Pro League/Season 21",
    "9081": "BLAST/Open/2025/Spring",
    "9160": "PGL/2025/Bucharest",
    "9230": "Intel Extreme Masters/2025/Melbourne",
    "9257": "BLAST/Major/2025/Austin/Europe",
    "9265": "BLAST/Rivals/2025/Spring",
    "9280": "PGL/2025/Astana",
    "9301": "Intel Extreme Masters/2025/Dallas",
    "9309": "BLAST/Major/2025/Austin",
    "9453": "FISSURE/Playground/1",
    "9495": "Intel Extreme Masters/2025/Cologne",
    "9554": "BLAST/Bounty/2025/Fall",
    "9570": "Esports World Cup/2025",
    "9596": "BLAST/Open/2025/Fall",
    "9662": "FISSURE/Playground/2",
    "9667": "ESL/Pro League/Season 22",
    "9746": "CS Asia Championships/2025",
    "9778": "Thunderpick/World Championship/2025",
    "9801": "PGL/2025/Masters Bucharest",
    "9806": "Intel Extreme Masters/2025/Chengdu",
    "9822": "StarLadder/2025/Major",
    "9826": "BLAST/Rivals/2025/Fall",
    "10034": "BLAST/Bounty/2026/Winter",
    "10095": "Intel Extreme Masters/2026/Krakow",
    "10192": "PGL/2026/Cluj-Napoca",
    "10256": "BLAST/Open/2026/Spring",
    "10268": "ESL/Pro League/Season 23",
    "10450": "PGL/2026/Bucharest",
    "10474": "Intel Extreme Masters/2026/Rio",
    "10488": "Intel Extreme Masters/2026/Cologne",
    "10506": "BLAST/Rivals/2026/Spring",
    "10574": "PGL/2026/Astana",
    "10583": "Intel Extreme Masters/2026/Atlanta",
    "10593": "CS Asia Championships/2026",
    "10685": "XSE Pro League/2026",
    "10801": "BLAST/Bounty/2026/Summer",
    "10846": "Esports World Cup/2026",
    "10874": "BLAST/Open/2026/Fall",
    "10895": "FISSURE/Playground/3",
    "10954": "StarLadder/StarSeries/2026/Fall",
    "11004": "ESL/Pro League/Season 24",
    "11069": "Thunderpick/World Championship/2026",
    "11118": "PGL/2026/Masters Bucharest",
    "11136": "Intel Extreme Masters/2026/Beijing",
    "11137": "BLAST/Rivals/2026/Fall",
    "11176": "PGL/2026/Singapore",
}


class Limiter:
    def __init__(self, spacing=2.2):
        self.spacing = spacing
        self.last = 0.0
        self.calls = 0

    def fetch(self, params):
        wait = self.last + self.spacing - time.monotonic()
        if wait > 0:
            time.sleep(wait)
        url = LIQUIPEDIA + "?" + urllib.parse.urlencode(params)
        req = urllib.request.Request(
            url, headers={"User-Agent": USER_AGENT, "Accept-Encoding": "gzip"}
        )
        self.last = time.monotonic()
        self.calls += 1
        with urllib.request.urlopen(req, timeout=45) as resp:
            raw = resp.read()
            if raw[:2] == b"\x1f\x8b":
                raw = gzip.decompress(raw)
        return raw, url


def iso_now():
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def record_provenance(tsv_path, rel_path, url, raw_bytes):
    digest = hashlib.sha256(raw_bytes).hexdigest()
    header = ("file", "request", "fetched_at", "sha256")
    rows = {}
    if tsv_path.exists():
        with tsv_path.open("r", encoding="utf8", newline="") as f:
            for r in csv.reader(f, delimiter="\t"):
                if r and r[0] != "file":
                    rows[r[0]] = r
    rows[rel_path] = [rel_path, url, iso_now(), digest]
    tmp = tsv_path.with_suffix(".tsv.tmp")
    with tmp.open("w", encoding="utf8", newline="") as f:
        w = csv.writer(f, delimiter="\t", lineterminator="\n")
        w.writerow(header)
        for k in sorted(rows):
            w.writerow(rows[k])
    tmp.replace(tsv_path)


def cached_query(out_dir, tsv_path, rel_name, params, limiter):
    path = out_dir / rel_name
    if path.exists():
        return json.loads(gzip.decompress(path.read_bytes())), False
    raw, url = limiter.fetch(params)
    data = json.loads(raw)
    if "error" in data:
        raise RuntimeError(f"Liquipedia API error: {data['error']}")
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp = path.with_suffix(".gz.tmp")
    tmp.write_bytes(gzip.compress(raw, mtime=0))
    tmp.replace(path)
    record_provenance(tsv_path, f"liquipedia/{rel_name}", url, raw)
    return data, True


def stage_subpages(titles, root):
    out = []
    prefix = root + "/"
    for t in titles:
        if not t.startswith(prefix):
            continue
        rest = t[len(prefix) :]
        if "/" in rest:
            continue
        low = rest.lower()
        if any(s in low for s in SKIP_SUBPAGES):
            continue
        out.append(t)
    return sorted(out)


def words(text):
    return sorted(re.findall(r"[a-z0-9]+", text.lower()))


def pick_title(label, hits):
    """The one search hit whose words are exactly the Event label's words, else None."""
    exact = [h["title"] for h in hits if words(h["title"]) == words(label)]
    return exact[0] if len(exact) == 1 else None


def resolve_titles(series, known, lp_dir, tsv_path, limiter, max_calls):
    """Verified Liquipedia titles for series without a curated one: one search request each
    (cached under `search-<serie>.json.gz`), accepted only on an exact word match."""
    found = {}
    for sid, label in sorted(series.items(), key=lambda kv: int(kv[0])):
        if sid in known:
            continue
        if not (lp_dir / f"search-{sid}.json.gz").exists() and limiter.calls >= max_calls:
            break
        data, _ = cached_query(
            lp_dir, tsv_path, f"search-{sid}.json.gz",
            {"action": "query", "list": "search", "srsearch": label, "srnamespace": 0,
             "srlimit": 10, "format": "json"},
            limiter,
        )
        title = pick_title(label, data.get("query", {}).get("search", []))
        if title:
            found[sid] = title
    return found


def discover_series(archive_dir):
    series = {}
    for p in sorted(archive_dir.glob("panda-*.json.gz")):
        rows = json.loads(gzip.decompress(p.read_bytes()))
        for r in rows:
            sid = str(r["serie_id"])
            if sid not in series:
                series[sid] = r["league"]["name"] + " " + r["serie"]["full_name"]
    # Also read titles from any roster-<sid>-*.json.gz already in archive
    titles = dict(KNOWN_SERIES_TITLES)
    for p in sorted(archive_dir.glob("roster-*.json.gz")):
        parts = p.name.split("-")
        if len(parts) >= 2:
            sid = parts[1]
            try:
                d = json.loads(gzip.decompress(p.read_bytes()))
                titles.setdefault(sid, d["query"]["pages"][0]["title"])
            except Exception:
                pass
    return series, titles


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--archive", type=pathlib.Path, required=True)
    ap.add_argument("--out", type=pathlib.Path)
    ap.add_argument("--max-calls", type=int, default=120)
    ap.add_argument("--resolve", action="store_true",
                    help="search Liquipedia for the title of series without a curated one (exact word match only)")
    ap.add_argument("--titles", type=pathlib.Path,
                    help="JSON object mapping PandaScore serie IDs to verified Liquipedia titles")
    args = ap.parse_args()

    out_root = args.out or (args.archive / "hltv")
    lp_dir = out_root / "liquipedia"
    lp_dir.mkdir(parents=True, exist_ok=True)
    tsv_path = out_root / "sources-liquipedia.tsv"

    series, titles = discover_series(args.archive)
    limiter = Limiter(2.2)
    if args.titles:
        titles.update(json.loads(args.titles.read_text(encoding="utf8")))
    if args.resolve:
        titles.update(resolve_titles(series, titles, lp_dir, tsv_path, limiter, args.max_calls))
    total_hltv_ids = set()
    resolved = 0
    unresolved = []

    for sid, label in sorted(series.items(), key=lambda kv: int(kv[0])):
        title = titles.get(sid)
        if not title:
            unresolved.append((sid, label))
            continue
        if limiter.calls >= args.max_calls:
            break
        # 1. Discover Stage subpages under `<title>/`
        ap_data, _ = cached_query(
            lp_dir,
            tsv_path,
            f"allpages-{sid}.json.gz",
            {
                "action": "query",
                "list": "allpages",
                "apprefix": title + "/",
                "apnamespace": 0,
                "aplimit": 50,
                "formatversion": 2,
                "format": "json",
            },
            limiter,
        )
        listed = [p["title"] for p in ap_data.get("query", {}).get("allpages", [])]
        pages = [title] + stage_subpages(listed, title)
        if limiter.calls >= args.max_calls:
            break
        # 2. Fetch latest post-Match revisions of root + all Stage subpages in one batched call
        # Note: MediaWiki returns the latest revision per page by default and rejects `rvlimit`
        # when multiple `titles` are supplied.
        rev_data, fetched = cached_query(
            lp_dir,
            tsv_path,
            f"event-{sid}.json.gz",
            {
                "action": "query",
                "prop": "revisions",
                "rvprop": "ids|timestamp|content",
                "rvslots": "main",
                "formatversion": 2,
                "format": "json",
                "titles": "|".join(pages),
            },
            limiter,
        )
        ids = set()
        for pg in rev_data.get("query", {}).get("pages", []):
            pg_title = pg.get("title", "")
            # Filter regional qualifier subpages from cached multi-page responses too,
            # while keeping RMR pages whose root title itself contains a region.
            if pg_title != title and any(s in pg_title.lower() for s in SKIP_SUBPAGES):
                continue
            revs = pg.get("revisions") or []
            if not revs:
                continue
            txt = revs[0].get("slots", {}).get("main", {}).get("content", "")
            for m in re.findall(r"\|hltv=(\d+)", txt):
                ids.add(int(m))
        total_hltv_ids.update(ids)
        resolved += 1
        tag = "fetched" if fetched else "cached"
        print(
            f"serie {sid:>5} | {tag:7} | pages={len(pages):2d} | hltv_ids={len(ids):3d} | {title}",
            flush=True,
        )

    # Write deterministic targets.tsv ordered newest-first (descending HLTV match ID)
    # so 2026 matches with Pinnacle market lines are collected first within daily budgets.
    targets_path = out_root / "targets.tsv"
    tmp_targets = targets_path.with_suffix(".tsv.tmp")
    with tmp_targets.open("w", encoding="utf8", newline="") as f:
        f.write("match_id\turl\n")
        for mid in sorted(total_hltv_ids, reverse=True):
            f.write(f"{mid}\thttps://www.hltv.org/matches/{mid}/match\n")
    tmp_targets.replace(targets_path)

    print(
        f"\nDONE: resolved={resolved}/{len(series)} unresolved={len(unresolved)} "
        f"liquipedia_calls={limiter.calls} unique_hltv_ids={len(total_hltv_ids)} "
        f"targets={targets_path}",
        flush=True,
    )
    if unresolved:
        print("Unresolved series (add to KNOWN_SERIES_TITLES):", unresolved[:20], flush=True)


if __name__ == "__main__":
    main()
