-- Ticket #24, spec-21 review: a failure of one step of a tick (one Match, one
-- League, one Stage, one Invite Link) is recorded here, and the tick goes on
-- with the next one.
CREATE TABLE tick_failures (
    id INTEGER PRIMARY KEY,
    at TEXT NOT NULL,
    bot TEXT NOT NULL,
    step TEXT NOT NULL,
    league INTEGER,
    match_id INTEGER,
    stage INTEGER,
    error TEXT NOT NULL
);

-- A Prediction submitted from the anchor alone because every LLM chain (or the
-- Arbiter) failed. It is not a decision of its run: a later tick with the same
-- dossier asks the LLM again (under the reuse and breaker rules) and, when it
-- answers, decides and submits as usual.
CREATE TABLE unadjusted_submissions (
    run_id INTEGER PRIMARY KEY REFERENCES dossier_runs(id),
    submitted_at TEXT NOT NULL,
    llm_error TEXT NOT NULL,
    payload TEXT NOT NULL
);
