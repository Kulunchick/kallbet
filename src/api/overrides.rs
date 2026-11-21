//! Overrides API (#13): League Admins correct their League's Matches; every
//! Member sees the history.
//!
//! - `GET /api/leagues/{id}/overrides` (Members) → [`OverridesView`]: the
//!   history, newest first (who, what, when), the Event's teams and Stages,
//!   and the hidden Matches.
//! - `POST /api/leagues/{id}/matches/{match_id}/overrides` (admins) with
//!   `{"field": "scheduled_start", "scheduled_start": ts}`,
//!   `{"field": "result", "winner": team_id}`, `{"field": "result", "void": true}`,
//!   `{"field": "teams", "team_a": id, "team_b": id}` or `{"field": "hidden"}`
//!   → 201 [`EntryView`]. A newer Override of the same field replaces the
//!   older one; the history keeps both.
//! - `DELETE /api/leagues/{id}/matches/{match_id}/overrides/{field}` (admins)
//!   removes the field's Override (the Data Sources' value applies again) →
//!   200 [`EntryView`] of the removal, which stays in the history.
//! - `POST /api/leagues/{id}/matches` (admins) adds a Match by hand with
//!   `{stage, team_a, team_b, scheduled_start, format, placement?}` → 201
//!   `{"id": <local negative id>}`.
//! - `GET /api/leagues/{id}/data-warnings` (admins) → the Event's Data Source
//!   conflicts and unmatched records, read-only.
//!
//! Errors: 404 `not_found` (non-Member, or no such Match in the League), 403
//! `forbidden` (not a League Admin), 409 `league_archived`, 422
//! `team_not_in_match`, `team_not_in_event`, `same_team`, `stage_not_found`,
//! `override_invalid`.
//!
//! Nothing here writes global data: a "team mapping fix" is a teams
//! Override in this League's effective view, never `external_ids::set_manual`.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::api::leagues::{require_admin, require_member, require_writable};
use crate::auth::session::CurrentUser;
use crate::domain::effective_match::EventView;
use crate::domain::{
    LeagueId, Match, MatchFormat, MatchId, MatchPlacement, MatchStatus, StageKind, StageOrdinal,
    TeamId, UserId,
};
use crate::error::{ApiError, ApiResult};
use crate::league_matches;
use crate::poller;
use crate::sources::fallback::mapping::IdKind;
use crate::state::AppState;
use crate::store::leagues::League;
use crate::store::overrides::{self, Entry, Field, NewEntry};
use crate::store::{events, external_ids};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/leagues/{id}/overrides", get(list))
        .route("/leagues/{id}/data-warnings", get(warnings))
        .route("/leagues/{id}/matches", post(add_match))
        .route("/leagues/{id}/matches/{match_id}/overrides", post(create))
        .route(
            "/leagues/{id}/matches/{match_id}/overrides/{field}",
            delete(remove),
        )
}

#[derive(Serialize, Clone)]
pub struct TeamView {
    pub id: TeamId,
    pub name: String,
}

#[derive(Serialize)]
pub struct StageRef {
    pub ordinal: StageOrdinal,
    pub kind: StageKind,
    pub name: String,
}

/// A Match as the history names it: its teams in this League.
#[derive(Serialize)]
pub struct MatchRef {
    pub id: MatchId,
    pub stage: StageOrdinal,
    pub team_a: Option<TeamView>,
    pub team_b: Option<TeamView>,
    pub scheduled_start: Option<DateTime<Utc>>,
    pub status: MatchStatus,
}

#[derive(Serialize)]
pub struct Author {
    pub user_id: UserId,
    pub nickname: String,
}

/// One history entry.
#[derive(Serialize)]
pub struct EntryView {
    pub id: i64,
    pub match_id: MatchId,
    /// `null` if the Match is no longer in the Event.
    #[serde(rename = "match")]
    pub game: Option<MatchRef>,
    pub field: Field,
    /// This entry removed the field's Override.
    pub cleared: bool,
    /// The value set (`null` fields are not part of this Override).
    pub scheduled_start: Option<DateTime<Utc>>,
    pub winner: Option<TeamView>,
    /// A result Override that makes the Match Void.
    pub void: bool,
    pub team_a: Option<TeamView>,
    pub team_b: Option<TeamView>,
    /// `null` if the author's account is gone.
    pub author: Option<Author>,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct OverridesView {
    /// Newest first.
    pub entries: Vec<EntryView>,
    /// The Event's teams (what a teams Override or a hand-added Match picks from).
    pub teams: Vec<TeamView>,
    pub stages: Vec<StageRef>,
    /// Matches hidden in this League (they are not in the Matches view).
    pub hidden: Vec<MatchRef>,
}

fn team_view(view: &EventView, id: TeamId) -> TeamView {
    TeamView {
        id,
        name: view
            .team(id)
            .map(|t| t.name.clone())
            .unwrap_or_else(|| id.to_string()),
    }
}

fn match_ref(view: &EventView, game: &Match) -> MatchRef {
    MatchRef {
        id: game.id,
        stage: game.stage,
        team_a: game.team_a.map(|t| team_view(view, t)),
        team_b: game.team_b.map(|t| team_view(view, t)),
        scheduled_start: game.scheduled_start,
        status: game.status,
    }
}

fn entry_view(view: &EventView, entry: &Entry) -> EntryView {
    let has_value = !entry.cleared;
    EntryView {
        id: entry.id,
        match_id: entry.match_id,
        game: view
            .match_by_id(entry.match_id)
            .map(|m| match_ref(view, &m.data)),
        field: entry.field,
        cleared: entry.cleared,
        scheduled_start: entry.scheduled_start,
        winner: entry.winner.map(|t| team_view(view, t)),
        void: has_value && entry.field == Field::Result && entry.winner.is_none(),
        team_a: entry.team_a.map(|t| team_view(view, t)),
        team_b: entry.team_b.map(|t| team_view(view, t)),
        author: entry
            .author_id
            .zip(entry.author_nickname.clone())
            .map(|(user_id, nickname)| Author { user_id, nickname }),
        created_at: entry.created_at,
    }
}

async fn list(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<LeagueId>,
) -> ApiResult<Json<OverridesView>> {
    let (league, _) = require_member(&state, &user, id).await?;
    let (view, active) = league_matches::admin_view(&state.db, &league).await?;
    let history = overrides::history(&state.db, league.id).await?;
    Ok(Json(OverridesView {
        entries: history.iter().rev().map(|e| entry_view(&view, e)).collect(),
        teams: view.teams.iter().map(|t| team_view(&view, t.id)).collect(),
        stages: view
            .stages
            .iter()
            .map(|s| StageRef {
                ordinal: s.ordinal,
                kind: s.kind,
                name: s.name.clone(),
            })
            .collect(),
        hidden: view
            .matches
            .iter()
            .filter(|m| active.of(m.data.id).hidden)
            .map(|m| match_ref(&view, &m.data))
            .collect(),
    }))
}

#[derive(Deserialize)]
#[serde(tag = "field", rename_all = "snake_case")]
enum Body {
    ScheduledStart {
        scheduled_start: DateTime<Utc>,
    },
    Result {
        #[serde(default)]
        winner: Option<TeamId>,
        #[serde(default)]
        void: bool,
    },
    Teams {
        team_a: TeamId,
        team_b: TeamId,
    },
    Hidden,
}

/// The admin, the League (writable) and the Match (also a hidden one).
async fn admin_and_match(
    state: &AppState,
    user: &crate::domain::User,
    id: LeagueId,
    match_id: MatchId,
) -> ApiResult<(League, EventView)> {
    let (league, _) = require_admin(state, user, id).await?;
    require_writable(&league)?;
    let (view, _) = league_matches::admin_view(&state.db, &league).await?;
    if view.match_by_id(match_id).is_none() {
        return Err(ApiError::not_found());
    }
    Ok((league, view))
}

fn check_teams(view: &EventView, a: TeamId, b: TeamId) -> ApiResult<()> {
    if view.team(a).is_none() || view.team(b).is_none() {
        return Err(ApiError::unprocessable("team_not_in_event"));
    }
    if a == b {
        return Err(ApiError::unprocessable("same_team"));
    }
    Ok(())
}

async fn create(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path((id, match_id)): Path<(LeagueId, MatchId)>,
    Json(body): Json<serde_json::Value>,
) -> ApiResult<(StatusCode, Json<EntryView>)> {
    let (league, view) = admin_and_match(&state, &user, id, match_id).await?;
    let body: Body =
        serde_json::from_value(body).map_err(|_| ApiError::unprocessable("override_invalid"))?;
    let game = &view.match_by_id(match_id).expect("checked").data;

    let (field, entry) = match body {
        Body::ScheduledStart { scheduled_start } => (
            Field::ScheduledStart,
            NewEntry {
                scheduled_start: Some(scheduled_start),
                ..NewEntry::default()
            },
        ),
        Body::Result { winner, void } => {
            match (winner, void) {
                (Some(team), false) if game.teams().contains(&Some(team)) => {}
                (Some(_), false) => return Err(ApiError::unprocessable("team_not_in_match")),
                (None, true) => {}
                _ => return Err(ApiError::unprocessable("override_invalid")),
            }
            (
                Field::Result,
                NewEntry {
                    winner,
                    ..NewEntry::default()
                },
            )
        }
        Body::Teams { team_a, team_b } => {
            check_teams(&view, team_a, team_b)?;
            (
                Field::Teams,
                NewEntry {
                    team_a: Some(team_a),
                    team_b: Some(team_b),
                    ..NewEntry::default()
                },
            )
        }
        Body::Hidden => (Field::Hidden, NewEntry::default()),
    };

    let now = state.clock.now();
    let entry_id =
        overrides::record(&state.db, league.id, match_id, field, &entry, user.id, now).await?;
    let response = after_change(&state, &league, entry_id, now).await?;
    Ok((StatusCode::CREATED, Json(response)))
}

async fn remove(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path((id, match_id, field)): Path<(LeagueId, MatchId, String)>,
) -> ApiResult<Json<EntryView>> {
    let (league, _) = admin_and_match(&state, &user, id, match_id).await?;
    let field = Field::from_code(&field).ok_or(ApiError::not_found())?;
    let history = overrides::history(&state.db, league.id).await?;
    let in_force = overrides::in_force(&history)
        .get(&match_id)
        .is_some_and(|o| o.fields().contains(&field.code()));
    if !in_force {
        return Err(ApiError::not_found());
    }
    let now = state.clock.now();
    let cleared = NewEntry {
        cleared: true,
        ..NewEntry::default()
    };
    let entry_id = overrides::record(
        &state.db, league.id, match_id, field, &cleared, user.id, now,
    )
    .await?;
    Ok(Json(after_change(&state, &league, entry_id, now).await?))
}

/// After any change: evaluate the League at once (Lock/Reveal stamps from
/// the new view), archive it if the change decided the final (#17), and
/// return the new history entry.
async fn after_change(
    state: &AppState,
    league: &League,
    entry_id: i64,
    now: DateTime<Utc>,
) -> ApiResult<EntryView> {
    let matches = league_matches::evaluate(&state.db, league, now).await?;
    poller::archive_if_final_decided(state, league, &matches, now).await?;
    let (view, _) = league_matches::admin_view(&state.db, league).await?;
    let history = overrides::history(&state.db, league.id).await?;
    let entry = history
        .iter()
        .find(|e| e.id == entry_id)
        .ok_or_else(|| ApiError::internal("override entry not stored"))?;
    Ok(entry_view(&view, entry))
}

#[derive(Deserialize)]
struct NewMatch {
    stage: StageOrdinal,
    team_a: TeamId,
    team_b: TeamId,
    scheduled_start: DateTime<Utc>,
    #[serde(default = "default_format")]
    format: MatchFormat,
    #[serde(default = "default_placement")]
    placement: MatchPlacement,
}

fn default_format() -> MatchFormat {
    MatchFormat::Bo3
}

fn default_placement() -> MatchPlacement {
    MatchPlacement::Unplaced
}

#[derive(Serialize)]
struct Added {
    id: MatchId,
}

async fn add_match(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<LeagueId>,
    Json(body): Json<serde_json::Value>,
) -> ApiResult<(StatusCode, Json<Added>)> {
    let (league, _) = require_admin(&state, &user, id).await?;
    require_writable(&league)?;
    let body: NewMatch =
        serde_json::from_value(body).map_err(|_| ApiError::unprocessable("override_invalid"))?;
    let (view, _) = league_matches::admin_view(&state.db, &league).await?;
    if view.stage(body.stage).is_none() {
        return Err(ApiError::unprocessable("stage_not_found"));
    }
    check_teams(&view, body.team_a, body.team_b)?;

    let now = state.clock.now();
    let id = MatchId(external_ids::allocate_local_ids(&state.db, IdKind::Match, 1).await?);
    let game = Match {
        id,
        stage: body.stage,
        placement: body.placement,
        team_a: Some(body.team_a),
        team_b: Some(body.team_b),
        format: body.format,
        scheduled_start: Some(body.scheduled_start),
        actual_start: None,
        status: MatchStatus::Upcoming,
        winner: None,
        score: None,
    };
    events::insert_added_match(&state.db, league.id, league.event_id, &game, now).await?;
    let entry = NewEntry {
        scheduled_start: game.scheduled_start,
        team_a: game.team_a,
        team_b: game.team_b,
        ..NewEntry::default()
    };
    overrides::record(&state.db, league.id, id, Field::Added, &entry, user.id, now).await?;
    league_matches::evaluate(&state.db, &league, now).await?;
    Ok((StatusCode::CREATED, Json(Added { id })))
}

#[derive(Serialize)]
struct Conflict {
    match_id: MatchId,
    #[serde(rename = "match")]
    game: Option<MatchRef>,
    kept_source: String,
    kept_winner: Option<TeamView>,
    other_source: String,
    /// `null`: the other source reported the Match as Void.
    other_winner: Option<TeamView>,
    detected_at: DateTime<Utc>,
}

#[derive(Serialize)]
struct Unmatched {
    source: String,
    kind: String,
    external_id: String,
    description: String,
    seen_at: DateTime<Utc>,
}

#[derive(Serialize)]
struct Warnings {
    conflicts: Vec<Conflict>,
    unmatched: Vec<Unmatched>,
}

/// The Event's data-quality flags (#9), for League Admins only. They are
/// about the shared base data, so they are read-only here: a League fixes
/// its own view with Overrides.
async fn warnings(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<LeagueId>,
) -> ApiResult<Json<Warnings>> {
    let (league, _) = require_admin(&state, &user, id).await?;
    let (view, _) = league_matches::admin_view(&state.db, &league).await?;
    let conflicts = external_ids::conflicts(&state.db, league.event_id)
        .await?
        .into_iter()
        .map(|c| Conflict {
            match_id: c.match_id,
            game: view
                .match_by_id(c.match_id)
                .map(|m| match_ref(&view, &m.data)),
            kept_source: c.kept_source,
            kept_winner: c.kept_winner.map(|t| team_view(&view, t)),
            other_source: c.other_source,
            other_winner: c.other_winner.map(|t| team_view(&view, t)),
            detected_at: c.detected_at,
        })
        .collect();
    let unmatched = external_ids::unmatched(&state.db, league.event_id)
        .await?
        .into_iter()
        .map(|u| Unmatched {
            source: u.source,
            kind: u.kind,
            external_id: u.external_id,
            description: u.description,
            seen_at: u.seen_at,
        })
        .collect();
    Ok(Json(Warnings {
        conflicts,
        unmatched,
    }))
}
