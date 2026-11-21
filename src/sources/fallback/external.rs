//! The contract a fallback Data Source adapter (PandaScore #10, Liquipedia
//! #11) implements: raw records carrying the source's own ids, team names and
//! start times. The fallback chain's mapping layer (`super::mapping`) links
//! them to canonical (HLTV) ids and builds the canonical snapshot; an adapter
//! never deals with canonical ids except to pass one on that its data
//! carries (`canonical` fields).

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::domain::{
    DataSourceKind, EventId, MatchFormat, MatchId, MatchPlacement, MatchScore, MatchStatus, Stage,
    StageOrdinal, TeamId,
};
use crate::ports::SourceError;

/// A fallback Data Source adapter. Errors follow `MatchSource`:
/// `NotConfigured` when it lacks configuration (skipped by the chain, not a
/// failure), `Unavailable` for network/HTTP/rate-limit failures, `Parse` for
/// anything it does not understand (never empty data), `NotFound` when it
/// answered and has no such Event.
#[async_trait]
pub trait ExternalSource: Send + Sync {
    /// Which Data Source this is; decides its place in the chain.
    fn kind(&self) -> DataSourceKind;

    /// Current and upcoming Events, as this source lists them.
    async fn list_events(&self) -> Result<Vec<ExternalEvent>, SourceError>;

    /// Candidates for the canonical Event described by `hint` (search by
    /// name and dates however the source can). The chain picks among them
    /// (see `mapping::choose_event`), so returning a few loose candidates is
    /// fine; return an empty list, not `NotFound`, when nothing looks close.
    /// Called only until the Event is mapped; after that the chain calls
    /// [`fetch_event`](Self::fetch_event) with the mapped id directly.
    async fn find_events(&self, hint: &EventHint) -> Result<Vec<ExternalEvent>, SourceError>;

    /// The whole Event (every Stage, for a Major too) by this source's id, as
    /// returned in [`ExternalEvent::external_id`].
    async fn fetch_event(&self, external_id: &str) -> Result<ExternalSnapshot, SourceError>;
}

/// What the chain knows about the canonical Event it wants from a fallback.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventHint {
    pub id: EventId,
    /// The Event's name as HLTV spells it. Empty when the chain has never
    /// stored the Event (HLTV was down when a League was created): then only
    /// a candidate carrying `id` itself (`ExternalEvent::canonical`) is
    /// accepted, so an adapter that can search by HLTV id should.
    pub name: String,
    pub starts_at: Option<DateTime<Utc>>,
    pub ends_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalEvent {
    /// The source's own id (PandaScore serie id, Liquipedia page title, ...).
    pub external_id: String,
    /// The HLTV event id, when the source's data carries it. Linked directly,
    /// bypassing name matching.
    pub canonical: Option<EventId>,
    pub name: String,
    pub starts_at: Option<DateTime<Utc>>,
    pub ends_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalTeam {
    pub external_id: String,
    /// The name as this source spells it; matched after normalisation.
    pub name: String,
    /// The HLTV team id, when the source's data carries it.
    pub canonical: Option<TeamId>,
}

/// Which of a Match's two teams.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    A,
    B,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalMatch {
    pub external_id: String,
    /// The HLTV match id, when the source's data carries it (Liquipedia's
    /// `hltv=` Match template field). Linked directly, bypassing name and
    /// time matching.
    pub canonical: Option<MatchId>,
    /// An ordinal of this snapshot's own `stages`.
    pub stage: StageOrdinal,
    pub placement: MatchPlacement,
    /// `None` while still to be decided.
    pub team_a: Option<ExternalTeam>,
    pub team_b: Option<ExternalTeam>,
    pub format: MatchFormat,
    pub scheduled_start: Option<DateTime<Utc>>,
    pub actual_start: Option<DateTime<Utc>>,
    pub status: MatchStatus,
    pub winner: Option<Side>,
    /// Maps won, in this record's team order.
    pub score: Option<MatchScore>,
}

/// One Event as a fallback source reports it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalSnapshot {
    pub event: ExternalEvent,
    /// The source's Stages, ordinals 1, 2, ... in order. Linked to canonical
    /// Stages by kind and position (the 2nd Swiss Stage is the 2nd Swiss
    /// Stage).
    pub stages: Vec<Stage>,
    pub matches: Vec<ExternalMatch>,
}
