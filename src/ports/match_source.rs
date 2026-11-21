use async_trait::async_trait;

use crate::domain::{Event, EventId, EventSnapshot};

/// Why a Data Source could not answer.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SourceError {
    /// The adapter lacks configuration (e.g. no PandaScore token). The
    /// fallback chain skips it; this is not a failure for the circuit breaker.
    #[error("data source is not configured")]
    NotConfigured,
    /// The Data Source answered and has no such Event.
    #[error("event not found")]
    NotFound,
    /// The Data Source could not be reached or refused us (network, HTTP
    /// status, Cloudflare challenge, rate limit, open circuit).
    #[error("data source unavailable: {0}")]
    Unavailable(String),
    /// The Data Source answered with something the parser does not
    /// understand. Never turn this into silently empty data.
    #[error("could not parse data source response: {0}")]
    Parse(String),
}

/// A provider of Event and Match data in the canonical model, keyed by
/// canonical ids. Implemented by each Data Source adapter, by the
/// `FallbackSource` that composes them, and by the scriptable test fake.
#[async_trait]
pub trait MatchSource: Send + Sync {
    /// Current and upcoming top Events.
    async fn list_events(&self) -> Result<Vec<Event>, SourceError>;

    /// One Event's whole structure: Stages, teams and Matches with their
    /// Scheduled Start, status, winner and format.
    async fn fetch_event(&self, id: EventId) -> Result<EventSnapshot, SourceError>;
}

/// The source used until a real adapter is wired in: always `NotConfigured`.
pub struct UnconfiguredSource;

#[async_trait]
impl MatchSource for UnconfiguredSource {
    async fn list_events(&self) -> Result<Vec<Event>, SourceError> {
        Err(SourceError::NotConfigured)
    }

    async fn fetch_event(&self, _id: EventId) -> Result<EventSnapshot, SourceError> {
        Err(SourceError::NotConfigured)
    }
}
