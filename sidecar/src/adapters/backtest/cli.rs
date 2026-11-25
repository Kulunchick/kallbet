//! `kallbet-ai backtest`: runs the two-track benchmark over a replay archive
//! and writes `report.md`, `report.json` and the audit file into `--out`.
//!
//! ```text
//! kallbet-ai backtest [--archive DIR] [--out DIR] [--llm FILE] [--answers FILE]
//!                     [--llm-stage ID]... [--blind-stage ID] [--blind-matches N]
//!                     [--blind-direct] [--simulations N]
//! ```
//!
//! `--llm FILE` is JSON: `{"llm": <the sidecar config's llm section>, "single":
//! [...], "advocate": [...], "devil": [...], "arbiter": [...]}` (provider names).
//! Without it no provider is called: the LLM profiles answer only from
//! `--answers` (recorded answers; the file is updated with every new answer).
use super::runner::{Backtest, LlmPlan, export_answers, import_answers};
use super::{ReplayArchive, archive};
use crate::adapters::llm::{LlmConfig, TokioRunner};
use crate::*;
use serde::Deserialize;
use std::{path::PathBuf, sync::Arc};
use tracing::info;

pub const USAGE: &str = "usage: kallbet-ai backtest [--archive DIR] [--out DIR] [--llm FILE] \
[--answers FILE] [--llm-stage ID]... [--blind-stage ID] [--blind-matches N] [--blind-direct] \
[--simulations N]";

/// PGL Major Singapore 2026 Stage 1 (PandaScore tournament): after every
/// model's training cutoff, so evaluated with real names.
pub const SINGAPORE_STAGE_1: i64 = 22378;
/// IEM Cologne Major 2026 Stage 1: before it, so evaluated blind.
pub const COLOGNE_STAGE_1: i64 = 20708;

#[derive(Debug, PartialEq)]
pub struct Options {
    pub archive: PathBuf,
    pub out: PathBuf,
    pub llm: Option<PathBuf>,
    pub answers: Option<PathBuf>,
    pub llm_stages: Vec<i64>,
    pub blind_stage: i64,
    pub blind_matches: usize,
    pub blind_direct: bool,
    pub simulations: usize,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            archive: "sidecar/tests/fixtures/backtest".into(),
            out: "target/backtest".into(),
            llm: None,
            answers: None,
            llm_stages: vec![],
            blind_stage: COLOGNE_STAGE_1,
            blind_matches: 6,
            blind_direct: false,
            simulations: crate::adapters::pickem::DEFAULT_SIMULATIONS,
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
            "--llm" => o.llm = Some(value()?.into()),
            "--answers" => o.answers = Some(value()?.into()),
            "--llm-stage" => o.llm_stages.push(value()?.parse()?),
            "--blind-stage" => o.blind_stage = value()?.parse()?,
            "--blind-matches" => o.blind_matches = value()?.parse()?,
            "--blind-direct" => o.blind_direct = true,
            "--simulations" => o.simulations = value()?.parse()?,
            other => return Err(format!("unknown backtest option {other:?}\n{USAGE}").into()),
        }
    }
    if o.llm_stages.is_empty() {
        o.llm_stages.push(SINGAPORE_STAGE_1);
    }
    Ok(o)
}

/// The `--llm` file.
#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct LlmFile {
    #[serde(default)]
    pub llm: LlmConfig,
    #[serde(default)]
    pub single: Vec<String>,
    #[serde(default)]
    pub advocate: Vec<String>,
    #[serde(default)]
    pub devil: Vec<String>,
    #[serde(default)]
    pub arbiter: Vec<String>,
}

/// The plan of the LLM track: every scored Match of the `--llm-stage` Stages
/// with real names, and the first `--blind-matches` of `--blind-stage` blind.
pub fn plan(archive: &ReplayArchive, o: &Options, file: &LlmFile) -> LlmPlan {
    let ids = |stage: i64| -> Vec<i64> {
        archive
            .stages
            .iter()
            .filter(|s| s.id == stage)
            .flat_map(|s| s.matches.iter().map(|m| m.game.id))
            .collect()
    };
    let direct: std::collections::BTreeSet<i64> =
        o.llm_stages.iter().flat_map(|s| ids(*s)).collect();
    let mut blind: std::collections::BTreeSet<i64> = ids(o.blind_stage)
        .into_iter()
        .take(o.blind_matches)
        .collect();
    let direct = if o.blind_direct {
        blind.extend(direct);
        Default::default()
    } else {
        direct
    };
    LlmPlan {
        direct,
        blind,
        single: file.single.clone(),
        advocate: file.advocate.clone(),
        devil: file.devil.clone(),
        arbiter: file.arbiter.clone(),
    }
}

/// Runs the command; returns the lines to print.
pub async fn run(args: &[String]) -> Result<String> {
    let o = parse(args)?;
    let archive = ReplayArchive::load(&o.archive)?;
    let file: LlmFile = match &o.llm {
        Some(path) => serde_json::from_slice(&archive::read_file(path)?)
            .map_err(|e| format!("{}: {e}", path.display()))?,
        None => LlmFile::default(),
    };
    let mut backtest = Backtest::new(archive, Arc::new(TokioRunner))?;
    backtest.plan = plan(&backtest.archive, &o, &file);
    backtest.llm = file.llm;
    backtest.simulations = o.simulations;

    std::fs::create_dir_all(&o.out)?;
    let audit_path = o.out.join("audit.sqlite");
    for suffix in ["", "-shm", "-wal"] {
        let p = PathBuf::from(format!("{}{suffix}", audit_path.display()));
        if p.exists() {
            std::fs::remove_file(p)?;
        }
    }
    let audit = Audit::open(&audit_path).await?;
    let imported = match &o.answers {
        Some(path) if path.exists() => {
            let turns: Vec<serde_json::Value> = serde_json::from_slice(&std::fs::read(path)?)?;
            import_answers(&audit, &turns).await?
        }
        _ => 0,
    };
    info!(
        stages = backtest.archive.stages.len(),
        files = backtest.archive.files,
        imported,
        llm_matches = backtest.plan.matches().len(),
        "backtest starting"
    );
    let outcome = backtest.run(&audit).await?;
    if let Some(path) = &o.answers {
        let turns = export_answers(&audit).await?;
        std::fs::write(path, serde_json::to_string_pretty(&turns)? + "\n")?;
    }
    let report = &outcome.report;
    std::fs::write(o.out.join("report.md"), report.markdown())?;
    std::fs::write(
        o.out.join("report.json"),
        serde_json::to_string_pretty(report)? + "\n",
    )?;
    Ok(format!(
        "backtest: {} predictions over {} Stages; report in {}",
        outcome.predictions.len(),
        backtest.archive.stages.len(),
        o.out.join("report.md").display()
    ))
}
