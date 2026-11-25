use crate::*;
use serde::{Deserialize, Serialize};

#[async_trait::async_trait]
pub trait ContextEnricher: Send + Sync {
    async fn facts(&self, game: &Match, cutoff: Instant) -> Result<Vec<AsOf<serde_json::Value>>>;
}

/// Attribution metadata required by upstream licenses (e.g. Liquipedia CC-BY-SA 3.0).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attribution {
    pub source: String,
    pub license: String,
    pub license_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StandInEntry {
    pub player: String,
    pub from_team: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoachEntry {
    pub player: String,
    pub emergency_sub: bool,
    pub former: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RosterNote {
    pub text: String,
    pub source_url: Option<String>,
}

/// One participating team parsed from a point-in-time Liquipedia Event revision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TeamRoster {
    pub team: String,
    pub aliases: Vec<String>,
    pub active_players: Vec<String>,
    pub stand_ins: Vec<StandInEntry>,
    pub benched_players: Vec<String>,
    pub unused_substitutes: Vec<String>,
    pub coaches: Vec<CoachEntry>,
    pub notes: Vec<RosterNote>,
}

/// All participating team rosters extracted from one historical MediaWiki revision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventRosters {
    pub page_title: String,
    pub revid: i64,
    pub revision_timestamp: Instant,
    pub source_url: String,
    pub attribution: Attribution,
    pub teams: Vec<TeamRoster>,
}

/// Team-scoped roster fact emitted into a Match dossier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TeamRosterFact {
    pub team: String,
    pub match_side: String,
    pub page_title: String,
    pub revid: i64,
    pub revision_timestamp: Instant,
    pub source_url: String,
    pub attribution: Attribution,
    pub active_players: Vec<String>,
    pub stand_ins: Vec<StandInEntry>,
    pub benched_players: Vec<String>,
    pub unused_substitutes: Vec<String>,
    pub coaches: Vec<CoachEntry>,
    pub has_stand_in: bool,
    pub notes: Vec<RosterNote>,
}

/// Difference between a team's pre-cutoff VRS baseline roster and active Event roster.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VrsRosterDiffFact {
    pub team: String,
    pub match_side: String,
    pub vrs_as_of: Instant,
    pub liquipedia_revid: i64,
    pub liquipedia_revision_timestamp: Instant,
    pub source_url: String,
    pub vrs_roster: Vec<String>,
    pub active_roster: Vec<String>,
    pub added_players: Vec<String>,
    pub removed_players: Vec<String>,
    pub changed: bool,
}

/// One dated article parsed from an RSS 2.0 news feed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewsItem {
    pub source: String,
    pub feed_url: String,
    pub title: String,
    pub description: String,
    pub url: String,
    pub published_at: Instant,
}

/// Team-relevant news item emitted into a Match dossier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewsItemFact {
    pub source: String,
    pub feed_url: String,
    pub title: String,
    pub description: String,
    pub url: String,
    pub published_at: Instant,
    pub matched_teams: Vec<String>,
}

/// Typed context facts serialized into `Dossier.context`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ContextFact {
    LiquipediaRoster(TeamRosterFact),
    VrsRosterDiff(VrsRosterDiffFact),
    NewsItem(NewsItemFact),
}

impl ContextFact {
    pub fn to_value(&self) -> Result<serde_json::Value> {
        Ok(serde_json::to_value(self)?)
    }
}
