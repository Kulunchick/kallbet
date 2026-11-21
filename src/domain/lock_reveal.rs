//! Lock/Reveal policy (pure). See ADR-0003 and `GLOSSARY.md`.
//!
//! Inputs: one Match of the effective Match view, the League's memberships,
//! who has predicted, the stored Lock and Reveal timestamps, and now. Output:
//! whether the Match is locked or revealed, and the transitions to persist.
//!
//! Lock happens at Scheduled Start minus [`LOCK_BEFORE_START`], or at the
//! actual start if that is earlier, recomputed from the latest Scheduled
//! Start until it happens. Once stored it is never undone: a later
//! postponement does not reopen the Match.
//!
//! #5 decides Lock. #6 adds Reveal to [`evaluate`] (all Eligible Members
//! predicted → Reveal, which also Locks; otherwise Reveal at Lock), using
//! `members` and `predicted`, which #5 already passes in.

use chrono::{DateTime, Duration, Utc};

use super::{Match, MatchStatus, UserId};

/// Lock is this long before the Scheduled Start.
pub const LOCK_BEFORE_START: Duration = Duration::minutes(15);

/// One stretch of a User's membership in the League (`left_at` is `None`
/// while still a Member). Eligible Members are derived from these.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemberSpan {
    pub user_id: UserId,
    pub joined_at: DateTime<Utc>,
    pub left_at: Option<DateTime<Utc>>,
}

/// The per-League timestamps stored for a Match. Set once, never cleared.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Stamps {
    pub locked_at: Option<DateTime<Utc>>,
    pub revealed_at: Option<DateTime<Utc>>,
}

pub struct Inputs<'a> {
    /// The Match as the League sees it (effective Match view).
    pub game: &'a Match,
    /// Every membership stretch in the League.
    pub members: &'a [MemberSpan],
    /// Users who have a Prediction for this Match.
    pub predicted: &'a [UserId],
    pub stored: Stamps,
    pub now: DateTime<Utc>,
}

/// Something that just happened and must be stored.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transition {
    Locked(DateTime<Utc>),
    Revealed(DateTime<Utc>),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Outcome {
    /// The stamps after this evaluation (stored ones plus `transitions`).
    pub stamps: Stamps,
    /// When the Match Locks (or Locked), for the countdown: the stored Lock,
    /// else Scheduled Start minus 15 minutes. `None` without a Scheduled
    /// Start.
    pub lock_time: Option<DateTime<Utc>>,
    /// New stamps to persist; empty when nothing changed.
    pub transitions: Vec<Transition>,
}

impl Outcome {
    pub fn is_locked(&self) -> bool {
        self.stamps.locked_at.is_some()
    }

    pub fn is_revealed(&self) -> bool {
        self.stamps.revealed_at.is_some()
    }
}

/// The Lock that the schedule asks for: Scheduled Start minus 15 minutes.
pub fn scheduled_lock(game: &Match) -> Option<DateTime<Utc>> {
    game.scheduled_start.map(|start| start - LOCK_BEFORE_START)
}

pub fn evaluate(inputs: &Inputs<'_>) -> Outcome {
    let Inputs {
        game, stored, now, ..
    } = *inputs;
    let mut stamps = stored;
    let mut transitions = Vec::new();

    if stamps.locked_at.is_none()
        && let Some(at) = lock_due(game, now)
    {
        stamps.locked_at = Some(at);
        transitions.push(Transition::Locked(at));
    }

    Outcome {
        stamps,
        lock_time: stamps.locked_at.or_else(|| scheduled_lock(game)),
        transitions,
    }
}

/// When the Match Locked, if it should be locked by `now`: the earliest of
/// the scheduled Lock and the actual start that is not in the future. A
/// started Match without a known start time, or a cancelled one, Locks now.
fn lock_due(game: &Match, now: DateTime<Utc>) -> Option<DateTime<Utc>> {
    let started = game
        .has_started()
        .then(|| game.actual_start.unwrap_or(now).min(now));
    let cancelled = (game.status == MatchStatus::Cancelled).then_some(now);
    let scheduled = scheduled_lock(game).filter(|at| *at <= now);
    [started, cancelled, scheduled].into_iter().flatten().min()
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;
    use crate::domain::{MatchFormat, MatchId, MatchPlacement, StageOrdinal};

    fn at(h: u32, m: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 1, 10, h, m, 0).unwrap()
    }

    fn game(start: DateTime<Utc>) -> Match {
        Match {
            id: MatchId(1),
            stage: StageOrdinal(1),
            placement: MatchPlacement::Unplaced,
            team_a: None,
            team_b: None,
            format: MatchFormat::Bo3,
            scheduled_start: Some(start),
            actual_start: None,
            status: MatchStatus::Upcoming,
            winner: None,
            score: None,
        }
    }

    fn run(game: &Match, stored: Stamps, now: DateTime<Utc>) -> Outcome {
        evaluate(&Inputs {
            game,
            members: &[],
            predicted: &[],
            stored,
            now,
        })
    }

    #[test]
    fn open_until_fifteen_minutes_before_the_start() {
        let g = game(at(14, 0));
        let open = run(&g, Stamps::default(), at(13, 44));
        assert!(!open.is_locked());
        assert_eq!(open.lock_time, Some(at(13, 45)));
        assert!(open.transitions.is_empty());

        let locked = run(&g, Stamps::default(), at(13, 45));
        assert_eq!(locked.transitions, [Transition::Locked(at(13, 45))]);
    }

    #[test]
    fn a_late_evaluation_stamps_the_scheduled_lock_time() {
        let locked = run(&game(at(14, 0)), Stamps::default(), at(15, 0));
        assert_eq!(locked.stamps.locked_at, Some(at(13, 45)));
    }

    #[test]
    fn an_early_actual_start_locks_at_that_start() {
        let mut g = game(at(14, 0));
        g.status = MatchStatus::Live;
        g.actual_start = Some(at(12, 30));
        assert_eq!(
            run(&g, Stamps::default(), at(12, 31)).stamps.locked_at,
            Some(at(12, 30))
        );
    }

    #[test]
    fn a_stored_lock_survives_a_postponement() {
        let stored = Stamps {
            locked_at: Some(at(13, 45)),
            revealed_at: None,
        };
        let out = run(&game(at(18, 0)), stored, at(14, 0));
        assert!(out.is_locked());
        assert_eq!(out.lock_time, Some(at(13, 45)));
        assert!(out.transitions.is_empty());
    }

    #[test]
    fn without_a_scheduled_start_a_match_stays_open() {
        let mut g = game(at(14, 0));
        g.scheduled_start = None;
        let out = run(&g, Stamps::default(), at(20, 0));
        assert!(!out.is_locked());
        assert_eq!(out.lock_time, None);
    }

    #[test]
    fn a_cancelled_match_locks() {
        let mut g = game(at(20, 0));
        g.status = MatchStatus::Cancelled;
        assert!(run(&g, Stamps::default(), at(12, 0)).is_locked());
    }
}
