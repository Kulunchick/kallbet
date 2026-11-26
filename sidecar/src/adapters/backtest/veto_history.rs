//! Complete HLTV Bo1 vetoes, linked to strictly dated PandaScore results.
use super::{
    ReplayStage,
    map_history::{MatchHistory, pair},
};
use crate::adapters::hltv_research::archive::{HltvMatchRecord, Side, VetoAction};
use crate::{AsOf, Format, team_names::team_key};
use std::collections::{BTreeMap, BTreeSet};

pub type VetoHistory = MatchHistory<CompletedVeto>;

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

fn complete(record: &HltvMatchRecord) -> Option<CompletedVeto> {
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

pub(super) fn link(
    stages: &[ReplayStage],
    records: impl IntoIterator<Item = HltvMatchRecord>,
) -> VetoHistory {
    let mut by_pair = BTreeMap::<_, Vec<_>>::new();
    for stage in stages {
        for game in stage
            .matches
            .iter()
            .filter(|g| g.game.format == Format::Bo1)
        {
            by_pair
                .entry(pair(&game.game.team_a.name, &game.game.team_b.name))
                .or_default()
                .push((stage.id, game));
        }
    }
    let mut linked = BTreeMap::new();
    for record in records.into_iter().filter(|r| r.format == "bo1") {
        let Some(veto) = complete(&record) else {
            continue;
        };
        let key = pair(&record.team_a.name, &record.team_b.name);
        let Some(candidates) = by_pair.get(&key) else {
            continue;
        };
        let mut candidates = candidates.iter().filter(|(_, game)| {
            (game.actual_start - record.started_at).abs() <= chrono::Duration::hours(6)
        });
        let Some(&(stage_id, game)) = candidates.next() else {
            continue;
        };
        if candidates.next().is_some() {
            continue;
        }
        let Some(end) = game.reported_finish.filter(|&end| end > game.actual_start) else {
            continue;
        };
        let replay_winner = if game.winner == game.game.team_a.id {
            &game.game.team_a.name
        } else {
            &game.game.team_b.name
        };
        if veto.winner != team_key(replay_winner) {
            continue;
        }
        // A page saved after the finish: its fetch time, or, for a raw page
        // saved without one (read as the Unix epoch), the complete veto with
        // played maps the parser found.
        let fetched = if record.fetched_at == chrono::DateTime::UNIX_EPOCH {
            end + chrono::Duration::seconds(1)
        } else {
            record.fetched_at
        };
        let Ok(entry) = AsOf::new(veto, end, fetched) else {
            continue;
        };
        linked
            .entry(game.game.id)
            .and_modify(|v| *v = None)
            .or_insert(Some((stage_id, entry)));
    }
    VetoHistory::new(linked.into_values().flatten().collect())
}
