//! Replay adapter for `PlayerStatsHistory` (ticket #45): indexes HLTV
//! `HltvMatchRecord`s loaded into `ReplayArchive`, joins each Match to its
//! PandaScore `ReplayMatch` (by `team_key` pair within ±6h) for verified
//! `finished_at`, and serves per-player prefixes strictly before `cutoff`.
use super::archive::{ReplayArchive, team_key};
use crate::adapters::hltv_research::archive::HltvMatchRecord;
use crate::*;
use chrono::Duration;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

const MATCH_WINDOW_HOURS: i64 = 6;

fn parse_format(raw: &str) -> Format {
    match raw.to_ascii_lowercase().as_str() {
        "bo1" => Format::Bo1,
        "bo5" => Format::Bo5,
        _ => Format::Bo3,
    }
}

fn conservative_duration(format: Format) -> Duration {
    match format {
        Format::Bo1 => Duration::hours(4),
        Format::Bo3 => Duration::hours(8),
        Format::Bo5 => Duration::hours(12),
    }
}

fn canonical_pair(a: &str, b: &str) -> (String, String) {
    let ka = team_key(a);
    let kb = team_key(b);
    if ka <= kb { (ka, kb) } else { (kb, ka) }
}

/// `PlayerStatsHistory` backed by the HLTV match records in a `ReplayArchive`.
pub struct ReplayPlayerHistory {
    by_player: BTreeMap<String, Arc<[AsOf<PlayerAppearance>]>>,
}

impl ReplayPlayerHistory {
    pub fn new(archive: &ReplayArchive) -> Self {
        let mut by_pair: BTreeMap<(String, String), Vec<(Instant, Instant)>> = BTreeMap::new();
        for stage in &archive.stages {
            for m in &stage.matches {
                let pair = canonical_pair(&m.game.team_a.name, &m.game.team_b.name);
                by_pair
                    .entry(pair)
                    .or_default()
                    .push((m.actual_start, m.finished_at));
            }
        }

        let window = Duration::hours(MATCH_WINDOW_HOURS);
        let mut seen_match_ids = BTreeSet::new();
        let mut per_player: BTreeMap<String, Vec<AsOf<PlayerAppearance>>> = BTreeMap::new();

        for rec in &archive.hltv_matches {
            if !seen_match_ids.insert(rec.match_id) {
                continue;
            }
            let format = parse_format(&rec.format);
            let pair = canonical_pair(&rec.team_a.name, &rec.team_b.name);
            let matched_finish = by_pair.get(&pair).and_then(|candidates| {
                candidates
                    .iter()
                    .filter(|(actual, _)| (*actual - rec.started_at).abs() <= window)
                    .min_by_key(|(actual, _)| (*actual - rec.started_at).abs())
                    .map(|(_, finished)| *finished)
            });
            let finished_at = matched_finish
                .unwrap_or_else(|| rec.started_at + conservative_duration(format))
                .max(rec.started_at);
            let maps_played = rec.maps.iter().filter(|m| m.played).count().max(1) as u8;
            let mut seen_player_ids = BTreeSet::new();
            for ts in &rec.overall_stats {
                let tk = team_key(&ts.team_name);
                for p in &ts.players {
                    if !p.rating.is_finite() || !seen_player_ids.insert(p.player_id) {
                        continue;
                    }
                    let nk = player_key(&p.nickname);
                    if nk.is_empty() {
                        continue;
                    }
                    let app = PlayerAppearance {
                        hltv_match_id: rec.match_id,
                        player_id: p.player_id,
                        nickname_key: nk.clone(),
                        nickname: p.nickname.clone(),
                        team_key: tk.clone(),
                        format,
                        maps_played,
                        rating: p.rating,
                        rating_version: p.rating_version.clone(),
                        finished_at,
                    };
                    let entry = AsOf::new(app, finished_at, finished_at + Duration::seconds(1))
                        .expect("finished_at + 1s is strictly after finished_at");
                    per_player.entry(nk).or_default().push(entry);
                }
            }
        }

        let by_player = per_player
            .into_iter()
            .map(|(k, mut v)| {
                v.sort_by_key(|e| {
                    let inner = e
                        .at(e.observed_at() + Duration::seconds(1))
                        .expect("observed_at + 1s");
                    (e.observed_at(), inner.hltv_match_id, inner.player_id)
                });
                (k, Arc::from(v))
            })
            .collect();

        Self { by_player }
    }
}

#[async_trait::async_trait]
impl PlayerStatsHistory for ReplayPlayerHistory {
    async fn player_matches(&self, nickname: &str, cutoff: Instant) -> Result<PlayerStatsSlice> {
        let key = player_key(nickname);
        let Some(entries) = self.by_player.get(&key) else {
            return Ok(PlayerStatsSlice::default());
        };
        let end = entries.partition_point(|e| e.observed_at() < cutoff);
        Ok(PlayerStatsSlice::prefix(entries.clone(), end))
    }
}

/// Checks if a JSON payload represents an `HltvMatchRecord`.
pub(super) fn parse_hltv_record_json(value: &serde_json::Value) -> Option<Result<HltvMatchRecord>> {
    if value["match_id"].is_i64() && value["overall_stats"].is_array() {
        Some(serde_json::from_value::<HltvMatchRecord>(value.clone()).map_err(Into::into))
    } else {
        None
    }
}
