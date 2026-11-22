CREATE TABLE pickem_edit_windows (
    league_id INTEGER NOT NULL REFERENCES leagues(id) ON DELETE CASCADE,
    stage_ordinal INTEGER NOT NULL,
    user_id INTEGER NOT NULL REFERENCES users(id),
    ends_at TEXT NOT NULL,
    PRIMARY KEY (league_id, stage_ordinal)
);
