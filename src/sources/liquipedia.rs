//! Liquipedia Data Source adapter: pure parser + MediaWiki API fetch layer.
//! Owner: #11.
//!
//! A fallback adapter: it implements
//! [`ExternalSource`](crate::sources::fallback::ExternalSource) (records with
//! Liquipedia's own ids; a Match template's `hltv=` field goes into
//! `ExternalMatch::canonical`), and the fallback chain maps them to canonical
//! ids. `runtime::match_source` already puts it in the chain; until #11 fills
//! it in, it answers `NotConfigured` and is skipped.

use async_trait::async_trait;

use crate::domain::DataSourceKind;
use crate::ports::SourceError;
use crate::sources::fallback::{EventHint, ExternalEvent, ExternalSnapshot, ExternalSource};

pub struct LiquipediaSource;

impl LiquipediaSource {
    pub fn new() -> LiquipediaSource {
        LiquipediaSource
    }
}

impl Default for LiquipediaSource {
    fn default() -> LiquipediaSource {
        LiquipediaSource::new()
    }
}

#[async_trait]
impl ExternalSource for LiquipediaSource {
    fn kind(&self) -> DataSourceKind {
        DataSourceKind::Liquipedia
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
