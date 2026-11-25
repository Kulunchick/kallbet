-- Ticket #29: an LLM answer is reused under a key coarser than the dossier hash
-- (the market reduced to its step and favourite), so that a line moving a
-- fraction of a point does not ask the subscriptions again. `input_hash` stays
-- the dossier hash of the run that asked.
ALTER TABLE debate_turns ADD COLUMN reuse_key TEXT NOT NULL DEFAULT '';
CREATE INDEX debate_turns_reuse ON debate_turns (bot, tier, reuse_key, role);
