//! Persistence of Pick'ems (`pickems`, `picks`), of each Stage Pick'em's
//! Locks (`pickem_locks`: the humans' Lock and the Hard Lock, each set once
//! and never cleared) and of its final edit window (`pickem_edit_windows`).

use chrono::{DateTime, Utc};
use sqlx::{Connection, SqliteConnection};

use crate::domain::effective_match::EventView;
use crate::domain::pickem::{self, LockInputs, Pick, PickKind, Stamps};
use crate::domain::{LeagueId, StageOrdinal, TeamId, UserId};

/// One Member's Pick'em for one Stage.
#[derive(Debug, Clone)]
pub struct StoredPickem {
    pub user_id: UserId,
    pub updated_at: DateTime<Utc>,
    pub picks: Vec<Pick>,
}

/// Whose Lock a write must respect.
#[derive(Clone, Copy)]
enum Writer {
    /// The humans' Lock, and another Member's final edit window.
    Human,
    /// The Hard Lock only.
    BotUser,
}

/// Creates or replaces `user`'s Pick'em for the Stage, unless the Stage's
/// Pick'em has a stored Lock, or its final edit window belongs to someone
/// else or has run out: then nothing is written and `false` is returned.
/// The caller has validated it.
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
    write(db, league, stage, user, picks, now, Writer::Human).await
}

/// The Bot User's counterpart of [`save`] (ADR-0003's amendment): the
/// humans' Lock and their final edit window do not hold it back, only a
/// stored Hard Lock does. The check is part of the write in the same way,
/// and the caller evaluates the Stage's Lock at `now` first, exactly as for
/// [`save`].
pub async fn save_for_bot(
    db: &mut SqliteConnection,
    league: LeagueId,
    stage: StageOrdinal,
    user: UserId,
    picks: &[Pick],
    now: DateTime<Utc>,
) -> Result<bool, sqlx::Error> {
    write(db, league, stage, user, picks, now, Writer::BotUser).await
}

async fn write(
    db: &mut SqliteConnection,
    league: LeagueId,
    stage: StageOrdinal,
    user: UserId,
    picks: &[Pick],
    now: DateTime<Utc>,
    writer: Writer,
) -> Result<bool, sqlx::Error> {
    let mut tx = db.begin().await?;
    // Deleting the row cascades to its Picks.
    sqlx::query("DELETE FROM pickems WHERE league_id = ? AND stage_ordinal = ? AND user_id = ?")
        .bind(league)
        .bind(stage.0)
        .bind(user)
        .execute(&mut *tx)
        .await?;
    let locked: bool = match writer {
        Writer::Human => {
            sqlx::query_scalar(
                "SELECT EXISTS (SELECT 1 FROM pickem_locks
                                WHERE league_id = ?1 AND stage_ordinal = ?2)
                 OR EXISTS (SELECT 1 FROM pickem_edit_windows
                            WHERE league_id = ?1 AND stage_ordinal = ?2
                              AND (user_id != ?3 OR ends_at <= ?4))",
            )
            .bind(league)
            .bind(stage.0)
            .bind(user)
            .bind(now)
            .fetch_one(&mut *tx)
            .await?
        }
        Writer::BotUser => {
            sqlx::query_scalar(
                "SELECT EXISTS (SELECT 1 FROM pickem_locks
                                WHERE league_id = ?1 AND stage_ordinal = ?2
                                  AND hard_locked_at IS NOT NULL)",
            )
            .bind(league)
            .bind(stage.0)
            .fetch_one(&mut *tx)
            .await?
        }
    };
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

/// A Stage Pick'em's Locks at one moment, and who an early Lock waits for.
pub struct StageState {
    /// The humans' Lock: their early Lock or the Hard Lock, whichever came
    /// first.
    pub locked_at: Option<DateTime<Utc>>,
    /// When the humans' Lock is expected (or happened), for a countdown.
    pub locks_at: Option<DateTime<Utc>>,
    /// The Hard Lock (the Stage's first Match starting), which binds Bot
    /// Users too.
    pub hard_locked_at: Option<DateTime<Utc>>,
    /// When the Hard Lock is expected (or happened): a Bot User's countdown.
    pub hard_locks_at: Option<DateTime<Utc>>,
    /// The final edit window: its owner (the last human to submit) and end.
    pub edit_window: Option<(UserId, DateTime<Utc>)>,
    /// The human current Members: the ones an early Lock waits for, and the
    /// only ones who can hold the final edit window.
    pub awaited: Vec<UserId>,
    /// The current Members, Bot Users included, who have submitted.
    pub submitted: Vec<UserId>,
}

impl StageState {
    /// Whether a human Member may write. A Bot User asks
    /// [`Self::is_hard_locked`].
    pub fn can_edit(&self, user: UserId) -> bool {
        self.locked_at.is_none() && self.edit_window.is_none_or(|(owner, _)| owner == user)
    }

    /// Locked for everyone, Bot Users included.
    pub fn is_hard_locked(&self) -> bool {
        self.hard_locked_at.is_some()
    }

    /// The Lock that binds a Member: the Hard Lock for a Bot User, the
    /// humans' Lock for anyone else. A Member's Pick'em counts if they
    /// joined by it.
    pub fn lock_of(&self, is_bot: bool) -> Option<DateTime<Utc>> {
        if is_bot {
            self.hard_locked_at
        } else {
            self.locked_at
        }
    }

    /// Whether `user`'s first Pick'em completes the humans' set, which opens
    /// the final edit window for them. Never true for a Bot User.
    pub fn is_final_submission(&self, user: UserId) -> bool {
        self.edit_window.is_none()
            && self.awaited.contains(&user)
            && !self.submitted.contains(&user)
            && self
                .awaited
                .iter()
                .all(|member| *member == user || self.submitted.contains(member))
    }
}

/// Evaluates the Stage Pick'em's Locks ([`pickem::evaluate_lock`]) and
/// stores any new stamp, under the caller's write transaction.
pub async fn evaluate_lock(
    db: &mut SqliteConnection,
    league: LeagueId,
    view: &EventView,
    stage: StageOrdinal,
    now: DateTime<Utc>,
) -> Result<StageState, sqlx::Error> {
    type Stamp = Option<DateTime<Utc>>;
    let row: Option<(Stamp, Stamp)> = sqlx::query_as(
        "SELECT locked_at, hard_locked_at FROM pickem_locks \
         WHERE league_id = ? AND stage_ordinal = ?",
    )
    .bind(league)
    .bind(stage.0)
    .fetch_optional(&mut *db)
    .await?;
    let stored = row
        .map(|(locked_at, hard_locked_at)| Stamps {
            locked_at,
            hard_locked_at,
        })
        .unwrap_or_default();
    let edit_window: Option<(UserId, DateTime<Utc>)> = sqlx::query_as(
        "SELECT user_id, ends_at FROM pickem_edit_windows WHERE league_id = ? AND stage_ordinal = ?",
    )
    .bind(league)
    .bind(stage.0)
    .fetch_optional(&mut *db)
    .await?;
    // The current Members: whether each is a Bot User, and has submitted.
    let members: Vec<(UserId, bool, bool)> = sqlx::query_as(
        "SELECT m.user_id, u.is_bot, p.user_id IS NOT NULL FROM memberships m
         JOIN users u ON u.id = m.user_id
         LEFT JOIN pickems p ON p.league_id = m.league_id AND p.user_id = m.user_id
                            AND p.stage_ordinal = ?
         WHERE m.league_id = ? AND m.left_at IS NULL",
    )
    .bind(stage.0)
    .bind(league)
    .fetch_all(&mut *db)
    .await?;
    let awaited: Vec<UserId> = members
        .iter()
        .filter_map(|&(user, is_bot, _)| (!is_bot).then_some(user))
        .collect();
    let submitted: Vec<UserId> = members
        .iter()
        .filter_map(|&(user, _, submitted)| submitted.then_some(user))
        .collect();

    let stamps = pickem::evaluate_lock(&LockInputs {
        view,
        stage,
        stored,
        awaited: &awaited,
        submitted: &submitted,
        edit_until: edit_window.map(|(_, ends)| ends),
        now,
    });
    // A stamp already stored is never overwritten.
    if stamps != stored
        && let Some(locked_at) = stamps.locked_at
    {
        sqlx::query(
            "INSERT INTO pickem_locks (league_id, stage_ordinal, locked_at, hard_locked_at) \
             VALUES (?, ?, ?, ?) \
             ON CONFLICT (league_id, stage_ordinal) DO UPDATE SET \
               hard_locked_at = COALESCE(hard_locked_at, excluded.hard_locked_at)",
        )
        .bind(league)
        .bind(stage.0)
        .bind(locked_at)
        .bind(stamps.hard_locked_at)
        .execute(&mut *db)
        .await?;
    }
    let hard_locks_at = stamps
        .hard_locked_at
        .or_else(|| pickem::expected_lock(view, stage));
    Ok(StageState {
        locked_at: stamps.locked_at,
        locks_at: stamps.locked_at.or_else(|| {
            hard_locks_at
                .into_iter()
                .chain(edit_window.map(|(_, ends)| ends))
                .min()
        }),
        hard_locked_at: stamps.hard_locked_at,
        hard_locks_at,
        edit_window,
        awaited,
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
