CREATE TABLE raw_feed_snapshots (
    id INTEGER PRIMARY KEY,
    source TEXT NOT NULL,
    observed_at TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    payload BLOB NOT NULL,
    UNIQUE(source, observed_at, content_hash)
);

CREATE TABLE vrs_snapshots (
    id INTEGER PRIMARY KEY,
    observed_at TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    markdown TEXT NOT NULL,
    UNIQUE(observed_at,content_hash)
);

CREATE TABLE dossier_runs (
    id INTEGER PRIMARY KEY,
    bot TEXT NOT NULL,
    league INTEGER NOT NULL,
    match_id INTEGER NOT NULL,
    tier TEXT NOT NULL,
    input_hash TEXT NOT NULL,
    cutoff TEXT NOT NULL,
    dossier TEXT NOT NULL,
    UNIQUE(bot,league,match_id,tier,input_hash)
);

CREATE TABLE decisions (
    run_id INTEGER PRIMARY KEY REFERENCES dossier_runs(id),
    prediction TEXT
);

CREATE TABLE submitted_predictions (
    run_id INTEGER PRIMARY KEY REFERENCES dossier_runs(id),
    submitted_at TEXT NOT NULL,
    payload TEXT NOT NULL
);

CREATE TABLE debate_turns (
    id INTEGER PRIMARY KEY,
    input_hash TEXT NOT NULL,
    role TEXT NOT NULL,
    provider TEXT NOT NULL,
    latency_ms INTEGER NOT NULL,
    status TEXT NOT NULL,
    brief TEXT
);

CREATE TABLE submitted_pickems (
    id INTEGER PRIMARY KEY,
    bot TEXT NOT NULL,
    league INTEGER NOT NULL,
    stage INTEGER NOT NULL,
    submitted_at TEXT NOT NULL,
    input_hash TEXT NOT NULL,
    payload TEXT NOT NULL
);

CREATE TABLE backtest_runs (
    id INTEGER PRIMARY KEY,
    started_at TEXT NOT NULL,
    profile TEXT NOT NULL,
    report TEXT NOT NULL
);
