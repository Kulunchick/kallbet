#!/usr/bin/env python3
"""Pass 1 of the Liquipedia-independent recovery: read every SAVED HLTV match page (no network) and write
  pages_meta.parquet   one row per page: teams, start, format, event, lineups (player ids per team)
  seen_matches.parquet one row per HLTV match id mentioned in a page's past-matches / head-to-head widgets
                       (slug gives the team pair, h2h rows give the exact start; past-matches only weeks-ago)
"""
import argparse, glob, gzip, json, os, re, sys
from multiprocessing import Pool
from lxml import html


def mid_slug(href):
    m = re.search(r"/matches/(\d+)/([^/?#]*)", href or "")
    return (int(m.group(1)), m.group(2)) if m else (None, None)


def parse(path):
    mid = int(re.search(r"hltv-match-(\d+)", path).group(1))
    d = html.fromstring(gzip.open(path).read())
    meta = dict(match_id=mid)
    for i, cls in ((1, "team1-gradient"), (2, "team2-gradient")):
        a = d.cssselect(f'.teamsBox .{cls} a[href^="/team/"]')
        if a:
            meta[f"team{i}_id"] = int(re.search(r"/team/(\d+)", a[0].get("href")).group(1))
            nm = a[0].cssselect(".teamName")
            meta[f"team{i}_name"] = nm[0].text_content().strip() if nm else None
    t = d.cssselect(".timeAndEvent .time[data-unix]")
    meta["start_unix_ms"] = int(t[0].get("data-unix")) if t else None
    ev = d.cssselect('.timeAndEvent .event a[href^="/events/"]')
    if ev:
        meta["event_id"] = int(re.search(r"/events/(\d+)", ev[0].get("href")).group(1))
        meta["event_name"] = ev[0].text_content().strip()
    fmt = None
    for pre in d.cssselect(".veto-box .preformatted-text"):
        low = pre.text_content().lower()
        for k, v in (("best of 5", "bo5"), ("best of 3", "bo3"), ("best of 1", "bo1")):
            if k in low:
                fmt = v
                break
        if fmt:
            meta["is_lan"] = "(lan)" in low
            break
    meta["format"] = fmt
    # lineups: one `.lineup` box per team (team link in its headline, players as /player/<id> links)
    lu = {}
    for box in d.cssselect(".lineups .lineup"):
        ta = box.cssselect('.box-headline a[href^="/team/"]')
        ids = []
        for x in box.cssselect('.players a[href^="/player/"]'):
            m = re.search(r"/player/(\d+)", x.get("href"))
            if m and int(m.group(1)) not in ids:
                ids.append(int(m.group(1)))
        if ta and ids:
            lu[int(re.search(r"/team/(\d+)", ta[0].get("href")).group(1))] = ids
    meta["lineups"] = json.dumps({str(k): v for k, v in lu.items()})
    stand = d.cssselect(".mapholder .mapname")
    meta["n_maps_listed"] = len([m for m in stand if m.text_content().strip().upper() != "TBA"])
    meta["has_stats"] = bool(d.cssselect(".matchstats"))
    dm = d.cssselect("[data-demo-link]")
    meta["demo_link"] = dm[0].get("data-demo-link") if dm else None
    seen = []
    for tr in d.cssselect(".past-matches tr"):
        a = tr.cssselect('a[href^="/matches/"]')
        if not a:
            continue
        i, slug = mid_slug(a[0].get("href"))
        tm = tr.cssselect('td.past-matches-team a[href^="/team/"]')
        ago = tr.cssselect(".past-matches-time-ago")
        fm = tr.cssselect("td.past-matches-map a")
        sc = tr.cssselect("td.past-matches-score a")
        if i:
            seen.append(dict(src=mid, kind="past", seen_id=i, slug=slug,
                             team_id=int(re.search(r"/team/(\d+)", tm[0].get("href")).group(1)) if tm else None,
                             team_name=tm[0].text_content().strip() if tm else None,
                             ago=ago[0].text_content().strip() if ago else None,
                             fmt=fm[0].text_content().strip() if fm else None,
                             score=sc[0].text_content().strip() if sc else None, unix_ms=None))
    for tr in d.cssselect(".head-to-head-listing tr"):
        a = tr.cssselect('td.date a[href^="/matches/"]')
        if not a:
            continue
        i, slug = mid_slug(a[0].get("href"))
        sp = a[0].cssselect("span[data-unix]")
        t1 = tr.cssselect('td.team1 a[href^="/team/"]')
        t2 = tr.cssselect('td.team2 a[href^="/team/"]')
        if i:
            seen.append(dict(src=mid, kind="h2h", seen_id=i, slug=slug,
                             team_id=int(re.search(r"/team/(\d+)", t1[0].get("href")).group(1)) if t1 else None,
                             team_name=(t1[0].text_content().strip() if t1 else None) + " | " + (t2[0].text_content().strip() if t2 else ""),
                             ago=None, fmt=None, score=None, unix_ms=int(sp[0].get("data-unix")) if sp else None))
    return meta, seen


def safe(path):
    try:
        return parse(path)
    except Exception as e:  # noqa
        return dict(match_id=int(re.search(r"hltv-match-(\d+)", path).group(1)), error=str(e)), []


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--archive", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--workers", type=int, default=3)
    ap.add_argument("--limit", type=int, default=0)
    ap.add_argument("--only", help="comma-separated match ids (default: every saved page)")
    a = ap.parse_args()
    import pandas as pd
    files = sorted(glob.glob(os.path.join(a.archive, "hltv", "matches", "hltv-match-*.html.gz")))
    if a.only:
        want = {int(x) for x in a.only.split(",")}
        files = [f for f in files if int(re.search(r"hltv-match-(\d+)", f).group(1)) in want]
    if a.limit:
        files = files[: a.limit]
    metas, seen = [], []
    with Pool(a.workers) as p:
        for i, (m, s) in enumerate(p.imap_unordered(safe, files, chunksize=20)):
            metas.append(m)
            seen += s
            if i % 1000 == 0:
                print(i, len(files), file=sys.stderr, flush=True)
    os.makedirs(a.out, exist_ok=True)
    pd.DataFrame(metas).to_parquet(os.path.join(a.out, "pages_meta.parquet"), index=False)
    pd.DataFrame(seen).to_parquet(os.path.join(a.out, "seen_matches.parquet"), index=False)
    print("pages", len(metas), "seen rows", len(seen))


if __name__ == "__main__":
    main()
