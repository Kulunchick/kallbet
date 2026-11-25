//! The `backtest_runs` audit table: one row per `kallbet-ai backtest` run.
use super::runner::Outcome;
use crate::*;
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// A recorded run, as the audit reader returns it.
#[derive(Clone, Debug, PartialEq)]
pub struct BacktestRun {
    pub started_at: Instant,
    pub finished_at: Instant,
    pub archive_sha256: String,
    pub archive_files: i64,
    /// `[[profile, description], ...]`
    pub profiles: Value,
    /// `{predictions: {profile: n}, llm_calls: {profile: n}, pickem_points,
    /// stages, llm_stopped}`
    pub headline: Value,
}

/// Records a finished run.
pub async fn record_run(
    audit: &Audit,
    started_at: Instant,
    finished_at: Instant,
    outcome: &Outcome,
) -> Result<()> {
    let report = &outcome.report;
    let mut predictions: BTreeMap<&str, usize> = BTreeMap::new();
    for p in &outcome.predictions {
        *predictions.entry(p.bot.as_str()).or_default() += 1;
    }
    let headline = json!({
        "predictions": predictions,
        "llm_calls": report.llm.calls,
        "pickem_points": report.pickem.iter().filter_map(|p| p.points).sum::<usize>(),
        "stages": report.stages.len(),
        "llm_stopped": report.llm.stopped,
    });
    sqlx::query(
        "INSERT INTO backtest_runs(started_at,finished_at,archive_sha256,archive_files,profiles,headline)
         VALUES(?,?,?,?,?,?)",
    )
    .bind(started_at)
    .bind(finished_at)
    .bind(&report.archive_digest)
    .bind(report.archive_files as i64)
    .bind(serde_json::to_string(&report.profiles)?)
    .bind(headline.to_string())
    .execute(&audit.db)
    .await?;
    Ok(())
}

impl Audit {
    /// Every recorded backtest run, oldest first.
    pub async fn backtest_runs(&self) -> Result<Vec<BacktestRun>> {
        type Row = (Instant, Instant, String, i64, String, String);
        let rows: Vec<Row> = sqlx::query_as(
            "SELECT started_at,finished_at,archive_sha256,archive_files,profiles,headline
             FROM backtest_runs ORDER BY id",
        )
        .fetch_all(&self.db)
        .await?;
        rows.into_iter()
            .map(|r| {
                Ok(BacktestRun {
                    started_at: r.0,
                    finished_at: r.1,
                    archive_sha256: r.2,
                    archive_files: r.3,
                    profiles: serde_json::from_str(&r.4)?,
                    headline: serde_json::from_str(&r.5)?,
                })
            })
            .collect()
    }
}
