use super::{CandidateSignal, FittedSignal, MatchView, Observed};
use crate::{AsOf, Result, adapters::backtest::ReplayArchive};

pub struct Bo1VetoSim;

impl CandidateSignal for Bo1VetoSim {
    fn name(&self) -> &'static str {
        "bo1_veto_sim"
    }

    fn description(&self) -> &'static str {
        "Bo1 map veto simulation"
    }

    fn needs(&self) -> &'static str {
        "historical map bans and picks in veto order"
    }

    fn missing_data(&self, _: &ReplayArchive) -> Option<String> {
        Some("Liquipedia records played and skipped maps only. PandaScore's free plan omits vetoes, and HLTV collection is not authorized".into())
    }

    fn fit(&self, _: &[Observed]) -> Result<Box<dyn FittedSignal>> {
        Ok(Box::new(Bo1VetoSim))
    }
}

impl FittedSignal for Bo1VetoSim {
    fn predict(&self, _: &MatchView) -> Result<Option<AsOf<f64>>> {
        Ok(None)
    }
}
