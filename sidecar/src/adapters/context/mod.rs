//! Point-in-time Context Enrichers (ticket #26):
//! - Liquipedia historical-revision rosters, coaches and stand-ins (rate-limited, gzip, SQLite-cached, CC-BY-SA)
//! - VRS roster-diff comparing active pre-cutoff Event lineups against the latest pre-cutoff VRS snapshot
//! - Curated CS2 news RSS headlines strictly predating `cutoff`

mod cache;
pub mod limiter;
pub mod liquipedia;
pub mod names;
pub mod rss;
pub mod vrs_diff;
mod wikitext;

pub use limiter::{Pause, TokioPause};

use crate::*;
use async_trait::async_trait;
use cache::CachedLiquipediaRevision;
use chrono::Duration;
use limiter::{MAX_WAIT, RateLimiter};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::time::Duration as StdDuration;
use tokio::sync::RwLock;

fn default_liquipedia_api_url() -> String {
    liquipedia::DEFAULT_API_URL.into()
}

fn default_user_agent() -> String {
    liquipedia::USER_AGENT.into()
}

fn default_liquipedia_cache_ttl_secs() -> i64 {
    900
}

fn default_rss_cache_ttl_secs() -> i64 {
    600
}

fn default_news_lookback_hours() -> i64 {
    14 * 24
}

fn default_max_news_items() -> usize {
    10
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RssFeedConfig {
    pub url: String,
}

impl RssFeedConfig {
    pub fn new(url: impl Into<String>) -> Self {
        Self { url: url.into() }
    }
}

/// Declarative configuration for point-in-time context enrichment in `SIDECAR_CONFIG`.
///
/// Defaults to empty `liquipedia_pages` and `rss_feeds` so minimal or offline configs
/// make no external network calls unless configured.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ContextConfig {
    #[serde(default)]
    pub liquipedia_pages: BTreeMap<i64, String>,
    #[serde(default = "default_liquipedia_api_url")]
    pub liquipedia_api_url: String,
    #[serde(default = "default_user_agent")]
    pub user_agent: String,
    #[serde(default = "default_liquipedia_cache_ttl_secs")]
    pub liquipedia_cache_ttl_secs: i64,
    #[serde(default)]
    pub rss_feeds: Vec<RssFeedConfig>,
    #[serde(default = "default_rss_cache_ttl_secs")]
    pub rss_cache_ttl_secs: i64,
    #[serde(default = "default_news_lookback_hours")]
    pub news_lookback_hours: i64,
    #[serde(default = "default_max_news_items")]
    pub max_news_items: usize,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            liquipedia_pages: BTreeMap::new(),
            liquipedia_api_url: default_liquipedia_api_url(),
            user_agent: default_user_agent(),
            liquipedia_cache_ttl_secs: default_liquipedia_cache_ttl_secs(),
            rss_feeds: Vec::new(),
            rss_cache_ttl_secs: default_rss_cache_ttl_secs(),
            news_lookback_hours: default_news_lookback_hours(),
            max_news_items: default_max_news_items(),
        }
    }
}

impl ContextConfig {
    /// Recommended live CS2 news RSS feeds verified directly reachable (HTTP 200 XML).
    pub fn curated_rss_feeds() -> Vec<RssFeedConfig> {
        vec![
            RssFeedConfig::new("https://www.hltv.org/rss/news"),
            RssFeedConfig::new("https://www.dust2.us/rss"),
        ]
    }
}

/// Production factory wired by `Sidecar::from_config`.
pub fn build(
    config: &ContextConfig,
    audit: &Audit,
    clock: Arc<dyn Clock>,
) -> Arc<dyn ContextEnricher> {
    if config.liquipedia_pages.is_empty() && config.rss_feeds.is_empty() {
        return Arc::new(super::baseline::Noop);
    }
    match CompositeContextEnricher::new(config, audit.db.clone(), clock, Arc::new(TokioPause)) {
        Ok(enricher) => Arc::new(enricher),
        Err(err) => {
            eprintln!("context enricher init failed (degrading to noop): {err}");
            Arc::new(super::baseline::Noop)
        }
    }
}

struct CachedVrsState {
    row_count: i64,
    index: vrs_diff::VrsRosterIndex,
}

/// Composite point-in-time enricher combining Liquipedia rosters, VRS roster diffs,
/// and curated CS2 news RSS feeds behind SQLite caching, rate limiting, and graceful degradation.
pub struct CompositeContextEnricher {
    config: ContextConfig,
    db: SqlitePool,
    clock: Arc<dyn Clock>,
    pause: Arc<dyn Pause>,
    client: reqwest::Client,
    limiter: Mutex<RateLimiter>,
    vrs_cache: RwLock<Option<CachedVrsState>>,
}

impl CompositeContextEnricher {
    pub fn new(
        config: &ContextConfig,
        db: SqlitePool,
        clock: Arc<dyn Clock>,
        pause: Arc<dyn Pause>,
    ) -> Result<Self> {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let client = reqwest::Client::builder()
            .user_agent(&config.user_agent)
            .gzip(true)
            .timeout(StdDuration::from_secs(30))
            .build()?;
        Ok(Self {
            config: config.clone(),
            db,
            clock,
            pause,
            client,
            limiter: Mutex::new(RateLimiter::new()),
            vrs_cache: RwLock::new(None),
        })
    }

    pub async fn from_audit(
        config: &ContextConfig,
        audit: &Audit,
        clock: Arc<dyn Clock>,
        pause: Arc<dyn Pause>,
    ) -> Result<Self> {
        Self::new(config, audit.db.clone(), clock, pause)
    }

    async fn load_vrs_index(&self) -> Result<vrs_diff::VrsRosterIndex> {
        let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM vrs_snapshots")
            .fetch_one(&self.db)
            .await?;
        if let Some(cached) = self.vrs_cache.read().await.as_ref()
            && cached.row_count == count
        {
            return Ok(cached.index.clone());
        }
        let index = vrs_diff::VrsRosterIndex::load_from_db(&self.db).await?;
        *self.vrs_cache.write().await = Some(CachedVrsState {
            row_count: count,
            index: index.clone(),
        });
        Ok(index)
    }

    async fn fetch_liquipedia_rosters(
        &self,
        page_title: &str,
        cutoff: Instant,
    ) -> Result<Option<AsOf<EventRosters>>> {
        let now = self.clock.now();
        let ttl = Duration::seconds(self.config.liquipedia_cache_ttl_secs.max(0));
        if let Some(cached_body) =
            cache::get_liquipedia_revision(&self.db, page_title, cutoff, now, ttl).await?
        {
            return liquipedia::parse_revision_rosters(&cached_body, cutoff);
        }

        let wait = self
            .limiter
            .lock()
            .unwrap()
            .reserve(self.clock.now(), MAX_WAIT)
            .ok_or("Liquipedia rate limit: next slot exceeds max wait")?;
        if wait > Duration::zero() {
            self.pause.pause(wait).await;
        }

        let rvstart = rvstart_strictly_before(cutoff);
        let url = reqwest::Url::parse_with_params(
            &self.config.liquipedia_api_url,
            &[
                ("action", "query"),
                ("prop", "revisions"),
                ("rvprop", "ids|timestamp|content"),
                ("rvslots", "main"),
                ("rvlimit", "2"),
                ("rvdir", "older"),
                ("rvstart", rvstart.as_str()),
                ("formatversion", "2"),
                ("format", "json"),
                ("titles", page_title),
            ],
        )?;

        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(reqwest::Error::without_url)?;
        let status = response.status();
        if status.as_u16() == 429 {
            let retry = response
                .headers()
                .get(reqwest::header::RETRY_AFTER)
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<i64>().ok())
                .map(Duration::seconds)
                .unwrap_or_else(|| Duration::minutes(2));
            self.limiter
                .lock()
                .unwrap()
                .back_off(self.clock.now(), retry);
            return Err("Liquipedia rate limited (HTTP 429)".into());
        }
        if status.as_u16() == 403 {
            self.limiter
                .lock()
                .unwrap()
                .back_off(self.clock.now(), Duration::minutes(10));
            return Err("Liquipedia refused request (HTTP 403)".into());
        }
        if !status.is_success() {
            return Err(format!("Liquipedia HTTP {status}").into());
        }
        let body = response
            .text()
            .await
            .map_err(reqwest::Error::without_url)?;

        let parsed = liquipedia::parse_revision_rosters(&body, cutoff)?;
        if let Some(ref as_of_rosters) = parsed {
            let entry = CachedLiquipediaRevision {
                query_cutoff: cutoff,
                fetched_at: self.clock.now(),
                rev_timestamp: as_of_rosters.observed_at(),
                body,
            };
            let _ = cache::put_liquipedia_revision(&self.db, page_title, &entry).await;
        }
        Ok(parsed)
    }

    async fn fetch_rss_items(&self, feed_url: &str, cutoff: Instant) -> Result<Vec<AsOf<NewsItem>>> {
        let now = self.clock.now();
        let ttl = Duration::seconds(self.config.rss_cache_ttl_secs.max(0));
        if let Some(cached_xml) = cache::get_rss_feed(&self.db, feed_url, now, ttl).await? {
            return rss::parse_rss_items(&cached_xml, feed_url, cutoff);
        }

        let response = self
            .client
            .get(feed_url)
            .send()
            .await
            .map_err(reqwest::Error::without_url)?;
        if !response.status().is_success() {
            return Err(format!("RSS HTTP {}", response.status()).into());
        }
        let xml = response
            .text()
            .await
            .map_err(reqwest::Error::without_url)?;
        let items = rss::parse_rss_items(&xml, feed_url, cutoff)?;
        let _ = cache::put_rss_feed(&self.db, feed_url, self.clock.now(), &xml).await;
        Ok(items)
    }
}

/// Computes MediaWiki `rvstart` (1-second resolution) strictly earlier than `cutoff`.
fn rvstart_strictly_before(cutoff: Instant) -> String {
    let target = if cutoff.timestamp_subsec_nanos() == 0 {
        cutoff - Duration::seconds(1)
    } else {
        cutoff
    };
    target.format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

#[async_trait]
impl ContextEnricher for CompositeContextEnricher {
    async fn facts(&self, game: &Match, cutoff: Instant) -> Result<Vec<AsOf<serde_json::Value>>> {
        let mut out: Vec<AsOf<serde_json::Value>> = Vec::new();
        let mut aliases_a: Vec<String> = Vec::new();
        let mut aliases_b: Vec<String> = Vec::new();

        // 1. Liquipedia historical-revision rosters + 2. VRS roster diff.
        if let Some(page_title) = self.config.liquipedia_pages.get(&game.event_id) {
            match self.fetch_liquipedia_rosters(page_title, cutoff).await {
                Ok(Some(event_as_of)) => {
                    let event = event_as_of.at(cutoff)?;
                    let vrs_index = match self.load_vrs_index().await {
                        Ok(idx) => Some(idx),
                        Err(err) => {
                            eprintln!("vrs roster index load failed (degrading gracefully): {err}");
                            None
                        }
                    };
                    for (side, team_name, aliases_slot) in [
                        ("team_a", &game.team_a.name, &mut aliases_a),
                        ("team_b", &game.team_b.name, &mut aliases_b),
                    ] {
                        if let Some(roster) = liquipedia::find_team_roster(event, team_name) {
                            aliases_slot.extend(roster.aliases.iter().cloned());
                            let roster_fact = liquipedia::build_roster_fact(event, roster, side);
                            let value = ContextFact::LiquipediaRoster(roster_fact).to_value()?;
                            out.push(AsOf::new(value, event_as_of.observed_at(), cutoff)?);

                            if let Some(ref vrs) = vrs_index {
                                match vrs.diff_team(&event_as_of, roster, side, cutoff) {
                                    Ok(Some(diff_as_of)) => {
                                        let ts = diff_as_of.observed_at();
                                        let diff = diff_as_of.at(cutoff)?.clone();
                                        let value = ContextFact::VrsRosterDiff(diff).to_value()?;
                                        out.push(AsOf::new(value, ts, cutoff)?);
                                    }
                                    Ok(None) => {}
                                    Err(err) if err.is::<TemporalLeakError>() => return Err(err),
                                    Err(err) => {
                                        eprintln!("vrs roster diff failed (degrading gracefully): {err}");
                                    }
                                }
                            }
                        }
                    }
                }
                Ok(None) => {}
                Err(err) if err.is::<TemporalLeakError>() => return Err(err),
                Err(err) => {
                    eprintln!("liquipedia enricher failed (degrading gracefully): {err}");
                }
            }
        }

        // 3. Curated CS2 news RSS feeds (strictly predating cutoff, team-relevant).
        if !self.config.rss_feeds.is_empty() {
            let mut all_items = Vec::new();
            for feed in &self.config.rss_feeds {
                match self.fetch_rss_items(&feed.url, cutoff).await {
                    Ok(items) => all_items.extend(items),
                    Err(err) if err.is::<TemporalLeakError>() => return Err(err),
                    Err(err) => {
                        eprintln!(
                            "rss feed enricher failed for {} (degrading gracefully): {err}",
                            feed.url
                        );
                    }
                }
            }
            let lookback = Duration::hours(self.config.news_lookback_hours.max(1));
            let matched = rss::filter_match_news(
                &all_items,
                &game.team_a.name,
                &aliases_a,
                &game.team_b.name,
                &aliases_b,
                cutoff,
                lookback,
                self.config.max_news_items,
            )?;
            for item_as_of in matched {
                let ts = item_as_of.observed_at();
                let fact = item_as_of.at(cutoff)?.clone();
                let value = ContextFact::NewsItem(fact).to_value()?;
                out.push(AsOf::new(value, ts, cutoff)?);
            }
        }

        Ok(out)
    }
}
