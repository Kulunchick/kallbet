#!/usr/bin/env python3
"""Extract every per-player stat column from the SAVED HLTV match pages (no network).

Output (long format, one row per player x table x scope), written under --out:
  player_stats_all_columns.parquet  every column seen, raw text values (audit input)
  page_headers.parquet              one row per page x table: header signature, rating version label

Used by hltv_rating_audit.py. Needs lxml, cssselect, pandas, pyarrow (see docs in the run report).
"""
import argparse, csv, glob, gzip, os, re, sys
from multiprocessing import Pool
from lxml import html

IGNORE = {"text-center", "hidden", "gtSmartphone-only", "smartphone-only", "traditional-data", "eco-adjusted-data"}


def cell_key(td):
    cls = (td.get("class") or "").split()
    base = [c for c in cls if c not in IGNORE]
    kind = "eco" if "eco-adjusted-data" in cls else ("trad" if "traditional-data" in cls else "")
    return ".".join(base) + (":" + kind if kind else "")


def parse_page(path):
    mid = int(re.search(r"hltv-match-(\d+)", path).group(1))
    out_rows, out_hdr = [], []
    try:
        raw = gzip.open(path).read()
        d = html.fromstring(raw)
    except Exception as e:  # noqa
        return mid, None, [], [], f"unreadable: {e}"
    unix = d.cssselect(".timeAndEvent .time[data-unix]")
    started = int(unix[0].get("data-unix")) if unix else None
    maps = {}
    for a in d.cssselect(".mapholder a.results-stats"):
        m = re.search(r"mapstatsid/(\d+)", a.get("href") or "")
        nm = a.getparent().cssselect(".mapname")
        if m and nm:
            maps[m.group(1)] = nm[0].text_content().strip()
    for content in d.cssselect('div.stats-content[id$="-content"]'):
        scope = content.get("id")[: -len("-content")]
        for table in content.cssselect("table"):
            tcls = (table.get("class") or "").split()
            side = "all" if "totalstats" in tcls else "ct" if "ctstats" in tcls else "t" if "tstats" in tcls else None
            if side is None:
                continue
            hdr = table.cssselect("tr.header-row")
            if not hdr:
                continue
            hr = hdr[0]
            tds = hr.cssselect("td")
            keys = [cell_key(td) for td in tds]
            team_a = hr.cssselect("a.teamName")
            team_id = None
            if team_a:
                m = re.search(r"/team/(\d+)", team_a[0].get("href") or "")
                team_id = int(m.group(1)) if m else None
            rd = hr.cssselect(".ratingDesc")
            rating_label = rd[0].text_content().strip() if rd else ""
            # header label text per column, e.g. "Rating" "Swing"
            labels = [re.sub(r"\s+", " ", td.text_content()).strip() for td in tds]
            out_hdr.append(dict(match_id=mid, scope=scope, side=side, team_id=team_id,
                                header_sig="|".join(f"{k}={l}" for k, l in zip(keys, labels)),
                                rating_label=rating_label))
            for tr in table.cssselect("tr"):
                if "header-row" in (tr.get("class") or ""):
                    continue
                pl = tr.cssselect('td.players a[href^="/player/"]')
                if not pl:
                    continue
                pm = re.search(r"/player/(\d+)", pl[0].get("href"))
                nick = tr.cssselect(".player-nick")
                row = dict(match_id=mid, scope=scope, map_name=maps.get(scope), side=side, team_id=team_id,
                           player_id=int(pm.group(1)), nickname=nick[0].text_content().strip() if nick else None,
                           rating_label=rating_label)
                for k, td in zip(keys[1:], tr.cssselect("td")[1:]):
                    row[k] = re.sub(r"\s+", " ", td.text_content()).strip()
                out_rows.append(row)
    return mid, started, out_rows, out_hdr, None


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--archive", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--workers", type=int, default=3)
    ap.add_argument("--limit", type=int, default=0)
    ap.add_argument("--append", action="store_true", help="only parse pages not yet in --out/pages.parquet and append")
    a = ap.parse_args()
    import pandas as pd
    hdir = os.path.join(a.archive, "hltv")
    files = sorted(glob.glob(os.path.join(hdir, "matches", "hltv-match-*.html.gz")))
    old = None
    if a.append and os.path.exists(os.path.join(a.out, "pages.parquet")):
        old = [pd.read_parquet(os.path.join(a.out, n + ".parquet")) for n in ("player_stats_all_columns", "page_headers", "pages")]
        done = set(old[2].match_id)
        files = [f for f in files if int(re.search(r"hltv-match-(\d+)", f).group(1)) not in done]
    if a.limit:
        files = files[: a.limit]
    fetched = {}
    with open(os.path.join(hdir, "sources.tsv"), encoding="utf8") as f:
        for r in csv.DictReader(f, delimiter="\t"):
            m = re.search(r"hltv-match-(\d+)", r["file"])
            if m:
                fetched[int(m.group(1))] = r["fetched_at"]
    rows, hdrs, pages, errs = [], [], [], []
    with Pool(a.workers) as p:
        for i, (mid, started, r, h, err) in enumerate(p.imap_unordered(parse_page, files, chunksize=20)):
            pages.append(dict(match_id=mid, started_unix_ms=started, fetched_at=fetched.get(mid), n_rows=len(r), error=err))
            rows += r
            hdrs += h
            if i % 1000 == 0:
                print(i, len(files), file=sys.stderr, flush=True)
    os.makedirs(a.out, exist_ok=True)
    new = [pd.DataFrame(rows), pd.DataFrame(hdrs), pd.DataFrame(pages)]
    if old:
        new = [pd.concat([o, n], ignore_index=True) for o, n in zip(old, new)]
    for name, df in zip(("player_stats_all_columns", "page_headers", "pages"), new):
        df.to_parquet(os.path.join(a.out, name + ".parquet"), index=False)
    print("pages", len(pages), "rows", len(rows), "tables", len(hdrs))


if __name__ == "__main__":
    main()
