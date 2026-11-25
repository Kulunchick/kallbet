use crate::*;
pub trait Clock: Send + Sync {
    fn now(&self) -> Instant;
}
