-- Ticket #13: League-scoped Overrides.

-- A Match a League Admin added by hand belongs to that League only. It lives
-- in `matches` (so Predictions, Locks and Reveals work as for any Match) with
-- a local negative id, but `league_id` marks it as League data: the base
-- view (what the Data Sources reported) leaves such rows out, so neither
-- other Leagues nor the fallback chain ever see it. NULL for every Match a
-- Data Source reported. Deleting the League deletes its hand-added Matches.
ALTER TABLE matches ADD COLUMN league_id INTEGER REFERENCES leagues (id) ON DELETE CASCADE;

CREATE INDEX matches_league_id ON matches (league_id);

-- The Override history of a League: append-only, one row per action. The
-- Override in force for (league, match, field) is the latest row; a row with
-- `cleared = 1` removes that field's Override (the Data Sources' value
-- applies again). Fields and their columns:
--   scheduled_start: scheduled_start
--   result:          winner (NULL = the Match is Void)
--   teams:           team_a, team_b
--   hidden:          no value
--   added:           no value (the Match itself is the `matches` row)
-- `match_id` has no foreign key: it may name a hand-added Match or a Match
-- with a local id that is re-keyed later (`store::external_ids::rekey_match`
-- moves it, `rekey_team` moves the team columns).
CREATE TABLE overrides (
    id              INTEGER PRIMARY KEY,
    league_id       INTEGER NOT NULL REFERENCES leagues (id) ON DELETE CASCADE,
    match_id        INTEGER NOT NULL,
    field           TEXT    NOT NULL CHECK (field IN ('scheduled_start', 'result', 'teams', 'hidden', 'added')),
    cleared         INTEGER NOT NULL DEFAULT 0,
    scheduled_start TEXT,
    winner          INTEGER,
    team_a          INTEGER,
    team_b          INTEGER,
    author_id       INTEGER REFERENCES users (id) ON DELETE SET NULL,
    created_at      TEXT    NOT NULL
);

CREATE INDEX overrides_league ON overrides (league_id, match_id);
