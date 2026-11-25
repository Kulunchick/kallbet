//! Ticket #29 owns this adapter family and its registration.
use crate::LlmRuntime;
use std::sync::Arc;
pub fn build(runtime: Arc<dyn LlmRuntime>) -> Arc<dyn LlmRuntime> {
    runtime
}
