//! Candidate Signal `market_rank_rule` (ticket #60, from #56's criterion C1;
//! added after spec #33's candidate list was fixed): a pick rule for Matches
//! where the market favourite and the rank favourite disagree.
//!
//! The rank favourite is the better team by VRS rank (the latest dated Valve
//! standings before the cutoff), else by HLTV world rank, else the Valve
//! rating's favourite. On the training Events only, the rule counts, per size
//! of the disagreement (`|ln rank_b - ln rank_a|`, three buckets) and series
//! format, how often the rank favourite beat the market favourite. In a
//! disagreement it picks the side that won more often there and gives it the
//! smoothed win rate `(wins + 1) / (n + 2)`; a cell with fewer than
//! `MIN_CELL` rows uses its size bucket pooled over formats, and a bucket with
//! no rows follows the anchor. Where the two agree (or either has no
//! favourite) it follows the anchor.
use super::{CandidateSignal, FittedSignal, MatchView, Observed};
use crate::*;
use std::collections::BTreeMap;

pub const NAME: &str = "market_rank_rule";
/// Upper edges of the first two size buckets of `|ln rank gap|`.
const EDGES: [f64; 2] = [0.5, 1.0];
/// Rows a (size, format) cell needs to be used alone.
const MIN_CELL: usize = 5;

pub struct MarketRankRule;

fn bucket(gap: f64) -> usize {
    EDGES.iter().take_while(|e| gap.abs() >= **e).count()
}

fn format_key(f: Format) -> u8 {
    match f {
        Format::Bo1 => 1,
        Format::Bo3 => 3,
        Format::Bo5 => 5,
    }
}

/// How often the rank favourite beat the market favourite, per cell.
#[derive(Default, Clone)]
pub struct RuleTable {
    /// (size bucket, format) -> (rank favourite wins, rows).
    cells: BTreeMap<(usize, u8), (usize, usize)>,
}

impl RuleTable {
    /// Rows: (absolute rank gap, format, whether the rank favourite won).
    pub fn fit(rows: &[(f64, Format, bool)]) -> Self {
        let mut cells: BTreeMap<(usize, u8), (usize, usize)> = BTreeMap::new();
        for (gap, format, rank_won) in rows {
            let c = cells.entry((bucket(*gap), format_key(*format))).or_default();
            c.0 += usize::from(*rank_won);
            c.1 += 1;
        }
        Self { cells }
    }

    /// `(pick the rank favourite, probability of the picked side)` for a
    /// disagreement, or `None` when training saw none of that size. A tie
    /// goes to the market side.
    pub fn pick(&self, gap: f64, format: Format) -> Option<(bool, f64)> {
        let b = bucket(gap);
        let own = self.cells.get(&(b, format_key(format))).copied();
        let (wins, n) = match own {
            Some(c) if c.1 >= MIN_CELL => c,
            _ => self
                .cells
                .iter()
                .filter(|((cb, _), _)| *cb == b)
                .fold((0, 0), |a, (_, c)| (a.0 + c.0, a.1 + c.1)),
        };
        if n == 0 {
            return None;
        }
        let rank = wins * 2 > n;
        let picked = if rank { wins } else { n - wins };
        Some((rank, (picked as f64 + 1.0) / (n as f64 + 2.0)))
    }

    fn parameters(&self) -> serde_json::Value {
        let cells: Vec<_> = self
            .cells
            .iter()
            .map(|((b, f), (w, n))| {
                serde_json::json!({"size_bucket": b, "format": f, "rank_wins": w, "rows": n})
            })
            .collect();
        serde_json::json!({ "cells": cells })
    }
}

/// The disagreement of `view`: `(rank favourite is team A, |rank gap|)` when
/// the market favourite is the other team.
fn disagreement(view: &MatchView) -> Result<Option<(bool, f64)>> {
    let Some(market) = view.market()? else { return Ok(None) };
    if market == 0.5 {
        return Ok(None);
    }
    let gap = |pair: &Option<AsOf<(u32, u32)>>| -> Result<Option<f64>> {
        let Some(pair) = pair else { return Ok(None) };
        let (a, b) = *pair.at(view.cutoff)?;
        let g = f64::from(b.max(1)).ln() - f64::from(a.max(1)).ln();
        Ok((g != 0.0).then_some(g))
    };
    let g = match gap(&view.ranks.vrs)? {
        Some(g) => Some(g),
        None => gap(&view.ranks.hltv)?,
    };
    let (rank_is_a, size) = match g {
        Some(g) => (g > 0.0, g.abs()),
        None => match view.signal()? {
            Some(s) if s.series != 0.5 && s.series > 0.0 && s.series < 1.0 => {
                (s.series > 0.5, (s.series / (1.0 - s.series)).ln().abs())
            }
            _ => return Ok(None),
        },
    };
    Ok((rank_is_a != (market > 0.5)).then_some((rank_is_a, size)))
}

impl CandidateSignal for MarketRankRule {
    fn name(&self) -> &'static str {
        NAME
    }
    fn description(&self) -> &'static str {
        "where the market favourite and the rank favourite (VRS, else HLTV, else the Valve \
         rating) disagree, picks the side that won more often in such disagreements on the \
         Events that ended before, by size of the disagreement and series format; where they \
         agree it follows the anchor"
    }
    fn needs(&self) -> &'static str {
        "a market line and a rank (VRS or HLTV) or Valve rating for both teams"
    }
    fn added_after_list_fixed(&self) -> bool {
        true
    }
    fn subset_label(&self) -> Option<&'static str> {
        Some("disagreement Matches (market favourite and rank favourite differ)")
    }
    fn fit(&self, training: &[Observed]) -> Result<Box<dyn FittedSignal>> {
        let mut rows = vec![];
        for o in training {
            if let Some((rank_is_a, size)) = disagreement(&o.view)? {
                rows.push((size, o.view.game.format, rank_is_a == o.team_a_won));
            }
        }
        Ok(Box::new(Fitted(RuleTable::fit(&rows))))
    }
}

struct Fitted(RuleTable);
impl FittedSignal for Fitted {
    fn predict(&self, m: &MatchView) -> Result<Option<AsOf<f64>>> {
        let (Some(anchor), Some(observed)) = (m.anchor()?, m.observed_at()) else {
            return Ok(None);
        };
        let p = match disagreement(m)? {
            Some((rank_is_a, size)) => match self.0.pick(size, m.game.format) {
                Some((pick_rank, p)) => {
                    if rank_is_a == pick_rank { p } else { 1.0 - p }
                }
                None => anchor,
            },
            None => anchor,
        };
        Ok(Some(AsOf::new(p, observed, m.cutoff)?))
    }
    fn in_subset(&self, m: &MatchView) -> Result<bool> {
        Ok(disagreement(m)?.is_some())
    }
    fn parameters(&self) -> Option<serde_json::Value> {
        Some(self.0.parameters())
    }
}
