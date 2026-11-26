//! The lab's report: the run's identity (archive digest, protocol version,
//! seed) and one row per Candidate Signal, every one of them, whatever its
//! result. Nothing in it depends on when it was produced, so the same archive
//! and seed give the same bytes.
use super::stats::{Interval, Verdict};
use crate::*;
use serde::Serialize;
use std::fmt::Write;

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Metrics {
    /// Mean negative log of the probability given to the winner (primary).
    pub log_loss: f64,
    pub brier: f64,
    pub accuracy: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct MarketSubset {
    /// Paired Matches with a market line.
    pub matches: usize,
    /// Mean log-loss difference (variant − anchor) on them.
    pub log_loss_difference: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct EventRow {
    pub event_id: i64,
    pub title: String,
    /// Matches the Candidate Signal was fitted on for this Event.
    pub training_matches: usize,
    pub computed: usize,
    pub paired: usize,
    pub log_loss_difference: Option<f64>,
    /// What the Candidate Signal fitted for this Event (`FittedSignal::parameters`).
    pub parameters: Option<serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CandidateReport {
    pub name: String,
    pub description: String,
    pub verdict: Verdict,
    /// Why this verdict, in words.
    pub reason: String,
    /// Matches the Candidate Signal gave a probability for.
    pub computed_on: usize,
    /// Of those, the Matches the anchor predicted too: the compared set.
    pub paired: usize,
    /// Events among the paired Matches (the bootstrap's units).
    pub events: usize,
    /// The anchor on the paired Matches.
    pub baseline: Option<Metrics>,
    pub variant: Option<Metrics>,
    /// variant − anchor: a negative log-loss or Brier difference is better, a
    /// positive accuracy difference is better.
    pub difference: Option<Metrics>,
    /// The interval of the log-loss difference, from the Event bootstrap.
    pub interval: Option<Interval>,
    pub std_error: Option<f64>,
    /// The smallest log-loss difference these Matches could have detected.
    pub detectable: Option<f64>,
    /// The paired Matches with a market line (`None`: there are none).
    pub market: Option<MarketSubset>,
    pub per_event: Vec<EventRow>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct EventInfo {
    /// PandaScore serie id.
    pub id: i64,
    pub title: String,
    /// The first cutoff of its Stages.
    pub begins: Instant,
    /// Its last result.
    pub ends: Instant,
    /// Scored Matches.
    pub matches: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct LabReport {
    pub protocol_version: String,
    pub seed: u64,
    pub resamples: usize,
    pub level: f64,
    pub min_matches: usize,
    pub archive_digest: String,
    pub archive_files: usize,
    pub baseline: String,
    pub events: Vec<EventInfo>,
    pub candidates: Vec<CandidateReport>,
    /// Forward selection of the `promote`d Candidate Signals (ticket #37).
    pub selection: SelectionReport,
    /// The frozen combination on the Holdout, the final step (ticket #37).
    pub holdout: HoldoutReport,
}

/// The combination's combiner as fitted for one Event.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CombinationEvent {
    pub event_id: i64,
    pub title: String,
    /// Training-side Matches the combiner could be fitted on for this Event.
    pub training_matches: usize,
    /// `Combiner::parameters` (`None`: nothing was promoted).
    pub parameters: Option<serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SelectionStep {
    pub entered: String,
    /// Walk-forward log-loss of the combination after this step.
    pub log_loss: f64,
    /// After - before (the anchor before the first step): what it added.
    pub added: f64,
    /// The interval of that difference, from the Event bootstrap.
    pub interval: Option<Interval>,
    pub tried: Vec<super::selection::Tried>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SelectionReport {
    /// The Candidate Signals marked `promote`, in registry order.
    pub considered: Vec<String>,
    /// Training-side Matches the anchor predicted: where log-losses are taken.
    pub matches: usize,
    pub anchor_log_loss: Option<f64>,
    pub final_log_loss: Option<f64>,
    pub steps: Vec<SelectionStep>,
    /// The frozen combination, in the order its members entered.
    pub frozen: Vec<String>,
    pub per_event: Vec<CombinationEvent>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum HoldoutStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "evaluated")]
    Evaluated,
}
impl HoldoutStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Evaluated => "evaluated",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct HoldoutReport {
    pub status: HoldoutStatus,
    /// The boundary, in words.
    pub rule: String,
    /// The archive's Holdout Stages ("Event: Stage (n scored)").
    pub stages: Vec<String>,
    /// The frozen combination evaluated (empty: the anchor itself).
    pub combination: Vec<String>,
    /// When and on what the Holdout was evaluated, or that it is pending.
    pub statement: String,
    pub matches: usize,
    pub events: usize,
    pub played_from: Option<Instant>,
    pub played_to: Option<Instant>,
    pub baseline: Option<Metrics>,
    pub variant: Option<Metrics>,
    pub difference: Option<Metrics>,
    pub interval: Option<Interval>,
    pub std_error: Option<f64>,
    pub market: Option<MarketSubset>,
    /// The written rule applied to the Holdout's numbers, for reading only.
    pub verdict: Option<Verdict>,
    pub reason: String,
    pub per_event: Vec<CombinationEvent>,
}

fn params(v: &Option<serde_json::Value>) -> String {
    v.as_ref().map_or("—".into(), |v| format!("`{v}`"))
}

fn interval(i: Option<Interval>) -> String {
    i.map_or("—".into(), |i| format!("[{:+.4}, {:+.4}]", i.low, i.high))
}

fn names(n: &[String], sep: &str) -> String {
    n.iter()
        .map(|n| format!("`{n}`"))
        .collect::<Vec<_>>()
        .join(sep)
}

fn num(v: Option<f64>) -> String {
    v.map_or("—".into(), |v| format!("{v:+.4}"))
}

impl LabReport {
    pub fn markdown(&self) -> String {
        let mut md = String::new();
        let level = (self.level * 100.0).round();
        let _ = writeln!(md, "# Signal Lab report\n");
        let _ = writeln!(
            md,
            "Archive SHA-256 `{}` ({} files), protocol `{}`, seed {}, {} Event resamples, \
             {level}% intervals, floor {} paired Matches.\n",
            self.archive_digest,
            self.archive_files,
            self.protocol_version,
            self.seed,
            self.resamples,
            self.min_matches
        );
        let _ = writeln!(
            md,
            "Baseline: {}. Each Candidate Signal is fitted, for each Event, only on the Matches \
             of the Events that ended before it began, and every datum it reads is observed \
             before the Match cutoff. Differences are variant − anchor on the Matches both \
             predicted (log-loss and Brier: below zero is better). The interval resamples whole \
             Events. Verdict: `promote` when the interval of the log-loss difference is below \
             zero and the Matches with a market line are not worse; `harmful` when it is above \
             zero; `no evidence` otherwise or under the floor; `not testable` when the data \
             could not be had. \"Detectable\" is the smallest log-loss difference these Matches \
             would show with 80% probability: an effect smaller than it can hide as \
             `no evidence`.\n",
            self.baseline
        );
        let _ = writeln!(
            md,
            "| Candidate Signal | Verdict | Computed on | Paired | Events | Δ log-loss | \
             {level}% interval | Detectable | Δ Brier | Δ accuracy | Market subset (n, Δ log-loss) |"
        );
        let _ = writeln!(md, "|---|---|---|---|---|---|---|---|---|---|---|");
        for c in &self.candidates {
            let d = c.difference;
            let _ = writeln!(
                md,
                "| `{}` | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
                c.name,
                c.verdict.as_str(),
                c.computed_on,
                c.paired,
                c.events,
                num(d.map(|d| d.log_loss)),
                c.interval
                    .map_or("—".into(), |i| format!("[{:+.4}, {:+.4}]", i.low, i.high)),
                c.detectable.map_or("—".into(), |v| format!("{v:.4}")),
                num(d.map(|d| d.brier)),
                num(d.map(|d| d.accuracy)),
                c.market.as_ref().map_or("—".into(), |m| format!(
                    "{}, {:+.4}",
                    m.matches, m.log_loss_difference
                )),
            );
        }
        for c in &self.candidates {
            let _ = writeln!(md, "\n## `{}`: {}\n", c.name, c.verdict.as_str());
            let _ = writeln!(md, "{}.\n\nVerdict: {}.\n", c.description, c.reason);
            if let (Some(b), Some(v)) = (c.baseline, c.variant) {
                let _ = writeln!(
                    md,
                    "| On the {} paired Matches | Log-loss | Brier | Accuracy |",
                    c.paired
                );
                let _ = writeln!(md, "|---|---|---|---|");
                for (name, m) in [("anchor", b), (c.name.as_str(), v)] {
                    let _ = writeln!(
                        md,
                        "| {name} | {:.4} | {:.4} | {:.1}% |",
                        m.log_loss,
                        m.brier,
                        m.accuracy * 100.0
                    );
                }
                let _ = writeln!(md);
            }
            let _ = writeln!(
                md,
                "| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |\n\
                 |---|---|---|---|---|---|"
            );
            for e in &c.per_event {
                let _ = writeln!(
                    md,
                    "| {} | {} | {} | {} | {} | {} |",
                    e.title,
                    e.training_matches,
                    e.computed,
                    e.paired,
                    num(e.log_loss_difference),
                    params(&e.parameters)
                );
            }
        }
        self.selection_markdown(&mut md);
        self.holdout_markdown(&mut md);
        let _ = writeln!(md, "\n## Events (walk-forward order)\n");
        let _ = writeln!(
            md,
            "| Event | Begins | Ends | Scored Matches |\n|---|---|---|---|"
        );
        for e in &self.events {
            let _ = writeln!(
                md,
                "| {} | {} | {} | {} |",
                e.title,
                e.begins.format("%Y-%m-%d %H:%M"),
                e.ends.format("%Y-%m-%d %H:%M"),
                e.matches
            );
        }
        md
    }

    fn selection_markdown(&self, md: &mut String) {
        let s = &self.selection;
        let level = (self.level * 100.0).round();
        let _ = writeln!(md, "\n## Forward selection\n");
        let _ = writeln!(
            md,
            "The Candidate Signals marked `promote` enter one combination, one at a time: at \
             each step the one that lowers the walk-forward log-loss most (on the {} \
             training-side Matches the anchor predicted) enters, and selection stops when none \
             lowers it. The combination is the combiner (a regularised logistic model without \
             intercept, penalty {} towards the anchor's weights) over the logits of the market \
             line and the rating plus each member's features, refitted for every Event on the \
             Events that ended before it. \"Added\" is the log-loss after the step − before it \
             (the anchor before the first).\n",
            s.matches,
            super::combiner::LAMBDA
        );
        if s.considered.is_empty() {
            let _ = writeln!(
                md,
                "No Candidate Signal is marked `promote`: nothing enters, and the frozen \
                 combination is the anchor."
            );
            return;
        }
        let _ = writeln!(
            md,
            "Considered: {}. Anchor log-loss {}.\n",
            names(&s.considered, ", "),
            s.anchor_log_loss.map_or("—".into(), |v| format!("{v:.4}"))
        );
        let _ = writeln!(
            md,
            "| Order | Entered | Log-loss after | Added | {level}% interval | Tried |\n\
             |---|---|---|---|---|---|"
        );
        for (i, step) in s.steps.iter().enumerate() {
            let tried: Vec<String> = step
                .tried
                .iter()
                .map(|t| format!("`{}` {:.4}", t.name, t.log_loss))
                .collect();
            let _ = writeln!(
                md,
                "| {} | `{}` | {:.4} | {:+.4} | {} | {} |",
                i + 1,
                step.entered,
                step.log_loss,
                step.added,
                interval(step.interval),
                tried.join(", ")
            );
        }
        if s.steps.is_empty() {
            let _ = writeln!(md, "| — | none lowered the anchor's log-loss | | | | |");
        }
        let frozen = if s.frozen.is_empty() {
            "the anchor".to_string()
        } else {
            names(&s.frozen, " + ")
        };
        let _ = writeln!(md, "\nFrozen combination: {frozen}.");
        if !s.frozen.is_empty() {
            let _ = writeln!(
                md,
                "\n| Event | Fitted on | Combiner weights |\n|---|---|---|"
            );
            for e in &s.per_event {
                let _ = writeln!(
                    md,
                    "| {} | {} | {} |",
                    e.title,
                    e.training_matches,
                    params(&e.parameters)
                );
            }
        }
    }

    fn holdout_markdown(&self, md: &mut String) {
        let h = &self.holdout;
        let level = (self.level * 100.0).round();
        let _ = writeln!(md, "\n## Holdout: {}\n", h.status.as_str());
        let _ = writeln!(
            md,
            "The Holdout is {}. No fitting and no selection step reads it; the frozen \
             combination is evaluated on it once, last.\n",
            h.rule
        );
        let _ = writeln!(md, "{}", h.statement);
        if !h.stages.is_empty() {
            let _ = writeln!(
                md,
                "\nHoldout Stages in this archive: {}.",
                h.stages.join("; ")
            );
        }
        if let (Some(b), Some(v), Some(d)) = (h.baseline, h.variant, h.difference) {
            let _ = writeln!(
                md,
                "\n| On the {} Holdout Matches | Log-loss | Brier | Accuracy |\n|---|---|---|---|",
                h.matches
            );
            for (name, m) in [("anchor", b), ("frozen combination", v)] {
                let _ = writeln!(
                    md,
                    "| {name} | {:.4} | {:.4} | {:.1}% |",
                    m.log_loss,
                    m.brier,
                    m.accuracy * 100.0
                );
            }
            let _ = writeln!(
                md,
                "\nΔ log-loss {} ({level}% interval {}), Δ Brier {}, Δ accuracy {}; market subset \
                 {}. By the written rule: {} ({}).",
                num(Some(d.log_loss)),
                interval(h.interval),
                num(Some(d.brier)),
                num(Some(d.accuracy)),
                h.market.as_ref().map_or("—".into(), |m| format!(
                    "{}, {:+.4}",
                    m.matches, m.log_loss_difference
                )),
                h.verdict.map_or("—", Verdict::as_str),
                h.reason
            );
        }
        if h.per_event.iter().any(|e| e.parameters.is_some()) {
            let _ = writeln!(
                md,
                "\n| Holdout Event | Fitted on | Combiner weights |\n|---|---|---|"
            );
            for e in &h.per_event {
                let _ = writeln!(
                    md,
                    "| {} | {} | {} |",
                    e.title,
                    e.training_matches,
                    params(&e.parameters)
                );
            }
        }
    }
}
