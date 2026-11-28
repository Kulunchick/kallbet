//! Each team's rank as known at a cutoff (ticket #57, `rank_gap`).
//!
//! - **VRS rank**: the team's standing in the latest saved Valve global
//!   standings strictly before the cutoff (a date-only publication is known
//!   once its UTC day is over, as `vrs::Standings` reads it). The VRS badge of
//!   the HLTV match page is not used: the audit found it dated by the Match day
//!   and already moving with that Match's own result.
//! - **HLTV rank**: the badge of the HLTV match pages, dated by the weekly
//!   ranking it links to, known once that ranking's UTC day is over (HLTV
//!   publishes it during that Monday), and at most `HLTV_MAX_AGE_DAYS` old.
//!   Any saved page may tell a team's rank (a later page carries the same
//!   earlier ranking), except the pages of Matches the caller hides (the
//!   Holdout's, for a training-side view).
use super::archive::{ReplayArchive, team_key};
use super::player_history::Linker;
use crate::*;
use chrono::{Duration, NaiveDate};
use std::collections::{BTreeMap, BTreeSet};

/// The oldest HLTV ranking still read (five weekly rankings).
pub const HLTV_MAX_AGE_DAYS: i64 = 35;

/// Both teams' ranks (team A, team B) at a Match's cutoff, each pair dated by
/// the later of its two observations.
#[derive(Clone, Debug, Default)]
pub struct Ranks {
    pub vrs: Option<AsOf<(u32, u32)>>,
    pub hltv: Option<AsOf<(u32, u32)>>,
}

#[derive(Clone, Debug)]
struct HltvRank {
    observed: Instant,
    rank: u32,
    /// The PandaScore Match the page belongs to, when linked.
    linked: Option<i64>,
}

#[derive(Default)]
pub struct RankHistory {
    /// Valve snapshots, oldest first: when known, team key → standing.
    vrs: Vec<(Instant, BTreeMap<String, u32>)>,
    /// Team key → HLTV ranks, by (observed, page) ascending.
    hltv: BTreeMap<String, Vec<HltvRank>>,
}

/// The end of a UTC day: when a date-only publication becomes known.
fn end_of(date: NaiveDate) -> Instant {
    date.and_hms_nano_opt(23, 59, 59, 999_999_999).unwrap().and_utc()
}

impl RankHistory {
    pub fn new(archive: &ReplayArchive) -> Result<Self> {
        let mut vrs = vec![];
        for doc in &archive.standings {
            vrs.push(parse_standings(doc)?);
        }
        vrs.sort_by_key(|(at, _)| *at);
        let linker = Linker::new(archive);
        let mut hltv: BTreeMap<String, Vec<(Instant, i64, HltvRank)>> = BTreeMap::new();
        for record in &archive.hltv_matches {
            let Some(badges) = archive.hltv_ranks.get(&record.match_id) else {
                continue;
            };
            let linked = linker.link(record).map(|(id, _)| id);
            for team in badges {
                let name = if team.team_id == record.team_a.id {
                    &record.team_a.name
                } else if team.team_id == record.team_b.id {
                    &record.team_b.name
                } else {
                    continue;
                };
                let Some(badge) = team.hltv else { continue };
                let observed = end_of(badge.date);
                hltv.entry(team_key(name)).or_default().push((
                    observed,
                    record.match_id,
                    HltvRank { observed, rank: badge.rank, linked },
                ));
            }
        }
        let hltv = hltv
            .into_iter()
            .map(|(team, mut rows)| {
                rows.sort_by_key(|(at, page, _)| (*at, *page));
                (team, rows.into_iter().map(|(_, _, r)| r).collect())
            })
            .collect();
        Ok(Self { vrs, hltv })
    }

    /// `team`'s standing in the latest Valve snapshot strictly before `cutoff`.
    pub fn vrs(&self, team: &str, cutoff: Instant) -> Option<AsOf<u32>> {
        let (observed, ranks) = self.vrs.iter().rev().find(|(at, _)| *at < cutoff)?;
        let rank = *ranks.get(&team_key(team))?;
        AsOf::new(rank, *observed, cutoff).ok()
    }

    /// `team`'s latest HLTV rank known strictly before `cutoff`, not older than
    /// `HLTV_MAX_AGE_DAYS`, from no page of a Match in `hidden`. Among pages
    /// showing the same ranking, the first saved page (lowest HLTV id) wins.
    pub fn hltv(
        &self,
        team: &str,
        cutoff: Instant,
        hidden: &BTreeSet<i64>,
    ) -> Result<Option<AsOf<u32>>> {
        let Some(rows) = self.hltv.get(&team_key(team)) else {
            return Ok(None);
        };
        let oldest = cutoff - Duration::days(HLTV_MAX_AGE_DAYS);
        let usable = |r: &&HltvRank| {
            r.observed < cutoff && !r.linked.is_some_and(|id| hidden.contains(&id))
        };
        let Some(latest) = rows.iter().filter(usable).map(|r| r.observed).max() else {
            return Ok(None);
        };
        if latest < oldest {
            return Ok(None);
        }
        let row = rows.iter().filter(usable).find(|r| r.observed == latest).unwrap();
        Ok(Some(AsOf::new(row.rank, row.observed, cutoff)?))
    }

    /// Both teams' ranks for `game` at `cutoff`.
    pub fn at(&self, game: &Match, cutoff: Instant, hidden: &BTreeSet<i64>) -> Result<Ranks> {
        let pair = |a: Option<AsOf<u32>>, b: Option<AsOf<u32>>| -> Result<Option<AsOf<(u32, u32)>>> {
            let (Some(a), Some(b)) = (a, b) else { return Ok(None) };
            let observed = a.observed_at().max(b.observed_at());
            Ok(Some(AsOf::new((*a.at(cutoff)?, *b.at(cutoff)?), observed, cutoff)?))
        };
        let (a, b) = (&game.team_a.name, &game.team_b.name);
        Ok(Ranks {
            vrs: pair(self.vrs(a, cutoff), self.vrs(b, cutoff))?,
            hltv: pair(self.hltv(a, cutoff, hidden)?, self.hltv(b, cutoff, hidden)?)?,
        })
    }
}

/// One Valve global standings document: when it is known, and each team's
/// standing by team key (the first row of a name wins, as in `vrs::Standings`).
fn parse_standings(doc: &str) -> Result<(Instant, BTreeMap<String, u32>)> {
    let mut lines = doc.lines();
    let header = lines
        .next()
        .and_then(|s| s.strip_prefix("### Standings as of "))
        .ok_or("missing Valve header")?;
    let date = header.split('<').next().unwrap_or_default().trim().replace('-', "_");
    let date = NaiveDate::parse_from_str(&date, "%Y_%m_%d")?;
    let mut ranks = BTreeMap::new();
    for row in lines.filter(|s| s.starts_with('|')) {
        let cells: Vec<&str> = row.trim_matches('|').split('|').map(str::trim).collect();
        let (Some(rank), Some(team)) = (cells.first(), cells.get(2)) else {
            continue;
        };
        let Ok(rank) = rank.parse::<u32>() else { continue };
        ranks.entry(team_key(team)).or_insert(rank);
    }
    Ok((end_of(date), ranks))
}
