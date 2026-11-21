//! Circuit breaker for one Data Source: open after `failures` consecutive
//! failures, half-open (one more try allowed) once `open_for` has passed
//! since it opened. A failed half-open try opens it again from that moment;
//! a success closes it. Time comes from the caller's `Clock`.

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
}

impl Breaker {
    pub fn new(failures: u32, open_for: Duration) -> Breaker {
        Breaker {
            failures: failures.max(1),
            open_for,
            state: Mutex::new(State::default()),
        }
    }

    /// Whether a call may go to the source at `now` (closed or half-open).
    pub fn allows(&self, now: DateTime<Utc>) -> bool {
        match self.state.lock().unwrap().opened_at {
            None => true,
            Some(at) => now - at >= self.open_for,
        }
    }

    pub fn succeeded(&self) {
        *self.state.lock().unwrap() = State::default();
    }

    pub fn failed(&self, now: DateTime<Utc>) {
        let mut state = self.state.lock().unwrap();
        state.consecutive += 1;
        if state.opened_at.is_some() || state.consecutive >= self.failures {
            state.opened_at = Some(now);
        }
    }
}
