//! HLTV vetoes and played map scores linked to strictly dated PandaScore results.
use super::{
    ReplayStage,
    map_history::{MatchHistory, pair},
};
use crate::adapters::{
    hltv_research::archive::{HltvMatchRecord, Side, VetoAction},
    lab::signals::per_map_rating::map_key,
};
use crate::{AsOf, Format, Instant, TemporalLeakError, team_names::team_key};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Default)]
pub struct VetoHistory {
    bo1: MatchHistory<CompletedVeto>,
    bo3: MatchHistory<CompletedSeriesVeto>,
    all: MatchHistory<CompletedSeriesVeto>,
    round_scores: MatchHistory<PlayedRoundScores>,
}

impl VetoHistory {
    /// Whether any complete linked HLTV Bo1 veto exists (backwards-compatible
    /// check used by `bo1_veto_sim`).
    pub fn is_empty(&self) -> bool {
        self.bo1.is_empty()
    }

    /// Whether any complete linked HLTV Bo3 veto exists.
    pub fn has_bo3(&self) -> bool {
        !self.bo3.is_empty()
    }

    pub fn before(&self, cutoff: Instant) -> Self {
        Self {
            bo1: self.bo1.before(cutoff),
            bo3: self.bo3.before(cutoff),
            all: self.all.before(cutoff),
            round_scores: self.round_scores.before(cutoff),
        }
    }

    pub(crate) fn without_stages(&self, excluded: &BTreeSet<i64>) -> Self {
        Self {
            bo1: self.bo1.without_stages(excluded),
            bo3: self.bo3.without_stages(excluded),
            all: self.all.without_stages(excluded),
            round_scores: self.round_scores.without_stages(excluded),
        }
    }

    pub(crate) fn exclude_round_scores(&mut self, stages: &BTreeSet<i64>) {
        self.round_scores = self.round_scores.without_stages(stages);
    }

    pub(crate) fn has_round_scores(&self) -> bool {
        !self.round_scores.is_empty()
    }

    pub(crate) fn round_scores_at(
        &self,
        cutoff: Instant,
    ) -> impl Iterator<Item = Result<(&PlayedRoundScores, Instant), TemporalLeakError>> {
        self.round_scores.at(cutoff)
    }

    /// Linked Bo1 vetoes, each rechecked against `cutoff`.
    pub fn at(
        &self,
        cutoff: Instant,
    ) -> impl Iterator<Item = Result<(&CompletedVeto, Instant), TemporalLeakError>> {
        self.bo1.at(cutoff)
    }

    /// Linked Bo3 vetoes, each rechecked against `cutoff`.
    pub fn bo3_at(
        &self,
        cutoff: Instant,
    ) -> impl Iterator<Item = Result<(&CompletedSeriesVeto, Instant), TemporalLeakError>> {
        self.bo3.at(cutoff)
    }

    /// All linked complete 7-step vetoes (Bo1, Bo3, Bo5), each rechecked against `cutoff`.
    pub fn all_at(
        &self,
        cutoff: Instant,
    ) -> impl Iterator<Item = Result<(&CompletedSeriesVeto, Instant), TemporalLeakError>> {
        self.all.at(cutoff)
    }
}

#[derive(Clone)]
pub(crate) struct PlayedRoundScores {
    pub teams: [String; 2],
    pub scores: Vec<(u32, u32)>,
}

/// A complete series score, in HLTV team order, independently of its veto.
fn round_scores(record: &HltvMatchRecord, game: &super::ReplayMatch) -> Option<PlayedRoundScores> {
    let teams = [team_key(&record.team_a.name), team_key(&record.team_b.name)];
    if teams[0] == teams[1] {
        return None;
    }
    let mut wins = (0, 0);
    let mut scores = Vec::new();
    for map in record.maps.iter().filter(|map| map.played) {
        let (a, b) = (map.team_a_score?, map.team_b_score?);
        match map.winner? {
            Side::A if a > b => wins.0 += 1,
            Side::B if b > a => wins.1 += 1,
            _ => return None,
        }
        scores.push((a, b));
    }
    let needed = u32::from(format_code(game.game.format)) / 2 + 1;
    if wins.0.max(wins.1) != needed || wins.0.min(wins.1) >= needed {
        return None;
    }
    let replay_a = team_key(&game.game.team_a.name);
    let a_won = game.winner == game.game.team_a.id;
    if (teams[usize::from(wins.1 > wins.0)] == replay_a) != a_won {
        return None;
    }
    let replay_score = if teams[0] == replay_a {
        wins
    } else {
        (wins.1, wins.0)
    };
    if game.maps_score.is_some_and(|score| score != replay_score) {
        return None;
    }
    Some(PlayedRoundScores { teams, scores })
}

#[derive(Clone)]
pub struct CompletedVeto {
    /// The first and second banning teams, in that order.
    pub teams: [String; 2],
    /// Six bans in order, followed by the played map.
    pub maps: [String; 7],
    /// Banning side (index into `teams`) at each step.
    pub ban_order: [usize; 6],
    pub winner: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VetoStepKind {
    Ban,
    Pick,
}

#[derive(Clone)]
pub struct CompletedSeriesVeto {
    pub format: Format,
    /// The first and second acting teams (`teams[0]` acts at step 1).
    pub teams: [String; 2],
    /// Seven distinct canonical map names in veto step order (`maps[6]` is the decider/leftover).
    pub maps: [String; 7],
    /// Acting side (`0` or `1`, index into `teams`) at each of steps 1..=6.
    pub step_roles: [usize; 6],
    /// Action (`Ban` or `Pick`) at each of steps 1..=6.
    pub step_kinds: [VetoStepKind; 6],
    /// Played maps `(canonical_map_name, teams[0] won it)` in played order.
    pub played_maps: Vec<(String, bool)>,
    /// Normalized key of the series winner.
    pub winner: String,
    /// Map score `(teams[0]_wins, teams[1]_wins)` verified against played maps.
    score: (u32, u32),
}

fn complete_bo1(record: &HltvMatchRecord) -> Option<CompletedVeto> {
    let [first, _, _, _, _, _, leftover] = record.veto.as_slice() else {
        return None;
    };
    let mut teams = [team_key(&record.team_a.name), team_key(&record.team_b.name)];
    if teams[0] == teams[1] || leftover.action != VetoAction::LeftOver || leftover.team.is_some() {
        return None;
    }
    let first = team_key(first.team.as_deref()?);
    if first == teams[1] {
        teams.swap(0, 1);
    }
    if first != teams[0] {
        return None;
    }
    let maps = std::array::from_fn(|i| record.veto[i].map_name.trim().to_ascii_lowercase());
    if maps.iter().any(String::is_empty) || maps.iter().collect::<BTreeSet<_>>().len() != 7 {
        return None;
    }
    if leftover.step != 7 {
        return None;
    }
    let mut ban_order = [0; 6];
    for (i, step) in record.veto[..6].iter().enumerate() {
        if usize::from(step.step) != i + 1 || step.action != VetoAction::Removed {
            return None;
        }
        let team = team_key(step.team.as_deref()?);
        ban_order[i] = teams.iter().position(|key| key == &team)?;
    }
    if ban_order.iter().filter(|&&side| side == 0).count() != 3 {
        return None;
    }
    let mut played = record.maps.iter().filter(|map| map.played);
    let map = played.next()?;
    if played.next().is_some() || !map.map_name.trim().eq_ignore_ascii_case(&maps[6]) {
        return None;
    }
    let winner = match map.winner? {
        Side::A => team_key(&record.team_a.name),
        Side::B => team_key(&record.team_b.name),
    };
    Some(CompletedVeto {
        teams,
        maps,
        ban_order,
        winner,
    })
}

fn complete_series(record: &HltvMatchRecord, format: Format) -> Option<CompletedSeriesVeto> {
    let [first, _, _, _, _, _, leftover] = record.veto.as_slice() else {
        return None;
    };
    let mut teams = [team_key(&record.team_a.name), team_key(&record.team_b.name)];
    if teams[0] == teams[1]
        || leftover.step != 7
        || leftover.action != VetoAction::LeftOver
        || leftover.team.is_some()
    {
        return None;
    }
    let first_team = team_key(first.team.as_deref()?);
    let mut swapped = false;
    if first_team == teams[1] {
        teams.swap(0, 1);
        swapped = true;
    }
    if first_team != teams[0] {
        return None;
    }
    let maps: [String; 7] = std::array::from_fn(|i| map_key(&record.veto[i].map_name));
    if maps.iter().any(String::is_empty) || maps.iter().collect::<BTreeSet<_>>().len() != 7 {
        return None;
    }
    let mut step_roles = [0usize; 6];
    let mut step_kinds = [VetoStepKind::Ban; 6];
    for (i, step) in record.veto[..6].iter().enumerate() {
        if usize::from(step.step) != i + 1 {
            return None;
        }
        step_kinds[i] = match step.action {
            VetoAction::Removed => VetoStepKind::Ban,
            VetoAction::Picked => VetoStepKind::Pick,
            VetoAction::LeftOver => return None,
        };
        let team = team_key(step.team.as_deref()?);
        step_roles[i] = teams.iter().position(|key| key == &team)?;
    }

    match format {
        Format::Bo1 => {
            if step_kinds.iter().any(|&k| k != VetoStepKind::Ban)
                || step_roles.iter().filter(|&&s| s == 0).count() != 3
            {
                return None;
            }
        }
        Format::Bo3 => {
            if step_kinds
                != [
                    VetoStepKind::Ban,
                    VetoStepKind::Ban,
                    VetoStepKind::Pick,
                    VetoStepKind::Pick,
                    VetoStepKind::Ban,
                    VetoStepKind::Ban,
                ]
                || step_roles[0] != 0
                || step_roles[1] != 1
                || step_roles[2] == step_roles[3]
                || step_roles[4] == step_roles[5]
            {
                return None;
            }
        }
        Format::Bo5 => {
            if step_kinds
                != [
                    VetoStepKind::Ban,
                    VetoStepKind::Ban,
                    VetoStepKind::Pick,
                    VetoStepKind::Pick,
                    VetoStepKind::Pick,
                    VetoStepKind::Pick,
                ]
                || step_roles[0] != 0
                || step_roles[1] != 1
                || step_roles[2..6].iter().filter(|&&s| s == 0).count() != 2
            {
                return None;
            }
        }
    }

    let mut played_maps = Vec::new();
    let mut wins_0 = 0u32;
    let mut wins_1 = 0u32;
    for map in record.maps.iter().filter(|m| m.played) {
        let name = map_key(&map.map_name);
        if name.is_empty() {
            return None;
        }
        let team0_won = match map.winner? {
            Side::A => !swapped,
            Side::B => swapped,
        };
        if team0_won {
            wins_0 += 1;
        } else {
            wins_1 += 1;
        }
        played_maps.push((name, team0_won));
    }

    let played_set: BTreeSet<&str> = played_maps.iter().map(|(m, _)| m.as_str()).collect();
    if played_set.len() != played_maps.len() {
        return None;
    }
    match format {
        Format::Bo1 => {
            if played_maps.len() != 1 || played_maps[0].0 != maps[6] {
                return None;
            }
        }
        Format::Bo3 => {
            if wins_0.max(wins_1) != 2 || played_maps.len() != (wins_0 + wins_1) as usize {
                return None;
            }
            let expected: BTreeSet<&str> = match played_maps.len() {
                2 => [maps[2].as_str(), maps[3].as_str()].into_iter().collect(),
                3 => [maps[2].as_str(), maps[3].as_str(), maps[6].as_str()]
                    .into_iter()
                    .collect(),
                _ => return None,
            };
            if played_set != expected {
                return None;
            }
        }
        Format::Bo5 => {
            if wins_0.max(wins_1) != 3 || !(3..=5).contains(&played_maps.len()) {
                return None;
            }
            let allowed: BTreeSet<&str> = [
                maps[2].as_str(),
                maps[3].as_str(),
                maps[4].as_str(),
                maps[5].as_str(),
                maps[6].as_str(),
            ]
            .into_iter()
            .collect();
            if !played_set.is_subset(&allowed) {
                return None;
            }
        }
    }

    let winner = if wins_0 > wins_1 {
        teams[0].clone()
    } else {
        teams[1].clone()
    };
    Some(CompletedSeriesVeto {
        format,
        teams,
        maps,
        step_roles,
        step_kinds,
        played_maps,
        winner,
        score: (wins_0, wins_1),
    })
}

fn format_code(format: Format) -> u8 {
    match format {
        Format::Bo1 => 1,
        Format::Bo3 => 3,
        Format::Bo5 => 5,
    }
}

pub(super) fn link(
    stages: &[ReplayStage],
    records: impl IntoIterator<Item = HltvMatchRecord>,
) -> VetoHistory {
    let mut by_format_pair = BTreeMap::<_, Vec<_>>::new();
    for stage in stages {
        for game in &stage.matches {
            by_format_pair
                .entry((
                    format_code(game.game.format),
                    pair(&game.game.team_a.name, &game.game.team_b.name),
                ))
                .or_default()
                .push((stage.id, game));
        }
    }

    let mut linked_bo1 = BTreeMap::new();
    let mut linked_bo3 = BTreeMap::new();
    let mut linked_all = BTreeMap::new();
    let mut linked_round_scores = BTreeMap::new();

    for record in records {
        let key = pair(&record.team_a.name, &record.team_b.name);

        if record.format == "bo1"
            && let Some(veto) = complete_bo1(&record)
            && let Some(candidates) = by_format_pair.get(&(format_code(Format::Bo1), key.clone()))
        {
            let mut matches = candidates.iter().filter(|(_, game)| {
                (game.actual_start - record.started_at).abs() <= chrono::Duration::hours(6)
            });
            if let Some(&(stage_id, game)) = matches.next()
                && matches.next().is_none()
                && let Some(end) = game.reported_finish.filter(|&end| end > game.actual_start)
            {
                let replay_winner = if game.winner == game.game.team_a.id {
                    &game.game.team_a.name
                } else {
                    &game.game.team_b.name
                };
                if veto.winner == team_key(replay_winner) {
                    let fetched = if record.fetched_at == chrono::DateTime::UNIX_EPOCH {
                        end + chrono::Duration::seconds(1)
                    } else {
                        record.fetched_at
                    };
                    if let Ok(entry) = AsOf::new(veto, end, fetched) {
                        linked_bo1
                            .entry(game.game.id)
                            .and_modify(|v| *v = None)
                            .or_insert(Some((stage_id, entry)));
                    }
                }
            }
        }

        let format = match record.format.as_str() {
            "bo1" => Format::Bo1,
            "bo3" => Format::Bo3,
            "bo5" => Format::Bo5,
            _ => continue,
        };
        let Some(candidates) = by_format_pair.get(&(format_code(format), key)) else {
            continue;
        };
        let mut matches = candidates.iter().filter(|(_, game)| {
            (game.actual_start - record.started_at).abs() <= chrono::Duration::hours(6)
        });
        let Some(&(stage_id, game)) = matches.next() else {
            continue;
        };
        if matches.next().is_some() {
            continue;
        }
        let Some(end) = game.reported_finish.filter(|&end| end > game.actual_start) else {
            continue;
        };
        let fetched = if record.fetched_at == chrono::DateTime::UNIX_EPOCH {
            end + chrono::Duration::seconds(1)
        } else {
            record.fetched_at
        };
        // Historical scores become observable at the reported finish. The
        // archive's fetch must postdate it, just as for the veto replay.
        if let Some(scores) = round_scores(&record, game)
            && let Ok(entry) = AsOf::new(scores, end, fetched)
        {
            linked_round_scores
                .entry(game.game.id)
                .and_modify(|v| *v = None)
                .or_insert(Some((stage_id, entry)));
        }
        let Some(series) = complete_series(&record, format) else {
            continue;
        };
        let replay_winner = if game.winner == game.game.team_a.id {
            &game.game.team_a.name
        } else {
            &game.game.team_b.name
        };
        if series.winner != team_key(replay_winner) {
            continue;
        }
        if let Some(expected_score) = game.maps_score {
            let replay_a = team_key(&game.game.team_a.name);
            let actual_score = if series.teams[0] == replay_a {
                series.score
            } else {
                (series.score.1, series.score.0)
            };
            if actual_score != expected_score {
                continue;
            }
        }
        let Ok(entry) = AsOf::new(series, end, fetched) else {
            continue;
        };
        if game.game.format == Format::Bo3 {
            linked_bo3
                .entry(game.game.id)
                .and_modify(|v| *v = None)
                .or_insert(Some((stage_id, entry.clone())));
        }
        linked_all
            .entry(game.game.id)
            .and_modify(|v| *v = None)
            .or_insert(Some((stage_id, entry)));
    }

    VetoHistory {
        bo1: MatchHistory::new(linked_bo1.into_values().flatten().collect()),
        bo3: MatchHistory::new(linked_bo3.into_values().flatten().collect()),
        all: MatchHistory::new(linked_all.into_values().flatten().collect()),
        round_scores: MatchHistory::new(linked_round_scores.into_values().flatten().collect()),
    }
}
