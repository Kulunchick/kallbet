//! Point-in-time HLTV player statistics history port (ticket #45, spec #33):
//! per-Match individual player ratings parsed from HLTV `/matches/*` pages and
//! served strictly for Matches finished before the requested cutoff.
use crate::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Normalized comparison key for a CS2 player handle across Liquipedia rosters
/// and HLTV match scoreboards: lowercase alphanumeric characters only.
pub fn player_key(nickname: &str) -> String {
    nickname
        .trim()
        .chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

/// One player's series-level HLTV appearance and rating from a completed Match.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlayerAppearance {
    pub hltv_match_id: i64,
    pub player_id: i64,
    pub nickname_key: String,
    pub nickname: String,
    /// Shared `team_names::team_key` of the team this player represented.
    pub team_key: String,
    pub format: Format,
    pub maps_played: u8,
    pub rating: f64,
    pub rating_version: String,
    /// When the Match finished (PandaScore verified `end_at`, else conservative
    /// upper bound from HLTV `started_at`). Every `AsOf<PlayerAppearance>` is
    /// dated by this instant.
    pub finished_at: Instant,
}

/// Zero-copy point-in-time prefix of `AsOf<PlayerAppearance>` entries.
#[derive(Clone, Debug, Default)]
pub struct PlayerStatsSlice {
    entries: Arc<[AsOf<PlayerAppearance>]>,
    len: usize,
}

impl PlayerStatsSlice {
    pub fn from_vec(entries: Vec<AsOf<PlayerAppearance>>) -> Self {
        let len = entries.len();
        Self {
            entries: Arc::from(entries),
            len,
        }
    }

    pub fn prefix(entries: Arc<[AsOf<PlayerAppearance>]>, len: usize) -> Self {
        Self {
            len: len.min(entries.len()),
            entries,
        }
    }

    pub fn as_slice(&self) -> &[AsOf<PlayerAppearance>] {
        &self.entries[..self.len]
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Checks every entry against `cutoff`, returning `TemporalLeakError` if
    /// any appearance finished at or after `cutoff`.
    pub fn at(&self, cutoff: Instant) -> Result<Vec<&PlayerAppearance>, TemporalLeakError> {
        self.as_slice().iter().map(|e| e.at(cutoff)).collect()
    }

    pub fn observed_at(&self) -> Option<Instant> {
        self.as_slice().iter().map(AsOf::observed_at).max()
    }
}

/// Point-in-time player statistics port: answers what HLTV appearances
/// finished strictly before `cutoff` (`finished_at < cutoff`).
#[async_trait::async_trait]
pub trait PlayerStatsHistory: Send + Sync {
    async fn player_matches(&self, nickname: &str, cutoff: Instant) -> Result<PlayerStatsSlice>;
}
