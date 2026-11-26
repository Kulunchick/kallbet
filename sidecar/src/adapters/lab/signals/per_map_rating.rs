//! Per-map Elo, updated from played maps of previously completed Matches.
use super::{CandidateSignal, FittedSignal, MatchView, Observed};
use crate::{AsOf, Format, Result, Signal, team_names::team_key};
use std::collections::BTreeMap;

pub struct PerMapRating;

impl CandidateSignal for PerMapRating {
    fn name(&self) -> &'static str {
        "per_map_rating"
    }

    fn description(&self) -> &'static str {
        "Per-map Elo from completed Liquipedia series, averaged over equally likely map sets and blended with the market"
    }

    fn needs(&self) -> &'static str {
        "played maps for both teams and enough distinct historical maps for the series format"
    }

    fn missing_data(&self, archive: &crate::adapters::backtest::ReplayArchive) -> Option<String> {
        archive.map_history.is_empty().then(|| {
            "No played Liquipedia maps linked to completed PandaScore Matches in the archive".into()
        })
    }

    fn fit(&self, _: &[Observed]) -> Result<Box<dyn FittedSignal>> {
        // Elo has a fixed update rate. All updates use the Match's own history.
        Ok(Box::new(PerMapRating))
    }
}

fn expected(a: f64, b: f64) -> f64 {
    1.0 / (1.0 + 10.0_f64.powf((b - a) / 400.0))
}

/// Majority-win probability with independent maps. If the pool contains more
/// maps than the series length, average over all distinct subsets uniformly.
pub fn map_series_probability(pool: &[f64], format: Format) -> Option<f64> {
    let length = match format {
        Format::Bo1 => 1,
        Format::Bo3 => 3,
        Format::Bo5 => 5,
    };
    if pool.len() < length {
        return None;
    }
    // wins[k][w] is the probability of winning w maps in a uniform
    // sample of k maps from the part of the pool seen so far.
    let mut wins = [[0.0; 6]; 6];
    wins[0][0] = 1.0;
    for (seen, &p) in pool.iter().enumerate() {
        for k in (1..=length.min(seen + 1)).rev() {
            let include = k as f64 / (seen + 1) as f64;
            for w in 0..=k {
                let lose = wins[k - 1][w] * (1.0 - p);
                let win = w.checked_sub(1).map_or(0.0, |w| wins[k - 1][w] * p);
                wins[k][w] = (1.0 - include) * wins[k][w] + include * (lose + win);
            }
        }
    }
    Some(
        wins[length][(length / 2 + 1)..=length]
            .iter()
            .sum::<f64>()
            .clamp(0.0, 1.0),
    )
}

impl FittedSignal for PerMapRating {
    fn predict(&self, view: &MatchView) -> Result<Option<AsOf<f64>>> {
        const K: f64 = 32.0;
        let mut ratings = BTreeMap::<&str, BTreeMap<&str, f64>>::new();
        let mut observed = None;
        for entry in view.map_history.at(view.cutoff) {
            let (game, finished_at) = entry?;
            observed = Some(finished_at);
            for (map, a_won) in &game.results {
                let teams = ratings.entry(map).or_default();
                let a = *teams.get(game.team_a.as_str()).unwrap_or(&0.0);
                let b = *teams.get(game.team_b.as_str()).unwrap_or(&0.0);
                let adjustment = K * (f64::from(*a_won) - expected(a, b));
                teams.insert(&game.team_a, a + adjustment);
                teams.insert(&game.team_b, b - adjustment);
            }
        }
        let Some(observed) = observed else {
            return Ok(None);
        };
        let key_a = team_key(&view.game.team_a.name);
        let key_b = team_key(&view.game.team_b.name);
        let mut seen_a = false;
        let mut seen_b = false;
        let mut pool = Vec::new();
        for teams in ratings.values() {
            let a = teams.get(key_a.as_str());
            let b = teams.get(key_b.as_str());
            seen_a |= a.is_some();
            seen_b |= b.is_some();
            if a.is_some() || b.is_some() {
                pool.push(expected(*a.unwrap_or(&0.0), *b.unwrap_or(&0.0)));
            }
        }
        if !seen_a || !seen_b {
            return Ok(None);
        }
        let Some(series) = map_series_probability(&pool, view.game.format) else {
            return Ok(None);
        };
        let map = pool.iter().sum::<f64>() / pool.len() as f64;
        let Some(p) = view.anchor_with(Some(Signal { map, series }))? else {
            return Ok(None);
        };
        let observed = view
            .market
            .as_ref()
            .map_or(observed, |m| observed.max(m.observed_at()));
        Ok(Some(AsOf::new(p, observed, view.cutoff)?))
    }
}
