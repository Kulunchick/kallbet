-- Ticket #31, spec-21 review: one row per `kallbet-ai backtest` run. The table
-- 2400 created was never written; it is replaced by this shape.
DROP TABLE backtest_runs;
CREATE TABLE backtest_runs (
    id INTEGER PRIMARY KEY,
    started_at TEXT NOT NULL,
    finished_at TEXT NOT NULL,
    -- `Report::archive_digest`: the SHA-256 of the replay archive.
    archive_sha256 TEXT NOT NULL,
    archive_files INTEGER NOT NULL,
    -- JSON [[profile, description], ...]
    profiles TEXT NOT NULL,
    -- JSON: Predictions per profile, LLM calls per profile, Pick'em Points of
    -- the finished Stages, Stages replayed, why the LLM track stopped (or null).
    headline TEXT NOT NULL
);
