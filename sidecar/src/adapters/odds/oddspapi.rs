use super::lines::{self, Line, normalize, timestamp};
use crate::{AsOf, Instant, Match, Result};
use serde_json::Value;

pub(super) const FREE_PINNACLE: &str = "pinnacle+30";
const PINNACLE: &str = "pinnacle";
const WINNER: &str = "171";
const PARTICIPANTS: [&str; 2] = ["171", "172"];

/// Parse the pre-match Pinnacle series-winner lines from a tournament batch.
pub fn parse(body: &[u8], game: &Match, cutoff: Instant) -> Result<Option<AsOf<f64>>> {
    lines::quote(&parse_lines(body)?, game, None, cutoff)
}

pub(super) fn parse_lines(body: &[u8]) -> Result<Vec<Line>> {
    let fixtures: Vec<Value> = serde_json::from_slice(body)?;
    Ok(fixtures.iter().filter_map(live_line).collect())
}

fn live_line(fixture: &Value) -> Option<Line> {
    if fixture["statusId"].as_u64()? != 0 || !fixture["trueStartTime"].is_null() {
        return None;
    }
    let start = timestamp(&fixture["startTime"])?;
    let (probabilities, observed_at) = [PINNACLE, FREE_PINNACLE]
        .into_iter()
        .find_map(|book| active_prices(&fixture["bookmakerOdds"][book]))?;
    Some(
        Line {
            teams: fixture_teams(fixture)?,
            start,
            observed_at: Some(observed_at),
            probabilities,
            raw_hash: None,
            raw: None,
        }
        .with_record(fixture),
    )
}

fn fixture_teams(fixture: &Value) -> Option<[String; 2]> {
    Some([
        fixture["participant1Name"].as_str()?.into(),
        fixture["participant2Name"].as_str()?.into(),
    ])
}

fn active_prices(book: &Value) -> Option<([f64; 2], Instant)> {
    let market = &book["markets"][WINNER];
    if !book["bookmakerIsActive"].as_bool()?
        || book["suspended"].as_bool()?
        || !market["marketActive"].as_bool()?
    {
        return None;
    }
    let [a, b] = PARTICIPANTS.map(|id| &market["outcomes"][id]["players"]["0"]);
    if ![a, b]
        .iter()
        .all(|price| price["active"] == true && price["mainLine"] == true)
    {
        return None;
    }
    let observed_at = timestamp(&a["changedAt"])?.max(timestamp(&b["changedAt"])?);
    Some((fair_decimal(a, b)?, observed_at))
}

fn fair_decimal(a: &Value, b: &Value) -> Option<[f64; 2]> {
    normalize(a["price"].as_f64()?.recip(), b["price"].as_f64()?.recip())
}

pub struct SettledFixture {
    id: String,
    side: usize,
    hard_lock: Instant,
}

impl SettledFixture {
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Match the history response to this fixture. Return team A's closing
    /// probability as of the requested cutoff, capped at Hard Lock.
    pub fn quote(&self, body: &[u8], cutoff: Instant) -> Result<Option<AsOf<f64>>> {
        self.quote_parsed(&serde_json::from_slice(body)?, cutoff)
    }

    /// `quote` on a history already parsed.
    pub fn quote_parsed(&self, history: &Value, cutoff: Instant) -> Result<Option<AsOf<f64>>> {
        let cutoff = cutoff.min(self.hard_lock);
        let Some(quote) = history_quote(history, &self.id, cutoff)? else {
            return Ok(None);
        };
        let first = *quote.at(cutoff)?;
        let team_a = if self.side == 0 { first } else { 1.0 - first };
        Ok(Some(AsOf::new(team_a, quote.observed_at(), cutoff)?))
    }

    /// Every pre-cutoff series price of team A, oldest first, capped at Hard Lock
    /// like `quote` (a price point is one tick time at which both sides were active).
    pub fn points(&self, body: &[u8], cutoff: Instant) -> Result<Vec<(Instant, f64)>> {
        self.points_parsed(&serde_json::from_slice(body)?, cutoff)
    }

    /// `points` on a history already parsed.
    pub fn points_parsed(&self, history: &Value, cutoff: Instant) -> Result<Vec<(Instant, f64)>> {
        let cutoff = cutoff.min(self.hard_lock);
        let points = history_points(history, &self.id, cutoff)?;
        Ok(points
            .into_iter()
            .map(|(at, p)| (at, if self.side == 0 { p } else { 1.0 - p }))
            .collect())
    }
}

/// Link a canonical Match to an OddsPapi fixture from the Event's settled
/// `/v4/fixtures` response. Replay can then load that fixture's saved history.
pub fn match_settled_fixture(body: &[u8], game: &Match) -> Result<Option<SettledFixture>> {
    let fixtures: Vec<Value> = serde_json::from_slice(body)?;
    Ok(match_settled_in(&fixtures, game))
}

/// `match_settled_fixture` on a fixture list already parsed.
pub fn match_settled_in(fixtures: &[Value], game: &Match) -> Option<SettledFixture> {
    let scheduled = game.scheduled_start?;
    let candidates: Vec<_> = fixtures
        .iter()
        .filter(|fixture| fixture["statusId"] == 2)
        .filter_map(|fixture| {
            Some((
                fixture["fixtureId"].as_str()?,
                fixture_teams(fixture)?,
                timestamp(&fixture["startTime"])?,
                timestamp(&fixture["trueStartTime"])?,
            ))
        })
        .collect();
    lines::best(game, &candidates, |item| (&item.1, item.2)).map(|(item, side)| SettledFixture {
        id: item.0.into(),
        side,
        hard_lock: crate::hard_lock(scheduled).min(item.3),
    })
}

/// Return the last active pre-cutoff series quote, in OddsPapi participant
/// order. Replay supplies the matched fixture ID and the Match's Hard Lock cutoff.
pub fn parse_history(body: &[u8], fixture_id: &str, cutoff: Instant) -> Result<Option<AsOf<f64>>> {
    history_quote(&serde_json::from_slice(body)?, fixture_id, cutoff)
}

fn history_quote(history: &Value, fixture_id: &str, cutoff: Instant) -> Result<Option<AsOf<f64>>> {
    if history["fixtureId"] != fixture_id {
        return Err("OddsPapi history returned a different fixture".into());
    }
    let latest = |book: &str, outcome: &str| {
        history["bookmakers"][book]["markets"][WINNER]["outcomes"][outcome]["players"]["0"]
            .as_array()?
            .iter()
            .filter_map(|tick| Some((timestamp(&tick["createdAt"])?, tick)))
            .filter(|(time, _)| *time < cutoff)
            .max_by_key(|(time, _)| *time)
            .filter(|(_, tick)| tick["active"] == true)
    };
    let fair = [PINNACLE, FREE_PINNACLE].into_iter().find_map(|book| {
        let (a_time, a) = latest(book, PARTICIPANTS[0])?;
        let (b_time, b) = latest(book, PARTICIPANTS[1])?;
        Some((fair_decimal(a, b)?[0], a_time.max(b_time)))
    });
    Ok(fair
        .map(|(p, time)| AsOf::new(p, time, cutoff))
        .transpose()?)
}

/// Every active pre-cutoff series price (OddsPapi participant order), oldest
/// first: merges both outcome tick streams in time order and emits one point
/// whenever both sides are active, from the first bookmaker that has any.
pub fn parse_history_points(
    body: &[u8],
    fixture_id: &str,
    cutoff: Instant,
) -> Result<Vec<(Instant, f64)>> {
    history_points(&serde_json::from_slice(body)?, fixture_id, cutoff)
}

fn history_points(history: &Value, fixture_id: &str, cutoff: Instant) -> Result<Vec<(Instant, f64)>> {
    if history["fixtureId"] != fixture_id {
        return Err("OddsPapi history returned a different fixture".into());
    }
    for book in [PINNACLE, FREE_PINNACLE] {
        // At each tick timestamp, record the update (Some(price) when active,
        // None when suspended/inactive) for each participant side.
        let mut events: std::collections::BTreeMap<Instant, [Option<Option<f64>>; 2]> =
            std::collections::BTreeMap::new();
        for (side, outcome) in PARTICIPANTS.iter().enumerate() {
            let ticks = history["bookmakers"][book]["markets"][WINNER]["outcomes"][outcome]
                ["players"]["0"]
                .as_array();
            for tick in ticks.into_iter().flatten() {
                let Some(time) = timestamp(&tick["createdAt"]) else {
                    continue;
                };
                if time >= cutoff {
                    continue;
                }
                let active_price = (tick["active"] == true)
                    .then(|| tick["price"].as_f64())
                    .flatten();
                events.entry(time).or_default()[side] = Some(active_price);
            }
        }
        let mut current: [Option<f64>; 2] = [None, None];
        let mut points = Vec::new();
        for (time, update) in events {
            for side in 0..2 {
                if let Some(state) = update[side] {
                    current[side] = state;
                }
            }
            if let [Some(pa), Some(pb)] = current
                && let Some([fair_a, _]) = normalize(pa.recip(), pb.recip())
            {
                points.push((time, fair_a));
            }
        }
        if !points.is_empty() {
            return Ok(points);
        }
    }
    Ok(vec![])
}
