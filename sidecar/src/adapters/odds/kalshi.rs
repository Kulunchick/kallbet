use super::lines::{self, Line, normalize, timestamp};
use crate::{AsOf, Instant, Match, Result};
use chrono::DateTime;
use serde::Deserialize;
use serde_json::Value;

pub(super) const SERIES: &str = "KXCS2GAME";

/// Parse one page of the CS2 series-winner events with nested markets.
pub fn parse(
    body: &[u8],
    game: &Match,
    captured_at: Instant,
    cutoff: Instant,
) -> Result<Option<AsOf<f64>>> {
    lines::quote(
        &parse_page(body, captured_at)?.0,
        game,
        Some(captured_at),
        cutoff,
    )
}

#[derive(Deserialize)]
struct Page {
    events: Vec<Value>,
    cursor: String,
}

pub(super) fn parse_page(body: &[u8], captured_at: Instant) -> Result<(Vec<Line>, String)> {
    let page: Page = serde_json::from_slice(body)?;
    let lines = page
        .events
        .iter()
        .filter_map(|item| event(item, captured_at))
        .collect();
    Ok((lines, page.cursor))
}

fn event(event: &Value, captured_at: Instant) -> Option<Line> {
    if event["series_ticker"] != SERIES || event["mutually_exclusive"] != true {
        return None;
    }
    let [a, b] = event["markets"].as_array()?.as_slice() else {
        return None;
    };
    let name = |market: &Value| market["yes_sub_title"].as_str().map(str::to_owned);
    let start = scheduled_start(&a["rules_primary"])?;
    if start != scheduled_start(&b["rules_primary"])? || captured_at >= start {
        return None;
    }
    Some(
        Line {
            teams: [name(a)?, name(b)?],
            start,
            observed_at: Some(captured_at),
            probabilities: normalize(midpoint(a, captured_at)?, midpoint(b, captured_at)?)?,
            raw_hash: None,
            raw: None,
        }
        .with_record(event),
    )
}

fn scheduled_start(rules: &Value) -> Option<Instant> {
    // occurrence_datetime and expiration fields refer to settlement. The rules
    // give the original Scheduled Start, with the explicit ET offset.
    let (_, time) = rules.as_str()?.rsplit_once(" originally scheduled for ")?;
    let (date, _) = time.split_once(", then ")?;
    let (local, zone) = date.rsplit_once(' ')?;
    let offset = match zone {
        "EST" => "-0500",
        "EDT" => "-0400",
        _ => return None,
    };
    DateTime::parse_from_str(&format!("{local} {offset}"), "%b %e, %Y at %I:%M %p %z")
        .ok()
        .map(|time| time.to_utc())
}

fn midpoint(market: &Value, captured_at: Instant) -> Option<f64> {
    if market["status"] != "active"
        || market["result"] != ""
        || timestamp(&market["open_time"])? > captured_at
    {
        return None;
    }
    let bid: f64 = market["yes_bid_dollars"].as_str()?.parse().ok()?;
    let ask: f64 = market["yes_ask_dollars"].as_str()?.parse().ok()?;
    (0.0 < bid && bid <= ask && ask < 1.0).then_some((bid + ask) / 2.0)
}
