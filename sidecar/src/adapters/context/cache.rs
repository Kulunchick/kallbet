//! SQLite-backed caching for Liquipedia revisions and CS2 RSS feeds using `raw_feed_snapshots`.

use crate::*;
use chrono::Duration;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedLiquipediaRevision {
    pub query_cutoff: Instant,
    pub fetched_at: Instant,
    pub rev_timestamp: Instant,
    pub body: String,
}

fn liquipedia_source_key(page_title: &str) -> String {
    format!("liquipedia:rev:{page_title}")
}

fn rss_source_key(feed_url: &str) -> String {
    format!("rss:{feed_url}")
}

/// Looks up a cached Liquipedia MediaWiki revision strictly predating `cutoff`.
///
/// A cached revision `R` (with `rev_timestamp < cutoff`) is reusable when either:
/// - `cutoff <= query_cutoff`: MediaWiki proved no revision existed in `(rev_timestamp, query_cutoff)`,
///   so `R` is the exact latest revision strictly before `cutoff`; or
/// - live TTL coverage: `cutoff` matches `now` (within 1s) and `now - fetched_at < ttl`.
pub async fn get_liquipedia_revision(
    db: &SqlitePool,
    page_title: &str,
    cutoff: Instant,
    now: Instant,
    ttl: Duration,
) -> Result<Option<String>> {
    let source = liquipedia_source_key(page_title);
    let rows = sqlx::query_scalar::<_, Vec<u8>>(
        "SELECT payload FROM raw_feed_snapshots WHERE source = ?",
    )
    .bind(&source)
    .fetch_all(db)
    .await?;

    let is_live_cutoff = (now - cutoff).abs() <= Duration::seconds(1);
    let mut best: Option<CachedLiquipediaRevision> = None;

    for blob in rows {
        let Ok(entry) = serde_json::from_slice::<CachedLiquipediaRevision>(&blob) else {
            continue;
        };
        // Never reuse a revision at or after cutoff.
        if entry.rev_timestamp >= cutoff {
            continue;
        }
        let historical_hit = cutoff <= entry.query_cutoff;
        let live_ttl_hit = is_live_cutoff
            && now >= entry.fetched_at
            && (now - entry.fetched_at) < ttl;
        if !(historical_hit || live_ttl_hit) {
            continue;
        }
        let replace = best
            .as_ref()
            .is_none_or(|current| entry.rev_timestamp > current.rev_timestamp);
        if replace {
            best = Some(entry);
        }
    }

    Ok(best.map(|entry| entry.body))
}

pub async fn put_liquipedia_revision(
    db: &SqlitePool,
    page_title: &str,
    entry: &CachedLiquipediaRevision,
) -> Result<()> {
    let source = liquipedia_source_key(page_title);
    let payload = serde_json::to_vec(entry)?;
    let content_hash = crate::digest(&payload);
    sqlx::query(
        "INSERT INTO raw_feed_snapshots(source, observed_at, content_hash, payload)
         VALUES(?, ?, ?, ?) ON CONFLICT DO NOTHING",
    )
    .bind(source)
    .bind(entry.rev_timestamp)
    .bind(content_hash)
    .bind(payload)
    .execute(db)
    .await?;
    Ok(())
}

/// Looks up a fresh cached RSS XML payload within `ttl` relative to `now`.
pub async fn get_rss_feed(
    db: &SqlitePool,
    feed_url: &str,
    now: Instant,
    ttl: Duration,
) -> Result<Option<String>> {
    let source = rss_source_key(feed_url);
    let rows = sqlx::query_as::<_, (Instant, Vec<u8>)>(
        "SELECT observed_at, payload FROM raw_feed_snapshots WHERE source = ?",
    )
    .bind(&source)
    .fetch_all(db)
    .await?;

    let latest = rows
        .into_iter()
        .filter(|(fetched_at, _)| *fetched_at <= now && (now - *fetched_at) < ttl)
        .max_by_key(|(fetched_at, _)| *fetched_at);

    match latest {
        Some((_, bytes)) => Ok(String::from_utf8(bytes).ok()),
        None => Ok(None),
    }
}

pub async fn put_rss_feed(
    db: &SqlitePool,
    feed_url: &str,
    fetched_at: Instant,
    xml: &str,
) -> Result<()> {
    let source = rss_source_key(feed_url);
    let bytes = xml.as_bytes();
    let content_hash = crate::digest(bytes);
    sqlx::query(
        "INSERT INTO raw_feed_snapshots(source, observed_at, content_hash, payload)
         VALUES(?, ?, ?, ?) ON CONFLICT DO NOTHING",
    )
    .bind(source)
    .bind(fetched_at)
    .bind(content_hash)
    .bind(bytes)
    .execute(db)
    .await?;
    Ok(())
}
