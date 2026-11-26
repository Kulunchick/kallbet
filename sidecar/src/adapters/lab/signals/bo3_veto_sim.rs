//! Exact integration over a seven-step Bo3 veto (Ban-Ban-Pick-Pick-Ban-Ban-Decider),
//! the Bo3 counterpart of #44's `bo1_veto_sim` (ticket #52).
//!
//! The active seven-map pool and Bo3 step role order come from the latest prior
//! completed Bo3 veto; each team's ban and pick frequencies from prior HLTV
//! vetoes weight its draws; each map's win probability comes from `per_map_rating`
//! Elo; and the three selected maps combine into team A's Bo3 series probability
//! (two of three, in map order) before the anchor blend with the market.
use super::{
    CandidateSignal, FittedSignal, MatchView, Observed,
    per_map_rating::{elo_expected, update_elo},
};
use crate::{
    AsOf, Format, Instant, Result, Signal,
    adapters::backtest::{ReplayArchive, veto_history::VetoStepKind},
    team_names::team_key,
};
use std::collections::{BTreeMap, BTreeSet};

pub struct Bo3VetoSim;

impl CandidateSignal for Bo3VetoSim {
    fn name(&self) -> &'static str {
        "bo3_veto_sim"
    }

    fn description(&self) -> &'static str {
        "Bo3 veto simulation from past HLTV vetoes and per-map Elo ratings, blended with the market"
    }

    fn needs(&self) -> &'static str {
        "a prior complete seven-step HLTV Bo3 veto and veto history for either team"
    }

    fn added_after_list_fixed(&self) -> bool {
        true
    }

    fn missing_data(&self, archive: &ReplayArchive) -> Option<String> {
        (!archive.veto_history.has_bo3()).then(|| {
            "No complete HLTV Bo3 vetoes linked to finished PandaScore Matches in the archive"
                .into()
        })
    }

    fn fit(&self, _: &[Observed]) -> Result<Box<dyn FittedSignal>> {
        Ok(Box::new(Bo3VetoSim))
    }
}

/// Team A's Bo3 series-win probability from three independent map probabilities
/// `(p1, p2, p3)` in map order: win 2-0 (`p1 * p2`), or win 2-1 (`W-L-W` or `L-W-W`).
pub fn bo3_series_probability(p1: f64, p2: f64, p3: f64) -> f64 {
    p1 * p2 + p1 * (1.0 - p2) * p3 + (1.0 - p1) * p2 * p3
}

/// Expected decider-map win probability after the two phase-2 bans (`order_ban2`)
/// remove two of the three maps remaining in `mask3`.
fn expected_decider(
    mask3: u8,
    order_ban2: [usize; 2],
    ban_weights: &[[f64; 7]; 2],
    wins: &[f64; 7],
) -> f64 {
    let mut maps = [0usize; 3];
    let mut n = 0;
    for i in 0..7 {
        if mask3 & (1 << i) != 0 {
            maps[n] = i;
            n += 1;
        }
    }
    debug_assert_eq!(n, 3);
    let side4 = order_ban2[0];
    let side5 = order_ban2[1];
    let w4 = [
        ban_weights[side4][maps[0]],
        ban_weights[side4][maps[1]],
        ban_weights[side4][maps[2]],
    ];
    let total4 = w4[0] + w4[1] + w4[2];
    let mut expected = 0.0;
    for banned_idx in 0..3 {
        let a = maps[(banned_idx + 1) % 3];
        let b = maps[(banned_idx + 2) % 3];
        let wa = ban_weights[side5][a];
        let wb = ban_weights[side5][b];
        // If `a` is banned, `b` is the decider; if `b` is banned, `a` is the decider.
        let decider_win = (wa * wins[b] + wb * wins[a]) / (wa + wb);
        expected += (w4[banned_idx] / total4) * decider_win;
    }
    expected
}

/// Exact integration over one fixed role assignment of a 7-step Bo3 veto:
/// Ban (`order[0]`), Ban (`order[1]`), Pick 1 (`order[2]`), Pick 2 (`order[3]`),
/// Ban (`order[4]`), Ban (`order[5]`), LeftOver (decider).
fn simulate_fixed(
    order: &[usize; 6],
    ban_weights: &[[f64; 7]; 2],
    pick_weights: &[[f64; 7]; 2],
    wins: &[f64; 7],
) -> f64 {
    let mut decider_cache = [None; 128];
    let mut after_bans1_cache = [None; 128];

    let mut eval_after_bans1 = |mask5: u8| -> f64 {
        if let Some(v) = after_bans1_cache[usize::from(mask5)] {
            return v;
        }
        let side2 = order[2];
        let side3 = order[3];
        let mut total_p1 = 0.0;
        for i in 0..7 {
            if mask5 & (1 << i) != 0 {
                total_p1 += pick_weights[side2][i];
            }
        }
        let mut sum_p1 = 0.0;
        for m1 in 0..7 {
            let bit1 = 1u8 << m1;
            if mask5 & bit1 == 0 {
                continue;
            }
            let mask4 = mask5 ^ bit1;
            let mut total_p2 = 0.0;
            for j in 0..7 {
                if mask4 & (1 << j) != 0 {
                    total_p2 += pick_weights[side3][j];
                }
            }
            let mut sum_p2 = 0.0;
            for m2 in 0..7 {
                let bit2 = 1u8 << m2;
                if mask4 & bit2 == 0 {
                    continue;
                }
                let mask3 = mask4 ^ bit2;
                let p3 = *decider_cache[usize::from(mask3)]
                    .get_or_insert_with(|| expected_decider(mask3, [order[4], order[5]], ban_weights, wins));
                // `bo3_series_probability(p1, p2, p3)` is affine in `p3`, so
                // substituting `E[p3 | mask3]` is exact.
                sum_p2 += (pick_weights[side3][m2] / total_p2)
                    * bo3_series_probability(wins[m1], wins[m2], p3);
            }
            sum_p1 += (pick_weights[side2][m1] / total_p1) * sum_p2;
        }
        after_bans1_cache[usize::from(mask5)] = Some(sum_p1);
        sum_p1
    };

    let side0 = order[0];
    let side1 = order[1];
    let total_b1: f64 = ban_weights[side0].iter().sum();
    let mut value = 0.0;
    for b1 in 0..7 {
        let mask6 = 127u8 ^ (1u8 << b1);
        let mut total_b2 = 0.0;
        for j in 0..7 {
            if mask6 & (1 << j) != 0 {
                total_b2 += ban_weights[side1][j];
            }
        }
        let mut sum_b2 = 0.0;
        for b2 in 0..7 {
            let bit2 = 1u8 << b2;
            if mask6 & bit2 == 0 {
                continue;
            }
            sum_b2 += (ban_weights[side1][b2] / total_b2) * eval_after_bans1(mask6 ^ bit2);
        }
        value += (ban_weights[side0][b1] / total_b1) * sum_b2;
    }
    value
}

/// Exact Bo3 series probability integrating over both unknown veto role
/// assignments (Team A acting first vs Team B acting first).
pub fn simulate_bo3_veto(
    order: [usize; 6],
    ban_weights: [[f64; 7]; 2],
    pick_weights: [[f64; 7]; 2],
    wins: [f64; 7],
) -> f64 {
    let a_first = simulate_fixed(&order, &ban_weights, &pick_weights, &wins);
    let b_first = simulate_fixed(
        &order.map(|side| 1 - side),
        &ban_weights,
        &pick_weights,
        &wins,
    );
    ((a_first + b_first) / 2.0).clamp(0.0, 1.0)
}

#[derive(Default)]
struct VetoCounts {
    bans: [u32; 2],
    picks: [u32; 2],
}

fn sorted_pair<'a>(a: &'a str, b: &'a str) -> (&'a str, &'a str) {
    if a <= b { (a, b) } else { (b, a) }
}

impl FittedSignal for Bo3VetoSim {
    fn predict(&self, view: &MatchView) -> Result<Option<AsOf<f64>>> {
        if view.game.format != Format::Bo3 {
            return Ok(None);
        }
        let keys = [
            team_key(&view.game.team_a.name),
            team_key(&view.game.team_b.name),
        ];

        // Collect veto ban/pick habits from all prior complete vetoes (Bo1/Bo3/Bo5)
        // and the active 7-map pool + step order from the latest prior Bo3 veto.
        let mut by_map = BTreeMap::<&str, VetoCounts>::new();
        let mut latest_bo3 = None;
        let mut observed: Option<Instant> = None;
        let mut veto_games = Vec::new();
        for entry in view.veto_history.all_at(view.cutoff) {
            let (veto, finished_at) = entry?;
            observed = Some(observed.map_or(finished_at, |at| at.max(finished_at)));
            if veto.format == Format::Bo3 {
                latest_bo3 = Some(veto);
            }
            veto_games.push((
                finished_at,
                veto.teams[0].as_str(),
                veto.teams[1].as_str(),
                veto.played_maps.as_slice(),
            ));
            for (side, key) in keys.iter().enumerate() {
                let Some(role) = veto.teams.iter().position(|team| team == key) else {
                    continue;
                };
                for (map, (&step_role, &kind)) in veto.maps[..6]
                    .iter()
                    .zip(veto.step_roles.iter().zip(&veto.step_kinds))
                {
                    if step_role == role {
                        let counts = by_map.entry(map.as_str()).or_default();
                        match kind {
                            VetoStepKind::Ban => counts.bans[side] += 1,
                            VetoStepKind::Pick => counts.picks[side] += 1,
                        }
                    }
                }
            }
        }

        let Some(latest) = latest_bo3.filter(|_| !by_map.is_empty()) else {
            return Ok(None);
        };

        // Replay `per_map_rating` Elo from completed Liquipedia map results,
        // filling in linked HLTV played maps not already present in `map_history`
        // (deduplicated by finish instant and team pair, strictly before cutoff).
        let mut seen_matches = BTreeSet::new();
        let mut all_maps: Vec<(Instant, &str, &str, &[(String, bool)])> = Vec::new();
        for entry in view.map_history.at(view.cutoff) {
            let (game, finished_at) = entry?;
            observed = Some(observed.map_or(finished_at, |at| at.max(finished_at)));
            seen_matches.insert((finished_at, sorted_pair(&game.team_a, &game.team_b)));
            all_maps.push((
                finished_at,
                game.team_a.as_str(),
                game.team_b.as_str(),
                game.results.as_slice(),
            ));
        }
        for (finished_at, team_a, team_b, results) in veto_games {
            if !seen_matches.contains(&(finished_at, sorted_pair(team_a, team_b))) {
                all_maps.push((finished_at, team_a, team_b, results));
            }
        }
        all_maps.sort_by(|a, b| (a.0, a.1, a.2).cmp(&(b.0, b.1, b.2)));

        let mut ratings = BTreeMap::<&str, BTreeMap<&str, f64>>::new();
        for (_, team_a, team_b, results) in all_maps {
            update_elo(&mut ratings, team_a, team_b, results);
        }

        let mut ban_weights = [[1.0; 7]; 2];
        let mut pick_weights = [[1.0; 7]; 2];
        let mut wins = [0.5; 7];
        for (i, map) in latest.maps.iter().enumerate() {
            if let Some(counts) = by_map.get(map.as_str()) {
                for side in 0..2 {
                    ban_weights[side][i] += f64::from(counts.bans[side]);
                    pick_weights[side][i] += f64::from(counts.picks[side]);
                }
            }
            if let Some(teams) = ratings.get(map.as_str()) {
                let a = *teams.get(keys[0].as_str()).unwrap_or(&0.0);
                let b = *teams.get(keys[1].as_str()).unwrap_or(&0.0);
                wins[i] = elo_expected(a, b);
            }
        }

        let series = simulate_bo3_veto(latest.step_roles, ban_weights, pick_weights, wins);
        let map = wins.iter().sum::<f64>() / 7.0;
        let Some(p) = view.anchor_with(Some(Signal { map, series }))? else {
            return Ok(None);
        };
        let observed = observed.expect("at least one prior Bo3 veto was read");
        let observed = view
            .market
            .as_ref()
            .map_or(observed, |m| observed.max(m.observed_at()));
        Ok(Some(AsOf::new(p, observed, view.cutoff)?))
    }
}
