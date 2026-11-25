//! Ticket #31: the retrospective backtest. See docs/agents/codebase.md,
//! "Retrospective backtest (ticket #31)".
//!
//! The live pipeline (`Sidecar::tick` with its scheduler, policies, debate and
//! Pick'em step) runs over a replay of completed Stages: `ReplayPlatform` is
//! the League Platform port, `feeds` answer from saved real responses, the
//! clock is virtual and every LLM runtime is built in `Mode::Backtest` (no
//! tools, no web, whatever the config says). `BlindAnonymizer` masks what the
//! models see of Matches from before their training cutoff. `runner` drives
//! the steps and `report` scores the submissions.
pub mod anonymizer;
pub mod archive;
pub mod cli;
pub mod feeds;
pub mod platform;
pub mod report;
pub mod runner;
pub mod runs;

pub use anonymizer::{Blind, BlindAnonymizer};
pub use archive::{ReplayArchive, ReplayMatch, ReplayStage, StageKind};
pub use platform::{ReplayClock, ReplayPlatform, Submitted, SubmittedPickem};
pub use report::Report;
pub use runner::{Backtest, Feeds, LlmPlan, export_answers, import_answers};
pub use runs::{BacktestRun, record_run};

use crate::*;

/// A saved datum and the instant it was observed (published, revised).
#[derive(Clone, Debug)]
pub struct Dated<T> {
    pub observed_at: Instant,
    pub value: T,
}

/// A Match's evaluation cutoff in a retrospective run: the earlier of
/// Scheduled Start − 15 minutes and the actual start.
pub fn match_cutoff(scheduled: Instant, actual: Instant) -> Instant {
    hard_lock(scheduled).min(actual)
}
