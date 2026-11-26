//! Opponent-adjusted Elo with round share as the map-level outcome (#61).
//!
//! A map's signed margin is normalized as (rounds_a - rounds_b) / total_rounds.
//! Mapping it from [-1, 1] to [0, 1] gives team A's round share. We average
//! those shares over a finished Match, so each map counts equally and each
//! Match has equal update weight. Normalization limits the influence of long
//! overtime maps. Ratings start at zero. After each Match, A gains 32 times
//! (mean round share - pre-Match Elo expectation), and B loses the same amount.
//! For prediction, the standard Elo expectation is treated as a map win
//! probability, converted by vrs::series_probability, then blended with the
//! market through the existing anchor.
use super::{CandidateSignal, FittedSignal, MatchView, Observed, per_map_rating::elo_expected};
use crate::{AsOf, Result, Signal, adapters::backtest::ReplayArchive, team_names::team_key, vrs};
use std::collections::BTreeMap;

pub struct RoundDiffRating;

impl CandidateSignal for RoundDiffRating {
    fn name(&self) -> &'static str {
        "round_diff_rating"
    }

    fn description(&self) -> &'static str {
        "Opponent-adjusted Elo updated by mean normalized HLTV map round differential, blended with the market"
    }

    fn needs(&self) -> &'static str {
        "a prior linked Match with HLTV map round scores for at least one team"
    }

    fn added_after_list_fixed(&self) -> bool {
        true
    }

    fn missing_data(&self, archive: &ReplayArchive) -> Option<String> {
        (!archive.veto_history.has_round_scores()).then(|| {
            "No complete HLTV map round scores linked to finished PandaScore Matches".into()
        })
    }

    fn fit(&self, _: &[Observed]) -> Result<Box<dyn FittedSignal>> {
        Ok(Box::new(RoundDiffRating))
    }
}

fn adjustment(a: f64, b: f64, scores: &[(u32, u32)]) -> f64 {
    let share = scores
        .iter()
        .map(|&(a, b)| {
            let (a, b) = (f64::from(a), f64::from(b));
            a / (a + b)
        })
        .sum::<f64>()
        / scores.len() as f64;
    32.0 * (share - elo_expected(a, b))
}

impl FittedSignal for RoundDiffRating {
    fn predict(&self, view: &MatchView) -> Result<Option<AsOf<f64>>> {
        let mut ratings = BTreeMap::new();
        let mut observed = None;
        for entry in view.veto_history.round_scores_at(view.cutoff) {
            let (result, finished) = entry?;
            let [a, b] = result.teams.each_ref().map(String::as_str);
            let rating_a = ratings.get(a).copied().unwrap_or_default();
            let rating_b = ratings.get(b).copied().unwrap_or_default();
            let delta = adjustment(rating_a, rating_b, &result.scores);
            ratings.insert(a, rating_a + delta);
            ratings.insert(b, rating_b - delta);
            observed = Some(finished);
        }
        let a = ratings
            .get(team_key(&view.game.team_a.name).as_str())
            .copied();
        let b = ratings
            .get(team_key(&view.game.team_b.name).as_str())
            .copied();
        let Some(observed) = observed.filter(|_| a.is_some() || b.is_some()) else {
            return Ok(None);
        };
        let map = elo_expected(a.unwrap_or_default(), b.unwrap_or_default());
        let series = vrs::series_probability(map, view.game.format);
        let observed = view
            .market
            .as_ref()
            .map_or(observed, |m| observed.max(m.observed_at()));
        view.anchor_with(Some(Signal { map, series }))?
            .map(|p| AsOf::new(p, observed, view.cutoff))
            .transpose()
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::adjustment;

    #[test]
    fn updates_use_mean_round_share_and_adjust_for_the_opponent() {
        let close = adjustment(0.0, 0.0, &[(13, 11)]);
        assert!((close - 4.0 / 3.0).abs() < 1e-12);
        let two_maps = adjustment(0.0, 0.0, &[(13, 11), (13, 3)]);
        assert!((two_maps - (close + 10.0) / 2.0).abs() < 1e-12);
        assert!(adjustment(150.0, 0.0, &[(13, 11)]) < 0.0);
        let forward = adjustment(150.0, -50.0, &[(13, 11)]);
        let reverse = adjustment(-50.0, 150.0, &[(11, 13)]);
        assert!((forward + reverse).abs() < 1e-12);
    }
}
