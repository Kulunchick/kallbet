//! A League's Matches: the effective Match view, evaluated against the
//! Lock/Reveal policy, with transitions persisted.
//!
//! The one place League logic gets its Matches from. The Matches API, the
//! Prediction endpoint and the poller all call [`evaluate`], so a Lock (or,
//! from #6, a Reveal) that any of them observes is stored at once and seen the
//! same way by all of them.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use sqlx::SqlitePool;

use crate::domain::effective_match::{self, EventView};
use crate::domain::lock_reveal::{self, MemberSpan, Outcome};
use crate::domain::{MatchId, StageKind, UserId};
use crate::store::events;
use crate::store::leagues::{self, League};
use crate::store::pickems;
use crate::store::predictions::{self, Prediction};

/// Everything about a League's Matches at one moment.
pub struct LeagueMatches {
    /// The effective Match view (base data + this League's Overrides).
    pub view: EventView,
    /// The Lock/Reveal outcome of every Match in `view`.
    pub outcomes: HashMap<MatchId, Outcome>,
    /// Every Prediction in the League. Never send other Members' ones to a
    /// client before Reveal.
    pub predictions: Vec<Prediction>,
    /// Every membership stretch, for Eligible Members.
    pub members: Vec<MemberSpan>,
}

impl LeagueMatches {
    pub fn outcome(&self, id: MatchId) -> Option<&Outcome> {
        self.outcomes.get(&id)
    }

    pub fn prediction_of(&self, id: MatchId, user: UserId) -> Option<&Prediction> {
        self.predictions
            .iter()
            .find(|p| p.match_id == id && p.user_id == user)
    }
}

/// The League's effective Match view. #13 loads the League's Overrides here
/// and passes them to [`effective_match::effective_view`].
pub async fn effective_view(db: &SqlitePool, league: &League) -> Result<EventView, sqlx::Error> {
    let base = events::base_view(db, league.event_id)
        .await?
        .ok_or_else(|| sqlx::Error::RowNotFound)?;
    Ok(effective_match::effective_view(base))
}

/// Loads the League's Matches, evaluates Lock/Reveal for each at `now`, and
/// stores any new Lock/Reveal stamps.
pub async fn evaluate(
    db: &SqlitePool,
    league: &League,
    now: DateTime<Utc>,
) -> Result<LeagueMatches, sqlx::Error> {
    let view = effective_view(db, league).await?;
    let members: Vec<MemberSpan> = leagues::all_memberships(db, league.id)
        .await?
        .into_iter()
        .map(|m| MemberSpan {
            user_id: m.user_id,
            joined_at: m.joined_at,
            left_at: m.left_at,
        })
        .collect();
    let predictions = predictions::for_league(db, league.id).await?;
    let stored = predictions::stamps(db, league.id).await?;

    let mut outcomes = HashMap::new();
    for m in &view.matches {
        let id = m.data.id;
        let predicted: Vec<UserId> = predictions
            .iter()
            .filter(|p| p.match_id == id)
            .map(|p| p.user_id)
            .collect();
        let outcome = lock_reveal::evaluate(&lock_reveal::Inputs {
            game: &m.data,
            members: &members,
            predicted: &predicted,
            stored: stored.get(&id).copied().unwrap_or_default(),
            now,
        });
        if !outcome.transitions.is_empty() {
            predictions::record(db, league.id, id, &outcome.transitions).await?;
        }
        outcomes.insert(id, outcome);
    }

    // Store each Stage Pick'em's Lock the moment it is due (#14), for the
    // same reason as the Match Locks above: a postponement must not undo it.
    for stage in view.stages.iter().filter(|s| s.kind == StageKind::Swiss) {
        pickems::evaluate_lock(db, league.id, &view, stage.ordinal, now).await?;
    }

    Ok(LeagueMatches {
        view,
        outcomes,
        predictions,
        members,
    })
}
