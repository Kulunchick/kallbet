//! Played maps from Liquipedia, joined to the replay's PandaScore finish times.
use super::archive::ReplayStage;
use crate::adapters::hltv_research::archive::{LiquipediaStageMaps, Side};
use crate::team_names::team_key;
use crate::{AsOf, Instant, TemporalLeakError};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

#[derive(Clone)]
pub struct PlayedMaps {
    pub team_a: String,
    pub team_b: String,
    /// Map name and whether team A won it, in played order.
    pub results: Vec<(String, bool)>,
}

pub type MapHistory = MatchHistory<PlayedMaps>;

pub struct MatchHistory<T> {
    entries: Arc<[(i64, AsOf<T>)]>,
    end: usize,
}

impl<T> Default for MatchHistory<T> {
    fn default() -> Self {
        Self {
            entries: Arc::default(),
            end: 0,
        }
    }
}

impl<T> Clone for MatchHistory<T> {
    fn clone(&self) -> Self {
        Self {
            entries: self.entries.clone(),
            end: self.end,
        }
    }
}

impl<T> MatchHistory<T> {
    pub(super) fn new(mut entries: Vec<(i64, AsOf<T>)>) -> Self {
        entries.sort_by_key(|(_, entry)| entry.observed_at());
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
        let end =
            self.entries[..self.end].partition_point(|(_, entry)| entry.observed_at() < cutoff);
        Self {
            entries: self.entries.clone(),
            end,
        }
    }

    pub fn at(
        &self,
        cutoff: Instant,
    ) -> impl Iterator<Item = Result<(&T, Instant), TemporalLeakError>> {
        self.entries[..self.end]
            .iter()
            .map(move |(_, entry)| Ok((entry.at(cutoff)?, entry.observed_at())))
    }
}

impl<T: Clone> MatchHistory<T> {
    pub(crate) fn without_stages(&self, excluded: &BTreeSet<i64>) -> Self {
        if excluded.is_empty() {
            return self.clone();
        }
        Self::new(
            self.entries[..self.end]
                .iter()
                .filter(|(stage, _)| !excluded.contains(stage))
                .cloned()
                .collect(),
        )
    }
}

pub(super) fn pair(a: &str, b: &str) -> (String, String) {
    let mut keys = [team_key(a), team_key(b)];
    keys.sort();
    let [a, b] = keys;
    (a, b)
}

pub(super) fn link_stage(
    stage: &ReplayStage,
    page: LiquipediaStageMaps,
) -> Vec<(i64, AsOf<PlayedMaps>)> {
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
        let Some(end) = replay.reported_finish else {
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
    linked
        .into_values()
        .flatten()
        .map(|entry| (stage.id, entry))
        .collect()
}

/// Links a serie's post-Match pages (ticket #48: the #36 collector keeps them
/// per PandaScore serie, not per Stage) to the serie's Stages. Each Stage is
/// linked against every page as `link_stage` does; a Match two pages describe
/// differently, or the same played maps landing in two Stages (a rematch with
/// the same score read off the other Stage's page), is ambiguous and dropped.
pub(super) fn link_serie(
    stages: &[&ReplayStage],
    pages: &[LiquipediaStageMaps],
) -> Vec<(i64, AsOf<PlayedMaps>)> {
    type Key = (String, String, Vec<(String, bool)>);
    let key = |entry: &AsOf<PlayedMaps>| -> Key {
        let m = entry
            .at(Instant::MAX_UTC)
            .expect("a finished Match is observed before the end of time");
        (m.team_a.clone(), m.team_b.clone(), m.results.clone())
    };
    // (stage, pair, finish) identifies a replay Match within its Stage.
    let mut by_match: BTreeMap<(i64, String, String, Instant), Option<AsOf<PlayedMaps>>> =
        BTreeMap::new();
    for stage in stages {
        for page in pages {
            for (stage_id, entry) in link_stage(stage, page.clone()) {
                let (a, b, _) = key(&entry);
                let id = (stage_id, a, b, entry.observed_at());
                match by_match.get_mut(&id) {
                    None => {
                        by_match.insert(id, Some(entry));
                    }
                    Some(Some(seen)) if key(seen) == key(&entry) => {}
                    Some(slot) => *slot = None,
                }
            }
        }
    }
    let mut stages_of: BTreeMap<Key, BTreeSet<i64>> = BTreeMap::new();
    for ((stage, ..), entry) in &by_match {
        if let Some(entry) = entry {
            stages_of.entry(key(entry)).or_default().insert(*stage);
        }
    }
    by_match
        .into_iter()
        .filter_map(|((stage, ..), entry)| Some((stage, entry?)))
        .filter(|(_, entry)| stages_of[&key(entry)].len() == 1)
        .collect()
}
