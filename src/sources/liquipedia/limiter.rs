//! Liquipedia's published API limits, as a pure schedule driven by the
//! caller's clock (so tests never sleep).
//!
//! Source: <https://liquipedia.net/api-terms-of-use> (read 2026-11-21):
//! - "Rate limit all HTTP requests to no more than 1 request per 2 seconds."
//! - "API action=parse requests should not exceed 1 request per 30 seconds as
//!   these are more resource intensive."
//!
//! We never call `action=parse`. `action=expandtemplates` (which runs the
//! wiki's Lua too) is held to the same 30 seconds, to stay on the safe side.

use chrono::{DateTime, Duration, Utc};

/// Between any two requests.
pub const MIN_GAP: Duration = Duration::seconds(2);
/// Between two requests of the resource-intensive kind.
pub const HEAVY_GAP: Duration = Duration::seconds(30);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    /// `action=query`.
    Normal,
    /// `action=expandtemplates` (and `action=parse`).
    Heavy,
}

#[derive(Debug, Default)]
pub struct RateLimiter {
    next_any: Option<DateTime<Utc>>,
    next_heavy: Option<DateTime<Utc>>,
}

impl RateLimiter {
    pub fn new() -> RateLimiter {
        RateLimiter::default()
    }

    /// Books the next slot for a request of `class` and returns how long the
    /// caller must wait before sending it. `None`, with nothing booked, when
    /// that wait would exceed `max_wait` (the caller reports the source
    /// unavailable instead of holding a poll for minutes).
    pub fn reserve(
        &mut self,
        now: DateTime<Utc>,
        class: Class,
        max_wait: Duration,
    ) -> Option<Duration> {
        let mut at = now;
        if let Some(t) = self.next_any {
            at = at.max(t);
        }
        if class == Class::Heavy
            && let Some(t) = self.next_heavy
        {
            at = at.max(t);
        }
        if at - now > max_wait {
            return None;
        }
        self.next_any = Some(at + MIN_GAP);
        if class == Class::Heavy {
            self.next_heavy = Some(at + HEAVY_GAP);
        }
        Some(at - now)
    }

    /// Nothing is sent before `now + for_`: the wiki told us to back off
    /// (HTTP 429) or refused us.
    pub fn back_off(&mut self, now: DateTime<Utc>, for_: Duration) {
        let until = now + for_;
        self.next_any = Some(self.next_any.map_or(until, |t| t.max(until)));
        self.next_heavy = Some(self.next_heavy.map_or(until, |t| t.max(until)));
    }
}
