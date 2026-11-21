-- Ticket #14: Pick'em. One `pickems` row per Member and Stage, its Picks in
-- `picks`. The shape is generic so the Playoffs bracket (#15) is a second
-- `kind` of Pick, not a new table: Swiss uses kinds 'three_nil', 'zero_three'
-- and 'advance' (slot = position in the list); the bracket will use 'bracket'
-- (slot = the bracket slot). A team may appear in several bracket slots, so
-- uniqueness of teams is a domain rule, not a constraint.

CREATE TABLE pickems (
    league_id     INTEGER NOT NULL REFERENCES leagues (id) ON DELETE CASCADE,
    stage_ordinal INTEGER NOT NULL,
    user_id       INTEGER NOT NULL REFERENCES users (id),
    updated_at    TEXT    NOT NULL,
    PRIMARY KEY (league_id, stage_ordinal, user_id)
);

CREATE TABLE picks (
    league_id     INTEGER NOT NULL,
    stage_ordinal INTEGER NOT NULL,
    user_id       INTEGER NOT NULL,
    kind          TEXT    NOT NULL,
    slot          INTEGER NOT NULL,
    team_id       INTEGER NOT NULL REFERENCES teams (id),
    PRIMARY KEY (league_id, stage_ordinal, user_id, kind, slot),
    FOREIGN KEY (league_id, stage_ordinal, user_id)
        REFERENCES pickems (league_id, stage_ordinal, user_id) ON DELETE CASCADE
);

-- The Lock of a Stage's Pick'em, per League (Overrides differ between
-- Leagues). Stored once when it happens and never cleared (ADR-0003).
CREATE TABLE pickem_locks (
    league_id     INTEGER NOT NULL REFERENCES leagues (id) ON DELETE CASCADE,
    stage_ordinal INTEGER NOT NULL,
    locked_at     TEXT    NOT NULL,
    PRIMARY KEY (league_id, stage_ordinal)
);
