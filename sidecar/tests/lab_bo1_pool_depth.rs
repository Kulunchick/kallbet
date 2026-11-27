//! Tests for ticket #62 (`bo1_pool_depth`: Bo1 map-pool depth and first-ban features).
//! Uses only saved real fixtures in `sidecar/tests/fixtures/{backtest,research}/`; no network.
use chrono::Duration;
use kallbet_ai::{
    Format,
    adapters::{
        backtest::{ReplayArchive, archive::read_file},
        hltv_research::parse::parse_match_page,
        lab::{
            Lab, LabOutcome, LabPrediction,
            signals::{
                self,
                bo1_pool_depth::{
                    COMPETENCE_PRIOR, FEATURES, STABILITY_PRIOR, TeamVetoStats, compute_features,
                    first_ban_disruption, pool_overlap, summarize_team,
                },
            },
            stats::Verdict,
        },
    },
};
use std::path::Path;

fn sample_pool() -> [String; 7] {
    [
        "mirage".into(),
        "inferno".into(),
        "nuke".into(),
        "ancient".into(),
        "dust2".into(),
        "anubis".into(),
        "train".into(),
    ]
}

#[test]
fn pool_depth_first_ban_and_overlap_features_match_hand_checked_values_and_anti_symmetry() {
    let pool = sample_pool();

    // Team A: 2 vetoes (equal weight 1.0), always first-bans train.
    // Wins: mirage 2-0, inferno 1-1, nuke 0-1.
    let mut stats_a = TeamVetoStats::default();
    stats_a.record_observation(Some("train"), &[("mirage", true), ("inferno", true)], 1.0);
    stats_a.record_observation(
        Some("train"),
        &[("mirage", true), ("inferno", false), ("nuke", false)],
        1.0,
    );
    let sum_a = summarize_team(&stats_a, &pool);

    // Hand-checked competence for Team A (COMPETENCE_PRIOR = 1.0):
    // mirage: 2 / (2 + 1) * (1 - 0) = 2/3
    // inferno: 1 / (2 + 1) * (1 - 0) = 1/3
    // nuke: 0 / (1 + 1) = 0.0 (0 wins never counts as competent)
    // all others unplayed = 0.0
    assert!((sum_a.competence[0] - 2.0 / (2.0 + COMPETENCE_PRIOR)).abs() < 1e-12);
    assert!((sum_a.competence[1] - 1.0 / (2.0 + COMPETENCE_PRIOR)).abs() < 1e-12);
    assert_eq!(sum_a.competence[2], 0.0);
    assert_eq!(sum_a.competence[6], 0.0);
    assert!((sum_a.depth - 1.0).abs() < 1e-12);
    // First-ban stability: (2 + 2/7) / (2 + 2) = (16/7) / 4 = 4/7
    let expected_stab_a = (2.0 + STABILITY_PRIOR / 7.0) / (2.0 + STABILITY_PRIOR);
    assert!((sum_a.first_ban_stability - expected_stab_a).abs() < 1e-12);

    // Team B: 2 vetoes, splits first bans between mirage and dust2,
    // wins inferno 1-0 and ancient 1-1.
    let mut stats_b = TeamVetoStats::default();
    stats_b.record_observation(Some("mirage"), &[("inferno", true), ("ancient", true)], 1.0);
    stats_b.record_observation(Some("dust2"), &[("ancient", false)], 1.0);
    let sum_b = summarize_team(&stats_b, &pool);

    // Team B competence: inferno = 1/2 = 0.5, ancient = 1/3, others 0 -> depth = 5/6
    assert!((sum_b.competence[1] - 0.5).abs() < 1e-12);
    assert!((sum_b.competence[3] - 1.0 / 3.0).abs() < 1e-12);
    assert!((sum_b.depth - 5.0 / 6.0).abs() < 1e-12);
    // Team B stability is strictly lower than Team A's strict permaban.
    assert!(sum_b.first_ban_stability < sum_a.first_ban_stability);

    // Weighted Jaccard overlap:
    // min: mirage 0, inferno min(1/3, 1/2)=1/3, ancient 0 -> sum = 1/3
    // max: mirage 2/3, inferno 1/2, ancient 1/3 -> sum = 9/6 = 3/2
    // overlap = (1/3) / (3/2) = 2/9
    assert!((pool_overlap(&sum_a, &sum_b) - 2.0 / 9.0).abs() < 1e-12);
    assert!((pool_overlap(&sum_a, &sum_a) - 1.0).abs() < 1e-12);
    assert_eq!(pool_overlap(&sum_a, &sum_b), pool_overlap(&sum_b, &sum_a));

    // First-ban disruption:
    // A first-bans train (100%), B competence on train is 0 -> disrupt(A on B) = 0
    // B first-bans mirage (50%) and dust2 (50%), A competence on mirage is 2/3 -> disrupt(B on A) = 1/3
    assert!((first_ban_disruption(&sum_a, &sum_b) - 0.0).abs() < 1e-12);
    assert!((first_ban_disruption(&sum_b, &sum_a) - 1.0 / 3.0).abs() < 1e-12);

    // Rotated-out permaban drops active-pool first-ban stability below uniform.
    let mut rotated_pool = pool.clone();
    rotated_pool[6] = "overpass".into();
    let sum_a_rotated = summarize_team(&stats_a, &rotated_pool);
    assert!(
        sum_a_rotated.first_ban_stability < 1.0 / 7.0,
        "rotated-out permaban should yield low active stability: {}",
        sum_a_rotated.first_ban_stability
    );

    // Strict anti-symmetry of all 4 features under swapping Team A and Team B.
    for anchor in [0.25, 0.5, 0.72] {
        let fab = compute_features(&sum_a, &sum_b, anchor);
        let fba = compute_features(&sum_b, &sum_a, 1.0 - anchor);
        for i in 0..4 {
            assert!(
                (fab[i] + fba[i]).abs() < 1e-12,
                "feature {i} ({}) not anti-symmetric at anchor={anchor}: {} vs {}",
                FEATURES[i],
                fab[i],
                fba[i]
            );
        }
    }
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

#[tokio::test]
async fn bo1_pool_depth_is_registered_marked_post_hoc_and_not_testable_when_vetoes_missing() {
    let signal = signals::by_name("bo1_pool_depth").expect("bo1_pool_depth is registered");
    assert!(signal.added_after_list_fixed());
    assert_eq!(signal.feature_names(), FEATURES.to_vec());

    // Archive without any HLTV vetoes: reported `not testable` with post-hoc marker.
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let dir = tempfile::tempdir().unwrap();
    for name in ["panda-20708.json.gz", "standings_global_2026_06_01.md.gz"] {
        std::fs::copy(root.join("backtest").join(name), dir.path().join(name)).unwrap();
    }
    let archive = ReplayArchive::load(dir.path()).unwrap();
    let mut lab = Lab::new(archive).unwrap();
    lab.signals = vec![signal];
    let outcome = lab.run().await.unwrap();

    let row = &outcome.report.candidates[0];
    assert_eq!(row.name, "bo1_pool_depth");
    assert_eq!(row.verdict, Verdict::NotTestable);
    assert!(row.added_after_list_fixed);
    assert!(row.reason.contains("HLTV"), "{}", row.reason);

    let json = serde_json::to_value(&outcome.report).unwrap();
    assert_eq!(json["candidates"][0]["added_after_list_fixed"], true);
    let md = outcome.report.markdown();
    assert!(
        md.contains("not testable (added after list fixed)"),
        "markdown must mark post-hoc status next to verdict:\n{md}"
    );
}

fn cologne_with_bo1_and_bo3_vetoes() -> tempfile::TempDir {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let dir = tempfile::tempdir().unwrap();
    for name in [
        "panda-20708.json.gz",
        "panda-21115.json.gz",
        "standings_global_2026_06_01.md.gz",
    ] {
        std::fs::copy(root.join("backtest").join(name), dir.path().join(name)).unwrap();
    }
    let hltv_dir = dir.path().join("hltv");
    let matches_dir = hltv_dir.join("matches");
    std::fs::create_dir_all(&matches_dir).unwrap();
    std::fs::write(hltv_dir.join("manifest.json"), br#"{"version":1}"#).unwrap();
    std::fs::write(
        matches_dir.join("hltv-match-2394774.json"),
        serde_json::to_vec(&heroic_sharks_record()).unwrap(),
    )
    .unwrap();
    std::fs::write(
        matches_dir.join("hltv-match-2394898.json"),
        serde_json::to_vec(&vitality_fut_record()).unwrap(),
    )
    .unwrap();
    dir
}

#[tokio::test]
async fn bo1_pool_depth_predicts_only_bo1_uses_bo3_vetoes_strictly_before_cutoff_and_excludes_holdout() {
    let dir = cologne_with_bo1_and_bo3_vetoes();
    let path = dir.path();

    // Stage 1 (20708) has Heroic vs Sharks (Bo1, Match 1494593).
    // Stage 3 (21115) has Vitality vs FUT (Bo3, Match 1513120).
    // Take a later Bo1 in Stage 1 (Match 1508621, Heroic vs opponent) and pair Heroic against
    // Vitality at `bo3_finish + 15m + shift`, so:
    // - Heroic's prior veto comes from a Bo1 (1494593)
    // - Vitality's prior veto comes from a Bo3 (1513120)
    // - We test strict finish boundary on Vitality's Bo3 finish and Holdout exclusion of Stage 3.
    let run = |shift: Duration, bo3_stage_in_holdout: bool| async move {
        let mut archive = ReplayArchive::load(path).unwrap();
        let stage3 = archive.stages.iter().find(|s| s.id == 21115).unwrap();
        let vit_match = stage3.matches.iter().find(|m| m.game.id == 1513120).unwrap();
        let bo3_finish = vit_match.reported_finish.unwrap();
        let vitality_team = vit_match.game.team_a.clone();

        let stage1 = archive.stages.iter_mut().find(|s| s.id == 20708).unwrap();
        let probe = stage1.matches.iter_mut().find(|m| m.game.id == 1508621).unwrap();
        assert_eq!(probe.game.format, Format::Bo1);
        // Set team_b to Vitality (keeping Heroic as team_a, winner valid).
        probe.game.team_b = vitality_team;
        probe.winner = probe.game.team_a.id;
        let start = bo3_finish + Duration::minutes(15) + shift;
        probe.actual_start = start;
        probe.game.scheduled_start = Some(start);
        probe.finished_at = start + Duration::hours(1);
        probe.reported_finish = Some(probe.finished_at);
        assert_eq!(probe.cutoff(), bo3_finish + shift);

        let mut lab = Lab::new(archive).unwrap();
        if bo3_stage_in_holdout {
            // Put Stage 3 (21115, containing Vitality's Bo3 veto) in Holdout while keeping
            // Stage 1 (20708) on the training side.
            lab.holdout.from = bo3_finish - Duration::days(1);
            lab.holdout.training_stages = vec![20708];
        }
        lab.signals = vec![signals::by_name("bo1_pool_depth").unwrap()];
        lab.run().await.unwrap()
    };

    let at_finish = run(Duration::zero(), false).await;
    let after_finish = run(Duration::seconds(1), false).await;
    let holdout_excluded = run(Duration::seconds(1), true).await;

    let prediction = |outcome: &LabOutcome, id: i64| {
        outcome
            .predictions
            .iter()
            .find(|p| p.signal == "bo1_pool_depth" && p.match_id == id)
            .unwrap()
            .variant
    };

    // Match 1494593 (Heroic vs Sharks) never reads its own veto.
    assert_eq!(prediction(&after_finish, 1494593), None);
    // Bo3 Matches (Stage 3, e.g. 1513120 and 1513135) are NEVER predicted by bo1_pool_depth.
    assert_eq!(prediction(&after_finish, 1513120), None);
    assert_eq!(prediction(&after_finish, 1513135), None);
    // Strict finish boundary: at Vitality's Bo3 finish -> None; 1s after -> Some.
    assert_eq!(prediction(&at_finish, 1508621), None);
    let p = prediction(&after_finish, 1508621)
        .expect("later Heroic vs Vitality Bo1 is predicted using Bo1 + Bo3 prior vetoes");
    assert!((0.0..=1.0).contains(&p));
    // Holdout exclusion: Vitality's Bo3 veto in Holdout never reaches training-side views.
    assert_eq!(prediction(&holdout_excluded, 1508621), None);

    let row = &after_finish.report.candidates[0];
    assert_eq!(row.name, "bo1_pool_depth");
    assert_eq!(row.computed_on, 1);
    assert_eq!(row.paired, 1);
    assert!(row.difference.is_some());
    assert!(row.interval.is_some());
    assert!(row.accuracy_interval.is_some());
    assert_eq!(row.verdict, Verdict::NoEvidence);
    assert!(row.added_after_list_fixed);
}

#[tokio::test]
async fn bo1_pool_depth_fits_walk_forward_across_events_without_future_leakage() {
    let dir = cologne_with_bo1_and_bo3_vetoes();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    for name in ["panda-22378.json.gz", "standings_global_2026_11_02.md.gz"] {
        std::fs::copy(root.join("backtest").join(name), dir.path().join(name)).unwrap();
    }
    let path = dir.path();

    let run = |flip_event1_winner: bool| async move {
        let mut archive = ReplayArchive::load(path).unwrap();
        let stage3 = archive.stages.iter().find(|s| s.id == 21115).unwrap();
        let vit_match = stage3.matches.iter().find(|m| m.game.id == 1513120).unwrap();
        let bo3_finish = vit_match.reported_finish.unwrap();
        let vitality_team = vit_match.game.team_a.clone();

        // Event 1 (Cologne Stage 1, 20708): place Heroic vs Vitality Bo1 after Vitality's Bo3.
        let stage1 = archive.stages.iter_mut().find(|s| s.id == 20708).unwrap();
        let e1_probe = stage1.matches.iter_mut().find(|m| m.game.id == 1508621).unwrap();
        e1_probe.game.team_b = vitality_team.clone();
        e1_probe.winner = if flip_event1_winner {
            e1_probe.game.team_b.id
        } else {
            e1_probe.game.team_a.id
        };
        let e1_start = bo3_finish + Duration::hours(1);
        e1_probe.actual_start = e1_start;
        e1_probe.game.scheduled_start = Some(e1_start);
        e1_probe.finished_at = e1_start + Duration::hours(1);
        e1_probe.reported_finish = Some(e1_probe.finished_at);

        // Event 2 (Singapore Stage 1, 22378): pair Heroic vs Vitality in a later Bo1 (1816914).
        let singapore = archive.stages.iter_mut().find(|s| s.id == 22378).unwrap();
        let e2_probe = singapore.matches.iter_mut().find(|m| m.game.id == 1816914).unwrap();
        assert_eq!(e2_probe.game.format, Format::Bo1);
        e2_probe.game.team_b = vitality_team;
        e2_probe.winner = e2_probe.game.team_a.id;

        let mut lab = Lab::new(archive).unwrap();
        lab.signals = vec![signals::by_name("bo1_pool_depth").unwrap()];
        lab.run().await.unwrap()
    };

    let normal = run(false).await;
    let flipped = run(true).await;

    fn pred(o: &LabOutcome, id: i64) -> &LabPrediction {
        o.predictions
            .iter()
            .find(|p| p.signal == "bo1_pool_depth" && p.match_id == id)
            .unwrap()
    }

    // Event 1 has 0 prior training Bo1s -> prediction equals anchor and does not depend on its own outcome.
    let e1_n = pred(&normal, 1508621);
    let e1_f = pred(&flipped, 1508621);
    assert_eq!(e1_n.variant, e1_f.variant);
    assert!((e1_n.variant.unwrap() - e1_n.baseline.unwrap()).abs() < 1e-12);

    // Event 2 trains on Event 1's Bo1 -> reversing Event 1's outcome moves Event 2's prediction!
    let e2_n = pred(&normal, 1816914);
    let e2_f = pred(&flipped, 1816914);
    assert!(e2_n.variant.is_some() && e2_f.variant.is_some());
    assert_ne!(
        e2_n.variant, e2_f.variant,
        "reversing Event 1's Bo1 winner must shift Event 2's fitted Bo1 prediction"
    );

    // Per-event parameters report training_bo1_matches and feature weights.
    let row = &normal.report.candidates[0];
    assert_eq!(row.computed_on, 2);
    assert_eq!(row.paired, 2);
    let e2_params = row.per_event.last().unwrap().parameters.as_ref().unwrap();
    assert_eq!(e2_params["training_bo1_matches"], 1);
    assert_eq!(e2_params["features"].as_array().unwrap().len(), 4);
    assert_eq!(e2_params["weights"].as_array().unwrap().len(), 4);
}

#[tokio::test]
async fn bo1_pool_depth_runs_alongside_bo1_veto_sim_and_recency_weights_newer_vetoes_higher() {
    // Recency weighting: a recent first ban on nuke (weight 1.0) outweighs an older first ban
    // on mirage from 180 days earlier (weight 0.25) when summarizing first-ban shares.
    let pool = sample_pool();
    let mut stats = TeamVetoStats::default();
    stats.record_observation(Some("mirage"), &[("inferno", true)], 0.25);
    stats.record_observation(Some("nuke"), &[("inferno", false)], 1.0);
    let summary = summarize_team(&stats, &pool);
    assert!(
        summary.first_ban_share[2] > summary.first_ban_share[0],
        "recent first ban (nuke={}) must outweigh old first ban (mirage={})",
        summary.first_ban_share[2],
        summary.first_ban_share[0]
    );

    // Running alongside `bo1_veto_sim` over saved fixtures leaves `bo1_veto_sim` unmarked
    // and marks `bo1_pool_depth` as added after the list was fixed.
    let dir = cologne_with_bo1_and_bo3_vetoes();
    let archive = ReplayArchive::load(dir.path()).unwrap();
    let mut lab = Lab::new(archive).unwrap();
    lab.signals = ["bo1_pool_depth", "bo1_veto_sim"]
        .map(|name| signals::by_name(name).unwrap())
        .into();
    let outcome = lab.run().await.unwrap();
    assert_eq!(outcome.report.candidates.len(), 2);
    assert_eq!(outcome.report.candidates[0].name, "bo1_pool_depth");
    assert!(outcome.report.candidates[0].added_after_list_fixed);
    assert_eq!(outcome.report.candidates[1].name, "bo1_veto_sim");
    assert!(!outcome.report.candidates[1].added_after_list_fixed);
}

