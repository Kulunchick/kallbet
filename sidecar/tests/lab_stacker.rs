//! Tests for ticket #58 (`stacker`: the accuracy-fitted combiner). The lab runs
//! use only the saved backtest archive in `sidecar/tests/fixtures/backtest/`;
//! the selection rule is tested as a pure function on fixed-seed rows. No network.
use chrono::{TimeZone, Utc};
use kallbet_ai::{
    Format, Instant,
    adapters::{
        backtest::ReplayArchive,
        lab::{
            Lab,
            signals::{self, stacker},
            stats::Verdict,
        },
    },
};
use std::path::Path;

fn fixtures() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

/// A fixed-seed generator (64-bit LCG), uniform in [0, 1).
struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> f64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 11) as f64 / (1u64 << 53) as f64
    }
}

/// `events` Events of `per_event` Matches, one day apart. Team A wins when the
/// HLTV rank gap says so with probability `rank_skill` and otherwise at random;
/// the market line knows nothing (0.5 ± noise).
fn rows(seed: u64, events: i64, per_event: i64, rank_skill: f64) -> Vec<stacker::Row> {
    let mut rng = Lcg(seed);
    let start = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
    let mut out = vec![];
    for e in 0..events {
        for i in 0..per_event {
            let cutoff: Instant = start + chrono::Duration::days(e) + chrono::Duration::minutes(i);
            let gap = rng.next() * 4.0 - 2.0;
            let follows = rng.next() < rank_skill;
            let won = if follows { gap > 0.0 } else { rng.next() < 0.5 };
            out.push(stacker::Row {
                raw: stacker::Raw {
                    market: Some(0.45 + rng.next() * 0.1),
                    hltv_gap: Some(gap),
                    ..Default::default()
                },
                format: Format::Bo3,
                event: e,
                cutoff,
                won,
            });
        }
    }
    out
}

#[test]
fn the_penalty_is_chosen_by_accuracy_on_the_latest_training_events() {
    // Ranks pick winners: the selection moves away from the market (small λ).
    let informative = stacker::Stacked::fit(&rows(7, 9, 40, 0.8));
    assert_eq!(informative.chosen_by, stacker::ChosenBy::ValidationAccuracy);
    assert_eq!(
        informative.validation_events, 3,
        "the latest third of 9 Events"
    );
    assert!(informative.lambda <= 10.0, "λ {}", informative.lambda);
    let best = informative
        .validation
        .iter()
        .map(|v| v.accuracy)
        .fold(f64::MIN, f64::max);
    let chosen = informative
        .validation
        .iter()
        .find(|v| v.lambda == informative.lambda)
        .unwrap();
    assert_eq!(
        chosen.accuracy, best,
        "the most accurate λ on validation wins"
    );
    let p = informative
        .predict(
            &stacker::Raw {
                market: Some(0.5),
                hltv_gap: Some(1.5),
                ..Default::default()
            },
            Format::Bo3,
        )
        .unwrap();
    assert!(p > 0.6, "a large positive gap favours team A: {p}");

    // Ranks are noise: nothing beats staying near the anchor, ties go to the larger λ.
    let noise = stacker::Stacked::fit(&rows(11, 9, 40, 0.0));
    assert_eq!(noise.chosen_by, stacker::ChosenBy::ValidationAccuracy);

    // Too few training Events to hold some out: the default penalty.
    let few = stacker::Stacked::fit(&rows(3, 1, 40, 0.8));
    assert_eq!(few.chosen_by, stacker::ChosenBy::Default);
    assert_eq!(few.lambda, kallbet_ai::adapters::lab::combiner::LAMBDA);

    // Same rows, same fit (pure, no hidden state).
    assert_eq!(
        stacker::Stacked::fit(&rows(7, 9, 40, 0.8)).parameters(),
        informative.parameters()
    );
}

#[test]
fn missing_inputs_read_as_no_shift_and_no_line_and_no_rating_gives_no_prediction() {
    let fitted = stacker::Stacked::fit(&rows(5, 6, 30, 0.8));
    // Only a market line: the market-only model, every extra read as zero.
    let alone = fitted.predict(
        &stacker::Raw {
            market: Some(0.7),
            ..Default::default()
        },
        Format::Bo3,
    );
    let with_zero_gap = fitted.predict(
        &stacker::Raw {
            market: Some(0.7),
            hltv_gap: Some(0.0),
            vrs_gap: Some(0.0),
            ..Default::default()
        },
        Format::Bo3,
    );
    assert_eq!(alone, with_zero_gap);
    // Ranks alone: neither base input, no prediction (the anchor has none either).
    assert_eq!(
        fitted.predict(
            &stacker::Raw {
                hltv_gap: Some(1.0),
                ..Default::default()
            },
            Format::Bo3
        ),
        None
    );
}

#[tokio::test]
async fn stacker_is_registered_marked_and_reported_with_per_event_parameters() {
    let signal = signals::by_name("stacker").expect("stacker is registered");
    assert!(signal.added_after_list_fixed());
    let description = signal.description();
    assert!(
        description.contains("accuracy") && description.contains("log-loss"),
        "{description}"
    );

    let archive = ReplayArchive::load(&fixtures().join("backtest")).unwrap();
    let mut lab = Lab::new(archive).unwrap();
    lab.signals = vec![signal];
    let outcome = lab.run().await.unwrap();
    let row = &outcome.report.candidates[0];
    assert_eq!(row.name, "stacker");
    assert!(row.added_after_list_fixed);
    assert!(
        row.computed_on > 0 && row.paired > 0,
        "computed {} paired {}",
        row.computed_on,
        row.paired
    );
    assert!(row.difference.is_some() && row.interval.is_some() && row.accuracy_interval.is_some());
    assert!(matches!(
        row.verdict,
        Verdict::Promote | Verdict::NoEvidence | Verdict::Harmful
    ));
    let md = outcome.report.markdown();
    assert!(
        md.contains("stacker") && md.contains("(added after list fixed)"),
        "{md}"
    );

    // Every evaluated Event has its fit in report.json, and that fit read only
    // Matches of Events that ended before it began.
    let json = serde_json::to_value(&outcome.report).unwrap();
    let per_event = json["candidates"][0]["per_event"].as_array().unwrap();
    assert!(!per_event.is_empty());
    for e in per_event {
        let p = &e["parameters"];
        assert!(p["lambda"].is_number() && p["weights"].is_object(), "{p}");
        assert_eq!(p["training_matches"], e["training_matches"], "{p}");
        let info = outcome
            .report
            .events
            .iter()
            .find(|i| i.id == e["event_id"].as_i64().unwrap())
            .unwrap();
        if let Some(through) = p["fitted_through"].as_str() {
            let through: Instant = through.parse().unwrap();
            assert!(
                through < info.begins,
                "{} fitted through {through}",
                info.title
            );
        } else {
            assert_eq!(p["training_matches"], 0, "{p}");
        }
    }

    // With the later half of the archive in the Holdout, no fit reads past its start.
    let mut begins: Vec<Instant> = outcome.report.events.iter().map(|e| e.begins).collect();
    begins.sort();
    let from = begins[begins.len() / 2];
    let archive = ReplayArchive::load(&fixtures().join("backtest")).unwrap();
    let mut lab = Lab::new(archive).unwrap();
    lab.holdout.from = from;
    lab.holdout.training_stages = vec![];
    lab.signals = vec![signals::by_name("stacker").unwrap()];
    let outcome = lab.run().await.unwrap();
    let json = serde_json::to_value(&outcome.report).unwrap();
    for e in json["candidates"][0]["per_event"].as_array().unwrap() {
        if let Some(through) = e["parameters"]["fitted_through"].as_str() {
            assert!(through.parse::<Instant>().unwrap() < from, "{e}");
        }
    }
}
