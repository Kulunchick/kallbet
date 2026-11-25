//! Persistence of Predictions and the per-League Match lock state
//! (`match_locks`: Lock, Reveal and Hard Lock timestamps, set once and never
//! cleared).

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{Executor, FromRow, Sqlite, SqliteConnection};

use crate::domain::lock_reveal::{Stamps, Transition};
use crate::domain::rationale::Rationale;
use crate::domain::{LeagueId, MatchId, TeamId, UserId};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Prediction {
    pub match_id: MatchId,
    pub user_id: UserId,
    pub team_id: TeamId,
    pub updated_at: DateTime<Utc>,
    /// #23: never serialized with the Prediction itself. It leaves the server
    /// only through `api::matches::predictions_view`, once revealed.
    #[serde(skip)]
    pub rationale: Option<Rationale>,
}

/// A `predictions` row as stored: the Rationale is JSON text.
#[derive(FromRow)]
struct Row {
    match_id: MatchId,
    user_id: UserId,
    team_id: TeamId,
    updated_at: DateTime<Utc>,
    rationale_json: Option<String>,
}

impl From<Row> for Prediction {
    fn from(row: Row) -> Prediction {
        Prediction {
            match_id: row.match_id,
            user_id: row.user_id,
            team_id: row.team_id,
            updated_at: row.updated_at,
            // Only the upserts write it, from a validated Rationale; text
            // that does not read back is treated as none.
            rationale: row
                .rationale_json
                .and_then(|json| serde_json::from_str(&json).ok()),
        }
    }
}

fn rationale_json(rationale: Option<&Rationale>) -> Option<String> {
    rationale.map(|r| serde_json::to_string(r).expect("a Rationale serializes"))
}

/// Every Prediction in the League. Only the server sees all of them: what a
/// Member may see is decided by the Reveal state.
pub async fn for_league<'e>(
    db: impl Executor<'e, Database = Sqlite>,
    league: LeagueId,
) -> Result<Vec<Prediction>, sqlx::Error> {
    let rows: Vec<Row> = sqlx::query_as(
        "SELECT match_id, user_id, team_id, updated_at, rationale_json FROM predictions \
         WHERE league_id = ? ORDER BY match_id, updated_at, user_id",
    )
    .bind(league)
    .fetch_all(db)
    .await?;
    Ok(rows.into_iter().map(Prediction::from).collect())
}

/// Creates or changes `user`'s Prediction with its Rationale (replaced as a
/// whole: `None` leaves it without one), unless the Match has a stored
/// Lock in the League: then nothing is written and `None` is returned.
///
/// The check is part of the write (one statement), so a Lock or Reveal stored
/// by anyone after the caller evaluated the League still wins (ADR-0003). The
/// caller evaluates the League at `now` first, so a Lock due by the data it
/// has is stored before this runs.
pub async fn upsert<'e>(
    db: impl Executor<'e, Database = Sqlite>,
    league: LeagueId,
    game: MatchId,
    user: UserId,
    team: TeamId,
    rationale: Option<&Rationale>,
    now: DateTime<Utc>,
) -> Result<Option<Prediction>, sqlx::Error> {
    let row: Option<Row> = sqlx::query_as(
        "INSERT INTO predictions \
           (league_id, match_id, user_id, team_id, updated_at, rationale_json) \
         SELECT ?1, ?2, ?3, ?4, ?5, ?6 WHERE NOT EXISTS ( \
           SELECT 1 FROM match_locks \
           WHERE league_id = ?1 AND match_id = ?2 AND locked_at IS NOT NULL) \
         AND NOT EXISTS (SELECT 1 FROM prediction_edit_windows \
           WHERE league_id = ?1 AND match_id = ?2 AND (user_id != ?3 OR ends_at <= ?5)) \
         ON CONFLICT (league_id, match_id, user_id) DO UPDATE SET \
           team_id = excluded.team_id, updated_at = excluded.updated_at, \
           rationale_json = excluded.rationale_json \
         RETURNING match_id, user_id, team_id, updated_at, rationale_json",
    )
    .bind(league)
    .bind(game)
    .bind(user)
    .bind(team)
    .bind(now)
    .bind(rationale_json(rationale))
    .fetch_optional(db)
    .await?;
    Ok(row.map(Prediction::from))
}

/// Creates or changes a Bot User's Prediction with its Rationale (replaced as
/// a whole, like [`upsert`]), unless the Match has a stored
/// Hard Lock in the League: then nothing is written and `None` is returned.
///
/// The Bot User's counterpart of [`upsert`] (ADR-0003's amendment): the
/// humans' Lock (an early Reveal) and their final edit window do not hold it
/// back, only the Hard Lock does. The check is part of the write, and the
/// caller evaluates the League at `now` first, exactly as for [`upsert`].
pub async fn upsert_for_bot<'e>(
    db: impl Executor<'e, Database = Sqlite>,
    league: LeagueId,
    game: MatchId,
    user: UserId,
    team: TeamId,
    rationale: Option<&Rationale>,
    now: DateTime<Utc>,
) -> Result<Option<Prediction>, sqlx::Error> {
    let row: Option<Row> = sqlx::query_as(
        "INSERT INTO predictions \
           (league_id, match_id, user_id, team_id, updated_at, rationale_json) \
         SELECT ?1, ?2, ?3, ?4, ?5, ?6 WHERE NOT EXISTS ( \
           SELECT 1 FROM match_locks \
           WHERE league_id = ?1 AND match_id = ?2 AND hard_locked_at IS NOT NULL) \
         ON CONFLICT (league_id, match_id, user_id) DO UPDATE SET \
           team_id = excluded.team_id, updated_at = excluded.updated_at, \
           rationale_json = excluded.rationale_json \
         RETURNING match_id, user_id, team_id, updated_at, rationale_json",
    )
    .bind(league)
    .bind(game)
    .bind(user)
    .bind(team)
    .bind(now)
    .bind(rationale_json(rationale))
    .fetch_optional(db)
    .await?;
    Ok(row.map(Prediction::from))
}

/// The stored Lock/Reveal stamps of every Match in the League that has any.
pub async fn stamps<'e>(
    db: impl Executor<'e, Database = Sqlite>,
    league: LeagueId,
) -> Result<HashMap<MatchId, Stamps>, sqlx::Error> {
    type Stamp = Option<DateTime<Utc>>;
    let rows: Vec<(MatchId, Stamp, Stamp, Stamp)> = sqlx::query_as(
        "SELECT match_id, locked_at, revealed_at, hard_locked_at FROM match_locks \
         WHERE league_id = ?",
    )
    .bind(league)
    .fetch_all(db)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(id, locked_at, revealed_at, hard_locked_at)| {
            (
                id,
                Stamps {
                    locked_at,
                    revealed_at,
                    hard_locked_at,
                },
            )
        })
        .collect())
}

/// Persists the transitions of one evaluation. A stamp already stored is
/// never overwritten.
pub async fn record(
    db: &mut SqliteConnection,
    league: LeagueId,
    game: MatchId,
    transitions: &[Transition],
) -> Result<(), sqlx::Error> {
    for transition in transitions {
        let (locked_at, revealed_at, hard_locked_at) = match *transition {
            Transition::Locked(at) => (Some(at), None, None),
            Transition::Revealed(at) => (None, Some(at), None),
            Transition::HardLocked(at) => (None, None, Some(at)),
        };
        sqlx::query(
            "INSERT INTO match_locks (league_id, match_id, locked_at, revealed_at, hard_locked_at) \
             VALUES (?, ?, ?, ?, ?) \
             ON CONFLICT (league_id, match_id) DO UPDATE SET \
               locked_at = COALESCE(match_locks.locked_at, excluded.locked_at), \
               revealed_at = COALESCE(match_locks.revealed_at, excluded.revealed_at), \
               hard_locked_at = COALESCE(match_locks.hard_locked_at, excluded.hard_locked_at)",
        )
        .bind(league)
        .bind(game)
        .bind(locked_at)
        .bind(revealed_at)
        .bind(hard_locked_at)
        .execute(&mut *db)
        .await?;
    }
    Ok(())
}
