-- Ticket #29: a debate turn is found again by the run it belongs to (bot, tier,
-- dossier hash) and its seat, so that a repeated run reuses the answer.
ALTER TABLE debate_turns ADD COLUMN bot TEXT NOT NULL DEFAULT '';
ALTER TABLE debate_turns ADD COLUMN match_id INTEGER NOT NULL DEFAULT 0;
ALTER TABLE debate_turns ADD COLUMN tier TEXT NOT NULL DEFAULT '';
CREATE INDEX debate_turns_run ON debate_turns (bot, tier, input_hash, role);
