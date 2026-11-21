//! PandaScore Data Source adapter: pure parser + REST fetch layer. Owner: #10.
//!
//! A fallback adapter: it implements
//! [`ExternalSource`](crate::sources::fallback::ExternalSource) (records with
//! PandaScore's own ids), and the fallback chain maps them to canonical ids.
//! `runtime::match_source` already puts it in the chain; until #10 fills it
//! in, it answers `NotConfigured` and is skipped.

use async_trait::async_trait;

use crate::domain::DataSourceKind;
use crate::ports::SourceError;
use crate::sources::fallback::{EventHint, ExternalEvent, ExternalSnapshot, ExternalSource};

pub struct PandaScoreSource {
    /// `PANDASCORE_TOKEN`; `None` means the adapter is skipped.
    #[allow(dead_code)] // read by #10's fetch layer
    token: Option<String>,
}

impl PandaScoreSource {
    pub fn new(token: Option<String>) -> PandaScoreSource {
        PandaScoreSource { token }
    }
}

#[async_trait]
impl ExternalSource for PandaScoreSource {
    fn kind(&self) -> DataSourceKind {
        DataSourceKind::PandaScore
    }

    async fn list_events(&self) -> Result<Vec<ExternalEvent>, SourceError> {
        Err(SourceError::NotConfigured)
    }

    async fn find_events(&self, _hint: &EventHint) -> Result<Vec<ExternalEvent>, SourceError> {
        Err(SourceError::NotConfigured)
    }

    async fn fetch_event(&self, _external_id: &str) -> Result<ExternalSnapshot, SourceError> {
        Err(SourceError::NotConfigured)
    }
}
