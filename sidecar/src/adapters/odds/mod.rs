//! Ticket #25 owns this adapter family and its registration.
use crate::OddsSource;
use std::sync::Arc;
pub fn build() -> Arc<dyn OddsSource> {
    Arc::new(super::baseline::Noop)
}
