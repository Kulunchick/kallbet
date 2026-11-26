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
        "First place",
        "Expected place",
        "Verdict",
        "points behind",
    ] {
        assert!(report.contains(label), "{label}");
    }
}
