//! FallbackSource: composes the Data Sources HLTV -> PandaScore ->
//! Liquipedia behind one `MatchSource`, with a circuit breaker per source
//! (ADR-0001). Ticket #9; the full rules are in `docs/agents/codebase.md`
//! ("Fallback chain").
//!
//! - The **primary** source (HLTV) speaks canonical ids: its snapshot is
//!   returned as it is. A fresh HLTV snapshot also re-keys Matches and teams
//!   that were known only from a fallback (local ids) and records conflicts
//!   with stored lower-priority results.
//! - **Fallbacks** implement [`ExternalSource`] (records with their own
//!   ids); [`mapping`] links them to canonical ids and merges them over the
//!   stored data, so a fallback never spoils better data.
//! - A source answering `NotConfigured` is skipped and does not count as a
//!   failure. `Unavailable` and `Parse` count; after N in a row its breaker
//!   opens and the source is skipped for M minutes, then tried once more.
//! - `NotFound` from HLTV ends the search (the Event does not exist).

use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use sqlx::SqlitePool;

use crate::config::Config;
use crate::domain::{DataSourceKind, Event, EventId, EventSnapshot};
use crate::ports::{Clock, MatchSource, SourceError};
use crate::store::{events, external_ids};

mod breaker;
mod external;
pub mod mapping;
pub mod names;

use breaker::Breaker;
pub use external::{
    EventHint, ExternalEvent, ExternalMatch, ExternalSnapshot, ExternalSource, ExternalTeam, Side,
};
use mapping::{Context, IdKind, LocalIds, Origin};

/// The chain's tunables (from `Config`: `KALLBET_BREAKER_FAILURES`,
/// `KALLBET_BREAKER_OPEN_MINUTES`, `KALLBET_MATCH_WINDOW_HOURS`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FallbackSettings {
    /// Consecutive failures that open a source's breaker (N).
    pub breaker_failures: u32,
    /// How long an open breaker skips its source (M).
    pub breaker_open: Duration,
    /// Start times of the same Match at two sources differ by at most this.
    pub match_window: Duration,
}

impl Default for FallbackSettings {
    fn default() -> FallbackSettings {
        FallbackSettings {
            breaker_failures: 3,
            breaker_open: Duration::minutes(15),
            match_window: Duration::hours(6),
        }
    }
}

impl FallbackSettings {
    pub fn from_config(config: &Config) -> FallbackSettings {
        FallbackSettings {
            breaker_failures: config.breaker_failures,
            breaker_open: Duration::minutes(config.breaker_open_minutes),
            match_window: Duration::hours(config.match_window_hours),
        }
    }
}

struct Link<S: ?Sized> {
    kind: DataSourceKind,
    source: Arc<S>,
    /// For `fetch_event`.
    breaker: Breaker,
    /// For `list_events` and `find_events`: a working Event list says
    /// nothing about failing Event pages, so it must not reset `breaker`.
    lists: Breaker,
}

pub struct FallbackSource {
    db: SqlitePool,
    clock: Arc<dyn Clock>,
    settings: FallbackSettings,
    primary: Option<Link<dyn MatchSource>>,
    fallbacks: Vec<Link<dyn ExternalSource>>,
    /// Linking and local-id handout run one at a time (the poller and a
    /// League being created may fetch at once).
    mapping: tokio::sync::Mutex<()>,
}

/// Why one source gave no snapshot; what the caller finally sees.
fn combine(errors: Vec<SourceError>) -> SourceError {
    let failures: Vec<String> = errors
        .iter()
        .filter_map(|e| match e {
            SourceError::Unavailable(m) | SourceError::Parse(m) => Some(m.clone()),
            _ => None,
        })
        .collect();
    if !failures.is_empty() {
        SourceError::Unavailable(failures.join("; "))
    } else if errors.contains(&SourceError::NotFound) {
        SourceError::NotFound
    } else {
        SourceError::NotConfigured
    }
}

fn db_error(e: sqlx::Error) -> SourceError {
    SourceError::Unavailable(format!("database: {e}"))
}

impl FallbackSource {
    /// A chain with no sources yet; every call answers `NotConfigured`.
    pub fn new(
        db: SqlitePool,
        clock: Arc<dyn Clock>,
        settings: FallbackSettings,
    ) -> FallbackSource {
        FallbackSource {
            db,
            clock,
            settings,
            primary: None,
            fallbacks: Vec::new(),
            mapping: tokio::sync::Mutex::new(()),
        }
    }

    /// The canonical source (HLTV).
    pub fn with_primary(mut self, source: Arc<dyn MatchSource>) -> FallbackSource {
        self.primary = Some(Link {
            kind: DataSourceKind::Hltv,
            source,
            breaker: self.breaker(),
            lists: self.breaker(),
        });
        self
    }

    /// A fallback; the chain keeps them in `DataSourceKind` priority order.
    pub fn with_fallback(mut self, source: Arc<dyn ExternalSource>) -> FallbackSource {
        self.fallbacks.push(Link {
            kind: source.kind(),
            source,
            breaker: self.breaker(),
            lists: self.breaker(),
        });
        self.fallbacks.sort_by_key(|l| l.kind.priority());
        self
    }

    fn breaker(&self) -> Breaker {
        Breaker::new(self.settings.breaker_failures, self.settings.breaker_open)
    }

    /// Runs one `fetch_event` call through a source's breaker.
    async fn guarded<S: ?Sized, T>(
        link: &Link<S>,
        now: DateTime<Utc>,
        call: impl Future<Output = Result<T, SourceError>>,
    ) -> Result<T, SourceError> {
        Self::through(link.kind, &link.breaker, now, call).await
    }

    /// Runs one `list_events` / `find_events` call through a source's
    /// breaker for lists.
    async fn guarded_list<S: ?Sized, T>(
        link: &Link<S>,
        now: DateTime<Utc>,
        call: impl Future<Output = Result<T, SourceError>>,
    ) -> Result<T, SourceError> {
        Self::through(link.kind, &link.lists, now, call).await
    }

    async fn through<T>(
        kind: DataSourceKind,
        breaker: &Breaker,
        now: DateTime<Utc>,
        call: impl Future<Output = Result<T, SourceError>>,
    ) -> Result<T, SourceError> {
        if !breaker.allows(now) {
            return Err(SourceError::Unavailable(format!(
                "{}: circuit open",
                kind.code()
            )));
        }
        let result = call.await;
        match &result {
            Ok(_) | Err(SourceError::NotFound) => breaker.succeeded(),
            Err(SourceError::Unavailable(_) | SourceError::Parse(_)) => breaker.failed(now),
            Err(SourceError::NotConfigured) => breaker.released(),
        }
        result.map_err(|e| match e {
            SourceError::Unavailable(m) => {
                SourceError::Unavailable(format!("{}: {m}", kind.code()))
            }
            SourceError::Parse(m) => SourceError::Parse(format!("{}: {m}", kind.code())),
            other => other,
        })
    }

    /// The canonical Event an id names: HLTV Stage event ids are aliases of
    /// their Major's id once HLTV has shown it.
    async fn canonical_event(&self, id: EventId) -> Result<EventId, SourceError> {
        let alias = external_ids::canonical(
            &self.db,
            DataSourceKind::Hltv,
            IdKind::Event,
            &id.to_string(),
        )
        .await
        .map_err(db_error)?;
        Ok(alias.map_or(id, EventId))
    }

    /// Bookkeeping after a good HLTV snapshot. Failures are logged: the
    /// snapshot itself is fine.
    async fn after_primary(
        &self,
        asked: EventId,
        snapshot: &EventSnapshot,
    ) -> Result<(), sqlx::Error> {
        let now = self.clock.now();
        let id = snapshot.event.id;
        if asked != id {
            external_ids::record_mappings(
                &self.db,
                DataSourceKind::Hltv,
                &[mapping::NewMapping {
                    kind: IdKind::Event,
                    external_id: asked.to_string(),
                    canonical: id.0,
                    origin: Origin::Source,
                }],
                now,
            )
            .await?;
        }
        let Some(stored) = events::base_view(&mut *self.db.acquire().await?, id).await? else {
            return Ok(());
        };

        let _guard = self.mapping.lock().await;
        let adopt = mapping::adoptions(&stored, snapshot, self.settings.match_window);
        let adopted = !adopt.teams.is_empty() || !adopt.matches.is_empty();
        for (from, to) in adopt.teams {
            external_ids::rekey_team(&self.db, from, to).await?;
        }
        for (from, to) in adopt.matches {
            if !external_ids::rekey_match(&self.db, from, to).await? {
                tracing::warn!("local Match {from} is HLTV Match {to}, which is already stored");
            }
        }
        // Conflicts are looked up by id, so only after adoption: a local
        // Match's fallback result is compared under its HLTV id.
        let stored = if adopted {
            match events::base_view(&mut *self.db.acquire().await?, id).await? {
                Some(view) => view,
                None => return Ok(()),
            }
        } else {
            stored
        };
        let conflicts = mapping::primary_conflicts(&stored, snapshot);
        external_ids::record_conflicts(&self.db, id, &conflicts, now).await?;
        Ok(())
    }

    /// The external id of the Event at `link`, finding and mapping it first
    /// if needed. `NotFound` when the source has nothing that fits (flagged)
    /// or there is nothing to search by.
    async fn external_event(
        &self,
        link: &Link<dyn ExternalSource>,
        id: EventId,
        now: DateTime<Utc>,
    ) -> Result<String, SourceError> {
        if let Some(external) = external_ids::external(&self.db, link.kind, IdKind::Event, id.0)
            .await
            .map_err(db_error)?
        {
            return Ok(external);
        }
        // Never stored (HLTV down when the League was created): only a
        // candidate carrying the HLTV id itself can be it.
        let event = events::find(&self.db, id).await.map_err(db_error)?;
        let hint = EventHint {
            id,
            name: event.as_ref().map(|e| e.name.clone()).unwrap_or_default(),
            starts_at: event.as_ref().and_then(|e| e.starts_at),
            ends_at: event.as_ref().and_then(|e| e.ends_at),
        };
        let candidates = Self::guarded_list(link, now, link.source.find_events(&hint)).await?;
        let Some((external, origin)) = mapping::choose_event(&hint, &candidates) else {
            let flag = mapping::Unmatched {
                kind: IdKind::Event,
                external_id: String::new(),
                description: format!("no Event like \"{}\" found", hint.name),
            };
            external_ids::replace_unmatched(&self.db, id, link.kind, &[flag], now)
                .await
                .map_err(db_error)?;
            return Err(SourceError::NotFound);
        };
        external_ids::record_mappings(
            &self.db,
            link.kind,
            &[mapping::NewMapping {
                kind: IdKind::Event,
                external_id: external.clone(),
                canonical: id.0,
                origin,
            }],
            now,
        )
        .await
        .map_err(db_error)?;
        Ok(external)
    }

    async fn fetch_from(
        &self,
        link: &Link<dyn ExternalSource>,
        id: EventId,
        now: DateTime<Utc>,
    ) -> Result<EventSnapshot, SourceError> {
        let _guard = self.mapping.lock().await;
        let external = self.external_event(link, id, now).await?;
        let ext = Self::guarded(link, now, link.source.fetch_event(&external)).await?;

        let stored = events::base_view(&mut *self.db.acquire().await.map_err(db_error)?, id)
            .await
            .map_err(db_error)?;
        let event = match &stored {
            Some(view) => view.event.clone(),
            None => Event {
                id,
                name: ext.event.name.clone(),
                starts_at: ext.event.starts_at,
                ends_at: ext.event.ends_at,
            },
        };
        let teams = external_ids::all_teams(&self.db).await.map_err(db_error)?;
        let mappings = external_ids::mappings(&self.db, link.kind)
            .await
            .map_err(db_error)?;
        let ctx = Context {
            source: link.kind,
            event: &event,
            stored: stored.as_ref(),
            teams: &teams,
            mappings: &mappings,
            window: self.settings.match_window,
            now,
        };
        let (matches_needed, teams_needed) =
            mapping::merge(&ctx, &ext, &mut LocalIds::counting()).local_ids_taken;
        let first_match = if matches_needed > 0 {
            external_ids::allocate_local_ids(&self.db, IdKind::Match, matches_needed)
                .await
                .map_err(db_error)?
        } else {
            -1
        };
        let first_team = if teams_needed > 0 {
            external_ids::allocate_local_ids(&self.db, IdKind::Team, teams_needed)
                .await
                .map_err(db_error)?
        } else {
            -1
        };
        let merged = mapping::merge(&ctx, &ext, &mut LocalIds::reserved(first_match, first_team));

        external_ids::record_mappings(&self.db, link.kind, &merged.mappings, now)
            .await
            .map_err(db_error)?;
        external_ids::replace_unmatched(&self.db, id, link.kind, &merged.unmatched, now)
            .await
            .map_err(db_error)?;
        external_ids::record_conflicts(&self.db, id, &merged.conflicts, now)
            .await
            .map_err(db_error)?;
        Ok(merged.snapshot)
    }

    /// Canonical id of an Event a fallback lists, if it can be linked: a
    /// mapping, the HLTV id it carries, or a stored Event it matches.
    async fn link_listed(
        &self,
        link: &Link<dyn ExternalSource>,
        listed: &ExternalEvent,
        stored: &[Event],
        now: DateTime<Utc>,
    ) -> Result<Option<Event>, sqlx::Error> {
        let mut id =
            external_ids::canonical(&self.db, link.kind, IdKind::Event, &listed.external_id)
                .await?
                .map(EventId);
        if id.is_none() {
            let found = stored.iter().find(|event| {
                let hint = EventHint {
                    id: event.id,
                    name: event.name.clone(),
                    starts_at: event.starts_at,
                    ends_at: event.ends_at,
                };
                mapping::choose_event(&hint, std::slice::from_ref(listed)).is_some()
            });
            id = listed.canonical.or(found.map(|e| e.id));
            if let Some(id) = id {
                let origin = if listed.canonical.is_some() {
                    Origin::Source
                } else {
                    Origin::Auto
                };
                external_ids::record_mappings(
                    &self.db,
                    link.kind,
                    &[mapping::NewMapping {
                        kind: IdKind::Event,
                        external_id: listed.external_id.clone(),
                        canonical: id.0,
                        origin,
                    }],
                    now,
                )
                .await?;
            }
        }
        Ok(id.map(|id| {
            stored
                .iter()
                .find(|e| e.id == id)
                .cloned()
                .unwrap_or(Event {
                    id,
                    name: listed.name.clone(),
                    starts_at: listed.starts_at,
                    ends_at: listed.ends_at,
                })
        }))
    }
}

#[async_trait]
impl MatchSource for FallbackSource {
    async fn list_events(&self) -> Result<Vec<Event>, SourceError> {
        let now = self.clock.now();
        let mut errors = Vec::new();
        if let Some(link) = &self.primary {
            match Self::guarded_list(link, now, link.source.list_events()).await {
                Ok(events) => return Ok(events),
                Err(e) => errors.push(e),
            }
        }
        for link in &self.fallbacks {
            match Self::guarded_list(link, now, link.source.list_events()).await {
                Ok(listed) => {
                    let stored = external_ids::all_events(&self.db).await.map_err(db_error)?;
                    let mut events = Vec::new();
                    for item in &listed {
                        if let Some(event) = self
                            .link_listed(link, item, &stored, now)
                            .await
                            .map_err(db_error)?
                        {
                            events.push(event);
                        }
                    }
                    return Ok(events);
                }
                Err(e) => errors.push(e),
            }
        }
        Err(combine(errors))
    }

    async fn fetch_event(&self, asked: EventId) -> Result<EventSnapshot, SourceError> {
        let now = self.clock.now();
        let id = self.canonical_event(asked).await?;
        let mut errors = Vec::new();
        if let Some(link) = &self.primary {
            match Self::guarded(link, now, link.source.fetch_event(id)).await {
                Ok(snapshot) => {
                    if let Err(e) = self.after_primary(asked, &snapshot).await {
                        tracing::warn!("bookkeeping after the HLTV snapshot of {id} failed: {e}");
                    }
                    return Ok(snapshot);
                }
                Err(SourceError::NotFound) => return Err(SourceError::NotFound),
                Err(e) => errors.push(e),
            }
        }
        for link in &self.fallbacks {
            match self.fetch_from(link, id, now).await {
                Ok(snapshot) => return Ok(snapshot),
                Err(e) => errors.push(e),
            }
        }
        Err(combine(errors))
    }
}
