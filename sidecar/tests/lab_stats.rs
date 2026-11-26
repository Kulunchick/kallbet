//! Ticket #34: the Signal Lab's statistics, as pure functions on hand-checkable
//! inputs with fixed seeds (the protocol of spec #33).
use kallbet_ai::{
    Format,
    adapters::lab::{
        Protocol,
        signals::calibration::fit_scale,
        stats::{self, Interval, Verdict},
    },
};

fn close(a: f64, b: f64, eps: f64) -> bool {
    (a - b).abs() <= eps
}

#[test]
fn per_match_scores_are_log_loss_brier_and_a_pick_of_team_a_at_even_odds() {
    assert!(close(stats::log_loss(0.8, true), -(0.8f64).ln(), 1e-12));
    assert!(close(stats::log_loss(0.8, false), -(0.2f64).ln(), 1e-12));
    // Clipped at 1e-6, as the backtest's report.
    assert!(close(stats::log_loss(1.0, false), -(1e-6f64).ln(), 1e-9));
    assert!(close(stats::brier(0.8, true), 0.04, 1e-12));
    assert!(close(stats::brier(0.8, false), 0.64, 1e-12));
    assert!(stats::correct(0.5, true));
    assert!(!stats::correct(0.5, false));
    assert!(stats::correct(0.3, false));
}

#[test]
fn the_event_bootstrap_resamples_whole_events_and_pools_their_matches() {
    // Two Events of one Match each, -1 and +1: a resample draws each Event with
    // probability 1/2, so the pooled mean is -1, 0 or +1 with 1/4, 1/2, 1/4.
    let b = stats::event_bootstrap(&[(1, -1.0), (2, 1.0)], 4000, 0.90, 7).unwrap();
    assert_eq!(b.mean, 0.0);
    assert_eq!(
        b.interval,
        Interval {
            low: -1.0,
            high: 1.0
        }
    );
    assert!(close(b.std_error, 0.5f64.sqrt(), 0.03), "{}", b.std_error);

    // The Matches of one Event move together: three Matches at -0.1 in Event 1
    // and one at +0.1 in Event 2 pool to -0.05, and a resample holding only
    // Event 1 gives -0.1, only Event 2 +0.1.
    let diffs = [(1, -0.1), (1, -0.1), (1, -0.1), (2, 0.1)];
    let b = stats::event_bootstrap(&diffs, 4000, 0.90, 7).unwrap();
    assert!(close(b.mean, -0.05, 1e-12));
    assert!(close(b.interval.low, -0.1, 1e-12));
    assert!(close(b.interval.high, 0.1, 1e-12));

    // One Event, or every Event alike: no spread at all.
    let one = stats::event_bootstrap(&[(1, 0.2), (1, 0.4)], 500, 0.90, 1).unwrap();
    assert!(close(one.interval.low, 0.3, 1e-12) && close(one.interval.high, 0.3, 1e-12));
    assert_eq!(one.std_error, 0.0);
    assert!(stats::event_bootstrap(&[], 500, 0.90, 1).is_none());
}

#[test]
fn the_same_seed_gives_the_same_interval_and_another_seed_may_not() {
    let diffs: Vec<(i64, f64)> = (0..12)
        .flat_map(|e| (0..5).map(move |m| (e, ((e * 7 + m * 3) % 11) as f64 / 10.0 - 0.5)))
        .collect();
    let a = stats::event_bootstrap(&diffs, 2000, 0.90, 34).unwrap();
    let b = stats::event_bootstrap(&diffs, 2000, 0.90, 34).unwrap();
    assert_eq!(a, b);
    let c = stats::event_bootstrap(&diffs, 2000, 0.90, 35).unwrap();
    assert_ne!(a.interval, c.interval);
    assert!(a.interval.low < a.mean && a.mean < a.interval.high);
}

#[test]
fn the_smallest_detectable_effect_is_the_one_a_90_interval_excludes_with_80_power() {
    // (z_0.95 + z_0.80) * SE = (1.6449 + 0.8416) * SE.
    assert!(close(stats::detectable_effect(0.01), 0.024865, 1e-5));
    assert_eq!(stats::detectable_effect(0.0), 0.0);
}

#[test]
fn the_verdict_is_the_written_rule() {
    let p = Protocol::default();
    let floor = p.min_matches;
    let i = |low, high| Interval { low, high };
    // Improvement (a lower log-loss) whose interval excludes zero, market subset not worse.
    assert_eq!(
        stats::verdict(floor, i(-0.03, -0.001), None, Some(-0.01), &p).0,
        Verdict::Promote
    );
    // No Match with a market line: nothing there is worsened.
    assert_eq!(
        stats::verdict(floor, i(-0.03, -0.001), None, None, &p).0,
        Verdict::Promote
    );
    // The market subset is worse: not promoted, and the reason says why.
    let (v, why) = stats::verdict(floor, i(-0.03, -0.001), None, Some(0.002), &p);
    assert_eq!(v, Verdict::NoEvidence);
    assert!(why.contains("market"), "{why}");
    // Worse, interval excluding zero.
    assert_eq!(
        stats::verdict(floor, i(0.001, 0.02), None, None, &p).0,
        Verdict::Harmful
    );
    // Straddles zero.
    assert_eq!(
        stats::verdict(floor, i(-0.02, 0.01), None, None, &p).0,
        Verdict::NoEvidence
    );
    // Below the floor, whatever the interval says.
    let (v, why) = stats::verdict(floor - 1, i(-0.5, -0.4), None, None, &p);
    assert_eq!(v, Verdict::NoEvidence);
    assert!(why.contains(&floor.to_string()), "{why}");
    assert_eq!(
        stats::verdict(floor - 1, i(0.4, 0.5), None, None, &p).0,
        Verdict::NoEvidence
    );
}

#[test]
fn verdicts_are_written_as_the_spec_names_them() {
    let names: Vec<String> = [
        Verdict::Promote,
        Verdict::Harmful,
        Verdict::NoEvidence,
        Verdict::NotTestable,
    ]
    .iter()
    .map(|v| {
        serde_json::to_value(v)
            .unwrap()
            .as_str()
            .unwrap()
            .to_owned()
    })
    .collect();
    assert_eq!(names, ["promote", "harmful", "no evidence", "not testable"]);
}

#[test]
fn the_calibration_scale_is_the_maximum_likelihood_one() {
    // 100 Bo1 Matches whose favourite has a map probability of 0.8 and wins 60:
    // the likelihood is highest where sigmoid(s * logit(0.8)) = 0.6.
    let x = (0.8f64 / 0.2).ln();
    let data: Vec<(f64, Format, bool)> = (0..100).map(|i| (x, Format::Bo1, i < 60)).collect();
    let s = fit_scale(&data).unwrap();
    let expected = (0.6f64 / 0.4).ln() / x;
    assert!(close(s, expected, 1e-4), "{s} vs {expected}");
    // The underdog side written as team A gives the same scale.
    let mirrored: Vec<(f64, Format, bool)> = data.iter().map(|(x, f, w)| (-x, *f, !w)).collect();
    assert!(close(fit_scale(&mirrored).unwrap(), expected, 1e-4));
    assert!(fit_scale(&[]).is_none());
}

// Finding 17 (review of 6eea194): no bootstrap is made of zero resamples
// (it was taken as one), and the CLI refuses `--resamples 0`.
#[test]
fn zero_resamples_give_no_interval_and_are_refused() {
    let diffs = [(1, -0.1), (2, -0.2), (3, -0.05)];
    assert!(kallbet_ai::adapters::lab::stats::event_bootstrap(&diffs, 0, 0.9, 33).is_none());
    let args: Vec<String> = ["--resamples", "0"].iter().map(|s| s.to_string()).collect();
    assert!(kallbet_ai::adapters::lab::cli::parse(&args).is_err());
}

// signal-lab/3: KallBet scores only the picked winner. Better probabilities
// with an accuracy interval wholly below zero are not promoted; an accuracy
// interval that reaches zero does not stop a promotion; harmful is unchanged.
#[test]
fn promote_also_needs_the_accuracy_interval_not_wholly_below_zero() {
    let p = Protocol::default();
    let i = |low, high| Interval { low, high };
    let better = i(-0.03, -0.001);
    let (v, why) = stats::verdict(150, better, Some(i(-0.05, -0.01)), None, &p);
    assert_eq!(v, Verdict::NoEvidence);
    assert!(why.contains("better probabilities but fewer winners picked"), "{why}");
    assert_eq!(stats::verdict(150, better, Some(i(-0.05, 0.0)), None, &p).0, Verdict::Promote);
    assert_eq!(stats::verdict(150, better, Some(i(0.01, 0.05)), None, &p).0, Verdict::Promote);
    assert_eq!(
        stats::verdict(150, i(0.001, 0.02), Some(i(-0.05, -0.01)), None, &p).0,
        Verdict::Harmful
    );

    // The accuracy interval by hand: every row of one Event has the variant
    // right and the anchor wrong (+1), or the reverse (-1).
    let right = (0..3).map(|_| (1, true, 0.4, 0.6));
    assert_eq!(stats::accuracy_bootstrap(right, &p), Some(i(1.0, 1.0)));
    let wrong = (0..3).map(|_| (1, true, 0.6, 0.4));
    assert_eq!(stats::accuracy_bootstrap(wrong, &p), Some(i(-1.0, -1.0)));
    // Two Events, +1 and 0: each resample's mean is 0, 1/2 or 1.
    let mixed = [(1, true, 0.4, 0.6), (2, true, 0.6, 0.7)].into_iter();
    let a = stats::accuracy_bootstrap(mixed, &p).unwrap();
    assert!(a.low >= 0.0 && a.high <= 1.0 && a.low < a.high, "{a:?}");
}
