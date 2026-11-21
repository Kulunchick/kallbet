//! The canonical tournament model: what every Data Source adapter produces
//! and what the rest of KallBet consumes. Keyed by canonical (HLTV) ids.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{EventId, MatchId, StageOrdinal, TeamId};

/// An external provider of Event and Match data, in fallback priority order
/// (ADR-0001): a smaller [`DataSourceKind::priority`] wins a conflict.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DataSourceKind {
    Hltv,
    PandaScore,
    Liquipedia,
}

impl DataSourceKind {
    /// Highest priority first.
    pub const BY_PRIORITY: [DataSourceKind; 3] = [
        DataSourceKind::Hltv,
        DataSourceKind::PandaScore,
        DataSourceKind::Liquipedia,
    ];

    /// 0 is the highest priority.
    pub fn priority(self) -> u8 {
        self as u8
    }

    /// Stable lowercase name, as stored in the database and sent to the SPA.
    pub fn code(self) -> &'static str {
        match self {
            DataSourceKind::Hltv => "hltv",
            DataSourceKind::PandaScore => "pandascore",
            DataSourceKind::Liquipedia => "liquipedia",
        }
    }

    pub fn from_code(code: &str) -> Option<DataSourceKind> {
        Self::BY_PRIORITY.into_iter().find(|k| k.code() == code)
    }
}

/// A pro CS2 tournament as listed on HLTV.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Event {
    pub id: EventId,
    pub name: String,
    pub starts_at: Option<DateTime<Utc>>,
    pub ends_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Team {
    pub id: TeamId,
    /// The team's name as HLTV spells it.
    pub name: String,
    /// The logo at the Data Source. Never sent to the SPA: logos are fetched
    /// once and served from our side.
    pub logo_url: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StageKind {
    /// A Swiss Stage: has a Swiss Pick'em.
    Swiss,
    /// The Playoffs: has the bracket Pick'em.
    Playoffs,
    /// Any other phase that precedes the Playoffs at an Event without a Swiss
    /// Stage (e.g. GSL groups). Its Matches take Predictions; it has no Pick'em.
    Groups,
}

/// A phase of an Event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stage {
    pub ordinal: StageOrdinal,
    pub kind: StageKind,
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MatchFormat {
    Bo1,
    Bo3,
    Bo5,
}

impl MatchFormat {
    pub fn maps(self) -> u8 {
        match self {
            MatchFormat::Bo1 => 1,
            MatchFormat::Bo3 => 3,
            MatchFormat::Bo5 => 5,
        }
    }

    pub fn from_maps(maps: u8) -> Option<MatchFormat> {
        match maps {
            1 => Some(MatchFormat::Bo1),
            3 => Some(MatchFormat::Bo3),
            5 => Some(MatchFormat::Bo5),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MatchStatus {
    Upcoming,
    Live,
    Finished,
    Cancelled,
}

/// Where a Match sits within its Stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum MatchPlacement {
    /// A Swiss round Match. `wins`-`losses` is the record both teams bring
    /// into it (the "record bucket": 0-0, 1-0, 0-1, ...). `round` starts at 1.
    Swiss { round: u8, wins: u8, losses: u8 },
    /// A Playoffs bracket slot. `rounds_before_final` is 0 for the final, 1
    /// for the semifinals, 2 for the quarterfinals. `slot` counts from 0, top
    /// to bottom; the winners of slots `2n` and `2n + 1` meet in slot `n` of
    /// the next round.
    Playoffs { rounds_before_final: u8, slot: u8 },
    /// No structure known (a `Groups` Stage, or a Data Source that lacks it).
    Unplaced,
}

/// Maps won by each team.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MatchScore {
    pub team_a: u8,
    pub team_b: u8,
}

/// A series between two teams within an Event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Match {
    pub id: MatchId,
    pub stage: StageOrdinal,
    pub placement: MatchPlacement,
    /// `None` while the team is still to be decided (a later bracket slot).
    pub team_a: Option<TeamId>,
    pub team_b: Option<TeamId>,
    pub format: MatchFormat,
    /// The latest known start time; it can move until the Match begins.
    pub scheduled_start: Option<DateTime<Utc>>,
    /// When the Match actually began, if the Data Source says. Most only
    /// report `status == Live`; the poller then stamps the time it first saw
    /// that.
    pub actual_start: Option<DateTime<Utc>>,
    pub status: MatchStatus,
    pub winner: Option<TeamId>,
    pub score: Option<MatchScore>,
}

impl Match {
    /// A Void Match counts for nothing: cancelled, or reported without a winner.
    pub fn is_void(&self) -> bool {
        match self.status {
            MatchStatus::Cancelled => true,
            MatchStatus::Finished => self.winner.is_none(),
            MatchStatus::Upcoming | MatchStatus::Live => false,
        }
    }

    pub fn has_started(&self) -> bool {
        self.actual_start.is_some()
            || matches!(self.status, MatchStatus::Live | MatchStatus::Finished)
    }

    pub fn teams(&self) -> [Option<TeamId>; 2] {
        [self.team_a, self.team_b]
    }
}

/// One Event's full structure as one Data Source reported it at one moment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventSnapshot {
    pub event: Event,
    /// Ordered by `ordinal`.
    pub stages: Vec<Stage>,
    /// Every team referenced by `matches`.
    pub teams: Vec<Team>,
    pub matches: Vec<Match>,
    pub source: DataSourceKind,
    pub fetched_at: DateTime<Utc>,
}

impl EventSnapshot {
    pub fn match_by_id(&self, id: MatchId) -> Option<&Match> {
        self.matches.iter().find(|m| m.id == id)
    }

    pub fn team(&self, id: TeamId) -> Option<&Team> {
        self.teams.iter().find(|t| t.id == id)
    }

    pub fn team_named(&self, name: &str) -> Option<&Team> {
        self.teams.iter().find(|t| t.name == name)
    }

    pub fn stage(&self, ordinal: StageOrdinal) -> Option<&Stage> {
        self.stages.iter().find(|s| s.ordinal == ordinal)
    }

    pub fn matches_in(&self, stage: StageOrdinal) -> impl Iterator<Item = &Match> {
        self.matches.iter().filter(move |m| m.stage == stage)
    }
}
