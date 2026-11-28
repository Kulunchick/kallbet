//! Candidate Signal `stacker_no_map_elo` (ticket #59, from #56's criterion C4;
//! added after spec #33's candidate list was fixed): ablation of `stacker`
//! without `per_map_rating::map_signal`, so `stacker` vs `stacker_no_map_elo`
//! isolates the map-level Elo input on the exact same Matches and fitting rule.
use super::{CandidateSignal, FittedSignal, Observed, stacker};
use crate::*;
use std::sync::Arc;

pub const NAME: &str = "stacker_no_map_elo";

/// Pure fit entry point without `per_map_rating` for unit tests.
pub fn fit(rows: &[stacker::Row]) -> stacker::Stacked {
    stacker::Stacked::fit_with(rows, false)
}

#[derive(Default)]
pub struct StackerNoMapElo(Arc<stacker::Cache>);

impl CandidateSignal for StackerNoMapElo {
    fn name(&self) -> &'static str {
        NAME
    }
    fn description(&self) -> &'static str {
        "ablation of `stacker` with identical fitting and inputs minus `per_map_rating`: \
         `stacker` vs `stacker_no_map_elo` isolates the map-level Elo input on the same Matches"
    }
    fn needs(&self) -> &'static str {
        "a market line or a Valve rating, and Matches of Events that ended before"
    }
    fn added_after_list_fixed(&self) -> bool {
        true
    }
    fn fit(&self, training: &[Observed]) -> Result<Box<dyn FittedSignal>> {
        stacker::fit_signal(&self.0, training, false)
    }
}
