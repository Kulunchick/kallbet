//! Ticket #27 owns this adapter family and its registration.
use crate::LlmRuntime;
use std::sync::Arc;
pub fn build() -> Arc<dyn LlmRuntime> {
    Arc::new(super::baseline::Noop)
}
