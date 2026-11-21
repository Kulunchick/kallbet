//! Leaderboard and Pick'em Leaderboard API.
//!
//! `GET /api/leagues/{id}/leaderboard` ranks the current Members by Match
//! Points. Nothing is stored: [`ranking`] computes the ranking from the
//! League's effective Match view and Predictions, so every poll that delivers
//! a result (and, from #13, every Override) shows up at once. #14 adds the
//! Pick'em Leaderboard here.

use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;

use crate::api::leagues::require_member;
use crate::auth::session::CurrentUser;
use crate::domain::scoring::{self, ScoredMatch, LeaderboardRow};
use crate::domain::{LeagueId, StageKind, UserId};
use crate::error::ApiResult;
use crate::league_matches::{self, LeagueMatches};
use crate::state::AppState;
use crate::store::leagues;

pub fn router() -> Router<AppState> {
    Router::new().route("/leagues/{id}/leaderboard", get(show))
}

/// Ranks `members` by Match Points over every Match of the League's
/// effective view. Reusable: #16 scores a subset of Matches the same way for
/// "Points gained today", #17 renders the final ranking with medals.
pub fn ranking(matches: &LeagueMatches, members: &[UserId]) -> Vec<LeaderboardRow> {
    let games: Vec<ScoredMatch> = matches
        .view
        .matches
        .iter()
        .map(|m| ScoredMatch {
            game: &m.data,
            playoffs: matches
                .view
                .stage(m.data.stage)
                .is_some_and(|s| s.kind == StageKind::Playoffs),
        })
        .collect();
    scoring::leaderboard(members, &games, |user, game| {
        matches.prediction_of(game, user).map(|p| p.team_id)
    })
}

#[derive(Serialize)]
struct Leaderboard {
    rows: Vec<Row>,
}

#[derive(Serialize)]
struct Row {
    /// Members with equal Points share a place (1, 1, 3).
    place: u32,
    user_id: UserId,
    nickname: String,
    avatar_url: Option<String>,
    points: u32,
    playoff_points: u32,
    /// Correct Predictions as a % of those made on decided, non-Void
    /// Matches; `null` while there are none.
    percent_correct: Option<u32>,
}

async fn show(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<LeagueId>,
) -> ApiResult<Json<Leaderboard>> {
    let (league, _) = require_member(&state, &user, id).await?;
    let matches = league_matches::evaluate(&state.db, &league, state.clock.now()).await?;
    let profiles = leagues::members(&state.db, league.id).await?;
    let ids: Vec<UserId> = profiles.iter().map(|p| p.user_id).collect();

    let mut rows: Vec<Row> = ranking(&matches, &ids)
        .into_iter()
        .filter_map(|s| {
            let profile = profiles.iter().find(|p| p.user_id == s.user_id)?;
            Some(Row {
                place: s.place,
                user_id: s.user_id,
                nickname: profile.nickname.clone(),
                avatar_url: profile.avatar_url.clone(),
                points: s.points,
                playoff_points: s.playoff_points,
                percent_correct: s.percent_correct(),
            })
        })
        .collect();
    // Within a shared place, alphabetical.
    rows.sort_by(|a, b| {
        (a.place, a.nickname.to_lowercase()).cmp(&(b.place, b.nickname.to_lowercase()))
    });
    Ok(Json(Leaderboard { rows }))
}
