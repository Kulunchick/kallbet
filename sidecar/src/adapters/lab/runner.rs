//! The protocol's runner: reads every scored Match of the archive through the
//! replay ports at its cutoff, then evaluates each Candidate Signal Event by
//! Event, walk-forward, against the anchor.
use super::report::{CandidateReport, EventInfo, EventRow, LabReport, MarketSubset, Metrics};
use super::signals::roster::RosterHistory;
use super::signals::{self, CandidateSignal, MatchView, Observed};
use super::stats::{self, Verdict};
use super::{PROTOCOL_VERSION, Protocol};
use crate::adapters::backtest::{Feeds, ReplayArchive};
use crate::*;
use std::{collections::BTreeMap, sync::Arc};

pub struct Lab {
    pub archive: ReplayArchive,
    /// The replay ports. Tests replace one to inject a datum.
    pub feeds: Feeds,
    pub protocol: Protocol,
    /// The Candidate Signals evaluated, in report order (default: the registry).
    pub signals: Vec<Arc<dyn CandidateSignal>>,
}

/// One Match as one Candidate Signal saw it: the anchor's probability and the
/// variant's, for team A (`None`: not predicted).
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct LabPrediction {
    pub signal: String,
    pub match_id: i64,
    pub event_id: i64,
    pub baseline: Option<f64>,
    pub variant: Option<f64>,
}

pub struct LabOutcome {
    pub report: LabReport,
    pub predictions: Vec<LabPrediction>,
}

/// An Event of the walk-forward: when it began (its first Match's cutoff) and
/// when it ended (its last result), and its scored Matches.
struct Event {
    info: EventInfo,
    matches: Vec<Observed>,
}

impl Lab {
    pub fn new(archive: ReplayArchive) -> Result<Self> {
        Ok(Self {
            feeds: Feeds::replay(&archive)?,
            archive,
            protocol: Protocol::default(),
            signals: signals::registry(),
        })
    }

    async fn view(
        &self,
        game: &Match,
        cutoff: Instant,
        rosters: Option<&RosterHistory>,
    ) -> Result<MatchView> {
        let view = MatchView {
            signal: self.feeds.signal.signal(game, cutoff).await?,
            market: self.feeds.odds.quote(game, cutoff).await?,
            roster: rosters
                .map(|history| history.at(game, cutoff))
                .transpose()?
                .flatten(),
            game: game.clone(),
            cutoff,
        };
        // A port that answers with a datum at or after the cutoff ends the run.
        view.signal()?;
        view.market()?;
        Ok(view)
    }

    async fn events(&self) -> Result<Vec<Event>> {
        let rosters = self
            .signals
            .iter()
            .any(|signal| signal.uses_rosters())
            .then(|| RosterHistory::new(&self.archive))
            .transpose()?;
        let mut by_id: BTreeMap<i64, Event> = BTreeMap::new();
        for stage in &self.archive.stages {
            let event = by_id.entry(stage.event_id).or_insert_with(|| Event {
                info: EventInfo {
                    id: stage.event_id,
                    title: stage.event.clone(),
                    begins: stage.cutoff,
                    ends: stage.cutoff,
                    matches: 0,
                },
                matches: vec![],
            });
            event.info.begins = event.info.begins.min(stage.cutoff);
            for m in &stage.matches {
                event.info.ends = event.info.ends.max(m.finished_at);
                let view = self.view(&m.game, m.cutoff(), rosters.as_ref()).await?;
                event.matches.push(Observed {
                    view,
                    team_a_won: m.winner == m.game.team_a.id,
                });
            }
        }
        let mut events: Vec<Event> = by_id.into_values().collect();
        for e in &mut events {
            e.info.matches = e.matches.len();
        }
        events.sort_by_key(|e| (e.info.begins, e.info.id));
        Ok(events)
    }

    /// Runs the protocol. Any datum observed at or after its Match's cutoff,
    /// read from a port or returned by a Candidate Signal, aborts the run with
    /// `TemporalLeakError`.
    pub async fn run(&self) -> Result<LabOutcome> {
        let events = self.events().await?;
        let mut predictions = Vec::new();
        let mut candidates = Vec::new();
        for signal in &self.signals {
            let (report, rows) = self.evaluate(signal.as_ref(), &events)?;
            candidates.push(report);
            predictions.extend(rows);
        }
        Ok(LabOutcome {
            report: LabReport {
                protocol_version: PROTOCOL_VERSION.into(),
                seed: self.protocol.seed,
                resamples: self.protocol.resamples,
                level: self.protocol.level,
                min_matches: self.protocol.min_matches,
                archive_digest: self.archive.digest.clone(),
                archive_files: self.archive.files,
                baseline: "the anchor: HybridAnchored without LLM, 0.80 market and 0.20 Valve \
                           rating logit (`debate::base`)"
                    .into(),
                events: events.iter().map(|e| e.info.clone()).collect(),
                candidates,
            },
            predictions,
        })
    }

    /// The walk-forward boundary: the training Matches of `event` are those of
    /// the Events that ended before it began.
    fn training(events: &[Event], event: &Event) -> Vec<Observed> {
        events
            .iter()
            .filter(|e| e.info.ends < event.info.begins)
            .flat_map(|e| e.matches.iter().cloned())
            .collect()
    }

    fn evaluate(
        &self,
        signal: &dyn CandidateSignal,
        events: &[Event],
    ) -> Result<(CandidateReport, Vec<LabPrediction>)> {
        let missing = signal.missing_data(&self.archive);
        let mut rows = Vec::new();
        // (event, a won, baseline, variant, has market) of every paired Match.
        let mut paired: Vec<(i64, bool, f64, f64, bool)> = Vec::new();
        let mut per_event = Vec::new();
        for event in events {
            let training = Self::training(events, event);
            let fitted = match missing {
                None => Some(signal.fit(&training)?),
                Some(_) => None,
            };
            let (mut computed, mut both, mut diff) = (0, 0, 0.0);
            for m in &event.matches {
                let baseline = m.view.anchor()?;
                let variant = match &fitted {
                    Some(f) => match f.predict(&m.view)? {
                        Some(p) => Some(*p.at(m.view.cutoff)?),
                        None => None,
                    },
                    None => None,
                };
                if let Some(p) = variant
                    && !(0.0..=1.0).contains(&p)
                {
                    return Err(format!(
                        "Candidate Signal {} gave {p} for Match {}",
                        signal.name(),
                        m.view.game.id
                    )
                    .into());
                }
                computed += usize::from(variant.is_some());
                if let (Some(b), Some(v)) = (baseline, variant) {
                    both += 1;
                    diff += stats::log_loss(v, m.team_a_won) - stats::log_loss(b, m.team_a_won);
                    paired.push((event.info.id, m.team_a_won, b, v, m.view.market.is_some()));
                }
                rows.push(LabPrediction {
                    signal: signal.name().into(),
                    match_id: m.view.game.id,
                    event_id: event.info.id,
                    baseline,
                    variant,
                });
            }
            per_event.push(EventRow {
                event_id: event.info.id,
                title: event.info.title.clone(),
                training_matches: training.len(),
                computed,
                paired: both,
                log_loss_difference: (both > 0).then(|| diff / both as f64),
            });
        }
        let computed_on = rows.iter().filter(|r| r.variant.is_some()).count();
        let mut report = CandidateReport {
            name: signal.name().into(),
            description: signal.description().into(),
            verdict: Verdict::NotTestable,
            reason: String::new(),
            computed_on,
            paired: paired.len(),
            events: paired
                .iter()
                .map(|p| p.0)
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            baseline: None,
            variant: None,
            difference: None,
            interval: None,
            std_error: None,
            detectable: None,
            market: None,
            per_event,
        };
        if let Some(reason) = missing {
            report.reason = reason;
            return Ok((report, rows));
        }
        if computed_on == 0 {
            report.reason = format!("computed on no Match: it needs {}", signal.needs());
            return Ok((report, rows));
        }
        let metrics = |pick: fn(&(i64, bool, f64, f64, bool)) -> f64| {
            let n = paired.len() as f64;
            let mean = |f: &dyn Fn(f64, bool) -> f64| {
                paired.iter().map(|r| f(pick(r), r.1)).sum::<f64>() / n
            };
            Metrics {
                log_loss: mean(&stats::log_loss),
                brier: mean(&stats::brier),
                accuracy: mean(&|p, w| f64::from(u8::from(stats::correct(p, w)))),
            }
        };
        let diffs: Vec<(i64, f64)> = paired
            .iter()
            .map(|r| (r.0, stats::log_loss(r.3, r.1) - stats::log_loss(r.2, r.1)))
            .collect();
        let Some(boot) = stats::event_bootstrap(
            &diffs,
            self.protocol.resamples,
            self.protocol.level,
            self.protocol.seed,
        ) else {
            report.verdict = Verdict::NoEvidence;
            report.reason = "no Match was predicted by both the anchor and the variant".into();
            return Ok((report, rows));
        };
        let (b, v) = (metrics(|r| r.2), metrics(|r| r.3));
        report.difference = Some(Metrics {
            log_loss: v.log_loss - b.log_loss,
            brier: v.brier - b.brier,
            accuracy: v.accuracy - b.accuracy,
        });
        report.baseline = Some(b);
        report.variant = Some(v);
        report.interval = Some(boot.interval);
        report.std_error = Some(boot.std_error);
        report.detectable = Some(stats::detectable_effect(boot.std_error));
        let market: Vec<f64> = paired
            .iter()
            .zip(&diffs)
            .filter(|(r, _)| r.4)
            .map(|(_, d)| d.1)
            .collect();
        report.market = (!market.is_empty()).then(|| MarketSubset {
            matches: market.len(),
            log_loss_difference: market.iter().sum::<f64>() / market.len() as f64,
        });
        let (verdict, reason) = stats::verdict(
            report.paired,
            boot.interval,
            report.market.as_ref().map(|m| m.log_loss_difference),
            &self.protocol,
        );
        report.verdict = verdict;
        report.reason = reason;
        Ok((report, rows))
    }
}
