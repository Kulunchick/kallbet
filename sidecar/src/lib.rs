use chrono::{DateTime, Utc};

#[derive(Debug, thiserror::Error)]
#[error("observation {observed_at} is at or after cutoff {cutoff}")]
pub struct TemporalLeakError {
    pub observed_at: DateTime<Utc>,
    pub cutoff: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct AsOf<T> {
    value: T,
    observed_at: DateTime<Utc>,
}

impl<T> AsOf<T> {
    pub fn new(
        value: T,
        observed_at: DateTime<Utc>,
        cutoff: DateTime<Utc>,
    ) -> Result<Self, TemporalLeakError> {
        let observation = Self { value, observed_at };
        observation.at(cutoff)?;
        Ok(observation)
    }
    pub fn at(&self, cutoff: DateTime<Utc>) -> Result<&T, TemporalLeakError> {
        if self.observed_at >= cutoff {
            return Err(TemporalLeakError {
                observed_at: self.observed_at,
                cutoff,
            });
        }
        Ok(&self.value)
    }
    pub fn observed_at(&self) -> DateTime<Utc> {
        self.observed_at
    }
}

pub mod model;
pub use model::*;
pub mod config;
pub mod vrs;
pub use config::*;
pub mod ports;
pub use ports::*;
pub mod adapters;
pub mod audit;
pub use audit::*;
mod scheduler;
pub use scheduler::*;
mod registry;
