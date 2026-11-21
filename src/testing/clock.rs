use std::sync::Mutex;

use chrono::{DateTime, Duration, TimeZone, Utc};

use crate::ports::Clock;

/// A clock that stands still until a test moves it.
pub struct FakeClock {
    now: Mutex<DateTime<Utc>>,
}

impl FakeClock {
    /// Starts at 2026-01-10 12:00:00 UTC (a Saturday).
    pub fn new() -> FakeClock {
        FakeClock::at(Utc.with_ymd_and_hms(2026, 1, 10, 12, 0, 0).unwrap())
    }

    pub fn at(now: DateTime<Utc>) -> FakeClock {
        FakeClock {
            now: Mutex::new(now),
        }
    }

    pub fn advance(&self, by: Duration) {
        *self.now.lock().unwrap() += by;
    }

    pub fn set(&self, now: DateTime<Utc>) {
        *self.now.lock().unwrap() = now;
    }
}

impl Default for FakeClock {
    fn default() -> FakeClock {
        FakeClock::new()
    }
}

impl Clock for FakeClock {
    fn now(&self) -> DateTime<Utc> {
        *self.now.lock().unwrap()
    }
}
