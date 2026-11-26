//! Ticket #37 (spec #33): the Signal Lab's combiner and forward selection as
//! pure functions, on hand-checkable inputs and fixed seeds.
use kallbet_ai::{
    adapters::lab::combiner::{self, Combiner, Inputs, LAMBDA},
    adapters::lab::selection,
    ports::{RandomSource, SeededRng},
};

fn anchor(market: Option<f64>, rating: Option<f64>) -> Option<f64> {
    match (market, rating) {
        (Some(m), Some(r)) => Some(combiner::sigmoid(
            0.8 * combiner::logit(m) + 0.2 * combiner::logit(r),
        )),
        (Some(p), None) | (None, Some(p)) => Some(p),
        (None, None) => None,
    }
}

#[test]
fn without_training_matches_the_combiner_is_the_anchor() {
    let fitted = Combiner::fit(&[], &[], LAMBDA);
    for (m, r) in [
        (Some(0.7), Some(0.4)),
        (Some(0.31), None),
        (None, Some(0.82)),
        (None, None),
    ] {
        let p = fitted.predict(&Inputs::base(m, r));
        match (p, anchor(m, r)) {
            (Some(p), Some(a)) => assert!((p - a).abs() < 1e-9, "{m:?} {r:?}: {p} vs {a}"),
            (None, None) => {}
            other => panic!("{other:?}"),
        }
    }
    // An extra feature starts at weight zero: it moves nothing until data says so.
    let with_extra = Combiner::fit(&["x".into()], &[], LAMBDA);
    let mut inputs = Inputs::base(Some(0.7), Some(0.4));
    inputs.extras = vec![Some(3.0)];
    let p = with_extra.predict(&inputs).unwrap();
    assert!((p - anchor(Some(0.7), Some(0.4)).unwrap()).abs() < 1e-9);
}

/// Synthetic Matches whose outcomes follow `sigmoid(wm·xm + wr·xr + we·xe)`.
fn synthetic(n: usize, w: [f64; 3], seed: u64) -> Vec<(Inputs, bool)> {
    let mut rng = SeededRng::new(seed);
    (0..n)
        .map(|_| {
            let xm = (rng.next_f64() - 0.5) * 4.0;
            let xr = (rng.next_f64() - 0.5) * 4.0;
            let xe = (rng.next_f64() - 0.5) * 4.0;
            let p = combiner::sigmoid(w[0] * xm + w[1] * xr + w[2] * xe);
            let mut inputs = Inputs::base(Some(combiner::sigmoid(xm)), Some(combiner::sigmoid(xr)));
            inputs.extras = vec![Some(xe)];
            (inputs, rng.next_f64() < p)
        })
        .collect()
}

#[test]
fn the_fit_recovers_the_weights_that_generated_the_outcomes() {
    let rows = synthetic(20_000, [0.5, 0.9, -0.6], 37);
    let fitted = Combiner::fit(&["x".into()], &rows, LAMBDA);
    let w = &fitted.model(combiner::Pattern::Both).unwrap().weights;
    for (got, want) in w.iter().zip([0.5, 0.9, -0.6]) {
        assert!((got - want).abs() < 0.06, "{w:?}");
    }
    assert_eq!(
        fitted.model(combiner::Pattern::Both).unwrap().matches,
        20_000
    );
    // The same rows give the same weights, bit for bit.
    let again = Combiner::fit(&["x".into()], &rows, LAMBDA);
    assert_eq!(fitted, again);
}

#[test]
fn the_penalty_pulls_the_weights_towards_the_anchor() {
    let rows = synthetic(200, [0.5, 0.9, -0.6], 5);
    let weak = Combiner::fit(&["x".into()], &rows, 1e-6);
    let strong = Combiner::fit(&["x".into()], &rows, 1e9);
    let w = |c: &Combiner| c.model(combiner::Pattern::Both).unwrap().weights.clone();
    let prior = [0.8, 0.2, 0.0];
    for (s, p) in w(&strong).iter().zip(prior) {
        assert!((s - p).abs() < 1e-4, "{:?}", w(&strong));
    }
    let dist = |v: Vec<f64>| -> f64 { v.iter().zip(prior).map(|(a, b)| (a - b).powi(2)).sum() };
    assert!(dist(w(&weak)) > dist(w(&Combiner::fit(&["x".into()], &rows, LAMBDA))));
}

/// Ticket #48: on the research archive the rating-only fit flipped between
/// -8.98 and 33.41 for ever (an undamped Newton step overshoots when the rating
/// is near-certain and often wrong), and the learned weight sent log-loss to 3.
/// Here: ten Matches with a rating logit of +/-8, six won by the side it
/// favours. The penalised minimum, prior 1 and LAMBDA 10, is 0.1125 (found by
/// golden-section search on the objective; at the prior the objective is far
/// higher).
#[test]
fn a_near_certain_and_often_wrong_rating_is_fitted_to_its_minimum_not_a_cycle() {
    let favourite = combiner::sigmoid(8.0);
    let rows: Vec<(Inputs, bool)> = [(true, 3), (false, 2)]
        .into_iter()
        .flat_map(|(won, n)| {
            std::iter::repeat_n((Inputs::base(None, Some(favourite)), won), n).chain(
                std::iter::repeat_n((Inputs::base(None, Some(1.0 - favourite)), !won), n),
            )
        })
        .collect();
    let fit = Combiner::fit(&[], &rows, LAMBDA);
    let w = fit.model(combiner::Pattern::RatingOnly).unwrap().weights[0];
    assert!((w - 0.1125).abs() < 1e-3, "rating weight {w}");
}

#[test]
fn each_availability_of_the_inputs_has_its_own_weights() {
    // Rating-only Matches where the rating is overconfident by half.
    let mut rng = SeededRng::new(11);
    let rows: Vec<(Inputs, bool)> = (0..5000)
        .map(|_| {
            let xr = (rng.next_f64() - 0.5) * 6.0;
            let won = rng.next_f64() < combiner::sigmoid(0.5 * xr);
            (Inputs::base(None, Some(combiner::sigmoid(xr))), won)
        })
        .collect();
    let fitted = Combiner::fit(&[], &rows, LAMBDA);
    let rating = fitted.model(combiner::Pattern::RatingOnly).unwrap();
    assert!(
        (rating.weights[0] - 0.5).abs() < 0.08,
        "{:?}",
        rating.weights
    );
    // The Matches with a market line had no training Match: still the anchor.
    let both = fitted.model(combiner::Pattern::Both).unwrap();
    assert_eq!((both.matches, both.weights.clone()), (0, vec![0.8, 0.2]));
    let parameters = fitted.parameters();
    assert_eq!(parameters["rating"]["matches"], 5000);
    assert_eq!(parameters["market+rating"]["matches"], 0);
}

#[test]
fn forward_selection_adds_the_best_improvement_first_and_stops_when_none_helps() {
    // Log-loss of a combination: a fixed gain per member, with "b" and "c"
    // redundant (together they add only b's gain).
    let score = |set: &[String]| -> f64 {
        let has = |n: &str| set.iter().any(|s| s == n);
        let mut ll = 0.70;
        if has("a") {
            ll -= 0.02;
        }
        if has("b") || has("c") {
            ll -= 0.03;
        }
        if has("d") {
            ll += 0.01;
        }
        ll
    };
    let names: Vec<String> = ["a", "b", "c", "d"].map(String::from).to_vec();
    let steps = selection::forward(&names, 0.70, score);
    let entered: Vec<_> = steps.iter().map(|s| s.entered.as_str()).collect();
    // b and c tie: the first in registry order enters; c then adds nothing.
    assert_eq!(entered, ["b", "a"]);
    assert!((steps[0].added - -0.03).abs() < 1e-12);
    assert!((steps[1].added - -0.02).abs() < 1e-12);
    assert!((steps[1].log_loss - 0.65).abs() < 1e-12);
    // Deterministic: the same inputs give the same steps.
    assert_eq!(steps, selection::forward(&names, 0.70, score));
    // Nothing improves the start: nothing enters.
    assert!(selection::forward(&names, 0.60, score).is_empty());
}
