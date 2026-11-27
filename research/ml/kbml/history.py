"""Point-in-time access to past results. A feature function cannot see the future by construction.

Two ways in, both guarded by the same rule: a result is visible at `cutoff` only when its known
finish is set and strictly before `cutoff`. A result without a known finish is dropped at
construction and is never visible (a start time is not a finish; no duration is estimated).

- `History.before(cutoff)`: an immutable slice of the results (and played maps) known at `cutoff`.
- `Timeline`: the results in known-finish order, released to stateful feature groups (ratings)
  only by `advance_to(cutoff)`, which never goes backwards. A group sees a result only once the
  driver has advanced past its finish, so it cannot read anything later than the Match it is
  asked about.

What a feature function gets about the Match itself is a `MatchInfo`: the pre-match columns only
(teams, format, times, market and Valve inputs, venue); the winner, map score and finish are not
on it.
"""
from __future__ import annotations

from dataclasses import dataclass

import numpy as np
import pandas as pd

#: Columns of the Match table that are known before a Match starts.
PRE_MATCH = [
    "match_id", "event_id", "event", "tier", "is_qualifier", "in_scope", "stage_id", "stage", "stage_kind",
    "round", "team_a_id", "team_a", "team_a_key", "team_b_id", "team_b", "team_b_key", "format",
    "scheduled_start", "cutoff", "market_p", "market_source", "vrs_map_p", "vrs_series_p",
    "vrs_points_a", "vrs_points_b", "anchor_p", "hltv_is_lan",
]

#: Columns a past result exposes to features.
RESULT = PRE_MATCH + ["known_finish", "team_a_won", "maps_a", "maps_b", "winner_id"]


class LeakError(RuntimeError):
    pass


@dataclass(frozen=True)
class MatchInfo:
    """The pre-match view of one Match."""
    row: dict

    def __getitem__(self, key):
        if key not in PRE_MATCH:
            raise LeakError(f"{key!r} is not known before the Match")
        return self.row[key]

    @property
    def cutoff(self) -> int:
        return int(self.row["cutoff"])


def match_infos(matches: pd.DataFrame) -> list[MatchInfo]:
    cols = [c for c in PRE_MATCH if c in matches.columns]
    return [MatchInfo(r) for r in matches[cols].to_dict("records")]


class _Sorted:
    """Rows with a known finish, sorted by it; slices by `known_finish < cutoff`."""

    def __init__(self, frame: pd.DataFrame, columns: list[str]):
        known = frame[frame["known_finish"].notna()]
        cols = [c for c in columns if c in known.columns]
        self.frame = known[cols].sort_values(["known_finish", "match_id"], kind="stable").reset_index(drop=True)
        self.finish = self.frame["known_finish"].to_numpy(dtype=np.int64)

    def before(self, cutoff: int) -> pd.DataFrame:
        n = int(np.searchsorted(self.finish, int(cutoff), side="left"))  # strictly before
        out = self.frame.iloc[:n]
        return out


class History:
    def __init__(self, matches: pd.DataFrame, maps: pd.DataFrame | None = None):
        self._matches = _Sorted(matches, RESULT)
        map_cols = ["match_id", "hltv_match_id", "known_finish", "map_name", "team_a_key", "team_b_key",
                    "rounds_a", "rounds_b", "a_won", "is_lan", "position"]
        self._maps = _Sorted(maps, map_cols) if maps is not None and len(maps) else None

    def before(self, cutoff: int) -> "Slice":
        return Slice(self._matches.before(cutoff),
                     self._maps.before(cutoff) if self._maps is not None else pd.DataFrame(), int(cutoff))


@dataclass(frozen=True)
class Slice:
    matches: pd.DataFrame
    maps: pd.DataFrame
    cutoff: int

    def team(self, key: str) -> pd.DataFrame:
        m = self.matches
        return m[(m["team_a_key"] == key) | (m["team_b_key"] == key)]


class Timeline:
    """Results released in known-finish order; `advance_to` never goes backwards."""

    def __init__(self, history: History):
        self._results = history._matches.frame
        self._rfinish = history._matches.finish
        self._maps = history._maps.frame if history._maps is not None else None
        self._mfinish = history._maps.finish if history._maps is not None else None
        self._ri = 0
        self._mi = 0
        self.now: int | None = None

    def advance_to(self, cutoff: int):
        """(new results, new maps) whose known finish is strictly before `cutoff`."""
        cutoff = int(cutoff)
        if self.now is not None and cutoff < self.now:
            raise LeakError("the timeline only moves forward")
        self.now = cutoff
        rj = int(np.searchsorted(self._rfinish, cutoff, side="left"))
        new_r = self._results.iloc[self._ri:rj]
        self._ri = rj
        new_m = pd.DataFrame()
        if self._maps is not None:
            mj = int(np.searchsorted(self._mfinish, cutoff, side="left"))
            new_m = self._maps.iloc[self._mi:mj]
            self._mi = mj
        return new_r, new_m
