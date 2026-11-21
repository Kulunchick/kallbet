//! Persistence of Users.

use chrono::{DateTime, Utc};
use sqlx::{FromRow, SqlitePool};

use crate::domain::{Language, TelegramId, User, UserId};

const COLUMNS: &str = "id, telegram_id, nickname, avatar_url, language";

#[derive(FromRow)]
struct UserRow {
    id: UserId,
    telegram_id: TelegramId,
    nickname: String,
    avatar_url: Option<String>,
    language: String,
}

impl From<UserRow> for User {
    fn from(row: UserRow) -> User {
        User {
            id: row.id,
            telegram_id: row.telegram_id,
            nickname: row.nickname,
            avatar_url: row.avatar_url,
            language: Language::from_code(&row.language).unwrap_or_default(),
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
         RETURNING id, telegram_id, nickname, avatar_url, language",
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
pub async fn find_by_nickname(db: &SqlitePool, nickname: &str) -> Result<Option<User>, sqlx::Error> {
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
