//! Predictions API: upsert a Member's Prediction.
//!
//! `PUT /api/leagues/{id}/matches/{match_id}/prediction` with
//! `{"team_id": n}` → 200 `{match_id, team_id, updated_at}`.
//! Errors: 404 `not_found` (not a Member, or no such Match in the League's
//! effective view), 409 `match_locked`, 422 `team_not_in_match`.
//!
//! Lock is re-checked on the server at submission, from the same evaluation
//! the Matches view uses, so a stale page cannot bypass it.

use axum::extract::{Path, State};
use axum::routing::put;
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::SqliteConnection;

use crate::api::leagues::require_writable;
use crate::auth::session::CurrentUser;
use crate::db;
use crate::domain::{LeagueId, MatchId, TeamId, UserId};
use crate::error::{ApiError, ApiResult};
use crate::league_matches;
use crate::state::AppState;
use crate::store::leagues::{self, League};
use crate::store::predictions::{self, Prediction};

pub fn router() -> Router<AppState> {
    Router::new().route("/leagues/{id}/matches/{match_id}/prediction", put(upsert))
}

#[derive(Deserialize)]
struct Body {
    team_id: TeamId,
}

async fn upsert(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path((id, match_id)): Path<(LeagueId, MatchId)>,
    Json(body): Json<Body>,
) -> ApiResult<Json<Prediction>> {
    // Check, write and the Reveal evaluation are one write transaction
    // (ADR-0003): no Lock or Reveal can be stored between them, and `now` is
    // read once the transaction holds the write lock.
    let mut tx = db::begin_write(&state.db).await?;
    let (league, _) = leagues::member_of(&mut *tx, id, user.id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    require_writable(&league)?;
    let now = state.clock.now();
    let result = predict(&mut tx, &league, user.id, match_id, body.team_id, now).await;
    db::finish(tx, result).await.map(Json)
}

async fn predict(
    db: &mut SqliteConnection,
    league: &League,
    user: UserId,
    match_id: MatchId,
    team: TeamId,
    now: DateTime<Utc>,
) -> ApiResult<Prediction> {
    let matches = league_matches::evaluate_in(db, league, now).await?;
    let game = matches
        .view
        .match_by_id(match_id)
        .ok_or(ApiError::not_found())?;
    let outcome = matches.outcome(match_id).ok_or(ApiError::not_found())?;
    let window = matches.edit_windows.get(&match_id);
    if outcome.is_locked() || window.is_some_and(|(owner, _)| *owner != user) {
        return Err(ApiError::conflict("match_locked"));
    }
    if !game.data.teams().contains(&Some(team)) {
        return Err(ApiError::unprocessable("team_not_in_match"));
    }
    let prediction = predictions::upsert(&mut *db, league.id, match_id, user, team, now)
        .await?
        .ok_or(ApiError::conflict("match_locked"))?;
    if window.is_none()
        && matches.prediction_of(match_id, user).is_none()
        && outcome.eligible.contains(&user)
        && outcome
            .eligible
            .iter()
            .all(|&member| member == user || matches.prediction_of(match_id, member).is_some())
    {
        sqlx::query(
            "INSERT INTO prediction_edit_windows (league_id, match_id, user_id, ends_at)
             VALUES (?, ?, ?, ?)",
        )
        .bind(league.id)
        .bind(match_id)
        .bind(user)
        .bind(now + crate::domain::lock_reveal::FINAL_EDIT_WINDOW)
        .execute(&mut *db)
        .await?;
    }
    league_matches::evaluate_in(db, league, now).await?;
    Ok(prediction)
}
