-- Ticket #16: notification preferences and the notification log.

-- One switch per notification type; all on by default.
ALTER TABLE users ADD COLUMN notify_reminder INTEGER NOT NULL DEFAULT 1;
ALTER TABLE users ADD COLUMN notify_reveal   INTEGER NOT NULL DEFAULT 1;
ALTER TABLE users ADD COLUMN notify_digest   INTEGER NOT NULL DEFAULT 1;

-- Dedupe: one row per notification that was handled (delivered, or found
-- undeliverable for good). `key` names the notification, e.g.
-- `reminder:<league>:<match>:<user>`. Rows go with their League or User.
CREATE TABLE notification_log (
    key       TEXT    PRIMARY KEY,
    league_id INTEGER NOT NULL REFERENCES leagues (id) ON DELETE CASCADE,
    user_id   INTEGER NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    sent_at   TEXT    NOT NULL
);

CREATE INDEX notification_log_league_id ON notification_log (league_id);
