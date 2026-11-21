//! Pick'em API: get and upsert a Member's Pick'em per Stage.
//!
//! - `GET /api/leagues/{id}/stages/{ordinal}/pickem` → the Stage's team list,
//!   its Lock, the caller's own Pick'em and, only after Lock, every Member's.
//! - `PUT /api/leagues/{id}/stages/{ordinal}/pickem` → saves it. Errors: 404
//!   `not_found` (not a Member, or no Pick'em for that Stage), 409
//!   `pickem_locked`, 422 `pickem_wrong_count`, `pickem_duplicate_team`,
//!   `pickem_team_not_in_stage`.
//!
//! The Pick'em JSON carries a `shape`: `"swiss"` (#14) with `three_nil`,
//! `zero_three` and `advance` team-id lists. #15 adds the `"playoffs"` shape
//! (a bracket) as a second variant of the same endpoints, and returns it from
//! the same handlers by Stage kind.
//!
//! The Pick'em Leaderboard (`GET /api/leagues/{id}/pickem-leaderboard`) is
//! served from here too, so that this ticket needs no edit in
//! `api/leaderboard.rs` (the Match Leaderboard's file).

use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::api::leagues::{require_member, require_writable};
use crate::auth::session::CurrentUser;
use crate::domain::effective_match::EventView;
use crate::domain::pickem::{self, SwissPickem};
use crate::domain::{LeagueId, Stage, StageKind, StageOrdinal, TeamId, UserId};
use crate::error::{ApiError, ApiResult};
use crate::league_matches;
use crate::state::AppState;
use crate::store::{leagues, pickems};

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/leagues/{id}/stages/{ordinal}/pickem",
            get(show).put(save),
        )
        .route("/leagues/{id}/pickem-leaderboard", get(leaderboard))
}

#[derive(Serialize)]
struct LeaderboardEntry {
    place: u32,
    user_id: UserId,
    nickname: String,
    avatar_url: Option<String>,
    points: u32,
}

#[derive(Serialize)]
struct Leaderboard {
    entries: Vec<LeaderboardEntry>,
}

/// The Pick'em Leaderboard: every current Member ranked by Pick'em Points
/// (all Stages together), equal Points sharing a place. Separate from the
/// Match Leaderboard: no Match Prediction counts here. It holds only Points,
/// so it reveals nothing about unrevealed Picks (a Stage scores only once its
/// Matches have finished, so after its Lock).
async fn leaderboard(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<LeagueId>,
) -> ApiResult<Json<Leaderboard>> {
    let (league, _) = require_member(&state, &user, id).await?;
    let view = league_matches::effective_view(&state.db, &league).await?;
    let mut points: std::collections::HashMap<UserId, u32> = std::collections::HashMap::new();
    // #15: add the Playoffs Stage's scoring here, in the same sum.
    for stage in view.stages.iter().filter(|s| s.kind == StageKind::Swiss) {
        let outcomes = pickem::swiss_outcomes(&view, stage.ordinal);
        for stored in pickems::for_stage(&state.db, league.id, stage.ordinal).await? {
            let swiss = SwissPickem::from_picks(&stored.picks);
            *points.entry(stored.user_id).or_default() += pickem::score_swiss(&swiss, &outcomes);
        }
    }
    let mut members = leagues::members(&state.db, league.id).await?;
    members.sort_by_key(|m| m.nickname.to_lowercase());
    let entries = pickem::rank(
        members
            .into_iter()
            .map(|m| {
                let p = points.get(&m.user_id).copied().unwrap_or(0);
                (m, p)
            })
            .collect(),
    )
    .into_iter()
    .map(|(place, m, points)| LeaderboardEntry {
        place,
        user_id: m.user_id,
        nickname: m.nickname,
        avatar_url: m.avatar_url,
        points,
    })
    .collect();
    Ok(Json(Leaderboard { entries }))
}

#[derive(Serialize, Deserialize)]
struct SwissBody {
    three_nil: Vec<TeamId>,
    zero_three: Vec<TeamId>,
    advance: Vec<TeamId>,
}

impl From<SwissBody> for SwissPickem {
    fn from(b: SwissBody) -> SwissPickem {
        SwissPickem {
            three_nil: b.three_nil,
            zero_three: b.zero_three,
            advance: b.advance,
        }
    }
}

#[derive(Serialize)]
struct PickemJson {
    shape: &'static str,
    #[serde(flatten)]
    swiss: SwissBody,
    updated_at: DateTime<Utc>,
}

#[derive(Serialize)]
struct TeamRef {
    id: TeamId,
    name: String,
}

#[derive(Serialize)]
struct StageInfo {
    ordinal: StageOrdinal,
    kind: StageKind,
    name: String,
}

#[derive(Serialize)]
struct LockJson {
    /// When the Lock is expected: the Scheduled Start of the Stage's first Match.
    locks_at: Option<DateTime<Utc>>,
    locked_at: Option<DateTime<Utc>>,
    locked: bool,
}

/// Another Member's Pick'em. Only ever sent after Lock.
#[derive(Serialize)]
struct MemberPickem {
    user_id: UserId,
    nickname: String,
    avatar_url: Option<String>,
    #[serde(flatten)]
    pickem: PickemJson,
    points: u32,
}

#[derive(Serialize)]
struct PickemView {
    stage: StageInfo,
    teams: Vec<TeamRef>,
    lock: LockJson,
    my_pickem: Option<PickemJson>,
    /// Every current Member's Pick'em: `null` until the Stage's Lock.
    pickems: Option<Vec<MemberPickem>>,
}

/// The Stage if it has a Swiss Pick'em (the only shape until #15).
fn pickem_stage(view: &EventView, ordinal: StageOrdinal) -> ApiResult<&Stage> {
    view.stage(ordinal)
        .filter(|s| s.kind == StageKind::Swiss)
        .ok_or(ApiError::not_found())
}

fn swiss_json(p: SwissPickem, updated_at: DateTime<Utc>) -> PickemJson {
    PickemJson {
        shape: "swiss",
        swiss: SwissBody {
            three_nil: p.three_nil,
            zero_three: p.zero_three,
            advance: p.advance,
        },
        updated_at,
    }
}

async fn show(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path((id, ordinal)): Path<(LeagueId, StageOrdinal)>,
) -> ApiResult<Json<PickemView>> {
    let (league, _) = require_member(&state, &user, id).await?;
    let now = state.clock.now();
    let matches = league_matches::evaluate(&state.db, &league, now).await?;
    let view = &matches.view;
    let stage = pickem_stage(view, ordinal)?;
    let locked_at = pickems::evaluate_lock(&state.db, league.id, view, ordinal, now).await?;
    let all = pickems::for_stage(&state.db, league.id, ordinal).await?;

    let mine = all
        .iter()
        .find(|p| p.user_id == user.id)
        .map(|p| swiss_json(SwissPickem::from_picks(&p.picks), p.updated_at));
    // Other Members' Pick'ems exist in the response only after Lock.
    let others = if locked_at.is_some() {
        let outcomes = pickem::swiss_outcomes(view, ordinal);
        let members = leagues::members(&state.db, league.id).await?;
        Some(
            members
                .into_iter()
                .filter_map(|m| {
                    let p = all.iter().find(|p| p.user_id == m.user_id)?;
                    let swiss = SwissPickem::from_picks(&p.picks);
                    let points = pickem::score_swiss(&swiss, &outcomes);
                    Some(MemberPickem {
                        user_id: m.user_id,
                        nickname: m.nickname,
                        avatar_url: m.avatar_url,
                        pickem: swiss_json(swiss, p.updated_at),
                        points,
                    })
                })
                .collect(),
        )
    } else {
        None
    };
    let teams = pickem::swiss_teams(view, ordinal)
        .into_iter()
        .filter_map(|id| view.team(id))
        .map(|t| TeamRef {
            id: t.id,
            name: t.name.clone(),
        })
        .collect();
    Ok(Json(PickemView {
        stage: StageInfo {
            ordinal,
            kind: stage.kind,
            name: stage.name.clone(),
        },
        teams,
        lock: LockJson {
            locks_at: pickem::expected_lock(view, ordinal),
            locked_at,
            locked: locked_at.is_some(),
        },
        my_pickem: mine,
        pickems: others,
    }))
}

async fn save(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path((id, ordinal)): Path<(LeagueId, StageOrdinal)>,
    Json(body): Json<SwissBody>,
) -> ApiResult<Json<PickemJson>> {
    let (league, _) = require_member(&state, &user, id).await?;
    require_writable(&league)?;
    let now = state.clock.now();
    let matches = league_matches::evaluate(&state.db, &league, now).await?;
    let view = &matches.view;
    pickem_stage(view, ordinal)?;
    // Lock is re-checked on the server, so a stale page cannot bypass it.
    if pickems::evaluate_lock(&state.db, league.id, view, ordinal, now)
        .await?
        .is_some()
    {
        return Err(ApiError::conflict("pickem_locked"));
    }
    let picks = SwissPickem::from(body);
    picks
        .validate(&pickem::swiss_teams(view, ordinal))
        .map_err(|e| ApiError::unprocessable(e.code()))?;
    pickems::save(&state.db, league.id, ordinal, user.id, &picks.to_picks(), now).await?;
    Ok(Json(swiss_json(picks, now)))
}
