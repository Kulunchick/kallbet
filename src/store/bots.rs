//! Persistence of Bot Users: the `users` row with `is_bot = 1` and its one
//! `bot_tokens` row (bot key and the SHA-256 of the current Bearer token).
//!
//! Registration runs on one connection inside a write transaction
//! (`db::begin_write`), so finding a free Nickname and a free synthetic
//! Telegram id cannot race with another registration.

use chrono::{DateTime, Utc};
use sqlx::{SqliteConnection, SqlitePool};

use crate::domain::{Language, UserId};

/// The Bot User registered under `bot_key`, if any.
pub async fn find_by_key(
    db: &mut SqliteConnection,
    bot_key: &str,
) -> Result<Option<UserId>, sqlx::Error> {
    sqlx::query_scalar("SELECT user_id FROM bot_tokens WHERE bot_key = ?")
        .bind(bot_key)
        .fetch_optional(db)
        .await
}

/// The Bot User a Bearer token (given by its hash) belongs to.
pub async fn find_by_token(
    db: &SqlitePool,
    token_hash: &str,
) -> Result<Option<UserId>, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT t.user_id FROM bot_tokens t JOIN users u ON u.id = t.user_id \
         WHERE t.token_hash = ? AND u.is_bot = 1",
    )
    .bind(token_hash)
    .fetch_optional(db)
    .await
}

/// Whether another User than `except` has this Nickname (case-insensitive,
/// like Nickname uniqueness).
pub async fn nickname_taken(
    db: &mut SqliteConnection,
    nickname: &str,
    except: Option<UserId>,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM users WHERE nickname = ?1 AND id IS NOT ?2)")
        .bind(nickname)
        .bind(except)
        .fetch_one(db)
        .await
}

/// Inserts a Bot User: `is_bot`, every notification off, and the next free
/// negative Telegram id (below every dev-login User's and every other Bot
/// User's, so `telegram_id UNIQUE` holds).
pub async fn create(
    db: &mut SqliteConnection,
    nickname: &str,
    avatar_url: Option<&str>,
    language: Language,
    now: DateTime<Utc>,
) -> Result<UserId, sqlx::Error> {
    sqlx::query_scalar(
        "INSERT INTO users (telegram_id, nickname, avatar_url, language, created_at, is_bot, \
                            notify_reminder, notify_reveal, notify_digest) \
         VALUES ((SELECT MIN(COALESCE(MIN(telegram_id), 0), 0) - 1 FROM users), \
                 ?, ?, ?, ?, 1, 0, 0, 0) \
         RETURNING id",
    )
    .bind(nickname)
    .bind(avatar_url)
    .bind(language.code())
    .bind(now)
    .fetch_one(db)
    .await
}

/// Sets a Bot User's profile as its sidecar configures it. `language` `None`
/// keeps the stored one; `avatar_url` `None` removes the avatar.
pub async fn update_profile(
    db: &mut SqliteConnection,
    user: UserId,
    nickname: &str,
    avatar_url: Option<&str>,
    language: Option<Language>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE users SET nickname = ?, avatar_url = ?, language = COALESCE(?, language) \
         WHERE id = ? AND is_bot = 1",
    )
    .bind(nickname)
    .bind(avatar_url)
    .bind(language.map(Language::code))
    .bind(user)
    .execute(db)
    .await?;
    Ok(())
}

/// Makes `token_hash` the one Bearer token of the Bot User registered under
/// `bot_key`; the previous token stops working.
pub async fn set_token(
    db: &mut SqliteConnection,
    bot_key: &str,
    user: UserId,
    token_hash: &str,
    now: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO bot_tokens (token_hash, user_id, bot_key, created_at) VALUES (?, ?, ?, ?) \
         ON CONFLICT (bot_key) DO UPDATE SET \
           token_hash = excluded.token_hash, created_at = excluded.created_at",
    )
    .bind(token_hash)
    .bind(user)
    .bind(bot_key)
    .bind(now)
    .execute(db)
    .await?;
    Ok(())
}
