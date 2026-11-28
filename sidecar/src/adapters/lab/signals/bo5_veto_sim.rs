//! Exact integration over a seven-step Bo5 veto (Ban-Ban-Pick-Pick-Pick-Pick-Decider),
//! the Bo5 counterpart of #52's `bo3_veto_sim` (ticket #54).
//!
//! The Bo5 step role order comes from the latest prior completed Bo5 veto, and
//! the active seven-map pool comes from the latest prior completed veto of any
//! format (so map-pool rotations observed in earlier Matches of the same Event
//! apply immediately). Each team's ban and pick frequencies from prior HLTV
//! vetoes weight its draws; each map's win probability comes from
//! `per_map_rating` Elo; and the five selected maps combine into team A's Bo5
//! series probability (three of five, in map order) before the anchor blend
//! with the market.
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

pub struct Bo5VetoSim;

impl CandidateSignal for Bo5VetoSim {
    fn name(&self) -> &'static str {
        "bo5_veto_sim"
    }

    fn description(&self) -> &'static str {
        "Bo5 veto simulation from past HLTV vetoes and per-map Elo ratings, blended with the market"
    }

    fn needs(&self) -> &'static str {
        "a prior complete seven-step HLTV Bo5 veto and veto history for either team"
    }

    fn added_after_list_fixed(&self) -> bool {
        true
    }

    fn missing_data(&self, archive: &ReplayArchive) -> Option<String> {
        (!archive.veto_history.has_bo5()).then(|| {
            "No complete HLTV Bo5 vetoes linked to finished PandaScore Matches in the archive"
                .into()
        })
    }

    fn fit(&self, _: &[Observed]) -> Result<Box<dyn FittedSignal>> {
        Ok(Box::new(Bo5VetoSim))
    }
}

/// Team A's Bo5 series-win probability from five independent map probabilities
/// `[p1, p2, p3, p4, p5]` in map order:
/// win 3-0 (`p1 * p2 * p3`), win 3-1 (two wins in maps 1..3, win map 4),
/// or win 3-2 (two wins in maps 1..4, win map 5).
pub fn bo5_series_probability(p: [f64; 5]) -> f64 {
    let [p1, p2, p3, p4, p5] = p;
    let [q1, q2, q3, q4] = [1.0 - p1, 1.0 - p2, 1.0 - p3, 1.0 - p4];
    let win_3_0 = p1 * p2 * p3;
    let two_of_3 = p1 * p2 * q3 + p1 * q2 * p3 + q1 * p2 * p3;
    let win_3_1 = two_of_3 * p4;
    let two_of_4 = p1 * p2 * q3 * q4
        + p1 * q2 * p3 * q4
        + p1 * q2 * q3 * p4
        + q1 * p2 * p3 * q4
        + q1 * p2 * q3 * p4
        + q1 * q2 * p3 * p4;
    let win_3_2 = two_of_4 * p5;
    win_3_0 + win_3_1 + win_3_2
}

/// The sum of `weights` over the maps still in `mask`, in map order.
fn masked_sum(weights: &[f64; 7], mask: u8) -> f64 {
    weights
        .iter()
        .enumerate()
        .filter(|(i, _)| mask & (1 << i) != 0)
        .fold(0.0, |total, (_, w)| total + w)
}

/// Exact integration over the four Bo5 picks (`order[2..6]`) and the remaining
/// decider map once the two opening bans leave `mask5`.
fn eval_picks_and_decider(
    mask5: u8,
    pick_order: [usize; 4],
    pick_weights: &[[f64; 7]; 2],
    wins: &[f64; 7],
) -> f64 {
    let [side2, side3, side4, side5] = pick_order;
    let total_p1 = masked_sum(&pick_weights[side2], mask5);
    let mut sum_p1 = 0.0;
    for (m1, &w1) in pick_weights[side2].iter().enumerate() {
        let bit1 = 1u8 << m1;
        if mask5 & bit1 == 0 {
            continue;
        }
        let mask4 = mask5 ^ bit1;
        let total_p2 = masked_sum(&pick_weights[side3], mask4);
        let mut sum_p2 = 0.0;
        for (m2, &w2) in pick_weights[side3].iter().enumerate() {
            let bit2 = 1u8 << m2;
            if mask4 & bit2 == 0 {
                continue;
            }
            let mask3 = mask4 ^ bit2;
            let total_p3 = masked_sum(&pick_weights[side4], mask3);
            let mut sum_p3 = 0.0;
            for (m3, &w3) in pick_weights[side4].iter().enumerate() {
                let bit3 = 1u8 << m3;
                if mask3 & bit3 == 0 {
                    continue;
                }
                let mask2 = mask3 ^ bit3;
                let total_p4 = masked_sum(&pick_weights[side5], mask2);
                let mut sum_p4 = 0.0;
                for (m4, &w4) in pick_weights[side5].iter().enumerate() {
                    let bit4 = 1u8 << m4;
                    if mask2 & bit4 == 0 {
                        continue;
                    }
                    let decider = (mask2 ^ bit4).trailing_zeros() as usize;
                    sum_p4 += (w4 / total_p4)
                        * bo5_series_probability([
                            wins[m1],
                            wins[m2],
                            wins[m3],
                            wins[m4],
                            wins[decider],
                        ]);
                }
                sum_p3 += (w3 / total_p3) * sum_p4;
            }
            sum_p2 += (w2 / total_p2) * sum_p3;
        }
        sum_p1 += (w1 / total_p1) * sum_p2;
    }
    sum_p1
}

/// Exact integration over one fixed role assignment of a 7-step Bo5 veto:
/// Ban (`order[0]`), Ban (`order[1]`), Pick 1..4 (`order[2..6]`), LeftOver (decider).
fn simulate_fixed(
    order: &[usize; 6],
    ban_weights: &[[f64; 7]; 2],
    pick_weights: &[[f64; 7]; 2],
    wins: &[f64; 7],
) -> f64 {
    let mut after_bans_cache = [None; 128];
    let pick_order = [order[2], order[3], order[4], order[5]];
    let side0 = order[0];
    let side1 = order[1];
    let total_b1: f64 = ban_weights[side0].iter().sum();
    let mut value = 0.0;
    for (b1, &wb1) in ban_weights[side0].iter().enumerate() {
        let mask6 = 127u8 ^ (1u8 << b1);
        let total_b2 = masked_sum(&ban_weights[side1], mask6);
        let mut sum_b2 = 0.0;
        for (b2, &wb2) in ban_weights[side1].iter().enumerate() {
            let bit2 = 1u8 << b2;
            if mask6 & bit2 == 0 {
                continue;
            }
            let mask5 = mask6 ^ bit2;
            let after_bans = *after_bans_cache[usize::from(mask5)]
                .get_or_insert_with(|| eval_picks_and_decider(mask5, pick_order, pick_weights, wins));
            sum_b2 += (wb2 / total_b2) * after_bans;
        }
        value += (wb1 / total_b1) * sum_b2;
    }
    value
}

/// Exact Bo5 series probability integrating over both unknown veto role
/// assignments (Team A acting first vs Team B acting first).
pub fn simulate_bo5_veto(
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

/// A finished Match's played maps: finish, team A, team B, (map, team A won).
type PlayedGame<'a> = (Instant, &'a str, &'a str, &'a [(String, bool)]);

fn sorted_pair<'a>(a: &'a str, b: &'a str) -> (&'a str, &'a str) {
    if a <= b { (a, b) } else { (b, a) }
}

impl FittedSignal for Bo5VetoSim {
    fn predict(&self, view: &MatchView) -> Result<Option<AsOf<f64>>> {
        if view.game.format != Format::Bo5 {
            return Ok(None);
        }
        let keys = [
            team_key(&view.game.team_a.name),
            team_key(&view.game.team_b.name),
        ];

        // Collect veto ban/pick habits from all prior complete vetoes (Bo1/Bo3/Bo5),
        // the Bo5 step role order from the latest prior Bo5 veto, and the active
        // 7-map pool from the latest prior complete veto of any format.
        let mut by_map = BTreeMap::<&str, VetoCounts>::new();
        let mut latest_any = None;
        let mut latest_bo5 = None;
        let mut observed: Option<Instant> = None;
        let mut veto_games = Vec::new();
        for entry in view.veto_history.all_at(view.cutoff) {
            let (veto, finished_at) = entry?;
            observed = Some(observed.map_or(finished_at, |at| at.max(finished_at)));
            latest_any = Some(veto);
            if veto.format == Format::Bo5 {
                latest_bo5 = Some(veto);
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

        let (Some(latest_bo5), Some(latest_pool)) = (latest_bo5, latest_any) else {
            return Ok(None);
        };
        if by_map.is_empty() {
            return Ok(None);
        }

        // Replay `per_map_rating` Elo from completed Liquipedia map results,
        // filling in linked HLTV played maps not already present in `map_history`
        // (deduplicated by finish instant and team pair, strictly before cutoff).
        let mut seen_matches = BTreeSet::new();
        let mut all_maps: Vec<PlayedGame<'_>> = Vec::new();
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
        for (i, map) in latest_pool.maps.iter().enumerate() {
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

        let series = simulate_bo5_veto(latest_bo5.step_roles, ban_weights, pick_weights, wins);
        let map = wins.iter().sum::<f64>() / 7.0;
        let Some(p) = view.anchor_with(Some(Signal { map, series }))? else {
            return Ok(None);
        };
        let observed = observed.expect("at least one prior Bo5 veto was read");
        let observed = view
            .market
            .as_ref()
            .map_or(observed, |m| observed.max(m.observed_at()));
        Ok(Some(AsOf::new(p, observed, view.cutoff)?))
    }
}
