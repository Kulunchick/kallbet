-- Ticket #37 (spec #33): whether a lab run evaluated the Holdout.
-- 'pending' (no Holdout Match played) or 'evaluated: <frozen combination>'
-- ('evaluated: anchor' when nothing was promoted). Older runs: NULL.
ALTER TABLE lab_runs ADD COLUMN holdout TEXT;
