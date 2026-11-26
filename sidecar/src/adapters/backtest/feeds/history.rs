//! Ticket #38 (spec #33): the price history of one Match from one market
//! source, for the Candidate Signals that compare sources or read the line's
//! movement. Same matching as `ReplayOdds::quote_from`; only points observed
//! strictly before the cutoff are returned (the saved histories hold in-play
//! points too), each carried as an `AsOf` observed at its own time.
use super::{MarketSource, ReplayOdds, kalshi_start, same_match};
use crate::adapters::odds::oddspapi;
use crate::*;
use serde_json::Value;
use std::collections::BTreeMap;

/// A market's price points for team A's series win, oldest first.
#[async_trait::async_trait]
pub trait PriceHistory: Send + Sync {
    async fn points(
        &self,
        source: MarketSource,
        game: &Match,
        cutoff: Instant,
    ) -> Result<Vec<AsOf<f64>>>;
}

#[async_trait::async_trait]
impl PriceHistory for ReplayOdds {
    async fn points(
        &self,
        source: MarketSource,
        game: &Match,
        cutoff: Instant,
    ) -> Result<Vec<AsOf<f64>>> {
        self.points_from(source, game, cutoff)
    }
}

impl ReplayOdds {
    /// Every price point of `source` observed strictly before `cutoff`, oldest first.
    pub fn points_from(
        &self,
        source: MarketSource,
        game: &Match,
        cutoff: Instant,
    ) -> Result<Vec<AsOf<f64>>> {
        let raw = match source {
            MarketSource::OddsPapi => self.oddspapi_points(game, cutoff)?,
            MarketSource::Polymarket => self.polymarket_points(game, cutoff),
            MarketSource::Kalshi => self.kalshi_points(game, cutoff),
        };
        let mut raw: Vec<_> = raw
            .into_iter()
            .filter(|(at, p)| *at < cutoff && *p > 0.0 && *p < 1.0)
            .collect();
        raw.sort_by_key(|(at, _)| *at);
        raw.into_iter()
            .map(|(at, p)| Ok(AsOf::new(p, at, cutoff)?))
            .collect()
    }

    fn oddspapi_points(&self, game: &Match, cutoff: Instant) -> Result<Vec<(Instant, f64)>> {
        for list in &self.lists {
            if let Some(fixture) = oddspapi::match_settled_fixture(list, game)?
                && let Some(body) = self.histories.get(fixture.id())
            {
                return fixture.points(body, cutoff);
            }
        }
        Ok(vec![])
    }

    fn polymarket_points(&self, game: &Match, cutoff: Instant) -> Vec<(Instant, f64)> {
        for event in &self.polymarket {
            let Some(line) = event["markets"]
                .as_array()
                .into_iter()
                .flatten()
                .find(|m| m["sportsMarketType"] == "moneyline")
            else {
                continue;
            };
            let outcomes: Vec<String> = line["outcomes"]
                .as_str()
                .and_then(|o| serde_json::from_str(o).ok())
                .unwrap_or_default();
            let start = line["gameStartTime"].as_str().and_then(|t| {
                chrono::DateTime::parse_from_str(&format!("{t}00"), "%Y-%m-%d %H:%M:%S%z").ok()
            });
            let start = start.map(|t| t.with_timezone(&chrono::Utc));
            let Some(side) = same_match(game, &outcomes, start) else {
                continue;
            };
            let Some(history) = event["slug"]
                .as_str()
                .and_then(|s| self.polymarket_histories.get(s))
            else {
                return vec![];
            };
            let mut by_time: BTreeMap<Instant, f64> = BTreeMap::new();
            for point in history["history"].as_array().into_iter().flatten() {
                let (Some(at), Some(p)) = (
                    point["t"]
                        .as_i64()
                        .and_then(|t| Instant::from_timestamp(t, 0)),
                    point["p"].as_f64(),
                ) else {
                    continue;
                };
                if at < cutoff && p > 0.0 && p < 1.0 {
                    by_time.insert(at, if side == 0 { p } else { 1.0 - p });
                }
            }
            return by_time.into_iter().collect();
        }
        vec![]
    }

    fn kalshi_points(&self, game: &Match, cutoff: Instant) -> Vec<(Instant, f64)> {
        let mut events: BTreeMap<&str, Vec<&Value>> = BTreeMap::new();
        for m in &self.kalshi {
            if let Some(e) = m["event_ticker"].as_str() {
                events.entry(e).or_default().push(m);
            }
        }
        for (event, markets) in events {
            let [first, second] = markets[..] else {
                continue;
            };
            let names =
                [first, second].map(|m| m["yes_sub_title"].as_str().unwrap_or("").to_owned());
            let Some(side) = same_match(game, &names, kalshi_start(event)) else {
                continue;
            };
            let mids = |m: &Value| -> BTreeMap<Instant, f64> {
                let Some(candles) = m["ticker"].as_str().and_then(|t| self.kalshi_candles.get(t))
                else {
                    return BTreeMap::new();
                };
                candles["candlesticks"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(|c| {
                        let at = Instant::from_timestamp(c["end_period_ts"].as_i64()?, 0)?;
                        let close = |book: &str| -> Option<f64> {
                            c[book]["close_dollars"].as_str()?.parse().ok()
                        };
                        Some((at, (close("yes_bid")? + close("yes_ask")?) / 2.0))
                    })
                    .filter(|(at, _)| *at < cutoff)
                    .collect()
            };
            let (a, b) = (mids(first), mids(second));
            let mut merged: BTreeMap<Instant, [Option<f64>; 2]> = BTreeMap::new();
            for (at, v) in a {
                merged.entry(at).or_default()[0] = Some(v);
            }
            for (at, v) in b {
                merged.entry(at).or_default()[1] = Some(v);
            }
            let mut current: [Option<f64>; 2] = [None, None];
            let mut out = Vec::new();
            for (at, update) in merged {
                for s in 0..2 {
                    if let Some(v) = update[s] {
                        current[s] = Some(v);
                    }
                }
                if let [Some(va), Some(vb)] = current
                    && va > 0.0
                    && vb > 0.0
                {
                    let first_side = va / (va + vb);
                    out.push((at, if side == 0 { first_side } else { 1.0 - first_side }));
                }
            }
            return out;
        }
        vec![]
    }
}
