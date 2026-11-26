//! The `lab_runs` audit table: one row per `kallbet-ai lab` run.
use super::report::LabReport;
use crate::*;
use serde_json::{Map, Value};

/// A recorded run, as the audit reader returns it.
#[derive(Clone, Debug, PartialEq)]
pub struct LabRun {
    pub started_at: Instant,
    pub finished_at: Instant,
    pub archive_sha256: String,
    pub archive_files: i64,
    pub protocol_version: String,
    pub seed: u64,
    /// `{candidate signal name: verdict}`
    pub verdicts: Value,
    /// `pending`, or `evaluated: <frozen combination>` (`None`: a run before #37).
    pub holdout: Option<String>,
}

/// What a run did with the Holdout, as recorded.
pub fn holdout_record(report: &LabReport) -> String {
    let h = &report.holdout;
    match h.status {
        super::HoldoutStatus::Pending => "pending".into(),
        super::HoldoutStatus::Evaluated if h.combination.is_empty() => "evaluated: anchor".into(),
        super::HoldoutStatus::Evaluated => format!("evaluated: {}", h.combination.join(" + ")),
    }
}

/// Records a finished run.
pub async fn record_lab_run(
    audit: &Audit,
    started_at: Instant,
    finished_at: Instant,
    report: &LabReport,
) -> Result<()> {
    let verdicts: Map<String, Value> = report
        .candidates
        .iter()
        .map(|c| (c.name.clone(), Value::from(c.verdict.as_str())))
        .collect();
    sqlx::query(
        "INSERT INTO lab_runs(started_at,finished_at,archive_sha256,archive_files,protocol_version,seed,verdicts,holdout)
         VALUES(?,?,?,?,?,?,?,?)",
    )
    .bind(started_at)
    .bind(finished_at)
    .bind(&report.archive_digest)
    .bind(report.archive_files as i64)
    .bind(&report.protocol_version)
    .bind(report.seed as i64)
    .bind(Value::Object(verdicts).to_string())
    .bind(holdout_record(report))
    .execute(&audit.db)
    .await?;
    Ok(())
}

impl Audit {
    /// Every recorded Signal Lab run, oldest first.
    pub async fn lab_runs(&self) -> Result<Vec<LabRun>> {
        type Row = (
            Instant,
            Instant,
            String,
            i64,
            String,
            i64,
            String,
            Option<String>,
        );
        let rows: Vec<Row> = sqlx::query_as(
            "SELECT started_at,finished_at,archive_sha256,archive_files,protocol_version,seed,verdicts,holdout
             FROM lab_runs ORDER BY id",
        )
        .fetch_all(&self.db)
        .await?;
        rows.into_iter()
            .map(|r| {
                Ok(LabRun {
                    started_at: r.0,
                    finished_at: r.1,
                    archive_sha256: r.2,
                    archive_files: r.3,
                    protocol_version: r.4,
                    seed: r.5 as u64,
                    verdicts: serde_json::from_str(&r.6)?,
                    holdout: r.7,
                })
            })
            .collect()
    }
}
