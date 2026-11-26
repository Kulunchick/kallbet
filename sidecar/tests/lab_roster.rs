use kallbet_ai::adapters::{
    backtest::ReplayArchive,
    lab::{
        Lab,
        signals::roster::{RosterHistory, RosterSignal},
    },
};
use std::path::Path;

fn archive() -> ReplayArchive {
    ReplayArchive::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/backtest"))
        .unwrap()
}

#[tokio::test]
async fn roster_signals_report_coverage_and_paired_metrics() {
    let lab = Lab::new(archive()).unwrap();
    let outcome = lab.run().await.unwrap();
    for signal in RosterSignal::ALL {
        let name = signal.name();
        let report = outcome
            .report
            .candidates
            .iter()
            .find(|c| c.name == name)
            .unwrap();
        let rows = outcome.predictions.iter().filter(|r| r.signal == name);
        assert_eq!(
            report.computed_on,
            rows.clone().filter(|r| r.variant.is_some()).count()
        );
        assert!(
            rows.clone()
                .any(|r| r.baseline.is_some() && r.variant.is_none())
        );
        assert!(report.paired > 0, "{name}");
        assert!(
            report.difference.is_some() && report.interval.is_some(),
            "{name}"
        );
        assert!(!report.reason.is_empty(), "{name}");
    }
}

#[test]
fn saved_rosters_distinguish_active_stand_ins_from_regular_fives() {
    let archive = archive();
    let history = RosterHistory::new(&archive).unwrap();
    for (id, expected) in [(1513148, [0.0, 1.0]), (1513120, [0.0, 0.0])] {
        let game = archive
            .stages
            .iter()
            .flat_map(|s| &s.matches)
            .find(|m| m.game.id == id)
            .unwrap();
        let features = history.at(&game.game, game.cutoff()).unwrap().unwrap();
        assert_eq!(
            features
                .at(game.cutoff())
                .unwrap()
                .values(RosterSignal::StandIn),
            Some(expected),
            "Match {id}"
        );
    }
}

#[test]
fn the_revision_at_the_cutoff_is_excluded() {
    let archive = archive();
    let m = archive
        .stages
        .iter()
        .flat_map(|s| &s.matches)
        .find(|m| m.game.id == 1513148)
        .unwrap();
    let title = &archive.event_titles[&m.game.event_id];
    let revision = archive.event_pages[title][0].timestamp;
    let history = RosterHistory::new(&archive).unwrap();
    assert!(history.at(&m.game, revision).unwrap().is_none());
    assert!(
        history
            .at(&m.game, revision - chrono::Duration::seconds(1))
            .unwrap()
            .is_none()
    );
    assert!(
        history
            .at(&m.game, revision + chrono::Duration::seconds(1))
            .unwrap()
            .is_some()
    );
}

#[tokio::test]
async fn a_saved_revision_with_an_unparsable_lineup_leaves_the_match_uncovered() {
    let mut archive = archive();
    let m = archive
        .stages
        .iter()
        .flat_map(|s| &s.matches)
        .find(|m| m.game.id == 1513148)
        .unwrap();
    let title = &archive.event_titles[&m.game.event_id];
    for revision in archive.event_pages.get_mut(title).unwrap() {
        let mut body: serde_json::Value = serde_json::from_str(&revision.body).unwrap();
        body["query"]["pages"][0]["revisions"][0]["slots"]["main"]["content"] =
            "==Participants==\nNo lineup template in this revision.".into();
        revision.body = body.to_string();
    }
    let outcome = Lab::new(archive).unwrap().run().await.unwrap();
    for name in RosterSignal::ALL.map(RosterSignal::name) {
        let row = outcome
            .predictions
            .iter()
            .find(|row| row.match_id == 1513148 && row.signal == name)
            .unwrap();
        assert!(row.baseline.is_some());
        assert_eq!(row.variant, None, "{name}");
    }
}
