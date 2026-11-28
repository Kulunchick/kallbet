//! Candidate Signal `rank_gap` (ticket #57, from #56's criterion C1; added
//! after spec #33's candidate list was fixed): the gap between the teams'
//! ranks, `ln(rank_b) − ln(rank_a)` (positive when team A is ranked better),
//! for the VRS rank (the latest dated Valve standings before the cutoff) and
//! the HLTV world rank (the dated weekly ranking on the HLTV match page
//! badges). The VRS badge of those pages is not read: the leakage audit found
//! it moves with the Match's own result (docs/signal-lab/criteria-research.md).
//!
//! The two gaps enter the combiner beside the anchor's market and rating
//! inputs, fitted walk-forward on the training Matches that have a gap,
//! regularised towards the anchor (gap weights towards zero). A Match without
//! either gap gets no prediction.
use super::{CandidateSignal, FittedSignal, MatchView, Observed};
use crate::adapters::lab::combiner::{Combiner, Inputs, LAMBDA};
use crate::*;

pub const NAME: &str = "rank_gap";
pub const FEATURES: [&str; 2] = ["vrs_rank_gap", "hltv_rank_gap"];

pub struct RankGap;

/// `ln(b) − ln(a)` of a pair of ranks, with its observation.
fn gap(pair: &Option<AsOf<(u32, u32)>>, cutoff: Instant) -> Result<Option<AsOf<f64>>> {
    let Some(pair) = pair else { return Ok(None) };
    let (a, b) = *pair.at(cutoff)?;
    let value = f64::from(b.max(1)).ln() - f64::from(a.max(1)).ln();
    Ok(Some(AsOf::new(value, pair.observed_at(), cutoff)?))
}

/// The two gaps of `view`, VRS then HLTV.
pub fn gaps(view: &MatchView) -> Result<[Option<AsOf<f64>>; 2]> {
    Ok([
        gap(&view.ranks.vrs, view.cutoff)?,
        gap(&view.ranks.hltv, view.cutoff)?,
    ])
}

fn inputs(view: &MatchView, gaps: &[Option<AsOf<f64>>; 2]) -> Result<Inputs> {
    Ok(Inputs {
        market: view.market()?,
        rating: view.signal()?.map(|s| s.series),
        extras: gaps
            .iter()
            .map(|g| g.as_ref().map(|g| g.at(view.cutoff).copied()).transpose())
            .collect::<Result<_, _>>()?,
    })
}

impl CandidateSignal for RankGap {
    fn name(&self) -> &'static str {
        NAME
    }
    fn description(&self) -> &'static str {
        "the log rank gap of the two teams in Valve's dated standings (VRS) and in HLTV's dated \
         weekly world ranking (HLTV match page badges), combined with the anchor's market and \
         rating by the combiner fitted on the Events that ended before"
    }
    fn needs(&self) -> &'static str {
        "both teams' VRS rank (dated Valve standings) or HLTV world rank (HLTV match page badges)"
    }
    fn added_after_list_fixed(&self) -> bool {
        true
    }
    fn feature_names(&self) -> Vec<&'static str> {
        FEATURES.to_vec()
    }
    fn fit(&self, training: &[Observed]) -> Result<Box<dyn FittedSignal>> {
        let mut rows = vec![];
        for o in training {
            let g = gaps(&o.view)?;
            if g.iter().any(Option::is_some) {
                rows.push((inputs(&o.view, &g)?, o.team_a_won));
            }
        }
        let names: Vec<String> = FEATURES.iter().map(|s| (*s).to_owned()).collect();
        Ok(Box::new(Fitted(Combiner::fit(&names, &rows, LAMBDA))))
    }
}

struct Fitted(Combiner);
impl FittedSignal for Fitted {
    fn predict(&self, m: &MatchView) -> Result<Option<AsOf<f64>>> {
        let g = gaps(m)?;
        let ranked = g.iter().flatten().map(AsOf::observed_at).max();
        let Some(ranked) = ranked else { return Ok(None) };
        let Some(p) = self.0.predict(&inputs(m, &g)?) else {
            return Ok(None);
        };
        let observed = m.observed_at().map_or(ranked, |o| o.max(ranked));
        Ok(Some(AsOf::new(p, observed, m.cutoff)?))
    }
    fn parameters(&self) -> Option<serde_json::Value> {
        Some(self.0.parameters())
    }
    fn features(&self, m: &MatchView) -> Result<Vec<Option<AsOf<f64>>>> {
        Ok(gaps(m)?.into_iter().collect())
    }
}
