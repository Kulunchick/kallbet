//! Persistence of Pick'ems (`pickems`, `picks`) and of each Stage Pick'em's
//! Lock (`pickem_locks`, set once, never cleared).

use chrono::{DateTime, Utc};
use sqlx::{Connection, SqliteConnection};

use crate::domain::effective_match::EventView;
use crate::domain::pickem::{self, Pick, PickKind};
use crate::domain::{LeagueId, StageOrdinal, TeamId, UserId};

/// One Member's Pick'em for one Stage.
#[derive(Debug, Clone)]
pub struct StoredPickem {
    pub user_id: UserId,
    pub updated_at: DateTime<Utc>,
    pub picks: Vec<Pick>,
}

/// Creates or replaces `user`'s Pick'em for the Stage, unless the Stage's
/// Pick'em has a stored Lock: then nothing is written and `false` is
/// returned. The caller has validated it.
///
/// The Lock check runs inside the write's transaction, after its first
/// write, so the transaction already holds SQLite's write lock: a Lock
/// stored by anyone after the caller evaluated the League still wins
/// (ADR-0003).
pub async fn save(
    db: &mut SqliteConnection,
    league: LeagueId,
    stage: StageOrdinal,
    user: UserId,
    picks: &[Pick],
    now: DateTime<Utc>,
) -> Result<bool, sqlx::Error> {
    let mut tx = db.begin().await?;
    // Deleting the row cascades to its Picks.
    sqlx::query("DELETE FROM pickems WHERE league_id = ? AND stage_ordinal = ? AND user_id = ?")
        .bind(league)
        .bind(stage.0)
        .bind(user)
        .execute(&mut *tx)
        .await?;
    let locked: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM pickem_locks WHERE league_id = ?1 AND stage_ordinal = ?2)
         OR EXISTS (SELECT 1 FROM pickem_edit_windows WHERE league_id = ?1 AND stage_ordinal = ?2
                    AND (user_id != ?3 OR ends_at <= ?4))",
    )
    .bind(league)
    .bind(stage.0)
    .bind(user)
    .bind(now)
    .fetch_one(&mut *tx)
    .await?;
    if locked {
        tx.rollback().await?;
        return Ok(false);
    }
    sqlx::query(
        "INSERT INTO pickems (league_id, stage_ordinal, user_id, updated_at) VALUES (?, ?, ?, ?)",
    )
    .bind(league)
    .bind(stage.0)
    .bind(user)
    .bind(now)
    .execute(&mut *tx)
    .await?;
    for pick in picks {
        sqlx::query(
            "INSERT INTO picks (league_id, stage_ordinal, user_id, kind, slot, team_id) \
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(league)
        .bind(stage.0)
        .bind(user)
        .bind(pick.kind.code())
        .bind(pick.slot)
        .bind(pick.team)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(true)
}

/// Every Pick'em of the Stage. Only the server sees all of them: Members see
/// others' only after Lock.
pub async fn for_stage(
    db: &mut SqliteConnection,
    league: LeagueId,
    stage: StageOrdinal,
) -> Result<Vec<StoredPickem>, sqlx::Error> {
    let heads: Vec<(UserId, DateTime<Utc>)> = sqlx::query_as(
        "SELECT user_id, updated_at FROM pickems \
         WHERE league_id = ? AND stage_ordinal = ? ORDER BY updated_at, user_id",
    )
    .bind(league)
    .bind(stage.0)
    .fetch_all(&mut *db)
    .await?;
    let rows: Vec<(UserId, String, u8, TeamId)> = sqlx::query_as(
        "SELECT user_id, kind, slot, team_id FROM picks \
         WHERE league_id = ? AND stage_ordinal = ?",
    )
    .bind(league)
    .bind(stage.0)
    .fetch_all(&mut *db)
    .await?;
    Ok(heads
        .into_iter()
        .map(|(user_id, updated_at)| StoredPickem {
            user_id,
            updated_at,
            picks: rows
                .iter()
                .filter(|r| r.0 == user_id)
                .filter_map(|r| {
                    Some(Pick {
                        kind: PickKind::from_code(&r.1)?,
                        slot: r.2,
                        team: r.3,
                    })
                })
                .collect(),
        })
        .collect())
}

pub struct StageState {
    pub locked_at: Option<DateTime<Utc>>,
    pub locks_at: Option<DateTime<Utc>>,
    pub edit_window: Option<(UserId, DateTime<Utc>)>,
    pub members: Vec<UserId>,
    pub submitted: Vec<UserId>,
}

impl StageState {
    pub fn can_edit(&self, user: UserId) -> bool {
        self.locked_at.is_none() && self.edit_window.is_none_or(|(owner, _)| owner == user)
    }

    pub fn is_final_submission(&self, user: UserId) -> bool {
        self.edit_window.is_none()
            && self.members.contains(&user)
            && !self.submitted.contains(&user)
            && self
                .members
                .iter()
                .all(|member| *member == user || self.submitted.contains(member))
    }
}

/// Evaluates and persists the lock under the caller's write transaction.
pub async fn evaluate_lock(
    db: &mut SqliteConnection,
    league: LeagueId,
    view: &EventView,
    stage: StageOrdinal,
    now: DateTime<Utc>,
) -> Result<StageState, sqlx::Error> {
    let stored: Option<DateTime<Utc>> = sqlx::query_scalar(
        "SELECT locked_at FROM pickem_locks WHERE league_id = ? AND stage_ordinal = ?",
    )
    .bind(league)
    .bind(stage.0)
    .fetch_optional(&mut *db)
    .await?;
    let edit_window: Option<(UserId, DateTime<Utc>)> = sqlx::query_as(
        "SELECT user_id, ends_at FROM pickem_edit_windows WHERE league_id = ? AND stage_ordinal = ?",
    )
    .bind(league)
    .bind(stage.0)
    .fetch_optional(&mut *db)
    .await?;
    let (members, submitted): (Vec<_>, Vec<_>) = {
        let rows: Vec<(UserId, bool)> = sqlx::query_as(
            "SELECT m.user_id, p.user_id IS NOT NULL FROM memberships m
             LEFT JOIN pickems p ON p.league_id = m.league_id AND p.user_id = m.user_id
                                AND p.stage_ordinal = ?
             WHERE m.league_id = ? AND m.left_at IS NULL",
        )
        .bind(stage.0)
        .bind(league)
        .fetch_all(&mut *db)
        .await?;
        (
            rows.iter().map(|&(user, _)| user).collect(),
            rows.into_iter()
                .filter_map(|(user, picked)| picked.then_some(user))
                .collect(),
        )
    };
    let early = match edit_window {
        Some((_, ends)) => (ends <= now).then_some(ends),
        None => (!members.is_empty() && members.len() == submitted.len()).then_some(now),
    };
    let locked_at = stored.or_else(|| {
        pickem::lock(view, stage, None, now)
            .into_iter()
            .chain(early)
            .min()
    });
    if stored.is_none()
        && let Some(at) = locked_at
    {
        sqlx::query(
            "INSERT INTO pickem_locks (league_id, stage_ordinal, locked_at) VALUES (?, ?, ?)",
        )
        .bind(league)
        .bind(stage.0)
        .bind(at)
        .execute(&mut *db)
        .await?;
    }
    Ok(StageState {
        locked_at,
        locks_at: locked_at.or_else(|| {
            pickem::expected_lock(view, stage)
                .into_iter()
                .chain(edit_window.map(|(_, ends)| ends))
                .min()
        }),
        edit_window,
        members,
        submitted,
    })
}

pub async fn open_edit_window(
    db: &mut SqliteConnection,
    league: LeagueId,
    stage: StageOrdinal,
    user: UserId,
    now: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO pickem_edit_windows (league_id, stage_ordinal, user_id, ends_at) VALUES (?, ?, ?, ?)",
    )
    .bind(league)
    .bind(stage.0)
    .bind(user)
    .bind(now + crate::domain::lock_reveal::FINAL_EDIT_WINDOW)
    .execute(db)
    .await?;
    Ok(())
}
