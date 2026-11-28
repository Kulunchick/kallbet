//! Candidate Signal `learned_weights` (spec #33, story 20; ticket #37): the
//! anchor's market and rating weights learned walk-forward by the combiner
//! instead of fixed at 0.80 and 0.20. The fitted weights of every evaluated
//! Event are in the report (`FittedSignal::parameters`).
use super::{CandidateSignal, FittedSignal, MatchView, Observed};
use crate::adapters::lab::combiner::{Combiner, Inputs, LAMBDA};
use crate::*;

pub const NAME: &str = "learned_weights";

pub struct LearnedWeights;

fn inputs(view: &MatchView) -> Result<Inputs> {
    Ok(Inputs::base(
        view.market()?,
        view.signal()?.map(|s| s.series),
    ))
}

impl CandidateSignal for LearnedWeights {
    fn name(&self) -> &'static str {
        NAME
    }
    fn description(&self) -> &'static str {
        "the anchor's market and Valve rating weights (logit space) learned by the combiner on \
         the Events that ended before, one set per availability of the two inputs, regularised \
         towards 0.80 / 0.20"
    }
    fn needs(&self) -> &'static str {
        "a market line or a Valve rating"
    }
    fn fit(&self, training: &[Observed]) -> Result<Box<dyn FittedSignal>> {
        let rows = training
            .iter()
            .map(|o| Ok((inputs(&o.view)?, o.team_a_won)))
            .collect::<Result<Vec<_>>>()?;
        Ok(Box::new(Learned(Combiner::fit(&[], &rows, LAMBDA))))
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Learned(pub Combiner);
impl FittedSignal for Learned {
    fn deployment(&self) -> Result<Option<serde_json::Value>> {
        Ok(Some(serde_json::to_value(self)?))
    }
    fn predict(&self, m: &MatchView) -> Result<Option<AsOf<f64>>> {
        let (Some(p), Some(observed)) = (self.0.predict(&inputs(m)?), m.observed_at()) else {
            return Ok(None);
        };
        Ok(Some(AsOf::new(p, observed, m.cutoff)?))
    }
    fn parameters(&self) -> Option<serde_json::Value> {
        Some(self.0.parameters())
    }
}
