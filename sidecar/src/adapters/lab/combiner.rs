//! Ticket #37 (spec #33): the lab's combiner, a regularised logistic model over
//! the logits of the market line and of the Valve rating plus the features of
//! Candidate Signals. Pure: it is fitted on the rows it is given and nothing
//! else; the protocol (`runner`) decides which rows those are (walk-forward).
//!
//! The model has no intercept (which team is "A" carries no meaning) and one
//! weight vector per availability of the anchor's inputs, because the anchor
//! itself treats them apart: with both a market line and a rating, the weights
//! of `logit(market)`, `logit(rating)` and the extra features; with only one of
//! them, its weight and the extras'. The penalty `λ/2 · |w − w₀|²` pulls the
//! weights towards the anchor's (`0.80 / 0.20` with both, `1` alone, `0` for
//! every extra feature), so without training Matches the combiner *is* the
//! anchor and it moves away from it only as far as the data carries it.
use serde_json::{Map, Value, json};

/// The penalty of the fit (spec #33: "regularised"), fixed before any result:
/// about the information of fifty Matches in each weight.
pub const LAMBDA: f64 = 10.0;
/// The anchor's weights (`debate::policy`): the penalty's centre.
pub const MARKET_PRIOR: f64 = 0.80;
pub const RATING_PRIOR: f64 = 0.20;

/// Probabilities are clipped here before the logit.
const CLIP: f64 = 1e-6;

pub fn logit(p: f64) -> f64 {
    let p = p.clamp(CLIP, 1.0 - CLIP);
    (p / (1.0 - p)).ln()
}
pub fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

/// What the anchor's inputs give for a Match.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum Pattern {
    /// A market line and a rating.
    Both,
    MarketOnly,
    RatingOnly,
}
impl Pattern {
    pub const ALL: [Pattern; 3] = [Pattern::Both, Pattern::MarketOnly, Pattern::RatingOnly];
    pub fn label(self) -> &'static str {
        match self {
            Pattern::Both => "market+rating",
            Pattern::MarketOnly => "market",
            Pattern::RatingOnly => "rating",
        }
    }
    fn base_names(self) -> &'static [&'static str] {
        match self {
            Pattern::Both => &["market", "rating"],
            Pattern::MarketOnly => &["market"],
            Pattern::RatingOnly => &["rating"],
        }
    }
    fn base_prior(self) -> &'static [f64] {
        match self {
            Pattern::Both => &[MARKET_PRIOR, RATING_PRIOR],
            Pattern::MarketOnly | Pattern::RatingOnly => &[1.0],
        }
    }
}

/// One Match's inputs: team A's market and rating series probabilities, and
/// the extra features (already on a logit scale; `None`: not available, read
/// as zero, no shift).
#[derive(Clone, Debug, PartialEq)]
pub struct Inputs {
    pub market: Option<f64>,
    pub rating: Option<f64>,
    pub extras: Vec<Option<f64>>,
}
impl Inputs {
    pub fn base(market: Option<f64>, rating: Option<f64>) -> Self {
        Self {
            market,
            rating,
            extras: vec![],
        }
    }
    pub fn pattern(&self) -> Option<Pattern> {
        match (self.market, self.rating) {
            (Some(_), Some(_)) => Some(Pattern::Both),
            (Some(_), None) => Some(Pattern::MarketOnly),
            (None, Some(_)) => Some(Pattern::RatingOnly),
            (None, None) => None,
        }
    }
    /// The feature vector of `pattern`'s model, `extras` long in extras.
    fn vector(&self, extras: usize) -> Vec<f64> {
        let mut x: Vec<f64> = [self.market, self.rating]
            .into_iter()
            .flatten()
            .map(logit)
            .collect();
        x.extend((0..extras).map(|i| self.extras.get(i).copied().flatten().unwrap_or(0.0)));
        x
    }
}

/// The weights of one availability pattern and the Matches they were fitted on.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Model {
    pub matches: usize,
    /// Base inputs first (market, rating, as the pattern has them), then the extras.
    pub weights: Vec<f64>,
}

/// A fitted combiner.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Combiner {
    pub extras: Vec<String>,
    models: Vec<(Pattern, Model)>,
}

impl Combiner {
    /// Fits one model per pattern on `rows` (inputs, team A won) by Newton's
    /// method on the penalised log-likelihood (strictly convex for `λ > 0`).
    pub fn fit(extras: &[String], rows: &[(Inputs, bool)], lambda: f64) -> Self {
        let models = Pattern::ALL
            .into_iter()
            .map(|pattern| {
                let data: Vec<(Vec<f64>, bool)> = rows
                    .iter()
                    .filter(|(i, _)| i.pattern() == Some(pattern))
                    .map(|(i, won)| (i.vector(extras.len()), *won))
                    .collect();
                let mut prior = pattern.base_prior().to_vec();
                prior.extend(std::iter::repeat_n(0.0, extras.len()));
                let weights = ridge_logistic(&data, &prior, lambda);
                (
                    pattern,
                    Model {
                        matches: data.len(),
                        weights,
                    },
                )
            })
            .collect();
        Self {
            extras: extras.to_vec(),
            models,
        }
    }

    pub fn model(&self, pattern: Pattern) -> Option<&Model> {
        self.models.iter().find(|m| m.0 == pattern).map(|m| &m.1)
    }

    /// Team A's series probability; `None` with neither a market nor a rating.
    pub fn predict(&self, inputs: &Inputs) -> Option<f64> {
        let model = self.model(inputs.pattern()?)?;
        let x = inputs.vector(self.extras.len());
        Some(sigmoid(dot(&model.weights, &x)))
    }

    /// The fitted weights by pattern, for the report.
    pub fn parameters(&self) -> Value {
        let mut out = Map::new();
        for (pattern, model) in &self.models {
            let mut m = Map::new();
            m.insert("matches".into(), json!(model.matches));
            let names = pattern
                .base_names()
                .iter()
                .map(|s| s.to_string())
                .chain(self.extras.iter().cloned());
            for (name, w) in names.zip(&model.weights) {
                m.insert(name, json!((w * 1e6).round() / 1e6));
            }
            out.insert(pattern.label().into(), Value::Object(m));
        }
        Value::Object(out)
    }
}

fn dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(a, b)| a * b).sum()
}

/// Minimises `Σ log-loss(σ(w·x), y) + λ/2 · |w − prior|²`, starting at `prior`.
pub fn ridge_logistic(rows: &[(Vec<f64>, bool)], prior: &[f64], lambda: f64) -> Vec<f64> {
    let rows: Vec<_> = rows
        .iter()
        .map(|(x, won)| (0.0, x.as_slice(), *won))
        .collect();
    ridge_logistic_with_offset(&rows, prior, lambda)
}

pub(super) fn ridge_logistic_with_offset(
    rows: &[(f64, &[f64], bool)],
    prior: &[f64],
    lambda: f64,
) -> Vec<f64> {
    let k = prior.len();
    let mut w = prior.to_vec();
    if rows.is_empty() || k == 0 {
        return w;
    }
    for _ in 0..100 {
        let mut g: Vec<f64> = (0..k).map(|j| lambda * (w[j] - prior[j])).collect();
        let mut h = vec![vec![0.0; k]; k];
        for (j, row) in h.iter_mut().enumerate() {
            row[j] = lambda;
        }
        for (offset, x, won) in rows {
            let p = sigmoid(offset + dot(&w, x));
            let r = p - f64::from(u8::from(*won));
            let s = p * (1.0 - p);
            for a in 0..k {
                g[a] += r * x[a];
                for b in 0..k {
                    h[a][b] += s * x[a] * x[b];
                }
            }
        }
        let Some(step) = solve(h, g) else { break };
        // A full Newton step can overshoot on near-certain inputs and cycle
        // for ever (ticket #48); halve it until the objective does not rise.
        let before = objective(rows, &w, prior, lambda);
        let mut t = 1.0;
        let mut next: Vec<f64>;
        loop {
            next = w.iter().zip(&step).map(|(wj, sj)| wj - t * sj).collect();
            if objective(rows, &next, prior, lambda) <= before || t < 1e-10 {
                break;
            }
            t /= 2.0;
        }
        let moved = step.iter().map(|s| (t * s).abs()).fold(0.0, f64::max);
        w = next;
        if moved < 1e-12 {
            break;
        }
    }
    w
}

/// `Σ log-loss(σ(offset + w·x), y) + λ/2 · |w − prior|²`, computed stably.
fn objective(rows: &[(f64, &[f64], bool)], w: &[f64], prior: &[f64], lambda: f64) -> f64 {
    let penalty: f64 = w.iter().zip(prior).map(|(a, b)| (a - b).powi(2)).sum();
    let loss: f64 = rows
        .iter()
        .map(|(offset, x, won)| {
            // −log σ(±z) = softplus(∓z)
            let z = offset + dot(w, x);
            let z = if *won { -z } else { z };
            z.max(0.0) + (-z.abs()).exp().ln_1p()
        })
        .sum();
    loss + lambda / 2.0 * penalty
}

/// Solves `a · x = b` by Gaussian elimination with partial pivoting.
fn solve(mut a: Vec<Vec<f64>>, mut b: Vec<f64>) -> Option<Vec<f64>> {
    let n = b.len();
    for col in 0..n {
        let pivot = (col..n).max_by(|&i, &j| a[i][col].abs().total_cmp(&a[j][col].abs()))?;
        if a[pivot][col].abs() < 1e-300 {
            return None;
        }
        a.swap(col, pivot);
        b.swap(col, pivot);
        for row in col + 1..n {
            let f = a[row][col] / a[col][col];
            for c in col..n {
                a[row][c] -= f * a[col][c];
            }
            b[row] -= f * b[col];
        }
    }
    let mut x = vec![0.0; n];
    for row in (0..n).rev() {
        let s: f64 = (row + 1..n).map(|c| a[row][c] * x[c]).sum();
        x[row] = (b[row] - s) / a[row][row];
    }
    Some(x)
}
