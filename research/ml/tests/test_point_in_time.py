"""No feature can read a result that was not known strictly before the Match's cutoff."""
import numpy as np
import pandas as pd
import pytest

from kbml import features as F
from kbml import timeutil as T
from kbml.history import History, LeakError, MatchInfo, Timeline, match_infos

GROUPS = F.resolve("baseline")


def compute(m, mp):
    X, _ = F.compute(m, mp, GROUPS)
    return X


def test_a_later_result_changes_no_earlier_feature(table):
    m, mp = table
    before = compute(m, mp)
    # Inject a result: a new Match far in the future, and flip the last Match's winner.
    pivot = int(m["known_finish"].max())
    extra = m.iloc[[0]].copy()
    extra["match_id"] = 99999
    extra["cutoff"] = pivot + T.DAY
    extra["known_finish"] = pd.array([pivot + 2 * T.DAY], dtype="Int64")
    changed = pd.concat([m, extra], ignore_index=True)
    flip = changed.index[changed["known_finish"] == pivot]
    changed.loc[flip, "team_a_won"] = ~changed.loc[flip, "team_a_won"].astype(bool)
    after = compute(changed, mp)
    earlier = m.loc[m["cutoff"] <= pivot, "match_id"]
    pd.testing.assert_frame_equal(before.loc[earlier], after.loc[earlier])
    # the injected Match itself sees the flipped result
    assert not after.loc[[99999]].equals(before.loc[[m["match_id"].iloc[0]]].set_axis([99999]))


def test_every_flip_only_moves_matches_after_its_finish(table):
    m, mp = table
    base = compute(m, mp)
    rng = np.random.default_rng(0)
    for i in rng.choice(len(m), 8, replace=False):
        changed = m.copy()
        changed.loc[i, "team_a_won"] = not bool(changed.loc[i, "team_a_won"])
        changed.loc[i, "maps_a"], changed.loc[i, "maps_b"] = changed.loc[i, "maps_b"], changed.loc[i, "maps_a"]
        cmaps = mp.copy()
        sel = cmaps["match_id"] == changed.loc[i, "match_id"]
        cmaps.loc[sel, "a_won"] = ~cmaps.loc[sel, "a_won"]
        cmaps.loc[sel, ["rounds_a", "rounds_b"]] = cmaps.loc[sel, ["rounds_b", "rounds_a"]].to_numpy()
        after = compute(changed, cmaps)
        finish = int(m.loc[i, "known_finish"])
        unaffected = m.loc[m["cutoff"] <= finish, "match_id"]
        pd.testing.assert_frame_equal(base.loc[unaffected], after.loc[unaffected])
        affected = m.loc[m["cutoff"] > finish, "match_id"]
        assert not base.loc[affected].equals(after.loc[affected]), "a flip must reach later Matches"


def test_a_result_finishing_at_the_cutoff_is_not_visible(table):
    m, mp = table
    target = m.iloc[20]
    h = History(m, mp)
    s = h.before(int(target["cutoff"]))
    assert (s.matches["known_finish"] < target["cutoff"]).all()
    # move one result's finish exactly onto the cutoff: still not visible
    m2 = m.copy()
    m2.loc[5, "known_finish"] = int(target["cutoff"])
    s2 = History(m2, mp).before(int(target["cutoff"]))
    assert m2.loc[5, "match_id"] not in set(s2.matches["match_id"])
    m2.loc[5, "known_finish"] = int(target["cutoff"]) - 1
    s3 = History(m2, mp).before(int(target["cutoff"]))
    assert m2.loc[5, "match_id"] in set(s3.matches["match_id"])


def test_a_result_without_known_finish_is_never_visible(table):
    m, mp = table
    base = compute(m, mp)
    m2 = m.copy()
    m2.loc[3, "known_finish"] = pd.NA
    m2.loc[3, "team_a_won"] = not bool(m2.loc[3, "team_a_won"])
    mp2 = mp.copy()
    mp2.loc[mp2["match_id"] == m2.loc[3, "match_id"], "known_finish"] = pd.NA
    h = History(m2, mp2)
    end = int(m["known_finish"].max()) + 10 * T.DAY
    assert m2.loc[3, "match_id"] not in set(h.before(end).matches["match_id"])
    no_result = m.drop(index=3)
    no_maps = mp[mp["match_id"] != m.loc[3, "match_id"]]
    pd.testing.assert_frame_equal(compute(m2, mp2).drop(index=m.loc[3, "match_id"]),
                                  compute(no_result, no_maps))
    assert base is not None


def test_timeline_only_moves_forward(table):
    m, mp = table
    tl = Timeline(History(m, mp))
    tl.advance_to(int(m["cutoff"].iloc[10]))
    with pytest.raises(LeakError):
        tl.advance_to(int(m["cutoff"].iloc[2]))


def test_match_info_holds_no_result(table):
    m, _ = table
    info = match_infos(m)[0]
    assert isinstance(info, MatchInfo)
    for col in ("team_a_won", "winner_id", "maps_a", "known_finish"):
        with pytest.raises(LeakError):
            info[col]
