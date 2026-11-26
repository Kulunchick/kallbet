//! Candidate Signal `favourite_longshot` (spec #33, story 22): a correction
//! for favourite-longshot bias. The market's series probability is rescaled in
//! logit space, `q' = sigmoid(gamma * logit(q))`, and the anchor is blended
//! with `q'` in place of `q`. `gamma > 1` marks longshots as overpriced,
//! `gamma < 1` favourites. `gamma` is fitted by maximum likelihood on the series
//! results of the training Matches (those of Events that ended before the
//! evaluated one, so only earlier Events), at least `MIN_FIT` of them with a
//! market price, else no prediction.
use super::line_movement::{logit, sigmoid};
use super::{CandidateSignal, FittedSignal, MatchView, Observed};
use crate::adapters::backtest::ReplayArchive;
use crate::*;

pub const NAME: &str = "favourite_longshot";
pub const MIN_FIT: usize = 30;

pub struct FavouriteLongshot;

/// The maximum-likelihood `gamma` over `(logit of the market's price, team A won)`
/// rows, by golden-section search on `[0.2, 3]`. `None` without rows.
pub fn fit_gamma(rows: &[(f64, bool)]) -> Option<f64> {
    if rows.is_empty() {
        return None;
    }
    let nll = |g: f64| -> f64 {
        rows.iter()
            .map(|(x, won)| {
                let p = sigmoid(g * x);
                -(if *won { p } else { 1.0 - p }).clamp(1e-12, 1.0).ln()
            })
            .sum()
    };
    let ratio = (5f64.sqrt() - 1.0) / 2.0;
    let (mut a, mut b) = (0.2, 3.0);
    let mut c = b - ratio * (b - a);
    let mut d = a + ratio * (b - a);
    let (mut fc, mut fd) = (nll(c), nll(d));
    for _ in 0..100 {
        if fc < fd {
            b = d;
            d = c;
            fd = fc;
            c = b - ratio * (b - a);
            fc = nll(c);
        } else {
            a = c;
            c = d;
            fc = fd;
            d = a + ratio * (b - a);
            fd = nll(d);
        }
    }
    Some((a + b) / 2.0)
}

impl CandidateSignal for FavouriteLongshot {
    fn name(&self) -> &'static str {
        NAME
    }
    fn description(&self) -> &'static str {
        "the market's price rescaled in logit space by a factor fitted walk-forward on earlier \
         Events (favourite-longshot bias), then blended as the anchor does"
    }
    fn needs(&self) -> &'static str {
        "a market price, and at least 30 priced Matches of Events that ended before"
    }
    fn missing_data(&self, archive: &ReplayArchive) -> Option<String> {
        (archive.histories.is_empty()
            && archive.polymarket_histories.is_empty()
            && archive.kalshi_candles.is_empty())
        .then(|| "the archive holds no market price history".into())
    }
    fn fit(&self, training: &[Observed]) -> Result<Box<dyn FittedSignal>> {
        let mut rows = Vec::new();
        for o in training {
            if let Some(q) = o.view.market()? {
                rows.push((logit(q), o.team_a_won));
            }
        }
        let gamma = if rows.len() >= MIN_FIT {
            fit_gamma(&rows)
        } else {
            None
        };
        Ok(Box::new(Fitted { gamma }))
    }
}

struct Fitted {
    gamma: Option<f64>,
}
impl FittedSignal for Fitted {
    fn predict(&self, m: &MatchView) -> Result<Option<AsOf<f64>>> {
        let (Some(gamma), Some(q), Some(observed)) = (self.gamma, m.market()?, m.observed_at())
        else {
            return Ok(None);
        };
        let Some(p) = m.anchor_with_market(Some(sigmoid(gamma * logit(q))))? else {
            return Ok(None);
        };
        Ok(Some(AsOf::new(p, observed, m.cutoff)?))
    }
}
