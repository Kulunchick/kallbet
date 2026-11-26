//! Candidate Signal `lineup_strength` (ticket #45, spec #33): strength of the
//! five players who actually play (from the pre-cutoff Liquipedia roster
//! revision), priced from each player's HLTV per-Match ratings strictly before
//! the Match cutoff.
//!
//! A debutant or unknown stand-in with no prior HLTV appearance uses a stated
//! neutral prior rating (`PRIOR_RATING = 1.00`), provided at least
//! `MIN_KNOWN_PLAYERS` (3 of 5) on each team have prior HLTV history; otherwise
//! the Match is left uncovered.
use super::own_rating::{logit, sigmoid};
use super::recent_form::fit_logit_slope;
use super::{CandidateSignal, FittedSignal, MatchView, Observed};
use crate::team_names::team_key;
use crate::*;
use serde_json::json;
use std::collections::BTreeSet;

pub const NAME: &str = "lineup_strength";
/// Neutral HLTV Rating prior assigned to a debutant or unknown stand-in.
pub const PRIOR_RATING: f64 = 1.00;
/// Map-equivalent shrinkage weight toward `PRIOR_RATING`.
pub const PRIOR_WEIGHT: f64 = 6.0;
/// Exponential recency half-life for player ratings (days).
pub const HALF_LIFE_DAYS: f64 = 90.0;
/// Minimum players on each 5-man lineup with prior HLTV appearances.
pub const MIN_KNOWN_PLAYERS: usize = 3;
/// Fewest anchored training Matches with lineup strength before fitting a slope.
pub const MIN_FIT: usize = 30;
/// L2 regularization penalty on the logit slope.
pub const L2_LAMBDA: f64 = 1.0;

/// Extracts the 5 playing players from a Liquipedia `TeamRosterFact`,
/// padding from active `emergency_sub` coaches only if fewer than 5 starters
/// are listed. Returns `None` if a 5-player lineup cannot be formed.
pub(super) fn effective_lineup(roster: &TeamRosterFact) -> Option<Vec<String>> {
    let mut seen = BTreeSet::new();
    let mut lineup = Vec::with_capacity(5);
    for nick in &roster.active_players {
        let key = player_key(nick);
        if !key.is_empty() && seen.insert(key) {
            lineup.push(nick.clone());
        }
    }
    if lineup.len() < 5 {
        for coach in &roster.coaches {
            if !coach.emergency_sub || coach.former {
                continue;
            }
            let key = player_key(&coach.player);
            if !key.is_empty() && seen.insert(key) {
                lineup.push(coach.player.clone());
            }
            if lineup.len() == 5 {
                break;
            }
        }
    }
    (lineup.len() == 5).then_some(lineup)
}

/// Resolves a player's `player_id` (preferring appearances for `tk`, else most
/// recent overall for stand-ins/transfers) and returns only their appearances
/// strictly before `cutoff`.
pub(super) fn resolve_player_appearances<'a>(
    raw: &'a [&'a PlayerAppearance],
    tk: &str,
    cutoff: Instant,
) -> Vec<&'a PlayerAppearance> {
    let pre: Vec<&PlayerAppearance> = raw
        .iter()
        .copied()
        .filter(|a| a.finished_at < cutoff)
        .collect();
    let pid = pre
        .iter()
        .rev()
        .find(|a| a.team_key == tk)
        .or_else(|| pre.last())
        .map(|a| a.player_id);
    let Some(pid) = pid else {
        return Vec::new();
    };
    pre.into_iter().filter(|a| a.player_id == pid).collect()
}

/// Recency- and map-weighted HLTV rating of one player at `cutoff`, shrunk
/// toward `PRIOR_RATING`, plus the latest `finished_at` read. `None` if the
/// player has no prior appearance.
pub fn player_rating(appearances: &[&PlayerAppearance], cutoff: Instant) -> Option<(f64, Instant)> {
    let (mut num, mut den, mut latest): (f64, f64, Option<Instant>) = (0.0, 0.0, None);
    for app in appearances {
        if app.finished_at >= cutoff {
            continue;
        }
        let age_days = (cutoff - app.finished_at).num_seconds().max(0) as f64 / 86_400.0;
        let w = 0.5f64.powf(age_days / HALF_LIFE_DAYS) * f64::from(app.maps_played.max(1));
        num += w * app.rating;
        den += w;
        latest = Some(latest.map_or(app.finished_at, |t| t.max(app.finished_at)));
    }
    latest.map(|t| ((num + PRIOR_WEIGHT * PRIOR_RATING) / (den + PRIOR_WEIGHT), t))
}

/// Generic 5-player lineup aggregator shared by `lineup_strength` and `player_form`.
pub(super) fn lineup_diff_with<F>(
    m: &MatchView,
    prior: f64,
    min_known: usize,
    eval_player: F,
) -> Result<Option<(f64, Instant)>>
where
    F: Fn(&[&PlayerAppearance], Instant) -> Option<(f64, Instant)>,
{
    let (Some(ra_wrap), Some(rb_wrap)) = (&m.team_a_roster, &m.team_b_roster) else {
        return Ok(None);
    };
    let ra = ra_wrap.at(m.cutoff)?;
    let rb = rb_wrap.at(m.cutoff)?;

    let team_score = |roster: &TeamRosterFact,
                      roster_obs: Instant,
                      team_name: &str|
     -> Result<Option<(f64, Instant)>> {
        let Some(lineup) = effective_lineup(roster) else {
            return Ok(None);
        };
        let tk = team_key(team_name);
        let mut sum = 0.0;
        let mut known = 0usize;
        let mut latest = roster_obs;
        for nick in &lineup {
            let raw = m.player_matches(nick)?;
            let apps = resolve_player_appearances(&raw, &tk, m.cutoff);
            if let Some((val, obs)) = eval_player(&apps, m.cutoff) {
                sum += val;
                known += 1;
                latest = latest.max(obs);
            } else {
                sum += prior;
            }
        }
        Ok((known >= min_known).then_some((sum / 5.0, latest)))
    };

    let (Some((sa, obs_a)), Some((sb, obs_b))) = (
        team_score(ra, ra_wrap.observed_at(), &m.game.team_a.name)?,
        team_score(rb, rb_wrap.observed_at(), &m.game.team_b.name)?,
    ) else {
        return Ok(None);
    };
    Ok(Some((sa - sb, obs_a.max(obs_b))))
}

/// Team A minus Team B lineup strength difference and latest observation instant.
pub fn lineup_strength_feature(m: &MatchView) -> Result<Option<(f64, Instant)>> {
    lineup_diff_with(m, PRIOR_RATING, MIN_KNOWN_PLAYERS, player_rating)
}

pub(super) fn fit_slope_rows<F>(training: &[Observed], feature_fn: F) -> Result<Vec<(f64, f64, bool)>>
where
    F: Fn(&MatchView) -> Result<Option<(f64, Instant)>>,
{
    let mut rows = Vec::new();
    for o in training {
        let Some(anchor) = o.view.anchor()? else {
            continue;
        };
        if let Some((x, _)) = feature_fn(&o.view)? {
            rows.push((logit(anchor), x, o.team_a_won));
        }
    }
    Ok(rows)
}

pub(super) fn predict_from_slope<F>(
    beta: Option<f64>,
    m: &MatchView,
    feature_fn: F,
) -> Result<Option<AsOf<f64>>>
where
    F: Fn(&MatchView) -> Result<Option<(f64, Instant)>>,
{
    let (Some(beta), Some(anchor)) = (beta, m.anchor()?) else {
        return Ok(None);
    };
    let Some((x, feat_obs)) = feature_fn(m)? else {
        return Ok(None);
    };
    let p = sigmoid(logit(anchor) + beta * x);
    let observed = m.observed_at().map_or(feat_obs, |o| o.max(feat_obs));
    Ok(Some(AsOf::new(p, observed, m.cutoff)?))
}

pub(super) fn raw_feature<F>(m: &MatchView, feature_fn: F) -> Result<Vec<Option<AsOf<f64>>>>
where
    F: Fn(&MatchView) -> Result<Option<(f64, Instant)>>,
{
    match feature_fn(m)? {
        Some((x, obs)) => Ok(vec![Some(AsOf::new(x, obs, m.cutoff)?)]),
        None => Ok(vec![None]),
    }
}

pub struct LineupStrength;

impl CandidateSignal for LineupStrength {
    fn name(&self) -> &'static str {
        NAME
    }

    fn description(&self) -> &'static str {
        "Mean HLTV player rating of the 5 playing starters from the pre-cutoff Liquipedia \
         roster (90-day half-life, shrunk toward prior 1.00 with weight 6 maps; debutants/\
         unknown stand-ins take the 1.00 prior when at least 3 of 5 starters on each team have \
         HLTV history, else uncovered), added as a fitted logit offset to the anchor"
    }

    fn needs(&self) -> &'static str {
        "pre-cutoff Liquipedia rosters, prior HLTV player ratings for at least 3 of 5 starters on \
         both teams, and at least 30 anchored training Matches"
    }

    fn missing_data(&self, archive: &crate::adapters::backtest::ReplayArchive) -> Option<String> {
        archive.hltv_matches.is_empty().then(|| {
            "the archive holds no HLTV player statistics records (hltv/), so lineup strength \
             cannot be evaluated"
                .into()
        })
    }

    fn feature_names(&self) -> Vec<&'static str> {
        vec![NAME]
    }

    fn fit(&self, training: &[Observed]) -> Result<Box<dyn FittedSignal>> {
        let rows = fit_slope_rows(training, lineup_strength_feature)?;
        let beta = (rows.len() >= MIN_FIT)
            .then(|| fit_logit_slope(&rows, L2_LAMBDA))
            .flatten();
        Ok(Box::new(FittedLineupStrength {
            beta,
            training_matches: rows.len(),
        }))
    }
}

pub struct FittedLineupStrength {
    pub beta: Option<f64>,
    pub training_matches: usize,
}

impl FittedSignal for FittedLineupStrength {
    fn predict(&self, m: &MatchView) -> Result<Option<AsOf<f64>>> {
        predict_from_slope(self.beta, m, lineup_strength_feature)
    }

    fn parameters(&self) -> Option<serde_json::Value> {
        Some(json!({
            "beta": self.beta,
            "training_matches": self.training_matches,
        }))
    }

    fn features(&self, m: &MatchView) -> Result<Vec<Option<AsOf<f64>>>> {
        raw_feature(m, lineup_strength_feature)
    }
}
