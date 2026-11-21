//! Effective Match view (pure): base Match data merged with one League's
//! Overrides. All League logic (Lock/Reveal, scoring, Pick'em, notifications,
//! archive) reads this view, never raw Data Source data.
//!
//! #5 introduced it as the identity view; #13 adds the League's Overrides as
//! a parameter of [`effective_view`] and applies them here. Callers get it
//! through `crate::league_matches::effective_view`, which loads the inputs.

use chrono::{DateTime, Utc};

use super::{DataSourceKind, Event, Match, MatchId, Stage, StageOrdinal, Team, TeamId};

/// A Match and where its data came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcedMatch {
    pub data: Match,
    /// The Data Source of the last successful poll.
    pub source: DataSourceKind,
    /// When KallBet last fetched it successfully: the data freshness.
    pub updated_at: DateTime<Utc>,
}

/// One Event's tournament data as one League sees it (or, before
/// [`effective_view`], as the Data Sources reported it).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventView {
    pub event: Event,
    /// Ordered by ordinal.
    pub stages: Vec<Stage>,
    /// Every team referenced by `matches`.
    pub teams: Vec<Team>,
    /// Ordered by id.
    pub matches: Vec<SourcedMatch>,
}

impl EventView {
    pub fn match_by_id(&self, id: MatchId) -> Option<&SourcedMatch> {
        self.matches.iter().find(|m| m.data.id == id)
    }

    pub fn team(&self, id: TeamId) -> Option<&Team> {
        self.teams.iter().find(|t| t.id == id)
    }

    pub fn stage(&self, ordinal: StageOrdinal) -> Option<&Stage> {
        self.stages.iter().find(|s| s.ordinal == ordinal)
    }
}

/// The League's view of an Event: `base` with the League's Overrides
/// applied. Without Overrides (until #13) it is `base` unchanged.
pub fn effective_view(base: EventView) -> EventView {
    base
}
