//! Persistence of Predictions and the per-League Match lock state
//! (`match_locks`: Lock and Reveal timestamps, set once and never cleared).

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{Executor, FromRow, Sqlite, SqliteConnection};

use crate::domain::lock_reveal::{Stamps, Transition};
use crate::domain::{LeagueId, MatchId, TeamId, UserId};

#[derive(Debug, Clone, PartialEq, Eq, FromRow, Serialize)]
pub struct Prediction {
    pub match_id: MatchId,
    pub user_id: UserId,
    pub team_id: TeamId,
    pub updated_at: DateTime<Utc>,
}

/// Every Prediction in the League. Only the server sees all of them: what a
/// Member may see is decided by the Reveal state.
pub async fn for_league<'e>(
    db: impl Executor<'e, Database = Sqlite>,
    league: LeagueId,
) -> Result<Vec<Prediction>, sqlx::Error> {
    sqlx::query_as(
        "SELECT match_id, user_id, team_id, updated_at FROM predictions \
         WHERE league_id = ? ORDER BY match_id, updated_at, user_id",
    )
    .bind(league)
    .fetch_all(db)
    .await
}

/// Creates or changes `user`'s Prediction, unless the Match has a stored
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
    now: DateTime<Utc>,
) -> Result<Option<Prediction>, sqlx::Error> {
    sqlx::query_as(
        "INSERT INTO predictions (league_id, match_id, user_id, team_id, updated_at) \
         SELECT ?1, ?2, ?3, ?4, ?5 WHERE NOT EXISTS ( \
           SELECT 1 FROM match_locks \
           WHERE league_id = ?1 AND match_id = ?2 AND locked_at IS NOT NULL) \
         AND NOT EXISTS (SELECT 1 FROM prediction_edit_windows \
           WHERE league_id = ?1 AND match_id = ?2 AND (user_id != ?3 OR ends_at <= ?5)) \
         ON CONFLICT (league_id, match_id, user_id) DO UPDATE SET \
           team_id = excluded.team_id, updated_at = excluded.updated_at \
         RETURNING match_id, user_id, team_id, updated_at",
    )
    .bind(league)
    .bind(game)
    .bind(user)
    .bind(team)
    .bind(now)
    .fetch_optional(db)
    .await
}

/// The stored Lock/Reveal stamps of every Match in the League that has any.
pub async fn stamps<'e>(
    db: impl Executor<'e, Database = Sqlite>,
    league: LeagueId,
) -> Result<HashMap<MatchId, Stamps>, sqlx::Error> {
    type Row = (MatchId, Option<DateTime<Utc>>, Option<DateTime<Utc>>);
    let rows: Vec<Row> = sqlx::query_as(
        "SELECT match_id, locked_at, revealed_at FROM match_locks WHERE league_id = ?",
    )
    .bind(league)
    .fetch_all(db)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(id, locked_at, revealed_at)| {
            (
                id,
                Stamps {
                    locked_at,
                    revealed_at,
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
        let (locked_at, revealed_at) = match *transition {
            Transition::Locked(at) => (Some(at), None),
            Transition::Revealed(at) => (None, Some(at)),
        };
        sqlx::query(
            "INSERT INTO match_locks (league_id, match_id, locked_at, revealed_at) \
             VALUES (?, ?, ?, ?) \
             ON CONFLICT (league_id, match_id) DO UPDATE SET \
               locked_at = COALESCE(match_locks.locked_at, excluded.locked_at), \
               revealed_at = COALESCE(match_locks.revealed_at, excluded.revealed_at)",
        )
        .bind(league)
        .bind(game)
        .bind(locked_at)
        .bind(revealed_at)
        .execute(&mut *db)
        .await?;
    }
    Ok(())
}
