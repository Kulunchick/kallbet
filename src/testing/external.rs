use std::sync::Arc;

use async_trait::async_trait;

use super::ScriptedSource;
use crate::domain::{DataSourceKind, Event, EventId, EventSnapshot, Match, TeamId};
use crate::ports::{MatchSource, SourceError};
use crate::sources::fallback::{
    EventHint, ExternalEvent, ExternalMatch, ExternalSnapshot, ExternalSource, ExternalTeam, Side,
};

/// A fallback Data Source (PandaScore, Liquipedia) for tests: a
/// [`ScriptedSource`] seen through the fallback adapter contract.
///
/// Script it like any `ScriptedSource` (made with `ScriptedSource::of(kind)`),
/// using that source's *own* ids: its Event `e<id>`, Matches `m<id>` and teams
/// `t<id>` become external ids the chain must map by team name and start time.
/// [`with_canonical_ids`](Self::with_canonical_ids) makes the records carry
/// their ids as canonical (HLTV) ids instead, like Liquipedia's `hltv=` field.
/// `find_events` returns every scripted Event as a candidate.
pub struct ExternalFake {
    inner: Arc<ScriptedSource>,
    canonical_ids: bool,
}

impl ExternalFake {
    pub fn new(inner: Arc<ScriptedSource>) -> ExternalFake {
        ExternalFake {
            inner,
            canonical_ids: false,
        }
    }

    pub fn with_canonical_ids(mut self) -> ExternalFake {
        self.canonical_ids = true;
        self
    }

    fn event(&self, event: &Event) -> ExternalEvent {
        ExternalEvent {
            external_id: format!("e{}", event.id),
            canonical: self.canonical_ids.then_some(event.id),
            name: event.name.clone(),
            starts_at: event.starts_at,
            ends_at: event.ends_at,
        }
    }

    fn team(&self, snapshot: &EventSnapshot, id: Option<TeamId>) -> Option<ExternalTeam> {
        let team = snapshot.team(id?)?;
        Some(ExternalTeam {
            external_id: format!("t{}", team.id),
            name: team.name.clone(),
            canonical: self.canonical_ids.then_some(team.id),
        })
    }

    fn game(&self, snapshot: &EventSnapshot, m: &Match) -> ExternalMatch {
        let winner = m.winner.and_then(|w| {
            if Some(w) == m.team_a {
                Some(Side::A)
            } else if Some(w) == m.team_b {
                Some(Side::B)
            } else {
                None
            }
        });
        ExternalMatch {
            external_id: format!("m{}", m.id),
            canonical: self.canonical_ids.then_some(m.id),
            stage: m.stage,
            placement: m.placement,
            team_a: self.team(snapshot, m.team_a),
            team_b: self.team(snapshot, m.team_b),
            format: m.format,
            scheduled_start: m.scheduled_start,
            actual_start: m.actual_start,
            status: m.status,
            winner,
            score: m.score,
        }
    }
}

#[async_trait]
impl ExternalSource for ExternalFake {
    fn kind(&self) -> DataSourceKind {
        self.inner.kind()
    }

    async fn list_events(&self) -> Result<Vec<ExternalEvent>, SourceError> {
        let events = self.inner.list_events().await?;
        Ok(events.iter().map(|e| self.event(e)).collect())
    }

    async fn find_events(&self, _hint: &EventHint) -> Result<Vec<ExternalEvent>, SourceError> {
        self.list_events().await
    }

    async fn fetch_event(&self, external_id: &str) -> Result<ExternalSnapshot, SourceError> {
        let id = external_id
            .strip_prefix('e')
            .and_then(|id| id.parse().ok())
            .ok_or(SourceError::NotFound)?;
        let snapshot = self.inner.fetch_event(EventId(id)).await?;
        Ok(ExternalSnapshot {
            event: self.event(&snapshot.event),
            stages: snapshot.stages.clone(),
            matches: snapshot
                .matches
                .iter()
                .map(|m| self.game(&snapshot, m))
                .collect(),
        })
    }
}
