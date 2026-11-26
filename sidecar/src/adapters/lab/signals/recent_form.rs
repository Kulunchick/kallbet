//! Candidate Signal `recent_form` (ticket #40, spec #33 story 26): recent
//! results over the last weeks (`WINDOW_DAYS` = 45, exponential half-life
//! `HALF_LIFE_DAYS` = 14) weighted by opponent strength from the point-in-time
//! Glicko-2 rating, added as a walk-forward fitted logit offset to the anchor.
use super::own_rating::{Glicko2Table, PrefixTable, format_weight, logit, sigmoid};
use super::{CandidateSignal, FittedSignal, MatchView, Observed};
use crate::team_names::team_key;
use crate::*;
use chrono::Duration;

pub const NAME: &str = "recent_form";
/// Lookback window for recent form (days).
pub const WINDOW_DAYS: i64 = 45;
/// Exponential recency half-life inside the window (days).
pub const HALF_LIFE_DAYS: f64 = 14.0;
/// Prior pseudo-weight shrinking small samples toward zero.
pub const PRIOR_WEIGHT: f64 = 2.0;
/// Fewest anchored training Matches with recent form before fitting a slope.
pub const MIN_FIT: usize = 30;
/// L2 regularization penalty on the logit slope.
pub const L2_LAMBDA: f64 = 1.0;

/// Opponent-strength-weighted recent form of `team_key` at `cutoff`, in `(-1, 1)`.
///
/// Each Match finished in `[cutoff - WINDOW_DAYS, cutoff)` contributes
/// `+opp_strength` on a win and `-(1 - opp_strength)` on a loss, weighted by
/// recency and series format, shrunk toward 0 by `PRIOR_WEIGHT`. Returns `None`
/// if the team has no Match in the window.
pub fn team_form(
    team_key: &str,
    team_history: &[&HistoricalMatch],
    ratings: &Glicko2Table,
    cutoff: Instant,
) -> Option<f64> {
    let start = cutoff - Duration::days(WINDOW_DAYS);
    let (mut num, mut den, mut count) = (0.0, 0.0, 0usize);
    for m in team_history {
        if m.finished_at < start || m.finished_at >= cutoff {
            continue;
        }
        let Some(opp) = m.opponent_of(team_key) else {
            continue;
        };
        let age_days = (cutoff - m.finished_at).num_seconds().max(0) as f64 / 86_400.0;
        let w = 0.5f64.powf(age_days / HALF_LIFE_DAYS) * format_weight(m.format);
        let opp_strength = ratings.strength(opp);
        let score = if m.won_by(team_key) {
            opp_strength
        } else {
            -(1.0 - opp_strength)
        };
        num += w * score;
        den += w;
        count += 1;
    }
    (count > 0).then(|| num / (den + PRIOR_WEIGHT))
}

/// Fits `beta` in `logit(p) = base_logit + beta * x` by L2-regularized
/// maximum likelihood (golden-section search on `[-4, 4]`). `None` without rows.
pub fn fit_logit_slope(data: &[(f64, f64, bool)], lambda: f64) -> Option<f64> {
    if data.is_empty() {
        return None;
    }
    let loss = |beta: f64| -> f64 {
        let nll: f64 = data
            .iter()
            .map(|(base, x, won)| {
                let p = sigmoid(base + beta * x);
                let p = if *won { p } else { 1.0 - p };
                -p.clamp(1e-12, 1.0).ln()
            })
            .sum();
        nll + 0.5 * lambda * beta * beta
    };
    let ratio = (5f64.sqrt() - 1.0) / 2.0;
    let (mut a, mut b) = (-4.0, 4.0);
    let mut c = b - ratio * (b - a);
    let mut d = a + ratio * (b - a);
    let (mut fc, mut fd) = (loss(c), loss(d));
    for _ in 0..100 {
        if fc < fd {
            b = d;
            d = c;
            fd = fc;
            c = b - ratio * (b - a);
            fc = loss(c);
        } else {
            a = c;
            c = d;
            fc = fd;
            d = a + ratio * (b - a);
            fd = loss(d);
        }
    }
    Some((a + b) / 2.0)
}

pub struct RecentForm;

impl CandidateSignal for RecentForm {
    fn name(&self) -> &'static str {
        NAME
    }
    fn description(&self) -> &'static str {
        "Recent results over the last 45 days (14-day half-life) weighted by opponent strength \
         from the point-in-time Glicko-2 rating, added as a fitted logit offset to the anchor"
    }
    fn needs(&self) -> &'static str {
        "completed Matches within 45 days for both teams and at least 30 anchored training Matches"
    }
    fn missing_data(&self, archive: &crate::adapters::backtest::ReplayArchive) -> Option<String> {
        let total: usize = archive.stages.iter().map(|s| s.matches.len()).sum();
        (total < 2).then(|| {
            "the archive holds fewer than two completed Matches, so no recent form can be computed"
                .into()
        })
    }
    fn fit(&self, training: &[Observed]) -> Result<Box<dyn FittedSignal>> {
        let mut prefix = PrefixTable::empty();
        let mut rows = Vec::new();
        for o in training {
            let Some(anchor) = o.view.anchor()? else {
                continue;
            };
            let history = o.view.history_before()?;
            prefix.sync_to(&history);
            let hist_a = o.view.team_a_history()?;
            let hist_b = o.view.team_b_history()?;
            let ka = team_key(&o.view.game.team_a.name);
            let kb = team_key(&o.view.game.team_b.name);
            if let (Some(fa), Some(fb)) = (
                team_form(&ka, &hist_a, prefix.table(), o.view.cutoff),
                team_form(&kb, &hist_b, prefix.table(), o.view.cutoff),
            ) {
                rows.push((logit(anchor), fa - fb, o.team_a_won));
            }
        }
        let beta = (rows.len() >= MIN_FIT)
            .then(|| fit_logit_slope(&rows, L2_LAMBDA))
            .flatten();
        Ok(Box::new(FittedRecentForm { beta, base: prefix }))
    }
}

pub struct FittedRecentForm {
    pub beta: Option<f64>,
    base: PrefixTable,
}

impl FittedSignal for FittedRecentForm {
    fn predict(&self, m: &MatchView) -> Result<Option<AsOf<f64>>> {
        let (Some(beta), Some(anchor)) = (self.beta, m.anchor()?) else {
            return Ok(None);
        };
        let history = m.history_before()?;
        let hist_a = m.team_a_history()?;
        let hist_b = m.team_b_history()?;
        let ratings = self.base.derive(&history);
        let ka = team_key(&m.game.team_a.name);
        let kb = team_key(&m.game.team_b.name);
        let (Some(fa), Some(fb)) = (
            team_form(&ka, &hist_a, &ratings, m.cutoff),
            team_form(&kb, &hist_b, &ratings, m.cutoff),
        ) else {
            return Ok(None);
        };
        let p = sigmoid(logit(anchor) + beta * (fa - fb));
        let Some(observed) = [
            m.observed_at(),
            m.history_before.observed_at(),
            m.team_a_history.observed_at(),
            m.team_b_history.observed_at(),
        ]
        .into_iter()
        .flatten()
        .max() else {
            return Ok(None);
        };
        Ok(Some(AsOf::new(p, observed, m.cutoff)?))
    }
}
