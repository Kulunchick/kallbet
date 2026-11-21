-- Ticket #4: Events (as far as a League needs them), Leagues and memberships.
-- Stages, teams and Matches are added by ticket #5 in its own migration.
-- Timestamps are UTC, written by sqlx from chrono::DateTime<Utc>.

-- id is the canonical (HLTV) event id.
CREATE TABLE events (
    id        INTEGER PRIMARY KEY,
    name      TEXT NOT NULL,
    starts_at TEXT,
    ends_at   TEXT
);

CREATE TABLE leagues (
    id           INTEGER PRIMARY KEY,
    event_id     INTEGER NOT NULL REFERENCES events (id),
    name         TEXT    NOT NULL,
    creator_id   INTEGER NOT NULL REFERENCES users (id),
    -- The secret in the Invite Link. Reissuing replaces it.
    invite_token TEXT    NOT NULL UNIQUE,
    archived_at  TEXT,
    created_at   TEXT    NOT NULL
);

CREATE INDEX leagues_event_id ON leagues (event_id);

-- One row per stretch of membership. Leaving sets left_at; joining again
-- later adds a new row, so joined_at / left_at keep the history that the
-- Eligible Member rule needs. A current Member has left_at IS NULL.
CREATE TABLE memberships (
    id        INTEGER PRIMARY KEY,
    league_id INTEGER NOT NULL REFERENCES leagues (id) ON DELETE CASCADE,
    user_id   INTEGER NOT NULL REFERENCES users (id),
    role      TEXT    NOT NULL CHECK (role IN ('member', 'admin')),
    joined_at TEXT    NOT NULL,
    left_at   TEXT
);

-- At most one current membership per User per League.
CREATE UNIQUE INDEX memberships_current ON memberships (league_id, user_id) WHERE left_at IS NULL;
CREATE INDEX memberships_user_id ON memberships (user_id);
