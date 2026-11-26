//! Archive data shapes produced by the #36 research parsers and collector,
//! designed for consumption by #44 (map-level) and #45 (player-level) tickets.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    A,
    B,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TeamRef {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MapResult {
    pub position: u8,
    pub map_name: String,
    pub mapstats_id: Option<i64>,
    pub team_a_score: Option<u32>,
    pub team_b_score: Option<u32>,
    pub winner: Option<Side>,
    pub half_scores: Option<String>,
    pub played: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VetoAction {
    Removed,
    Picked,
    LeftOver,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VetoStep {
    pub step: u8,
    pub team: Option<String>,
    pub action: VetoAction,
    pub map_name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlayerMatchStats {
    pub player_id: i64,
    pub nickname: String,
    pub kills: u32,
    pub deaths: u32,
    pub ek_kills: Option<u32>,
    pub ek_deaths: Option<u32>,
    pub swing_pct: Option<f64>,
    pub adr: Option<f64>,
    pub eadr: Option<f64>,
    pub kast_pct: Option<f64>,
    pub ekast_pct: Option<f64>,
    pub rating: f64,
    pub rating_version: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TeamPlayerStats {
    pub team_id: Option<i64>,
    pub team_name: String,
    pub players: Vec<PlayerMatchStats>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MapPlayerStats {
    pub mapstats_id: i64,
    pub map_name: Option<String>,
    pub teams: Vec<TeamPlayerStats>,
}

/// Complete archive record parsed from a single HLTV `/matches/<id>/<slug>` page.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HltvMatchRecord {
    pub match_id: i64,
    pub url: String,
    pub fetched_at: DateTime<Utc>,
    pub event_id: Option<i64>,
    pub event_name: Option<String>,
    pub started_at: DateTime<Utc>,
    pub format: String,
    pub is_lan: Option<bool>,
    pub team_a: TeamRef,
    pub team_b: TeamRef,
    pub maps: Vec<MapResult>,
    pub veto: Vec<VetoStep>,
    pub overall_stats: Vec<TeamPlayerStats>,
    pub per_map_stats: Vec<MapPlayerStats>,
}

/// One map entry parsed from a post-Match Liquipedia `{{Match}}` block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiquipediaMapResult {
    pub position: u8,
    pub map_name: String,
    pub played: bool,
    pub team_a_first_side: Option<String>,
    pub team_a_rounds: Option<u32>,
    pub team_b_rounds: Option<u32>,
    pub winner: Option<Side>,
    pub mapstats_id: Option<i64>,
}

/// One finished Match's map breakdown from a Liquipedia Stage page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiquipediaMatchMaps {
    pub team_a_key: String,
    pub team_b_key: String,
    pub date_raw: Option<String>,
    pub hltv_match_id: Option<i64>,
    pub maps: Vec<LiquipediaMapResult>,
}

/// All finished Matches parsed from a post-Match Liquipedia Stage revision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiquipediaStageMaps {
    pub title: String,
    pub revid: i64,
    pub revision_timestamp: DateTime<Utc>,
    pub matches: Vec<LiquipediaMatchMaps>,
}

/// Summary index entry for one parsed HLTV match inside `manifest.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestMatchEntry {
    pub match_id: i64,
    pub json_file: String,
    pub raw_file: String,
    pub url: String,
    pub event_id: Option<i64>,
    pub event_name: Option<String>,
    pub started_at: DateTime<Utc>,
    pub format: String,
    pub team_a: String,
    pub team_b: String,
    pub maps: usize,
    pub veto_steps: usize,
    pub players: usize,
}

/// Details recorded when the stop rule halts collection (403, 429, or challenge).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StopRecord {
    pub at: DateTime<Utc>,
    pub match_id: i64,
    pub url: String,
    pub status: u16,
    pub reason: String,
    pub saved_file: String,
}

/// Persistent resumable manifest stored at `<out_dir>/manifest.json`.
/// Tracks UTC daily request spending, indexed matches for #44/#45, parse errors,
/// and any stop-rule event.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HltvArchiveManifest {
    pub updated_at: Option<DateTime<Utc>>,
    pub last_ios_user_agent: Option<String>,
    /// UTC date (`YYYY-MM-DD`) -> total HTTP requests sent to HLTV on that day.
    pub daily_requests: std::collections::BTreeMap<String, u32>,
    /// HLTV match_id -> indexed metadata of parsed archive JSON.
    pub matches: std::collections::BTreeMap<i64, ManifestMatchEntry>,
    /// HLTV match_id -> parse error (raw HTML is still cached so it is never refetched).
    pub parse_errors: std::collections::BTreeMap<i64, String>,
    /// Set if the hard stop rule (403, 429, or Cloudflare challenge) triggered.
    pub last_stop: Option<StopRecord>,
}
