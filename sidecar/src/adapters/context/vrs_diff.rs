//! Point-in-time VRS roster-diff enricher comparing active Event lineups against
//! the latest strictly pre-cutoff Valve Regional Standings snapshot.

use super::names::team_key;
use crate::*;
use chrono::NaiveDate;
use sqlx::SqlitePool;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone)]
struct VrsRosterSnapshot {
    observed: Instant,
    rosters: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Default)]
pub struct VrsRosterIndex(Vec<VrsRosterSnapshot>);

impl VrsRosterIndex {
    pub fn parse<'a>(documents: impl IntoIterator<Item = &'a str>) -> Result<Self> {
        let mut index = Self::default();
        for doc in documents {
            index.add(doc)?;
        }
        Ok(index)
    }

    pub async fn load_from_db(db: &SqlitePool) -> Result<Self> {
        let docs = sqlx::query_scalar::<_, String>(
            "SELECT markdown FROM vrs_snapshots ORDER BY observed_at ASC",
        )
        .fetch_all(db)
        .await?;
        let mut index = Self::default();
        for doc in docs {
            index.add(&doc)?;
        }
        Ok(index)
    }

    pub fn add(&mut self, doc: &str) -> Result<Instant> {
        let mut lines = doc.trim_start_matches('\u{feff}').lines();
        let header = lines
            .next()
            .and_then(|s| s.strip_prefix("### Standings as of "))
            .ok_or("missing Valve header")?;
        let header = header.split('<').next().unwrap();
        let date = NaiveDate::parse_from_str(header, "%Y_%m_%d")
            .or_else(|_| NaiveDate::parse_from_str(header, "%Y-%m-%d"))?;
        let observed = date
            .and_hms_nano_opt(23, 59, 59, 999_999_999)
            .unwrap()
            .and_utc();
        let mut rosters = BTreeMap::new();
        for row in lines.filter(|s| s.starts_with('|')) {
            let mut cells = row.trim_matches('|').split('|').map(str::trim);
            let rank = cells.next().unwrap_or("");
            if matches!(rank, "Standing" | ":-") {
                continue;
            }
            rank.parse::<usize>()?;
            let _points = cells.next().ok_or("missing points")?;
            let team = cells.next().ok_or("missing team")?;
            let roster_cell = cells.next().ok_or("missing roster")?;
            let players: Vec<String> = roster_cell
                .split(',')
                .map(str::trim)
                .filter(|p| !p.is_empty())
                .map(str::to_string)
                .collect();
            let key = team_key(team);
            if !key.is_empty() && !players.is_empty() {
                rosters.entry(key).or_insert(players);
            }
        }
        if rosters.is_empty() {
            return Err("empty VRS roster snapshot".into());
        }
        self.0.push(VrsRosterSnapshot { observed, rosters });
        self.0.sort_by_key(|s| s.observed);
        Ok(observed)
    }

    /// Returns the team's baseline roster from the latest VRS snapshot strictly before `cutoff`.
    pub fn roster_for(
        &self,
        team: &str,
        aliases: &[String],
        cutoff: Instant,
    ) -> Result<Option<AsOf<Vec<String>>>> {
        let Some(snapshot) = self.0.iter().rev().find(|s| s.observed < cutoff) else {
            return Ok(None);
        };
        let keys: Vec<_> = std::iter::once(team)
            .chain(aliases.iter().map(String::as_str))
            .map(team_key)
            .collect();
        let Some(roster) = snapshot.lookup(&keys) else {
            return Ok(None);
        };
        Ok(Some(AsOf::new(roster.clone(), snapshot.observed, cutoff)?))
    }

    /// Dated observations of a team's five, in chronological order.
    pub fn roster_history(
        &self,
        team: &str,
        aliases: &[String],
        cutoff: Instant,
    ) -> Result<Vec<AsOf<Vec<String>>>> {
        let keys: Vec<_> = std::iter::once(team)
            .chain(aliases.iter().map(String::as_str))
            .map(team_key)
            .collect();
        self.0
            .iter()
            .take_while(|s| s.observed < cutoff)
            .filter_map(|s| Some(AsOf::new(s.lookup(&keys)?.clone(), s.observed, cutoff)))
            .collect::<std::result::Result<_, _>>()
            .map_err(Into::into)
    }

    /// Compares a team's active pre-cutoff Liquipedia roster against the latest pre-cutoff
    /// VRS baseline roster and returns a `VrsRosterDiffFact` wrapped in `AsOf`.
    pub fn diff_team(
        &self,
        event: &AsOf<EventRosters>,
        team: &TeamRoster,
        match_side: &str,
        cutoff: Instant,
    ) -> Result<Option<AsOf<VrsRosterDiffFact>>> {
        let event_data = event.at(cutoff)?;
        let Some(vrs_entry) = self.roster_for(&team.team, &team.aliases, cutoff)? else {
            return Ok(None);
        };
        let vrs_roster = vrs_entry.at(cutoff)?;

        let vrs_lower: BTreeSet<String> = vrs_roster.iter().map(|p| p.to_lowercase()).collect();
        let active_lower: BTreeSet<String> = team
            .active_players
            .iter()
            .map(|p| p.to_lowercase())
            .collect();

        let mut added_players: Vec<String> = team
            .active_players
            .iter()
            .filter(|p| !vrs_lower.contains(&p.to_lowercase()))
            .cloned()
            .collect();
        added_players.sort();

        let mut removed_players: Vec<String> = vrs_roster
            .iter()
            .filter(|p| !active_lower.contains(&p.to_lowercase()))
            .cloned()
            .collect();
        removed_players.sort();

        let changed = !added_players.is_empty() || !removed_players.is_empty();
        let observed_at = event.observed_at().max(vrs_entry.observed_at());
        let fact = VrsRosterDiffFact {
            team: team.team.clone(),
            match_side: match_side.to_string(),
            vrs_as_of: vrs_entry.observed_at(),
            liquipedia_revid: event_data.revid,
            liquipedia_revision_timestamp: event_data.revision_timestamp,
            source_url: event_data.source_url.clone(),
            vrs_roster: vrs_roster.clone(),
            active_roster: team.active_players.clone(),
            added_players,
            removed_players,
            changed,
        };
        Ok(Some(AsOf::new(fact, observed_at, cutoff)?))
    }
}

impl VrsRosterSnapshot {
    fn lookup(&self, keys: &[String]) -> Option<&Vec<String>> {
        keys.iter().find_map(|key| self.rosters.get(key))
    }
}
