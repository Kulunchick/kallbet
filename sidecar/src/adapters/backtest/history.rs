//! Replay adapter for `ResultsHistory` (ticket #40): builds an indexed,
//! point-in-time results history from a `ReplayArchive`, joining teams across
//! sources and renames via `team_names::team_key`.
use super::archive::{ReplayArchive, StageKind, team_key};
use crate::*;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

fn stage_kind(kind: StageKind) -> HistoryStageKind {
    match kind {
        StageKind::Swiss => HistoryStageKind::Swiss,
        StageKind::Playoffs => HistoryStageKind::Playoffs,
        StageKind::Groups => HistoryStageKind::Groups,
        StageKind::Other => HistoryStageKind::Other,
    }
}

/// `ResultsHistory` backed by the completed Matches of a `ReplayArchive`.
///
/// Matches are sorted chronologically by `(finished_at, match_id)`, indexed per
/// team by `team_names::team_key`, and served as zero-copy `HistorySlice`
/// prefixes strictly before the requested `cutoff` (`finished_at < cutoff`).
pub struct ReplayHistory {
    all: Arc<[AsOf<HistoricalMatch>]>,
    by_team: BTreeMap<String, Arc<[AsOf<HistoricalMatch>]>>,
}

impl ReplayHistory {
    pub fn new(archive: &ReplayArchive) -> Self {
        let mut seen = BTreeSet::new();
        let mut raw = Vec::new();
        for stage in &archive.stages {
            let kind = stage_kind(stage.kind);
            for m in &stage.matches {
                if !seen.insert(m.game.id) {
                    continue;
                }
                let team_a_key = team_key(&m.game.team_a.name);
                let team_b_key = team_key(&m.game.team_b.name);
                let winner_key = if m.winner == m.game.team_a.id {
                    team_a_key.clone()
                } else {
                    team_b_key.clone()
                };
                raw.push(HistoricalMatch {
                    match_id: m.game.id,
                    event_id: stage.event_id,
                    event_name: stage.event.clone(),
                    stage_id: stage.id,
                    stage_name: stage.name.clone(),
                    stage_kind: kind,
                    round: m.round,
                    team_a_key,
                    team_a_name: m.game.team_a.name.clone(),
                    team_b_key,
                    team_b_name: m.game.team_b.name.clone(),
                    winner_key,
                    format: m.game.format,
                    maps_score: m.maps_score,
                    scheduled_start: m.game.scheduled_start,
                    actual_start: m.actual_start,
                    finished_at: m.finished_at,
                });
            }
        }
        raw.sort_by_key(|m| (m.finished_at, m.match_id));

        let mut per_team: BTreeMap<String, Vec<AsOf<HistoricalMatch>>> = BTreeMap::new();
        let mut all = Vec::with_capacity(raw.len());
        for m in raw {
            let observed = m.finished_at;
            let entry = AsOf::new(m, observed, observed + chrono::Duration::seconds(1))
                .expect("observed + 1s is strictly after observed");
            let inner = entry.at(observed + chrono::Duration::seconds(1)).unwrap();
            let a_key = inner.team_a_key.clone();
            let b_key = inner.team_b_key.clone();
            per_team.entry(a_key.clone()).or_default().push(entry.clone());
            if b_key != a_key {
                per_team.entry(b_key).or_default().push(entry.clone());
            }
            all.push(entry);
        }

        let by_team = per_team
            .into_iter()
            .map(|(k, v)| (k, Arc::from(v)))
            .collect();
        Self {
            all: Arc::from(all),
            by_team,
        }
    }
}

#[async_trait::async_trait]
impl ResultsHistory for ReplayHistory {
    async fn team_matches(&self, team: &str, cutoff: Instant) -> Result<HistorySlice> {
        let key = team_key(team);
        let Some(entries) = self.by_team.get(&key) else {
            return Ok(HistorySlice::default());
        };
        let end = entries.partition_point(|e| e.observed_at() < cutoff);
        Ok(HistorySlice::prefix(entries.clone(), end))
    }

    async fn matches_before(&self, cutoff: Instant) -> Result<HistorySlice> {
        let end = self.all.partition_point(|e| e.observed_at() < cutoff);
        Ok(HistorySlice::prefix(self.all.clone(), end))
    }
}
