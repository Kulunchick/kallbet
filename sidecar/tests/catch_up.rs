use kallbet_ai::adapters::{
    backtest::ReplayArchive,
    lab::{Lab, leagues},
};
use std::path::Path;

#[tokio::test]
async fn saved_archive_reports_paired_places_by_model_size_and_trigger() {
    let archive =
        ReplayArchive::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/backtest"))
            .unwrap();
    let lab = Lab::new(archive).unwrap();
    let report = leagues::report(&lab, 42, 64).await.unwrap();
    assert_eq!(report, leagues::report(&lab, 42, 64).await.unwrap());
    for label in [
        "Favourite follower",
        "Market only",
        "| 4 |",
        "| 12 |",
        "| 4 | plain |",
        "| 4 | catch-up |",
        "| 12 | plain |",
        "| 12 | catch-up |",
        "First place probability",
        "Expected place",
        "Verdict",
        "points behind",
        "Matches left",
        "Difference in first-place probability",
        "Difference in expected place",
        "[0.",
        "11+",
    ] {
        assert!(report.contains(label), "{label}");
    }
}

// Finding 5 (review of 6eea194): the catch-up trials draw training-side Events
// only. A completed Event after the boundary changes nothing in the report.
#[tokio::test]
async fn a_holdout_event_changes_no_catch_up_figure() {
    let archive = || {
        ReplayArchive::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/backtest"))
            .unwrap()
    };
    let plain = leagues::report(&Lab::new(archive()).unwrap(), 42, 64).await.unwrap();
    let mut extended = archive();
    let shift = chrono::Duration::days(200);
    let later: Vec<_> = extended
        .stages
        .iter()
        .filter(|s| s.event.contains("Cologne"))
        .cloned()
        .map(|mut s| {
            s.id += 900_000;
            s.event_id += 900_000;
            s.event = "A later Major".into();
            s.cutoff += shift;
            for m in &mut s.matches {
                m.game.id += 900_000_000;
                m.actual_start += shift;
                m.finished_at += shift;
                m.reported_finish = m.reported_finish.map(|t| t + shift);
                m.game.scheduled_start = m.game.scheduled_start.map(|t| t + shift);
            }
            s
        })
        .collect();
    extended.stages.extend(later);
    let lab = Lab::new(extended).unwrap();
    assert!(!lab.holdout_stages().is_empty());
    let with_holdout = leagues::report(&lab, 42, 64).await.unwrap();
    assert_eq!(plain, with_holdout);
}
