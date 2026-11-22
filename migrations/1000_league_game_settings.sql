CREATE TABLE prediction_edit_windows (
    league_id INTEGER NOT NULL REFERENCES leagues(id) ON DELETE CASCADE,
    match_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    ends_at TEXT NOT NULL,
    PRIMARY KEY (league_id, match_id)
);

CREATE TABLE league_stage_points (
    league_id INTEGER NOT NULL REFERENCES leagues(id) ON DELETE CASCADE,
    stage INTEGER NOT NULL,
    points INTEGER NOT NULL CHECK (points BETWEEN 0 AND 1000),
    PRIMARY KEY (league_id, stage)
);
