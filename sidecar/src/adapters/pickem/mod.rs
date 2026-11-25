//! Ticket #28 owns this adapter family and its registration.
use crate::PickemStep;
use std::sync::Arc;
pub fn build() -> Arc<dyn PickemStep> {
    Arc::new(super::baseline::Noop)
}
