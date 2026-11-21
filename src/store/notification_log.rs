//! The notification log: which notifications were already handled, so the
//! poller (which runs every 20 seconds) never sends one twice.

use chrono::{DateTime, Utc};
use sqlx::SqlitePool;

use crate::domain::{LeagueId, UserId};

pub async fn contains(db: &SqlitePool, key: &str) -> Result<bool, sqlx::Error> {
    let found: Option<(i64,)> = sqlx::query_as("SELECT 1 FROM notification_log WHERE key = ?")
        .bind(key)
        .fetch_optional(db)
        .await?;
    Ok(found.is_some())
}

/// Records `key` as handled. Recording twice is harmless.
pub async fn record(
    db: &SqlitePool,
    key: &str,
    league: LeagueId,
    user: UserId,
    now: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT OR IGNORE INTO notification_log (key, league_id, user_id, sent_at) \
         VALUES (?, ?, ?, ?)",
    )
    .bind(key)
    .bind(league)
    .bind(user)
    .bind(now)
    .execute(db)
    .await?;
    Ok(())
}
