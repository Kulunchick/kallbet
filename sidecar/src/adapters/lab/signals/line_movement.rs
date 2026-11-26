//! Candidate Signal `line_movement` (spec #33, story 22): how the line moved
//! before the cutoff. The line is the first market source (the Odds Source
//! order) that has price points for the Match. Two features, in logit units:
//! `total`, the closing price against the opening one, and `late`, the closing
//! price against the last one at least `LATE_HOURS` before the cutoff (the
//! opening one when there is none). A Match whose line has fewer than two
//! points has **no** movement, so no prediction (never a zero).
//!
//! The variant is `sigmoid(logit(anchor) + b1 * total + b2 * late)`, with the
//! two coefficients fitted by ridge-penalised maximum likelihood on the
//! training Matches (at least `MIN_FIT` of them with a movement), else no prediction.
use super::{CandidateSignal, FittedSignal, MatchView, Observed};
use crate::adapters::backtest::{ReplayArchive, feeds::MarketSource};
use crate::*;

pub const NAME: &str = "line_movement";
pub const MIN_FIT: usize = 30;
pub const LATE_HOURS: i64 = 6;
const RIDGE: f64 = 1.0;

pub struct LineMovement;

/// How a line moved before the cutoff.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Movement {
    pub total: f64,
    pub late: f64,
}

pub fn logit(p: f64) -> f64 {
    let p = p.clamp(1e-9, 1.0 - 1e-9);
    (p / (1.0 - p)).ln()
}
pub fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

/// The movement of a line given as `(observed at, team A's price)`, oldest
/// first, or `None` when it has fewer than two points.
pub fn movement(points: &[(Instant, f64)], cutoff: Instant) -> Option<Movement> {
    if points.len() < 2 {
        return None;
    }
    let (first, last) = (points.first()?, points.last()?);
    let edge = cutoff - chrono::Duration::hours(LATE_HOURS);
    let reference = points.iter().rev().find(|p| p.0 <= edge).unwrap_or(first);
    Some(Movement {
        total: logit(last.1) - logit(first.1),
        late: logit(last.1) - logit(reference.1),
    })
}

/// The movement of the Match's line and the timestamp of its latest price point:
/// the first source with any price points.
pub fn view_movement(m: &MatchView) -> Result<Option<(Movement, Instant)>> {
    for source in MarketSource::ALL {
        let points = m.points(source)?;
        if !points.is_empty() {
            let last_at = points.last().map(|p| p.0);
            return Ok(movement(&points, m.cutoff).zip(last_at));
        }
    }
    Ok(None)
}

/// Ridge-penalised logistic regression of `won` on `(total, late)` with a
/// fixed offset per row (the anchor's logit), by damped Newton's method.
pub fn fit_coefficients(rows: &[(Movement, f64, bool)], ridge: f64) -> [f64; 2] {
    let objective = |b: [f64; 2]| -> f64 {
        let ll: f64 = rows
            .iter()
            .map(|(mv, offset, won)| {
                let z = offset + b[0] * mv.total + b[1] * mv.late;
                let p = sigmoid(z);
                (if *won { p } else { 1.0 - p }).clamp(1e-12, 1.0).ln()
            })
            .sum();
        ll - 0.5 * ridge * (b[0] * b[0] + b[1] * b[1])
    };
    let mut b = [0.0f64; 2];
    let mut best = objective(b);
    for _ in 0..50 {
        let mut g = [-ridge * b[0], -ridge * b[1]];
        let mut h = [[ridge, 0.0], [0.0, ridge]];
        for (mv, offset, won) in rows {
            let x = [mv.total, mv.late];
            let p = sigmoid(offset + b[0] * x[0] + b[1] * x[1]);
            let r = f64::from(u8::from(*won)) - p;
            let w = p * (1.0 - p);
            for i in 0..2 {
                g[i] += r * x[i];
                for j in 0..2 {
                    h[i][j] += w * x[i] * x[j];
                }
            }
        }
        let det = h[0][0] * h[1][1] - h[0][1] * h[1][0];
        if det.abs() < 1e-12 {
            break;
        }
        let step = [
            (h[1][1] * g[0] - h[0][1] * g[1]) / det,
            (h[0][0] * g[1] - h[1][0] * g[0]) / det,
        ];
        let mut alpha = 1.0;
        let mut next = b;
        let mut accepted = false;
        for _ in 0..20 {
            let candidate = [b[0] + alpha * step[0], b[1] + alpha * step[1]];
            let val = objective(candidate);
            if val >= best - 1e-12 {
                next = candidate;
                best = val;
                accepted = true;
                break;
            }
            alpha *= 0.5;
        }
        if !accepted {
            break;
        }
        let delta = (next[0] - b[0]).abs().max((next[1] - b[1]).abs());
        b = next;
        if delta < 1e-10 {
            break;
        }
    }
    b
}

impl CandidateSignal for LineMovement {
    fn name(&self) -> &'static str {
        NAME
    }
    fn description(&self) -> &'static str {
        "the anchor's logit moved by how the line moved (closing against opening, and \
         closing against six hours before the cutoff), coefficients fitted walk-forward"
    }
    fn needs(&self) -> &'static str {
        "a line with at least two price points before the cutoff, and at least 30 such Matches of Events that ended before"
    }
    fn missing_data(&self, archive: &ReplayArchive) -> Option<String> {
        (archive.histories.is_empty()
            && archive.polymarket_histories.is_empty()
            && archive.kalshi_candles.is_empty())
        .then(|| "the archive holds no market price history".into())
    }
    fn fit(&self, training: &[Observed]) -> Result<Box<dyn FittedSignal>> {
        let mut rows = Vec::new();
        for o in training {
            if let (Some((mv, _)), Some(anchor)) = (view_movement(&o.view)?, o.view.anchor()?) {
                rows.push((mv, logit(anchor), o.team_a_won));
            }
        }
        let coefficients = (rows.len() >= MIN_FIT).then(|| fit_coefficients(&rows, RIDGE));
        Ok(Box::new(Fitted { coefficients }))
    }
}

struct Fitted {
    coefficients: Option<[f64; 2]>,
}
impl FittedSignal for Fitted {
    fn predict(&self, m: &MatchView) -> Result<Option<AsOf<f64>>> {
        let (Some(b), Some((mv, mv_at)), Some(anchor)) =
            (self.coefficients, view_movement(m)?, m.anchor()?)
        else {
            return Ok(None);
        };
        let observed = m.observed_at().map_or(mv_at, |a| a.max(mv_at));
        let p = sigmoid(logit(anchor) + b[0] * mv.total + b[1] * mv.late);
        Ok(Some(AsOf::new(p, observed, m.cutoff)?))
    }
}
