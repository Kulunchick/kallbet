//! Point-in-time results history port (ticket #40, spec #33): a team's earlier
//! completed Matches derived from the archive and served through the replay
//! ports. Never reads from a live API during a lab run.
use crate::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Stage kind for a historical Match (mirrors `backtest::StageKind` without
/// coupling ports to adapters).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HistoryStageKind {
    Swiss,
    Playoffs,
    Groups,
    Other,
}

/// One completed, non-forfeit Match in the point-in-time results history.
/// Teams are identified by the sidecar's shared `team_names::team_key`, so a
/// rename or alias never splits a team's history across sources or Events.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoricalMatch {
    pub match_id: i64,
    pub event_id: i64,
    pub event_name: String,
    pub stage_id: i64,
    pub stage_name: String,
    pub stage_kind: HistoryStageKind,
    /// Swiss round number when known (e.g. PandaScore "Round 3: ...").
    pub round: Option<u32>,
    /// Shared `team_names::team_key` of team A.
    pub team_a_key: String,
    pub team_a_name: String,
    /// Shared `team_names::team_key` of team B.
    pub team_b_key: String,
    pub team_b_name: String,
    /// Shared `team_names::team_key` of the winning team.
    pub winner_key: String,
    pub format: Format,
    /// Map score `(team_a_maps, team_b_maps)` when reported in the archive.
    pub maps_score: Option<(u32, u32)>,
    pub scheduled_start: Option<Instant>,
    pub actual_start: Instant,
    /// When the result became known (`end_at`, else `actual_start`, never
    /// earlier than `actual_start`). Every `AsOf<HistoricalMatch>` is dated by
    /// this instant.
    pub finished_at: Instant,
}

impl HistoricalMatch {
    /// Whether `team_key` (normalized via `team_names::team_key`) played in this Match.
    pub fn played(&self, team_key: &str) -> bool {
        self.team_a_key == team_key || self.team_b_key == team_key
    }

    /// Whether `team_key` won this Match.
    pub fn won_by(&self, team_key: &str) -> bool {
        self.winner_key == team_key
    }

    /// The opponent's `team_key`, if `team_key` played in this Match.
    pub fn opponent_of(&self, team_key: &str) -> Option<&str> {
        if self.team_a_key == team_key {
            Some(&self.team_b_key)
        } else if self.team_b_key == team_key {
            Some(&self.team_a_key)
        } else {
            None
        }
    }

    /// Total maps played in this series (`team_a_maps + team_b_maps`), if known.
    pub fn maps_played(&self) -> Option<u32> {
        self.maps_score.map(|(a, b)| a + b)
    }
}

/// A point-in-time prefix of `AsOf<HistoricalMatch>` records, backed by a
/// shared `Arc` slice so thousands of `MatchView`s share one allocation without
/// duplicating historical records.
#[derive(Clone, Debug, Default)]
pub struct HistorySlice {
    entries: Arc<[AsOf<HistoricalMatch>]>,
    len: usize,
}

impl HistorySlice {
    /// Wraps an owned vector of `AsOf<HistoricalMatch>` records.
    pub fn from_vec(entries: Vec<AsOf<HistoricalMatch>>) -> Self {
        let len = entries.len();
        Self {
            entries: Arc::from(entries),
            len,
        }
    }

    /// Shares the first `len` entries of `entries` without cloning.
    pub fn prefix(entries: Arc<[AsOf<HistoricalMatch>]>, len: usize) -> Self {
        Self {
            len: len.min(entries.len()),
            entries,
        }
    }

    pub fn as_slice(&self) -> &[AsOf<HistoricalMatch>] {
        &self.entries[..self.len]
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Checks every entry against `cutoff`, returning `TemporalLeakError` if
    /// any record was observed at or after `cutoff`.
    pub fn at(&self, cutoff: Instant) -> Result<Vec<&HistoricalMatch>, TemporalLeakError> {
        self.as_slice().iter().map(|e| e.at(cutoff)).collect()
    }

    /// The latest observation timestamp among the entries, if non-empty.
    pub fn observed_at(&self) -> Option<Instant> {
        self.as_slice().iter().map(AsOf::observed_at).max()
    }
}

/// Point-in-time results history port: answers what Matches finished strictly
/// before `cutoff` (`finished_at < cutoff`), joining teams by
/// `team_names::team_key`.
#[async_trait::async_trait]
pub trait ResultsHistory: Send + Sync {
    /// All completed Matches played by `team` (matched via
    /// `team_names::team_key`) that finished strictly before `cutoff`, ordered
    /// chronologically by `(finished_at, match_id)`.
    async fn team_matches(&self, team: &str, cutoff: Instant) -> Result<HistorySlice>;

    /// All completed Matches across the archive that finished strictly before
    /// `cutoff`, ordered chronologically by `(finished_at, match_id)`.
    async fn matches_before(&self, cutoff: Instant) -> Result<HistorySlice>;
}
