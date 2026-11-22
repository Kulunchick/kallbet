//! A League's Matches: the effective Match view, evaluated against the
//! Lock/Reveal policy, with transitions persisted.
//!
//! The one place League logic gets its Matches from. The Matches API, the
//! Prediction endpoint and the poller all call [`evaluate`], so a Lock or a
//! Reveal that any of them observes is stored at once and seen the
//! same way by all of them.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use sqlx::{SqliteConnection, SqlitePool};

use crate::domain::effective_match::{self, EventView, LeagueOverrides};
use crate::domain::lock_reveal::{self, MemberSpan, Outcome};
use crate::domain::{MatchId, StageKind, StageOrdinal, UserId};
use crate::store::events;
use crate::store::leagues::{self, League, MemberProfile};
use crate::store::overrides as store_overrides;
use crate::store::pickems;
use crate::store::predictions::{self, Prediction};

/// Everything about a League's Matches at one moment.
pub struct LeagueMatches {
    /// The effective Match view (base data + this League's Overrides).
    pub view: EventView,
    /// The Lock/Reveal outcome of every Match in `view`.
    pub outcomes: HashMap<MatchId, Outcome>,
    pub edit_windows: HashMap<MatchId, (UserId, DateTime<Utc>)>,
    pub stage_points: HashMap<StageOrdinal, u32>,
    /// Every Prediction in the League. Never send other Members' ones to a
    /// client before Reveal.
    pub predictions: Vec<Prediction>,
    /// Every membership stretch, for Eligible Members.
    pub members: Vec<MemberSpan>,
    /// Current Members with Nickname and avatar, for the revealed Predictions.
    pub profiles: Vec<MemberProfile>,
    /// The League's hand-added Matches and Overrides in force (#13), already
    /// applied in `view`; for showing which fields are overridden.
    pub overrides: LeagueOverrides,
}

impl LeagueMatches {
    pub fn points_for(&self, stage: StageOrdinal) -> u32 {
        self.stage_points.get(&stage).copied().unwrap_or(1)
    }

    pub fn outcome(&self, id: MatchId) -> Option<&Outcome> {
        self.outcomes.get(&id)
    }

    pub fn prediction_of(&self, id: MatchId, user: UserId) -> Option<&Prediction> {
        self.predictions
            .iter()
            .find(|p| p.match_id == id && p.user_id == user)
    }
}

/// The League's effective Match view: the base data, the League's
/// hand-added Matches and its Overrides in force (#13), merged by
/// [`effective_match::effective_view`].
pub async fn effective_view(db: &SqlitePool, league: &League) -> Result<EventView, sqlx::Error> {
    let mut conn = db.acquire().await?;
    Ok(effective_view_and_overrides(&mut conn, league).await?.0)
}

/// The League Admins' working view (#13): [`effective_view`] including the
/// hidden Matches, and the Overrides in force.
pub async fn admin_view(
    db: &mut SqliteConnection,
    league: &League,
) -> Result<(EventView, LeagueOverrides), sqlx::Error> {
    let base = events::base_view(&mut *db, league.event_id)
        .await?
        .ok_or_else(|| sqlx::Error::RowNotFound)?;
    let overrides = store_overrides::for_league(&mut *db, league.id).await?;
    Ok((effective_match::with_hidden(base, &overrides), overrides))
}

/// [`effective_view`] and the Overrides it was built with.
pub async fn effective_view_and_overrides(
    db: &mut SqliteConnection,
    league: &League,
) -> Result<(EventView, LeagueOverrides), sqlx::Error> {
    let base = events::base_view(&mut *db, league.event_id)
        .await?
        .ok_or_else(|| sqlx::Error::RowNotFound)?;
    let overrides = store_overrides::for_league(&mut *db, league.id).await?;
    Ok((effective_match::effective_view(base, &overrides), overrides))
}

/// Loads the League's Matches, evaluates Lock/Reveal for each at `now`, and
/// stores any new Lock/Reveal stamps. For reads; a write that depends on Lock
/// uses [`evaluate_in`] inside its transaction.
pub async fn evaluate(
    db: &SqlitePool,
    league: &League,
    now: DateTime<Utc>,
) -> Result<LeagueMatches, sqlx::Error> {
    let mut tx = crate::db::begin_write(db).await?;
    let matches = evaluate_in(&mut tx, league, now).await?;
    tx.commit().await?;
    Ok(matches)
}

/// [`evaluate`] on one connection, typically a write transaction from
/// [`crate::db::begin_write`]: a write that must respect Lock (a Prediction,
/// a Pick'em, an Override, a poll that may move a Scheduled Start) opens the
/// transaction, reads `now` from the clock, evaluates (storing every Lock due
/// by the data it has), checks, writes, evaluates again and commits. No other
/// writer runs in between, so no write lands after a Lock or Reveal that has
/// happened, and no later data un-happens one (ADR-0003).
pub async fn evaluate_in(
    db: &mut SqliteConnection,
    league: &League,
    now: DateTime<Utc>,
) -> Result<LeagueMatches, sqlx::Error> {
    let (view, overrides) = effective_view_and_overrides(&mut *db, league).await?;
    let members: Vec<MemberSpan> = leagues::all_memberships(&mut *db, league.id)
        .await?
        .into_iter()
        .map(|m| MemberSpan {
            user_id: m.user_id,
            joined_at: m.joined_at,
            left_at: m.left_at,
        })
        .collect();
    // A Prediction counts only while its team plays the Match in the
    // effective view (#13): after a teams Override, a pick of a team that no
    // longer plays it is as good as none (it does not count toward Reveal and
    // scores nothing). The row is kept, so clearing the Override restores it.
    let predictions: Vec<Prediction> = predictions::for_league(&mut *db, league.id)
        .await?
        .into_iter()
        .filter(|p| {
            view.match_by_id(p.match_id)
                .is_some_and(|m| m.data.teams().contains(&Some(p.team_id)))
        })
        .collect();
    let stored = predictions::stamps(&mut *db, league.id).await?;

    let windows: Vec<(MatchId, UserId, DateTime<Utc>)> = sqlx::query_as(
        "SELECT match_id, user_id, ends_at FROM prediction_edit_windows WHERE league_id = ?",
    )
    .bind(league.id)
    .fetch_all(&mut *db)
    .await?;
    let edit_windows: HashMap<_, _> = windows
        .into_iter()
        .map(|(game, user, end)| (game, (user, end)))
        .collect();
    let points: Vec<(StageOrdinal, u32)> =
        sqlx::query_as("SELECT stage, points FROM league_stage_points WHERE league_id = ?")
            .bind(league.id)
            .fetch_all(&mut *db)
            .await?;

    let mut outcomes = HashMap::new();
    for m in &view.matches {
        let id = m.data.id;
        let predicted: Vec<UserId> = predictions
            .iter()
            .filter(|p| p.match_id == id)
            .map(|p| p.user_id)
            .collect();
        let outcome = lock_reveal::evaluate(&lock_reveal::Inputs {
            game: &m.data,
            members: &members,
            predicted: &predicted,
            stored: stored.get(&id).copied().unwrap_or_default(),
            edit_until: edit_windows.get(&id).map(|(_, end)| *end),
            now,
        });
        if !outcome.transitions.is_empty() {
            predictions::record(&mut *db, league.id, id, &outcome.transitions).await?;
        }
        outcomes.insert(id, outcome);
    }

    // Store each Stage Pick'em's Lock the moment it is due (#14), for the
    // same reason as the Match Locks above: a postponement must not undo it.
    for stage in view
        .stages
        .iter()
        .filter(|s| matches!(s.kind, StageKind::Swiss | StageKind::Playoffs))
    {
        pickems::evaluate_lock(&mut *db, league.id, &view, stage.ordinal, now).await?;
    }

    Ok(LeagueMatches {
        view,
        outcomes,
        edit_windows,
        stage_points: points.into_iter().collect(),
        predictions,
        members,
        profiles: leagues::members(&mut *db, league.id).await?,
        overrides,
    })
}
