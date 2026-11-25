//! Persistence of Users.

use chrono::{DateTime, Utc};
use sqlx::{FromRow, SqlitePool};

use crate::domain::user::NotificationPrefs;
use crate::domain::{Language, TelegramId, User, UserId};

const COLUMNS: &str = "id, telegram_id, nickname, avatar_url, language, notify_reminder, notify_reveal, notify_digest, is_bot";

#[derive(FromRow)]
struct UserRow {
    id: UserId,
    telegram_id: TelegramId,
    nickname: String,
    avatar_url: Option<String>,
    language: String,
    notify_reminder: bool,
    notify_reveal: bool,
    notify_digest: bool,
    is_bot: bool,
}

impl From<UserRow> for User {
    fn from(row: UserRow) -> User {
        User {
            id: row.id,
            telegram_id: row.telegram_id,
            nickname: row.nickname,
            avatar_url: row.avatar_url,
            language: Language::from_code(&row.language).unwrap_or_default(),
            notifications: NotificationPrefs {
                reminder: row.notify_reminder,
                reveal: row.notify_reveal,
                digest: row.notify_digest,
            },
            is_bot: row.is_bot,
        }
    }
}

pub struct NewUser {
    pub telegram_id: TelegramId,
    pub nickname: String,
    pub avatar_url: Option<String>,
    pub language: Language,
}

/// Inserts a User. A taken Nickname (case-insensitive) or Telegram id
/// surfaces as a unique-violation database error.
pub async fn create(
    db: &SqlitePool,
    new_user: &NewUser,
    now: DateTime<Utc>,
) -> Result<User, sqlx::Error> {
    let row: UserRow = sqlx::query_as(
        "INSERT INTO users (telegram_id, nickname, avatar_url, language, created_at) \
         VALUES (?, ?, ?, ?, ?) \
         RETURNING id, telegram_id, nickname, avatar_url, language, \
                   notify_reminder, notify_reveal, notify_digest, is_bot",
    )
    .bind(new_user.telegram_id)
    .bind(&new_user.nickname)
    .bind(&new_user.avatar_url)
    .bind(new_user.language.code())
    .bind(now)
    .fetch_one(db)
    .await?;
    Ok(row.into())
}

pub async fn find_by_id(db: &SqlitePool, id: UserId) -> Result<Option<User>, sqlx::Error> {
    find_where(db, "id = ?", id.0).await
}

pub async fn find_by_telegram_id(
    db: &SqlitePool,
    telegram_id: TelegramId,
) -> Result<Option<User>, sqlx::Error> {
    find_where(db, "telegram_id = ?", telegram_id.0).await
}

/// Case-insensitive, like Nickname uniqueness.
pub async fn find_by_nickname(
    db: &SqlitePool,
    nickname: &str,
) -> Result<Option<User>, sqlx::Error> {
    let row: Option<UserRow> = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "SELECT {COLUMNS} FROM users WHERE nickname = ?"
    )))
    .bind(nickname)
    .fetch_optional(db)
    .await?;
    Ok(row.map(User::from))
}

async fn find_where(
    db: &SqlitePool,
    condition: &'static str,
    value: i64,
) -> Result<Option<User>, sqlx::Error> {
    let row: Option<UserRow> = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "SELECT {COLUMNS} FROM users WHERE {condition}"
    )))
    .bind(value)
    .fetch_optional(db)
    .await?;
    Ok(row.map(User::from))
}

/// A Telegram id for a User made by the dev login bypass. Negative, so it
/// can never collide with a real Telegram account.
pub async fn next_dev_telegram_id(db: &SqlitePool) -> Result<TelegramId, sqlx::Error> {
    let (lowest,): (Option<i64>,) = sqlx::query_as("SELECT MIN(telegram_id) FROM users")
        .fetch_one(db)
        .await?;
    Ok(TelegramId(lowest.unwrap_or(0).min(0) - 1))
}

/// Whether a database error is a unique-constraint violation (a taken
/// Nickname or Telegram id).
pub fn is_unique_violation(error: &sqlx::Error) -> bool {
    error
        .as_database_error()
        .is_some_and(|e| e.is_unique_violation())
}

/// Changes the given fields in one statement; `None` leaves a field alone.
/// A taken Nickname surfaces as a unique violation.
pub async fn update_profile(
    db: &SqlitePool,
    id: UserId,
    nickname: Option<&str>,
    language: Option<Language>,
) -> Result<User, sqlx::Error> {
    let row: UserRow = sqlx::query_as(
        "UPDATE users SET nickname = COALESCE(?, nickname), language = COALESCE(?, language) \
         WHERE id = ? \
         RETURNING id, telegram_id, nickname, avatar_url, language, \
                   notify_reminder, notify_reveal, notify_digest, is_bot",
    )
    .bind(nickname)
    .bind(language.map(Language::code))
    .bind(id)
    .fetch_one(db)
    .await?;
    Ok(row.into())
}

/// Refreshes the avatar from Telegram (kept as is when Telegram sends none).
pub async fn update_avatar(
    db: &SqlitePool,
    id: UserId,
    avatar_url: Option<&str>,
) -> Result<User, sqlx::Error> {
    let row: UserRow = sqlx::query_as(
        "UPDATE users SET avatar_url = COALESCE(?, avatar_url) WHERE id = ? \
         RETURNING id, telegram_id, nickname, avatar_url, language, \
                   notify_reminder, notify_reveal, notify_digest, is_bot",
    )
    .bind(avatar_url)
    .bind(id)
    .fetch_one(db)
    .await?;
    Ok(row.into())
}

/// A partial change of the notification switches: `None` keeps a switch.
#[derive(Debug, Clone, Copy, Default, serde::Deserialize)]
pub struct NotificationPrefsChange {
    pub reminder: Option<bool>,
    pub reveal: Option<bool>,
    pub digest: Option<bool>,
}

pub async fn update_notification_prefs(
    db: &SqlitePool,
    id: UserId,
    change: &NotificationPrefsChange,
) -> Result<User, sqlx::Error> {
    let row: UserRow = sqlx::query_as(
        "UPDATE users SET notify_reminder = COALESCE(?, notify_reminder), \
                          notify_reveal = COALESCE(?, notify_reveal), \
                          notify_digest = COALESCE(?, notify_digest) \
         WHERE id = ? \
         RETURNING id, telegram_id, nickname, avatar_url, language, \
                   notify_reminder, notify_reveal, notify_digest, is_bot",
    )
    .bind(change.reminder)
    .bind(change.reveal)
    .bind(change.digest)
    .bind(id)
    .fetch_one(db)
    .await?;
    Ok(row.into())
}
