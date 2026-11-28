//! The external feeds of a replay, read from the archive: dated Valve standings,
//! OddsPapi pre-match history, and Liquipedia Event revisions. Each answers only
//! with data observed strictly before the cutoff it is asked for.
use super::archive::{ReplayArchive, WikiRevision, team_key};
use crate::adapters::{context::liquipedia, odds::oddspapi};
use crate::*;
use chrono::Datelike;
use serde_json::Value;
use std::collections::BTreeMap;

/// `SignalEstimator` over every saved Valve global standings snapshot. Team
/// names are compared by `team_key`, so PandaScore's names find Valve's.
pub struct ReplaySignal(vrs::Standings);
impl ReplaySignal {
    pub fn new(documents: &[String]) -> Result<Self> {
        let mut standings = vrs::Standings::default();
        for doc in documents {
            standings.add(&keyed_standings(doc))?;
        }
        Ok(Self(standings))
    }
}
/// The document with each team name replaced by its key, and a 2024 header
/// (`2024-11-25`) in the `2024_11_25` form `vrs::Standings` reads.
fn keyed_standings(doc: &str) -> String {
    let mut lines = doc.lines();
    let header = lines.next().unwrap_or_default().replace('-', "_");
    let mut out = vec![header];
    for line in lines {
        let cells: Vec<&str> = line.split('|').collect();
        let is_row = cells.len() > 4 && cells[1].trim().parse::<usize>().is_ok();
        if is_row {
            let mut cells: Vec<String> = cells.iter().map(|c| (*c).to_owned()).collect();
            cells[3] = format!(" {} ", team_key(&cells[3]));
            out.push(cells.join("|"));
        } else {
            out.push(line.to_owned());
        }
    }
    out.join("\n")
}
#[async_trait::async_trait]
impl SignalEstimator for ReplaySignal {
    async fn signal(&self, game: &Match, cutoff: Instant) -> Result<Option<AsOf<Signal>>> {
        self.0.estimate(
            &team_key(&game.team_a.name),
            &team_key(&game.team_b.name),
            game.format,
            cutoff,
        )
    }
}

mod history;
pub use history::PriceHistory;

/// The market sources of a replay, in the live chain's order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum MarketSource {
    /// Pinnacle's series-winner line from OddsPapi's historical odds.
    OddsPapi,
    /// The last Polymarket moneyline price.
    Polymarket,
    /// The mid of Kalshi's two "team wins" markets, normalised.
    Kalshi,
}
impl MarketSource {
    pub const ALL: [MarketSource; 3] = [Self::OddsPapi, Self::Polymarket, Self::Kalshi];
    pub fn name(self) -> &'static str {
        match self {
            Self::OddsPapi => "OddsPapi (Pinnacle)",
            Self::Polymarket => "Polymarket",
            Self::Kalshi => "Kalshi",
        }
    }
}

/// `OddsSource` over the saved market histories: OddsPapi, then Polymarket,
/// then Kalshi, as the live chain. Each answers with the last price observed
/// strictly before the cutoff (the histories hold in-play points too), for a
/// listing of the same two teams within six hours of the Scheduled Start.
pub struct ReplayOdds {
    /// The settled fixture lists and their histories, parsed once (a saved
    /// response that is not JSON is the error of every read that needs it).
    lists: Vec<std::result::Result<Vec<Value>, String>>,
    histories: BTreeMap<String, std::result::Result<Value, String>>,
    polymarket: Vec<Value>,
    polymarket_histories: BTreeMap<String, Value>,
    kalshi: Vec<Value>,
    kalshi_candles: BTreeMap<String, Value>,
}
impl ReplayOdds {
    pub fn new(archive: &ReplayArchive) -> Self {
        Self {
            lists: archive
                .fixture_lists
                .iter()
                .map(|l| serde_json::from_slice(l).map_err(|e| e.to_string()))
                .collect(),
            histories: archive
                .histories
                .iter()
                .map(|(id, h)| (id.clone(), serde_json::from_slice(h).map_err(|e| e.to_string())))
                .collect(),
            polymarket: archive.polymarket_events.clone(),
            polymarket_histories: archive.polymarket_histories.clone(),
            kalshi: archive.kalshi_markets.clone(),
            kalshi_candles: archive.kalshi_candles.clone(),
        }
    }

    /// Team A's fair series probability from one source.
    pub fn quote_from(
        &self,
        source: MarketSource,
        game: &Match,
        cutoff: Instant,
    ) -> Result<Option<AsOf<f64>>> {
        match source {
            MarketSource::OddsPapi => self.oddspapi(game, cutoff),
            MarketSource::Polymarket => self.polymarket(game, cutoff),
            MarketSource::Kalshi => self.kalshi(game, cutoff),
        }
    }

    fn oddspapi(&self, game: &Match, cutoff: Instant) -> Result<Option<AsOf<f64>>> {
        for list in &self.lists {
            if let Some(fixture) = oddspapi::match_settled_in(list.as_ref().map_err(|e| e.clone())?, game)
                && let Some(history) = self.histories.get(fixture.id())
            {
                return fixture.quote_parsed(history.as_ref().map_err(|e| e.clone())?, cutoff);
            }
        }
        Ok(None)
    }

    fn polymarket(&self, game: &Match, cutoff: Instant) -> Result<Option<AsOf<f64>>> {
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
            // "2026-11-24 05:00:00+00"
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
                return Ok(None);
            };
            let last = history["history"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|point| {
                    let at = Instant::from_timestamp(point["t"].as_i64()?, 0)?;
                    Some((at, point["p"].as_f64()?))
                })
                .filter(|(at, _)| *at < cutoff)
                .max_by_key(|(at, _)| *at);
            return Ok(match last {
                Some((at, p)) if p > 0.0 && p < 1.0 => {
                    Some(AsOf::new(if side == 0 { p } else { 1.0 - p }, at, cutoff)?)
                }
                _ => None,
            });
        }
        Ok(None)
    }

    fn kalshi(&self, game: &Match, cutoff: Instant) -> Result<Option<AsOf<f64>>> {
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
            let mid = |m: &Value| -> Option<(f64, Instant)> {
                let candles = self.kalshi_candles.get(m["ticker"].as_str()?)?;
                candles["candlesticks"]
                    .as_array()?
                    .iter()
                    .filter_map(|c| {
                        let at = Instant::from_timestamp(c["end_period_ts"].as_i64()?, 0)?;
                        let close = |book: &str| -> Option<f64> {
                            c[book]["close_dollars"].as_str()?.parse().ok()
                        };
                        Some((at, (close("yes_bid")? + close("yes_ask")?) / 2.0))
                    })
                    .filter(|(at, _)| *at < cutoff)
                    .max_by_key(|(at, _)| *at)
                    .map(|(at, p)| (p, at))
            };
            let (Some((a, a_at)), Some((b, b_at))) = (mid(first), mid(second)) else {
                return Ok(None);
            };
            if !(a > 0.0 && b > 0.0) {
                return Ok(None);
            }
            let p_first = a / (a + b);
            let p = if side == 0 { p_first } else { 1.0 - p_first };
            return Ok(Some(AsOf::new(p, a_at.max(b_at), cutoff)?));
        }
        Ok(None)
    }
}

/// Which side of `names` team A is, when `names` are the Match's two teams
/// (by `team_key`) and `start` is within six hours of its Scheduled Start.
fn same_match(game: &Match, names: &[String], start: Option<Instant>) -> Option<usize> {
    let [first, second] = names else {
        return None;
    };
    let scheduled = game.scheduled_start?;
    if (start? - scheduled).abs() > chrono::Duration::hours(6) {
        return None;
    }
    let (a, b) = (team_key(&game.team_a.name), team_key(&game.team_b.name));
    let (x, y) = (team_key(first), team_key(second));
    if a == x && b == y {
        Some(0)
    } else if a == y && b == x {
        Some(1)
    } else {
        None
    }
}

/// The start a Kalshi event ticker names (`KXCS2GAME-26NOV250530CWGOSH`:
/// 2026-11-25 05:30 US Eastern), in UTC.
pub fn kalshi_start(event_ticker: &str) -> Option<Instant> {
    let stamp = event_ticker.split('-').nth(1)?.get(..11)?;
    let local = chrono::NaiveDateTime::parse_from_str(stamp, "%y%b%d%H%M").ok()?;
    // US daylight saving time: second Sunday of March 02:00 to first Sunday of November 02:00.
    let sunday = |month: u32, nth: u32| {
        let first = chrono::NaiveDate::from_ymd_opt(local.year(), month, 1)?;
        let to_sunday = (7 - first.weekday().num_days_from_sunday()) % 7;
        first
            .checked_add_days(chrono::Days::new(u64::from(to_sunday + 7 * (nth - 1))))?
            .and_hms_opt(2, 0, 0)
    };
    let daylight = sunday(3, 2)? <= local && local < sunday(11, 1)?;
    let offset = chrono::Duration::hours(if daylight { 4 } else { 5 });
    Some((local + offset).and_utc())
}

#[async_trait::async_trait]
impl OddsSource for ReplayOdds {
    async fn quote(&self, game: &Match, cutoff: Instant) -> Result<Option<AsOf<f64>>> {
        for source in MarketSource::ALL {
            if let Some(q) = self.quote_from(source, game, cutoff)? {
                return Ok(Some(q));
            }
        }
        Ok(None)
    }
}

/// `ContextEnricher` over saved Liquipedia Event page revisions: the roster
/// facts of both teams from the latest saved revision strictly before the cutoff.
pub struct ReplayContext {
    pages: BTreeMap<i64, Vec<WikiRevision>>,
}
impl ReplayContext {
    pub fn new(archive: &ReplayArchive) -> Self {
        let pages = archive
            .event_titles
            .iter()
            .filter_map(|(event, title)| Some((*event, archive.event_pages.get(title)?.clone())))
            .collect();
        Self { pages }
    }
}
#[async_trait::async_trait]
impl ContextEnricher for ReplayContext {
    async fn facts(&self, game: &Match, cutoff: Instant) -> Result<Vec<AsOf<Value>>> {
        let Some(revision) = self
            .pages
            .get(&game.event_id)
            .and_then(|revs| revs.iter().rev().find(|r| r.timestamp < cutoff))
        else {
            return Ok(vec![]);
        };
        let Some(rosters) = liquipedia::parse_revision_rosters(&revision.body, cutoff)? else {
            return Ok(vec![]);
        };
        let event = rosters.at(cutoff)?;
        let mut facts = Vec::new();
        for (side, team) in [("team_a", &game.team_a), ("team_b", &game.team_b)] {
            let roster = liquipedia::find_team_roster(event, &team.name).or_else(|| {
                let key = team_key(&team.name);
                event.teams.iter().find(|r| {
                    team_key(&r.team) == key || r.aliases.iter().any(|a| team_key(a) == key)
                })
            });
            if let Some(roster) = roster {
                let fact = ContextFact::LiquipediaRoster(liquipedia::build_roster_fact(
                    event, roster, side,
                ));
                facts.push(AsOf::new(fact.to_value()?, rosters.observed_at(), cutoff)?);
            }
        }
        Ok(facts)
    }
}
