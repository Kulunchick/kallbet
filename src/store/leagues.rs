//! Persistence of Leagues and memberships.
//!
//! A membership is a stretch of time: `joined_at` to `left_at` (`None` while
//! the User is a current Member). Leaving sets `left_at`; joining again adds a
//! new row. Later tickets compute Eligible Members from `all_memberships`.

use chrono::{DateTime, Utc};
use sqlx::{FromRow, SqlitePool};

use crate::domain::{EventId, LeagueId, Role, UserId};

#[derive(Debug, Clone, FromRow)]
pub struct League {
    pub id: LeagueId,
    pub event_id: EventId,
    pub name: String,
    pub creator_id: UserId,
    pub invite_token: String,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

const LEAGUE_COLUMNS: &str =
    "id, event_id, name, creator_id, invite_token, archived_at, created_at";

#[derive(Debug, Clone, FromRow)]
struct MembershipRow {
    league_id: LeagueId,
    user_id: UserId,
    role: String,
    joined_at: DateTime<Utc>,
    left_at: Option<DateTime<Utc>>,
}

/// One stretch of a User's membership in a League.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Membership {
    pub league_id: LeagueId,
    pub user_id: UserId,
    pub role: Role,
    pub joined_at: DateTime<Utc>,
    pub left_at: Option<DateTime<Utc>>,
}

impl From<MembershipRow> for Membership {
    fn from(row: MembershipRow) -> Membership {
        Membership {
            league_id: row.league_id,
            user_id: row.user_id,
            role: Role::from_code(&row.role).unwrap_or(Role::Member),
            joined_at: row.joined_at,
            left_at: row.left_at,
        }
    }
}

/// A current Member, with what the Member list shows.
#[derive(Debug, Clone)]
pub struct MemberProfile {
    pub user_id: UserId,
    pub nickname: String,
    pub avatar_url: Option<String>,
    pub role: Role,
    pub joined_at: DateTime<Utc>,
}

#[derive(FromRow)]
struct MemberProfileRow {
    user_id: UserId,
    nickname: String,
    avatar_url: Option<String>,
    role: String,
    joined_at: DateTime<Utc>,
}

/// Creates a League and makes `creator` its admin Member, atomically.
pub async fn create(
    db: &SqlitePool,
    event_id: EventId,
    name: &str,
    creator: UserId,
    invite_token: &str,
    now: DateTime<Utc>,
) -> Result<League, sqlx::Error> {
    let mut tx = db.begin().await?;
    let league: League = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "INSERT INTO leagues (event_id, name, creator_id, invite_token, created_at) \
         VALUES (?, ?, ?, ?, ?) RETURNING {LEAGUE_COLUMNS}"
    )))
    .bind(event_id)
    .bind(name)
    .bind(creator)
    .bind(invite_token)
    .bind(now)
    .fetch_one(&mut *tx)
    .await?;
    sqlx::query(
        "INSERT INTO memberships (league_id, user_id, role, joined_at) VALUES (?, ?, 'admin', ?)",
    )
    .bind(league.id)
    .bind(creator)
    .bind(now)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(league)
}

pub async fn find(db: &SqlitePool, id: LeagueId) -> Result<Option<League>, sqlx::Error> {
    sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "SELECT {LEAGUE_COLUMNS} FROM leagues WHERE id = ?"
    )))
    .bind(id)
    .fetch_optional(db)
    .await
}

pub async fn find_by_invite_token(
    db: &SqlitePool,
    token: &str,
) -> Result<Option<League>, sqlx::Error> {
    sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "SELECT {LEAGUE_COLUMNS} FROM leagues WHERE invite_token = ?"
    )))
    .bind(token)
    .fetch_optional(db)
    .await
}

/// Leagues that are not archived, oldest first: the ones the poller
/// evaluates after each pass.
pub async fn active(db: &SqlitePool) -> Result<Vec<League>, sqlx::Error> {
    sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "SELECT {LEAGUE_COLUMNS} FROM leagues WHERE archived_at IS NULL ORDER BY id"
    )))
    .fetch_all(db)
    .await
}

/// Leagues `user` currently belongs to, newest first, with their role.
pub async fn leagues_of(db: &SqlitePool, user: UserId) -> Result<Vec<(League, Role)>, sqlx::Error> {
    #[derive(FromRow)]
    struct Row {
        #[sqlx(flatten)]
        league: League,
        role: String,
    }
    let rows: Vec<Row> = sqlx::query_as(
        "SELECT l.id, l.event_id, l.name, l.creator_id, l.invite_token, l.archived_at, \
                l.created_at, m.role \
         FROM leagues l JOIN memberships m ON m.league_id = l.id \
         WHERE m.user_id = ? AND m.left_at IS NULL \
         ORDER BY l.created_at DESC, l.id DESC",
    )
    .bind(user)
    .fetch_all(db)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| (r.league, Role::from_code(&r.role).unwrap_or(Role::Member)))
        .collect())
}

/// The current membership of `user` in `league`; `None` when not a Member
/// (never joined, left, or was removed).
pub async fn membership(
    db: &SqlitePool,
    league: LeagueId,
    user: UserId,
) -> Result<Option<Membership>, sqlx::Error> {
    let row: Option<MembershipRow> = sqlx::query_as(
        "SELECT league_id, user_id, role, joined_at, left_at FROM memberships \
         WHERE league_id = ? AND user_id = ? AND left_at IS NULL",
    )
    .bind(league)
    .bind(user)
    .fetch_optional(db)
    .await?;
    Ok(row.map(Membership::from))
}

/// Every stretch of membership ever in `league`, including ended ones. The
/// input of the Eligible Member rule.
pub async fn all_memberships(
    db: &SqlitePool,
    league: LeagueId,
) -> Result<Vec<Membership>, sqlx::Error> {
    let rows: Vec<MembershipRow> = sqlx::query_as(
        "SELECT league_id, user_id, role, joined_at, left_at FROM memberships \
         WHERE league_id = ? ORDER BY joined_at, id",
    )
    .bind(league)
    .fetch_all(db)
    .await?;
    Ok(rows.into_iter().map(Membership::from).collect())
}

/// Current Members, in the order they joined.
pub async fn members(db: &SqlitePool, league: LeagueId) -> Result<Vec<MemberProfile>, sqlx::Error> {
    let rows: Vec<MemberProfileRow> = sqlx::query_as(
        "SELECT u.id AS user_id, u.nickname, u.avatar_url, m.role, m.joined_at \
         FROM memberships m JOIN users u ON u.id = m.user_id \
         WHERE m.league_id = ? AND m.left_at IS NULL \
         ORDER BY m.joined_at, m.id",
    )
    .bind(league)
    .fetch_all(db)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| MemberProfile {
            user_id: r.user_id,
            nickname: r.nickname,
            avatar_url: r.avatar_url,
            role: Role::from_code(&r.role).unwrap_or(Role::Member),
            joined_at: r.joined_at,
        })
        .collect())
}

/// Adds a plain Member. Does nothing when `user` is already a current Member.
pub async fn add_member(
    db: &SqlitePool,
    league: LeagueId,
    user: UserId,
    now: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO memberships (league_id, user_id, role, joined_at) VALUES (?, ?, 'member', ?) \
         ON CONFLICT DO NOTHING",
    )
    .bind(league)
    .bind(user)
    .bind(now)
    .execute(db)
    .await?;
    Ok(())
}

/// Ends the current membership, recording the leave time. Returns whether
/// there was one.
pub async fn end_membership(
    db: &SqlitePool,
    league: LeagueId,
    user: UserId,
    now: DateTime<Utc>,
) -> Result<bool, sqlx::Error> {
    let done = sqlx::query(
        "UPDATE memberships SET left_at = ? WHERE league_id = ? AND user_id = ? AND left_at IS NULL",
    )
    .bind(now)
    .bind(league)
    .bind(user)
    .execute(db)
    .await?;
    Ok(done.rows_affected() > 0)
}

/// Archives the League at `now`. The timestamp is set once and never
/// cleared: archiving an archived League keeps the first time. Returns
/// whether this call archived it.
pub async fn archive(
    db: &SqlitePool,
    league: LeagueId,
    now: DateTime<Utc>,
) -> Result<bool, sqlx::Error> {
    let done = sqlx::query("UPDATE leagues SET archived_at = ? WHERE id = ? AND archived_at IS NULL")
        .bind(now)
        .bind(league)
        .execute(db)
        .await?;
    Ok(done.rows_affected() > 0)
}

/// Replaces the Invite Link secret; the old one stops working.
pub async fn set_invite_token(
    db: &SqlitePool,
    league: LeagueId,
    token: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE leagues SET invite_token = ? WHERE id = ?")
        .bind(token)
        .bind(league)
        .execute(db)
        .await?;
    Ok(())
}

/// Sets the role of the current membership. Returns whether there was one.
pub async fn set_role(
    db: &SqlitePool,
    league: LeagueId,
    user: UserId,
    role: Role,
) -> Result<bool, sqlx::Error> {
    let done = sqlx::query(
        "UPDATE memberships SET role = ? WHERE league_id = ? AND user_id = ? AND left_at IS NULL",
    )
    .bind(role.code())
    .bind(league)
    .bind(user)
    .execute(db)
    .await?;
    Ok(done.rows_affected() > 0)
}

/// Deletes a League. Everything that hangs off it goes through
/// `ON DELETE CASCADE` foreign keys, so this is the only statement needed.
pub async fn delete(db: &SqlitePool, league: LeagueId) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM leagues WHERE id = ?")
        .bind(league)
        .execute(db)
        .await?;
    Ok(())
}

pub async fn member_count(db: &SqlitePool, league: LeagueId) -> Result<i64, sqlx::Error> {
    let (n,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM memberships WHERE league_id = ? AND left_at IS NULL")
            .bind(league)
            .fetch_one(db)
            .await?;
    Ok(n)
}
