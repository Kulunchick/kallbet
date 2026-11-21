//! Poller: refreshes Events and Matches from the `MatchSource`, then
//! evaluates every active League (Lock/Reveal, and later scoring,
//! notifications and archive).
//!
//! [`tick`] is one pass driven by `state.clock.now()`: it polls whatever is
//! due and returns. Production calls it every [`TICK_EVERY`]
//! (`runtime::spawn_background_tasks`); tests advance the `FakeClock` and call
//! it themselves.
//!
//! Cadence:
//! - the Event list every [`EVENT_LIST_EVERY`];
//! - the Matches of each Event with at least one non-archived League every
//!   [`MATCHES_EVERY`], or every [`MATCHES_EVERY_WHEN_HOT`] while one of its
//!   Matches is live or starts within [`HOT_WINDOW`].
//!
//! A failed fetch counts as a poll for the cadence (the next try waits its
//! turn; the `FallbackSource` circuit breaker handles back-off).

use std::time::Duration as StdDuration;

use chrono::{DateTime, Duration, Utc};

use crate::domain::league::final_decided;
use crate::domain::{EventId, Match, MatchStatus};
use crate::league_matches::{self, LeagueMatches};
use crate::state::AppState;
use crate::store::events;
use crate::store::leagues::{self, League};

pub const EVENT_LIST_EVERY: Duration = Duration::hours(6);
pub const MATCHES_EVERY: Duration = Duration::minutes(5);
pub const MATCHES_EVERY_WHEN_HOT: Duration = Duration::minutes(1);
/// A Match that starts within this window makes its Event "hot".
pub const HOT_WINDOW: Duration = Duration::minutes(45);
/// An upcoming Match whose Scheduled Start passed longer ago than this no
/// longer keeps its Event hot (stale data, not an imminent start).
const OVERDUE_LIMIT: Duration = Duration::hours(6);
/// How often production runs [`tick`].
pub const TICK_EVERY: StdDuration = StdDuration::from_secs(20);

const EVENT_LIST_KEY: &str = "event_list";

fn event_key(id: EventId) -> String {
    format!("event:{id}")
}

/// One poller pass at `state.clock.now()`. Errors are logged, never raised:
/// the next tick tries again.
pub async fn tick(state: &AppState) {
    let now = state.clock.now();
    if let Err(e) = run(state, now).await {
        tracing::error!("poller pass failed: {e}");
    }
}

async fn run(state: &AppState, now: DateTime<Utc>) -> Result<(), sqlx::Error> {
    // Store every Lock that is due by the data we already have, before new
    // data can move a Scheduled Start: a Lock that happened is never undone.
    for league in leagues::active(&state.db).await? {
        league_matches::evaluate(&state.db, &league, now).await?;
    }

    poll_event_list(state, now).await?;
    for event in events::with_active_leagues(&state.db).await? {
        poll_event(state, event, now).await?;
    }

    for league in leagues::active(&state.db).await? {
        let matches = league_matches::evaluate(&state.db, &league, now).await?;
        after_poll(state, &league, &matches, now).await?;
    }
    Ok(())
}

/// What happens for each active League after a poll, once its Lock/Reveal
/// state is up to date. Each later ticket adds one call here:
/// #7 scoring, #16 notifications, #17 archive.
async fn after_poll(
    state: &AppState,
    league: &League,
    matches: &LeagueMatches,
    now: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    archive_if_final_decided(state, league, matches, now).await?;
    Ok(())
}

/// Archives the League once the final has a winner in its effective view
/// (#17). Idempotent, so it may run on every tick; #13's Override endpoint
/// that sets a winner calls it too, so the League archives at once. Returns
/// whether this call archived it.
pub async fn archive_if_final_decided(
    state: &AppState,
    league: &League,
    matches: &LeagueMatches,
    now: DateTime<Utc>,
) -> Result<bool, sqlx::Error> {
    if league.archived_at.is_some() || !final_decided(&matches.view) {
        return Ok(false);
    }
    leagues::archive(&state.db, league.id, now).await
}

async fn poll_event_list(state: &AppState, now: DateTime<Utc>) -> Result<(), sqlx::Error> {
    let last = events::last_polled(&state.db, EVENT_LIST_KEY).await?;
    if last.is_some_and(|at| now - at < EVENT_LIST_EVERY) {
        return Ok(());
    }
    events::record_poll(&state.db, EVENT_LIST_KEY, now).await?;
    match state.source.list_events().await {
        Ok(listed) => {
            for event in &listed {
                events::upsert(&state.db, event).await?;
            }
        }
        Err(e) => tracing::warn!("polling the Event list failed: {e}"),
    }
    Ok(())
}

async fn poll_event(state: &AppState, id: EventId, now: DateTime<Utc>) -> Result<(), sqlx::Error> {
    let key = event_key(id);
    let stored: Vec<Match> = events::base_view(&state.db, id)
        .await?
        .map(|view| view.matches.into_iter().map(|m| m.data).collect())
        .unwrap_or_default();
    let last = events::last_polled(&state.db, &key).await?;
    if last.is_some_and(|at| now - at < match_poll_interval(&stored, now)) {
        return Ok(());
    }
    events::record_poll(&state.db, &key, now).await?;
    match state.source.fetch_event(id).await {
        Ok(snapshot) => events::save_snapshot(&state.db, &snapshot, now).await?,
        Err(e) => tracing::warn!("polling Event {id} failed: {e}"),
    }
    Ok(())
}

/// How long to wait between polls of an Event with these Matches.
pub fn match_poll_interval(matches: &[Match], now: DateTime<Utc>) -> Duration {
    let hot = matches.iter().any(|m| match m.status {
        MatchStatus::Live => true,
        MatchStatus::Upcoming => m
            .scheduled_start
            .is_some_and(|start| start <= now + HOT_WINDOW && start >= now - OVERDUE_LIMIT),
        MatchStatus::Finished | MatchStatus::Cancelled => false,
    });
    if hot {
        MATCHES_EVERY_WHEN_HOT
    } else {
        MATCHES_EVERY
    }
}
