//! Persistence of Overrides (#13): a League's append-only Override history,
//! and the Overrides in force derived from it.
//!
//! Hand-added Matches themselves are `matches` rows owned by the League
//! (`store::events::added_matches`); their history entry has field `added`.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{Executor, FromRow, Sqlite, SqliteConnection};

use crate::domain::effective_match::{LeagueOverrides, MatchOverrides, ResultOverride};
use crate::domain::{LeagueId, MatchId, TeamId, UserId};
use crate::store::events;

/// What an Override changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Field {
    ScheduledStart,
    Result,
    Teams,
    Hidden,
    /// The Match was added by hand. Never cleared (hide it instead).
    Added,
}

impl Field {
    pub fn code(self) -> &'static str {
        match self {
            Field::ScheduledStart => "scheduled_start",
            Field::Result => "result",
            Field::Teams => "teams",
            Field::Hidden => "hidden",
            Field::Added => "added",
        }
    }

    pub fn from_code(code: &str) -> Option<Field> {
        [
            Field::ScheduledStart,
            Field::Result,
            Field::Teams,
            Field::Hidden,
            Field::Added,
        ]
        .into_iter()
        .find(|f| f.code() == code)
    }
}

/// One action in a League's Override history.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub id: i64,
    pub match_id: MatchId,
    pub field: Field,
    /// This entry removes the field's Override.
    pub cleared: bool,
    pub scheduled_start: Option<DateTime<Utc>>,
    pub winner: Option<TeamId>,
    pub team_a: Option<TeamId>,
    pub team_b: Option<TeamId>,
    /// `None` once the author's account is gone.
    pub author_id: Option<UserId>,
    pub author_nickname: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(FromRow)]
struct Row {
    id: i64,
    match_id: MatchId,
    field: String,
    cleared: bool,
    scheduled_start: Option<DateTime<Utc>>,
    winner: Option<TeamId>,
    team_a: Option<TeamId>,
    team_b: Option<TeamId>,
    author_id: Option<UserId>,
    author_nickname: Option<String>,
    created_at: DateTime<Utc>,
}

/// A new entry; `cleared` entries carry no value.
#[derive(Debug, Clone, Default)]
pub struct NewEntry {
    pub scheduled_start: Option<DateTime<Utc>>,
    pub winner: Option<TeamId>,
    pub team_a: Option<TeamId>,
    pub team_b: Option<TeamId>,
    pub cleared: bool,
}

/// The League's whole Override history, oldest first.
pub async fn history<'e>(
    db: impl Executor<'e, Database = Sqlite>,
    league: LeagueId,
) -> Result<Vec<Entry>, sqlx::Error> {
    let rows: Vec<Row> = sqlx::query_as(
        "SELECT o.id, o.match_id, o.field, o.cleared, o.scheduled_start, o.winner, o.team_a, \
           o.team_b, o.author_id, u.nickname AS author_nickname, o.created_at \
         FROM overrides o LEFT JOIN users u ON u.id = o.author_id \
         WHERE o.league_id = ? ORDER BY o.id",
    )
    .bind(league)
    .fetch_all(db)
    .await?;
    Ok(rows
        .into_iter()
        .filter_map(|r| {
            Some(Entry {
                id: r.id,
                match_id: r.match_id,
                field: Field::from_code(&r.field)?,
                cleared: r.cleared,
                scheduled_start: r.scheduled_start,
                winner: r.winner,
                team_a: r.team_a,
                team_b: r.team_b,
                author_id: r.author_id,
                author_nickname: r.author_nickname,
                created_at: r.created_at,
            })
        })
        .collect())
}

/// Appends one entry to the League's history and returns its id.
pub async fn record<'e>(
    db: impl Executor<'e, Database = Sqlite>,
    league: LeagueId,
    game: MatchId,
    field: Field,
    entry: &NewEntry,
    author: UserId,
    now: DateTime<Utc>,
) -> Result<i64, sqlx::Error> {
    let (id,): (i64,) = sqlx::query_as(
        "INSERT INTO overrides (league_id, match_id, field, cleared, scheduled_start, winner, \
           team_a, team_b, author_id, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?) \
         RETURNING id",
    )
    .bind(league)
    .bind(game)
    .bind(field.code())
    .bind(entry.cleared)
    .bind(entry.scheduled_start)
    .bind(entry.winner)
    .bind(entry.team_a)
    .bind(entry.team_b)
    .bind(author)
    .bind(now)
    .fetch_one(db)
    .await?;
    Ok(id)
}

/// The Overrides in force: for each (Match, field), the latest entry, unless
/// it clears the field.
pub fn in_force(history: &[Entry]) -> HashMap<MatchId, MatchOverrides> {
    let mut latest: HashMap<(MatchId, Field), &Entry> = HashMap::new();
    for entry in history {
        latest.insert((entry.match_id, entry.field), entry);
    }
    let mut result: HashMap<MatchId, MatchOverrides> = HashMap::new();
    for ((game, field), entry) in latest {
        if entry.cleared {
            continue;
        }
        let o = result.entry(game).or_default();
        match field {
            Field::ScheduledStart => o.scheduled_start = entry.scheduled_start,
            Field::Result => {
                o.result = Some(match entry.winner {
                    Some(team) => ResultOverride::Winner(team),
                    None => ResultOverride::Void,
                })
            }
            Field::Teams => {
                if let (Some(a), Some(b)) = (entry.team_a, entry.team_b) {
                    o.teams = Some((a, b));
                }
            }
            Field::Hidden => o.hidden = true,
            Field::Added => {}
        }
    }
    result.retain(|_, o| !o.is_empty());
    result
}

/// Everything the League changed: its hand-added Matches and the Overrides
/// in force. The input of `domain::effective_match::effective_view`.
pub async fn for_league(
    db: &mut SqliteConnection,
    league: LeagueId,
) -> Result<LeagueOverrides, sqlx::Error> {
    let history = history(&mut *db, league).await?;
    Ok(LeagueOverrides {
        added: events::added_matches(&mut *db, league).await?,
        matches: in_force(&history),
    })
}
