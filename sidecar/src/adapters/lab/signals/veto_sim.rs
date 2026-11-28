//! Format-agnostic veto simulation routing each Match to `bo1_veto_sim`,
//! `bo3_veto_sim` or `bo5_veto_sim` by series format (ticket #54).
use super::{
    CandidateSignal, FittedSignal, MatchView, Observed, bo1_veto_sim::Bo1VetoSim,
    bo3_veto_sim::Bo3VetoSim, bo5_veto_sim::Bo5VetoSim,
};
use crate::{AsOf, Format, Result, adapters::backtest::ReplayArchive};

pub struct VetoSim;

impl CandidateSignal for VetoSim {
    fn name(&self) -> &'static str {
        "veto_sim"
    }

    fn description(&self) -> &'static str {
        "Format-agnostic veto simulation routing each Match to Bo1, Bo3 or Bo5 simulation"
    }

    fn needs(&self) -> &'static str {
        "a prior complete seven-step HLTV veto matching the Match format and veto history for either team"
    }

    fn added_after_list_fixed(&self) -> bool {
        true
    }

    fn missing_data(&self, archive: &ReplayArchive) -> Option<String> {
        (!archive.veto_history.has_any()).then(|| {
            "No complete HLTV vetoes linked to finished PandaScore Matches in the archive".into()
        })
    }

    fn fit(&self, _: &[Observed]) -> Result<Box<dyn FittedSignal>> {
        Ok(Box::new(VetoSim))
    }
}

impl FittedSignal for VetoSim {
    fn predict(&self, view: &MatchView) -> Result<Option<AsOf<f64>>> {
        match view.game.format {
            Format::Bo1 => Bo1VetoSim.predict(view),
            Format::Bo3 => Bo3VetoSim.predict(view),
            Format::Bo5 => Bo5VetoSim.predict(view),
        }
    }
}
