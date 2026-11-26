//! Exact integration over a six-ban Bo1 veto. The pool and turn order come
//! from the last completed veto, and a team's historical bans weight its draws.
use super::{CandidateSignal, FittedSignal, MatchView, Observed};
use crate::{
    AsOf, Format, Result, Signal, adapters::backtest::ReplayArchive, team_names::team_key,
};
use std::collections::BTreeMap;

pub struct Bo1VetoSim;

impl CandidateSignal for Bo1VetoSim {
    fn name(&self) -> &'static str {
        "bo1_veto_sim"
    }

    fn description(&self) -> &'static str {
        "Bo1 ban simulation from past HLTV vetoes and map results, blended with the market"
    }

    fn needs(&self) -> &'static str {
        "a prior complete seven-step HLTV Bo1 veto for either team"
    }

    fn missing_data(&self, archive: &ReplayArchive) -> Option<String> {
        archive.veto_history.is_empty().then(|| {
            "No complete HLTV Bo1 vetoes linked to finished PandaScore Matches in the archive"
                .into()
        })
    }

    fn fit(&self, _: &[Observed]) -> Result<Box<dyn FittedSignal>> {
        Ok(Box::new(Bo1VetoSim))
    }
}

/// Expected map-win probability over all possible bans. Each turn samples
/// a remaining map proportionally to its banning team's positive weight.
fn simulate(
    mask: u8,
    turn_order: &[usize; 6],
    weights: &[[f64; 7]; 2],
    win: &[f64; 7],
    memo: &mut [Option<f64>; 128],
) -> f64 {
    if let Some(value) = memo[usize::from(mask)] {
        return value;
    }
    let remaining = mask.count_ones() as usize;
    if remaining == 1 {
        return win[mask.trailing_zeros() as usize];
    }
    let side = turn_order[7 - remaining];
    let mut total_weight = 0.0;
    let mut weighted_win = 0.0;
    for (i, &weight) in weights[side].iter().enumerate() {
        let bit = 1 << i;
        if mask & bit != 0 {
            total_weight += weight;
            weighted_win += weight * simulate(mask ^ bit, turn_order, weights, win, memo);
        }
    }
    let value = weighted_win / total_weight;
    memo[usize::from(mask)] = Some(value);
    value
}

fn series_probability(order: [usize; 6], weights: [[f64; 7]; 2], wins: [f64; 7]) -> f64 {
    // The evaluated Match's veto role is unknown. Integrate both assignments.
    let a_first = simulate(127, &order, &weights, &wins, &mut [None; 128]);
    let b_first = simulate(
        127,
        &order.map(|side| 1 - side),
        &weights,
        &wins,
        &mut [None; 128],
    );
    (a_first + b_first) / 2.0
}

#[derive(Default)]
struct MapHistory {
    bans: [u32; 2],
    wins: [u32; 2],
    played: [u32; 2],
}

impl FittedSignal for Bo1VetoSim {
    fn predict(&self, view: &MatchView) -> Result<Option<AsOf<f64>>> {
        if view.game.format != Format::Bo1 {
            return Ok(None);
        }
        let keys = [
            team_key(&view.game.team_a.name),
            team_key(&view.game.team_b.name),
        ];
        let mut by_map = BTreeMap::<&str, MapHistory>::new();
        let mut latest = None;
        for entry in view.veto_history.at(view.cutoff) {
            let (veto, finished_at) = entry?;
            latest = Some((veto, finished_at));
            for (side, key) in keys.iter().enumerate() {
                let Some(role) = veto.teams.iter().position(|team| team == key) else {
                    continue;
                };
                for (map, &banning_role) in veto.maps.iter().zip(&veto.ban_order) {
                    if role == banning_role {
                        by_map.entry(map).or_default().bans[side] += 1;
                    }
                }
                let played = by_map.entry(&veto.maps[6]).or_default();
                played.played[side] += 1;
                played.wins[side] += u32::from(veto.winner == *key);
            }
        }
        let Some((veto, observed)) = latest.filter(|_| !by_map.is_empty()) else {
            return Ok(None);
        };
        let prior = view.signal()?.map_or(0.5, |signal| signal.map);
        let mut weights = [[1.0; 7]; 2];
        let mut wins = [prior; 7];
        for (i, map) in veto.maps.iter().enumerate() {
            if let Some(history) = by_map.get(map.as_str()) {
                for (side, weights) in weights.iter_mut().enumerate() {
                    weights[i] += f64::from(history.bans[side]);
                }
                let wins_for_a = history.wins[0] + history.played[1] - history.wins[1];
                let played = history.played.iter().sum::<u32>();
                // Two prior observations shrink sparse map results toward Valve.
                wins[i] = (f64::from(wins_for_a) + 2.0 * prior) / (f64::from(played) + 2.0);
            }
        }
        let series = series_probability(veto.ban_order, weights, wins);
        let probability = view.anchor_with(Some(Signal {
            map: series,
            series,
        }))?;
        let observed = view.observed_at().map_or(observed, |at| at.max(observed));
        probability
            .map(|p| AsOf::new(p, observed, view.cutoff).map_err(Into::into))
            .transpose()
    }
}

#[cfg(test)]
mod tests {
    use super::series_probability;

    #[test]
    fn veto_averages_uniform_bans_and_favors_a_map_both_teams_avoid_banning() {
        let order = [0, 0, 1, 1, 1, 0];
        let wins = [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        assert!((series_probability(order, [[1.0; 7]; 2], wins) - 1.0 / 7.0).abs() < 1e-12);
        let weights = [[1.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0]; 2];
        assert!(series_probability(order, weights, wins) > 0.6);
    }
}
