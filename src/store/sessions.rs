//! Persistence of sessions (keyed by the SHA-256 of the cookie token).

use chrono::{DateTime, Utc};
use sqlx::SqlitePool;

use crate::domain::UserId;

pub async fn insert(
    db: &SqlitePool,
    token_hash: &str,
    user_id: UserId,
    now: DateTime<Utc>,
    expires_at: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO sessions (token_hash, user_id, created_at, expires_at) VALUES (?, ?, ?, ?)",
    )
    .bind(token_hash)
    .bind(user_id)
    .bind(now)
    .bind(expires_at)
    .execute(db)
    .await?;
    Ok(())
}

/// The User of an unexpired session.
pub async fn find_user(
    db: &SqlitePool,
    token_hash: &str,
    now: DateTime<Utc>,
) -> Result<Option<UserId>, sqlx::Error> {
    let row: Option<(UserId, DateTime<Utc>)> =
        sqlx::query_as("SELECT user_id, expires_at FROM sessions WHERE token_hash = ?")
            .bind(token_hash)
            .fetch_optional(db)
            .await?;
    Ok(row
        .filter(|(_, expires_at)| *expires_at > now)
        .map(|(user_id, _)| user_id))
}

pub async fn delete(db: &SqlitePool, token_hash: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM sessions WHERE token_hash = ?")
        .bind(token_hash)
        .execute(db)
        .await?;
    Ok(())
}
