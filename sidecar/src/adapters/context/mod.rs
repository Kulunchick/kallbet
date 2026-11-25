//! Ticket #26 owns this adapter family and its registration.
use crate::ContextEnricher;
use std::sync::Arc;
pub fn build() -> Arc<dyn ContextEnricher> {
    Arc::new(super::baseline::Noop)
}
