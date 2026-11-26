//! Candidate Signal `lineup_strength` (ticket #45, spec #33): strength of the
//! five players who actually play, from point-in-time individual ratings as
//! they stood before the Match.
//!
//! Per the #36 data research (`docs/signal-lab/map-veto-player-research.md`)
//! and operator decision, no permitted source of point-in-time player
//! statistics exists today, so this signal registers as `not testable` and
//! builds no model. A later ticket can plug a permitted source into
//! `missing_data`, `fit`, and `MatchView` without changing the lab protocol.
use super::{CandidateSignal, FittedSignal, MatchView, Observed};
use crate::*;

pub const NAME: &str = "lineup_strength";
pub const MISSING_DATA_REASON: &str = "no permitted source of point-in-time player statistics \
    exists (PandaScore free plan returns 403 on match/player stats, Liquipedia carries rosters \
    without per-Match player statistics, GRID Open Access excludes tier-1 Majors/ESL/BLAST, and \
    HLTV scraping is not authorized by the operator under HLTV ToS §2.2)";

pub struct LineupStrength;

impl CandidateSignal for LineupStrength {
    fn name(&self) -> &'static str {
        NAME
    }

    fn description(&self) -> &'static str {
        "Strength of the five players who actually play, from point-in-time individual ratings \
         before the Match"
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

pub(super) struct NoModel;

impl FittedSignal for NoModel {
    fn predict(&self, _m: &MatchView) -> Result<Option<AsOf<f64>>> {
        Ok(None)
    }
}
