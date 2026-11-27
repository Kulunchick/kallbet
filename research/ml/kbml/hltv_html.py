"""Parses the raw HLTV match pages (`hltv/matches/hltv-match-<id>.html.gz`) for what the JSON lacks.

The output of one page is split by when it is known (audit: ml-notes/hltv-features.md):

- `pre`: known before the Match starts, so usable for the Match itself: the HLTV world rank badge
  (with the ranking date it links to), the two lineups with stand-in markers and role pills, the
  "past 3 months" match lists (team and core) with the streak badges, the head-to-head summary and
  map listing, and the structured lines of the match note (format, stage, coach substitutes).
- `post`: produced by the Match, so usable only for later Matches (released by the timeline once
  the Match's result is known): the CT/T side of every half score, per-player CT-only and T-only
  stats.

Never read: the VRS badge (it already holds the Match's result), any odds widget, `time-ago` texts
(relative to the fetch), map pick badges and the veto of the Match itself (post-veto), POTM, VODs,
comments, and free-form `**` admin notes other than coach substitutions.
"""
from __future__ import annotations

import gzip
import re
from datetime import datetime
from pathlib import Path

from lxml import html

MONTHS = {m: i for i, m in enumerate(
    ["january", "february", "march", "april", "may", "june", "july", "august", "september", "october",
     "november", "december"], start=1)}
_RANK_HREF = re.compile(r"/ranking/teams/(\d{4})/([a-z]+)/(\d{1,2})/(\d+)")
_ID = re.compile(r"/(?:team|player|matches)/(\d+)")
_COACH = re.compile(r"(\S+)\s+\((?:assistant\s+|head\s+)?coach\)\s+(?:substitutes|stands in for|replaces)\s+(\S+?)\.?$",
                    re.I)


def _id(href: str | None) -> int | None:
    m = _ID.search(href or "")
    return int(m.group(1)) if m else None


def _text(el) -> str:
    return " ".join(el.text_content().split())


def _num(s: str):
    s = s.strip().replace("%", "").replace("+", "")
    try:
        return float(s)
    except ValueError:
        return None


def parse_page(text: str) -> dict:
    doc = html.fromstring(text)
    return {"pre": _pre(doc), "post": _post(doc)}


def parse_file(path: Path) -> dict:
    return parse_page(gzip.decompress(Path(path).read_bytes()).decode("utf-8"))


# ------------------------------------------------------------------ pre-match

def _pre(doc) -> dict:
    out: dict = {"teams": [], "ranks": [], "lineups": [], "past": {}, "streak": {}, "h2h": None, "notes": None}
    for box in doc.cssselect(".lineups .lineup")[:2] or doc.cssselect(".lineups > div")[:2]:
        head = box.cssselect(".box-headline a")
        tid = _id(head[0].get("href")) if head else None
        out["teams"].append({"id": tid, "name": _text(head[0]) if head else None})
        badge = box.cssselect(".teamRanking a.hltv-ranking")
        rank = None
        if badge:
            m = _RANK_HREF.search(badge[0].get("href") or "")
            r = re.search(r"#(\d+)", _text(badge[0]))
            if m and r and m.group(2) in MONTHS:
                rank = {"rank": int(r.group(1)),
                        "date": f"{int(m.group(1)):04d}-{MONTHS[m.group(2)]:02d}-{int(m.group(3)):02d}"}
        out["ranks"].append(rank)
        players = []
        for td in box.cssselect(".players td.player-image, .players td.player.player-image"):
            a = td.cssselect("a[href^='/player/']")
            if not a:
                continue
            standin = td.cssselect(".lineup-standin")
            players.append({
                "id": _id(a[0].get("href")),
                "nick": (a[0].get("href") or "").rstrip("/").split("/")[-1],
                "standin": bool(standin),
                "replaced": (standin[0].get("title") or "").replace("Substituting for", "").strip() if standin else None,
                "igl": bool(td.cssselect(".role-pill--igl")),
                "awp": bool(td.cssselect(".role-pill--awp")),
            })
        out["lineups"].append(players)
    for kind in ("team", "core"):
        grid = doc.cssselect(f".past-matches-grid[data-past-matches-{kind}]")
        sides, streaks = [], []
        for b in (grid[0].cssselect(".past-matches-box")[:2] if grid else []):
            rows = []
            for tr in b.cssselect("table.past-matches-table tr"):
                opp = tr.cssselect(".past-matches-teamname a")
                score = tr.cssselect("td.past-matches-score a")
                fmt = tr.cssselect("td.past-matches-map a")
                if not opp or not score:
                    continue
                cls = score[0].get("class") or ""
                s = re.findall(r"\d+", _text(score[0]))
                rows.append({"mid": _id(score[0].get("href")), "opp_id": _id(opp[0].get("href")),
                             "opp": _text(opp[0]), "won": "won" in cls.split(),
                             "for": int(s[0]) if len(s) == 2 else None, "against": int(s[1]) if len(s) == 2 else None,
                             "token": _text(fmt[0]) if fmt else None})
            sides.append(rows)
            st = b.cssselect(".past-matches-streak")
            n = re.search(r"(\d+)", _text(st[0])) if st else None
            streaks.append(int(n.group(1)) if n else 0)
        out["past"][kind] = sides
        out["streak"][kind] = streaks
    h2h = doc.cssselect(".head-to-head")
    if h2h:
        nums = [int(x) for x in re.findall(r"\d+", " ".join(_text(b) for b in h2h[0].cssselect(".bold")))]
        rows = []
        for tr in doc.cssselect(".head-to-head-listing tr.row"):
            t = tr.cssselect("td.date [data-unix]")
            t1, t2 = tr.cssselect("td.team1"), tr.cssselect("td.team2")
            res = re.findall(r"\d+", _text(tr.cssselect("td.result")[0])) if tr.cssselect("td.result") else []
            mp = tr.cssselect("td.map .dynamic-map-name-full")
            if not t or not t1 or not t2 or len(res) != 2:
                continue
            rows.append({
                "unix_ms": int(t[0].get("data-unix")),
                "mid": _id((tr.cssselect("td.date a") or [None])[0].get("href") if tr.cssselect("td.date a") else None),
                "team1": _id((t1[0].cssselect("a") or [None])[0].get("href")) if t1[0].cssselect("a") else None,
                "team2": _id((t2[0].cssselect("a") or [None])[0].get("href")) if t2[0].cssselect("a") else None,
                "players1": [p.strip() for p in (t1[0].get("title") or "").split(",") if p.strip()],
                "players2": [p.strip() for p in (t2[0].get("title") or "").split(",") if p.strip()],
                "winner1": "winner" in (t1[0].get("class") or "").split(),
                "winner2": "winner" in (t2[0].get("class") or "").split(),
                "map": _text(mp[0]) if mp else None,
                "s1": int(res[0]), "s2": int(res[1]),
            })
        out["h2h"] = {"summary": nums[:3] if len(nums) >= 3 else None, "rows": rows}
    note = doc.cssselect(".veto-box .preformatted-text")
    if note:
        lines = [ln.strip() for ln in note[0].text_content().splitlines() if ln.strip()]
        stage = [ln[1:].strip() for ln in lines if ln.startswith("*") and not ln.startswith("**")]
        coach = []
        for ln in lines:
            if ln.startswith("**"):
                m = _COACH.search(ln.lstrip("* ").strip())
                if m:
                    coach.append({"coach": m.group(1), "for": m.group(2)})
        out["notes"] = {"format": lines[0] if lines and not lines[0].startswith("*") else None,
                        "stage": stage, "coach_subs": coach}
    return out


# ------------------------------------------------------------------ post-match

def _post(doc) -> dict:
    halves = []
    for mh in doc.cssselect(".mapholder"):
        name = mh.cssselect(".mapname")
        hs = mh.cssselect(".results-center-half-score")
        if not name or not hs:
            continue
        spans = [(s.get("class") or "", (s.text or "").strip()) for s in hs[0].cssselect("span")]
        sided = [(c, int(t)) for c, t in spans if c in ("ct", "t") and t.isdigit()]
        h = {"map": _text(name[0]), "a_ct": 0, "a_t": 0, "b_ct": 0, "b_t": 0, "a_ct_n": 0, "a_t_n": 0}
        for i in range(0, len(sided) - 1, 2):
            (ca, ra), (cb, rb) = sided[i], sided[i + 1]
            if ca == cb:
                continue
            h["a_" + ca] += ra
            h["b_" + cb] += rb
            h["a_" + ca + "_n"] += ra + rb  # rounds team A played on that side
        if len(sided) >= 4:
            halves.append(h)
    sides = {}
    content = doc.cssselect("#all-content")
    if content:
        for kind in ("ctstats", "tstats"):
            for table in content[0].cssselect(f"table.{kind}"):
                for tr in table.cssselect("tr")[1:]:
                    a = tr.cssselect("td.players a[href^='/player/']")
                    if not a:
                        continue
                    pid = _id(a[0].get("href"))
                    cells = {
                        "rating": tr.cssselect("td.rating"),
                        "swing": tr.cssselect("td.roundSwing"),
                        "eadr": tr.cssselect("td.adr.eco-adjusted-data"),
                    }
                    sides.setdefault(pid, {})[kind[:-5]] = {
                        k: (_num(_text(v[0])) if v else None) for k, v in cells.items()}
    return {"halves": halves, "sides": sides}


# ------------------------------------------------------------------ weekly world ranking

def parse_ranking(text: str) -> dict:
    """An HLTV world ranking page (`hltv/ranking/hltv-ranking-<date>.html.gz`), as the Rust
    `hltv_research::ranking::parse_ranking`: the date from the canonical link, and per ranked team
    rank, HLTV team id, name and points."""
    doc = html.fromstring(text)
    canon = doc.cssselect("link[rel=canonical]")
    href = canon[0].get("href") if canon else ""
    parts = href.split("/ranking/teams/")[1].split("/") if "/ranking/teams/" in href else []
    if len(parts) < 3 or parts[1] not in MONTHS:
        raise ValueError("canonical is not a dated ranking page")
    date = f"{int(parts[0]):04d}-{MONTHS[parts[1]]:02d}-{int(parts[2]):02d}"
    teams = []
    for t in doc.cssselect(".ranked-team"):
        pos = t.cssselect(".ranking-header .position")
        name = t.cssselect(".teamLine .name")
        pts = t.cssselect(".teamLine .points")
        link = [a for a in t.cssselect(".more a.moreLink") if (a.get("href") or "").startswith("/team/")]
        if not (pos and name and pts and link):
            raise ValueError("incomplete ranked team")
        teams.append({"rank": int(_text(pos[0]).lstrip("#")), "team_id": _id(link[0].get("href")),
                      "name": _text(name[0]), "points": int(_text(pts[0]).lstrip("(").split()[0].replace(",", ""))})
    if not teams:
        raise ValueError("no ranked teams")
    return {"date": date, "teams": teams}
