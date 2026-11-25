CREATE TABLE odds_batches (
    scope TEXT NOT NULL,
    captured_at TEXT NOT NULL,
    lines BLOB NOT NULL,
    PRIMARY KEY (scope, captured_at)
);
