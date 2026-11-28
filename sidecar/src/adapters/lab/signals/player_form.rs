//! Candidate Signal `player_form` (ticket #45, spec #33): each playing
//! starter's recent HLTV rating (`RECENT_WINDOW_DAYS = 30`, 14-day half-life)
//! minus their own longer-run HLTV rating (`LONG_WINDOW_DAYS = 180`, 90-day
//! half-life), shrunk toward zero (`FORM_PRIOR_WEIGHT = 4.0` maps) and
//! averaged across the 5 playing starters from the pre-cutoff roster.
//!
//! A player without at least `MIN_FORM_APPEARANCES` (2) appearances including at
//! least one recent Match receives a neutral `0.0` form prior, provided at
//! least `MIN_FORM_PLAYERS` (3 of 5) on each team have measurable form;
//! otherwise the Match is left uncovered.
use super::lineup_strength::{fit_slope_rows, lineup_diff_with, predict_from_slope, raw_feature};
use super::recent_form::fit_logit_slope;
use super::{CandidateSignal, FittedSignal, MatchView, Observed};
use crate::*;
use chrono::Duration;
use serde_json::json;

pub const NAME: &str = "player_form";
/// Recent-level lookback window (days).
pub const RECENT_WINDOW_DAYS: i64 = 30;
/// Exponential half-life inside the recent window (days).
pub const RECENT_HALF_LIFE_DAYS: f64 = 14.0;
/// Longer-run baseline lookback window (days).
pub const LONG_WINDOW_DAYS: i64 = 180;
/// Exponential half-life inside the longer-run baseline window (days).
pub const LONG_HALF_LIFE_DAYS: f64 = 90.0;
/// Map-equivalent shrinkage toward zero form deviation.
pub const FORM_PRIOR_WEIGHT: f64 = 4.0;
/// Fewest prior appearances required to establish a player's baseline.
pub const MIN_FORM_APPEARANCES: usize = 2;
/// Minimum players on each 5-man lineup with measurable form.
pub const MIN_FORM_PLAYERS: usize = 3;
/// Fewest anchored training Matches with player form before fitting a slope.
pub const MIN_FIT: usize = 30;
/// L2 regularization penalty on the logit slope.
pub const L2_LAMBDA: f64 = 1.0;

/// One player's shrunk recent-minus-longer-run HLTV rating deviation at `cutoff`.
/// Returns `None` if the player lacks either a recent appearance or at least
/// `MIN_FORM_APPEARANCES` in the baseline window.
pub fn player_form_delta(
    appearances: &[&PlayerAppearance],
    cutoff: Instant,
) -> Option<(f64, Instant)> {
    let recent_start = cutoff - Duration::days(RECENT_WINDOW_DAYS);
    let long_start = cutoff - Duration::days(LONG_WINDOW_DAYS);

    let (mut r_num, mut r_den, mut r_count) = (0.0, 0.0, 0usize);
    let (mut l_num, mut l_den, mut l_count) = (0.0, 0.0, 0usize);
    let mut latest: Option<Instant> = None;

    for app in appearances {
        if app.finished_at >= cutoff || app.finished_at < long_start {
            continue;
        }
        let age_days = (cutoff - app.finished_at).num_seconds().max(0) as f64 / 86_400.0;
        let maps = f64::from(app.maps_played.max(1));

        let lw = 0.5f64.powf(age_days / LONG_HALF_LIFE_DAYS) * maps;
        l_num += lw * app.rating;
        l_den += lw;
        l_count += 1;
        latest = Some(latest.map_or(app.finished_at, |t| t.max(app.finished_at)));

        if app.finished_at >= recent_start {
            let rw = 0.5f64.powf(age_days / RECENT_HALF_LIFE_DAYS) * maps;
            r_num += rw * app.rating;
            r_den += rw;
            r_count += 1;
        }
    }

    if r_count == 0 || l_count < MIN_FORM_APPEARANCES || r_den <= 0.0 || l_den <= 0.0 {
        return None;
    }
    let recent_avg = r_num / r_den;
    let long_avg = l_num / l_den;
    let shrink = r_den / (r_den + FORM_PRIOR_WEIGHT);
    latest.map(|t| (shrink * (recent_avg - long_avg), t))
}

/// Team A minus Team B lineup form difference and latest observation instant.
pub fn player_form_feature(m: &MatchView) -> Result<Option<(f64, Instant)>> {
    lineup_diff_with(m, 0.0, MIN_FORM_PLAYERS, player_form_delta)
}

pub struct PlayerForm;

impl CandidateSignal for PlayerForm {
    fn name(&self) -> &'static str {
        NAME
    }

    fn description(&self) -> &'static str {
        "Mean recent-minus-longer-run HLTV rating form of the 5 playing starters from the \
         pre-cutoff Liquipedia roster (30d/14d half-life vs 180d/90d half-life, shrunk toward \
         neutral 0.0 with weight 4 maps; debutants/players without 2+ baseline & recent Matches \
         take 0.0 when at least 3 of 5 starters on each team have form, else uncovered), added \
         as a fitted logit offset to the anchor"
    }

    fn needs(&self) -> &'static str {
        "pre-cutoff Liquipedia rosters, recent and baseline HLTV player ratings for at least 3 of \
         5 starters on both teams, and at least 30 anchored training Matches"
    }

    fn missing_data(&self, archive: &crate::adapters::backtest::ReplayArchive) -> Option<String> {
        archive.hltv_matches.is_empty().then(|| {
            "the archive holds no HLTV player statistics records (hltv/), so player form cannot \
             be evaluated"
                .into()
        })
    }

    fn feature_names(&self) -> Vec<&'static str> {
        vec![NAME]
    }

    fn fit(&self, training: &[Observed]) -> Result<Box<dyn FittedSignal>> {
        let rows = fit_slope_rows(training, player_form_feature)?;
        let beta = (rows.len() >= MIN_FIT)
            .then(|| fit_logit_slope(&rows, L2_LAMBDA))
            .flatten();
        Ok(Box::new(FittedPlayerForm {
            beta,
            training_matches: rows.len(),
        }))
    }
}

pub struct FittedPlayerForm {
    pub beta: Option<f64>,
    pub training_matches: usize,
}

impl FittedSignal for FittedPlayerForm {
    fn predict(&self, m: &MatchView) -> Result<Option<AsOf<f64>>> {
        predict_from_slope(self.beta, m, player_form_feature)
    }

    fn parameters(&self) -> Option<serde_json::Value> {
        Some(json!({
            "beta": self.beta,
            "training_matches": self.training_matches,
        }))
    }

    fn features(&self, m: &MatchView) -> Result<Vec<Option<AsOf<f64>>>> {
        raw_feature(m, player_form_feature)
    }
}
