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
        .from_record(fixture),
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
        let cutoff = cutoff.min(self.hard_lock);
        let Some(quote) = parse_history(body, &self.id, cutoff)? else {
            return Ok(None);
        };
        let first = *quote.at(cutoff)?;
        let team_a = if self.side == 0 { first } else { 1.0 - first };
        Ok(Some(AsOf::new(team_a, quote.observed_at(), cutoff)?))
    }
}

/// Link a canonical Match to an OddsPapi fixture from the Event's settled
/// `/v4/fixtures` response. Replay can then load that fixture's saved history.
pub fn match_settled_fixture(body: &[u8], game: &Match) -> Result<Option<SettledFixture>> {
    let Some(scheduled) = game.scheduled_start else {
        return Ok(None);
    };
    let fixtures: Vec<Value> = serde_json::from_slice(body)?;
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
    Ok(
        lines::best(game, &candidates, |item| (&item.1, item.2)).map(|(item, side)| {
            SettledFixture {
                id: item.0.into(),
                side,
                hard_lock: crate::hard_lock(scheduled).min(item.3),
            }
        }),
    )
}

/// Return the last active pre-cutoff series quote, in OddsPapi participant
/// order. Replay supplies the matched fixture ID and the Match's Hard Lock cutoff.
pub fn parse_history(body: &[u8], fixture_id: &str, cutoff: Instant) -> Result<Option<AsOf<f64>>> {
    let history: Value = serde_json::from_slice(body)?;
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
