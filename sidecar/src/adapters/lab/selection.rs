//! Ticket #37 (spec #33): the selection procedure after every Candidate Signal
//! was evaluated alone. Forward selection combines the ones marked `promote`
//! (through the combiner, walk-forward, on the training side only), the
//! resulting combination is frozen, and the frozen combination is evaluated
//! once on the Holdout as the final step.
use super::combiner::{Combiner, Inputs, LAMBDA};
use super::holdout::Holdout;
use super::report::{CombinationEvent, HoldoutReport, HoldoutStatus, MarketSubset, Metrics};
use super::report::{SelectionReport, SelectionStep};
use super::runner::{Event, LabPrediction};
use super::signals::{CandidateSignal, FittedSignal, MatchView, Observed};
use super::{Protocol, stats};
use crate::*;
use serde::Serialize;
use std::{collections::BTreeMap, sync::Arc};

/// One step of forward selection: who entered, the combination's log-loss
/// after it, what it added (log-loss after − before: below zero is better)
/// and every Candidate Signal tried at that step.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Step {
    pub entered: String,
    pub log_loss: f64,
    pub added: f64,
    pub tried: Vec<Tried>,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Tried {
    pub name: String,
    pub log_loss: f64,
}

/// Greedy forward selection over `names` (in registry order) from a start
/// whose log-loss is `start`: at each step every name not yet chosen is tried
/// with the chosen ones, the lowest `score` enters (a tie: the earlier name),
/// and the procedure stops when no name lowers the log-loss. Deterministic.
pub fn forward(names: &[String], start: f64, mut score: impl FnMut(&[String]) -> f64) -> Vec<Step> {
    let mut chosen: Vec<String> = vec![];
    let mut current = start;
    let mut steps = vec![];
    loop {
        let mut best: Option<(&String, f64)> = None;
        let mut tried = vec![];
        for name in names.iter().filter(|n| !chosen.contains(n)) {
            let mut set = chosen.clone();
            set.push(name.clone());
            let ll = score(&set);
            tried.push(Tried {
                name: name.clone(),
                log_loss: ll,
            });
            if best.is_none_or(|(_, b)| ll < b) {
                best = Some((name, ll));
            }
        }
        match best {
            Some((name, ll)) if ll < current - 1e-12 => {
                steps.push(Step {
                    entered: name.clone(),
                    log_loss: ll,
                    added: ll - current,
                    tried,
                });
                chosen.push(name.clone());
                current = ll;
            }
            _ => return steps,
        }
    }
}

/// A Candidate Signal with its fit for every training-side Event.
pub(crate) struct Member<'a> {
    pub signal: Arc<dyn CandidateSignal>,
    pub fits: &'a [Box<dyn FittedSignal>],
}

/// The features of one Candidate Signal for every Match: `[event][match][feature]`.
type Features = Vec<Vec<Vec<Option<f64>>>>;

pub(crate) fn features_of(
    signal: &dyn CandidateSignal,
    fit: &dyn FittedSignal,
    view: &MatchView,
) -> Result<Vec<Option<f64>>> {
    let want = signal.feature_names().len();
    let got = fit.features(view)?;
    if got.len() != want {
        return Err(format!(
            "Candidate Signal {} gave {} features for Match {}, not {want}",
            signal.name(),
            got.len(),
            view.game.id
        )
        .into());
    }
    got.into_iter()
        .map(|f| {
            f.map(|f| f.at(view.cutoff).copied())
                .transpose()
                .map_err(Into::into)
        })
        .collect()
}

/// Each Match's features, from the fit of its own Event (out of sample: that
/// fit saw only the Events that ended before it).
fn cache(member: &Member, events: &[Event]) -> Result<Features> {
    events
        .iter()
        .zip(member.fits)
        .map(|(e, fit)| {
            e.matches
                .iter()
                .map(|m| features_of(member.signal.as_ref(), fit.as_ref(), &m.view))
                .collect()
        })
        .collect()
}

fn base(view: &MatchView) -> Result<Inputs> {
    Ok(Inputs::base(
        view.market()?,
        view.signal()?.map(|s| s.series),
    ))
}

fn extra_names(signals: &[&Arc<dyn CandidateSignal>]) -> Vec<String> {
    signals
        .iter()
        .flat_map(|s| {
            s.feature_names()
                .into_iter()
                .map(move |f| format!("{}.{f}", s.name()))
        })
        .collect()
}

/// The training-side Events that ended before `begins`: the walk-forward boundary.
fn before(events: &[Event], begins: Instant) -> impl Iterator<Item = usize> + '_ {
    (0..events.len()).filter(move |&j| events[j].ended_before(begins))
}

/// Training rows of the combiner for an Event that begins at `begins`.
fn rows(
    events: &[Event],
    bases: &[Vec<Inputs>],
    feats: &[&Features],
    begins: Instant,
) -> Vec<(Inputs, bool)> {
    before(events, begins)
        .flat_map(|j| {
            events[j].matches.iter().enumerate().map(move |(k, m)| {
                let mut inputs = bases[j][k].clone();
                inputs.extras = feats.iter().flat_map(|f| f[j][k].clone()).collect();
                (inputs, m.team_a_won)
            })
        })
        .collect()
}

/// The walk-forward predictions of the combination over the training side,
/// `[event][match]`, and its fitted combiner per Event.
fn walk_forward(
    events: &[Event],
    bases: &[Vec<Inputs>],
    feats: &[&Features],
    names: &[String],
) -> (Vec<Vec<Option<f64>>>, Vec<Combiner>) {
    let mut predictions = vec![];
    let mut fitted = vec![];
    for (i, e) in events.iter().enumerate() {
        let training = rows(events, bases, feats, e.info.begins);
        let combiner = Combiner::fit(names, &training, LAMBDA);
        predictions.push(
            (0..e.matches.len())
                .map(|k| {
                    let mut inputs = bases[i][k].clone();
                    inputs.extras = feats.iter().flat_map(|f| f[i][k].clone()).collect();
                    combiner.predict(&inputs)
                })
                .collect(),
        );
        fitted.push(combiner);
    }
    (predictions, fitted)
}

/// `(event, team A won, anchor, variant, has market)` of the Matches both predicted.
type Paired = Vec<(i64, bool, f64, f64, bool)>;

fn paired(events: &[Event], anchor: &[Vec<Option<f64>>], variant: &[Vec<Option<f64>>]) -> Paired {
    let mut out = vec![];
    for (i, e) in events.iter().enumerate() {
        for (k, m) in e.matches.iter().enumerate() {
            if let (Some(a), Some(v)) = (anchor[i][k], variant[i][k]) {
                out.push((e.info.id, m.team_a_won, a, v, m.view.market.is_some()));
            }
        }
    }
    out
}

fn mean_log_loss(p: &[(bool, f64)]) -> f64 {
    p.iter().map(|(w, p)| stats::log_loss(*p, *w)).sum::<f64>() / p.len().max(1) as f64
}

pub(crate) fn metrics(rows: &[(bool, f64)]) -> Metrics {
    let n = rows.len().max(1) as f64;
    Metrics {
        log_loss: mean_log_loss(rows),
        brier: rows.iter().map(|(w, p)| stats::brier(*p, *w)).sum::<f64>() / n,
        accuracy: rows
            .iter()
            .map(|(w, p)| f64::from(u8::from(stats::correct(*p, *w))))
            .sum::<f64>()
            / n,
    }
}

fn anchors(events: &[Event]) -> Result<Vec<Vec<Option<f64>>>> {
    events
        .iter()
        .map(|e| e.matches.iter().map(|m| m.view.anchor()).collect())
        .collect()
}

fn bases(events: &[Event]) -> Result<Vec<Vec<Inputs>>> {
    events
        .iter()
        .map(|e| e.matches.iter().map(|m| base(&m.view)).collect())
        .collect()
}

pub(crate) struct Selected {
    pub report: SelectionReport,
    /// The frozen combination's walk-forward predictions on the training side.
    pub predictions: Vec<LabPrediction>,
}

/// Forward selection of the `promote`d Candidate Signals on the training side.
pub(crate) fn select(
    events: &[Event],
    promoted: &[Member],
    protocol: &Protocol,
) -> Result<Selected> {
    let anchor = anchors(events)?;
    let bases = bases(events)?;
    let caches: Vec<Features> = promoted
        .iter()
        .map(|m| cache(m, events))
        .collect::<Result<_>>()?;
    let by_name: BTreeMap<String, usize> = promoted
        .iter()
        .enumerate()
        .map(|(i, m)| (m.signal.name().to_string(), i))
        .collect();
    let combination = |set: &[String]| {
        let idx: Vec<usize> = set.iter().map(|n| by_name[n]).collect();
        let feats: Vec<&Features> = idx.iter().map(|&i| &caches[i]).collect();
        let signals: Vec<&Arc<dyn CandidateSignal>> =
            idx.iter().map(|&i| &promoted[i].signal).collect();
        walk_forward(events, &bases, &feats, &extra_names(&signals))
    };
    let anchor_rows: Vec<(bool, f64)> = paired(events, &anchor, &anchor)
        .iter()
        .map(|r| (r.1, r.2))
        .collect();
    let start = mean_log_loss(&anchor_rows);
    let considered: Vec<String> = promoted
        .iter()
        .map(|m| m.signal.name().to_string())
        .collect();
    let steps = forward(&considered, start, |set| {
        let (p, _) = combination(set);
        let rows: Vec<(bool, f64)> = paired(events, &anchor, &p)
            .iter()
            .map(|r| (r.1, r.3))
            .collect();
        mean_log_loss(&rows)
    });

    // What each entrant added, with its interval: the combination after the
    // step against the one before it (the anchor before the first).
    let mut previous = anchor.clone();
    let mut reported = vec![];
    let mut chosen: Vec<String> = vec![];
    let mut fitted = vec![];
    for step in &steps {
        chosen.push(step.entered.clone());
        let (p, combiners) = combination(&chosen);
        let diffs: Vec<(i64, f64)> = paired(events, &previous, &p)
            .iter()
            .map(|r| (r.0, stats::log_loss(r.3, r.1) - stats::log_loss(r.2, r.1)))
            .collect();
        let boot =
            stats::event_bootstrap(&diffs, protocol.resamples, protocol.level, protocol.seed);
        reported.push(SelectionStep {
            entered: step.entered.clone(),
            log_loss: step.log_loss,
            added: step.added,
            interval: boot.map(|b| b.interval),
            tried: step.tried.clone(),
        });
        previous = p;
        fitted = combiners;
    }
    let mut predictions = vec![];
    for (i, e) in events.iter().enumerate() {
        for (k, m) in e.matches.iter().enumerate() {
            predictions.push(LabPrediction {
                signal: "combination".into(),
                match_id: m.view.game.id,
                event_id: e.info.id,
                baseline: anchor[i][k],
                variant: previous[i][k],
            });
        }
    }
    let per_event = events
        .iter()
        .enumerate()
        .map(|(i, e)| CombinationEvent {
            event_id: e.info.id,
            title: e.info.title.clone(),
            training_matches: before(events, e.info.begins)
                .map(|j| events[j].matches.len())
                .sum(),
            parameters: fitted.get(i).map(Combiner::parameters),
        })
        .collect();
    Ok(Selected {
        report: SelectionReport {
            considered,
            matches: anchor_rows.len(),
            anchor_log_loss: (!anchor_rows.is_empty()).then_some(start),
            final_log_loss: steps.last().map(|s| s.log_loss),
            steps: reported,
            frozen: chosen,
            per_event,
        },
        predictions,
    })
}

pub(crate) struct Evaluated {
    pub report: HoldoutReport,
    pub predictions: Vec<LabPrediction>,
}

/// The final step: the frozen combination on the Holdout, once. Its members
/// and its combiner are fitted on the training side only (for each Holdout
/// Event, the training-side Events that ended before it began); no Holdout
/// outcome is read before the scores are computed.
pub(crate) fn holdout(
    rule: &Holdout,
    events: &[Event],
    holdout: &[Event],
    stages: Vec<String>,
    frozen: &[Member],
    protocol: &Protocol,
) -> Result<Evaluated> {
    let names: Vec<String> = frozen.iter().map(|m| m.signal.name().to_string()).collect();
    let mut report = HoldoutReport {
        status: HoldoutStatus::Pending,
        rule: rule.describe(),
        stages,
        combination: names.clone(),
        statement: String::new(),
        matches: 0,
        events: 0,
        played_from: None,
        played_to: None,
        baseline: None,
        variant: None,
        difference: None,
        interval: None,
        std_error: None,
        market: None,
        verdict: None,
        reason: String::new(),
        per_event: vec![],
    };
    let scored: usize = holdout.iter().map(|e| e.matches.len()).sum();
    if scored == 0 {
        report.statement = "The Holdout is pending: no Holdout Match has been played in this \
                            archive, so the frozen combination was not evaluated."
            .into();
        return Ok(Evaluated {
            report,
            predictions: vec![],
        });
    }
    let bases_train = bases(events)?;
    let caches: Vec<Features> = frozen
        .iter()
        .map(|m| cache(m, events))
        .collect::<Result<_>>()?;
    let feats: Vec<&Features> = caches.iter().collect();
    let signals: Vec<&Arc<dyn CandidateSignal>> = frozen.iter().map(|m| &m.signal).collect();
    let extras = extra_names(&signals);
    let mut predictions = vec![];
    let mut pairs: Paired = vec![];
    for h in holdout {
        let training: Vec<Observed> = before(events, h.info.begins)
            .flat_map(|j| events[j].matches.iter().cloned())
            .collect();
        let fits: Vec<Box<dyn FittedSignal>> = frozen
            .iter()
            .map(|m| m.signal.fit(&training))
            .collect::<Result<_>>()?;
        let combiner = (!frozen.is_empty()).then(|| {
            Combiner::fit(
                &extras,
                &rows(events, &bases_train, &feats, h.info.begins),
                LAMBDA,
            )
        });
        for m in &h.matches {
            let anchor = m.view.anchor()?;
            let variant = match &combiner {
                // Nothing was promoted: the frozen combination is the anchor.
                None => anchor,
                Some(c) => {
                    let mut inputs = base(&m.view)?;
                    for (member, fit) in frozen.iter().zip(&fits) {
                        inputs.extras.extend(features_of(
                            member.signal.as_ref(),
                            fit.as_ref(),
                            &m.view,
                        )?);
                    }
                    c.predict(&inputs)
                }
            };
            if let (Some(a), Some(v)) = (anchor, variant) {
                pairs.push((h.info.id, m.team_a_won, a, v, m.view.market.is_some()));
            }
            predictions.push(LabPrediction {
                signal: "combination".into(),
                match_id: m.view.game.id,
                event_id: h.info.id,
                baseline: anchor,
                variant,
            });
        }
        report.per_event.push(CombinationEvent {
            event_id: h.info.id,
            title: h.info.title.clone(),
            training_matches: training.len(),
            parameters: combiner.as_ref().map(Combiner::parameters),
        });
    }
    report.status = HoldoutStatus::Evaluated;
    report.matches = pairs.len();
    report.events = holdout.iter().filter(|e| !e.matches.is_empty()).count();
    report.played_from = holdout.iter().map(|e| e.info.begins).min();
    report.played_to = holdout.iter().map(|e| e.info.ends).max();
    let combination = if names.is_empty() {
        "the anchor (no Candidate Signal was promoted)".to_string()
    } else {
        format!("`{}`", names.join(" + "))
    };
    report.statement = format!(
        "Evaluated once, as the final step of this run, after the selection was frozen: \
         {combination} on {} Holdout Matches played from {} to {}. The time of the run is in \
         the audit store's `lab_runs`.",
        pairs.len(),
        report
            .played_from
            .map_or("—".into(), |t| t.format("%Y-%m-%d %H:%M").to_string()),
        report
            .played_to
            .map_or("—".into(), |t| t.format("%Y-%m-%d %H:%M").to_string()),
    );
    if pairs.is_empty() {
        report.reason = "no Holdout Match was predicted by the anchor".into();
        return Ok(Evaluated {
            report,
            predictions,
        });
    }
    let b = metrics(&pairs.iter().map(|r| (r.1, r.2)).collect::<Vec<_>>());
    let v = metrics(&pairs.iter().map(|r| (r.1, r.3)).collect::<Vec<_>>());
    report.difference = Some(Metrics {
        log_loss: v.log_loss - b.log_loss,
        brier: v.brier - b.brier,
        accuracy: v.accuracy - b.accuracy,
    });
    report.baseline = Some(b);
    report.variant = Some(v);
    let diffs: Vec<(i64, f64)> = pairs
        .iter()
        .map(|r| (r.0, stats::log_loss(r.3, r.1) - stats::log_loss(r.2, r.1)))
        .collect();
    let market: Vec<f64> = pairs
        .iter()
        .zip(&diffs)
        .filter(|(r, _)| r.4)
        .map(|(_, d)| d.1)
        .collect();
    report.market = (!market.is_empty()).then(|| MarketSubset {
        matches: market.len(),
        log_loss_difference: market.iter().sum::<f64>() / market.len() as f64,
    });
    if let Some(boot) =
        stats::event_bootstrap(&diffs, protocol.resamples, protocol.level, protocol.seed)
    {
        report.interval = Some(boot.interval);
        report.std_error = Some(boot.std_error);
        let (verdict, reason) = stats::verdict(
            pairs.len(),
            boot.interval,
            report.market.as_ref().map(|m| m.log_loss_difference),
            protocol,
        );
        report.verdict = Some(verdict);
        report.reason = reason;
    }
    Ok(Evaluated {
        report,
        predictions,
    })
}

pub(crate) fn deployment_combiner(events: &[Event], members: &[Member]) -> Result<Combiner> {
    let cached = members
        .iter()
        .map(|m| cache(m, events))
        .collect::<Result<Vec<_>>>()?;
    let features = cached.iter().collect::<Vec<_>>();
    let signals = members.iter().map(|m| &m.signal).collect::<Vec<_>>();
    let inputs = rows(events, &bases(events)?, &features, Instant::MAX_UTC);
    Ok(Combiner::fit(&extra_names(&signals), &inputs, LAMBDA))
}
