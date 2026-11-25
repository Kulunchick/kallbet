//! Liquipedia API rate limiter driven by the injected Clock and Pause port (so tests never sleep).
//!
//! Follows <https://liquipedia.net/api-terms-of-use>:
//! - At most 1 HTTP request per 2 seconds (`MIN_GAP`).
//! - Never calls `action=parse`.
//! - Backs off on HTTP 429 (`Retry-After`, default 2 min) and HTTP 403 (10 min).

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};

pub const MIN_GAP: Duration = Duration::seconds(2);
pub const MAX_WAIT: Duration = Duration::seconds(60);

/// Waits for a duration; abstracted behind a trait so tests advance FakeClock without sleeping.
#[async_trait]
pub trait Pause: Send + Sync {
    async fn pause(&self, duration: Duration);
}

pub struct TokioPause;

#[async_trait]
impl Pause for TokioPause {
    async fn pause(&self, duration: Duration) {
        if let Ok(d) = duration.to_std() {
            tokio::time::sleep(d).await;
        }
    }
}

#[derive(Debug, Default)]
pub struct RateLimiter {
    next_slot: Option<DateTime<Utc>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Reserves the next request slot and returns the required wait duration.
    /// Returns `None` without booking if waiting would exceed `max_wait`.
    pub fn reserve(&mut self, now: DateTime<Utc>, max_wait: Duration) -> Option<Duration> {
        let at = self.next_slot.map_or(now, |next| now.max(next));
        let wait = at - now;
        if wait > max_wait {
            return None;
        }
        self.next_slot = Some(at + MIN_GAP);
        Some(wait)
    }

    /// Prevents any request before `now + duration` (e.g. after HTTP 429 or 403).
    pub fn back_off(&mut self, now: DateTime<Utc>, duration: Duration) {
        let until = now + duration;
        self.next_slot = Some(self.next_slot.map_or(until, |t| t.max(until)));
    }
}
