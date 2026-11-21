//! Circuit breaker for one Data Source: open after `failures` consecutive
//! failures, half-open (exactly one more try allowed) once `open_for` has
//! passed since it opened. A failed half-open try opens it again from that
//! moment; a success closes it. While the half-open try runs, other calls
//! are refused; a try that never reports back (its call was dropped) stops
//! counting after another `open_for`. Time comes from the caller's `Clock`.

use std::sync::Mutex;

use chrono::{DateTime, Duration, Utc};

pub struct Breaker {
    failures: u32,
    open_for: Duration,
    state: Mutex<State>,
}

#[derive(Default)]
struct State {
    consecutive: u32,
    opened_at: Option<DateTime<Utc>>,
    /// When the half-open try now running was let through.
    probe_since: Option<DateTime<Utc>>,
}

impl Breaker {
    pub fn new(failures: u32, open_for: Duration) -> Breaker {
        Breaker {
            failures: failures.max(1),
            open_for,
            state: Mutex::new(State::default()),
        }
    }

    /// Whether a call may go to the source at `now`: always while closed;
    /// once open, only the single half-open try. A `true` must be followed by
    /// [`succeeded`](Self::succeeded), [`failed`](Self::failed) or
    /// [`released`](Self::released).
    pub fn allows(&self, now: DateTime<Utc>) -> bool {
        let mut state = self.state.lock().unwrap();
        let Some(at) = state.opened_at else {
            return true;
        };
        if now - at < self.open_for {
            return false;
        }
        if state.probe_since.is_some_and(|p| now - p < self.open_for) {
            return false;
        }
        state.probe_since = Some(now);
        true
    }

    pub fn succeeded(&self) {
        *self.state.lock().unwrap() = State::default();
    }

    pub fn failed(&self, now: DateTime<Utc>) {
        let mut state = self.state.lock().unwrap();
        state.consecutive += 1;
        state.probe_since = None;
        if state.opened_at.is_some() || state.consecutive >= self.failures {
            state.opened_at = Some(now);
        }
    }

    /// The call let through ended with no verdict on the source (it is not
    /// configured): a half-open try may be made again.
    pub fn released(&self) {
        self.state.lock().unwrap().probe_since = None;
    }
}
