use chrono::{DateTime, Utc};

/// The current time. Every "now" in KallBet comes from here (never
/// `Utc::now()` directly), so tests can move time.
pub trait Clock: Send + Sync {
    fn now(&self) -> DateTime<Utc>;
}

pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}
