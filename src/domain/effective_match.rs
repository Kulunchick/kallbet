//! Effective Match view (pure): base Match data merged with one League's
//! Overrides. All League logic (Lock/Reveal, scoring, Pick'em, notifications,
//! archive) reads this view, never raw Data Source data.
//!
//! [`effective_view`] is the single place Overrides (#13) are applied.
//! Callers get it through `crate::league_matches::effective_view`, which
//! loads the inputs.

use std::collections::HashMap;

use chrono::{DateTime, Utc};

use super::{
    DataSourceKind, Event, Match, MatchId, MatchStatus, Stage, StageOrdinal, Team, TeamId,
};

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

/// A result set by Override.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultOverride {
    /// The Match is Finished and this team won.
    Winner(TeamId),
    /// The Match is Void (counts for nothing).
    Void,
}

/// The Overrides in force on one Match of one League. `None` / `false`
/// means the Data Sources' value applies.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MatchOverrides {
    pub scheduled_start: Option<DateTime<Utc>>,
    pub result: Option<ResultOverride>,
    /// `(team_a, team_b)`.
    pub teams: Option<(TeamId, TeamId)>,
    pub hidden: bool,
}

impl MatchOverrides {
    pub fn is_empty(&self) -> bool {
        *self == MatchOverrides::default()
    }

    /// The overridden fields' names, as the API spells them.
    pub fn fields(&self) -> Vec<&'static str> {
        let mut fields = Vec::new();
        if self.scheduled_start.is_some() {
            fields.push("scheduled_start");
        }
        if self.result.is_some() {
            fields.push("result");
        }
        if self.teams.is_some() {
            fields.push("teams");
        }
        if self.hidden {
            fields.push("hidden");
        }
        fields
    }
}

/// Everything one League changed about its Event's Matches.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LeagueOverrides {
    /// Matches the League's Admins added by hand (their base data).
    pub added: Vec<SourcedMatch>,
    /// The Overrides in force, per Match (base or hand-added).
    pub matches: HashMap<MatchId, MatchOverrides>,
}

impl LeagueOverrides {
    pub fn of(&self, id: MatchId) -> MatchOverrides {
        self.matches.get(&id).copied().unwrap_or_default()
    }

    pub fn is_added(&self, id: MatchId) -> bool {
        self.added.iter().any(|m| m.data.id == id)
    }
}

/// One Match with its Overrides applied (`hidden` is not applied here).
///
/// - A Scheduled Start Override replaces the Scheduled Start; Lock follows it
///   until the Lock is stored (ADR-0003: a stored Lock never moves).
/// - A teams Override puts those teams on sides A and B. A winner the Data
///   Source reported keeps its side: the record is the same, only which
///   team is on that side was wrong.
/// - A result Override makes the Match Finished with that winner, or Void
///   (Cancelled). The score is dropped unless it agrees with the winner.
pub fn apply(base: &Match, overrides: &MatchOverrides) -> Match {
    let mut game = base.clone();
    if let Some(start) = overrides.scheduled_start {
        game.scheduled_start = Some(start);
    }
    if let Some((a, b)) = overrides.teams {
        game.winner = match base.winner {
            Some(w) if Some(w) == base.team_a => Some(a),
            Some(w) if Some(w) == base.team_b => Some(b),
            other => other,
        };
        game.team_a = Some(a);
        game.team_b = Some(b);
    }
    match overrides.result {
        // A winner must play the Match: a result Override whose winner a
        // later teams Override (or source update) took out is not applied.
        Some(ResultOverride::Winner(team)) if game.teams().contains(&Some(team)) => {
            if game.winner != Some(team) || game.status != MatchStatus::Finished {
                game.score = None;
            }
            game.status = MatchStatus::Finished;
            game.winner = Some(team);
        }
        Some(ResultOverride::Void) => {
            game.status = MatchStatus::Cancelled;
            game.winner = None;
            game.score = None;
        }
        Some(ResultOverride::Winner(_)) | None => {}
    }
    // The same rule for a source winner (inconsistent data): the result is
    // absent, so the Match is undecided rather than won by a non-player.
    if game.winner.is_some_and(|w| !game.teams().contains(&Some(w))) {
        game.winner = None;
        game.score = None;
        if game.status == MatchStatus::Finished {
            game.status = MatchStatus::Live;
        }
    }
    game
}

/// The League's view of an Event: `base` plus the League's hand-added
/// Matches, each with the League's Overrides applied, without hidden ones.
/// Overrides win over the Data Sources for as long as they are in force,
/// however the base data changes later.
pub fn effective_view(base: EventView, overrides: &LeagueOverrides) -> EventView {
    let mut view = with_hidden(base, overrides);
    view.matches.retain(|m| !overrides.of(m.data.id).hidden);
    view
}

/// [`effective_view`] that keeps hidden Matches: what League Admins work on
/// (a hidden Match can still be overridden and unhidden).
pub fn with_hidden(base: EventView, overrides: &LeagueOverrides) -> EventView {
    let mut view = base;
    view.matches.extend(overrides.added.iter().cloned());
    for m in &mut view.matches {
        m.data = apply(&m.data, &overrides.of(m.data.id));
    }
    view.matches.sort_by_key(|m| m.data.id);
    view
}
