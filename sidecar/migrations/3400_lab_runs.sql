-- Ticket #34 (spec #33): one row per `kallbet-ai lab` run of the Signal Lab.
CREATE TABLE lab_runs (
    id INTEGER PRIMARY KEY,
    started_at TEXT NOT NULL,
    finished_at TEXT NOT NULL,
    -- `LabReport::archive_digest`: the SHA-256 of the replay archive.
    archive_sha256 TEXT NOT NULL,
    archive_files INTEGER NOT NULL,
    -- `lab::PROTOCOL_VERSION` of the run.
    protocol_version TEXT NOT NULL,
    -- Seed of the Event bootstrap (a u64 stored by its bits).
    seed INTEGER NOT NULL,
    -- JSON {candidate signal name: verdict}, in report order.
    verdicts TEXT NOT NULL
);
