"""Phase 3: the extended archive adds training rows only, and the weekly ranking is point in time."""
from pathlib import Path

import numpy as np
import pandas as pd
import pytest

from kbml import hltv_data, paths
from kbml import timeutil as T
from kbml.hltv_html import parse_ranking
from kbml.table import block_of

EXT = Path("D:/Projects/KallBet/target/research-archive-ext")

RANKING = """<html><head><link rel="canonical" href="https://www.hltv.org/ranking/teams/2024/january/1"></head><body>
<div class="ranked-team"><div class="ranking-header"><span class="position">#1</span>
 <div class="teamLine"><span class="name">Vitality</span><span class="points">(980 HLTV points)</span></div></div>
 <div class="more"><a class="moreLink" href="/team/9565/vitality">profile</a><a class="moreLink" href="/ranking/teams/2024/january/1/details/9565">d</a></div></div>
<div class="ranked-team"><div class="ranking-header"><span class="position">#2</span>
 <div class="teamLine"><span class="name">FaZe</span><span class="points">(1,020 HLTV points)</span></div></div>
 <div class="more"><a class="moreLink" href="/team/6667/faze">profile</a></div></div>
</body></html>"""


def test_ranking_page_parses_like_the_rust_parser():
    page = parse_ranking(RANKING)
    assert page["date"] == "2024-01-01"
    assert page["teams"] == [{"rank": 1, "team_id": 9565, "name": "Vitality", "points": 980},
                             {"rank": 2, "team_id": 6667, "name": "FaZe", "points": 1020}]


def test_a_monday_ranking_is_not_known_on_that_monday():
    pages = [{"date": "2024-01-01", "known": hltv_data.end_of_day("2024-01-01"),
              "teams": [{"rank": 3, "team_id": 1, "name": "Alpha", "points": 500}]},
             {"date": "2024-01-08", "known": hltv_data.end_of_day("2024-01-08"),
              "teams": [{"rank": 1, "team_id": 1, "name": "Alpha", "points": 900}]}]
    m = pd.DataFrame({"match_id": [1, 2, 3], "team_a_key": ["alpha"] * 3, "team_b_key": ["bravo"] * 3,
                      "cutoff": [T.parse("2024-01-08T18:00:00Z"), T.parse("2024-01-09T00:00:00Z"),
                                 T.parse("2024-03-01T00:00:00Z")]})
    out = hltv_data.weekly_columns(m.copy(), {}, pages)
    assert list(out["rw_rank_a"].astype("float").fillna(-1)) == [3, 1, -1]  # the last: older than 35 days
    assert out["rw_rank_b"].isna().all() and out["rw_size"].iloc[0] == 1


def test_blocks():
    assert block_of("s", False, True) == "scope"
    assert block_of("a", False, False) == "sa_old"
    assert block_of("c", True, False) == "qualifier"
    assert block_of("b", False, False) == "b" and block_of(None, False, False) == "other"


@pytest.fixture(scope="module")
def ext():
    if not EXT.is_dir() or not paths.ARCHIVE.is_dir():
        pytest.skip("archives not present")
    from kbml.run import with_hltv
    from kbml.table import load_table
    a, _, _ = load_table(paths.ARCHIVE, log=lambda *x: None)
    b, _, _ = load_table(EXT, log=lambda *x: None)
    b2, _ = with_hltv(b, EXT)
    return a, b, b2


def test_the_scored_set_is_exactly_the_snapshots(ext):
    a, b, _ = ext
    sa = a[a.in_scope & a.anchor_p.notna()].set_index("match_id").sort_index()
    sb = b[b.in_scope & b.anchor_p.notna()].set_index("match_id").sort_index()
    assert list(sa.index) == list(sb.index) and len(sa) == 2087
    for col in ("anchor_p", "event_id", "event_begins", "event_ended", "cutoff", "team_a_won", "tier"):
        assert sa[col].equals(sb[col]), col
    assert set(b.loc[~b.in_scope, "block"]) <= {"sa_old", "b", "c", "qualifier", "other"}


def test_weekly_rank_is_known_before_the_cutoff(ext):
    _, _, m = ext
    has = m["rw_known"].notna()
    assert has.mean() > 0.9
    known = m.loc[has, "rw_known"].astype("int64")
    cut = m.loc[has, "cutoff"].astype("int64")
    assert (known < cut).all() and (known >= cut - 35 * T.DAY).all()
    pages = hltv_data.weekly_rankings(EXT)
    k = np.array([p["known"] for p in pages])
    for c, kn in zip(cut, known):  # it is the latest page known before the cutoff
        assert kn == k[k < c].max()
