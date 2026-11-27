//! Ticket #34 (spec #33): the Signal Lab. See docs/agents/codebase.md,
//! "Signal Lab (ticket #34)".
//!
//! Every Candidate Signal is evaluated against the anchor (`debate::base`, the
//! 0.80 / 0.20 logit blend of market and Valve rating) under one fixed protocol,
//! over the backtest's replay archive, with no network and no provider:
//!
//! - **Point in time** (`MatchView`): a Match's inputs are read through the
//!   replay ports at its cutoff and every one is checked with `AsOf::at`; what a
//!   Candidate Signal returns is an `AsOf` checked again. A datum observed at or
//!   after the cutoff aborts the run with `TemporalLeakError`.
//! - **Walk-forward at Event granularity** (`runner`): to evaluate an Event, a
//!   Candidate Signal is fitted on the Matches of the Events that ended before it
//!   began, and only those; it never sees an outcome of its own Event.
//! - **Statistics** (`stats`): paired per-Match log-loss (primary), Brier score
//!   and accuracy; a bootstrap resampling whole Events with a recorded seed; the
//!   smallest detectable effect; the verdict rule.
//! - **Registry** (`signals`): a Candidate Signal is one module registered by
//!   name. Adding one touches neither the protocol nor the scheduler.
pub mod cli;
pub mod combiner;
pub mod holdout;
pub mod leagues;
pub mod llm;
pub mod pickem;
pub mod report;
pub mod runner;
pub mod runs;
pub mod selection;
pub mod signals;
pub mod stats;

pub use holdout::{Freeze, Holdout};
pub use report::{
    CandidateReport, DirectComparison, EventInfo, EventRow, HoldoutReport, HoldoutStatus,
    LabReport, MarketSubset, Metrics, SelectionReport, SubsetReport, TierView,
};
pub use runner::{Lab, LabOutcome, LabPrediction, TierFilter};
pub use runs::{LabRun, record_lab_run};
pub use signals::{CandidateSignal, FittedSignal, MatchView, Observed};

/// The protocol's version, named in every report and run record. Bump it when
/// the metric, the resampling, the verdict rule or the walk-forward boundary
/// changes; a new Candidate Signal does not change it.
pub const PROTOCOL_VERSION: &str = "signal-lab/3";

/// The fixed settings of the protocol (spec #33, "Protocol").
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct Protocol {
    /// Seed of the Event bootstrap.
    pub seed: u64,
    /// Bootstrap resamples.
    pub resamples: usize,
    /// Coverage of the interval of the log-loss difference.
    pub level: f64,
    /// Below this many paired Matches the verdict is `no evidence`.
    pub min_matches: usize,
}
pub const DEFAULT_SEED: u64 = 33;
impl Default for Protocol {
    fn default() -> Self {
        Self {
            seed: DEFAULT_SEED,
            resamples: 2000,
            level: 0.90,
            min_matches: 100,
        }
    }
}
