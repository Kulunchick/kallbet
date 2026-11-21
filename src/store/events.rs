//! Persistence of Events, Stages, teams and Matches.
//!
//! #4 stores the Event row a League points at; #5 adds Stages, teams and
//! Matches here.

use chrono::{DateTime, Utc};
use sqlx::{FromRow, SqlitePool};

use crate::domain::{Event, EventId};

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

pub async fn find(db: &SqlitePool, id: EventId) -> Result<Option<Event>, sqlx::Error> {
    let row: Option<EventRow> =
        sqlx::query_as("SELECT id, name, starts_at, ends_at FROM events WHERE id = ?")
            .bind(id)
            .fetch_optional(db)
            .await?;
    Ok(row.map(Event::from))
}
