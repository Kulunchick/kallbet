//! Ticket #37 (spec #33): the Holdout boundary. The Holdout is PGL Major
//! Singapore 2026 Stage 2, Stage 3 and Playoffs, and any Event completed after
//! spec #33 was published (2026-11-25 23:46:46 UTC) or not complete in the
//! archive (its last result may come after it). Singapore Stage 1 had been
//! looked at and belongs to the training side. No fitting and no selection
//! reads a Holdout Match: the runner keeps them apart and only the final step
//! (`selection::holdout`) sees them, once, after the selection is frozen.
use crate::adapters::backtest::ReplayStage;
use crate::*;

/// When spec #33 was published (GitHub issue #33, `createdAt`).
pub const SPEC_PUBLISHED: &str = "2026-11-25T23:46:46Z";
/// PandaScore tournament id of PGL Major Singapore 2026 Stage 1: training side.
pub const SINGAPORE_STAGE_1: i64 = 22378;

#[derive(Clone, Debug, PartialEq)]
pub struct Holdout {
    /// A Stage that begins at or after this, or whose Event has a result at or
    /// after it, is in the Holdout.
    pub from: Instant,
    /// Stages that stay on the training side whatever their dates.
    pub training_stages: Vec<i64>,
}
impl Default for Holdout {
    fn default() -> Self {
        Self {
            from: SPEC_PUBLISHED.parse().expect("a valid instant"),
            training_stages: vec![SINGAPORE_STAGE_1],
        }
    }
}
impl Holdout {
    /// Whether `stage` is in the Holdout; `event_last_result` is when the last
    /// result of its Event became known, `None` while the Event is not
    /// complete: its last result may yet come after the boundary, so it is in
    /// the Holdout.
    pub fn contains(&self, stage: &ReplayStage, event_last_result: Option<Instant>) -> bool {
        !self.training_stages.contains(&stage.id)
            && (stage.cutoff >= self.from || event_last_result.is_none_or(|t| t >= self.from))
    }
    pub fn describe(&self) -> String {
        let kept = if self.training_stages.is_empty() {
            String::new()
        } else {
            format!(
                " (except PandaScore tournament{} {}, kept on the training side)",
                if self.training_stages.len() > 1 {
                    "s"
                } else {
                    ""
                },
                self.training_stages
                    .iter()
                    .map(i64::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };
        format!(
            "every Stage that begins at or after {}, or whose Event has a result at or after \
             it or is not complete{kept}",
            self.from.format("%Y-%m-%d %H:%M:%S UTC")
        )
    }
}

/// A recorded Holdout evaluation (`lab_runs`): after it the Holdout may be
/// read again only to reproduce it, on the same archive with the same frozen
/// combination.
#[derive(Clone, Debug, PartialEq)]
pub struct Freeze {
    pub archive_sha256: String,
    /// The `lab_runs.holdout` record: `evaluated: <frozen combination>`.
    pub holdout: String,
    /// When it was evaluated (the run's start).
    pub evaluated_at: Instant,
}
