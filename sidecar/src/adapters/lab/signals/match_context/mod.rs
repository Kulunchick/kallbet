//! Walk-forward logit offsets for the five results-context Candidate Signals.
pub mod lan_online;
pub mod region_travel;
pub mod rest_schedule_load;
pub mod series_format;
pub mod stakes;

use super::{CandidateSignal, FittedSignal, MatchView, Observed};
use crate::adapters::lab::combiner::{logit, ridge_logistic_with_offset, sigmoid};
use crate::{AsOf, HistoricalMatch, Instant, Result, team_names::team_key};
use std::sync::Arc;

const MIN_FIT: usize = 20;
const RIDGE: f64 = 10.0;

type Features = (Vec<f64>, Instant);

#[derive(Clone)]
struct ContextSignal {
    name: &'static str,
    description: &'static str,
    needs: &'static str,
    columns: &'static [&'static str],
    read: fn(&MatchView) -> Result<Option<Features>>,
}

fn team_features<T>(
    view: &MatchView,
    read: impl Fn(&str, &[&HistoricalMatch]) -> Option<T>,
) -> Result<Option<(T, T, Instant)>> {
    let a = read(&team_key(&view.game.team_a.name), &view.team_a_history()?);
    let b = read(&team_key(&view.game.team_b.name), &view.team_b_history()?);
    Ok(a.zip(b).zip(history_at(view)).map(|((a, b), at)| (a, b, at)))
}

fn history_at(view: &MatchView) -> Option<Instant> {
    view.team_a_history.observed_at().max(view.team_b_history.observed_at())
}

fn win_rate(matches: impl Iterator<Item = bool>) -> Option<f64> {
    let (wins, played) = matches.fold((0, 0), |(wins, played), won| {
        (wins + u32::from(won), played + 1)
    });
    (played > 0).then(|| f64::from(wins + 1) / f64::from(played + 2))
}

impl CandidateSignal for ContextSignal {
    fn name(&self) -> &'static str { self.name }
    fn description(&self) -> &'static str { self.description }
    fn needs(&self) -> &'static str { self.needs }

    fn feature_names(&self) -> Vec<&'static str> { self.columns.to_vec() }

    fn fit(&self, training: &[Observed]) -> Result<Box<dyn FittedSignal>> {
        let mut rows = Vec::new();
        for match_ in training {
            let view = &match_.view;
            if let (Some(anchor), Some((values, at))) = (view.anchor()?, (self.read)(view)?) {
                AsOf::new((), at, view.cutoff)?;
                rows.push((logit(anchor), values, match_.team_a_won));
            }
        }
        let weights = (rows.len() >= MIN_FIT).then(|| {
            let refs: Vec<_> = rows.iter().map(|(offset, values, won)| {
                (*offset, values.as_slice(), *won)
            }).collect();
            ridge_logistic_with_offset(&refs, &vec![0.0; self.columns.len()], RIDGE)
        });
        Ok(Box::new(FittedContext { signal: self.clone(), weights }))
    }
}

#[derive(Clone)]
struct FittedContext {
    signal: ContextSignal,
    weights: Option<Vec<f64>>,
}

impl FittedSignal for FittedContext {
    fn predict(&self, m: &MatchView) -> Result<Option<AsOf<f64>>> {
        let (Some(weights), Some(anchor), Some((values, at))) =
            (&self.weights, m.anchor()?, (self.signal.read)(m)?)
        else {
            return Ok(None);
        };
        let shift: f64 = weights.iter().zip(values).map(|(w, x)| w * x).sum();
        let at = Some(at).max(m.observed_at()).unwrap_or(at);
        Ok(Some(AsOf::new(sigmoid(logit(anchor) + shift), at, m.cutoff)?))
    }

    fn features(&self, m: &MatchView) -> Result<Vec<Option<AsOf<f64>>>> {
        let Some((values, at)) = (self.signal.read)(m)? else {
            return Ok(vec![None; self.signal.columns.len()]);
        };
        values.into_iter().map(|value| AsOf::new(value, at, m.cutoff).map(Some))
            .collect::<std::result::Result<_, _>>().map_err(Into::into)
    }

    fn parameters(&self) -> Option<serde_json::Value> {
        self.weights.as_ref().map(|weights| serde_json::json!({
            "features": self.signal.columns,
            "weights": weights,
        }))
    }
}

pub(super) fn registry() -> [Arc<dyn CandidateSignal>; 5] {
    [
        Arc::new(rest_schedule_load::SIGNAL),
        Arc::new(stakes::SIGNAL),
        Arc::new(lan_online::SIGNAL),
        Arc::new(series_format::SIGNAL),
        Arc::new(region_travel::SIGNAL),
    ]
}

