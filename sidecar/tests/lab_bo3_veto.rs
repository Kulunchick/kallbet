//! Tests for ticket #52 (`bo3_veto_sim`, the Bo3 counterpart of #44's `bo1_veto_sim`).
//! Uses only saved real fixtures in `sidecar/tests/fixtures/{backtest,research}/`; no network.
use chrono::Duration;
use kallbet_ai::{
    Format,
    adapters::{
        backtest::{ReplayArchive, archive::read_file},
        hltv_research::parse::parse_match_page,
        lab::{
            Lab, LabOutcome, signals,
            signals::{
                bo3_veto_sim::{bo3_series_probability, simulate_bo3_veto},
                per_map_rating::map_series_probability,
            },
            stats::Verdict,
        },
    },
    vrs::series_probability,
};
use std::path::Path;

#[test]
fn bo3_series_probability_agrees_with_existing_conversion_and_hand_checked_cases() {
    for p in [0.0, 0.25, 0.5, 0.6, 0.73, 1.0] {
        let actual = bo3_series_probability(p, p, p);
        let existing = series_probability(p, Format::Bo3);
        assert!(
            (actual - existing).abs() < 1e-12,
            "p={p}: actual={actual} vs existing={existing}"
        );
    }
    // Hand-checked unequal maps:
    // 2-0: 0.8 * 0.2 = 0.16
    // W-L-W: 0.8 * 0.8 * 0.6 = 0.384
    // L-W-W: 0.2 * 0.2 * 0.6 = 0.024
    // Total: 0.568
    assert!((bo3_series_probability(0.8, 0.2, 0.6) - 0.568).abs() < 1e-12);
    // Certain win on map 1, certain loss on map 2 -> decided solely by map 3.
    assert!((bo3_series_probability(1.0, 0.0, 0.75) - 0.75).abs() < 1e-12);
    // Anti-symmetry: P(A wins) + P(B wins) == 1.
    let p_a = bo3_series_probability(0.7, 0.4, 0.55);
    let p_b = bo3_series_probability(0.3, 0.6, 0.45);
    assert!((p_a + p_b - 1.0).abs() < 1e-12);
}

#[test]
fn bo3_veto_simulation_respects_equal_maps_uniform_weights_symmetry_and_veto_habits() {
    let ab_ab_ab = [0, 1, 0, 1, 0, 1];
    let ab_ab_ba = [0, 1, 0, 1, 1, 0];

    // Equal map probabilities always agree with the existing Bo3 conversion,
    // regardless of ban/pick weights or second-phase ban order.
    let skew_bans = [[1.0, 9.0, 2.0, 4.0, 1.0, 3.0, 5.0], [8.0, 1.0, 3.0, 2.0, 6.0, 1.0, 2.0]];
    let skew_picks = [[7.0, 1.0, 4.0, 1.0, 5.0, 2.0, 1.0], [1.0, 6.0, 2.0, 8.0, 1.0, 3.0, 4.0]];
    for order in [ab_ab_ab, ab_ab_ba] {
        let actual = simulate_bo3_veto(order, skew_bans, skew_picks, [0.6; 7]);
        let expected = series_probability(0.6, Format::Bo3);
        assert!((actual - expected).abs() < 1e-12);
    }

    // Uniform ban and pick weights match uniform 3-map subset averaging (`per_map_rating`).
    let wins = [0.85, 0.70, 0.60, 0.55, 0.45, 0.35, 0.20];
    let uniform_expected = map_series_probability(&wins, Format::Bo3).unwrap();
    for order in [ab_ab_ab, ab_ab_ba] {
        let actual = simulate_bo3_veto(order, [[1.0; 7]; 2], [[1.0; 7]; 2], wins);
        assert!(
            (actual - uniform_expected).abs() < 1e-12,
            "uniform Bo3 veto ({actual}) should equal uniform 3-subset average ({uniform_expected})"
        );
    }

    // Anti-symmetry when swapping Team A and Team B.
    for order in [ab_ab_ab, ab_ab_ba] {
        let p_a = simulate_bo3_veto(order, skew_bans, skew_picks, wins);
        let p_b = simulate_bo3_veto(
            order,
            [skew_bans[1], skew_bans[0]],
            [skew_picks[1], skew_picks[0]],
            wins.map(|w| 1.0 - w),
        );
        assert!((p_a + p_b - 1.0).abs() < 1e-12);
    }

    // Team A strongly prefers picking map 0 (win=0.90) while Team B permabans map 6
    // raises Team A's series probability well above uniform; if Team B instead
    // permabans map 0, Team A's series probability drops.
    let strong_map0 = [0.90, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50];
    let baseline = simulate_bo3_veto(ab_ab_ab, [[1.0; 7]; 2], [[1.0; 7]; 2], strong_map0);
    let a_picks_0 = simulate_bo3_veto(
        ab_ab_ab,
        [[1.0; 7], [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 20.0]],
        [[20.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0], [1.0; 7]],
        strong_map0,
    );
    let b_bans_0 = simulate_bo3_veto(
        ab_ab_ab,
        [[1.0; 7], [50.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0]],
        [[20.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0], [1.0; 7]],
        strong_map0,
    );
    assert!(a_picks_0 > baseline + 0.05, "{a_picks_0} vs {baseline}");
    assert!(b_bans_0 < baseline, "{b_bans_0} vs {baseline}");
}

fn vitality_fut_record() -> kallbet_ai::adapters::hltv_research::archive::HltvMatchRecord {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let html = read_file(&root.join("research/hltv-match-2394898-vitality-vs-fut.html.gz")).unwrap();
    parse_match_page(
        2394898,
        "https://www.hltv.org/matches/2394898/vitality-vs-fut-iem-cologne-major-2026",
        "2026-11-26T01:15:00Z".parse().unwrap(),
        std::str::from_utf8(&html).unwrap(),
    )
    .unwrap()
}

fn stage3_with_bo3_veto(raw_only: bool) -> tempfile::TempDir {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let dir = tempfile::tempdir().unwrap();
    for name in ["panda-21115.json.gz", "standings_global_2026_06_01.md.gz"] {
        std::fs::copy(root.join("backtest").join(name), dir.path().join(name)).unwrap();
    }
    let hltv_dir = dir.path().join("hltv");
    let matches_dir = hltv_dir.join("matches");
    std::fs::create_dir_all(&matches_dir).unwrap();
    std::fs::write(hltv_dir.join("manifest.json"), br#"{"version":1}"#).unwrap();
    if raw_only {
        std::fs::copy(
            root.join("research/hltv-match-2394898-vitality-vs-fut.html.gz"),
            matches_dir.join("hltv-match-2394898-vitality-vs-fut.html.gz"),
        )
        .unwrap();
    } else {
        let record = vitality_fut_record();
        std::fs::write(
            matches_dir.join("hltv-match-2394898.json"),
            serde_json::to_vec(&record).unwrap(),
        )
        .unwrap();
    }
    dir
}

#[tokio::test]
async fn bo3_veto_sim_is_registered_reports_post_hoc_marker_and_not_testable_when_missing() {
    let signal = signals::by_name("bo3_veto_sim").expect("bo3_veto_sim is registered");
    assert!(signal.added_after_list_fixed());
    assert!(!signals::by_name("bo1_veto_sim").unwrap().added_after_list_fixed());
    assert!(!signals::by_name("per_map_rating").unwrap().added_after_list_fixed());

    // Archive without any HLTV Bo3 vetoes: reported `not testable` with reason and post-hoc mark.
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let dir = tempfile::tempdir().unwrap();
    for name in ["panda-21115.json.gz", "standings_global_2026_06_01.md.gz"] {
        std::fs::copy(root.join("backtest").join(name), dir.path().join(name)).unwrap();
    }
    let archive = ReplayArchive::load(dir.path()).unwrap();
    let mut lab = Lab::new(archive).unwrap();
    lab.signals = ["per_map_rating", "bo3_veto_sim"]
        .map(|name| signals::by_name(name).unwrap())
        .into();
    let outcome = lab.run().await.unwrap();
    let row = outcome
        .report
        .candidates
        .iter()
        .find(|c| c.name == "bo3_veto_sim")
        .unwrap();
    assert_eq!(row.verdict, Verdict::NotTestable);
    assert!(row.reason.contains("Bo3"), "{}", row.reason);
    assert!(row.added_after_list_fixed);
    assert!(!outcome.report.candidates[0].added_after_list_fixed);

    let json = serde_json::to_value(&outcome.report).unwrap();
    let bo3_json = json["candidates"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "bo3_veto_sim")
        .unwrap();
    assert_eq!(bo3_json["added_after_list_fixed"], true);

    let md = outcome.report.markdown();
    assert!(
        md.contains("not testable (added after list fixed)"),
        "markdown must mark bo3_veto_sim next to its verdict:\n{md}"
    );
}

#[tokio::test]
async fn bo3_veto_uses_earlier_saved_veto_never_its_own_and_respects_finish_and_holdout() {
    let dir = stage3_with_bo3_veto(false);
    let path = dir.path();

    let run = |shift: Option<Duration>, prior_is_holdout: bool| async move {
        let mut archive = ReplayArchive::load(path).unwrap();
        let stage = archive.stages.iter_mut().find(|s| s.id == 21115).unwrap();
        // Match 1513120 is Vitality vs FUT (HLTV 2394898).
        let finish = stage
            .matches
            .iter()
            .find(|m| m.game.id == 1513120)
            .unwrap()
            .reported_finish
            .unwrap();
        if let Some(shift) = shift {
            // Match 1513135 is Vitality vs 9z (a later Bo3 in Stage 3).
            let later = stage
                .matches
                .iter_mut()
                .find(|m| m.game.id == 1513135)
                .unwrap();
            later.actual_start = finish + Duration::minutes(15) + shift;
            later.game.scheduled_start = Some(later.actual_start);
            assert_eq!(later.cutoff(), finish + shift);
        }
        let training_stage = if prior_is_holdout {
            // Put Vitality vs FUT's Stage in the Holdout while keeping 1513135 on the training side.
            let mut training = stage.clone();
            training.id = 88052;
            let index = stage
                .matches
                .iter()
                .position(|m| m.game.id == 1513135)
                .unwrap();
            training.matches = vec![stage.matches.remove(index)];
            let id = training.id;
            archive.stages.push(training);
            Some(id)
        } else {
            None
        };
        let mut lab = Lab::new(archive).unwrap();
        if let Some(stage_id) = training_stage {
            lab.holdout.from = finish;
            lab.holdout.training_stages = vec![stage_id];
        }
        lab.signals = vec![signals::by_name("bo3_veto_sim").unwrap()];
        lab.run().await.unwrap()
    };

    let at_finish = run(Some(Duration::zero()), false).await;
    let after_finish = run(Some(Duration::seconds(1)), false).await;
    let excluded = run(Some(Duration::seconds(1)), true).await;

    let prediction = |outcome: &LabOutcome, id: i64| {
        outcome
            .predictions
            .iter()
            .find(|p| p.signal == "bo3_veto_sim" && p.match_id == id)
            .unwrap()
            .variant
    };

    // Match 1513120 never reads its own veto or map results.
    assert_eq!(prediction(&after_finish, 1513120), None);
    // Strict finish boundary: at finish is not available; 1s after is available.
    assert_eq!(prediction(&at_finish, 1513135), None);
    let p = prediction(&after_finish, 1513135).expect("later Vitality Bo3 is predicted");
    assert!((0.0..=1.0).contains(&p));
    assert!(
        (p - 0.5).abs() > 1e-4,
        "prediction should reflect Vitality's map ratings & veto, not flat 0.5: {p}"
    );
    // Holdout exclusion: Holdout veto & map results never reach training history.
    assert_eq!(prediction(&excluded, 1513135), None);

    let row = &after_finish.report.candidates[0];
    assert_eq!(row.name, "bo3_veto_sim");
    assert!(row.paired > 0);
    assert!(row.difference.is_some());
    assert!(row.interval.is_some());
    assert_eq!(row.verdict, Verdict::NoEvidence);
    assert!(row.added_after_list_fixed);
    let md = after_finish.report.markdown();
    assert!(md.contains("no evidence (added after list fixed)"), "{md}");
}

#[tokio::test]
async fn bo3_veto_reads_liquipedia_map_ratings_strictly_before_cutoff_and_excludes_holdout() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let dir = stage3_with_bo3_veto(true);
    // Also include Cologne Playoffs PandaScore + Liquipedia post-match maps.
    std::fs::copy(
        root.join("backtest/panda-20710.json.gz"),
        dir.path().join("panda-20710.json.gz"),
    )
    .unwrap();
    std::fs::copy(
        root.join("research/liquipedia-cologne-playoffs-postmatch.json"),
        dir.path().join("maps-20710.json"),
    )
    .unwrap();

    // Verify raw-only HLTV Bo3 page linked cleanly.
    let loaded = ReplayArchive::load(dir.path()).unwrap();
    assert!(loaded.veto_history.has_bo3());
    assert!(!loaded.map_history.is_empty());

    let path = dir.path();
    let run = |shift: Duration, maps_in_holdout: bool| async move {
        let mut archive = ReplayArchive::load(path).unwrap();
        // Match 1513154 (FURIA vs 9z, Playoffs QF) has Liquipedia played maps for 9z.
        // Move Vitality vs 9z (1513135, Stage 3) relative to 1513154's finish so we can
        // test that Liquipedia map ratings for 9z enter ONLY strictly after finish and
        // never when 20710 is in the Holdout.
        let qf_finish = archive
            .stages
            .iter()
            .find(|s| s.id == 20710)
            .unwrap()
            .matches
            .iter()
            .find(|m| m.game.id == 1513154)
            .unwrap()
            .reported_finish
            .unwrap();
        let stage3 = archive.stages.iter_mut().find(|s| s.id == 21115).unwrap();
        let vit_9z = stage3
            .matches
            .iter_mut()
            .find(|m| m.game.id == 1513135)
            .unwrap();
        let start = qf_finish + Duration::minutes(15) + shift;
        vit_9z.actual_start = start;
        vit_9z.game.scheduled_start = Some(start);
        vit_9z.finished_at = start + Duration::hours(2);
        vit_9z.reported_finish = Some(vit_9z.finished_at);
        let mut lab = Lab::new(archive).unwrap();
        if maps_in_holdout {
            lab.holdout.from = qf_finish - Duration::days(2);
            lab.holdout.training_stages = vec![21115];
        }
        lab.signals = vec![signals::by_name("bo3_veto_sim").unwrap()];
        let out = lab.run().await.unwrap();
        out.predictions
            .iter()
            .find(|p| p.signal == "bo3_veto_sim" && p.match_id == 1513135)
            .unwrap()
            .variant
            .unwrap()
    };

    let before_maps = run(Duration::seconds(-1), false).await;
    let at_maps = run(Duration::zero(), false).await;
    let after_maps = run(Duration::seconds(1), false).await;
    let holdout_maps = run(Duration::seconds(1), true).await;

    assert_eq!(before_maps, at_maps);
    assert_ne!(at_maps, after_maps, "9z's Liquipedia map results must update Elo after finish");
    assert_eq!(holdout_maps, at_maps, "Holdout map results must never reach training history");
}

#[test]
fn hltv_loader_skips_manifest_recorded_walkover_parse_errors_only() {
    let dir = stage3_with_bo3_veto(false);
    let hltv_dir = dir.path().join("hltv");
    let matches_dir = hltv_dir.join("matches");
    // A walkover HTML page without player stats fails unless recorded in manifest.json.
    std::fs::write(
        matches_dir.join("hltv-match-9999999.html"),
        b"<html><body>walkover</body></html>",
    )
    .unwrap();
    assert!(ReplayArchive::load(dir.path()).is_err());

    std::fs::write(
        hltv_dir.join("manifest.json"),
        br#"{"version":1,"parse_errors":{"9999999":"missing overall player stats (#all-content)"}}"#,
    )
    .unwrap();
    let loaded = ReplayArchive::load(dir.path()).unwrap();
    assert_eq!(loaded.hltv_matches.len(), 1);
    assert!(loaded.veto_history.has_bo3());
}
