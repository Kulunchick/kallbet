//! Played maps from Liquipedia, joined to the replay's PandaScore finish times.
use super::archive::ReplayStage;
use crate::adapters::hltv_research::archive::{LiquipediaStageMaps, Side};
use crate::team_names::team_key;
use crate::{AsOf, Instant, TemporalLeakError};
use serde_json::Value;
use std::{collections::BTreeMap, sync::Arc};

pub struct PlayedMaps {
    pub team_a: String,
    pub team_b: String,
    /// Map name and whether team A won it, in played order.
    pub results: Vec<(String, bool)>,
}

#[derive(Clone, Default)]
pub struct MapHistory {
    entries: Arc<[AsOf<PlayedMaps>]>,
    end: usize,
}

impl MapHistory {
    pub(super) fn new(mut entries: Vec<AsOf<PlayedMaps>>) -> Self {
        entries.sort_by_key(AsOf::observed_at);
        let end = entries.len();
        Self {
            entries: entries.into(),
            end,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.end == 0
    }

    pub fn before(&self, cutoff: Instant) -> Self {
        let end = self.entries[..self.end].partition_point(|entry| entry.observed_at() < cutoff);
        Self {
            entries: self.entries.clone(),
            end,
        }
    }

    pub fn at(
        &self,
        cutoff: Instant,
    ) -> impl Iterator<Item = Result<(&PlayedMaps, Instant), TemporalLeakError>> {
        self.entries[..self.end]
            .iter()
            .map(move |entry| Ok((entry.at(cutoff)?, entry.observed_at())))
    }
}

fn pair(a: &str, b: &str) -> (String, String) {
    let mut keys = [team_key(a), team_key(b)];
    keys.sort();
    let [a, b] = keys;
    (a, b)
}

pub(super) fn link_stage(
    stage: &ReplayStage,
    rows: &[Value],
    page: LiquipediaStageMaps,
) -> Vec<AsOf<PlayedMaps>> {
    let ends: BTreeMap<i64, Instant> = rows
        .iter()
        .filter_map(|row| Some((row["id"].as_i64()?, row["end_at"].as_str()?.parse().ok()?)))
        .collect();
    let mut by_pair = BTreeMap::new();
    for game in &stage.matches {
        let key = pair(&game.game.team_a.name, &game.game.team_b.name);
        by_pair
            .entry(key)
            .and_modify(|v| *v = None)
            .or_insert(Some(game));
    }
    let mut linked = BTreeMap::new();
    for game in page.matches {
        let key = pair(&game.team_a_key, &game.team_b_key);
        let Some(Some(replay)) = by_pair.get(&key) else {
            continue;
        };
        let Some(&end) = ends.get(&replay.game.id) else {
            continue;
        };
        if end <= replay.actual_start || end >= page.revision_timestamp {
            continue;
        }
        let team_a = team_key(&game.team_a_key);
        let team_b = team_key(&game.team_b_key);
        if team_a == team_b {
            continue;
        }
        let results: Option<Vec<_>> = game
            .maps
            .iter()
            .filter(|map| map.played)
            .map(|map| {
                let name = map
                    .map_name
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ")
                    .to_lowercase();
                Some((name, map.winner? == Side::A))
            })
            .collect();
        let Some(results) = results.filter(|maps| !maps.is_empty()) else {
            continue;
        };
        let wins_a = results.iter().filter(|(_, won)| *won).count() as u32;
        let wins_b = results.len() as u32 - wins_a;
        let replay_a = team_key(&replay.game.team_a.name);
        let score_in_replay_order = if team_a == replay_a {
            (wins_a, wins_b)
        } else {
            (wins_b, wins_a)
        };
        if replay.maps_score != Some(score_in_replay_order) {
            continue;
        }
        // Completed map scores describe historical outcomes dated by the Match
        // finish. The later revision is evidence for those outcomes.
        let entry = AsOf::new(
            PlayedMaps {
                team_a,
                team_b,
                results,
            },
            end,
            page.revision_timestamp,
        )
        .expect("finish precedes revision");
        linked
            .entry(replay.game.id)
            .and_modify(|v| *v = None)
            .or_insert(Some(entry));
    }
    linked.into_values().flatten().collect()
}
