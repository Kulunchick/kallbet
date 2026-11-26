//! Candidate Signals `market_source_pinnacle`, `market_source_polymarket`,
//! `market_source_kalshi` and `market_source_average` (spec #33, story 22): the
//! market sources compared and combined, so the Odds Source order is chosen on
//! evidence. Each replaces the market in the anchor's blend with that source's
//! last price before the cutoff (or the mean of the three), and is predicted
//! **only on Matches every source priced**, so the four rows of the report
//! cover the same Matches and differ only in the price they read. Nothing is
//! fitted.
use super::{CandidateSignal, FittedSignal, MatchView, Observed};
use crate::adapters::backtest::{ReplayArchive, feeds::MarketSource};
use crate::*;

#[derive(Clone, Copy)]
pub enum Pick {
    One(MarketSource),
    Average,
}

pub struct SourceSignal(pub Pick);

pub fn pinnacle() -> SourceSignal {
    SourceSignal(Pick::One(MarketSource::OddsPapi))
}
pub fn polymarket() -> SourceSignal {
    SourceSignal(Pick::One(MarketSource::Polymarket))
}
pub fn kalshi() -> SourceSignal {
    SourceSignal(Pick::One(MarketSource::Kalshi))
}
pub fn average() -> SourceSignal {
    SourceSignal(Pick::Average)
}

/// The four Candidate Signals of this module, in report order.
pub fn candidates() -> Vec<std::sync::Arc<dyn CandidateSignal>> {
    vec![
        std::sync::Arc::new(pinnacle()),
        std::sync::Arc::new(polymarket()),
        std::sync::Arc::new(kalshi()),
        std::sync::Arc::new(average()),
    ]
}

/// The last price of each source (in `MarketSource::ALL` order) with when it
/// was observed, or `None` unless every source has one.
pub fn last_prices(m: &MatchView) -> Result<Option<[(f64, Instant); 3]>> {
    let mut out = [(0.0, m.cutoff); 3];
    for (slot, source) in out.iter_mut().zip(MarketSource::ALL) {
        let Some((at, p)) = m.points(source)?.last().copied() else {
            return Ok(None);
        };
        *slot = (p, at);
    }
    Ok(Some(out))
}

impl CandidateSignal for SourceSignal {
    fn name(&self) -> &'static str {
        match self.0 {
            Pick::One(MarketSource::OddsPapi) => "market_source_pinnacle",
            Pick::One(MarketSource::Polymarket) => "market_source_polymarket",
            Pick::One(MarketSource::Kalshi) => "market_source_kalshi",
            Pick::Average => "market_source_average",
        }
    }
    fn description(&self) -> &'static str {
        match self.0 {
            Pick::One(MarketSource::OddsPapi) => {
                "the anchor with Pinnacle's last price before the cutoff as the market, on Matches every source priced"
            }
            Pick::One(MarketSource::Polymarket) => {
                "the anchor with Polymarket's last price before the cutoff as the market, on Matches every source priced"
            }
            Pick::One(MarketSource::Kalshi) => {
                "the anchor with Kalshi's last price before the cutoff as the market, on Matches every source priced"
            }
            Pick::Average => {
                "the anchor with the mean of Pinnacle, Polymarket and Kalshi as the market, on Matches every source priced"
            }
        }
    }
    fn needs(&self) -> &'static str {
        "a price from each of OddsPapi (Pinnacle), Polymarket and Kalshi before the cutoff"
    }
    fn missing_data(&self, archive: &ReplayArchive) -> Option<String> {
        let empty = |is: bool, name: &str| is.then(|| format!("the archive holds no {name} history"));
        empty(archive.histories.is_empty(), "OddsPapi")
            .or_else(|| empty(archive.polymarket_histories.is_empty(), "Polymarket"))
            .or_else(|| empty(archive.kalshi_candles.is_empty(), "Kalshi"))
    }
    fn fit(&self, _training: &[Observed]) -> Result<Box<dyn FittedSignal>> {
        Ok(Box::new(Fitted(self.0)))
    }
}

struct Fitted(Pick);
impl FittedSignal for Fitted {
    fn predict(&self, m: &MatchView) -> Result<Option<AsOf<f64>>> {
        let Some(prices) = last_prices(m)? else {
            return Ok(None);
        };
        let market = match self.0 {
            Pick::One(source) => {
                let at = MarketSource::ALL
                    .iter()
                    .position(|s| *s == source)
                    .unwrap_or(0);
                prices[at].0
            }
            Pick::Average => prices.iter().map(|p| p.0).sum::<f64>() / 3.0,
        };
        let observed = prices
            .iter()
            .map(|p| p.1)
            .chain(m.signal.as_ref().map(AsOf::observed_at))
            .max();
        let (Some(p), Some(observed)) = (m.anchor_with_market(Some(market))?, observed) else {
            return Ok(None);
        };
        Ok(Some(AsOf::new(p, observed, m.cutoff)?))
    }
}
