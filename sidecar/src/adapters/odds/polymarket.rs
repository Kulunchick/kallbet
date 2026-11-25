use super::lines::{self, Line, normalize, timestamp};
use crate::{AsOf, Instant, Match, Result};
use chrono::DateTime;
use serde_json::Value;

/// Parse open, pre-match series moneylines from a Gamma API events response.
pub fn parse(body: &[u8], game: &Match, cutoff: Instant) -> Result<Option<AsOf<f64>>> {
    lines::quote(&parse_page(body)?.0, game, None, cutoff)
}

pub(super) fn parse_page(body: &[u8]) -> Result<(Vec<Line>, usize)> {
    let events: Vec<Value> = serde_json::from_slice(body)?;
    let lines = events
        .iter()
        .filter(|event| event["closed"] == false && event["live"] != true && event["ended"] != true)
        .filter_map(|event| event["markets"].as_array())
        .flatten()
        .filter_map(moneyline)
        .collect();
    Ok((lines, events.len()))
}

fn moneyline(market: &Value) -> Option<Line> {
    if market["sportsMarketType"] != "moneyline"
        || market["closed"] != false
        || market["active"] != true
        || market["acceptingOrders"] != true
    {
        return None;
    }
    let teams: [String; 2] = serde_json::from_str(market["outcomes"].as_str()?).ok()?;
    let [a, b]: [String; 2] = serde_json::from_str(market["outcomePrices"].as_str()?).ok()?;
    let probabilities = normalize(a.parse().ok()?, b.parse().ok()?)?;
    let start = DateTime::parse_from_str(market["gameStartTime"].as_str()?, "%Y-%m-%d %H:%M:%S%#z")
        .ok()?
        .to_utc();
    Some(
        Line {
            teams,
            start,
            probabilities,
            observed_at: Some(timestamp(&market["updatedAt"])?),
            raw_hash: None,
            raw: None,
        }
        .from_record(market),
    )
}
