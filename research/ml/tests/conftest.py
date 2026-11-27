import numpy as np
import pandas as pd
import pytest

from kbml import timeutil as T

TEAMS = ["Alpha", "Bravo", "Charlie", "Delta", "Echo", "Foxtrot"]


def synthetic(n_events: int = 6, per_event: int = 12, seed: int = 1) -> tuple[pd.DataFrame, pd.DataFrame]:
    """A small Match table and map table shaped like `table.build`'s output."""
    rng = np.random.default_rng(seed)
    start = T.parse("2025-01-01T00:00:00Z")
    rows, maps = [], []
    mid = 1000
    for e in range(n_events):
        begins = start + e * 10 * T.DAY
        for k in range(per_event):
            a, b = rng.choice(len(TEAMS), 2, replace=False)
            sched = begins + k * 6 * T.HOUR
            actual = sched + 5 * T.MINUTE
            finish = actual + 2 * T.HOUR
            won = bool(rng.random() < 0.5 + 0.08 * (b - a))
            mk = float(np.clip(0.5 + 0.06 * (b - a) + rng.normal(0, 0.05), 0.05, 0.95))
            rows.append(dict(
                match_id=mid, duplicate=False, event_id=e + 1, event=f"Event {e + 1}", tier="s",
                is_qualifier=False, in_scope=True, event_begins=begins - 15 * T.MINUTE,
                event_ended=begins + (per_event - 1) * 6 * T.HOUR + 5 * T.MINUTE + 2 * T.HOUR,
                stage_id=e + 1, stage="Stage", stage_kind="other", stage_cutoff=begins - 15 * T.MINUTE,
                name="", round=None, team_a_id=int(a), team_a=TEAMS[a], team_a_key=TEAMS[a].lower(),
                team_b_id=int(b), team_b=TEAMS[b], team_b_key=TEAMS[b].lower(), format="bo3",
                scheduled_start=sched, actual_start=actual, cutoff=sched - 15 * T.MINUTE, known_finish=finish,
                finished_at=finish, winner_id=int(a if won else b), team_a_won=won, maps_a=2 if won else 1,
                maps_b=1 if won else 2, market_p=mk, market_source="oddspapi", market_observed=sched - T.HOUR,
                vrs_map_p=0.5, vrs_series_p=0.5, vrs_observed=None, vrs_points_a=1500, vrs_points_b=1500,
                anchor_p=mk, hltv_match_id=None, hltv_file=None, hltv_is_lan=True, lp_event_title=None,
                lp_roster_file=None, lp_roster_revision=None))
            for pos, map_name in enumerate(["Mirage", "Nuke", "Inferno"][: 3], start=1):
                a_won = won if pos != 2 else not won
                maps.append(dict(match_id=mid, hltv_match_id=mid * 10, known_finish=finish, started_at=actual,
                                 is_lan=True, position=pos, map_name=map_name, team_a_key=TEAMS[a].lower(),
                                 team_b_key=TEAMS[b].lower(), rounds_a=13 if a_won else 8,
                                 rounds_b=8 if a_won else 13, a_won=a_won))
            mid += 1
    m = pd.DataFrame(rows)
    for c in ("known_finish", "event_ended"):
        m[c] = m[c].astype("Int64")
    mp = pd.DataFrame(maps)
    mp["known_finish"] = mp["known_finish"].astype("Int64")
    return m, mp


@pytest.fixture
def table():
    return synthetic()
