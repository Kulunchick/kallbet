//! Matches API: a League's effective Matches grouped by Stage and round
//! (Swiss record bucket, Playoffs round), with each Match's Lock state and
//! the caller's own Prediction.
//!
//! `GET /api/leagues/{id}/matches` → [`MatchesView`]. Each Match is a
//! [`MatchCard`]: its `predictions` holds the Prediction count and, after
//! Reveal, everyone's Predictions (#6); #7 adds the Points of the caller's
//! Prediction. Never put another Member's Prediction in it before Reveal.

use std::collections::BTreeMap;

use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::api::leagues::require_member;
use crate::auth::session::CurrentUser;
use crate::domain::lock_reveal::Outcome;
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

/// A team as every API area shows it.
#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
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
    pub edit_window: bool,
    pub can_predict: bool,
}

/// The Predictions of a Match as a Member may see them (#6, ADR-0003).
#[derive(Serialize)]
pub struct PredictionsView {
    /// How many Eligible Members have predicted (N of "N of M predicted").
    pub count: usize,
    /// How many Eligible Members there are (M).
    pub eligible: usize,
    /// When the Predictions were revealed; `null` before Reveal.
    pub revealed_at: Option<DateTime<Utc>>,
    /// The Eligible Members' Predictions, `null` before Reveal. Never filled
    /// earlier: this is the only place other Members' Predictions leave the
    /// server.
    pub revealed: Option<Vec<RevealedPrediction>>,
}

#[derive(Serialize)]
pub struct RevealedPrediction {
    pub user_id: UserId,
    pub nickname: String,
    pub avatar_url: Option<String>,
    pub team_id: TeamId,
}

fn predictions_view(matches: &LeagueMatches, game: MatchId, outcome: &Outcome) -> PredictionsView {
    let made: Vec<_> = matches
        .predictions
        .iter()
        .filter(|p| p.match_id == game && outcome.eligible.contains(&p.user_id))
        .collect();
    let revealed = outcome.is_revealed().then(|| {
        made.iter()
            .filter_map(|p| {
                let member = matches.profiles.iter().find(|m| m.user_id == p.user_id)?;
                Some(RevealedPrediction {
                    user_id: p.user_id,
                    nickname: member.nickname.clone(),
                    avatar_url: member.avatar_url.clone(),
                    team_id: p.team_id,
                })
            })
            .collect()
    });
    PredictionsView {
        count: made.len(),
        eligible: outcome.eligible.len(),
        revealed_at: outcome.stamps.revealed_at,
        revealed,
    }
}

/// One Match as a Member sees it.
#[derive(Serialize)]
pub struct MatchCard {
    pub id: MatchId,
    /// `null` while the team is still to be decided.
    pub team_a: Option<TeamView>,
    pub team_b: Option<TeamView>,
    pub format: MatchFormat,
    /// A Playoffs Match's slot within its round (its group's
    /// `rounds_before_final`): the winners of slots 2n and 2n+1 meet in slot
    /// n of the next round. `null` outside the Playoffs.
    pub bracket_slot: Option<u8>,
    pub scheduled_start: Option<DateTime<Utc>>,
    pub actual_start: Option<DateTime<Utc>>,
    pub status: MatchStatus,
    pub winner: Option<TeamId>,
    pub score: Option<MatchScore>,
    pub lock: LockView,
    /// The team the caller picked, if any.
    pub my_prediction: Option<TeamId>,
    /// #6: "N of M predicted" and, once revealed, everyone's Predictions.
    pub predictions: PredictionsView,
    /// #7: how my Prediction scored: `pending` until the result is in, then
    /// `correct`, `wrong`, `missing` or `void`.
    pub my_result: Verdict,
    pub my_points: u32,
    /// Data freshness: where the data came from and when it was fetched.
    /// `null` for a Match a League Admin added by hand (#13).
    pub source: Option<DataSourceKind>,
    pub updated_at: DateTime<Utc>,
    /// #13: the fields a League Override sets on this Match
    /// (`scheduled_start`, `result`, `teams`), and whether it was added by hand.
    pub overridden: Vec<crate::domain::effective_match::Field>,
    pub added_by_hand: bool,
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
            let added = matches.overrides.is_added(game.id);
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
                    bracket_slot: match game.placement {
                        MatchPlacement::Playoffs { slot, .. } => Some(slot),
                        _ => None,
                    },
                    scheduled_start: game.scheduled_start,
                    actual_start: game.actual_start,
                    status: game.status,
                    winner: game.winner,
                    score: game.score,
                    lock: LockView {
                        locks_at: outcome.lock_time,
                        locked_at: outcome.stamps.locked_at,
                        locked: outcome.is_locked(),
                        can_predict: !outcome.is_locked()
                            && matches
                                .edit_windows
                                .get(&game.id)
                                .is_none_or(|(owner, _)| *owner == viewer),
                        edit_window: !outcome.is_locked()
                            && matches.edit_windows.contains_key(&game.id),
                    },
                    my_prediction: matches.prediction_of(game.id, viewer).map(|p| p.team_id),
                    predictions: predictions_view(matches, game.id, &outcome),
                    my_result: verdict,
                    my_points: verdict.points() * matches.points_for(game.stage),
                    source: (!added).then_some(m.source),
                    updated_at: m.updated_at,
                    overridden: matches.overrides.of(game.id).fields(),
                    added_by_hand: added,
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
