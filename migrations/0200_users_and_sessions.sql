-- Ticket #2: Users and their server-side sessions.
-- Timestamps are UTC, written by sqlx from chrono::DateTime<Utc> (RFC 3339 text).

CREATE TABLE users (
    id          INTEGER PRIMARY KEY,
    telegram_id INTEGER NOT NULL UNIQUE,
    nickname    TEXT    NOT NULL COLLATE NOCASE UNIQUE,
    avatar_url  TEXT,
    language    TEXT    NOT NULL DEFAULT 'en',
    created_at  TEXT    NOT NULL
);

CREATE TABLE sessions (
    token_hash TEXT    PRIMARY KEY,
    user_id    INTEGER NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    created_at TEXT    NOT NULL,
    expires_at TEXT    NOT NULL
);

CREATE INDEX sessions_user_id ON sessions (user_id);
