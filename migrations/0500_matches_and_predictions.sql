-- Ticket #5: the polled tournament data (Stages, teams, Matches), when each
-- thing was last polled, Predictions and the per-League Lock state.
-- Timestamps are UTC, written by sqlx from chrono::DateTime<Utc>.

-- A Stage is identified by its Event and its position in it.
CREATE TABLE stages (
    event_id INTEGER NOT NULL REFERENCES events (id),
    ordinal  INTEGER NOT NULL,
    kind     TEXT    NOT NULL CHECK (kind IN ('swiss', 'playoffs', 'groups')),
    name     TEXT    NOT NULL,
    PRIMARY KEY (event_id, ordinal)
);

-- id is the canonical (HLTV) team id.
CREATE TABLE teams (
    id       INTEGER PRIMARY KEY,
    name     TEXT NOT NULL,
    logo_url TEXT
);

-- The base Match data, as the Data Sources last reported it. League logic
-- never reads this directly: it goes through the effective Match view
-- (base data + that League's Overrides).
-- placement_kind: 'swiss' uses swiss_round / swiss_wins / swiss_losses,
-- 'playoffs' uses playoff_rounds_before_final / playoff_slot, 'unplaced' none.
CREATE TABLE matches (
    id                          INTEGER PRIMARY KEY,
    event_id                    INTEGER NOT NULL REFERENCES events (id),
    stage_ordinal               INTEGER NOT NULL,
    placement_kind              TEXT    NOT NULL CHECK (placement_kind IN ('swiss', 'playoffs', 'unplaced')),
    swiss_round                 INTEGER,
    swiss_wins                  INTEGER,
    swiss_losses                INTEGER,
    playoff_rounds_before_final INTEGER,
    playoff_slot                INTEGER,
    team_a                      INTEGER REFERENCES teams (id),
    team_b                      INTEGER REFERENCES teams (id),
    format                      TEXT    NOT NULL CHECK (format IN ('bo1', 'bo3', 'bo5')),
    scheduled_start             TEXT,
    -- From the Data Source, or the time the poller first saw the Match live.
    actual_start                TEXT,
    status                      TEXT    NOT NULL CHECK (status IN ('upcoming', 'live', 'finished', 'cancelled')),
    winner                      INTEGER REFERENCES teams (id),
    score_a                     INTEGER,
    score_b                     INTEGER,
    -- The Data Source of the last successful poll, and when it was.
    source                      TEXT    NOT NULL,
    updated_at                  TEXT    NOT NULL,
    FOREIGN KEY (event_id, stage_ordinal) REFERENCES stages (event_id, ordinal)
);

CREATE INDEX matches_event_id ON matches (event_id);

-- When the poller last tried each thing: 'event_list', or 'event:<id>'.
CREATE TABLE polls (
    key       TEXT PRIMARY KEY,
    polled_at TEXT NOT NULL
);

-- A Member's Prediction: the team they pick to win.
CREATE TABLE predictions (
    league_id  INTEGER NOT NULL REFERENCES leagues (id) ON DELETE CASCADE,
    match_id   INTEGER NOT NULL REFERENCES matches (id),
    user_id    INTEGER NOT NULL REFERENCES users (id),
    team_id    INTEGER NOT NULL REFERENCES teams (id),
    updated_at TEXT    NOT NULL,
    PRIMARY KEY (league_id, match_id, user_id)
);

-- Lock and Reveal per League (Eligible Members and Overrides differ between
-- Leagues). Set once when they happen and never cleared (ADR-0003).
CREATE TABLE match_locks (
    league_id   INTEGER NOT NULL REFERENCES leagues (id) ON DELETE CASCADE,
    match_id    INTEGER NOT NULL REFERENCES matches (id),
    locked_at   TEXT,
    revealed_at TEXT,
    PRIMARY KEY (league_id, match_id)
);
