CREATE TABLE odds_batches (
    scope TEXT NOT NULL,
    captured_at TEXT NOT NULL,
    lines BLOB NOT NULL,
    PRIMARY KEY (scope, captured_at)
);

CREATE TABLE oddspapi_months (
    month TEXT PRIMARY KEY NOT NULL,
    attempts INTEGER NOT NULL CHECK (attempts > 0)
);
