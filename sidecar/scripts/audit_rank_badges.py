"""Leakage audit of the HLTV match page rank badges (ticket #57).

Offline and read-only: `python -I sidecar/scripts/audit_rank_badges.py <research archive>`.
Reads `<archive>/hltv/matches/hltv-match-<id>.{json,html.gz}` and the archive's
dated Valve standings `standings_global_YYYY_MM_DD.md.gz`. Results are written
up in docs/signal-lab/criteria-research.md ("Rank badge audit").
"""
import collections
import datetime as dt
import gzip
import json
import os
import re
import sys

MONTHS = {m: i + 1 for i, m in enumerate(
    "january february march april may june july august september october november december".split())}
BADGE = re.compile(
    r'href="/(?:ranking/teams|valve-ranking/teams)/(\d{4})/([a-z]+)/(\d+)[^"]*" '
    r'class="ranking-badge (hltv|vrs)-ranking"><span>[A-Z]+: </span>#(\d+)</a>')


def key(name):
    return re.sub(r"[^a-z0-9]", "", name.lower())


def snapshots(archive):
    out = {}
    for f in os.listdir(archive):
        m = re.match(r"standings_global_(\d{4})_(\d\d)_(\d\d)\.md\.gz$", f)
        if not m:
            continue
        rows = {}
        for line in gzip.open(os.path.join(archive, f), "rt", encoding="utf8"):
            c = [x.strip() for x in line.split("|")]
            if len(c) > 4 and c[1].isdigit():
                rows.setdefault(key(c[3]), int(c[1]))
        out[dt.date(*map(int, m.groups()))] = rows
    return out


def pages(archive):
    folder = os.path.join(archive, "hltv", "matches")
    for f in sorted(os.listdir(folder)):
        if not f.endswith(".json"):
            continue
        rec = json.load(open(os.path.join(folder, f), encoding="utf8"))
        raw = os.path.join(folder, f.replace(".json", ".html.gz"))
        if not os.path.exists(raw):
            continue
        html = gzip.open(raw, "rt", encoding="utf8").read()
        badges = {"hltv": [], "vrs": []}
        for y, mon, d, kind, rank in BADGE.findall(html):
            badges[kind].append((dt.date(int(y), MONTHS[mon], int(d)), int(rank)))
        wa = sum(m.get("winner") == "a" for m in rec["maps"] if m.get("played"))
        wb = sum(m.get("winner") == "b" for m in rec["maps"] if m.get("played"))
        yield rec, {k: v[:2] for k, v in badges.items()}, (None if wa == wb else (0 if wa > wb else 1))


def pick(ranks):
    return None if ranks[0] == ranks[1] else ranks.index(min(ranks))


def main(archive):
    snaps = snapshots(archive)
    dates = sorted(snaps)
    lag = {"hltv": collections.Counter(), "vrs": collections.Counter()}
    fetch_dated = 0
    exact = collections.Counter()
    acc = collections.Counter()
    by_team = {"hltv": collections.defaultdict(list), "vrs": collections.defaultdict(list)}
    n_pages = 0
    for rec, badges, winner in pages(archive):
        n_pages += 1
        start = dt.datetime.fromisoformat(rec["started_at"].replace("Z", "+00:00"))
        names = [rec["team_a"]["name"], rec["team_b"]["name"]]
        ids = [rec["team_a"]["id"], rec["team_b"]["id"]]
        for kind, rows in badges.items():
            for d, _ in rows:
                lag[kind][(start.date() - d).days] += 1
                fetch_dated += str(d) == rec["fetched_at"][:10]
        vrs = badges["vrs"]
        if len(vrs) == 2:
            # (1) the badge against Valve's snapshot of the same date, where the archive has one
            for (d, r), n in zip(vrs, names):
                v = snaps.get(d, {}).get(key(n))
                if v is not None:
                    exact["n"] += 1
                    exact["equal"] += v == r
                    exact["within_2"] += abs(v - r) <= 2
        if winner is None:
            continue
        for kind, rows in badges.items():
            if len(rows) == 2:
                for i in (0, 1):
                    by_team[kind][ids[i]].append((start, rows[i][1], i == winner))
        # (2) picks: VRS badge, HLTV badge, latest Valve snapshot strictly before the Match day
        prior = [s for s in dates if s < start.date()]
        snap = [snaps[prior[-1]].get(key(n)) for n in names] if prior else [None, None]
        if len(vrs) == 2 and None not in snap and pick([r for _, r in vrs]) is not None \
                and pick(snap) is not None:
            acc["n"] += 1
            acc["vrs_badge"] += pick([r for _, r in vrs]) == winner
            acc["vrs_snapshot"] += pick(snap) == winner
            if len(badges["hltv"]) == 2 and pick([r for _, r in badges["hltv"]]) is not None:
                acc["n_hltv"] += 1
                acc["hltv_badge"] += pick([r for _, r in badges["hltv"]]) == winner
    print("pages with a parsed record and raw HTML:", n_pages)
    print("badge date lag (Match day - badge date, days):")
    for kind in ("vrs", "hltv"):
        print("  ", kind, dict(sorted(lag[kind].items())))
    print("badges dated by the page's fetch day:", fetch_dated)
    print("VRS badge vs Valve snapshot of the same date:", dict(exact))
    print("picks of the better rank:", dict(acc))
    # (3) a team's next page: does a changed rank follow the earlier Match's result (already played
    # when the later badge was drawn) or the later page's own Match (only if the badge leaks)?
    for kind in ("vrs", "hltv"):
        for hi in (1, 14):
            k = collections.Counter()
            for rows in by_team[kind].values():
                rows.sort()
                for (s1, r1, w1), (s2, r2, w2) in zip(rows, rows[1:]):
                    gap = (s2.date() - s1.date()).days
                    if not (1 <= gap <= hi) or r1 == r2:
                        continue
                    k["changed"] += 1
                    k["follows_earlier_result"] += (r2 < r1) == w1
                    k["follows_own_result"] += (r2 < r1) == w2
            print("%s next page 1-%d days later:" % (kind, hi), dict(k))


if __name__ == "__main__":
    main(sys.argv[1])
