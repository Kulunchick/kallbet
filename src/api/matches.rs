//! Matches API: a League's effective Matches grouped by Stage and round
//! (Swiss record bucket, Playoffs round), with each Match's Lock state and
//! the caller's own Prediction.
//!
//! `GET /api/leagues/{id}/matches` → [`MatchesView`]. Each Match is a
//! [`MatchCard`]: #6 adds the Prediction count and the revealed Predictions
//! there, #7 the Points of the caller's Prediction. Never put another
//! Member's Prediction in it before Reveal.

use std::collections::BTreeMap;

use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::api::leagues::require_member;
use crate::auth::session::CurrentUser;
use crate::domain::scoring::{self, Verdict};
use crate::domain::{
    DataSourceKind, Event, LeagueId, MatchFormat, MatchId, MatchPlacement, MatchScore, MatchStatus,
    StageKind, StageOrdinal, TeamId, UserId,
};
use crate::error::ApiResult;
use crate::league_matches::{self, LeagueMatches};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/leagues/{id}/matches", get(list))
}

#[derive(Serialize)]
pub struct MatchesView {
    pub event: Event,
    /// In Event order. Stages without Matches are left out.
    pub stages: Vec<StageView>,
}

#[derive(Serialize)]
pub struct StageView {
    pub ordinal: StageOrdinal,
    pub kind: StageKind,
    pub name: String,
    /// Swiss: one group per record bucket (0-0, 1-0, 0-1, 2-0, ...).
    /// Playoffs: one group per round, from the earliest to the final.
    pub groups: Vec<GroupView>,
}

/// Which part of a Stage a group of Matches is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum GroupKey {
    /// A Swiss record bucket: the record both teams bring into the Match.
    Record {
        wins: u8,
        losses: u8,
    },
    /// A Playoffs round: 0 is the final, 1 the semifinals, 2 the quarterfinals.
    PlayoffRound {
        rounds_before_final: u8,
    },
    Unplaced,
}

impl GroupKey {
    fn of(placement: MatchPlacement) -> GroupKey {
        match placement {
            MatchPlacement::Swiss { wins, losses, .. } => GroupKey::Record { wins, losses },
            MatchPlacement::Playoffs {
                rounds_before_final,
                ..
            } => GroupKey::PlayoffRound {
                rounds_before_final,
            },
            MatchPlacement::Unplaced => GroupKey::Unplaced,
        }
    }

    /// Display order: Swiss rounds in order (and within a round the better
    /// record first), Playoffs rounds towards the final, unplaced last.
    fn order(self) -> GroupOrder {
        match self {
            GroupKey::Record { wins, losses } => (0, wins + losses, losses),
            GroupKey::PlayoffRound {
                rounds_before_final,
            } => (1, u8::MAX - rounds_before_final, 0),
            GroupKey::Unplaced => (2, 0, 0),
        }
    }
}

/// Sort key of a group within its Stage.
type GroupOrder = (u8, u8, u8);

#[derive(Serialize)]
pub struct GroupView {
    #[serde(flatten)]
    pub key: GroupKey,
    pub matches: Vec<MatchCard>,
}

#[derive(Serialize)]
pub struct TeamView {
    pub id: TeamId,
    pub name: String,
}

#[derive(Serialize)]
pub struct LockView {
    /// When the Match Locks (or Locked): the countdown target. `null`
    /// without a Scheduled Start.
    pub locks_at: Option<DateTime<Utc>>,
    /// The stored Lock timestamp; `null` while open.
    pub locked_at: Option<DateTime<Utc>>,
    pub locked: bool,
}

/// One Match as a Member sees it.
#[derive(Serialize)]
pub struct MatchCard {
    pub id: MatchId,
    /// `null` while the team is still to be decided.
    pub team_a: Option<TeamView>,
    pub team_b: Option<TeamView>,
    pub format: MatchFormat,
    pub scheduled_start: Option<DateTime<Utc>>,
    pub actual_start: Option<DateTime<Utc>>,
    pub status: MatchStatus,
    pub winner: Option<TeamId>,
    pub score: Option<MatchScore>,
    pub lock: LockView,
    /// The team the caller picked, if any.
    pub my_prediction: Option<TeamId>,
    // #6: `predictions: { count, eligible, revealed: [...] }` goes here.
    /// #7: how my Prediction scored: `pending` until the result is in, then
    /// `correct`, `wrong`, `missing` or `void`.
    pub my_result: Verdict,
    pub my_points: u32,
    /// Data freshness: where the data came from and when it was fetched.
    pub source: DataSourceKind,
    pub updated_at: DateTime<Utc>,
}

async fn list(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<LeagueId>,
) -> ApiResult<Json<MatchesView>> {
    let (league, _) = require_member(&state, &user, id).await?;
    let matches = league_matches::evaluate(&state.db, &league, state.clock.now()).await?;
    Ok(Json(view_for(&matches, user.id)))
}

/// Builds the Matches view of `viewer`.
pub fn view_for(matches: &LeagueMatches, viewer: UserId) -> MatchesView {
    let view = &matches.view;
    let team = |id: Option<TeamId>| {
        id.map(|id| TeamView {
            id,
            name: view
                .team(id)
                .map(|t| t.name.clone())
                .unwrap_or_else(|| id.to_string()),
        })
    };

    let mut stages = Vec::new();
    for stage in &view.stages {
        let mut groups: BTreeMap<(GroupOrder, GroupKey), Vec<MatchCard>> = BTreeMap::new();
        let mut in_stage: Vec<_> = view
            .matches
            .iter()
            .filter(|m| m.data.stage == stage.ordinal)
            .collect();
        in_stage.sort_by_key(|m| (slot_of(m.data.placement), m.data.scheduled_start, m.data.id));
        for m in in_stage {
            let game = &m.data;
            let key = GroupKey::of(game.placement);
            let outcome = matches.outcome(game.id).cloned().unwrap_or_default();
            let verdict = scoring::judge(
                game,
                matches.prediction_of(game.id, viewer).map(|p| p.team_id),
            );
            groups
                .entry((key.order(), key))
                .or_default()
                .push(MatchCard {
                    id: game.id,
                    team_a: team(game.team_a),
                    team_b: team(game.team_b),
                    format: game.format,
                    scheduled_start: game.scheduled_start,
                    actual_start: game.actual_start,
                    status: game.status,
                    winner: game.winner,
                    score: game.score,
                    lock: LockView {
                        locks_at: outcome.lock_time,
                        locked_at: outcome.stamps.locked_at,
                        locked: outcome.is_locked(),
                    },
                    my_prediction: matches.prediction_of(game.id, viewer).map(|p| p.team_id),
                    my_result: verdict,
                    my_points: verdict.points(),
                    source: m.source,
                    updated_at: m.updated_at,
                });
        }
        if groups.is_empty() {
            continue;
        }
        stages.push(StageView {
            ordinal: stage.ordinal,
            kind: stage.kind,
            name: stage.name.clone(),
            groups: groups
                .into_iter()
                .map(|((_, key), matches)| GroupView { key, matches })
                .collect(),
        });
    }

    MatchesView {
        event: view.event.clone(),
        stages,
    }
}

/// A Playoffs Match sorts by its bracket slot; others by time.
fn slot_of(placement: MatchPlacement) -> u8 {
    match placement {
        MatchPlacement::Playoffs { slot, .. } => slot,
        _ => 0,
    }
}
