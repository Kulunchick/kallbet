//! Tests for ticket #59 (`stacker_no_map_elo`: ablation of `stacker` without
//! `per_map_rating::map_signal` so `stacker` vs `stacker_no_map_elo` isolates
//! the map-level Elo input). Saved fixtures only; no network.
use chrono::{TimeZone, Utc};
use kallbet_ai::{
    Format, Instant,
    adapters::{
        backtest::ReplayArchive,
        lab::{
            Lab,
            signals::{self, stacker, stacker_no_map_elo},
            stats::Verdict,
        },
    },
};
use std::collections::BTreeSet;
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

/// `events` Events of `per_event` Matches, one day apart. Team A wins when
/// `per_map` favours it (`map_skill`) or when `hltv_gap` favours it (`rank_skill`).
fn rows(
    seed: u64,
    events: i64,
    per_event: i64,
    map_skill: f64,
    rank_skill: f64,
) -> Vec<stacker::Row> {
    let mut rng = Lcg(seed);
    let start = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
    let mut out = vec![];
    for e in 0..events {
        for i in 0..per_event {
            let cutoff: Instant = start + chrono::Duration::days(e) + chrono::Duration::minutes(i);
            let gap = rng.next() * 4.0 - 2.0;
            let per_map = 0.15 + rng.next() * 0.70;
            let won = if rng.next() < map_skill {
                per_map > 0.5
            } else if rng.next() < rank_skill {
                gap > 0.0
            } else {
                rng.next() < 0.5
            };
            out.push(stacker::Row {
                raw: stacker::Raw {
                    market: Some(0.45 + rng.next() * 0.1),
                    per_map: Some(per_map),
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
fn ablation_ignores_per_map_input_and_keeps_identical_accuracy_lambda_selection() {
    let with_maps = rows(19, 9, 40, 0.85, 0.5);
    let without_maps: Vec<stacker::Row> = with_maps
        .iter()
        .cloned()
        .map(|mut r| {
            r.raw.per_map = None;
            r
        })
        .collect();

    let fitted_from_with = stacker_no_map_elo::fit(&with_maps);
    let fitted_from_without = stacker_no_map_elo::fit(&without_maps);
    assert_eq!(
        fitted_from_with.parameters(),
        fitted_from_without.parameters(),
        "per_map must not affect stacker_no_map_elo fitting"
    );
    assert_eq!(
        fitted_from_with.chosen_by,
        stacker::ChosenBy::ValidationAccuracy
    );
    assert_eq!(fitted_from_with.validation_events, 3);

    // Predictions ignore `raw.per_map` completely.
    let probe_with = stacker::Raw {
        market: Some(0.5),
        per_map: Some(0.95),
        hltv_gap: Some(0.8),
        ..Default::default()
    };
    let probe_without = stacker::Raw {
        per_map: None,
        ..probe_with
    };
    assert_eq!(
        fitted_from_with.predict(&probe_with, Format::Bo3),
        fitted_from_with.predict(&probe_without, Format::Bo3)
    );

    // Weights omit `per_map_rating` in `stacker_no_map_elo` while keeping the other extras.
    let params = fitted_from_with.parameters();
    let market_weights = params["weights"]["market"].as_object().unwrap();
    assert!(market_weights.contains_key("own_rating"));
    assert!(market_weights.contains_key("vrs_rank_gap"));
    assert!(market_weights.contains_key("hltv_rank_gap"));
    assert!(!market_weights.contains_key("per_map_rating"));

    // Contrast: `stacker` includes `per_map_rating` and shifts when `per_map` is informative.
    let full = stacker::Stacked::fit(&with_maps);
    let full_params = full.parameters();
    assert!(
        full_params["weights"]["market"]
            .as_object()
            .unwrap()
            .contains_key("per_map_rating")
    );
    assert_ne!(
        full.predict(&probe_with, Format::Bo3),
        full.predict(&probe_without, Format::Bo3)
    );

    // Too few Events -> Default lambda, identical to `stacker`.
    let few = stacker_no_map_elo::fit(&rows(3, 1, 40, 0.0, 0.8));
    assert_eq!(few.chosen_by, stacker::ChosenBy::Default);
    assert_eq!(few.lambda, kallbet_ai::adapters::lab::combiner::LAMBDA);
}

#[tokio::test]
async fn stacker_no_map_elo_is_registered_marked_and_reported_on_the_same_matches_as_stacker() {
    let no_map = signals::by_name("stacker_no_map_elo").expect("stacker_no_map_elo is registered");
    let full = signals::by_name("stacker").expect("stacker is registered");
    assert!(no_map.added_after_list_fixed());
    let description = no_map.description();
    assert!(
        description.contains("stacker")
            && description.contains("stacker_no_map_elo")
            && description.contains("map-level Elo"),
        "{description}"
    );

    let archive = ReplayArchive::load(&fixtures().join("backtest")).unwrap();
    let mut lab = Lab::new(archive).unwrap();
    lab.signals = vec![full, no_map];
    let outcome = lab.run().await.unwrap();
    assert_eq!(outcome.report.candidates.len(), 2);

    let full_row = outcome
        .report
        .candidates
        .iter()
        .find(|c| c.name == "stacker")
        .unwrap();
    let no_map_row = outcome
        .report
        .candidates
        .iter()
        .find(|c| c.name == "stacker_no_map_elo")
        .unwrap();

    assert!(no_map_row.added_after_list_fixed);
    assert_eq!(no_map_row.computed_on, full_row.computed_on);
    assert_eq!(no_map_row.paired, full_row.paired);
    assert!(
        no_map_row.computed_on > 0 && no_map_row.paired > 0,
        "computed {} paired {}",
        no_map_row.computed_on,
        no_map_row.paired
    );
    assert!(
        no_map_row.difference.is_some()
            && no_map_row.interval.is_some()
            && no_map_row.accuracy_interval.is_some()
    );
    assert!(matches!(
        no_map_row.verdict,
        Verdict::Promote | Verdict::NoEvidence | Verdict::Harmful
    ));

    let predicted_ids = |name: &str| -> BTreeSet<i64> {
        outcome
            .predictions
            .iter()
            .filter(|p| p.signal == name && p.variant.is_some())
            .map(|p| p.match_id)
            .collect()
    };
    assert_eq!(
        predicted_ids("stacker_no_map_elo"),
        predicted_ids("stacker"),
        "both stacker variants must cover the exact same Matches"
    );

    let md = outcome.report.markdown();
    assert!(
        md.contains("stacker_no_map_elo") && md.contains("(added after list fixed)"),
        "{md}"
    );

    let json = serde_json::to_value(&outcome.report).unwrap();
    let per_event = json["candidates"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "stacker_no_map_elo")
        .unwrap()["per_event"]
        .as_array()
        .unwrap();
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
}

#[tokio::test]
async fn stacker_and_stacker_no_map_elo_differ_when_maps_exist_and_exclude_holdout() {
    let dir = tempfile::tempdir().unwrap();
    for entry in std::fs::read_dir(fixtures().join("backtest")).unwrap() {
        let path = entry.unwrap().path();
        if path.is_file() {
            std::fs::copy(&path, dir.path().join(path.file_name().unwrap())).unwrap();
        }
    }
    // Add real Cologne Playoffs post-match Liquipedia maps so later Events have `per_map_rating` history.
    std::fs::copy(
        fixtures().join("research/liquipedia-cologne-playoffs-postmatch.json"),
        dir.path().join("maps-20710.json"),
    )
    .unwrap();

    let archive = ReplayArchive::load(dir.path()).unwrap();
    let mut lab = Lab::new(archive).unwrap();
    lab.signals = vec![
        signals::by_name("stacker").unwrap(),
        signals::by_name("stacker_no_map_elo").unwrap(),
    ];
    let outcome = lab.run().await.unwrap();
    let full_row = outcome
        .report
        .candidates
        .iter()
        .find(|c| c.name == "stacker")
        .unwrap();
    let no_map_row = outcome
        .report
        .candidates
        .iter()
        .find(|c| c.name == "stacker_no_map_elo")
        .unwrap();
    assert_eq!(no_map_row.computed_on, full_row.computed_on);
    assert_eq!(no_map_row.paired, full_row.paired);

    // Put the later half of Events in Holdout: no fit may read past Holdout start.
    let mut begins: Vec<Instant> = outcome.report.events.iter().map(|e| e.begins).collect();
    begins.sort();
    let from = begins[begins.len() / 2];
    let archive = ReplayArchive::load(dir.path()).unwrap();
    let mut lab = Lab::new(archive).unwrap();
    lab.holdout.from = from;
    lab.holdout.training_stages = vec![];
    lab.signals = vec![signals::by_name("stacker_no_map_elo").unwrap()];
    let outcome = lab.run().await.unwrap();
    let json = serde_json::to_value(&outcome.report).unwrap();
    for e in json["candidates"][0]["per_event"].as_array().unwrap() {
        if let Some(through) = e["parameters"]["fitted_through"].as_str() {
            assert!(through.parse::<Instant>().unwrap() < from, "{e}");
        }
    }
}
