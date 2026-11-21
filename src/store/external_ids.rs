//! External-id mapping table (source, kind, external id -> canonical id),
//! local ids, the data-quality flags (unmatched records, conflicting
//! winners) and re-keying local records once HLTV knows them. Ticket #9.
//!
//! **Local ids**: a Match or team that no HLTV id is known for (seen only in
//! a fallback source, or added by hand by #13) gets a negative id from
//! [`allocate_local_ids`]; HLTV ids are always positive, so the two never
//! collide. When HLTV later shows the same Match, [`rekey_match`] moves
//! every reference to the HLTV id.

use chrono::{DateTime, Utc};
use sqlx::{FromRow, SqlitePool};

use crate::domain::{DataSourceKind, Event, EventId, MatchId, Team, TeamId};
use crate::sources::fallback::mapping::{
    Conflict, IdKind, Mapping, Mappings, NewMapping, Origin, Unmatched,
};

/// The canonical id `external_id` of `kind` at `source` is mapped to.
pub async fn canonical(
    db: &SqlitePool,
    source: DataSourceKind,
    kind: IdKind,
    external_id: &str,
) -> Result<Option<i64>, sqlx::Error> {
    let row: Option<(i64,)> = sqlx::query_as(
        "SELECT canonical_id FROM external_ids WHERE source = ? AND kind = ? AND external_id = ?",
    )
    .bind(source.code())
    .bind(kind.code())
    .bind(external_id)
    .fetch_optional(db)
    .await?;
    Ok(row.map(|(id,)| id))
}

/// The external id `source` has for a canonical id (a manual mapping first).
pub async fn external(
    db: &SqlitePool,
    source: DataSourceKind,
    kind: IdKind,
    canonical: i64,
) -> Result<Option<String>, sqlx::Error> {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT external_id FROM external_ids WHERE source = ? AND kind = ? AND canonical_id = ? \
         ORDER BY origin = 'manual' DESC, updated_at DESC LIMIT 1",
    )
    .bind(source.code())
    .bind(kind.code())
    .bind(canonical)
    .fetch_optional(db)
    .await?;
    Ok(row.map(|(id,)| id))
}

/// Every mapping of one source.
pub async fn mappings(db: &SqlitePool, source: DataSourceKind) -> Result<Mappings, sqlx::Error> {
    let rows: Vec<(String, String, i64, String)> = sqlx::query_as(
        "SELECT kind, external_id, canonical_id, origin FROM external_ids WHERE source = ?",
    )
    .bind(source.code())
    .fetch_all(db)
    .await?;
    Ok(rows
        .into_iter()
        .filter_map(|(kind, external_id, canonical, origin)| {
            Some((
                (IdKind::from_code(&kind)?, external_id),
                Mapping {
                    canonical,
                    origin: Origin::from_code(&origin),
                },
            ))
        })
        .collect())
}

/// Stores automatic mappings. Never replaces a manual one.
pub async fn record_mappings(
    db: &SqlitePool,
    source: DataSourceKind,
    mappings: &[NewMapping],
    now: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    for m in mappings {
        sqlx::query(
            "INSERT INTO external_ids (source, kind, external_id, canonical_id, origin, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?) \
             ON CONFLICT (source, kind, external_id) DO UPDATE SET \
               canonical_id = excluded.canonical_id, origin = excluded.origin, \
               updated_at = excluded.updated_at \
             WHERE external_ids.origin <> 'manual'",
        )
        .bind(source.code())
        .bind(m.kind.code())
        .bind(&m.external_id)
        .bind(m.canonical)
        .bind(m.origin.code())
        .bind(now)
        .execute(db)
        .await?;
    }
    Ok(())
}

/// A League Admin's fix (#13): `external_id` of `kind` at `source` is
/// `canonical`. Wins over every automatic mapping from now on; the record's
/// unmatched flag clears on that source's next fetch.
pub async fn set_manual(
    db: &SqlitePool,
    source: DataSourceKind,
    kind: IdKind,
    external_id: &str,
    canonical: i64,
    now: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO external_ids (source, kind, external_id, canonical_id, origin, updated_at) \
         VALUES (?, ?, ?, ?, 'manual', ?) \
         ON CONFLICT (source, kind, external_id) DO UPDATE SET \
           canonical_id = excluded.canonical_id, origin = 'manual', updated_at = excluded.updated_at",
    )
    .bind(source.code())
    .bind(kind.code())
    .bind(external_id)
    .bind(canonical)
    .bind(now)
    .execute(db)
    .await?;
    Ok(())
}

/// Reserves `count` local ids of `kind` (`Match` or `Team`) and returns the
/// first; the others count down from it (-1, -2, ...). #13 takes the id of a
/// Match added by hand from here.
pub async fn allocate_local_ids(
    db: &SqlitePool,
    kind: IdKind,
    count: usize,
) -> Result<i64, sqlx::Error> {
    let (last,): (i64,) = sqlx::query_as(
        "INSERT INTO local_ids (kind, last) VALUES (?, -?) \
         ON CONFLICT (kind) DO UPDATE SET last = local_ids.last - ? RETURNING last",
    )
    .bind(kind.code())
    .bind(count as i64)
    .bind(count as i64)
    .fetch_one(db)
    .await?;
    Ok(last + count as i64 - 1)
}

/// Every known team, of all Events.
pub async fn all_teams(db: &SqlitePool) -> Result<Vec<Team>, sqlx::Error> {
    let rows: Vec<(TeamId, String, Option<String>)> =
        sqlx::query_as("SELECT id, name, logo_url FROM teams ORDER BY id")
            .fetch_all(db)
            .await?;
    Ok(rows
        .into_iter()
        .map(|(id, name, logo_url)| Team { id, name, logo_url })
        .collect())
}

/// Every stored Event.
pub async fn all_events(db: &SqlitePool) -> Result<Vec<Event>, sqlx::Error> {
    type Row = (
        EventId,
        String,
        Option<DateTime<Utc>>,
        Option<DateTime<Utc>>,
    );
    let rows: Vec<Row> =
        sqlx::query_as("SELECT id, name, starts_at, ends_at FROM events ORDER BY id")
            .fetch_all(db)
            .await?;
    Ok(rows
        .into_iter()
        .map(|(id, name, starts_at, ends_at)| Event {
            id,
            name,
            starts_at,
            ends_at,
        })
        .collect())
}

/// A fallback record the chain could not link, for League Admins (#13).
#[derive(Debug, Clone, PartialEq, Eq, FromRow)]
pub struct UnmatchedRecord {
    pub event_id: EventId,
    /// `DataSourceKind::code`.
    pub source: String,
    /// `event`, `match` or `team`.
    pub kind: String,
    /// What to pass to [`set_manual`] to fix it.
    pub external_id: String,
    /// Team name, or "A vs B, start".
    pub description: String,
    pub seen_at: DateTime<Utc>,
}

/// Replaces the unmatched flags of one source for one Event.
pub async fn replace_unmatched(
    db: &SqlitePool,
    event: EventId,
    source: DataSourceKind,
    records: &[Unmatched],
    now: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    let mut tx = db.begin().await?;
    sqlx::query("DELETE FROM unmatched_records WHERE event_id = ? AND source = ?")
        .bind(event)
        .bind(source.code())
        .execute(&mut *tx)
        .await?;
    for r in records {
        sqlx::query(
            "INSERT OR REPLACE INTO unmatched_records \
               (event_id, source, kind, external_id, description, seen_at) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(event)
        .bind(source.code())
        .bind(r.kind.code())
        .bind(&r.external_id)
        .bind(&r.description)
        .bind(now)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await
}

pub async fn unmatched(
    db: &SqlitePool,
    event: EventId,
) -> Result<Vec<UnmatchedRecord>, sqlx::Error> {
    sqlx::query_as(
        "SELECT event_id, source, kind, external_id, description, seen_at FROM unmatched_records \
         WHERE event_id = ? ORDER BY source, kind, external_id",
    )
    .bind(event)
    .fetch_all(db)
    .await
}

/// Two Data Sources disagreed on a Match's winner (#13 shows it).
#[derive(Debug, Clone, PartialEq, Eq, FromRow)]
pub struct SourceConflict {
    pub event_id: EventId,
    pub match_id: MatchId,
    /// The higher-priority source, whose result stands.
    pub kept_source: String,
    pub kept_winner: Option<TeamId>,
    pub other_source: String,
    pub other_winner: Option<TeamId>,
    pub detected_at: DateTime<Utc>,
}

/// Records conflicts; a repeated one keeps its first `detected_at`.
pub async fn record_conflicts(
    db: &SqlitePool,
    event: EventId,
    conflicts: &[Conflict],
    now: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    for c in conflicts {
        sqlx::query(
            "INSERT INTO source_conflicts (event_id, match_id, kept_source, kept_winner, \
               other_source, other_winner, detected_at) VALUES (?, ?, ?, ?, ?, ?, ?) \
             ON CONFLICT (match_id, other_source) DO UPDATE SET \
               kept_source = excluded.kept_source, kept_winner = excluded.kept_winner, \
               other_winner = excluded.other_winner",
        )
        .bind(event)
        .bind(c.match_id)
        .bind(c.kept_source.code())
        .bind(c.kept_winner)
        .bind(c.other_source.code())
        .bind(c.other_winner)
        .bind(now)
        .execute(db)
        .await?;
    }
    Ok(())
}

pub async fn conflicts(
    db: &SqlitePool,
    event: EventId,
) -> Result<Vec<SourceConflict>, sqlx::Error> {
    sqlx::query_as(
        "SELECT event_id, match_id, kept_source, kept_winner, other_source, other_winner, \
           detected_at FROM source_conflicts WHERE event_id = ? ORDER BY match_id, other_source",
    )
    .bind(event)
    .fetch_all(db)
    .await
}

/// Moves every reference to local team `from` to `to` (its HLTV id), and
/// drops the local row. **A ticket adding a table with a team id column
/// adds it here.**
pub async fn rekey_team(db: &SqlitePool, from: TeamId, to: TeamId) -> Result<(), sqlx::Error> {
    let mut tx = db.begin().await?;
    sqlx::query("PRAGMA defer_foreign_keys = ON")
        .execute(&mut *tx)
        .await?;
    let exists: Option<(i64,)> = sqlx::query_as("SELECT id FROM teams WHERE id = ?")
        .bind(to)
        .fetch_optional(&mut *tx)
        .await?;
    if exists.is_none() {
        sqlx::query("UPDATE teams SET id = ? WHERE id = ?")
            .bind(to)
            .bind(from)
            .execute(&mut *tx)
            .await?;
    }
    for sql in [
        "UPDATE matches SET team_a = ? WHERE team_a = ?",
        "UPDATE matches SET team_b = ? WHERE team_b = ?",
        "UPDATE matches SET winner = ? WHERE winner = ?",
        "UPDATE predictions SET team_id = ? WHERE team_id = ?",
        "UPDATE picks SET team_id = ? WHERE team_id = ?",
        "UPDATE source_conflicts SET kept_winner = ? WHERE kept_winner = ?",
        "UPDATE source_conflicts SET other_winner = ? WHERE other_winner = ?",
        "UPDATE external_ids SET canonical_id = ? WHERE kind = 'team' AND canonical_id = ?",
        "UPDATE overrides SET winner = ? WHERE winner = ?",
        "UPDATE overrides SET team_a = ? WHERE team_a = ?",
        "UPDATE overrides SET team_b = ? WHERE team_b = ?",
    ] {
        sqlx::query(sql)
            .bind(to)
            .bind(from)
            .execute(&mut *tx)
            .await?;
    }
    if exists.is_some() {
        sqlx::query("DELETE FROM teams WHERE id = ?")
            .bind(from)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await
}

/// Moves every reference to local Match `from` to `to` (its HLTV id). Does
/// nothing if a Match `to` is already stored. **A ticket adding a table with
/// a match id column (e.g. #13's Overrides) adds it here.**
pub async fn rekey_match(db: &SqlitePool, from: MatchId, to: MatchId) -> Result<bool, sqlx::Error> {
    let mut tx = db.begin().await?;
    sqlx::query("PRAGMA defer_foreign_keys = ON")
        .execute(&mut *tx)
        .await?;
    let exists: Option<(i64,)> = sqlx::query_as("SELECT id FROM matches WHERE id = ?")
        .bind(to)
        .fetch_optional(&mut *tx)
        .await?;
    if exists.is_some() {
        return Ok(false);
    }
    for sql in [
        "UPDATE matches SET id = ? WHERE id = ?",
        "UPDATE predictions SET match_id = ? WHERE match_id = ?",
        "UPDATE match_locks SET match_id = ? WHERE match_id = ?",
        "UPDATE source_conflicts SET match_id = ? WHERE match_id = ?",
        "UPDATE external_ids SET canonical_id = ? WHERE kind = 'match' AND canonical_id = ?",
        "UPDATE overrides SET match_id = ? WHERE match_id = ?",
    ] {
        sqlx::query(sql)
            .bind(to)
            .bind(from)
            .execute(&mut *tx)
            .await?;
    }
    // The notification log (#16): the per-Match dedupe keys
    // `<kind>:<league>:<match>:<user>` follow the Match, so a Reveal (or
    // reminder) already handled under the local id is not sent again.
    for kind in ["reminder", "reveal"] {
        sqlx::query(
            "UPDATE notification_log \
             SET key = ?1 || ':' || league_id || ':' || ?2 || ':' || user_id \
             WHERE key = ?1 || ':' || league_id || ':' || ?3 || ':' || user_id",
        )
        .bind(kind)
        .bind(to.0)
        .bind(from.0)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(true)
}
