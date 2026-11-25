-- Follow-up to ticket #22 (spec #21): Bot Users and Pick'em.

-- Hard Lock of a Stage's Pick'em (ADR-0003's amendment): the Stage's first
-- Match starting. `locked_at` may be earlier (the humans Lock early once all
-- of them have submitted); a Bot User may still write until `hard_locked_at`.
-- Set once and never cleared, like `locked_at`.
--
-- Deliberately no backfill (unlike `match_locks` in 2200): a Lock stored
-- before Bot Users existed keeps `hard_locked_at` NULL, and the first
-- evaluation after the deploy stamps it for every Stage whose first Match has
-- started by the data. Copying `locked_at` would close every Stage the humans
-- had already Locked early to the Bot Users that join later, although it has
-- not started. See "Bot Users" in docs/agents/codebase.md.
ALTER TABLE pickem_locks ADD COLUMN hard_locked_at TEXT;
