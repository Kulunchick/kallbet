//! The protocol's runner: reads every scored Match of the archive through the
//! replay ports at its cutoff, then evaluates each Candidate Signal Event by
//! Event, walk-forward, against the anchor.
use super::holdout::Holdout;
use super::report::{
    CandidateReport, DirectComparison, EventInfo, EventRow, LabReport, MarketSubset, Metrics,
};
use super::selection::{self, Member};
use super::signals::{self, CandidateSignal, FittedSignal, MatchView, Observed};
use super::stats::{self, Verdict};
use super::{PROTOCOL_VERSION, Protocol};
use crate::adapters::backtest::{Feeds, ReplayArchive};
use crate::*;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

pub struct Lab {
    pub archive: ReplayArchive,
    /// The replay ports. Tests replace one to inject a datum.
    pub feeds: Feeds,
    pub protocol: Protocol,
    /// The Candidate Signals evaluated, in report order (default: the registry).
    pub signals: Vec<Arc<dyn CandidateSignal>>,
    /// Which Stages are the Holdout (default: spec #33's).
    pub holdout: Holdout,
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
    /// Every Candidate Signal alone, on the training side.
    pub predictions: Vec<LabPrediction>,
    /// The frozen combination, walk-forward on the training side (`signal`: "combination").
    pub combination: Vec<LabPrediction>,
    /// The frozen combination on the Holdout (empty while it is pending).
    pub holdout: Vec<LabPrediction>,
}

/// An Event of the walk-forward: when it began (its first Match's cutoff) and
/// when it ended (its last result), and its scored Matches.
pub(crate) struct Event {
    pub(crate) info: EventInfo,
    pub(crate) matches: Vec<Observed>,
}

/// The training side and, apart, the Holdout.
struct Split {
    training: Vec<Event>,
    holdout: Vec<Event>,
    /// The Holdout Stages of the archive, for the report.
    holdout_stages: Vec<String>,
}

impl Lab {
    pub fn new(archive: ReplayArchive) -> Result<Self> {
        Ok(Self {
            feeds: Feeds::replay(&archive)?,
            archive,
            protocol: Protocol::default(),
            signals: signals::registry(),
            holdout: Holdout::default(),
        })
    }

    pub(super) async fn view(&self, game: &Match, cutoff: Instant) -> Result<MatchView> {
        let view = MatchView {
            map_history: self.archive.map_history.before(cutoff),
            veto_history: self.archive.veto_history.before(cutoff),
            signal: self.feeds.signal.signal(game, cutoff).await?,
            market: self.feeds.odds.quote(game, cutoff).await?,
            history_before: self.feeds.history.matches_before(cutoff).await?,
            team_a_history: self
                .feeds
                .history
                .team_matches(&game.team_a.name, cutoff)
                .await?,
            team_b_history: self
                .feeds
                .history
                .team_matches(&game.team_b.name, cutoff)
                .await?,
            game: game.clone(),
            cutoff,
        };
        // A port that answers with a datum at or after the cutoff ends the run.
        view.signal()?;
        view.market()?;
        view.history_before()?;
        view.team_a_history()?;
        view.team_b_history()?;
        Ok(view)
    }

    /// The archive's Events, the Holdout's Stages kept apart from the training side.
    async fn events(&self) -> Result<Split> {
        let mut last_result: BTreeMap<i64, Instant> = BTreeMap::new();
        for stage in &self.archive.stages {
            for m in &stage.matches {
                let e = last_result.entry(stage.event_id).or_insert(m.finished_at);
                *e = (*e).max(m.finished_at);
            }
        }
        let holdout_ids: BTreeSet<_> = self
            .archive
            .stages
            .iter()
            .filter(|stage| {
                let last = last_result
                    .get(&stage.event_id)
                    .copied()
                    .unwrap_or(stage.cutoff);
                self.holdout.contains(stage, last)
            })
            .map(|stage| stage.id)
            .collect();
        let map_history = self.archive.map_history.without_stages(&holdout_ids);
        let veto_history = self.archive.veto_history.without_stages(&holdout_ids);
        let mut sides: [BTreeMap<i64, Event>; 2] = [BTreeMap::new(), BTreeMap::new()];
        let mut holdout_stages = vec![];
        for stage in &self.archive.stages {
            let in_holdout = holdout_ids.contains(&stage.id);
            if in_holdout {
                holdout_stages.push(format!(
                    "{}: {} ({} scored)",
                    stage.event,
                    stage.name,
                    stage.matches.len()
                ));
            }
            let by_id = &mut sides[usize::from(in_holdout)];
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
                let mut view = self.view(&m.game, m.cutoff()).await?;
                view.map_history = map_history.before(view.cutoff);
                view.veto_history = veto_history.before(view.cutoff);
                event.matches.push(Observed {
                    view,
                    team_a_won: m.winner == m.game.team_a.id,
                });
            }
        }
        let [training, holdout] = sides.map(|by_id| {
            let mut events: Vec<Event> = by_id.into_values().collect();
            for e in &mut events {
                e.info.matches = e.matches.len();
            }
            events.sort_by_key(|e| (e.info.begins, e.info.id));
            events
        });
        Ok(Split {
            training,
            holdout,
            holdout_stages,
        })
    }

    /// Runs the protocol. Any datum observed at or after its Match's cutoff,
    /// read from a port or returned by a Candidate Signal, aborts the run with
    /// `TemporalLeakError`.
    ///
    /// Order of work (spec #33): each Candidate Signal alone against the
    /// anchor, then forward selection of those marked `promote`, both on the
    /// training side only; then the frozen combination on the Holdout, once.
    pub async fn run(&self) -> Result<LabOutcome> {
        let Split {
            training: events,
            holdout,
            holdout_stages,
        } = self.events().await?;
        let mut predictions = Vec::new();
        let mut candidates = Vec::new();
        let mut fitted = Vec::new();
        for signal in &self.signals {
            let (report, rows, fits) = self.evaluate(signal.as_ref(), &events)?;
            if report.verdict == Verdict::Promote {
                fitted.push((signal.clone(), fits));
            }
            candidates.push(report);
            predictions.extend(rows);
        }
        let promoted: Vec<Member> = fitted
            .iter()
            .map(|(signal, fits)| Member {
                signal: signal.clone(),
                fits,
            })
            .collect();
        let selected = selection::select(&events, &promoted, &self.protocol)?;
        // The selection is frozen here; only now is the Holdout read.
        let frozen: Vec<Member> = selected
            .report
            .frozen
            .iter()
            .filter_map(|name| promoted.iter().find(|m| m.signal.name() == name))
            .map(|m| Member {
                signal: m.signal.clone(),
                fits: m.fits,
            })
            .collect();
        let evaluated = selection::holdout(
            &self.holdout,
            &events,
            &holdout,
            holdout_stages,
            &frozen,
            &self.protocol,
        )?;
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
                selection: selected.report,
                holdout: evaluated.report,
            },
            predictions,
            combination: selected.predictions,
            holdout: evaluated.predictions,
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
    ) -> Result<(
        CandidateReport,
        Vec<LabPrediction>,
        Vec<Box<dyn FittedSignal>>,
    )> {
        let missing = signal.missing_data(&self.archive);
        let mut rows = Vec::new();
        let mut fits = Vec::new();
        // (event, a won, baseline, variant, has market) of every paired Match.
        let mut paired: Vec<(i64, bool, f64, f64, bool)> = Vec::new();
        // (event, a won, valve series, unblended rating) for direct comparison with Valve.
        let mut vs_valve_paired: Vec<(i64, bool, f64, f64)> = Vec::new();
        let mut vs_valve_per_event = Vec::new();
        let mut per_event = Vec::new();
        for event in events {
            let training = Self::training(events, event);
            let fitted = match missing {
                None => Some(signal.fit(&training)?),
                Some(_) => None,
            };
            let parameters = fitted.as_ref().and_then(|f| f.parameters());
            let (mut computed, mut both, mut diff) = (0, 0, 0.0);
            let (mut v_computed, mut v_both, mut v_diff) = (0, 0, 0.0);
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
                if let Some(f) = &fitted
                    && let Some(rp) = f.predict_rating(&m.view)?
                {
                    let rp = *rp.at(m.view.cutoff)?;
                    if !(0.0..=1.0).contains(&rp) {
                        return Err(format!(
                            "Candidate Signal {} gave rating {rp} for Match {}",
                            signal.name(),
                            m.view.game.id
                        )
                        .into());
                    }
                    v_computed += 1;
                    if let Some(valve) = m.view.signal()? {
                        v_both += 1;
                        v_diff += stats::log_loss(rp, m.team_a_won)
                            - stats::log_loss(valve.series, m.team_a_won);
                        vs_valve_paired.push((event.info.id, m.team_a_won, valve.series, rp));
                    }
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
                parameters,
            });
            vs_valve_per_event.push(EventRow {
                event_id: event.info.id,
                title: event.info.title.clone(),
                training_matches: training.len(),
                computed: v_computed,
                paired: v_both,
                log_loss_difference: (v_both > 0).then(|| v_diff / v_both as f64),
                parameters: None,
            });
            fits.extend(fitted);
        }
        let vs_valve = if vs_valve_paired.is_empty() {
            None
        } else {
            let v_diffs: Vec<(i64, f64)> = vs_valve_paired
                .iter()
                .map(|r| (r.0, stats::log_loss(r.3, r.1) - stats::log_loss(r.2, r.1)))
                .collect();
            stats::event_bootstrap(
                &v_diffs,
                self.protocol.resamples,
                self.protocol.level,
                self.protocol.seed,
            )
            .map(|boot| {
                let v_metrics = |pick: fn(&(i64, bool, f64, f64)) -> f64| {
                    let n = vs_valve_paired.len() as f64;
                    let mean = |f: &dyn Fn(f64, bool) -> f64| {
                        vs_valve_paired.iter().map(|r| f(pick(r), r.1)).sum::<f64>() / n
                    };
                    Metrics {
                        log_loss: mean(&stats::log_loss),
                        brier: mean(&stats::brier),
                        accuracy: mean(&|p, w| f64::from(u8::from(stats::correct(p, w)))),
                    }
                };
                let (b, v) = (v_metrics(|r| r.2), v_metrics(|r| r.3));
                DirectComparison {
                    reference: "Valve Regional Standings (unblended)".into(),
                    paired: vs_valve_paired.len(),
                    events: vs_valve_paired
                        .iter()
                        .map(|r| r.0)
                        .collect::<std::collections::BTreeSet<_>>()
                        .len(),
                    difference: Metrics {
                        log_loss: v.log_loss - b.log_loss,
                        brier: v.brier - b.brier,
                        accuracy: v.accuracy - b.accuracy,
                    },
                    baseline: b,
                    variant: v,
                    interval: boot.interval,
                    std_error: boot.std_error,
                    detectable: stats::detectable_effect(boot.std_error),
                    per_event: vs_valve_per_event,
                }
            })
        };
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
            vs_valve,
            per_event,
        };
        if let Some(reason) = missing {
            report.reason = reason;
            return Ok((report, rows, fits));
        }
        if computed_on == 0 {
            report.reason = format!("computed on no Match: it needs {}", signal.needs());
            return Ok((report, rows, fits));
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
            return Ok((report, rows, fits));
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
        Ok((report, rows, fits))
    }
}
