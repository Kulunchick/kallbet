//! `kallbet-ai lab`: runs the Signal Lab over a replay archive and writes
//! `report.md` and `report.json` into `--out`, recording the run in
//! `--out/audit.sqlite` (appended to, never cleared). No network, no provider.
//!
//! ```text
//! kallbet-ai lab [--archive DIR] [--out DIR] [--seed N] [--resamples N] [--signal NAME]...
//!               [--catch-up-trials N] [--pickem-simulations N]
//!               [--llm FILE --budget NAME=RUNS...] [--answers FILE] [--training-cutoff RFC3339]
//! ```
//!
//! `--catch-up-trials` (default 2000) and `--pickem-simulations` (default
//! 5000) size the catch-up and Pick'em experiments' own simulations; the
//! protocol's verdicts do not depend on them.
//!
//! The LLM track (`llm.rs`, ticket #43) runs only with `--llm` (providers,
//! which need a `--budget` each) or `--answers` (recorded answers, no call).
use super::{Lab, record_lab_run, signals};
use crate::adapters::backtest::ReplayArchive;
use crate::adapters::baseline::SystemClock;
use crate::*;
use std::path::PathBuf;
use tracing::info;

pub const USAGE: &str = "usage: kallbet-ai lab [--archive DIR] [--out DIR] [--seed N] \
[--resamples N] [--signal NAME]... [--catch-up-trials N] [--pickem-simulations N] \
[--llm FILE --budget NAME=RUNS...] [--answers FILE] [--training-cutoff RFC3339]";

#[derive(Debug, PartialEq)]
pub struct Options {
    pub archive: PathBuf,
    pub out: PathBuf,
    pub seed: Option<u64>,
    pub resamples: Option<usize>,
    /// Candidate Signals to evaluate, by registry name (default: all).
    pub signals: Vec<String>,
    /// Paired trials of the catch-up experiment.
    pub catch_up_trials: usize,
    /// Swiss Monte-Carlo runs per Pick'em of the Pick'em comparison.
    pub pickem_simulations: usize,
    pub llm: super::llm::Options,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            archive: "sidecar/tests/fixtures/backtest".into(),
            out: "target/lab".into(),
            seed: None,
            resamples: None,
            signals: vec![],
            catch_up_trials: 2000,
            pickem_simulations: super::pickem::PickemOptions::default().simulations,
            llm: Default::default(),
        }
    }
}

pub fn parse(args: &[String]) -> Result<Options> {
    let mut o = Options::default();
    let mut archive_given = false;
    let mut it = args.iter();
    while let Some(arg) = it.next() {
        let mut value = || it.next().cloned().ok_or(format!("{arg} needs a value"));
        match arg.as_str() {
            "--archive" => {
                o.archive = value()?.into();
                archive_given = true;
            }
            "--out" => o.out = value()?.into(),
            "--seed" => o.seed = Some(value()?.parse()?),
            "--resamples" => {
                let n: usize = value()?.parse()?;
                if n == 0 {
                    return Err("--resamples needs at least one resample".into());
                }
                o.resamples = Some(n);
            }
            "--signal" => o.signals.push(value()?),
            "--catch-up-trials" => o.catch_up_trials = value()?.parse()?,
            "--pickem-simulations" => o.pickem_simulations = value()?.parse()?,
            flag if super::llm::FLAGS.contains(&flag) => o.llm.set(flag, value()?)?,
            other => return Err(format!("unknown lab option {other:?}\n{USAGE}").into()),
        }
    }
    if o.llm.llm.is_some() && !archive_given {
        return Err(format!(
            "--llm calls providers: name the archive they are spent on with --archive \
             (the default, {}, is the small test archive)",
            o.archive.display()
        )
        .into());
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
    // The first recorded Holdout evaluation, if any: only a reproduction of it
    // may read the Holdout again.
    lab.freeze = audit
        .lab_runs()
        .await?
        .into_iter()
        .find(|r| r.holdout.as_deref().is_some_and(|h| h.starts_with("evaluated")))
        .map(|r| super::Freeze {
            archive_sha256: r.archive_sha256,
            holdout: r.holdout.unwrap_or_default(),
            evaluated_at: r.started_at,
        });
    info!(
        stages = lab.archive.stages.len(),
        files = lab.archive.files,
        signals = lab.signals.len(),
        "lab starting"
    );
    // When the run happened is the one wall-clock reading of a lab run; it is
    // kept in the audit store, never in the report.
    let started_at = SystemClock.now();
    lab.evaluated_at = Some(lab.freeze.as_ref().map_or(started_at, |f| f.evaluated_at));
    let runner = std::sync::Arc::new(crate::adapters::llm::TokioRunner);
    let track = super::llm::prepare(&mut lab, &o.llm, runner, &o.out).await?;
    let outcome = lab.run().await?;
    let leagues = super::leagues::report(&lab, lab.protocol.seed, o.catch_up_trials).await?;
    let pickem_options = super::pickem::PickemOptions {
        simulations: o.pickem_simulations,
        ..Default::default()
    };
    let pickem = super::pickem::run(&lab, &pickem_options).await?;
    record_lab_run(&audit, started_at, SystemClock.now(), &outcome.report).await?;
    let report = &outcome.report;
    std::fs::write(o.out.join("catch-up.md"), leagues)?;
    super::llm::write(&mut lab, track.as_ref(), &o.llm, &o.out).await?;
    std::fs::write(
        o.out.join("report.md"),
        report.markdown()
            + "\n[Catch-up mode in simulated Leagues](catch-up.md)\n"
            + "
[LLM track: single agent, debate and seatings](llm.md)
"
            + "
[Pick'em variants in Pick'em Points](pickem.md)
",
    )?;
    std::fs::write(o.out.join("pickem.md"), pickem.report.markdown())?;
    std::fs::write(
        o.out.join("pickem.json"),
        serde_json::to_string_pretty(&pickem.report)? + "
",
    )?;
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
