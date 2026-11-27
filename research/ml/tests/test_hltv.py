"""Point-in-time checks of every parsed HLTV field (audit: ml-notes/hltv-features.md, section 2).

The synthetic tests always run; the archive tests run when the research snapshot and the cache
are present (they build the cache if needed)."""
import copy
import json
import pickle
from pathlib import Path

import numpy as np
import pandas as pd
import pytest

from kbml import features as F
from kbml import hltv_data, paths
from kbml import timeutil as T
from kbml.hltv_html import parse_page

PAGE = """<html><body><div class="match-page">
<div class="lineups">
 <div class="lineup"><div class="box-headline"><a href="/team/1/alpha">Alpha</a>
  <div class="teamRanking"><a href="/ranking/teams/2025/march/3/1" class="ranking-badge hltv-ranking"><span>HLTV: </span>#4</a>
  <a href="/valve-ranking/teams/2025/march/9?teamId=1" class="ranking-badge vrs-ranking"><span>VRS: </span>#2</a></div></div>
  <div class="players"><table><tr>
   <td class="player player-image"><a href="/player/11/a1"><div><div class="role-pills"><span class="role-pill role-pill--igl"></span></div></div></a></td>
   <td class="player player-image"><a href="/player/12/a2"><div class="lineup-standin" title="Substituting for old"></div></a></td>
  </tr></table></div></div>
 <div class="lineup"><div class="box-headline"><a href="/team/2/bravo">Bravo</a>
  <div class="teamRanking"><a href="/ranking/teams/2025/march/3/2" class="ranking-badge hltv-ranking"><span>HLTV: </span>#9</a></div></div>
  <div class="players"><table><tr>
   <td class="player player-image"><a href="/player/21/coachy"><div></div></a></td>
  </tr></table></div></div>
</div>
<div class="veto-box"><div class="padding preformatted-text">Best of 3 (LAN)
* Swiss round 4 (teams with a 2-1 record). Winner advances to playoffs
** coachy (coach) substitutes b9.
** Map 1 was overturned by an admin decision.</div></div>
<div class="past-matches-grid" data-past-matches-team=""><div class="past-matches-box"><table class="past-matches-table">
 <tr><td class="past-matches-team"><div class="past-matches-teamname"><a href="/team/5/x">X</a></div><div class="past-matches-time-ago">3 weeks ago</div></td>
 <td class="past-matches-map"><a href="/matches/900/x">bo3</a></td><td class="past-matches-score"><a href="/matches/900/x" class="past-matches-cell won">2 - 0</a></td></tr>
</table><div class="past-matches-streak">1 match win streak</div></div><div class="past-matches-box"><table class="past-matches-table"></table></div></div>
<div class="head-to-head"><div class="bold">1</div><div class="bold">0</div><div class="bold">0</div></div>
<div class="head-to-head-listing"><table><tr class="row"><td class="date"><a href="/matches/800/x"><span data-unix="1735689600000">1/1 25</span></a></td>
 <td class="team1 winner" title="a1, a2"><a href="/team/1/alpha">Alpha</a></td><td class="team2" title="b1"><a href="/team/2/bravo">Bravo</a></td>
 <td class="map"><div class="dynamic-map-name-full">Nuke</div></td><td class="result"><span class="won">13</span> - <span class="lost">7</span></td></tr></table></div>
<div class="mapholder"><div class="mapname">Nuke</div><div class="results-center-half-score"><span>(</span><span class="ct">8</span><span>:</span><span class="t">4</span><span>;</span><span class="t">5</span><span>:</span><span class="ct">3</span><span>)</span></div><span class="results-right pick"></span></div>
<div class="fbw-vp-team-line-odds">1.50 2.40</div>
<div class="highlighted-player potm-container">POTM</div>
</div></body></html>"""


def test_parser_reads_safe_fields_and_never_the_leaky_ones():
    d = parse_page(PAGE)
    pre, post = d["pre"], d["post"]
    assert [r["rank"] for r in pre["ranks"]] == [4, 9]
    assert pre["ranks"][0]["date"] == "2025-03-03"  # the HLTV ranking date, not the VRS badge's
    flat = json.dumps(d)
    for leaky in ("vrs", "VRS", "odds", "1.50", "potm", "POTM", "weeks ago", "overturned", "pick"):
        assert leaky not in flat, leaky
    assert pre["lineups"][0][0]["igl"] and pre["lineups"][0][1]["standin"]
    assert pre["lineups"][0][1]["replaced"] == "old"
    assert pre["notes"]["stage"] == ["Swiss round 4 (teams with a 2-1 record). Winner advances to playoffs"]
    assert pre["notes"]["coach_subs"] == [{"coach": "coachy", "for": "b9"}]
    assert pre["past"]["team"][0][0] == {"mid": 900, "opp_id": 5, "opp": "X", "won": True, "for": 2,
                                         "against": 0, "token": "bo3"}
    assert pre["streak"]["team"] == [1, 0]
    assert pre["h2h"]["summary"] == [1, 0, 0] and pre["h2h"]["rows"][0]["winner1"]
    assert post["halves"] == [{"map": "Nuke", "a_ct": 8, "a_t": 5, "b_ct": 3, "b_t": 4, "a_ct_n": 12, "a_t_n": 8}]
    assert set(d) == {"pre", "post"} and "halves" not in pre and "lineups" not in post


def test_rank_history_reads_only_rankings_published_before_the_cutoff():
    r = hltv_data.RankHistory()
    r.add("alpha", hltv_data.end_of_day("2025-03-03"), 4)
    r.add("alpha", hltv_data.end_of_day("2025-03-10"), 7)
    r.finish()
    assert r.at("alpha", T.parse("2025-03-10T17:00:00Z")) == 4  # that Monday's ranking not yet known
    assert r.at("alpha", T.parse("2025-03-11T00:00:00Z")) == 7
    assert r.at("alpha", T.parse("2025-03-03T23:59:59Z")) is None
    assert r.at("alpha", T.parse("2025-04-20T00:00:00Z")) is None  # older than 35 days


# ------------------------------------------------------------------ on the research archive

def _have_archive():
    return paths.ARCHIVE.is_dir()


@pytest.fixture(scope="module")
def real():
    if not _have_archive():
        pytest.skip("research archive not present")
    from kbml.table import cache_dir, load_table
    m, mp, _ = load_table(log=lambda *a: None)
    m2, pre, rec, meta = hltv_data.load(m, cache_dir(), log=lambda *a: None)
    pages = pickle.loads((cache_dir() / "hltv_pages.pkl").read_bytes())["pages"]
    return m2, mp, pre, rec, meta, pages


def test_guards_found_nothing_to_drop(real):
    *_, meta, _ = real
    assert meta["guards"] == {"past_rows_dropped": 0, "h2h_rows_dropped": 0, "unoriented": 0, "unparsed": 0}


def test_rank_columns_are_known_before_the_cutoff_and_recent(real):
    """Every rank on a Match is the team's latest HLTV ranking published (end of its UTC day)
    strictly before the cutoff and at most 35 days old, never a VRS badge."""
    m, _, _, rec, _, pages = real
    by_id = {int(Path(k).name.split("-")[2].split(".")[0]): v for k, v in pages.items()}
    badges: dict[str, list] = {}
    for r in rec["rec"]:
        page = by_id.get(r["hltv_match_id"])
        for side, b in enumerate(page["pre"]["ranks"] if page else []):
            if b:
                badges.setdefault(r["keys"][side], []).append((hltv_data.end_of_day(b["date"]), b["rank"]))
    checked = 0
    for key, rank, c in zip(m["team_a_key"], m["hltv_rank_a"], m["cutoff"]):
        known = [x for x in badges.get(key, []) if x[0] < c]
        if pd.isna(rank):
            assert not known or max(known)[0] < c - 35 * T.DAY
            continue
        latest = max(o for o, _ in known)
        assert latest >= c - 35 * T.DAY
        assert int(rank) in {r for o, r in known if o == latest}
        checked += 1
    assert checked > 1500


def test_past_matches_rows_are_strictly_earlier_and_streaks_match_the_badge(real):
    m, _, pre, _, _, pages = real
    hid_cut = dict(zip(m["hltv_match_id"].dropna().astype(int), m.loc[m["hltv_match_id"].notna(), "cutoff"]))
    own = dict(zip(m["match_id"], m["hltv_match_id"]))
    cut = dict(zip(m["match_id"], m["cutoff"]))
    checked = 0
    for mid, p in pre.items():
        for kind in ("team", "core"):
            for side, rows in enumerate(p["past"][kind]):
                for r in rows:
                    assert r["mid"] != own[mid]
                    if r["mid"] in hid_cut:
                        assert hid_cut[r["mid"]] < cut[mid]
                        checked += 1
                lead = 0
                for r in rows:
                    if not r["won"]:
                        break
                    lead += 1
                badge = p["streak"][kind][side]
                assert badge == 0 or badge == lead
    assert checked > 1000


def test_head_to_head_rows_are_before_the_cutoff_and_add_up(real):
    m, _, pre, _, _, pages = real
    cut = dict(zip(m["match_id"], m["cutoff"]))
    for mid, p in pre.items():
        assert all(r["unix_ms"] * 1000 < cut[mid] for r in p["h2h"]["rows"])
    ok = n = 0
    for page in pages.values():
        h = page["pre"]["h2h"]
        if h and h["summary"]:
            n += 1
            w1, _, w2 = h["summary"]
            ok += (w1 + w2 + sum(1 for r in h["rows"] if not (r["winner1"] or r["winner2"]))) == len(h["rows"])
    assert ok / n > 0.99


def test_lineups_are_the_players_of_the_stats(real):
    m, _, pre, rec, _, _ = real
    same = total = 0
    for r in rec["rec"]:
        if not r["lineups"]:
            continue
        stats = [{p["player_id"] for p in t["players"]} for t in r["overall"][:2]]
        for side in (0, 1):
            total += 1
            same += set(r["lineups"][side]) == stats[side]
    assert same / total > 0.99


def test_half_score_sides_add_up_to_the_map_score(real):
    _, _, _, rec, _, _ = real
    ok = n = 0
    for r in rec["rec"]:
        played = [mp for mp in r["maps"] if mp.get("played") and mp.get("team_a_score") is not None]
        for h, mp in zip(r["halves"], played):
            n += 1
            sa, sb = mp["team_a_score"], mp["team_b_score"]
            got = (h["a_ct"] + h["a_t"], h["b_ct"] + h["b_t"])
            if max(sa, sb) <= 13:
                ok += got == (sa, sb)
            else:  # overtime halves carry no side tag: the 24 regulation rounds are read
                ok += sum(got) == 24 and got[0] <= sa and got[1] <= sb
    assert n > 3000 and ok / n > 0.99


def test_post_match_data_only_reaches_later_matches(real):
    """Scrambling everything one Match produced (stats, sides, half scores, maps, veto, lineups)
    moves no feature of any Match whose cutoff is not after that Match's known finish, and moves
    some later ones."""
    m, mp, pre, rec, _, _ = real
    groups = F.SETS["hltv"]
    base, _ = F.compute(m, mp, groups, (pre, rec))
    i = len(rec) // 2
    target = rec.iloc[i]
    changed = rec.copy()
    r = copy.deepcopy(target["rec"])
    for team in r["overall"]:
        for p in team["players"]:
            p["rating"], p["swing_pct"], p["eadr"] = 0.1, -20.0, 10.0
    for h in r["halves"]:
        h["a_ct"], h["b_ct"] = h["b_ct"], h["a_ct"]
    for x in r["maps"]:
        x["winner"] = {"a": "b", "b": "a"}.get(x.get("winner"), x.get("winner"))
    r["veto"] = [dict(s, action="removed") for s in r["veto"]]
    r["lineups"] = [[1, 2, 3, 4, 5], [6, 7, 8, 9, 10]]
    r["sides"] = {pid: {"ct": {"rating": 0.1}, "t": {"rating": 0.1}} for pid in r["sides"]}
    changed.at[changed.index[i], "rec"] = r
    after, _ = F.compute(m, mp, groups, (pre, changed))
    finish = int(target["known_finish"])
    early = m.loc[m["cutoff"] <= finish, "match_id"]
    late = m.loc[m["cutoff"] > finish, "match_id"]
    pd.testing.assert_frame_equal(base.loc[early], after.loc[early])
    assert not base.loc[late].equals(after.loc[late])


def test_pre_match_page_data_only_reaches_its_own_match(real):
    m, mp, pre, rec, _, _ = real
    groups = F.SETS["hltv"]
    base, _ = F.compute(m, mp, groups, (pre, rec))
    mid = sorted(pre)[len(pre) // 2]
    changed = dict(pre)
    p = copy.deepcopy(pre[mid])
    for kind in p["past"]:
        for rows in p["past"][kind]:
            for r in rows:
                r["won"] = not r["won"]
    for side in p["lineups"]:
        for pl in side:
            pl["standin"] = True
    changed[mid] = p
    after, _ = F.compute(m, mp, groups, (changed, rec))
    others = [x for x in base.index if x != mid]
    pd.testing.assert_frame_equal(base.loc[others], after.loc[others])
    assert not base.loc[[mid]].equals(after.loc[[mid]])
