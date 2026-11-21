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
//! `zero_three` and `advance` team-id lists, or `"playoffs"` (#15) with
//! `picks: [{rounds_before_final, slot, team_id}]`, the picked winner of every
//! bracket slot. The same endpoints serve both, by the Stage's kind. A
//! Playoffs GET also carries `bracket {ready, slots}`; PUT on a bracket whose
//! first-round teams are not all known answers 422 `pickem_bracket_not_ready`.
//!
//! The Pick'em Leaderboard (`GET /api/leagues/{id}/pickem-leaderboard`) is
//! served from here too, so that this ticket needs no edit in
//! `api/leaderboard.rs` (the Match Leaderboard's file).

use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqliteConnection;

use crate::api::leagues::{require_member, require_writable};
use crate::auth::session::CurrentUser;
use crate::db;
use crate::domain::effective_match::EventView;
use std::collections::HashMap;

use crate::domain::pickem::{self, Bracket, BracketPickem, Pick, SwissPickem};
use crate::domain::{LeagueId, Stage, StageKind, StageOrdinal, TeamId, UserId};
use crate::error::{ApiError, ApiResult};
use crate::league_matches;
use crate::state::AppState;
use crate::store::leagues::{self, League};
use crate::store::pickems;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/leagues/{id}/stages/{ordinal}/pickem", get(show).put(save))
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
    let mut points: HashMap<UserId, u32> = HashMap::new();
    for stage in view
        .stages
        .iter()
        .filter(|s| matches!(s.kind, StageKind::Swiss | StageKind::Playoffs))
    {
        let outcomes = pickem::swiss_outcomes(&view, stage.ordinal);
        let bracket = Bracket::of(&view, stage.ordinal);
        for stored in
            pickems::for_stage(&mut *state.db.acquire().await?, league.id, stage.ordinal).await?
        {
            *points.entry(stored.user_id).or_default() +=
                stored_points(stage.kind, &bracket, &outcomes, &stored.picks);
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

/// One picked slot winner of a bracket, in the request and the response.
#[derive(Serialize, Deserialize)]
struct BracketPickBody {
    rounds_before_final: u8,
    slot: u8,
    team_id: TeamId,
}

#[derive(Deserialize)]
struct BracketBody {
    picks: Vec<BracketPickBody>,
}

impl From<BracketBody> for BracketPickem {
    fn from(b: BracketBody) -> BracketPickem {
        BracketPickem {
            winners: b
                .picks
                .into_iter()
                .map(|p| (p.rounds_before_final, p.slot, p.team_id))
                .collect(),
        }
    }
}

/// A saved Pick'em, tagged by `shape`.
#[derive(Serialize)]
#[serde(tag = "shape", rename_all = "lowercase")]
enum PickemJson {
    Swiss {
        #[serde(flatten)]
        swiss: SwissBody,
        updated_at: DateTime<Utc>,
    },
    Playoffs {
        picks: Vec<BracketPickBody>,
        updated_at: DateTime<Utc>,
    },
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
struct BracketSlotJson {
    rounds_before_final: u8,
    slot: u8,
    /// Teams known from the data (a first slot's two teams, or the teams the
    /// Data Source has since placed in a later slot); the rest come from
    /// the Member's own picks.
    team_a: Option<TeamId>,
    team_b: Option<TeamId>,
    /// A first slot: its teams are not picked winners of earlier slots.
    leaf: bool,
    /// Who really won this slot, once decided.
    winner: Option<TeamId>,
}

#[derive(Serialize)]
struct BracketJson {
    /// False while a first-round slot's teams are still TBD: the Pick'em
    /// cannot be filled yet (it still Locks at the first Match start).
    ready: bool,
    slots: Vec<BracketSlotJson>,
}

#[derive(Serialize)]
struct PickemView {
    stage: StageInfo,
    /// Swiss: the Stage's teams. Playoffs: the teams of the first slots.
    teams: Vec<TeamRef>,
    lock: LockJson,
    my_pickem: Option<PickemJson>,
    /// Every current Member's Pick'em: `null` until the Stage's Lock.
    pickems: Option<Vec<MemberPickem>>,
    /// The Playoffs Stage's bracket; absent for a Swiss Stage.
    #[serde(skip_serializing_if = "Option::is_none")]
    bracket: Option<BracketJson>,
}

/// The Stage if it has a Pick'em: a Swiss Stage or the Playoffs. Group Stages
/// have none, so an Event without a Swiss Stage offers the Playoffs only.
fn pickem_stage(view: &EventView, ordinal: StageOrdinal) -> ApiResult<&Stage> {
    view.stage(ordinal)
        .filter(|s| matches!(s.kind, StageKind::Swiss | StageKind::Playoffs))
        .ok_or(ApiError::not_found())
}

fn swiss_json(p: SwissPickem, updated_at: DateTime<Utc>) -> PickemJson {
    PickemJson::Swiss {
        swiss: SwissBody {
            three_nil: p.three_nil,
            zero_three: p.zero_three,
            advance: p.advance,
        },
        updated_at,
    }
}

fn bracket_json(p: BracketPickem, updated_at: DateTime<Utc>) -> PickemJson {
    PickemJson::Playoffs {
        picks: p
            .winners
            .into_iter()
            .map(|(rounds_before_final, slot, team_id)| BracketPickBody {
                rounds_before_final,
                slot,
                team_id,
            })
            .collect(),
        updated_at,
    }
}

/// A stored Pick'em in the shape of its Stage.
fn stored_json(
    kind: StageKind,
    bracket: &Bracket,
    picks: &[Pick],
    updated_at: DateTime<Utc>,
) -> PickemJson {
    match kind {
        StageKind::Playoffs => bracket_json(BracketPickem::from_picks(picks, bracket), updated_at),
        _ => swiss_json(SwissPickem::from_picks(picks), updated_at),
    }
}

/// A stored Pick'em's Points, by the shape of its Stage.
fn stored_points(
    kind: StageKind,
    bracket: &Bracket,
    outcomes: &HashMap<TeamId, pickem::SwissOutcome>,
    picks: &[Pick],
) -> u32 {
    match kind {
        StageKind::Playoffs => {
            pickem::score_bracket(&BracketPickem::from_picks(picks, bracket), bracket)
        }
        _ => pickem::score_swiss(&SwissPickem::from_picks(picks), outcomes),
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
    let mut conn = state.db.acquire().await?;
    let locked_at = pickems::evaluate_lock(&mut conn, league.id, view, ordinal, now).await?;
    let all = pickems::for_stage(&mut conn, league.id, ordinal).await?;
    drop(conn);
    let bracket = Bracket::of(view, ordinal);
    let outcomes = pickem::swiss_outcomes(view, ordinal);

    let mine = all
        .iter()
        .find(|p| p.user_id == user.id)
        .map(|p| stored_json(stage.kind, &bracket, &p.picks, p.updated_at));
    // Other Members' Pick'ems exist in the response only after Lock.
    let others = if locked_at.is_some() {
        let members = leagues::members(&state.db, league.id).await?;
        Some(
            members
                .into_iter()
                .filter_map(|m| {
                    let p = all.iter().find(|p| p.user_id == m.user_id)?;
                    Some(MemberPickem {
                        user_id: m.user_id,
                        nickname: m.nickname,
                        avatar_url: m.avatar_url,
                        pickem: stored_json(stage.kind, &bracket, &p.picks, p.updated_at),
                        points: stored_points(stage.kind, &bracket, &outcomes, &p.picks),
                    })
                })
                .collect(),
        )
    } else {
        None
    };
    let team_ids = match stage.kind {
        StageKind::Playoffs => bracket.teams(),
        _ => pickem::swiss_teams(view, ordinal),
    };
    let teams = team_ids
        .into_iter()
        .filter_map(|id| view.team(id))
        .map(|t| TeamRef {
            id: t.id,
            name: t.name.clone(),
        })
        .collect();
    let bracket_view = (stage.kind == StageKind::Playoffs).then(|| BracketJson {
        ready: bracket.ready(),
        slots: bracket
            .slots
            .iter()
            .map(|s| BracketSlotJson {
                rounds_before_final: s.rounds_before_final,
                slot: s.slot,
                team_a: s.teams[0],
                team_b: s.teams[1],
                leaf: s.leaf,
                winner: s.winner,
            })
            .collect(),
    });
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
        bracket: bracket_view,
    }))
}

async fn save(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path((id, ordinal)): Path<(LeagueId, StageOrdinal)>,
    Json(body): Json<serde_json::Value>,
) -> ApiResult<Json<PickemJson>> {
    let (league, _) = require_member(&state, &user, id).await?;
    require_writable(&league)?;
    // Check and write are one write transaction (ADR-0003): no Lock can be
    // stored between them, and `now` is read once it holds the write lock.
    let mut tx = db::begin_write(&state.db).await?;
    let now = state.clock.now();
    let result = save_in(&mut tx, &league, user.id, ordinal, body, now).await;
    db::finish(tx, result).await.map(Json)
}

async fn save_in(
    db: &mut SqliteConnection,
    league: &League,
    user: UserId,
    ordinal: StageOrdinal,
    body: serde_json::Value,
    now: DateTime<Utc>,
) -> ApiResult<PickemJson> {
    let matches = league_matches::evaluate_in(db, league, now).await?;
    let view = &matches.view;
    let stage = pickem_stage(view, ordinal)?;
    // Lock is re-checked on the server, so a stale page cannot bypass it.
    if pickems::evaluate_lock(db, league.id, view, ordinal, now)
        .await?
        .is_some()
    {
        return Err(ApiError::conflict("pickem_locked"));
    }
    // A body of the wrong shape for the Stage is a wrong count of picks.
    let malformed = || ApiError::unprocessable(pickem::PickemError::WrongCount.code());
    let (picks, saved) = match stage.kind {
        StageKind::Playoffs => {
            let bracket = Bracket::of(view, ordinal);
            let picks: BracketPickem = serde_json::from_value::<BracketBody>(body)
                .map_err(|_| malformed())?
                .into();
            picks
                .validate(&bracket)
                .map_err(|e| ApiError::unprocessable(e.code()))?;
            (picks.to_picks(), bracket_json(picks, now))
        }
        _ => {
            let picks: SwissPickem = serde_json::from_value::<SwissBody>(body)
                .map_err(|_| malformed())?
                .into();
            picks
                .validate(&pickem::swiss_teams(view, ordinal))
                .map_err(|e| ApiError::unprocessable(e.code()))?;
            (picks.to_picks(), swiss_json(picks, now))
        }
    };
    if !pickems::save(db, league.id, ordinal, user, &picks, now).await? {
        return Err(ApiError::conflict("pickem_locked"));
    }
    Ok(saved)
}
