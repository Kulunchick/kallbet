//! Persistence of Events, Stages, teams and the base Match data.
//!
//! #4 stores the Event row a League points at; #5 adds Stages, teams and
//! Matches (written by the poller from an [`EventSnapshot`]). League logic
//! reads Matches through `crate::league_matches::effective_view`, never
//! through [`base_view`] directly.

use chrono::{DateTime, Utc};
use sqlx::{Connection, Executor, FromRow, Sqlite, SqliteConnection, SqlitePool};

use crate::domain::effective_match::{EventView, SourcedMatch};
use crate::domain::{
    DataSourceKind, Event, EventId, EventSnapshot, Match, MatchFormat, MatchId, MatchPlacement,
    MatchScore, MatchStatus, Stage, StageKind, StageOrdinal, Team, TeamId,
};

#[derive(FromRow)]
struct EventRow {
    id: EventId,
    name: String,
    starts_at: Option<DateTime<Utc>>,
    ends_at: Option<DateTime<Utc>>,
}

impl From<EventRow> for Event {
    fn from(row: EventRow) -> Event {
        Event {
            id: row.id,
            name: row.name,
            starts_at: row.starts_at,
            ends_at: row.ends_at,
        }
    }
}

/// Stores the Event, or refreshes its name and dates when already known.
pub async fn upsert(db: &SqlitePool, event: &Event) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO events (id, name, starts_at, ends_at) VALUES (?, ?, ?, ?) \
         ON CONFLICT (id) DO UPDATE SET name = excluded.name, \
           starts_at = excluded.starts_at, ends_at = excluded.ends_at",
    )
    .bind(event.id)
    .bind(&event.name)
    .bind(event.starts_at)
    .bind(event.ends_at)
    .execute(db)
    .await?;
    Ok(())
}

/// Stores the Event list a Data Source returned, in its order, replacing the
/// previous one; each Event is upserted too.
pub async fn save_list(db: &SqlitePool, listed: &[Event]) -> Result<(), sqlx::Error> {
    let mut tx = db.begin().await?;
    sqlx::query("DELETE FROM event_list").execute(&mut *tx).await?;
    for (position, event) in listed.iter().enumerate() {
        sqlx::query(
            "INSERT INTO events (id, name, starts_at, ends_at) VALUES (?, ?, ?, ?) \
             ON CONFLICT (id) DO UPDATE SET name = excluded.name, \
               starts_at = excluded.starts_at, ends_at = excluded.ends_at",
        )
        .bind(event.id)
        .bind(&event.name)
        .bind(event.starts_at)
        .bind(event.ends_at)
        .execute(&mut *tx)
        .await?;
        sqlx::query("INSERT OR REPLACE INTO event_list (event_id, position) VALUES (?, ?)")
            .bind(event.id)
            .bind(position as i64)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await
}

/// The Event list last stored by [`save_list`], in the source's order
/// (empty if none was stored yet, or the source listed nothing).
pub async fn stored_list(db: &SqlitePool) -> Result<Vec<Event>, sqlx::Error> {
    let rows: Vec<EventRow> = sqlx::query_as(
        "SELECT e.id, e.name, e.starts_at, e.ends_at FROM event_list l \
         JOIN events e ON e.id = l.event_id ORDER BY l.position",
    )
    .fetch_all(db)
    .await?;
    Ok(rows.into_iter().map(Event::from).collect())
}

pub async fn find<'e>(
    db: impl Executor<'e, Database = Sqlite>,
    id: EventId,
) -> Result<Option<Event>, sqlx::Error> {
    let row: Option<EventRow> =
        sqlx::query_as("SELECT id, name, starts_at, ends_at FROM events WHERE id = ?")
            .bind(id)
            .fetch_optional(db)
            .await?;
    Ok(row.map(Event::from))
}

/// Events that have at least one League that is not archived: the only ones
/// the poller fetches.
pub async fn with_active_leagues(db: &SqlitePool) -> Result<Vec<EventId>, sqlx::Error> {
    let rows: Vec<(EventId,)> = sqlx::query_as(
        "SELECT DISTINCT event_id FROM leagues WHERE archived_at IS NULL ORDER BY event_id",
    )
    .fetch_all(db)
    .await?;
    Ok(rows.into_iter().map(|(id,)| id).collect())
}

/// When the poller last tried `key` (`"event_list"`, `"event:<id>"`).
pub async fn last_polled(db: &SqlitePool, key: &str) -> Result<Option<DateTime<Utc>>, sqlx::Error> {
    let row: Option<(DateTime<Utc>,)> = sqlx::query_as("SELECT polled_at FROM polls WHERE key = ?")
        .bind(key)
        .fetch_optional(db)
        .await?;
    Ok(row.map(|(at,)| at))
}

pub async fn record_poll(db: &SqlitePool, key: &str, at: DateTime<Utc>) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO polls (key, polled_at) VALUES (?, ?) \
         ON CONFLICT (key) DO UPDATE SET polled_at = excluded.polled_at",
    )
    .bind(key)
    .bind(at)
    .execute(db)
    .await?;
    Ok(())
}

/// Stores everything in one Event snapshot: the Event, its Stages, teams and
/// Matches. Matches are inserted or updated, never deleted. `now` becomes the
/// Matches' `updated_at`, and the `actual_start` of a Match first seen live
/// (or finished) when the Data Source does not report one.
pub async fn save_snapshot(
    db: &mut SqliteConnection,
    snapshot: &EventSnapshot,
    now: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    let mut tx = db.begin().await?;
    let event = &snapshot.event;
    sqlx::query(
        "INSERT INTO events (id, name, starts_at, ends_at) VALUES (?, ?, ?, ?) \
         ON CONFLICT (id) DO UPDATE SET name = excluded.name, \
           starts_at = excluded.starts_at, ends_at = excluded.ends_at",
    )
    .bind(event.id)
    .bind(&event.name)
    .bind(event.starts_at)
    .bind(event.ends_at)
    .execute(&mut *tx)
    .await?;
    move_stage_data(&mut tx, snapshot).await?;

    for stage in &snapshot.stages {
        sqlx::query(
            "INSERT INTO stages (event_id, ordinal, kind, name) VALUES (?, ?, ?, ?) \
             ON CONFLICT (event_id, ordinal) DO UPDATE SET kind = excluded.kind, name = excluded.name",
        )
        .bind(event.id)
        .bind(stage.ordinal.0)
        .bind(stage_kind_code(stage.kind))
        .bind(&stage.name)
        .execute(&mut *tx)
        .await?;
    }

    for team in &snapshot.teams {
        sqlx::query(
            "INSERT INTO teams (id, name, logo_url) VALUES (?, ?, ?) \
             ON CONFLICT (id) DO UPDATE SET name = excluded.name, \
               logo_url = COALESCE(excluded.logo_url, teams.logo_url)",
        )
        .bind(team.id)
        .bind(&team.name)
        .bind(&team.logo_url)
        .execute(&mut *tx)
        .await?;
    }

    for m in &snapshot.matches {
        let placement = PlacementColumns::of(m.placement);
        let first_seen_started = m.has_started().then_some(now);
        let score = m.score;
        sqlx::query(
            "INSERT INTO matches (id, event_id, stage_ordinal, placement_kind, swiss_round, \
               swiss_wins, swiss_losses, playoff_rounds_before_final, playoff_slot, team_a, \
               team_b, format, scheduled_start, actual_start, status, winner, score_a, score_b, \
               source, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, COALESCE(?, ?), ?, ?, ?, ?, ?, ?) \
             ON CONFLICT (id) DO UPDATE SET \
               event_id = excluded.event_id, stage_ordinal = excluded.stage_ordinal, \
               placement_kind = excluded.placement_kind, swiss_round = excluded.swiss_round, \
               swiss_wins = excluded.swiss_wins, swiss_losses = excluded.swiss_losses, \
               playoff_rounds_before_final = excluded.playoff_rounds_before_final, \
               playoff_slot = excluded.playoff_slot, team_a = excluded.team_a, \
               team_b = excluded.team_b, format = excluded.format, \
               scheduled_start = excluded.scheduled_start, \
               actual_start = COALESCE(?, matches.actual_start, excluded.actual_start), \
               status = excluded.status, winner = excluded.winner, \
               score_a = excluded.score_a, score_b = excluded.score_b, \
               source = excluded.source, updated_at = excluded.updated_at",
        )
        .bind(m.id)
        .bind(event.id)
        .bind(m.stage.0)
        .bind(placement.kind)
        .bind(placement.swiss_round)
        .bind(placement.swiss_wins)
        .bind(placement.swiss_losses)
        .bind(placement.playoff_rounds_before_final)
        .bind(placement.playoff_slot)
        .bind(m.team_a)
        .bind(m.team_b)
        .bind(format_code(m.format))
        .bind(m.scheduled_start)
        .bind(m.actual_start)
        .bind(first_seen_started)
        .bind(status_code(m.status))
        .bind(m.winner)
        .bind(score.map(|s| s.team_a))
        .bind(score.map(|s| s.team_b))
        .bind(snapshot.source.code())
        .bind(now)
        .bind(m.actual_start)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}

/// When a snapshot puts stored Matches in another Stage ordinal than before
/// (HLTV returning with a Stage a fallback did not list, so the fallback's
/// ordinals were off), the League data kept by ordinal follows its Stage:
/// Pick'ems, their Picks and Locks, and hand-added Matches. The mapping is
/// read from the Matches both know; an ordinal whose Matches split between
/// two new ordinals is left alone.
async fn move_stage_data(
    db: &mut SqliteConnection,
    snapshot: &EventSnapshot,
) -> Result<(), sqlx::Error> {
    let stored: Vec<(MatchId, u8)> = sqlx::query_as(
        "SELECT id, stage_ordinal FROM matches WHERE event_id = ? AND league_id IS NULL",
    )
    .bind(snapshot.event.id)
    .fetch_all(&mut *db)
    .await?;
    let mut moves: std::collections::BTreeMap<u8, Option<u8>> = Default::default();
    for (id, old) in stored {
        let Some(new) = snapshot.matches.iter().find(|m| m.id == id).map(|m| m.stage.0) else {
            continue;
        };
        let entry = moves.entry(old).or_insert(Some(new));
        if *entry != Some(new) {
            *entry = None;
        }
    }
    let moves: Vec<(u8, u8)> = moves
        .into_iter()
        .filter_map(|(old, new)| new.filter(|new| *new != old).map(|new| (old, new)))
        .collect();
    let mut targets: Vec<u8> = moves.iter().map(|(_, new)| *new).collect();
    targets.sort();
    targets.dedup();
    if moves.is_empty() || targets.len() != moves.len() {
        return Ok(());
    }

    const PARKED: i64 = 1000;
    sqlx::query("PRAGMA defer_foreign_keys = ON").execute(&mut *db).await?;
    let tables = [
        "pickems WHERE league_id IN (SELECT id FROM leagues WHERE event_id = ?2)",
        "picks WHERE league_id IN (SELECT id FROM leagues WHERE event_id = ?2)",
        "pickem_locks WHERE league_id IN (SELECT id FROM leagues WHERE event_id = ?2)",
        "matches WHERE event_id = ?2 AND league_id IS NOT NULL",
    ];
    // Two steps (park, then place), so swapped ordinals never collide.
    let park = moves.iter().map(|&(old, _)| (i64::from(old), i64::from(old) + PARKED));
    let place = moves.iter().map(|&(old, new)| (i64::from(old) + PARKED, i64::from(new)));
    for (old, new) in park.chain(place) {
        for table in tables {
            let (name, filter) = table.split_once(" WHERE ").expect("table filter");
            sqlx::query(sqlx::AssertSqlSafe(format!(
                "UPDATE {name} SET stage_ordinal = ?3 WHERE stage_ordinal = ?1 AND {filter}"
            )))
            .bind(old)
            .bind(snapshot.event.id)
            .bind(new)
            .execute(&mut *db)
            .await?;
        }
    }
    Ok(())
}

/// The stored base data of one Event (no Overrides applied). `None` when the
/// Event is unknown.
pub async fn base_view(
    db: &mut SqliteConnection,
    id: EventId,
) -> Result<Option<EventView>, sqlx::Error> {
    let Some(event) = find(&mut *db, id).await? else {
        return Ok(None);
    };
    let stages: Vec<(u8, String, String)> = sqlx::query_as(
        "SELECT ordinal, kind, name FROM stages WHERE event_id = ? ORDER BY ordinal",
    )
    .bind(id)
    .fetch_all(&mut *db)
    .await?;
    let stages = stages
        .into_iter()
        .map(|(ordinal, kind, name)| Stage {
            ordinal: StageOrdinal(ordinal),
            kind: stage_kind_from_code(&kind),
            name,
        })
        .collect();

    let teams: Vec<(TeamId, String, Option<String>)> = sqlx::query_as(
        "SELECT DISTINCT t.id, t.name, t.logo_url FROM teams t \
         JOIN matches m ON t.id IN (m.team_a, m.team_b) \
         WHERE m.event_id = ? AND m.league_id IS NULL ORDER BY t.id",
    )
    .bind(id)
    .fetch_all(&mut *db)
    .await?;
    let teams = teams
        .into_iter()
        .map(|(id, name, logo_url)| Team { id, name, logo_url })
        .collect();

    let rows: Vec<MatchRow> = sqlx::query_as(
        "SELECT id, stage_ordinal, placement_kind, swiss_round, swiss_wins, swiss_losses, \
           playoff_rounds_before_final, playoff_slot, team_a, team_b, format, scheduled_start, \
           actual_start, status, winner, score_a, score_b, source, updated_at \
         FROM matches WHERE event_id = ? AND league_id IS NULL ORDER BY id",
    )
    .bind(id)
    .fetch_all(&mut *db)
    .await?;
    let matches = rows.into_iter().map(SourcedMatch::from).collect();

    Ok(Some(EventView {
        event,
        stages,
        teams,
        matches,
    }))
}

/// The Matches League Admins of `league` added by hand (#13): League data,
/// never part of [`base_view`].
pub async fn added_matches<'e>(
    db: impl Executor<'e, Database = Sqlite>,
    league: crate::domain::LeagueId,
) -> Result<Vec<SourcedMatch>, sqlx::Error> {
    let rows: Vec<MatchRow> = sqlx::query_as(
        "SELECT id, stage_ordinal, placement_kind, swiss_round, swiss_wins, swiss_losses, \
           playoff_rounds_before_final, playoff_slot, team_a, team_b, format, scheduled_start, \
           actual_start, status, winner, score_a, score_b, source, updated_at \
         FROM matches WHERE league_id = ? ORDER BY id",
    )
    .bind(league)
    .fetch_all(db)
    .await?;
    Ok(rows.into_iter().map(SourcedMatch::from).collect())
}

/// Stores a Match added by hand to `league` (#13). `game.id` is a local id
/// (`store::external_ids::allocate_local_ids`).
pub async fn insert_added_match<'e>(
    db: impl Executor<'e, Database = Sqlite>,
    league: crate::domain::LeagueId,
    event: EventId,
    game: &Match,
    now: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    let placement = PlacementColumns::of(game.placement);
    sqlx::query(
        "INSERT INTO matches (id, event_id, stage_ordinal, placement_kind, swiss_round, \
           swiss_wins, swiss_losses, playoff_rounds_before_final, playoff_slot, team_a, team_b, \
           format, scheduled_start, actual_start, status, winner, score_a, score_b, source, \
           updated_at, league_id) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, NULL, ?, NULL, NULL, NULL, 'manual', ?, ?)",
    )
    .bind(game.id)
    .bind(event)
    .bind(game.stage.0)
    .bind(placement.kind)
    .bind(placement.swiss_round)
    .bind(placement.swiss_wins)
    .bind(placement.swiss_losses)
    .bind(placement.playoff_rounds_before_final)
    .bind(placement.playoff_slot)
    .bind(game.team_a)
    .bind(game.team_b)
    .bind(format_code(game.format))
    .bind(game.scheduled_start)
    .bind(status_code(game.status))
    .bind(now)
    .bind(league)
    .execute(db)
    .await?;
    Ok(())
}

#[derive(FromRow)]
struct MatchRow {
    id: MatchId,
    stage_ordinal: u8,
    placement_kind: String,
    swiss_round: Option<u8>,
    swiss_wins: Option<u8>,
    swiss_losses: Option<u8>,
    playoff_rounds_before_final: Option<u8>,
    playoff_slot: Option<u8>,
    team_a: Option<TeamId>,
    team_b: Option<TeamId>,
    format: String,
    scheduled_start: Option<DateTime<Utc>>,
    actual_start: Option<DateTime<Utc>>,
    status: String,
    winner: Option<TeamId>,
    score_a: Option<u8>,
    score_b: Option<u8>,
    source: String,
    updated_at: DateTime<Utc>,
}

impl From<MatchRow> for SourcedMatch {
    fn from(row: MatchRow) -> SourcedMatch {
        let placement = match row.placement_kind.as_str() {
            "swiss" => MatchPlacement::Swiss {
                round: row.swiss_round.unwrap_or(1),
                wins: row.swiss_wins.unwrap_or(0),
                losses: row.swiss_losses.unwrap_or(0),
            },
            "playoffs" => MatchPlacement::Playoffs {
                rounds_before_final: row.playoff_rounds_before_final.unwrap_or(0),
                slot: row.playoff_slot.unwrap_or(0),
            },
            _ => MatchPlacement::Unplaced,
        };
        let score = match (row.score_a, row.score_b) {
            (Some(team_a), Some(team_b)) => Some(MatchScore { team_a, team_b }),
            _ => None,
        };
        SourcedMatch {
            data: Match {
                id: row.id,
                stage: StageOrdinal(row.stage_ordinal),
                placement,
                team_a: row.team_a,
                team_b: row.team_b,
                format: format_from_code(&row.format),
                scheduled_start: row.scheduled_start,
                actual_start: row.actual_start,
                status: status_from_code(&row.status),
                winner: row.winner,
                score,
            },
            source: DataSourceKind::from_code(&row.source).unwrap_or(DataSourceKind::Hltv),
            updated_at: row.updated_at,
        }
    }
}

struct PlacementColumns {
    kind: &'static str,
    swiss_round: Option<u8>,
    swiss_wins: Option<u8>,
    swiss_losses: Option<u8>,
    playoff_rounds_before_final: Option<u8>,
    playoff_slot: Option<u8>,
}

impl PlacementColumns {
    fn of(placement: MatchPlacement) -> PlacementColumns {
        let mut columns = PlacementColumns {
            kind: "unplaced",
            swiss_round: None,
            swiss_wins: None,
            swiss_losses: None,
            playoff_rounds_before_final: None,
            playoff_slot: None,
        };
        match placement {
            MatchPlacement::Swiss {
                round,
                wins,
                losses,
            } => {
                columns.kind = "swiss";
                columns.swiss_round = Some(round);
                columns.swiss_wins = Some(wins);
                columns.swiss_losses = Some(losses);
            }
            MatchPlacement::Playoffs {
                rounds_before_final,
                slot,
            } => {
                columns.kind = "playoffs";
                columns.playoff_rounds_before_final = Some(rounds_before_final);
                columns.playoff_slot = Some(slot);
            }
            MatchPlacement::Unplaced => {}
        }
        columns
    }
}

fn stage_kind_code(kind: StageKind) -> &'static str {
    match kind {
        StageKind::Swiss => "swiss",
        StageKind::Playoffs => "playoffs",
        StageKind::Groups => "groups",
    }
}

fn stage_kind_from_code(code: &str) -> StageKind {
    match code {
        "swiss" => StageKind::Swiss,
        "playoffs" => StageKind::Playoffs,
        _ => StageKind::Groups,
    }
}

fn format_code(format: MatchFormat) -> &'static str {
    match format {
        MatchFormat::Bo1 => "bo1",
        MatchFormat::Bo3 => "bo3",
        MatchFormat::Bo5 => "bo5",
    }
}

fn format_from_code(code: &str) -> MatchFormat {
    match code {
        "bo1" => MatchFormat::Bo1,
        "bo5" => MatchFormat::Bo5,
        _ => MatchFormat::Bo3,
    }
}

fn status_code(status: MatchStatus) -> &'static str {
    match status {
        MatchStatus::Upcoming => "upcoming",
        MatchStatus::Live => "live",
        MatchStatus::Finished => "finished",
        MatchStatus::Cancelled => "cancelled",
    }
}

fn status_from_code(code: &str) -> MatchStatus {
    match code {
        "live" => MatchStatus::Live,
        "finished" => MatchStatus::Finished,
        "cancelled" => MatchStatus::Cancelled,
        _ => MatchStatus::Upcoming,
    }
}
