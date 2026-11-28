//! The protocol's statistics, as pure functions: per-Match scores, the paired
//! bootstrap over whole Events, the smallest detectable effect and the verdict.
use super::Protocol;
use crate::ports::{RandomSource, SeededRng};
use serde::Serialize;
use std::collections::BTreeMap;

/// Probabilities are clipped here before the logarithm, as in the backtest's report.
pub const CLIP: f64 = 1e-6;

/// Negative natural log of the probability given to the winner.
pub fn log_loss(p_a: f64, a_won: bool) -> f64 {
    let p = if a_won { p_a } else { 1.0 - p_a };
    -p.clamp(CLIP, 1.0).ln()
}

/// Squared error of the probability given to the winner.
pub fn brier(p_a: f64, a_won: bool) -> f64 {
    let p = if a_won { p_a } else { 1.0 - p_a };
    (1.0 - p).powi(2)
}

/// Whether the pick is right, team A being picked at even odds (as the policies do).
pub fn correct(p_a: f64, a_won: bool) -> bool {
    (p_a >= 0.5) == a_won
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Interval {
    pub low: f64,
    pub high: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Bootstrap {
    /// The pooled mean over every Match.
    pub mean: f64,
    pub interval: Interval,
    /// Standard deviation of the resampled means.
    pub std_error: f64,
}

/// Paired bootstrap that resamples whole Events: `diffs` holds one
/// `(event id, difference)` per Match; each resample draws as many Events as
/// there are, with replacement, and takes the mean over all their Matches. The
/// interval is the central `level` of the resampled means (linear interpolation
/// between order statistics). `None` without a Match or without a resample.
pub fn event_bootstrap(
    diffs: &[(i64, f64)],
    resamples: usize,
    level: f64,
    seed: u64,
) -> Option<Bootstrap> {
    if diffs.is_empty() || resamples == 0 {
        return None;
    }
    let mut events: BTreeMap<i64, (f64, usize)> = BTreeMap::new();
    for (event, d) in diffs {
        let e = events.entry(*event).or_default();
        e.0 += d;
        e.1 += 1;
    }
    let events: Vec<(f64, usize)> = events.into_values().collect();
    let mean = diffs.iter().map(|d| d.1).sum::<f64>() / diffs.len() as f64;
    let mut rng = SeededRng::new(seed);
    let k = events.len();
    let mut means: Vec<f64> = (0..resamples)
        .map(|_| {
            let (mut sum, mut n) = (0.0, 0usize);
            for _ in 0..k {
                let (s, c) = events[(rng.next_u64() % k as u64) as usize];
                sum += s;
                n += c;
            }
            sum / n as f64
        })
        .collect();
    means.sort_by(f64::total_cmp);
    let avg = means.iter().sum::<f64>() / means.len() as f64;
    let var = means.iter().map(|m| (m - avg).powi(2)).sum::<f64>() / means.len() as f64;
    let tail = (1.0 - level) / 2.0;
    Some(Bootstrap {
        mean,
        interval: Interval {
            low: quantile(&means, tail),
            high: quantile(&means, 1.0 - tail),
        },
        // Below rounding noise, a degenerate spread is zero.
        std_error: if var.sqrt() < 1e-12 { 0.0 } else { var.sqrt() },
    })
}

fn quantile(sorted: &[f64], q: f64) -> f64 {
    let h = (sorted.len() - 1) as f64 * q;
    let (lo, hi) = (h.floor() as usize, h.ceil() as usize);
    sorted[lo] + (h - h.floor()) * (sorted[hi] - sorted[lo])
}

/// z of a one-sided 5% test (the 90% interval) plus z of 80% power.
pub const DETECTABLE_Z: f64 = 1.644_853_6 + 0.841_621_2;

/// The smallest true difference that the 90% interval would exclude zero for
/// with 80% probability, given the bootstrap's standard error (normal
/// approximation): what the available Matches could have detected.
pub fn detectable_effect(std_error: f64) -> f64 {
    DETECTABLE_Z * std_error
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Verdict {
    #[serde(rename = "promote")]
    Promote,
    #[serde(rename = "harmful")]
    Harmful,
    #[serde(rename = "no evidence")]
    NoEvidence,
    #[serde(rename = "not testable")]
    NotTestable,
}
impl Verdict {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Promote => "promote",
            Self::Harmful => "harmful",
            Self::NoEvidence => "no evidence",
            Self::NotTestable => "not testable",
        }
    }
}

/// The written rule, on a Candidate Signal that could be computed. `interval`
/// is that of the log-loss difference (variant − anchor: below zero is better);
/// `market_difference` the mean log-loss difference on the paired Matches with
/// a market line (`None`: there are none, so none is made worse).
///
/// `accuracy` is the interval of the accuracy difference (variant − anchor:
/// above zero is better) on the same paired Matches, from the same bootstrap.
/// KallBet scores only the picked winner, so a Candidate Signal that gives
/// better probabilities but picks fewer winners is not promoted.
///
/// - fewer than `min_matches` paired Matches: `no evidence`, whatever the numbers;
/// - the interval below zero, the market subset not worse and the accuracy
///   interval not wholly below zero: `promote`; with the accuracy interval
///   wholly below zero: `no evidence` ("better probabilities but fewer winners
///   picked");
/// - the interval above zero: `harmful`;
/// - anything else: `no evidence`.
pub fn verdict(
    paired: usize,
    interval: Interval,
    accuracy: Option<Interval>,
    market_difference: Option<f64>,
    protocol: &Protocol,
) -> (Verdict, String) {
    if paired < protocol.min_matches {
        return (
            Verdict::NoEvidence,
            format!(
                "computed on {paired} paired Matches, under the floor of {}",
                protocol.min_matches
            ),
        );
    }
    if interval.high < 0.0 {
        return match market_difference {
            Some(d) if d > 0.0 => (
                Verdict::NoEvidence,
                format!("better overall, but worse on the Matches with a market line ({d:+.4})"),
            ),
            _ if accuracy.is_some_and(|a| a.high < 0.0) => (
                Verdict::NoEvidence,
                "better probabilities but fewer winners picked: the interval of the log-loss difference is below zero, that of the accuracy difference too"
                    .into(),
            ),
            _ => (
                Verdict::Promote,
                "the interval of the log-loss improvement excludes zero and the Matches with a \
                 market line are not worse"
                    .into(),
            ),
        };
    }
    if interval.low > 0.0 {
        return (
            Verdict::Harmful,
            "the interval of the log-loss difference is above zero: worse than the anchor".into(),
        );
    }
    (
        Verdict::NoEvidence,
        "the interval of the log-loss difference includes zero".into(),
    )
}

/// The Event-bootstrap interval of the accuracy difference (variant − anchor)
/// over `(event, team A won, anchor, variant)` rows, with the protocol's
/// resamples, level and seed: the winners-picked side of the verdict.
pub fn accuracy_bootstrap(
    rows: impl Iterator<Item = (i64, bool, f64, f64)>,
    protocol: &Protocol,
) -> Option<Interval> {
    let diffs: Vec<(i64, f64)> = rows
        .map(|(event, won, anchor, variant)| {
            let hit = |p: f64| f64::from(u8::from(correct(p, won)));
            (event, hit(variant) - hit(anchor))
        })
        .collect();
    event_bootstrap(&diffs, protocol.resamples, protocol.level, protocol.seed).map(|b| b.interval)
}
