use super::{
    OddsTransport,
    feed::Feed,
    lines::{self, Line},
};
use crate::{AsOf, Audit, Clock, Instant, Match, OddsSource, Result};
use async_trait::async_trait;
use chrono::Duration;
use serde::de::DeserializeOwned;
use sqlx::SqlitePool;
use std::{collections::BTreeMap, sync::Arc};
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct OddsConfig {
    pub oddspapi_key: Option<String>,
    pub oddspapi_tournaments: BTreeMap<i64, Vec<u64>>,
    pub polymarket_tags: BTreeMap<i64, String>,
    pub kalshi_enabled: bool,
}

impl Default for OddsConfig {
    fn default() -> Self {
        Self {
            oddspapi_key: None,
            oddspapi_tournaments: BTreeMap::new(),
            polymarket_tags: BTreeMap::new(),
            kalshi_enabled: true,
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
    gates: [Mutex<()>; 3],
}

impl FallbackOdds {
    pub fn new(audit: &Audit, config: OddsConfig, transport: Arc<dyn OddsTransport>) -> Self {
        Self {
            config,
            transport,
            db: audit.db.clone(),
            gates: Default::default(),
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

    async fn lookup(
        &self,
        game: &Match,
        mut cutoff: Instant,
        clock: Option<&dyn Clock>,
    ) -> Result<(Instant, Option<AsOf<f64>>)> {
        let Some(hard_lock) = game
            .scheduled_start
            .map(|start| start - Duration::minutes(15))
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
            let _guard = gate.lock().await;
            if let Some(clock) = clock {
                cutoff = cutoff.max(clock.now()).min(hard_lock);
                if cutoff == hard_lock {
                    return Ok((cutoff, None));
                }
            }
            let cached = self
                .cached(&scope, cutoff, feed.ttl(), clock.is_some())
                .await?;
            let lines = match (cached, clock) {
                (Some(lines), _) => lines,
                (None, None) => continue,
                (None, Some(clock)) => {
                    let fetched = feed.fetch(self.transport.as_ref(), clock).await;
                    let captured_at = clock.now();
                    cutoff = cutoff.max(clock.now()).min(hard_lock);
                    let Ok(lines) = fetched else { continue };
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
            if let Some(quote) = lines::quote(&lines, game, None, cutoff)? {
                return Ok((cutoff, Some(quote)));
            }
        }
        Ok((cutoff, None))
    }
}

#[async_trait]
impl OddsSource for FallbackOdds {
    async fn quote(&self, game: &Match, cutoff: Instant) -> Result<Option<AsOf<f64>>> {
        Ok(self.lookup(game, cutoff, None).await?.1)
    }

    async fn live_quote(
        &self,
        game: &Match,
        cutoff: Instant,
        clock: &dyn Clock,
    ) -> Result<(Instant, Option<AsOf<f64>>)> {
        self.lookup(game, cutoff, Some(clock)).await
    }
}
