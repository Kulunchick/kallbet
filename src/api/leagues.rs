//! Leagues API: create, list mine, get, Invite Link, join, leave.
//! League management (#12): kick, co-admin roles, delete.
//!
//! Other areas that act inside a League call [`require_member`]: a User who
//! is not a Member gets the same 404 as for a League that does not exist.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{delete as http_delete, get, post, put};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::api::{events, leaderboard};
use crate::auth::session::CurrentUser;
use crate::domain::league::clean_league_name;
use crate::domain::{Event, EventId, LeagueId, Role, Stage, StageOrdinal, User, UserId};
use crate::error::{ApiError, ApiResult};
use crate::league_matches;
use crate::state::AppState;
use crate::store::events as event_store;
use crate::store::leagues::{self, League, Membership};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/leagues", post(create).get(list_mine))
        .route("/leagues/{id}", get(show))
        .route("/leagues/{id}/stage-points", put(set_stage_points))
        .route("/leagues/{id}/invite/reissue", post(reissue_invite))
        .route("/leagues/{id}/leave", post(leave))
        .route("/leagues/{id}/delete", post(delete_league))
        .route("/leagues/{id}/members/{user_id}", http_delete(kick))
        .route("/leagues/{id}/members/{user_id}/role", put(set_role))
        .route("/invites/{token}", get(preview_invite))
        .route("/invites/{token}/join", post(join))
}

/// The League and the caller's current membership in it, or 404 `not_found`.
/// The one door into a League for every Member-only endpoint.
pub async fn require_member(
    state: &AppState,
    user: &User,
    id: LeagueId,
) -> ApiResult<(League, Membership)> {
    leagues::member_of(&state.db, id, user.id)
        .await?
        .ok_or(ApiError::not_found())
}

/// [`require_member`] that also demands the League Admin role (403 otherwise).
pub async fn require_admin(
    state: &AppState,
    user: &User,
    id: LeagueId,
) -> ApiResult<(League, Membership)> {
    let (league, membership) = require_member(state, user, id).await?;
    if !membership.role.is_admin() {
        return Err(ApiError::forbidden());
    }
    Ok((league, membership))
}

/// The read-only guard (#17). Every write that changes what a League's game
/// holds (Predictions, Pick'ems, Overrides, joining, kicking) calls this with
/// the League from [`require_member`] and returns early on the error: an
/// archived League answers 409 `league_archived`. Leaving and deleting do not
/// call it. Call it before validating the body, so the answer does not depend
/// on the payload.
pub fn require_writable(league: &League) -> ApiResult<()> {
    if league.archived_at.is_some() {
        return Err(ApiError::conflict("league_archived"));
    }
    Ok(())
}

fn new_invite_token() -> String {
    hex::encode(rand::random::<[u8; 24]>())
}

async fn event_of(state: &AppState, league: &League) -> ApiResult<Event> {
    event_store::find(&state.db, league.event_id)
        .await?
        .ok_or_else(|| ApiError::internal("league without event"))
}

#[derive(Serialize)]
struct MemberView {
    user_id: crate::domain::UserId,
    nickname: String,
    avatar_url: Option<String>,
    role: Role,
    joined_at: DateTime<Utc>,
}

#[derive(Serialize)]
struct LeagueDetail {
    id: LeagueId,
    name: String,
    event: Event,
    my_role: Role,
    creator_id: crate::domain::UserId,
    is_creator: bool,
    archived_at: Option<DateTime<Utc>>,
    members: Vec<MemberView>,
    stages: Vec<Stage>,
    matches_pending: bool,
    stage_points: Vec<StagePoints>,
    /// Only League Admins get the Invite Link secret.
    #[serde(skip_serializing_if = "Option::is_none")]
    invite_token: Option<String>,
}

async fn detail(
    state: &AppState,
    viewer: UserId,
    league: League,
    role: Role,
) -> ApiResult<LeagueDetail> {
    let event = event_of(state, &league).await?;
    let members = leagues::members(&state.db, league.id)
        .await?
        .into_iter()
        .map(|m| MemberView {
            user_id: m.user_id,
            nickname: m.nickname,
            avatar_url: m.avatar_url,
            role: m.role,
            joined_at: m.joined_at,
        })
        .collect();
    let view = league_matches::effective_view(&state.db, &league).await?;
    let stage_points = sqlx::query_as(
        "SELECT stage, points FROM league_stage_points WHERE league_id = ? ORDER BY stage",
    )
    .bind(league.id)
    .fetch_all(&state.db)
    .await?;
    Ok(LeagueDetail {
        matches_pending: view.matches.is_empty(),
        stages: view.stages,
        stage_points,
        id: league.id,
        name: league.name,
        event,
        my_role: role,
        creator_id: league.creator_id,
        is_creator: league.creator_id == viewer,
        archived_at: league.archived_at,
        members,
        invite_token: role.is_admin().then_some(league.invite_token),
    })
}

#[derive(Deserialize, Serialize, sqlx::FromRow)]
struct StagePoints {
    stage: StageOrdinal,
    points: u32,
}

async fn set_stage_points(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<LeagueId>,
    Json(settings): Json<Vec<StagePoints>>,
) -> ApiResult<StatusCode> {
    let (league, _) = require_member(&state, &user, id).await?;
    if league.creator_id != user.id {
        return Err(ApiError::forbidden());
    }
    let mut tx = crate::db::begin_write(&state.db).await?;
    let (view, _) = league_matches::effective_view_and_overrides(&mut tx, &league).await?;
    let mut seen = std::collections::HashSet::new();
    for entry in &settings {
        if entry.points > 1000 || view.stage(entry.stage).is_none() || !seen.insert(entry.stage) {
            return Err(ApiError::unprocessable("stage_points_invalid"));
        }
    }
    for entry in settings {
        sqlx::query(
            "INSERT INTO league_stage_points (league_id, stage, points) VALUES (?, ?, ?)
             ON CONFLICT (league_id, stage) DO UPDATE SET points = excluded.points",
        )
        .bind(id)
        .bind(entry.stage)
        .bind(entry.points)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
struct NewLeague {
    event_id: EventId,
    name: String,
}

async fn create(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(body): Json<NewLeague>,
) -> ApiResult<(StatusCode, Json<LeagueDetail>)> {
    let name =
        clean_league_name(&body.name).ok_or(ApiError::unprocessable("league_name_invalid"))?;
    let event = match event_store::find(&state.db, body.event_id).await? {
        Some(event) => event,
        None => {
            let event = events::fetch(&state, body.event_id).await?.event;
            event_store::upsert(&state.db, &event).await?;
            event
        }
    };
    let league = leagues::create(
        &state.db,
        event.id,
        &name,
        user.id,
        &new_invite_token(),
        state.clock.now(),
    )
    .await?;
    let detail = detail(&state, user.id, league, Role::Admin).await?;
    Ok((StatusCode::CREATED, Json(detail)))
}

#[derive(Serialize)]
struct LeagueSummary {
    id: LeagueId,
    name: String,
    event: Event,
    my_role: Role,
    archived_at: Option<DateTime<Utc>>,
    /// My place on the Leaderboard (shared places share the number) and my
    /// Match Points.
    my_place: Option<u32>,
    my_points: u32,
}

async fn list_mine(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> ApiResult<Json<Vec<LeagueSummary>>> {
    let mut out = Vec::new();
    for (league, role) in leagues::leagues_of(&state.db, user.id).await? {
        let matches = league_matches::evaluate(&state.db, &league, state.clock.now()).await?;
        let members: Vec<UserId> = leagues::members(&state.db, league.id)
            .await?
            .into_iter()
            .map(|m| m.user_id)
            .collect();
        let mine = leaderboard::ranking(&matches, &members)
            .into_iter()
            .find(|s| s.user_id == user.id);
        out.push(LeagueSummary {
            my_place: mine.as_ref().map(|s| s.place),
            my_points: mine.map_or(0, |s| s.points),
            id: league.id,
            name: league.name.clone(),
            event: event_of(&state, &league).await?,
            my_role: role,
            archived_at: league.archived_at,
        });
    }
    Ok(Json(out))
}

async fn show(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<LeagueId>,
) -> ApiResult<Json<LeagueDetail>> {
    let (league, membership) = require_member(&state, &user, id).await?;
    Ok(Json(
        detail(&state, user.id, league, membership.role).await?,
    ))
}

async fn reissue_invite(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<LeagueId>,
) -> ApiResult<Json<serde_json::Value>> {
    require_admin(&state, &user, id).await?;
    let token = new_invite_token();
    leagues::set_invite_token(&state.db, id, &token).await?;
    Ok(Json(serde_json::json!({ "invite_token": token })))
}

async fn leave(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<LeagueId>,
) -> ApiResult<StatusCode> {
    let (league, _) = require_member(&state, &user, id).await?;
    if league.creator_id == user.id {
        return Err(ApiError::conflict("creator_cannot_leave"));
    }
    leagues::end_membership(&state.db, id, user.id, state.clock.now()).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn kick(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path((id, target)): Path<(LeagueId, UserId)>,
) -> ApiResult<StatusCode> {
    let (league, _) = require_admin(&state, &user, id).await?;
    require_writable(&league)?;
    if target == league.creator_id {
        return Err(ApiError::conflict("creator_protected"));
    }
    if target == user.id {
        return Err(ApiError::conflict("use_leave"));
    }
    if !leagues::end_membership(&state.db, id, target, state.clock.now()).await? {
        return Err(ApiError::not_found());
    }
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
struct NewRole {
    role: Role,
}

async fn set_role(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path((id, target)): Path<(LeagueId, UserId)>,
    Json(body): Json<NewRole>,
) -> ApiResult<StatusCode> {
    let (league, _) = require_admin(&state, &user, id).await?;
    if target == league.creator_id {
        return Err(ApiError::conflict("creator_protected"));
    }
    if !leagues::set_role(&state.db, id, user.id, target, body.role).await? {
        return Err(ApiError::not_found());
    }
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
struct ConfirmDelete {
    confirm_name: String,
}

/// Creator only, and only with the League's name typed back as confirmation.
async fn delete_league(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<LeagueId>,
    Json(body): Json<ConfirmDelete>,
) -> ApiResult<StatusCode> {
    let (league, _) = require_member(&state, &user, id).await?;
    if league.creator_id != user.id {
        return Err(ApiError::forbidden());
    }
    if body.confirm_name.trim() != league.name {
        return Err(ApiError::unprocessable("confirmation_mismatch"));
    }
    leagues::delete(&state.db, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Serialize)]
struct InvitePreview {
    id: LeagueId,
    name: String,
    event: Event,
    member_count: i64,
    already_member: bool,
}

async fn league_for_token(state: &AppState, token: &str) -> ApiResult<League> {
    leagues::find_by_invite_token(&state.db, token)
        .await?
        .ok_or(ApiError::new(StatusCode::NOT_FOUND, "invite_invalid"))
}

async fn preview_invite(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(token): Path<String>,
) -> ApiResult<Json<InvitePreview>> {
    let league = league_for_token(&state, &token).await?;
    Ok(Json(InvitePreview {
        id: league.id,
        name: league.name.clone(),
        event: event_of(&state, &league).await?,
        member_count: leagues::member_count(&state.db, league.id).await?,
        already_member: leagues::membership(&state.db, league.id, user.id)
            .await?
            .is_some(),
    }))
}

async fn join(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(token): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let league = league_for_token(&state, &token).await?;
    require_writable(&league)?;
    leagues::add_member(&state.db, league.id, user.id, state.clock.now()).await?;
    Ok(Json(serde_json::json!({ "id": league.id })))
}
