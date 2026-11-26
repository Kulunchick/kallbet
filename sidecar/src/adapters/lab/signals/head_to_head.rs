//! Candidate Signal `head_to_head` (ticket #40, spec #33 story 27): recency-
//! weighted head-to-head series record between the two teams (joined by
//! `team_names::team_key`), added as a walk-forward fitted logit offset to the
//! anchor.
use super::own_rating::{format_weight, logit, sigmoid};
use super::recent_form::fit_logit_slope;
use super::{CandidateSignal, FittedSignal, MatchView, Observed};
use crate::team_names::team_key;
use crate::*;

pub const NAME: &str = "head_to_head";
/// Exponential recency half-life for head-to-head meetings (days).
pub const HALF_LIFE_DAYS: f64 = 180.0;
/// Prior pseudo-weight shrinking sparse H2H samples toward zero.
pub const PRIOR_WEIGHT: f64 = 2.0;
/// Fewest anchored training Matches with prior H2H history before fitting a slope.
pub const MIN_FIT: usize = 20;
/// L2 regularization penalty on the logit slope.
pub const L2_LAMBDA: f64 = 1.0;

/// Recency-weighted head-to-head score of `key_a` against `key_b` before
/// `cutoff`, in `(-1, 1)`, together with the latest meeting's `finished_at`.
/// Returns `None` if the two teams have never met before `cutoff`.
pub fn h2h_feature(
    key_a: &str,
    key_b: &str,
    team_a_history: &[&HistoricalMatch],
    cutoff: Instant,
) -> Option<(f64, Instant)> {
    let (mut num, mut den, mut latest): (f64, f64, Option<Instant>) = (0.0, 0.0, None);
    for m in team_a_history {
        if m.finished_at >= cutoff || m.opponent_of(key_a) != Some(key_b) {
            continue;
        }
        let age_days = (cutoff - m.finished_at).num_seconds().max(0) as f64 / 86_400.0;
        let w = 0.5f64.powf(age_days / HALF_LIFE_DAYS) * format_weight(m.format);
        let s = if m.won_by(key_a) { 1.0 } else { -1.0 };
        num += w * s;
        den += w;
        latest = Some(latest.map_or(m.finished_at, |t| t.max(m.finished_at)));
    }
    latest.map(|t| (num / (den + PRIOR_WEIGHT), t))
}

pub struct HeadToHead;

impl CandidateSignal for HeadToHead {
    fn name(&self) -> &'static str {
        NAME
    }
    fn description(&self) -> &'static str {
        "Recency-weighted head-to-head series record between the two teams (180-day half-life, \
         shrunk toward even), added as a fitted logit offset to the anchor"
    }
    fn needs(&self) -> &'static str {
        "prior head-to-head Matches between the two teams and at least 20 anchored training Matches"
    }
    fn missing_data(&self, archive: &crate::adapters::backtest::ReplayArchive) -> Option<String> {
        let total: usize = archive.stages.iter().map(|s| s.matches.len()).sum();
        (total < 2).then(|| {
            "the archive holds fewer than two completed Matches, so no head-to-head history exists"
                .into()
        })
    }
    fn fit(&self, training: &[Observed]) -> Result<Box<dyn FittedSignal>> {
        let mut rows = Vec::new();
        for o in training {
            let Some(anchor) = o.view.anchor()? else {
                continue;
            };
            let hist_a = o.view.team_a_history()?;
            let ka = team_key(&o.view.game.team_a.name);
            let kb = team_key(&o.view.game.team_b.name);
            if let Some((h2h, _)) = h2h_feature(&ka, &kb, &hist_a, o.view.cutoff) {
                rows.push((logit(anchor), h2h, o.team_a_won));
            }
        }
        let beta = (rows.len() >= MIN_FIT)
            .then(|| fit_logit_slope(&rows, L2_LAMBDA))
            .flatten();
        Ok(Box::new(FittedHeadToHead { beta }))
    }
}

pub struct FittedHeadToHead {
    pub beta: Option<f64>,
}

impl FittedSignal for FittedHeadToHead {
    fn predict(&self, m: &MatchView) -> Result<Option<AsOf<f64>>> {
        let (Some(beta), Some(anchor)) = (self.beta, m.anchor()?) else {
            return Ok(None);
        };
        let hist_a = m.team_a_history()?;
        let ka = team_key(&m.game.team_a.name);
        let kb = team_key(&m.game.team_b.name);
        let Some((h2h, latest_h2h)) = h2h_feature(&ka, &kb, &hist_a, m.cutoff) else {
            return Ok(None);
        };
        let p = sigmoid(logit(anchor) + beta * h2h);
        let observed = m.observed_at().map_or(latest_h2h, |o| o.max(latest_h2h));
        Ok(Some(AsOf::new(p, observed, m.cutoff)?))
    }
}
