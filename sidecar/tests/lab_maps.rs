use chrono::Duration;
use kallbet_ai::{
    Format,
    adapters::{
        backtest::{ReplayArchive, archive::read_file},
        hltv_research::parse::parse_match_page,
        lab::{Lab, signals, signals::per_map_rating::map_series_probability, stats::Verdict},
    },
    vrs::series_probability,
};
use std::path::Path;

#[test]
fn map_probabilities_agree_with_the_existing_series_conversion() {
    for (format, count, expected) in [
        (Format::Bo1, 1, 0.6),
        (Format::Bo3, 3, 0.648),
        (Format::Bo5, 5, 0.68256),
    ] {
        let actual = map_series_probability(&vec![0.6; count], format).unwrap();
        assert!((actual - expected).abs() < 1e-12);
        assert!((actual - series_probability(0.6, format)).abs() < 1e-12);
    }
    // Exactly two successes, or three, with one certain win and one certain loss.
    assert_eq!(
        map_series_probability(&[1.0, 0.0, 0.75], Format::Bo3),
        Some(0.75)
    );
}

fn archive_with_maps() -> tempfile::TempDir {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let dir = tempfile::tempdir().unwrap();
    for name in ["panda-20710.json.gz", "standings_global_2026_06_01.md.gz"] {
        std::fs::copy(root.join("backtest").join(name), dir.path().join(name)).unwrap();
    }
    std::fs::copy(
        root.join("research/liquipedia-cologne-playoffs-postmatch.json"),
        dir.path().join("maps-20710.json"),
    )
    .unwrap();
    dir
}

#[tokio::test]
async fn map_ratings_report_coverage_and_respect_the_strict_finish_boundary() {
    let dir = archive_with_maps();
    let path = dir.path();
    let run = |cutoff_shift| async move {
        let mut archive = ReplayArchive::load(path).unwrap();
        let stage = archive.stages.iter_mut().find(|s| s.id == 20710).unwrap();
        // The final has both finalists' histories if the second semifinal has ended.
        let semifinal_end = stage
            .matches
            .iter()
            .find(|m| m.game.id == 1513160)
            .unwrap()
            .finished_at;
        let final_game = stage
            .matches
            .iter_mut()
            .find(|m| m.game.id == 1513171)
            .unwrap();
        if let Some(shift) = cutoff_shift {
            let start = semifinal_end + Duration::minutes(15) + shift;
            final_game.actual_start = start;
            final_game.game.scheduled_start = Some(start);
            assert_eq!(final_game.cutoff(), semifinal_end + shift);
        }
        let mut lab = Lab::new(archive).unwrap();
        lab.signals = ["per_map_rating", "bo1_veto_sim"]
            .map(|name| signals::by_name(name).unwrap())
            .into();
        lab.run().await.unwrap()
    };
    let normal = run(None).await;
    let maps = &normal.report.candidates[0];
    assert!(maps.paired > 0);
    assert!(maps.difference.is_some());
    assert!(maps.interval.is_some());
    let veto = &normal.report.candidates[1];
    assert_eq!(veto.verdict, Verdict::NotTestable);
    assert!(veto.reason.contains("HLTV"));
    fn final_pick(outcome: &kallbet_ai::adapters::lab::LabOutcome) -> Option<f64> {
        outcome
            .predictions
            .iter()
            .find(|p| p.signal == "per_map_rating" && p.match_id == 1513171)
            .unwrap()
            .variant
    }
    let before = run(Some(Duration::seconds(-1))).await;
    let equal = run(Some(Duration::zero())).await;
    let after = run(Some(Duration::seconds(1))).await;
    assert_eq!(final_pick(&before), final_pick(&equal));
    assert!(final_pick(&after).is_some());
    assert_ne!(final_pick(&equal), final_pick(&after));
}

#[tokio::test]
async fn bo1_veto_uses_an_earlier_saved_veto_and_never_its_own() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let dir = tempfile::tempdir().unwrap();
    for name in ["panda-20708.json.gz", "standings_global_2026_06_01.md.gz"] {
        std::fs::copy(root.join("backtest").join(name), dir.path().join(name)).unwrap();
    }
    let html =
        read_file(&root.join("research/hltv-match-2394774-heroic-vs-sharks.html.gz")).unwrap();
    let record = parse_match_page(
        2394774,
        "https://www.hltv.org/matches/2394774/heroic-vs-sharks-iem-cologne-major-2026-stage-1",
        "2026-11-26T01:15:00Z".parse().unwrap(),
        std::str::from_utf8(&html).unwrap(),
    )
    .unwrap();
    let hltv_dir = dir.path().join("hltv");
    let matches_dir = hltv_dir.join("matches");
    std::fs::create_dir_all(&matches_dir).unwrap();
    // Sidecar HLTV collector metadata alongside match records must be ignored.
    std::fs::write(hltv_dir.join("manifest.json"), br#"{"version":1}"#).unwrap();
    std::fs::write(
        matches_dir.join("hltv-match-2394774.json"),
        serde_json::to_vec(&record).unwrap(),
    )
    .unwrap();
    let path = dir.path();
    let run = |shift, prior_is_holdout| async move {
        let mut archive = ReplayArchive::load(path).unwrap();
        let stage = archive.stages.iter_mut().find(|s| s.id == 20708).unwrap();
        let finish = stage
            .matches
            .iter()
            .find(|m| m.game.id == 1494593)
            .unwrap()
            .finished_at;
        if let Some(shift) = shift {
            let later = stage
                .matches
                .iter_mut()
                .find(|m| m.game.id == 1508621)
                .unwrap();
            later.actual_start = finish + Duration::minutes(15) + shift;
            later.game.scheduled_start = Some(later.actual_start);
        }
        let training_stage = if prior_is_holdout {
            // Keep the later Match on the training side and the veto's source
            // Stage on the Holdout, as can happen with a Stage exemption.
            let mut training = stage.clone();
            training.id = 88001;
            let index = stage
                .matches
                .iter()
                .position(|m| m.game.id == 1508621)
                .unwrap();
            training.matches = vec![stage.matches.remove(index)];
            let id = training.id;
            archive.stages.push(training);
            Some(id)
        } else {
            None
        };
        let mut lab = Lab::new(archive).unwrap();
        if let Some(stage) = training_stage {
            lab.holdout.from = finish;
            lab.holdout.training_stages = vec![stage];
        }
        lab.signals = vec![signals::by_name("bo1_veto_sim").unwrap()];
        lab.run().await.unwrap()
    };
    let at_finish = run(Some(Duration::zero()), false).await;
    let after_finish = run(Some(Duration::seconds(1)), false).await;
    let excluded = run(Some(Duration::seconds(1)), true).await;
    let prediction = |outcome: &kallbet_ai::adapters::lab::LabOutcome, id| {
        outcome
            .predictions
            .iter()
            .find(|p| p.match_id == id)
            .unwrap()
            .variant
    };
    assert_eq!(prediction(&after_finish, 1494593), None);
    assert_eq!(prediction(&at_finish, 1508621), None);
    assert!(prediction(&after_finish, 1508621).is_some());
    assert_eq!(prediction(&excluded, 1508621), None);
    assert!(after_finish.report.candidates[0].paired > 0);
    assert_eq!(
        after_finish.report.candidates[0].verdict,
        Verdict::NoEvidence
    );
}

/// Ticket #48: the #36 collector saves post-Match Liquipedia revisions per
/// PandaScore serie under `hltv/liquipedia/event-<serie>.json.gz` (one
/// MediaWiki response, one page per Stage page). The loader links them to that
/// serie's Stages exactly as a `maps-<tournament>` file is linked.
#[test]
fn a_serie_bundle_from_the_map_collector_links_like_a_maps_file() {
    let dir = archive_with_maps();
    let by_file = ReplayArchive::load(dir.path()).unwrap();
    let serie = by_file
        .stages
        .iter()
        .find(|s| s.id == 20710)
        .unwrap()
        .event_id;
    type Entry = (String, String, Vec<(String, bool)>, String);
    let entries = |a: &ReplayArchive| -> Vec<Entry> {
        a.map_history
            .at(chrono::DateTime::<chrono::Utc>::MAX_UTC)
            .map(|e| {
                let (m, at) = e.unwrap();
                (
                    m.team_a.clone(),
                    m.team_b.clone(),
                    m.results.clone(),
                    at.to_rfc3339(),
                )
            })
            .collect()
    };
    assert!(!entries(&by_file).is_empty());

    let bundle = tempfile::tempdir().unwrap();
    for name in ["panda-20710.json.gz", "standings_global_2026_06_01.md.gz"] {
        std::fs::copy(dir.path().join(name), bundle.path().join(name)).unwrap();
    }
    let liquipedia = bundle.path().join("hltv/liquipedia");
    std::fs::create_dir_all(&liquipedia).unwrap();
    let page = std::fs::read(dir.path().join("maps-20710.json")).unwrap();
    let mut gz = flate2::write::GzEncoder::new(
        std::fs::File::create(liquipedia.join(format!("event-{serie}.json.gz"))).unwrap(),
        flate2::Compression::default(),
    );
    std::io::Write::write_all(&mut gz, &page).unwrap();
    gz.finish().unwrap();
    // Discovery caches next to it are not map revisions.
    std::fs::write(liquipedia.join(format!("allpages-{serie}.json")), b"{}").unwrap();

    let by_bundle = ReplayArchive::load(bundle.path()).unwrap();
    assert_eq!(entries(&by_bundle), entries(&by_file));
    assert_ne!(
        by_bundle.digest, by_file.digest,
        "the bundle is part of the digest"
    );
}

/// An archive of Cologne Stage 1 (PandaScore 20708) and the saved HLTV page of
/// Heroic vs Sharks (HLTV 2394774, PandaScore 1494593), as the raw page only.
fn stage_with_raw_page() -> tempfile::TempDir {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let dir = tempfile::tempdir().unwrap();
    for name in ["panda-20708.json.gz", "standings_global_2026_06_01.md.gz"] {
        std::fs::copy(root.join("backtest").join(name), dir.path().join(name)).unwrap();
    }
    let hltv = dir.path().join("hltv");
    std::fs::create_dir_all(&hltv).unwrap();
    std::fs::copy(
        root.join("research/hltv-match-2394774-heroic-vs-sharks.html.gz"),
        hltv.join("hltv-match-2394774-heroic-vs-sharks.html.gz"),
    )
    .unwrap();
    dir
}

fn heroic_sharks_record() -> kallbet_ai::adapters::hltv_research::archive::HltvMatchRecord {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let html = read_file(&root.join("research/hltv-match-2394774-heroic-vs-sharks.html.gz")).unwrap();
    parse_match_page(
        2394774,
        "https://www.hltv.org/matches/2394774/heroic-vs-sharks-iem-cologne-major-2026-stage-1",
        "2026-11-26T01:15:00Z".parse().unwrap(),
        std::str::from_utf8(&html).unwrap(),
    )
    .unwrap()
}

// Finding 18 (review of 6eea194): a saved raw page without its parsed JSON
// supplies its complete veto too, though its fetch time is not in the file.
#[test]
fn a_raw_only_page_supplies_its_veto() {
    let dir = stage_with_raw_page();
    let archive = ReplayArchive::load(dir.path()).unwrap();
    assert_eq!(archive.hltv_matches.len(), 1);
    assert!(!archive.veto_history.is_empty(), "the complete Bo1 veto is linked");
}

// Finding 15 (review of 6eea194): two parsed records with one HLTV Match id
// and different content are an error, never one silently kept; the same
// record saved twice is one.
#[test]
fn conflicting_hltv_records_of_one_match_are_an_error() {
    let dir = stage_with_raw_page();
    let hltv = dir.path().join("hltv");
    std::fs::remove_file(hltv.join("hltv-match-2394774-heroic-vs-sharks.html.gz")).unwrap();
    let matches = hltv.join("matches");
    std::fs::create_dir_all(&matches).unwrap();
    let record = heroic_sharks_record();
    std::fs::write(hltv.join("hltv-match-2394774.json"), serde_json::to_vec(&record).unwrap()).unwrap();
    std::fs::write(matches.join("hltv-match-2394774.json"), serde_json::to_vec(&record).unwrap()).unwrap();
    assert_eq!(ReplayArchive::load(dir.path()).unwrap().hltv_matches.len(), 1);

    let mut other = record.clone();
    other.veto.reverse();
    std::fs::write(matches.join("hltv-match-2394774.json"), serde_json::to_vec(&other).unwrap()).unwrap();
    let e = ReplayArchive::load(dir.path()).err().expect("refused");
    assert!(e.to_string().contains("2394774"), "{e}");
}
