//! Candidate Signal `rating_calibration` (spec #33, story 19): the Valve
//! rating's map probability rescaled in logit space, `p' = sigmoid(s * logit(p))`,
//! with `s` fitted by maximum likelihood on the series results of the training
//! Matches, one `s` per series format (the pooled one where a format has fewer
//! than `MIN_FIT` training Matches). The calibrated rating replaces the raw one
//! in the anchor's blend with the market.
use super::{CandidateSignal, FittedSignal, MatchView, Observed};
use crate::*;

pub const NAME: &str = "rating_calibration";
/// Fewest rated training Matches a scale is fitted on.
pub const MIN_FIT: usize = 30;

pub struct RatingCalibration;

fn logit(p: f64) -> f64 {
    let p = p.clamp(1e-9, 1.0 - 1e-9);
    (p / (1.0 - p)).ln()
}
fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}
fn slot(format: Format) -> usize {
    match format {
        Format::Bo1 => 0,
        Format::Bo3 => 1,
        Format::Bo5 => 2,
    }
}

/// The maximum-likelihood scale for `(logit of the map probability, format,
/// team A won)` rows, by golden-section search on `[0.01, 4]`. `None` without rows.
pub fn fit_scale(data: &[(f64, Format, bool)]) -> Option<f64> {
    if data.is_empty() {
        return None;
    }
    let nll = |s: f64| -> f64 {
        data.iter()
            .map(|(x, f, won)| {
                let p = vrs::series_probability(sigmoid(s * x), *f);
                let p = if *won { p } else { 1.0 - p };
                -p.clamp(1e-12, 1.0).ln()
            })
            .sum()
    };
    let ratio = (5f64.sqrt() - 1.0) / 2.0;
    let (mut a, mut b) = (0.01, 4.0);
    let mut c = b - ratio * (b - a);
    let mut d = a + ratio * (b - a);
    let (mut fc, mut fd) = (nll(c), nll(d));
    for _ in 0..100 {
        if fc < fd {
            b = d;
            d = c;
            fd = fc;
            c = b - ratio * (b - a);
            fc = nll(c);
        } else {
            a = c;
            c = d;
            fc = fd;
            d = a + ratio * (b - a);
            fd = nll(d);
        }
    }
    Some((a + b) / 2.0)
}

impl CandidateSignal for RatingCalibration {
    fn name(&self) -> &'static str {
        NAME
    }
    fn description(&self) -> &'static str {
        "Valve rating's map probability rescaled in logit space by a factor fitted per series \
         format, then blended with the market as the anchor does"
    }
    fn needs(&self) -> &'static str {
        "a Valve rating of both teams, and at least 30 rated Matches of Events that ended before"
    }
    fn missing_data(&self, archive: &crate::adapters::backtest::ReplayArchive) -> Option<String> {
        archive.standings.is_empty().then(|| {
            "the archive holds no Valve standings, so no Match has a rating to calibrate".into()
        })
    }
    fn fit(&self, training: &[Observed]) -> Result<Box<dyn FittedSignal>> {
        let mut data = Vec::new();
        for o in training {
            if let Some(s) = o.view.signal()? {
                data.push((logit(s.map), o.view.game.format, o.team_a_won));
            }
        }
        let pooled = (data.len() >= MIN_FIT).then(|| fit_scale(&data)).flatten();
        let mut scales = [pooled; 3];
        for format in [Format::Bo1, Format::Bo3, Format::Bo5] {
            let rows: Vec<_> = data
                .iter()
                .filter(|r| slot(r.1) == slot(format))
                .copied()
                .collect();
            if rows.len() >= MIN_FIT {
                scales[slot(format)] = fit_scale(&rows);
            }
        }
        Ok(Box::new(Calibrated { scales }))
    }
}

/// The fitted scales, by format (`None`: too few training Matches).
pub struct Calibrated {
    pub scales: [Option<f64>; 3],
}
impl FittedSignal for Calibrated {
    fn predict(&self, m: &MatchView) -> Result<Option<AsOf<f64>>> {
        let (Some(signal), Some(scale)) = (m.signal()?, self.scales[slot(m.game.format)]) else {
            return Ok(None);
        };
        let map = sigmoid(scale * logit(signal.map));
        let calibrated = Signal {
            map,
            series: vrs::series_probability(map, m.game.format),
        };
        let (Some(p), Some(observed)) = (m.anchor_with(Some(calibrated))?, m.observed_at()) else {
            return Ok(None);
        };
        Ok(Some(AsOf::new(p, observed, m.cutoff)?))
    }
}
