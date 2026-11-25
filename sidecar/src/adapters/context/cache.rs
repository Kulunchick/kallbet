//! SQLite-backed caching for Liquipedia revisions and CS2 RSS feeds using `raw_feed_snapshots`.
//!
//! Raw data follows the audit's rule (`audit::store_raw`): a revision's
//! MediaWiki response and a feed's XML are kept once per content
//! (`liquipedia:revision:<page>`, `rss:xml:<url>`); each fetch adds only a
//! small marker pointing at it by hash (`liquipedia:rev:<page>`, `rss:<url>`),
//! which is what the cache looks up.

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

fn liquipedia_body_key(page_title: &str) -> String {
    format!("liquipedia:revision:{page_title}")
}

fn rss_xml_key(feed_url: &str) -> String {
    format!("rss:xml:{feed_url}")
}

/// A fetch of a revision, as kept: the response itself is stored once by hash.
#[derive(Serialize, Deserialize)]
struct RevisionFetch {
    query_cutoff: Instant,
    fetched_at: Instant,
    rev_timestamp: Instant,
    body_hash: String,
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
    let mut best: Option<RevisionFetch> = None;

    for blob in rows {
        let Ok(entry) = serde_json::from_slice::<RevisionFetch>(&blob) else {
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

    let Some(entry) = best else {
        return Ok(None);
    };
    let body = crate::audit::load_raw(db, &liquipedia_body_key(page_title), &entry.body_hash).await?;
    Ok(body.and_then(|b| String::from_utf8(b).ok()))
}

pub async fn put_liquipedia_revision(
    db: &SqlitePool,
    page_title: &str,
    entry: &CachedLiquipediaRevision,
) -> Result<()> {
    let body_hash = crate::audit::store_raw(
        db,
        &liquipedia_body_key(page_title),
        entry.rev_timestamp,
        entry.body.as_bytes(),
    )
    .await?;
    let source = liquipedia_source_key(page_title);
    let payload = serde_json::to_vec(&RevisionFetch {
        query_cutoff: entry.query_cutoff,
        fetched_at: entry.fetched_at,
        rev_timestamp: entry.rev_timestamp,
        body_hash,
    })?;
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

    let Some((_, marker)) = latest else {
        return Ok(None);
    };
    let Ok(hash) = String::from_utf8(marker) else {
        return Ok(None);
    };
    let xml = crate::audit::load_raw(db, &rss_xml_key(feed_url), &hash).await?;
    Ok(xml.and_then(|b| String::from_utf8(b).ok()))
}

pub async fn put_rss_feed(
    db: &SqlitePool,
    feed_url: &str,
    fetched_at: Instant,
    xml: &str,
) -> Result<()> {
    let xml_hash =
        crate::audit::store_raw(db, &rss_xml_key(feed_url), fetched_at, xml.as_bytes()).await?;
    let source = rss_source_key(feed_url);
    let bytes = xml_hash.as_bytes();
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
