//! The heartbeat file. The loop writes it before and after every tick; the
//! `kallbet-ai health` subcommand (the container's HEALTHCHECK) reads it. The
//! sidecar serves no port, so a file on its data volume is the whole interface.
use crate::*;
use chrono::Duration;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Heartbeat {
    /// When the loop last began a tick.
    pub started_at: Instant,
    /// When the last tick ended, whatever its outcome. `None` before the first.
    pub finished_at: Option<Instant>,
    /// Ticks that failed in a row. 0 after any successful tick.
    pub consecutive_failures: u32,
    /// Predictions and Pick'ems the last finished tick submitted.
    pub submitted: usize,
}

#[derive(Clone, Debug)]
pub struct Health {
    pub path: PathBuf,
}
impl Health {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
    /// Beside the audit file: `/data/sidecar.db` -> `/data/sidecar.health`.
    pub fn beside(audit_file: &Path) -> Self {
        Self::new(audit_file.with_extension("health"))
    }
    pub fn write(&self, beat: &Heartbeat) -> Result<()> {
        // Through a temporary file, so that a reader never sees half a write.
        let partial = self.path.with_extension("health.partial");
        std::fs::write(&partial, serde_json::to_vec(beat)?)?;
        std::fs::rename(&partial, &self.path)?;
        Ok(())
    }
    pub fn read(&self) -> Result<Heartbeat> {
        let bytes = std::fs::read(&self.path)
            .map_err(|e| format!("no heartbeat at {}: {e}", self.path.display()))?;
        Ok(serde_json::from_slice(&bytes)?)
    }
}

pub struct Limits {
    /// A heartbeat older than this means the loop is stuck or dead.
    pub max_age: Duration,
    /// This many failed ticks in a row mean the sidecar is not doing its job.
    pub max_failures: u32,
}
impl Heartbeat {
    /// `Ok` with a one-line summary when healthy, `Err` with the reason when not.
    pub fn verdict(&self, now: Instant, limits: &Limits) -> Result<String, String> {
        let last = self.finished_at.unwrap_or(self.started_at).max(self.started_at);
        let age = now - last;
        if age > limits.max_age {
            return Err(format!(
                "last tick activity {} s ago (limit {} s)",
                age.num_seconds(),
                limits.max_age.num_seconds()
            ));
        }
        if self.consecutive_failures >= limits.max_failures {
            return Err(format!(
                "{} ticks failed in a row (limit {})",
                self.consecutive_failures, limits.max_failures
            ));
        }
        Ok(format!(
            "healthy: last activity {} s ago, {} failed ticks in a row",
            age.num_seconds(),
            self.consecutive_failures
        ))
    }
}
