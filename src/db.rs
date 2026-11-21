//! The SQLite database (ADR-0002): connecting and migrating.

use std::str::FromStr;

use sqlx::{Sqlite, SqlitePool, Transaction};

use crate::error::ApiResult;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

/// Opens (creating if needed) the database at `url` and applies migrations.
pub async fn connect(url: &str) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::from_str(url)?
        .create_if_missing(true)
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal);
    let pool = SqlitePoolOptions::new().connect_with(options).await?;
    migrate(&pool).await?;
    Ok(pool)
}

/// A fresh, private, migrated in-memory database, for tests.
///
/// It lives in its single connection, so the pool has exactly one: do not
/// hold a transaction open while awaiting another query on the pool.
pub async fn connect_in_memory() -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::from_str("sqlite::memory:")?.foreign_keys(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .min_connections(1)
        .idle_timeout(None)
        .max_lifetime(None)
        .connect_with(options)
        .await?;
    migrate(&pool).await?;
    Ok(pool)
}

/// Opens a write transaction that takes SQLite's write lock at once
/// (`BEGIN IMMEDIATE`), so a check-then-write sequence inside it is never
/// interleaved with another writer. Read the clock after it is open.
pub async fn begin_write(pool: &SqlitePool) -> Result<Transaction<'static, Sqlite>, sqlx::Error> {
    pool.begin_with("BEGIN IMMEDIATE").await
}

/// Ends a request's [`begin_write`] transaction by the request's result:
/// commits when it succeeded or was refused (a 4xx, so the Lock and Reveal
/// stamps its evaluation stored are kept), rolls back on a server error.
pub async fn finish<T>(tx: Transaction<'static, Sqlite>, result: ApiResult<T>) -> ApiResult<T> {
    if result.as_ref().is_err_and(|e| e.status.is_server_error()) {
        return result; // dropping `tx` rolls it back
    }
    tx.commit().await?;
    result
}

async fn migrate(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    MIGRATOR
        .run(pool)
        .await
        .map_err(|e| sqlx::Error::Migrate(Box::new(e)))
}
