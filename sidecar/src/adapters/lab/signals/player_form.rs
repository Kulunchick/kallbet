//! Candidate Signal `player_form` (ticket #45, spec #33): each player's
//! recent statistical level against their own longer-run level before the
//! Match.
//!
//! Per the #36 data research (`docs/signal-lab/map-veto-player-research.md`)
//! and operator decision, no permitted source of point-in-time player
//! statistics exists today, so this signal registers as `not testable` and
//! builds no model. A later ticket can plug a permitted source into
//! `missing_data`, `fit`, and `MatchView` without changing the lab protocol.
use super::lineup_strength::{MISSING_DATA_REASON as NO_PLAYER_STATS_REASON, NoModel};
use super::{CandidateSignal, FittedSignal, Observed};
use crate::*;

pub const NAME: &str = "player_form";
pub const MISSING_DATA_REASON: &str = NO_PLAYER_STATS_REASON;

pub struct PlayerForm;

impl CandidateSignal for PlayerForm {
    fn name(&self) -> &'static str {
        NAME
    }

    fn description(&self) -> &'static str {
        "Each player's recent statistical level against their own longer-run level before the Match"
    }

    fn needs(&self) -> &'static str {
        "pre-cutoff Match rosters and point-in-time per-Match player statistics from a permitted \
         source"
    }

    fn missing_data(&self, _archive: &crate::adapters::backtest::ReplayArchive) -> Option<String> {
        Some(MISSING_DATA_REASON.into())
    }

    fn fit(&self, _training: &[Observed]) -> Result<Box<dyn FittedSignal>> {
        Ok(Box::new(NoModel))
    }
}
