//! Candidate Signal `stacker` (ticket #58, from #56's criterion C2; added after
//! spec #33's candidate list was fixed): a combiner fitted walk-forward to pick
//! winners, not only to fit probabilities.
//!
//! Inputs, each of one Match and read before its cutoff: the market line, the
//! Valve rating calibrated as `rating_calibration` does (a logit scale per
//! series format, fitted on the training Matches), `own_rating`'s Glicko-2
//! series probability, `per_map_rating`'s per-map Elo series probability and
//! `rank_gap`'s two log rank gaps. They enter the lab's combiner: the market
//! and the calibrated rating as its base inputs (one weight vector per
//! availability of the two, regularised towards the anchor's 0.80 / 0.20), the
//! other four as extras on a logit scale; a missing extra reads as zero, no
//! shift. A Match with neither a market line nor a rating gets no prediction.
//!
//! Objective: the weights minimise the penalised log-loss; the penalty `λ`,
//! which sets how far the fit may move from the anchor, is chosen from `GRID`
//! by *accuracy*: refitted on the earlier training Events, each `λ` picks the
//! winner (probability above one half) of every Match of the latest third of
//! the training Events, and the most accurate `λ` wins (ties: lower log-loss
//! there, then the larger `λ`). The weights are then refitted with that `λ` on
//! every training Match. Everything is fitted on the Matches the protocol hands
//! `fit` (Events that ended before the evaluated one began), nothing else.
use super::{CandidateSignal, FittedSignal, MatchView, Observed};
use crate::adapters::lab::combiner::{Combiner, Inputs, LAMBDA, logit, sigmoid};
use crate::adapters::lab::{
    signals::{calibration, own_rating, per_map_rating, rank_gap},
    stats,
};
use crate::team_names::team_key;
use crate::*;
use serde_json::{Value, json};
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex};

pub const NAME: &str = "stacker";
/// The extras of the combiner, in order.
pub const EXTRAS: [&str; 4] = [
    "own_rating",
    "per_map_rating",
    "vrs_rank_gap",
    "hltv_rank_gap",
];
/// The extras without `per_map_rating` (ablation `stacker_no_map_elo`, #59).
pub(super) const EXTRAS_NO_MAP: [&str; 3] = ["own_rating", "vrs_rank_gap", "hltv_rank_gap"];
/// The penalties the accuracy rule chooses from.
pub const GRID: [f64; 6] = [1.0, 3.0, 10.0, 30.0, 100.0, 1000.0];
/// Fewest validation Matches the rule chooses on; below it, `LAMBDA`.
pub const MIN_VALIDATION: usize = 30;

/// What one Match's view gives, before any fit (`None`: not available).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Raw {
    /// Team A's market series probability.
    pub market: Option<f64>,
    /// Valve's map probability (uncalibrated).
    pub valve_map: Option<f64>,
    /// `own_rating`'s Glicko-2 map logit difference (scale 1).
    pub own_logit: Option<f64>,
    /// `per_map_rating`'s series probability.
    pub per_map: Option<f64>,
    pub vrs_gap: Option<f64>,
    pub hltv_gap: Option<f64>,
    /// The latest datum read.
    pub observed: Option<Instant>,
}

/// A training Match for the fit.
#[derive(Clone, Debug)]
pub struct Row {
    pub raw: Raw,
    pub format: Format,
    pub event: i64,
    pub cutoff: Instant,
    pub won: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChosenBy {
    /// The most accurate `λ` on the latest training Events.
    ValidationAccuracy,
    /// Too few training Events or validation Matches: `LAMBDA`.
    Default,
}

/// One `λ` of the grid on the validation Matches.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
pub struct Validation {
    pub lambda: f64,
    pub matches: usize,
    pub accuracy: f64,
    pub log_loss: f64,
}

/// A fitted stacker.
#[derive(Clone, Debug)]
pub struct Stacked {
    pub lambda: f64,
    pub chosen_by: ChosenBy,
    pub validation_events: usize,
    pub validation: Vec<Validation>,
    pub fit: Fit,
    pub training_matches: usize,
    pub training_events: usize,
    /// The latest cutoff among the training Matches.
    pub fitted_through: Option<Instant>,
}

/// The weights at one `λ`: calibration scales by format and the combiner.
#[derive(Clone, Debug)]
pub struct Fit {
    pub scales: [Option<f64>; 3],
    pub combiner: Combiner,
    include_map_elo: bool,
}

fn slot(format: Format) -> usize {
    match format {
        Format::Bo1 => 0,
        Format::Bo3 => 1,
        Format::Bo5 => 2,
    }
}

/// The calibration scales of `rating_calibration`, fitted on `rows`.
fn scales(rows: &[&Row]) -> [Option<f64>; 3] {
    let data: Vec<(f64, Format, bool)> = rows
        .iter()
        .filter_map(|r| r.raw.valve_map.map(|m| (logit(m), r.format, r.won)))
        .collect();
    let pooled = (data.len() >= calibration::MIN_FIT)
        .then(|| calibration::fit_scale(&data))
        .flatten();
    let mut out = [pooled; 3];
    for format in [Format::Bo1, Format::Bo3, Format::Bo5] {
        let rows: Vec<_> = data
            .iter()
            .filter(|r| slot(r.1) == slot(format))
            .copied()
            .collect();
        if rows.len() >= calibration::MIN_FIT {
            out[slot(format)] = calibration::fit_scale(&rows);
        }
    }
    out
}

impl Fit {
    fn new(rows: &[&Row], lambda: f64, include_map_elo: bool) -> Self {
        let scales = scales(rows);
        let extras: &[&str] = if include_map_elo {
            &EXTRAS
        } else {
            &EXTRAS_NO_MAP
        };
        let names: Vec<String> = extras.iter().map(|s| (*s).to_owned()).collect();
        let data: Vec<(Inputs, bool)> = rows
            .iter()
            .map(|r| (inputs(&r.raw, r.format, &scales, include_map_elo), r.won))
            .collect();
        Self {
            scales,
            combiner: Combiner::fit(&names, &data, lambda),
            include_map_elo,
        }
    }
    fn predict(&self, raw: &Raw, format: Format) -> Option<f64> {
        self.combiner
            .predict(&inputs(raw, format, &self.scales, self.include_map_elo))
    }
}

/// The combiner's inputs: market and calibrated Valve series as the base (a
/// format without a fitted scale keeps Valve's own), the extras on a logit scale.
fn inputs(raw: &Raw, format: Format, scales: &[Option<f64>; 3], include_map_elo: bool) -> Inputs {
    let rating = raw.valve_map.map(|m| {
        let map = sigmoid(scales[slot(format)].unwrap_or(1.0) * logit(m));
        vrs::series_probability(map, format)
    });
    let own = raw
        .own_logit
        .map(|x| logit(vrs::series_probability(sigmoid(x), format)));
    let mut extras = Vec::with_capacity(if include_map_elo { 4 } else { 3 });
    extras.push(own);
    if include_map_elo {
        extras.push(raw.per_map.map(logit));
    }
    extras.push(raw.vrs_gap);
    extras.push(raw.hltv_gap);
    Inputs {
        market: raw.market,
        rating,
        extras,
    }
}

/// Mean accuracy (a probability of one half counts half) and log-loss of `fit` on `rows`.
fn score(fit: &Fit, rows: &[&Row]) -> (usize, f64, f64) {
    let (mut n, mut hits, mut loss) = (0usize, 0.0, 0.0);
    for r in rows {
        let Some(p) = fit.predict(&r.raw, r.format) else {
            continue;
        };
        n += 1;
        hits += if p == 0.5 {
            0.5
        } else {
            f64::from(u8::from((p > 0.5) == r.won))
        };
        loss += stats::log_loss(p, r.won);
    }
    let d = n.max(1) as f64;
    (n, hits / d, loss / d)
}

impl Stacked {
    /// Fits on `rows`: `λ` by accuracy on the latest third of their Events
    /// (refitted on the Events that ended before those began), then the
    /// weights with it on every row.
    pub fn fit(rows: &[Row]) -> Self {
        Self::fit_with(rows, true)
    }

    pub(super) fn fit_with(rows: &[Row], include_map_elo: bool) -> Self {
        // Each Event's first and last cutoff, ordered by its last.
        let mut spans: BTreeMap<i64, (Instant, Instant)> = BTreeMap::new();
        for r in rows {
            let s = spans.entry(r.event).or_insert((r.cutoff, r.cutoff));
            s.0 = s.0.min(r.cutoff);
            s.1 = s.1.max(r.cutoff);
        }
        let mut events: Vec<(i64, Instant, Instant)> =
            spans.iter().map(|(e, (b, l))| (*e, *b, *l)).collect();
        events.sort_by_key(|e| (e.2, e.0));
        let held = events.len() / 3;
        let mut validation = vec![];
        let mut chosen = (LAMBDA, ChosenBy::Default);
        if held > 0 {
            let held_ids: Vec<i64> = events[events.len() - held..].iter().map(|e| e.0).collect();
            let begins = events[events.len() - held..]
                .iter()
                .map(|e| e.1)
                .min()
                .unwrap();
            let earlier: Vec<i64> = events
                .iter()
                .filter(|e| !held_ids.contains(&e.0) && e.2 < begins)
                .map(|e| e.0)
                .collect();
            let inner: Vec<&Row> = rows.iter().filter(|r| earlier.contains(&r.event)).collect();
            let held_rows: Vec<&Row> = rows
                .iter()
                .filter(|r| held_ids.contains(&r.event))
                .collect();
            if !inner.is_empty() {
                for lambda in GRID {
                    let (matches, accuracy, log_loss) =
                        score(&Fit::new(&inner, lambda, include_map_elo), &held_rows);
                    validation.push(Validation {
                        lambda,
                        matches,
                        accuracy,
                        log_loss,
                    });
                }
                if validation
                    .first()
                    .is_some_and(|v| v.matches >= MIN_VALIDATION)
                {
                    let best = validation
                        .iter()
                        .copied()
                        .reduce(|a, b| {
                            let better = b.accuracy > a.accuracy + 1e-12
                                || ((b.accuracy - a.accuracy).abs() <= 1e-12
                                    && b.log_loss <= a.log_loss + 1e-12);
                            if better { b } else { a }
                        })
                        .unwrap();
                    chosen = (best.lambda, ChosenBy::ValidationAccuracy);
                }
            }
        }
        let all: Vec<&Row> = rows.iter().collect();
        Self {
            lambda: chosen.0,
            chosen_by: chosen.1,
            validation_events: if chosen.1 == ChosenBy::Default {
                0
            } else {
                held
            },
            validation,
            fit: Fit::new(&all, chosen.0, include_map_elo),
            training_matches: rows.len(),
            training_events: spans.len(),
            fitted_through: rows.iter().map(|r| r.cutoff).max(),
        }
    }

    pub fn predict(&self, raw: &Raw, format: Format) -> Option<f64> {
        self.fit.predict(raw, format)
    }

    /// What was fitted, for the report's row of the evaluated Event.
    pub fn parameters(&self) -> Value {
        let round = |x: f64| (x * 1e6).round() / 1e6;
        let scales = |s: Option<f64>| s.map(round);
        json!({
            "objective": "penalised log-loss; λ chosen by accuracy on the latest training Events",
            "lambda": self.lambda,
            "chosen_by": self.chosen_by,
            "validation_events": self.validation_events,
            "validation": self.validation.iter().map(|v| json!({
                "lambda": v.lambda,
                "matches": v.matches,
                "accuracy": round(v.accuracy),
                "log_loss": round(v.log_loss),
            })).collect::<Vec<_>>(),
            "calibration_scales": {
                "bo1": scales(self.fit.scales[0]),
                "bo3": scales(self.fit.scales[1]),
                "bo5": scales(self.fit.scales[2]),
            },
            "weights": self.fit.combiner.parameters(),
            "training_matches": self.training_matches,
            "training_events": self.training_events,
            "fitted_through": self.fitted_through,
        })
    }
}

/// The view-only inputs of a Match, computed once per view: they read no fit.
/// Keyed by the Match, its cutoff and how much history the view holds.
type Key = (i64, Instant, usize, usize);
#[derive(Default)]
pub(super) struct Cache(Mutex<HashMap<Key, Raw>>);
impl Cache {
    pub(super) fn raw_with(&self, view: &MatchView, include_map_elo: bool) -> Result<Raw> {
        let map_count = if include_map_elo {
            view.map_history.at(view.cutoff).count()
        } else {
            0
        };
        let key = (
            view.game.id,
            view.cutoff,
            view.history_before.len(),
            map_count,
        );
        if let Some(raw) = self
            .0
            .lock()
            .map_err(|_| "stacker cache poisoned")?
            .get(&key)
        {
            return Ok(*raw);
        }
        let raw = raw_with(view, include_map_elo)?;
        self.0
            .lock()
            .map_err(|_| "stacker cache poisoned")?
            .insert(key, raw);
        Ok(raw)
    }
}

/// Reads every input of `view`, each checked against its cutoff.
fn raw_with(view: &MatchView, include_map_elo: bool) -> Result<Raw> {
    let mut observed = view.observed_at();
    let mut seen = |at: Instant| observed = Some(observed.map_or(at, |o| o.max(at)));
    let history = view.history_before()?;
    let own_logit = match view.history_before.observed_at() {
        Some(at) => {
            let table = own_rating::Glicko2Table::from_matches(&history);
            let x = table.logit_diff(
                &team_key(&view.game.team_a.name),
                &team_key(&view.game.team_b.name),
                view.cutoff,
            );
            if x.is_some() {
                seen(at);
            }
            x
        }
        None => None,
    };
    let per_map = if include_map_elo {
        per_map_rating::map_signal(view)?.map(|(s, at)| {
            seen(at);
            s.series
        })
    } else {
        None
    };
    let [vrs, hltv] = rank_gap::gaps(view)?;
    let mut gap = |g: Option<AsOf<f64>>| -> Result<Option<f64>> {
        let Some(g) = g else { return Ok(None) };
        seen(g.observed_at());
        Ok(Some(*g.at(view.cutoff)?))
    };
    let (vrs_gap, hltv_gap) = (gap(vrs)?, gap(hltv)?);
    Ok(Raw {
        market: view.market()?,
        valve_map: view.signal()?.map(|s| s.map),
        own_logit,
        per_map,
        vrs_gap,
        hltv_gap,
        observed,
    })
}

pub(super) fn fit_signal(
    cache: &Arc<Cache>,
    training: &[Observed],
    include_map_elo: bool,
) -> Result<Box<dyn FittedSignal>> {
    let rows = training
        .iter()
        .map(|o| {
            Ok(Row {
                raw: cache.raw_with(&o.view, include_map_elo)?,
                format: o.view.game.format,
                event: o.view.game.event_id,
                cutoff: o.view.cutoff,
                won: o.team_a_won,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(Box::new(Fitted {
        stacked: Stacked::fit_with(&rows, include_map_elo),
        cache: cache.clone(),
        include_map_elo,
    }))
}

#[derive(Default)]
pub struct Stacker(Arc<Cache>);

impl CandidateSignal for Stacker {
    fn name(&self) -> &'static str {
        NAME
    }
    fn description(&self) -> &'static str {
        "a combiner fitted to pick winners: the market line and the calibrated Valve rating (one \
         logit scale per format) as base inputs, own_rating, per_map_rating and the VRS and HLTV \
         rank gaps as extras (a missing one reads as no shift), fitted on the Events that ended \
         before. Objective: the weights minimise the log-loss penalised towards the anchor; the \
         penalty, which sets how far the fit moves from the anchor, is chosen from \
         1/3/10/30/100/1000 by accuracy (share of winners picked at probability above one half) \
         on the latest third of the training Events, refitted on the training Events before \
         them (ties: lower log-loss, then the larger penalty); so the side it picks is that of \
         the setting that picked most winners on Matches it was not fitted on, and its \
         probability stays a log-loss fit"
    }
    fn needs(&self) -> &'static str {
        "a market line or a Valve rating, and Matches of Events that ended before"
    }
    fn added_after_list_fixed(&self) -> bool {
        true
    }
    fn fit(&self, training: &[Observed]) -> Result<Box<dyn FittedSignal>> {
        fit_signal(&self.0, training, true)
    }
}

struct Fitted {
    stacked: Stacked,
    cache: Arc<Cache>,
    include_map_elo: bool,
}
impl FittedSignal for Fitted {
    fn predict(&self, m: &MatchView) -> Result<Option<AsOf<f64>>> {
        let raw = self.cache.raw_with(m, self.include_map_elo)?;
        let (Some(p), Some(observed)) = (self.stacked.predict(&raw, m.game.format), raw.observed)
        else {
            return Ok(None);
        };
        Ok(Some(AsOf::new(p, observed, m.cutoff)?))
    }
    fn parameters(&self) -> Option<Value> {
        Some(self.stacked.parameters())
    }
}
