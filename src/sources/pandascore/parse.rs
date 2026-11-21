//! The pure PandaScore parser: REST JSON (series, tournaments, matches) to
//! the records of [`ExternalSnapshot`]. No I/O.
//!
//! How PandaScore models an Event (docs, "fundamentals"): a **serie** is one
//! occurrence of a competition, our Event; a **tournament** inside it is a
//! Stage; a **match** belongs to one tournament. Two things PandaScore does
//! not say, and that we therefore derive:
//!
//! - **Stage kind.** `has_bracket` marks a knockout stage (Playoffs). The
//!   docs never mention Swiss; a stage without a bracket is taken as Swiss
//!   when its name says so ("Swiss", or "Stage <n>"), otherwise as Groups.
//!   UNVERIFIED against a real Major response.
//! - **Swiss round and record.** A Match carries neither. They are replayed
//!   from the results in Scheduled Start order: a Match between two known
//!   teams with equal records `w`-`l` is round `w + l + 1`, bucket `w`-`l`.
//!   Anything else (a team still TBD, unequal records) is `Unplaced`.
//!
//! Playoffs slots come from `previous_matches` (present on
//! `/tournaments/{id}/brackets`): a single-elimination tree gets
//! `Playoffs { rounds_before_final, slot }`, anything else is `Unplaced`.

use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::Value;

use crate::domain::{
    MatchFormat, MatchPlacement, MatchScore, MatchStatus, Stage, StageKind, StageOrdinal,
};
use crate::ports::SourceError;
use crate::sources::fallback::{
    ExternalEvent, ExternalMatch, ExternalSnapshot, ExternalTeam, Side,
};

/// One tournament (Stage) with its matches, both as PandaScore returned them.
#[derive(Debug, Clone)]
pub struct TournamentInput {
    pub tournament: Value,
    pub matches: Vec<Value>,
}

fn bad(what: impl std::fmt::Display) -> SourceError {
    SourceError::Parse(format!("PandaScore: {what}"))
}

fn from_value<T: serde::de::DeserializeOwned>(
    value: &Value,
    what: &str,
) -> Result<T, SourceError> {
    T::deserialize(value).map_err(|e| bad(format!("{what}: {e}")))
}

#[derive(Deserialize)]
struct RawSerie {
    id: i64,
    name: Option<String>,
    full_name: Option<String>,
    begin_at: Option<DateTime<Utc>>,
    end_at: Option<DateTime<Utc>>,
}

/// A list of series objects, as the Event list.
pub fn events(series: &Value) -> Result<Vec<ExternalEvent>, SourceError> {
    series
        .as_array()
        .ok_or_else(|| bad("expected a JSON array of series"))?
        .iter()
        .map(event)
        .collect()
}

/// One series object as an Event.
pub fn event(serie: &Value) -> Result<ExternalEvent, SourceError> {
    let raw: RawSerie = from_value(serie, "serie")?;
    let name = raw
        .full_name
        .filter(|n| !n.is_empty())
        .or(raw.name.filter(|n| !n.is_empty()))
        .ok_or_else(|| bad(format!("serie {} has no name", raw.id)))?;
    Ok(ExternalEvent {
        external_id: raw.id.to_string(),
        canonical: None,
        name,
        starts_at: raw.begin_at,
        ends_at: raw.end_at,
    })
}

#[derive(Deserialize)]
struct RawTournament {
    id: i64,
    name: String,
    #[serde(default)]
    has_bracket: bool,
    begin_at: Option<DateTime<Utc>>,
}

/// "Swiss", or "Stage 1" / "Stage 2 ...": the Major's Swiss Stages. Guessed
/// from the name; see the module docs.
fn names_a_swiss_stage(name: &str) -> bool {
    let lower = name.to_lowercase();
    if lower.contains("swiss") {
        return true;
    }
    lower
        .split_whitespace()
        .collect::<Vec<_>>()
        .windows(2)
        .any(|w| {
            w[0] == "stage"
                && w[1]
                    .trim_matches(|c: char| !c.is_alphanumeric())
                    .chars()
                    .all(|c| c.is_ascii_digit())
                && !w[1].trim_matches(|c: char| !c.is_alphanumeric()).is_empty()
        })
}

/// The whole Event: the serie plus its tournaments with their matches.
pub fn snapshot(
    serie: &Value,
    tournaments: &[TournamentInput],
) -> Result<ExternalSnapshot, SourceError> {
    let event = event(serie)?;

    let mut staged: Vec<(RawTournament, &TournamentInput)> = tournaments
        .iter()
        .map(|t| Ok((from_value::<RawTournament>(&t.tournament, "tournament")?, t)))
        .collect::<Result<_, SourceError>>()?;
    staged.sort_by_key(|(raw, _)| (raw.begin_at, raw.id));

    let mut stages = Vec::new();
    let mut matches = Vec::new();
    for (index, (raw, input)) in staged.iter().enumerate() {
        let ordinal = StageOrdinal(
            u8::try_from(index + 1).map_err(|_| bad("more than 255 tournaments in a serie"))?,
        );
        let kind = if raw.has_bracket {
            StageKind::Playoffs
        } else if names_a_swiss_stage(&raw.name) {
            StageKind::Swiss
        } else {
            StageKind::Groups
        };
        stages.push(Stage { ordinal, kind, name: raw.name.clone() });

        let placements = match kind {
            StageKind::Swiss => swiss_placements(&input.matches)?,
            StageKind::Playoffs => playoffs_placements(&input.matches)?,
            StageKind::Groups => vec![MatchPlacement::Unplaced; input.matches.len()],
        };
        for (m, placement) in input.matches.iter().zip(placements) {
            matches.push(match_record(m, ordinal, placement)?);
        }
    }
    Ok(ExternalSnapshot { event, stages, matches })
}

#[derive(Deserialize)]
struct RawMatch {
    id: Option<i64>,
    status: Option<String>,
    match_type: Option<String>,
    number_of_games: Option<u8>,
    scheduled_at: Option<DateTime<Utc>>,
    begin_at: Option<DateTime<Utc>>,
    #[serde(default)]
    opponents: Vec<RawOpponent>,
    #[serde(default)]
    results: Vec<RawResult>,
    winner_id: Option<i64>,
    #[serde(default)]
    forfeit: bool,
}

#[derive(Deserialize)]
struct RawOpponent {
    opponent: RawTeam,
    #[serde(rename = "type")]
    kind: Option<String>,
}

#[derive(Deserialize)]
struct RawTeam {
    id: i64,
    name: String,
}

#[derive(Deserialize)]
struct RawResult {
    score: Option<u8>,
    team_id: i64,
}

/// One match object as a record in the given Stage.
pub fn match_record(
    m: &Value,
    stage: StageOrdinal,
    placement: MatchPlacement,
) -> Result<ExternalMatch, SourceError> {
    let raw: RawMatch = from_value(m, "match")?;
    let id = raw.id.ok_or_else(|| bad("a match has no id"))?;
    let at = |what: &str| format!("match {id}: {what}");

    let mut teams = Vec::new();
    for o in &raw.opponents {
        if o.kind.as_deref().is_some_and(|k| k != "Team") {
            return Err(bad(at("an opponent that is not a team")));
        }
        teams.push(&o.opponent);
    }
    if teams.len() > 2 {
        return Err(bad(at("more than two opponents")));
    }
    let external = |t: Option<&&RawTeam>| {
        t.map(|t| ExternalTeam { external_id: t.id.to_string(), name: t.name.clone(), canonical: None })
    };
    let (team_a, team_b) = (external(teams.first()), external(teams.get(1)));

    let side_of = |team_id: i64| match (teams.first(), teams.get(1)) {
        (Some(a), _) if a.id == team_id => Some(Side::A),
        (_, Some(b)) if b.id == team_id => Some(Side::B),
        _ => None,
    };

    let maps = raw.number_of_games.ok_or_else(|| bad(at("no number_of_games")))?;
    let format = (raw.match_type.as_deref() == Some("best_of"))
        .then(|| MatchFormat::from_maps(maps))
        .flatten()
        .ok_or_else(|| {
            bad(at(&format!(
                "format {:?} with {maps} games is not a Bo1/Bo3/Bo5",
                raw.match_type
            )))
        })?;

    let status = match raw.status.as_deref() {
        Some("not_started") | Some("postponed") => MatchStatus::Upcoming,
        Some("running") => MatchStatus::Live,
        Some("finished") => MatchStatus::Finished,
        // `canceled` without a forfeit: will not be played. A forfeit names a
        // winner (the other side) and counts as played.
        Some("canceled") if raw.forfeit && raw.winner_id.is_some() => MatchStatus::Finished,
        Some("canceled") => MatchStatus::Cancelled,
        other => return Err(bad(at(&format!("unknown status {other:?}")))),
    };
    let postponed = raw.status.as_deref() == Some("postponed");

    // `scheduled_at` is not updated on `postponed`: the time is then unknown.
    let scheduled_start = if postponed { None } else { raw.scheduled_at.or(raw.begin_at) };
    // `begin_at` is the scheduled time until the match runs.
    let actual_start = match status {
        MatchStatus::Live | MatchStatus::Finished => raw.begin_at,
        MatchStatus::Upcoming | MatchStatus::Cancelled => None,
    };

    let winner = match raw.winner_id {
        None => None,
        Some(w) => Some(side_of(w).ok_or_else(|| bad(at("winner_id is not one of the opponents")))?),
    };

    let score = if raw.results.is_empty() || teams.len() < 2 {
        None
    } else {
        let of = |team: &RawTeam| {
            raw.results.iter().find(|r| r.team_id == team.id).and_then(|r| r.score)
        };
        match (of(teams[0]), of(teams[1])) {
            (Some(team_a), Some(team_b)) => Some(MatchScore { team_a, team_b }),
            _ => return Err(bad(at("results do not cover both opponents"))),
        }
    };

    Ok(ExternalMatch {
        external_id: id.to_string(),
        canonical: None,
        stage,
        placement,
        team_a,
        team_b,
        format,
        scheduled_start,
        actual_start,
        status,
        winner,
        score,
    })
}

/// Round and record bucket of each Swiss match (same order as `matches`).
fn swiss_placements(matches: &[Value]) -> Result<Vec<MatchPlacement>, SourceError> {
    struct Row {
        at: Option<DateTime<Utc>>,
        id: i64,
        teams: Option<(i64, i64)>,
        winner: Option<i64>,
        finished: bool,
    }
    let mut rows = Vec::new();
    for m in matches {
        let raw: RawMatch = from_value(m, "match")?;
        let id = raw.id.ok_or_else(|| bad("a match has no id"))?;
        let teams = match raw.opponents.as_slice() {
            [a, b] => Some((a.opponent.id, b.opponent.id)),
            _ => None,
        };
        rows.push(Row {
            at: raw.scheduled_at.or(raw.begin_at),
            id,
            teams,
            winner: raw.winner_id,
            finished: raw.status.as_deref() == Some("finished")
                || (raw.status.as_deref() == Some("canceled") && raw.forfeit),
        });
    }

    let mut order: Vec<usize> = (0..rows.len()).collect();
    order.sort_by_key(|&i| (rows[i].at, rows[i].id));

    let mut records: HashMap<i64, (u8, u8)> = HashMap::new();
    let mut placements = vec![MatchPlacement::Unplaced; rows.len()];
    for i in order {
        let row = &rows[i];
        let Some((a, b)) = row.teams else { continue };
        let (ra, rb) = (
            records.get(&a).copied().unwrap_or_default(),
            records.get(&b).copied().unwrap_or_default(),
        );
        if ra == rb {
            placements[i] = MatchPlacement::Swiss {
                round: ra.0.saturating_add(ra.1).saturating_add(1),
                wins: ra.0,
                losses: ra.1,
            };
        }
        if let (true, Some(w)) = (row.finished, row.winner) {
            let loser = if w == a { b } else { a };
            records.entry(w).or_default().0 += 1;
            records.entry(loser).or_default().1 += 1;
        }
    }
    Ok(placements)
}

/// Bracket slots from `previous_matches`; all `Unplaced` unless the matches
/// form one complete single-elimination tree.
fn playoffs_placements(matches: &[Value]) -> Result<Vec<MatchPlacement>, SourceError> {
    let unplaced = vec![MatchPlacement::Unplaced; matches.len()];

    let mut ids = Vec::new();
    let mut previous: HashMap<i64, Vec<i64>> = HashMap::new();
    for m in matches {
        let Some(id) = m["id"].as_i64() else {
            return Err(bad("a match has no id"));
        };
        let prev: Vec<i64> = m["previous_matches"]
            .as_array()
            .map(|a| a.iter().filter_map(|p| p["match_id"].as_i64()).collect())
            .unwrap_or_default();
        ids.push(id);
        previous.insert(id, prev);
    }
    if previous.values().all(Vec::is_empty) {
        return Ok(unplaced);
    }

    let known: HashSet<i64> = ids.iter().copied().collect();
    let mut referenced = HashSet::new();
    for prev in previous.values() {
        if !(prev.is_empty() || prev.len() == 2) {
            return Ok(unplaced);
        }
        for p in prev {
            if !known.contains(p) || !referenced.insert(*p) {
                return Ok(unplaced);
            }
        }
    }
    let roots: Vec<i64> = ids.iter().copied().filter(|i| !referenced.contains(i)).collect();
    let [root] = roots.as_slice() else { return Ok(unplaced) };

    // Walk down from the final; (depth, slot) per match, leaves must share a depth.
    let mut position: HashMap<i64, (u8, u8)> = HashMap::new();
    let mut leaf_depths = HashSet::new();
    let mut stack = vec![(*root, 0u8, 0u8)];
    while let Some((id, depth, slot)) = stack.pop() {
        position.insert(id, (depth, slot));
        let prev = &previous[&id];
        if prev.is_empty() {
            leaf_depths.insert(depth);
        }
        for (i, child) in prev.iter().enumerate() {
            let child_slot = slot.checked_mul(2).and_then(|s| s.checked_add(i as u8));
            match (depth.checked_add(1), child_slot) {
                (Some(d), Some(s)) => stack.push((*child, d, s)),
                _ => return Ok(unplaced),
            }
        }
    }
    if position.len() != ids.len() || leaf_depths.len() != 1 {
        return Ok(unplaced);
    }
    Ok(ids
        .iter()
        .map(|id| {
            let (rounds_before_final, slot) = position[id];
            MatchPlacement::Playoffs { rounds_before_final, slot }
        })
        .collect())
}
