//! Predictions API: upsert a Member's Prediction.
//!
//! `PUT /api/leagues/{id}/matches/{match_id}/prediction` with
//! `{"team_id": n, "rationale"?: Rationale | null}` → 200
//! `{match_id, user_id, team_id, updated_at}`.
//! Errors: 404 `not_found` (not a Member, or no such Match in the League's
//! effective view), 409 `match_locked`, 422 `rationale_invalid`,
//! `team_not_in_match`.
//!
//! The Rationale (#23, `domain::rationale`) is replaced with the pick: a
//! write without one leaves the Prediction without one. It is not in the
//! response: it leaves the server only with the revealed Prediction
//! (`api::matches::predictions_view`).
//!
//! The Prediction is always the caller's own (`CurrentUser`, a session or a
//! Bot User's Bearer token); nothing in the path or body names a User. A
//! human is Locked by an early Reveal, a Bot User only by the Hard Lock.
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
use crate::domain::rationale::Rationale;
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
    /// Read as any JSON value and validated by [`Rationale::parse`], so that
    /// every malformed Rationale answers 422 `rationale_invalid`.
    #[serde(default)]
    rationale: serde_json::Value,
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
    let rationale = Rationale::parse(body.rationale)
        .map_err(|_| ApiError::unprocessable("rationale_invalid"))?;
    let pick = Pick {
        team: body.team_id,
        rationale: rationale.as_ref(),
    };
    let now = state.clock.now();
    let result = if user.is_bot {
        predict_as_bot(&mut tx, &league, user.id, match_id, pick, now).await
    } else {
        predict(&mut tx, &league, user.id, match_id, pick, now).await
    };
    db::finish(tx, result).await.map(Json)
}

/// What a Prediction write sets: the team and its (validated) Rationale.
#[derive(Clone, Copy)]
struct Pick<'a> {
    team: TeamId,
    rationale: Option<&'a Rationale>,
}

/// A Bot User's Prediction (ADR-0003's amendment): open until the Match's
/// Hard Lock, whatever the humans' early Reveal and final edit window say.
/// It never opens that window, and it changes nothing about when the humans
/// Reveal.
async fn predict_as_bot(
    db: &mut SqliteConnection,
    league: &League,
    user: UserId,
    match_id: MatchId,
    pick: Pick<'_>,
    now: DateTime<Utc>,
) -> ApiResult<Prediction> {
    let matches = league_matches::evaluate_in(db, league, now).await?;
    let game = matches
        .view
        .match_by_id(match_id)
        .ok_or(ApiError::not_found())?;
    let outcome = matches.outcome(match_id).ok_or(ApiError::not_found())?;
    // A Bot User that joined after the Hard Lock is caught by it too.
    if outcome.is_hard_locked() {
        return Err(ApiError::conflict("match_locked"));
    }
    if !game.data.teams().contains(&Some(pick.team)) {
        return Err(ApiError::unprocessable("team_not_in_match"));
    }
    predictions::upsert_for_bot(
        &mut *db,
        league.id,
        match_id,
        user,
        pick.team,
        pick.rationale,
        now,
    )
    .await?
    .ok_or(ApiError::conflict("match_locked"))
}

async fn predict(
    db: &mut SqliteConnection,
    league: &League,
    user: UserId,
    match_id: MatchId,
    pick: Pick<'_>,
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
    if !game.data.teams().contains(&Some(pick.team)) {
        return Err(ApiError::unprocessable("team_not_in_match"));
    }
    let prediction = predictions::upsert(
        &mut *db,
        league.id,
        match_id,
        user,
        pick.team,
        pick.rationale,
        now,
    )
    .await?
    .ok_or(ApiError::conflict("match_locked"))?;
    // The last human Eligible Member to predict gets the final edit window.
    // Bot Users are not waited for (`awaited` holds the humans only).
    if window.is_none()
        && matches.prediction_of(match_id, user).is_none()
        && outcome.awaited.contains(&user)
        && outcome
            .awaited
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
