//! Candidate Signal `bo1_pool_depth` (ticket #62, from #56's criteria research;
//! added after spec #33's candidate list was fixed): Bo1 map-pool depth,
//! first-ban stability, pool overlap and first-ban disruption features derived
//! from all prior complete seven-step HLTV vetoes (Bo1, Bo3 and Bo5), fitted
//! walk-forward as an L2-regularised logit offset to the anchor.
//!
//! # Feature definitions (Bo1 Matches only; both teams require >= 1 prior veto)
//!
//! Let `active = [m_0, ..., m_6]` be the active seven-map pool from the latest
//! complete seven-step HLTV veto finished strictly before the Match cutoff.
//! Every prior complete veto finished at `t < cutoff` where team `T` appeared
//! receives exponential recency weight `w = 2^(-(cutoff - t) / HALF_LIFE_DAYS)`
//! (`HALF_LIFE_DAYS = 90.0` days).
//!
//! For each team `T` and each active map `m`:
//! - `first_bans[m]`: recency-weighted count of vetoes where `m` was `T`'s
//!   opening ban (earliest `VetoStepKind::Ban` step by `T`).
//! - `first_ban_total`: sum of `w` over all vetoes where `T` made a first ban.
//! - `fb_share(T, m) = first_bans[m] / first_ban_total` in `[0, 1]`.
//! - `played[m]`, `wins[m]`: recency-weighted played and won map counts from
//!   `veto.played_maps`.
//! - **Per-map competence**:
//!   `c(T, m) = (wins[m] / (played[m] + COMPETENCE_PRIOR)) * (1 - fb_share(T, m))`
//!   with `COMPETENCE_PRIOR = 1.0`. Zero for unplayed or 0-win maps, smoothly
//!   shrunk for small samples, and discounted when `T` normally first-bans `m`.
//! - **Map-pool depth**: `depth(T) = sum_{m in active} c(T, m)` in `[0, 7)` —
//!   the effective number of active maps team `T` plays competently.
//! - **First-ban stability**:
//!   `stability(T) = max_{m in active} (first_bans[m] + STABILITY_PRIOR / 7) / (first_ban_total + STABILITY_PRIOR)`
//!   with `STABILITY_PRIOR = 2.0` — Dirichlet-smoothed top-1 active-pool
//!   first-ban share (approaches 1 for a strict active permaban, 1/7 for
//!   uniform bans, and drops below 1/7 if the historical permaban rotated out).
//! - **Pool overlap**: weighted Jaccard (Ruzicka) similarity of `c(A, ·)` and
//!   `c(B, ·)` over the seven active maps:
//!   `overlap(A, B) = (sum_m min(c(A, m), c(B, m))) / (sum_m max(c(A, m), c(B, m)))`
//!   (0 when neither team has won an active map).
//! - **First-ban disruption**:
//!   `disrupt(A on B) = sum_{m in active} fb_share(A, m) * c(B, m)` — expected
//!   opponent map competence removed by team A's opening ban.
//!
//! The four combiner features are strictly anti-symmetric under swapping Team A
//! and Team B:
//! 1. `pool_depth_diff = (depth(A) - depth(B)) / 7`
//! 2. `first_ban_stability_diff = stability(A) - stability(B)`
//! 3. `pool_overlap_x_anchor = (overlap(A, B) - 0.5) * logit(anchor)`
//! 4. `first_ban_disruption_diff = disrupt(A on B) - disrupt(B on A)`
//!
//! Walk-forward fitting estimates the four weights by ridge-regularised
//! logistic regression (`LAMBDA = 10.0`, prior 0) with `logit(anchor)` as a
//! fixed offset on Bo1 training Matches from earlier completed Events.
use super::{CandidateSignal, FittedSignal, MatchView, Observed};
use crate::{
    AsOf, Format, Instant, Result,
    adapters::{
        backtest::{
            ReplayArchive,
            veto_history::{CompletedSeriesVeto, VetoStepKind},
        },
        lab::combiner::{LAMBDA, logit, ridge_logistic_with_offset, sigmoid},
    },
    team_names::team_key,
};
use std::collections::BTreeMap;

pub const NAME: &str = "bo1_pool_depth";
pub const FEATURES: [&str; 4] = [
    "pool_depth_diff",
    "first_ban_stability_diff",
    "pool_overlap_x_anchor",
    "first_ban_disruption_diff",
];
/// Exponential recency half-life (days) for weighting prior vetoes.
pub const HALF_LIFE_DAYS: f64 = 90.0;
/// Pseudo-play prior shrinking small per-map win samples toward zero competence.
pub const COMPETENCE_PRIOR: f64 = 1.0;
/// Dirichlet prior mass shrinking first-ban stability toward uniform (1/7).
pub const STABILITY_PRIOR: f64 = 2.0;

pub struct Bo1PoolDepth;

impl CandidateSignal for Bo1PoolDepth {
    fn name(&self) -> &'static str {
        NAME
    }

    fn description(&self) -> &'static str {
        "Bo1 map-pool depth, first-ban stability, pool overlap (interacted with anchor) and \
         first-ban disruption from all prior complete HLTV vetoes, fitted as a walk-forward \
         logit offset to the anchor"
    }

    fn needs(&self) -> &'static str {
        "a Bo1 Match, an anchor probability, and at least one prior complete HLTV veto for both teams"
    }

    fn added_after_list_fixed(&self) -> bool {
        true
    }

    fn missing_data(&self, archive: &ReplayArchive) -> Option<String> {
        (!archive.veto_history.has_any()).then(|| {
            "No complete HLTV vetoes linked to finished PandaScore Matches in the archive".into()
        })
    }

    fn feature_names(&self) -> Vec<&'static str> {
        FEATURES.to_vec()
    }

    fn fit(&self, training: &[Observed]) -> Result<Box<dyn FittedSignal>> {
        let mut rows: Vec<(f64, [f64; 4], bool)> = Vec::new();
        for o in training {
            if let Some((values, observed, anchor)) = extract_features(&o.view)? {
                AsOf::new((), observed, o.view.cutoff)?;
                rows.push((logit(anchor), values, o.team_a_won));
            }
        }
        let refs: Vec<(f64, &[f64], bool)> = rows
            .iter()
            .map(|(offset, values, won)| (*offset, values.as_slice(), *won))
            .collect();
        let fitted = ridge_logistic_with_offset(&refs, &[0.0; 4], LAMBDA);
        let weights: [f64; 4] = std::array::from_fn(|i| fitted[i]);
        Ok(Box::new(FittedBo1PoolDepth {
            training_matches: rows.len(),
            weights,
        }))
    }
}

struct FittedBo1PoolDepth {
    training_matches: usize,
    weights: [f64; 4],
}

impl FittedSignal for FittedBo1PoolDepth {
    fn predict(&self, view: &MatchView) -> Result<Option<AsOf<f64>>> {
        let Some((values, observed, anchor)) = extract_features(view)? else {
            return Ok(None);
        };
        let shift: f64 = self.weights.iter().zip(&values).map(|(w, x)| w * x).sum();
        Ok(Some(AsOf::new(
            sigmoid(logit(anchor) + shift),
            observed,
            view.cutoff,
        )?))
    }

    fn parameters(&self) -> Option<serde_json::Value> {
        Some(serde_json::json!({
            "training_bo1_matches": self.training_matches,
            "features": FEATURES,
            "weights": self.weights.iter().map(|w| (w * 1e6).round() / 1e6).collect::<Vec<_>>(),
        }))
    }

    fn features(&self, view: &MatchView) -> Result<Vec<Option<AsOf<f64>>>> {
        let Some((values, observed, _)) = extract_features(view)? else {
            return Ok(vec![None; FEATURES.len()]);
        };
        values
            .into_iter()
            .map(|value| AsOf::new(value, observed, view.cutoff).map(Some))
            .collect::<std::result::Result<_, _>>()
            .map_err(Into::into)
    }
}

/// Recency-weighted veto and played-map counts accumulated for one team.
#[derive(Clone, Debug, Default)]
pub struct TeamVetoStats {
    pub vetoes: u32,
    pub first_ban_total: f64,
    pub first_bans: BTreeMap<String, f64>,
    pub played: BTreeMap<String, f64>,
    pub wins: BTreeMap<String, f64>,
}

impl TeamVetoStats {
    /// Records one prior veto observation directly (useful for pure tests):
    /// `first_ban` is the team's opening banned canonical map, `played` lists
    /// `(canonical_map, team_won)` for maps played in that series.
    pub fn record_observation(
        &mut self,
        first_ban: Option<&str>,
        played: &[(&str, bool)],
        weight: f64,
    ) {
        self.vetoes += 1;
        if let Some(map) = first_ban {
            self.first_ban_total += weight;
            *self.first_bans.entry(map.to_owned()).or_default() += weight;
        }
        for &(map, won) in played {
            *self.played.entry(map.to_owned()).or_default() += weight;
            if won {
                *self.wins.entry(map.to_owned()).or_default() += weight;
            }
        }
    }

    /// Records a linked complete seven-step veto (`Bo1`, `Bo3` or `Bo5`) for
    /// the team sitting at `role` (`0` or `1` in `veto.teams`).
    pub fn record_veto(&mut self, veto: &CompletedSeriesVeto, role: usize, weight: f64) {
        let first_ban = veto.maps[..6]
            .iter()
            .zip(veto.step_roles.iter().zip(&veto.step_kinds))
            .find_map(|(map, (&step_role, &kind))| {
                (step_role == role && kind == VetoStepKind::Ban).then_some(map.as_str())
            });
        self.vetoes += 1;
        if let Some(map) = first_ban {
            self.first_ban_total += weight;
            *self.first_bans.entry(map.to_owned()).or_default() += weight;
        }
        for (map, team0_won) in &veto.played_maps {
            *self.played.entry(map.clone()).or_default() += weight;
            if (role == 0) == *team0_won {
                *self.wins.entry(map.clone()).or_default() += weight;
            }
        }
    }
}

/// Team map-pool summary over the active seven-map pool.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TeamPoolSummary {
    /// Per-active-map competence `c(T, m) in [0, 1)`.
    pub competence: [f64; 7],
    /// Empirical opening-ban share `first_bans[m] / first_ban_total` per active map.
    pub first_ban_share: [f64; 7],
    /// Effective competent map-pool depth `sum_m c(T, m) in [0, 7)`.
    pub depth: f64,
    /// Dirichlet-smoothed top-1 active-pool first-ban share in `(0, 1)`.
    pub first_ban_stability: f64,
}

/// Summarises a team's veto stats over the active seven-map pool.
pub fn summarize_team(stats: &TeamVetoStats, active_pool: &[String; 7]) -> TeamPoolSummary {
    let mut competence = [0.0; 7];
    let mut first_ban_share = [0.0; 7];
    let mut max_active_first_ban = 0.0_f64;
    let mut depth = 0.0;

    for (i, map) in active_pool.iter().enumerate() {
        let fb = stats.first_bans.get(map.as_str()).copied().unwrap_or(0.0);
        max_active_first_ban = max_active_first_ban.max(fb);
        let share = if stats.first_ban_total > 0.0 {
            (fb / stats.first_ban_total).clamp(0.0, 1.0)
        } else {
            0.0
        };
        first_ban_share[i] = share;

        let played = stats.played.get(map.as_str()).copied().unwrap_or(0.0);
        let wins = stats.wins.get(map.as_str()).copied().unwrap_or(0.0);
        let comp = if wins > 0.0 {
            (wins / (played + COMPETENCE_PRIOR)) * (1.0 - share)
        } else {
            0.0
        };
        competence[i] = comp;
        depth += comp;
    }

    let first_ban_stability =
        (max_active_first_ban + STABILITY_PRIOR / 7.0) / (stats.first_ban_total + STABILITY_PRIOR);

    TeamPoolSummary {
        competence,
        first_ban_share,
        depth,
        first_ban_stability,
    }
}

/// Weighted Jaccard (Ruzicka) overlap of two teams' active-pool competence
/// vectors, in `[0, 1]`. Returns `0.0` when neither team has won an active map.
pub fn pool_overlap(a: &TeamPoolSummary, b: &TeamPoolSummary) -> f64 {
    let mut num = 0.0;
    let mut den = 0.0;
    for i in 0..7 {
        num += a.competence[i].min(b.competence[i]);
        den += a.competence[i].max(b.competence[i]);
    }
    if den > 0.0 { num / den } else { 0.0 }
}

/// Expected active-pool competence of `defender` removed by `attacker`'s
/// opening ban distribution, in `[0, 1)`.
pub fn first_ban_disruption(attacker: &TeamPoolSummary, defender: &TeamPoolSummary) -> f64 {
    (0..7)
        .map(|i| attacker.first_ban_share[i] * defender.competence[i])
        .sum()
}

/// Computes the four anti-symmetric features for a Bo1 Match.
pub fn compute_features(
    a: &TeamPoolSummary,
    b: &TeamPoolSummary,
    anchor_prob: f64,
) -> [f64; 4] {
    let pool_depth_diff = (a.depth - b.depth) / 7.0;
    let first_ban_stability_diff = a.first_ban_stability - b.first_ban_stability;
    let pool_overlap_x_anchor = (pool_overlap(a, b) - 0.5) * logit(anchor_prob);
    let first_ban_disruption_diff = first_ban_disruption(a, b) - first_ban_disruption(b, a);
    [
        pool_depth_diff,
        first_ban_stability_diff,
        pool_overlap_x_anchor,
        first_ban_disruption_diff,
    ]
}

fn extract_features(view: &MatchView) -> Result<Option<([f64; 4], Instant, f64)>> {
    if view.game.format != Format::Bo1 {
        return Ok(None);
    }
    let Some(anchor) = view.anchor()? else {
        return Ok(None);
    };
    let keys = [
        team_key(&view.game.team_a.name),
        team_key(&view.game.team_b.name),
    ];

    let mut stats = [TeamVetoStats::default(), TeamVetoStats::default()];
    let mut latest_pool: Option<(&[String; 7], Instant)> = None;

    for entry in view.veto_history.all_at(view.cutoff) {
        let (veto, finished_at) = entry?;
        latest_pool = Some((&veto.maps, finished_at));
        let mut weight = None;
        for (side, key) in keys.iter().enumerate() {
            if let Some(role) = veto.teams.iter().position(|team| team == key) {
                let w = *weight.get_or_insert_with(|| {
                    let age_days =
                        (view.cutoff - finished_at).num_seconds().max(0) as f64 / 86_400.0;
                    0.5_f64.powf(age_days / HALF_LIFE_DAYS)
                });
                stats[side].record_veto(veto, role, w);
            }
        }
    }

    let Some((active_pool, latest_veto_at)) = latest_pool else {
        return Ok(None);
    };
    if stats[0].vetoes == 0 || stats[1].vetoes == 0 {
        return Ok(None);
    }

    let sum_a = summarize_team(&stats[0], active_pool);
    let sum_b = summarize_team(&stats[1], active_pool);
    let values = compute_features(&sum_a, &sum_b, anchor);
    let observed = view
        .observed_at()
        .map_or(latest_veto_at, |at| at.max(latest_veto_at));
    AsOf::new((), observed, view.cutoff)?;
    Ok(Some((values, observed, anchor)))
}
