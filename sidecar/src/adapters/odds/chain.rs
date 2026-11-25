use super::{
    FetchError, OddsTransport,
    feed::Feed,
    lines::{self, Line},
};
use crate::{AsOf, Audit, Clock, Instant, Match, OddsSource, Result, Tier};
use async_trait::async_trait;
use chrono::Duration;
use serde::de::DeserializeOwned;
use sqlx::SqlitePool;
use std::{collections::BTreeMap, sync::Arc};
use tokio::sync::Mutex;

/// OddsPapi batches (billable requests, cached or not, a 429 too) this audit
/// file may start per UTC month. The free plan has 250.
pub const ODDSPAPI_MONTHLY_BUDGET: i32 = 200;
/// Of those, what Draft and Pick'em lookups may use: the last 50 are kept for
/// the PreLock refresh, where the closing line matters.
pub const ODDSPAPI_DRAFT_BUDGET: i32 = 150;
/// Default of `SIDECAR_ODDS_PRELOCK_MAX_AGE_SECONDS`.
pub const DEFAULT_PRELOCK_MAX_AGE: Duration = Duration::minutes(10);

#[derive(Clone)]
pub struct OddsConfig {
    pub oddspapi_key: Option<String>,
    pub oddspapi_tournaments: BTreeMap<i64, Vec<u64>>,
    pub polymarket_tags: BTreeMap<i64, String>,
    pub kalshi_enabled: bool,
    /// The oldest batch the PreLock pass (T-25..15) reuses; older ones are
    /// fetched again, so the final Prediction sees the closing line. A Draft
    /// keeps each feed's own TTL (six hours for OddsPapi).
    pub prelock_max_age: Duration,
}

impl std::fmt::Debug for OddsConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OddsConfig")
            .field("oddspapi_key", &self.oddspapi_key.as_ref().map(|_| "[REDACTED]"))
            .field("oddspapi_tournaments", &self.oddspapi_tournaments)
            .field("polymarket_tags", &self.polymarket_tags)
            .field("kalshi_enabled", &self.kalshi_enabled)
            .field("prelock_max_age", &self.prelock_max_age)
            .finish()
    }
}

impl Default for OddsConfig {
    fn default() -> Self {
        Self {
            oddspapi_key: None,
            oddspapi_tournaments: BTreeMap::new(),
            polymarket_tags: BTreeMap::new(),
            kalshi_enabled: true,
            prelock_max_age: DEFAULT_PRELOCK_MAX_AGE,
        }
    }
}

impl OddsConfig {
    pub(super) fn from_env() -> Result<Self> {
        let config = Self {
            oddspapi_key: env("ODDSPAPI_API_KEY"),
            oddspapi_tournaments: env_json("SIDECAR_ODDSPAPI_TOURNAMENTS")?,
            polymarket_tags: env_json("SIDECAR_POLYMARKET_TAGS")?,
            kalshi_enabled: env("SIDECAR_KALSHI_ENABLED").map_or(Ok(true), |value| {
                value
                    .parse::<bool>()
                    .map_err(|_| "SIDECAR_KALSHI_ENABLED must be true or false")
            })?,
            prelock_max_age: match env("SIDECAR_ODDS_PRELOCK_MAX_AGE_SECONDS") {
                None => DEFAULT_PRELOCK_MAX_AGE,
                Some(value) => value
                    .parse::<i64>()
                    .ok()
                    .filter(|s| (60..=21_600).contains(s))
                    .map(Duration::seconds)
                    .ok_or("SIDECAR_ODDS_PRELOCK_MAX_AGE_SECONDS must be 60 to 21600")?,
            },
        };
        if config
            .oddspapi_tournaments
            .values()
            .any(|ids| !(1..=5).contains(&ids.len()))
        {
            return Err(
                "SIDECAR_ODDSPAPI_TOURNAMENTS requires 1 to 5 tournament IDs per Event".into(),
            );
        }
        Ok(config)
    }
}

fn env(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .filter(|value| !value.trim().is_empty())
}

fn env_json<T: DeserializeOwned + Default>(name: &str) -> Result<T> {
    match env(name) {
        Some(text) => {
            serde_json::from_str(&text).map_err(|_| format!("invalid {name} JSON").into())
        }
        None => Ok(T::default()),
    }
}

/// Shares persistent Event batches across Matches and Bot Users.
pub struct FallbackOdds {
    config: OddsConfig,
    db: SqlitePool,
    transport: Arc<dyn OddsTransport>,
    gates: [Mutex<Breaker>; 3],
    /// The records of each scope's last fetched batch, by content hash, until
    /// a quote is derived from one of them (cached batches keep the hash only).
    records: std::sync::Mutex<BTreeMap<String, BTreeMap<String, Vec<u8>>>>,
}

impl FallbackOdds {
    pub fn new(audit: &Audit, config: OddsConfig, transport: Arc<dyn OddsTransport>) -> Self {
        Self {
            config,
            transport,
            db: audit.db.clone(),
            gates: Default::default(),
            records: Default::default(),
        }
    }

    async fn cached(
        &self,
        scope: &str,
        now: Instant,
        ttl: Duration,
        live: bool,
    ) -> Result<Option<Vec<Line>>> {
        let raw: Option<Vec<u8>> = sqlx::query_scalar(
            "SELECT lines FROM odds_batches
             WHERE scope = ?1 AND captured_at > ?2
               AND (captured_at < ?3 OR (?4 AND captured_at = ?3))
             ORDER BY captured_at DESC LIMIT 1",
        )
        .bind(scope)
        .bind(now - ttl)
        .bind(now)
        .bind(live)
        .fetch_optional(&self.db)
        .await?;
        Ok(raw.map(|raw| serde_json::from_slice(&raw)).transpose()?)
    }

    /// Keeps the raw record a quote was derived from (story 36), once per
    /// content. A batch read from the cache after a restart has the hash but
    /// not the record; its record was kept when it was first quoted.
    async fn keep_record(&self, scope: &str, line: &Line, observed_at: Instant) -> Result<()> {
        let Some(hash) = &line.raw_hash else {
            return Ok(());
        };
        let record = self
            .records
            .lock()
            .unwrap()
            .get(scope)
            .and_then(|records| records.get(hash).cloned());
        if let Some(record) = record {
            let feed = scope.split(':').next().unwrap_or(scope);
            crate::audit::store_raw(&self.db, &format!("odds:{feed}"), observed_at, &record).await?;
        }
        Ok(())
    }

    async fn reserve_oddspapi_request(&self, now: Instant, tier: Tier) -> Result<bool> {
        let budget = match tier {
            Tier::PreLock => ODDSPAPI_MONTHLY_BUDGET,
            Tier::Draft => ODDSPAPI_DRAFT_BUDGET,
        };
        // Reserve before sending. Even an upstream 429 can count against
        // the free plan, and an interrupted tick must keep the reservation.
        let update = sqlx::query(
            "INSERT INTO oddspapi_months(month, attempts) VALUES (?1, 1)
             ON CONFLICT(month) DO UPDATE SET attempts = attempts + 1
             WHERE attempts < ?2",
        )
        .bind(now.format("%Y-%m").to_string())
        .bind(budget)
        .execute(&self.db)
        .await?;
        Ok(update.rows_affected() != 0)
    }

    async fn lookup(
        &self,
        game: &Match,
        mut cutoff: Instant,
        tier: Tier,
        clock: Option<&dyn Clock>,
    ) -> Result<(Instant, Option<AsOf<f64>>)> {
        let Some(hard_lock) = game
            .scheduled_start
            .map(crate::hard_lock)
        else {
            return Ok((cutoff, None));
        };
        if clock.is_some() {
            cutoff = cutoff.min(hard_lock);
        }
        let tournament_ids = self
            .config
            .oddspapi_tournaments
            .get(&game.event_id)
            .map(|ids| ids.iter().map(u64::to_string).collect::<Vec<_>>().join(","));
        let providers = [
            self.config
                .oddspapi_key
                .as_deref()
                .zip(tournament_ids.as_deref())
                .map(|(key, ids)| {
                    (
                        format!("oddspapi:{}:{ids}", game.event_id),
                        Feed::OddsPapi { ids, key },
                    )
                }),
            self.config
                .polymarket_tags
                .get(&game.event_id)
                .filter(|tag| !tag.trim().is_empty())
                .map(|tag| {
                    (
                        format!("polymarket:{}:{tag}", game.event_id),
                        Feed::Polymarket { tag },
                    )
                }),
            self.config
                .kalshi_enabled
                .then(|| ("kalshi:cs2".into(), Feed::Kalshi)),
        ];
        for (provider, gate) in providers.into_iter().zip(&self.gates) {
            let Some((scope, feed)) = provider else {
                continue;
            };
            let mut breaker = gate.lock().await;
            if let Some(clock) = clock {
                cutoff = cutoff.max(clock.now()).min(hard_lock);
                if cutoff == hard_lock {
                    return Ok((cutoff, None));
                }
            }
            let ttl = match tier {
                Tier::PreLock => feed.ttl().min(self.config.prelock_max_age),
                Tier::Draft => feed.ttl(),
            };
            let cached = self.cached(&scope, cutoff, ttl, clock.is_some()).await?;
            let lines = match (cached, clock) {
                (Some(lines), _) => lines,
                (None, None) => continue,
                (None, Some(clock)) => {
                    if !breaker.allows(cutoff) {
                        continue;
                    }
                    if matches!(feed, Feed::OddsPapi { .. }) {
                        if !self.reserve_oddspapi_request(cutoff, tier).await? {
                            tracing::warn!(tier = tier.name(), "OddsPapi monthly budget reached; trying the next Odds Source");
                            continue;
                        }
                        // OddsPapi rejects concurrent Event batches even when
                        // they use different tournament IDs.
                        breaker.next_request = Some(cutoff + Duration::seconds(1));
                    }
                    let fetched = feed.fetch(self.transport.as_ref(), clock).await;
                    let captured_at = clock.now();
                    cutoff = cutoff.max(clock.now()).min(hard_lock);
                    let lines = match fetched {
                        Ok(lines) => {
                            breaker.succeeded();
                            lines
                        }
                        Err(error) => {
                            let limited =
                                matches!(error.downcast_ref(), Some(FetchError::RateLimited));
                            breaker.failed(cutoff, limited);
                            continue;
                        }
                    };
                    self.records.lock().unwrap().insert(
                        scope.clone(),
                        lines
                            .iter()
                            .filter_map(|l| Some((l.raw_hash.clone()?, l.raw.clone()?)))
                            .collect(),
                    );
                    sqlx::query(
                        "INSERT INTO odds_batches(scope, captured_at, lines) VALUES (?, ?, ?)
                         ON CONFLICT DO NOTHING",
                    )
                    .bind(&scope)
                    .bind(captured_at)
                    .bind(serde_json::to_vec(&lines)?)
                    .execute(&self.db)
                    .await?;
                    lines
                }
            };
            if clock.is_some() && cutoff == hard_lock {
                return Ok((cutoff, None));
            }
            if let Some((quote, line)) = lines::quote_line(&lines, game, None, cutoff)? {
                self.keep_record(&scope, line, quote.observed_at()).await?;
                return Ok((cutoff, Some(quote)));
            }
        }
        Ok((cutoff, None))
    }
}

#[derive(Default)]
struct Breaker {
    failures: u32,
    open_until: Option<Instant>,
    next_request: Option<Instant>,
}

impl Breaker {
    fn allows(&self, now: Instant) -> bool {
        self.open_until.is_none_or(|until| now >= until)
            && self.next_request.is_none_or(|next| now >= next)
    }

    fn succeeded(&mut self) {
        self.failures = 0;
        self.open_until = None;
    }

    fn failed(&mut self, now: Instant, limited: bool) {
        self.failures = self.failures.saturating_add(1);
        if limited || self.failures >= 3 || self.open_until.is_some() {
            self.open_until = Some(now + Duration::minutes(15));
        }
    }
}

#[async_trait]
impl OddsSource for FallbackOdds {
    async fn quote(&self, game: &Match, cutoff: Instant) -> Result<Option<AsOf<f64>>> {
        Ok(self.lookup(game, cutoff, Tier::Draft, None).await?.1)
    }

    async fn live_quote(
        &self,
        game: &Match,
        cutoff: Instant,
        tier: Tier,
        clock: &dyn Clock,
    ) -> Result<(Instant, Option<AsOf<f64>>)> {
        self.lookup(game, cutoff, tier, Some(clock)).await
    }
}
