-- Ticket #22 (spec #21): Bot Users.

-- A Bot User is a User that authenticates with a Bearer token instead of
-- Telegram (ADR-0004's amendment). Its `telegram_id` is a synthetic negative
-- number, like a dev-login User's, so `telegram_id UNIQUE` holds.
ALTER TABLE users ADD COLUMN is_bot INTEGER NOT NULL DEFAULT 0;

-- The Bearer token of each Bot User: only its SHA-256 (hex) is stored, like
-- `sessions`. `bot_key` is the stable name the sidecar gives the bot; one row
-- per bot key, so registering again replaces the token.
CREATE TABLE bot_tokens (
    token_hash TEXT    PRIMARY KEY,
    user_id    INTEGER NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    bot_key    TEXT    NOT NULL UNIQUE,
    created_at TEXT    NOT NULL
);

-- Hard Lock (ADR-0003's amendment): Scheduled Start minus 15 minutes, actual
-- start or cancellation, whichever came first. `locked_at` may be earlier
-- (an early Reveal Locks the Match for humans); a Bot User may still write
-- until `hard_locked_at`. Set once and never cleared, like the other stamps.
ALTER TABLE match_locks ADD COLUMN hard_locked_at TEXT;

-- Every Lock from before Bot Users existed counts as a Hard Lock: no Bot User
-- gets to predict a Match whose Predictions were already seen.
UPDATE match_locks SET hard_locked_at = locked_at WHERE locked_at IS NOT NULL;
