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
                "| Event | Fitted on | Computed on | Paired | Δ log-loss |\n|---|---|---|---|---|"
            );
            for e in &c.per_event {
                let _ = writeln!(
                    md,
                    "| {} | {} | {} | {} | {} |",
                    e.title,
                    e.training_matches,
                    e.computed,
                    e.paired,
                    num(e.log_loss_difference)
                );
            }
        }
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
}
