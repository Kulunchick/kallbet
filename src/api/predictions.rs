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

use crate::api::leagues::{require_member, require_writable};
use crate::auth::session::CurrentUser;
use crate::db;
use crate::domain::{LeagueId, MatchId, TeamId, UserId};
use crate::error::{ApiError, ApiResult};
use crate::league_matches;
use crate::state::AppState;
use crate::store::leagues::League;
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
    let (league, _) = require_member(&state, &user, id).await?;
    require_writable(&league)?;
    // Check, write and the Reveal evaluation are one write transaction
    // (ADR-0003): no Lock or Reveal can be stored between them, and `now` is
    // read once the transaction holds the write lock.
    let mut tx = db::begin_write(&state.db).await?;
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
    if matches.outcome(match_id).is_none_or(|o| o.is_locked()) {
        return Err(ApiError::conflict("match_locked"));
    }
    if !game.data.teams().contains(&Some(team)) {
        return Err(ApiError::unprocessable("team_not_in_match"));
    }
    let prediction = predictions::upsert(&mut *db, league.id, match_id, user, team, now)
        .await?
        .ok_or(ApiError::conflict("match_locked"))?;
    // This Prediction may complete the set that triggers a Reveal (#6).
    league_matches::evaluate_in(db, league, now).await?;
    Ok(prediction)
}
