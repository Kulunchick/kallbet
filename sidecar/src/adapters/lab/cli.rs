//! `kallbet-ai lab`: runs the Signal Lab over a replay archive and writes
//! `report.md` and `report.json` into `--out`, recording the run in
//! `--out/audit.sqlite` (appended to, never cleared). No network, no provider.
//!
//! ```text
//! kallbet-ai lab [--archive DIR] [--out DIR] [--seed N] [--resamples N] [--signal NAME]...
//! ```
use super::{Lab, record_lab_run, signals};
use crate::adapters::backtest::ReplayArchive;
use crate::adapters::baseline::SystemClock;
use crate::*;
use std::path::PathBuf;
use tracing::info;

pub const USAGE: &str = "usage: kallbet-ai lab [--archive DIR] [--out DIR] [--seed N] \
[--resamples N] [--signal NAME]...";

#[derive(Debug, PartialEq)]
pub struct Options {
    pub archive: PathBuf,
    pub out: PathBuf,
    pub seed: Option<u64>,
    pub resamples: Option<usize>,
    /// Candidate Signals to evaluate, by registry name (default: all).
    pub signals: Vec<String>,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            archive: "sidecar/tests/fixtures/backtest".into(),
            out: "target/lab".into(),
            seed: None,
            resamples: None,
            signals: vec![],
        }
    }
}

pub fn parse(args: &[String]) -> Result<Options> {
    let mut o = Options::default();
    let mut it = args.iter();
    while let Some(arg) = it.next() {
        let mut value = || it.next().cloned().ok_or(format!("{arg} needs a value"));
        match arg.as_str() {
            "--archive" => o.archive = value()?.into(),
            "--out" => o.out = value()?.into(),
            "--seed" => o.seed = Some(value()?.parse()?),
            "--resamples" => o.resamples = Some(value()?.parse()?),
            "--signal" => o.signals.push(value()?),
            other => return Err(format!("unknown lab option {other:?}\n{USAGE}").into()),
        }
    }
    Ok(o)
}

/// Runs the command; returns the line to print.
pub async fn run(args: &[String]) -> Result<String> {
    let o = parse(args)?;
    let mut chosen = Vec::new();
    for name in &o.signals {
        chosen.push(signals::by_name(name).ok_or_else(|| {
            let known: Vec<_> = signals::registry().iter().map(|s| s.name()).collect();
            format!(
                "unknown Candidate Signal {name:?}; registered: {}",
                known.join(", ")
            )
        })?);
    }
    let mut lab = Lab::new(ReplayArchive::load(&o.archive)?)?;
    if !chosen.is_empty() {
        lab.signals = chosen;
    }
    if let Some(seed) = o.seed {
        lab.protocol.seed = seed;
    }
    if let Some(n) = o.resamples {
        lab.protocol.resamples = n;
    }
    std::fs::create_dir_all(&o.out)?;
    let audit = Audit::open(&o.out.join("audit.sqlite")).await?;
    info!(
        stages = lab.archive.stages.len(),
        files = lab.archive.files,
        signals = lab.signals.len(),
        "lab starting"
    );
    // When the run happened is the one wall-clock reading of a lab run; it is
    // kept in the audit store, never in the report.
    let started_at = SystemClock.now();
    let outcome = lab.run().await?;
    record_lab_run(&audit, started_at, SystemClock.now(), &outcome.report).await?;
    let report = &outcome.report;
    std::fs::write(o.out.join("report.md"), report.markdown())?;
    std::fs::write(
        o.out.join("report.json"),
        serde_json::to_string_pretty(report)? + "\n",
    )?;
    Ok(format!(
        "lab: {} Candidate Signals over {} Events; report in {}",
        report.candidates.len(),
        report.events.len(),
        o.out.join("report.md").display()
    ))
}
