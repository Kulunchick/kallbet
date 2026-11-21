//! Leagues API: create, list mine, get, Invite Link, join, leave.
//! Kick, roles and delete belong to #12.
//!
//! Other areas that act inside a League call [`require_member`]: a User who
//! is not a Member gets the same 404 as for a League that does not exist.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::api::events;
use crate::auth::session::CurrentUser;
use crate::domain::league::clean_league_name;
use crate::domain::{Event, EventId, LeagueId, Role, User, UserId};
use crate::error::{ApiError, ApiResult};
use crate::state::AppState;
use crate::store::events as event_store;
use crate::store::leagues::{self, League, Membership};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/leagues", post(create).get(list_mine))
        .route("/leagues/{id}", get(show))
        .route("/leagues/{id}/invite/reissue", post(reissue_invite))
        .route("/leagues/{id}/leave", post(leave))
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
    let membership = leagues::membership(&state.db, id, user.id)
        .await?
        .ok_or(ApiError::not_found())?;
    let league = leagues::find(&state.db, id)
        .await?
        .ok_or(ApiError::not_found())?;
    Ok((league, membership))
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
    Ok(LeagueDetail {
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
    let snapshot = events::fetch(&state, body.event_id).await?;
    event_store::upsert(&state.db, &snapshot.event).await?;
    let league = leagues::create(
        &state.db,
        body.event_id,
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
}

async fn list_mine(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> ApiResult<Json<Vec<LeagueSummary>>> {
    let mut out = Vec::new();
    for (league, role) in leagues::leagues_of(&state.db, user.id).await? {
        out.push(LeagueSummary {
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
    leagues::add_member(&state.db, league.id, user.id, state.clock.now()).await?;
    Ok(Json(serde_json::json!({ "id": league.id })))
}
