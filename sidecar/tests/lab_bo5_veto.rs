//! Tests for ticket #54 (`bo5_veto_sim` and format-agnostic `veto_sim`).
//! Uses only saved fixtures in `sidecar/tests/fixtures/{backtest,research}/`; no network.
use chrono::{DateTime, Duration, Utc};
use kallbet_ai::{
    Format,
    adapters::{
        backtest::{ReplayArchive, archive::read_file},
        hltv_research::{
            archive::{HltvMatchRecord, MapResult, Side, TeamRef, VetoAction, VetoStep},
            parse::parse_match_page,
        },
        lab::{
            Lab, LabOutcome, signals,
            signals::{
                bo5_veto_sim::{bo5_series_probability, simulate_bo5_veto},
                per_map_rating::map_series_probability,
            },
            stats::Verdict,
        },
    },
    vrs::series_probability,
};
use std::{collections::BTreeMap, path::Path};

#[test]
fn bo5_series_probability_agrees_with_existing_conversion_and_hand_checked_cases() {
    for p in [0.0, 0.25, 0.5, 0.6, 0.73, 1.0] {
        let actual = bo5_series_probability([p; 5]);
        let existing = series_probability(p, Format::Bo5);
        assert!(
            (actual - existing).abs() < 1e-12,
            "p={p}: actual={actual} vs existing={existing}"
        );
    }
    assert!((bo5_series_probability([0.6; 5]) - 0.68256).abs() < 1e-12);

    // Hand-checked unequal maps [0.8, 0.2, 0.6, 0.5, 0.7]:
    // 3-0: 0.8 * 0.2 * 0.6 = 0.096
    // 3-1: (0.8*0.2*0.4 + 0.8*0.8*0.6 + 0.2*0.2*0.6) * 0.5 = 0.472 * 0.5 = 0.236
    // 3-2: (0.032 + 0.192 + 0.128 + 0.012 + 0.008 + 0.048) * 0.7 = 0.420 * 0.7 = 0.294
    // Total: 0.626
    assert!((bo5_series_probability([0.8, 0.2, 0.6, 0.5, 0.7]) - 0.626).abs() < 1e-12);
    // Independent maps: reversing map order yields the same series win probability.
    assert!((bo5_series_probability([0.7, 0.5, 0.6, 0.2, 0.8]) - 0.626).abs() < 1e-12);
    // Two certain wins and two certain losses -> decided solely by map 5.
    assert!((bo5_series_probability([1.0, 1.0, 0.0, 0.0, 0.75]) - 0.75).abs() < 1e-12);
    // Anti-symmetry: P(A wins) + P(B wins) == 1.
    let p_a = bo5_series_probability([0.7, 0.4, 0.55, 0.65, 0.35]);
    let p_b = bo5_series_probability([0.3, 0.6, 0.45, 0.35, 0.65]);
    assert!((p_a + p_b - 1.0).abs() < 1e-12);
}

#[test]
fn bo5_veto_simulation_respects_equal_maps_uniform_weights_symmetry_and_ban_habits() {
    let ab_ab_ab = [0, 1, 0, 1, 0, 1];
    let ab_ab_ba = [0, 1, 0, 1, 1, 0];

    let skew_bans = [
        [1.0, 9.0, 2.0, 4.0, 1.0, 3.0, 5.0],
        [8.0, 1.0, 3.0, 2.0, 6.0, 1.0, 2.0],
    ];
    let skew_picks = [
        [7.0, 1.0, 4.0, 1.0, 5.0, 2.0, 1.0],
        [1.0, 6.0, 2.0, 8.0, 1.0, 3.0, 4.0],
    ];

    // Equal map probabilities always agree with the existing Bo5 conversion.
    for order in [ab_ab_ab, ab_ab_ba] {
        let actual = simulate_bo5_veto(order, skew_bans, skew_picks, [0.6; 7]);
        let expected = series_probability(0.6, Format::Bo5);
        assert!((actual - expected).abs() < 1e-12);
    }

    // Uniform ban and pick weights match uniform 5-map subset averaging (`per_map_rating`).
    let wins = [0.85, 0.70, 0.60, 0.55, 0.45, 0.35, 0.20];
    let uniform_expected = map_series_probability(&wins, Format::Bo5).unwrap();
    for order in [ab_ab_ab, ab_ab_ba] {
        let actual = simulate_bo5_veto(order, [[1.0; 7]; 2], [[1.0; 7]; 2], wins);
        assert!(
            (actual - uniform_expected).abs() < 1e-12,
            "uniform Bo5 veto ({actual}) should equal uniform 5-subset average ({uniform_expected})"
        );
    }

    // Anti-symmetry when swapping Team A and Team B.
    for order in [ab_ab_ab, ab_ab_ba] {
        let p_a = simulate_bo5_veto(order, skew_bans, skew_picks, wins);
        let p_b = simulate_bo5_veto(
            order,
            [skew_bans[1], skew_bans[0]],
            [skew_picks[1], skew_picks[0]],
            wins.map(|w| 1.0 - w),
        );
        assert!((p_a + p_b - 1.0).abs() < 1e-12);
    }

    // Team A permabanning its worst map (0.10) raises its Bo5 win probability above uniform;
    // Team B permabanning Team A's best map (0.90) lowers it below uniform.
    let asymmetric_maps = [0.10, 0.90, 0.50, 0.50, 0.50, 0.50, 0.50];
    let baseline = simulate_bo5_veto(ab_ab_ab, [[1.0; 7]; 2], [[1.0; 7]; 2], asymmetric_maps);
    let a_bans_worst = simulate_bo5_veto(
        ab_ab_ab,
        [[40.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0], [1.0; 7]],
        [[1.0; 7]; 2],
        asymmetric_maps,
    );
    let b_bans_best = simulate_bo5_veto(
        ab_ab_ab,
        [[1.0; 7], [1.0, 40.0, 1.0, 1.0, 1.0, 1.0, 1.0]],
        [[1.0; 7]; 2],
        asymmetric_maps,
    );
    assert!(a_bans_worst > baseline + 0.05, "{a_bans_worst} vs {baseline}");
    assert!(b_bans_best < baseline - 0.05, "{b_bans_best} vs {baseline}");
}

fn step(step: u8, team: Option<&str>, action: VetoAction, map_name: &str) -> VetoStep {
    VetoStep {
        step,
        team: team.map(str::to_owned),
        action,
        map_name: map_name.to_owned(),
    }
}

fn map_result(position: u8, map_name: &str, played: bool, winner: Option<Side>) -> MapResult {
    MapResult {
        position,
        map_name: map_name.to_owned(),
        mapstats_id: None,
        team_a_score: played.then_some(if winner == Some(Side::A) { 13 } else { 8 }),
        team_b_score: played.then_some(if winner == Some(Side::B) { 13 } else { 8 }),
        winner,
        half_scores: None,
        played,
    }
}

/// Real Budapest Major 2025 Bo5 Grand Final metadata (PandaScore 1269335: Vitality 3-1 FaZe).
fn budapest_bo5_record() -> HltvMatchRecord {
    HltvMatchRecord {
        match_id: 2388130,
        url: "https://www.hltv.org/matches/2388130/vitality-vs-faze-starladder-budapest-major-2025"
            .into(),
        fetched_at: "2026-11-26T17:41:38Z".parse::<DateTime<Utc>>().unwrap(),
        event_id: Some(8042),
        event_name: Some("StarLadder Budapest Major 2025".into()),
        started_at: "2025-12-14T17:00:00Z".parse::<DateTime<Utc>>().unwrap(),
        format: "bo5".into(),
        is_lan: Some(true),
        team_a: TeamRef {
            id: 9565,
            name: "Vitality".into(),
        },
        team_b: TeamRef {
            id: 6667,
            name: "FaZe".into(),
        },
        maps: vec![
            map_result(1, "Nuke", true, Some(Side::B)),
            map_result(2, "Dust2", true, Some(Side::A)),
            map_result(3, "Inferno", true, Some(Side::A)),
            map_result(4, "Overpass", true, Some(Side::A)),
            map_result(5, "Mirage", false, None),
        ],
        veto: vec![
            step(1, Some("FaZe"), VetoAction::Removed, "Train"),
            step(2, Some("Vitality"), VetoAction::Removed, "Ancient"),
            step(3, Some("FaZe"), VetoAction::Picked, "Nuke"),
            step(4, Some("Vitality"), VetoAction::Picked, "Dust2"),
            step(5, Some("FaZe"), VetoAction::Picked, "Inferno"),
            step(6, Some("Vitality"), VetoAction::Picked, "Overpass"),
            step(7, None, VetoAction::LeftOver, "Mirage"),
        ],
        overall_stats: vec![],
        per_map_stats: vec![],
    }
}

/// Budapest Playoffs QF Bo3 record matching PandaScore 1269331 (FURIA 1-2 Natus Vincere)
/// so Stage 17900 alone supplies both a prior Bo5 veto and FURIA veto habits.
fn budapest_qf_furia_bo3_record() -> HltvMatchRecord {
    HltvMatchRecord {
        match_id: 2388126,
        url: "https://www.hltv.org/matches/2388126/furia-vs-natus-vincere-starladder-budapest-major-2025"
            .into(),
        fetched_at: "2026-11-26T17:41:00Z".parse::<DateTime<Utc>>().unwrap(),
        event_id: Some(8042),
        event_name: Some("StarLadder Budapest Major 2025".into()),
        started_at: "2025-12-12T19:10:00Z".parse::<DateTime<Utc>>().unwrap(),
        format: "bo3".into(),
        is_lan: Some(true),
        team_a: TeamRef {
            id: 8297,
            name: "FURIA".into(),
        },
        team_b: TeamRef {
            id: 4608,
            name: "Natus Vincere".into(),
        },
        maps: vec![
            map_result(1, "Mirage", true, Some(Side::B)),
            map_result(2, "Inferno", true, Some(Side::A)),
            map_result(3, "Train", true, Some(Side::B)),
        ],
        veto: vec![
            step(1, Some("FURIA"), VetoAction::Removed, "Ancient"),
            step(2, Some("Natus Vincere"), VetoAction::Removed, "Overpass"),
            step(3, Some("FURIA"), VetoAction::Picked, "Inferno"),
            step(4, Some("Natus Vincere"), VetoAction::Picked, "Mirage"),
            step(5, Some("FURIA"), VetoAction::Removed, "Dust2"),
            step(6, Some("Natus Vincere"), VetoAction::Removed, "Nuke"),
            step(7, None, VetoAction::LeftOver, "Train"),
        ],
        overall_stats: vec![],
        per_map_stats: vec![],
    }
}

/// Cologne Playoffs QF Bo3 record matching PandaScore 1513154 (FURIA 2-1 9z) so FURIA has
/// pre-final veto habits before the Cologne Bo5 Grand Final (1513171).
fn cologne_qf_furia_bo3_record() -> HltvMatchRecord {
    HltvMatchRecord {
        match_id: 2394995,
        url: "https://www.hltv.org/matches/2394995/furia-vs-9z-iem-cologne-major-2026".into(),
        fetched_at: "2026-11-26T17:16:00Z".parse::<DateTime<Utc>>().unwrap(),
        event_id: Some(8240),
        event_name: Some("IEM Cologne Major 2026".into()),
        started_at: "2026-06-18T17:00:00Z".parse::<DateTime<Utc>>().unwrap(),
        format: "bo3".into(),
        is_lan: Some(true),
        team_a: TeamRef {
            id: 8297,
            name: "FURIA".into(),
        },
        team_b: TeamRef {
            id: 9996,
            name: "9z".into(),
        },
        maps: vec![
            map_result(1, "Inferno", true, Some(Side::A)),
            map_result(2, "Nuke", true, Some(Side::B)),
            map_result(3, "Mirage", true, Some(Side::A)),
        ],
        veto: vec![
            step(1, Some("FURIA"), VetoAction::Removed, "Ancient"),
            step(2, Some("9z"), VetoAction::Removed, "Overpass"),
            step(3, Some("FURIA"), VetoAction::Picked, "Inferno"),
            step(4, Some("9z"), VetoAction::Picked, "Nuke"),
            step(5, Some("FURIA"), VetoAction::Removed, "Anubis"),
            step(6, Some("9z"), VetoAction::Removed, "Dust2"),
            step(7, None, VetoAction::LeftOver, "Mirage"),
        ],
        overall_stats: vec![],
        per_map_stats: vec![],
    }
}

/// Cologne Playoffs Bo5 Grand Final record matching PandaScore 1513171 (FURIA 0-3 Team Falcons).
fn cologne_bo5_record() -> HltvMatchRecord {
    HltvMatchRecord {
        match_id: 2395002,
        url: "https://www.hltv.org/matches/2395002/furia-vs-falcons-iem-cologne-major-2026".into(),
        fetched_at: "2026-11-26T17:16:21Z".parse::<DateTime<Utc>>().unwrap(),
        event_id: Some(8240),
        event_name: Some("IEM Cologne Major 2026".into()),
        started_at: "2026-06-21T15:00:00Z".parse::<DateTime<Utc>>().unwrap(),
        format: "bo5".into(),
        is_lan: Some(true),
        team_a: TeamRef {
            id: 8297,
            name: "FURIA".into(),
        },
        team_b: TeamRef {
            id: 11283,
            name: "Falcons".into(),
        },
        maps: vec![
            map_result(1, "Mirage", true, Some(Side::B)),
            map_result(2, "Anubis", true, Some(Side::B)),
            map_result(3, "Inferno", true, Some(Side::B)),
            map_result(4, "Dust2", false, None),
            map_result(5, "Nuke", false, None),
        ],
        veto: vec![
            step(1, Some("FURIA"), VetoAction::Removed, "Ancient"),
            step(2, Some("Falcons"), VetoAction::Removed, "Overpass"),
            step(3, Some("FURIA"), VetoAction::Picked, "Mirage"),
            step(4, Some("Falcons"), VetoAction::Picked, "Anubis"),
            step(5, Some("FURIA"), VetoAction::Picked, "Inferno"),
            step(6, Some("Falcons"), VetoAction::Picked, "Dust2"),
            step(7, None, VetoAction::LeftOver, "Nuke"),
        ],
        overall_stats: vec![],
        per_map_stats: vec![],
    }
}

fn heroic_sharks_bo1_record() -> HltvMatchRecord {
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

fn vitality_fut_bo3_record() -> HltvMatchRecord {
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

fn playoffs_archive_with_bo5_vetoes() -> tempfile::TempDir {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let dir = tempfile::tempdir().unwrap();
    for name in [
        "panda-17900.json.gz",
        "panda-20710.json.gz",
        "standings_global_2025_12_01.md.gz",
        "standings_global_2026_06_01.md.gz",
    ] {
        std::fs::copy(root.join("backtest").join(name), dir.path().join(name)).unwrap();
    }
    std::fs::copy(
        root.join("research/liquipedia-cologne-playoffs-postmatch.json"),
        dir.path().join("maps-20710.json"),
    )
    .unwrap();
    let matches_dir = dir.path().join("hltv/matches");
    std::fs::create_dir_all(&matches_dir).unwrap();
    for record in [
        budapest_qf_furia_bo3_record(),
        budapest_bo5_record(),
        cologne_qf_furia_bo3_record(),
        cologne_bo5_record(),
    ] {
        std::fs::write(
            matches_dir.join(format!("hltv-match-{}.json", record.match_id)),
            serde_json::to_vec(&record).unwrap(),
        )
        .unwrap();
    }
    dir
}

#[tokio::test]
async fn bo5_veto_sim_and_veto_sim_are_registered_marked_post_hoc_and_not_testable_when_missing() {
    for name in ["bo5_veto_sim", "veto_sim"] {
        let signal = signals::by_name(name).unwrap_or_else(|| panic!("{name} is registered"));
        assert!(signal.added_after_list_fixed(), "{name} must be post-hoc");
    }
    assert!(!signals::by_name("bo1_veto_sim").unwrap().added_after_list_fixed());
    assert!(!signals::by_name("per_map_rating").unwrap().added_after_list_fixed());

    // Archive without any HLTV vetoes: both report `not testable` with reason and post-hoc mark.
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let dir = tempfile::tempdir().unwrap();
    for name in ["panda-20710.json.gz", "standings_global_2026_06_01.md.gz"] {
        std::fs::copy(root.join("backtest").join(name), dir.path().join(name)).unwrap();
    }
    let archive = ReplayArchive::load(dir.path()).unwrap();
    let mut lab = Lab::new(archive).unwrap();
    lab.signals = ["per_map_rating", "bo5_veto_sim", "veto_sim"]
        .map(|name| signals::by_name(name).unwrap())
        .into();
    let outcome = lab.run().await.unwrap();

    for name in ["bo5_veto_sim", "veto_sim"] {
        let row = outcome
            .report
            .candidates
            .iter()
            .find(|c| c.name == name)
            .unwrap();
        assert_eq!(row.verdict, Verdict::NotTestable, "{name}");
        assert!(row.reason.contains("HLTV"), "{name}: {}", row.reason);
        assert!(row.added_after_list_fixed, "{name}");
    }
    assert!(!outcome.report.candidates[0].added_after_list_fixed);

    let json = serde_json::to_value(&outcome.report).unwrap();
    for name in ["bo5_veto_sim", "veto_sim"] {
        let entry = json["candidates"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["name"] == name)
            .unwrap();
        assert_eq!(entry["added_after_list_fixed"], true, "{name}");
    }

    let md = outcome.report.markdown();
    assert!(
        md.contains("not testable (added after list fixed)"),
        "markdown must mark post-hoc signals next to verdict:\n{md}"
    );
}

#[tokio::test]
async fn bo5_veto_uses_earlier_saved_veto_never_its_own_and_respects_finish_and_holdout() {
    let dir = playoffs_archive_with_bo5_vetoes();
    let path = dir.path();

    // 1) Strict finish boundary on the prior Bo5 veto + Holdout exclusion + never reads its own veto.
    let run_veto_boundary = |shift: Option<Duration>, prior_is_holdout: bool| async move {
        let mut archive = ReplayArchive::load(path).unwrap();
        let budapest_finish = archive
            .stages
            .iter()
            .find(|s| s.id == 17900)
            .unwrap()
            .matches
            .iter()
            .find(|m| m.game.id == 1269335)
            .unwrap()
            .reported_finish
            .unwrap();

        if let Some(shift) = shift {
            // Move Cologne Bo5 Grand Final (1513171) right across Budapest Bo5 finish (1269335).
            // FURIA already has veto history from Budapest QF (1269331, finished 2025-12-12),
            // so the only gate at `budapest_finish` is the prior Bo5 veto itself.
            let cologne = archive.stages.iter_mut().find(|s| s.id == 20710).unwrap();
            let gf = cologne
                .matches
                .iter_mut()
                .find(|m| m.game.id == 1513171)
                .unwrap();
            let start = budapest_finish + Duration::minutes(15) + shift;
            gf.actual_start = start;
            gf.game.scheduled_start = Some(start);
            assert_eq!(gf.cutoff(), budapest_finish + shift);
        }

        let mut lab = Lab::new(archive).unwrap();
        if prior_is_holdout {
            // Put Budapest Playoffs (17900, the only prior Bo5 veto) in the Holdout while
            // keeping Cologne Playoffs (20710) on the training side.
            lab.holdout.from = budapest_finish - Duration::days(2);
            lab.holdout.training_stages = vec![20710];
        }
        lab.signals = vec![signals::by_name("bo5_veto_sim").unwrap()];
        lab.run().await.unwrap()
    };

    let normal = run_veto_boundary(None, false).await;
    let at_finish = run_veto_boundary(Some(Duration::zero()), false).await;
    let after_finish = run_veto_boundary(Some(Duration::seconds(1)), false).await;
    let excluded = run_veto_boundary(Some(Duration::seconds(1)), true).await;

    let prediction = |outcome: &LabOutcome, id: i64| {
        outcome
            .predictions
            .iter()
            .find(|p| p.signal == "bo5_veto_sim" && p.match_id == id)
            .unwrap()
            .variant
    };

    // First Bo5 (1269335) never reads its own veto.
    assert_eq!(prediction(&normal, 1269335), None);
    // Strict finish boundary on the prior Bo5 veto: at finish is None; 1s after is Some.
    assert_eq!(prediction(&at_finish, 1513171), None);
    let p_after = prediction(&after_finish, 1513171).expect("Cologne Bo5 predicted after prior Bo5");
    assert!((0.0..=1.0).contains(&p_after));
    // Holdout exclusion: prior Bo5 in Holdout never reaches training history.
    assert_eq!(prediction(&excluded, 1513171), None);

    // Normal archive run reports coverage, differences, intervals, verdict, and post-hoc mark.
    let p_normal = prediction(&normal, 1513171).expect("Cologne Bo5 predicted normally");
    assert!((0.0..=1.0).contains(&p_normal));
    assert!(
        (p_normal - 0.5).abs() > 1e-4,
        "prediction should reflect veto & map Elo, not flat 0.5: {p_normal}"
    );
    let row = &normal.report.candidates[0];
    assert_eq!(row.name, "bo5_veto_sim");
    assert_eq!(row.computed_on, 1);
    assert_eq!(row.paired, 1);
    assert!(row.difference.is_some());
    assert!(row.interval.is_some());
    assert!(row.accuracy_interval.is_some());
    assert_eq!(row.verdict, Verdict::NoEvidence);
    assert!(row.added_after_list_fixed);
    let md = normal.report.markdown();
    assert!(md.contains("no evidence (added after list fixed)"), "{md}");

    // 2) Strict finish boundary + Holdout exclusion for Liquipedia map results feeding bo5_veto_sim.
    // Semifinal 1513160 (Spirit vs Team Falcons) has Liquipedia played maps for Team Falcons.
    // Put prior vetoes exclusively in Stage 17900 so Stage 20710 can be placed in the Holdout
    // to isolate its Liquipedia map results cleanly.
    let map_dir = tempfile::tempdir().unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    for name in [
        "panda-17900.json.gz",
        "panda-20710.json.gz",
        "standings_global_2025_12_01.md.gz",
        "standings_global_2026_06_01.md.gz",
    ] {
        std::fs::copy(root.join("backtest").join(name), map_dir.path().join(name)).unwrap();
    }
    std::fs::copy(
        root.join("research/liquipedia-cologne-playoffs-postmatch.json"),
        map_dir.path().join("maps-20710.json"),
    )
    .unwrap();
    let map_matches = map_dir.path().join("hltv/matches");
    std::fs::create_dir_all(&map_matches).unwrap();
    for record in [budapest_bo5_record(), budapest_qf_furia_bo3_record()] {
        std::fs::write(
            map_matches.join(format!("hltv-match-{}.json", record.match_id)),
            serde_json::to_vec(&record).unwrap(),
        )
        .unwrap();
    }
    let map_path = map_dir.path();
    let run_map_boundary = |shift: Duration, maps_in_holdout: bool| async move {
        let mut archive = ReplayArchive::load(map_path).unwrap();
        let cologne = archive.stages.iter_mut().find(|s| s.id == 20710).unwrap();
        let sf_finish = cologne
            .matches
            .iter()
            .find(|m| m.game.id == 1513160)
            .unwrap()
            .reported_finish
            .unwrap();
        let gf_idx = cologne
            .matches
            .iter()
            .position(|m| m.game.id == 1513171)
            .unwrap();
        let mut gf = cologne.matches.remove(gf_idx);
        let start = sf_finish + Duration::minutes(15) + shift;
        gf.actual_start = start;
        gf.game.scheduled_start = Some(start);
        gf.finished_at = start + Duration::hours(3);
        gf.reported_finish = Some(gf.finished_at);
        assert_eq!(gf.cutoff(), sf_finish + shift);

        let mut training = cologne.clone();
        training.id = 88054;
        training.matches = vec![gf];
        archive.stages.push(training);

        let mut lab = Lab::new(archive).unwrap();
        if maps_in_holdout {
            lab.holdout.from = sf_finish - Duration::days(4);
            lab.holdout.training_stages = vec![17900, 88054];
        }
        lab.signals = vec![signals::by_name("bo5_veto_sim").unwrap()];
        let out = lab.run().await.unwrap();
        out.predictions
            .iter()
            .find(|p| p.signal == "bo5_veto_sim" && p.match_id == 1513171)
            .unwrap()
            .variant
            .unwrap()
    };

    let before_cologne_maps = run_map_boundary(Duration::days(-5), false).await;
    let before_sf = run_map_boundary(Duration::seconds(-1), false).await;
    let at_sf = run_map_boundary(Duration::zero(), false).await;
    let after_sf = run_map_boundary(Duration::seconds(1), false).await;
    let holdout_sf = run_map_boundary(Duration::seconds(1), true).await;
    assert_eq!(before_sf, at_sf);
    assert_ne!(
        at_sf, after_sf,
        "Team Falcons' semifinal Liquipedia maps must update Elo only strictly after finish"
    );
    assert_eq!(
        holdout_sf, before_cologne_maps,
        "Holdout Liquipedia maps must never reach bo5_veto_sim training history"
    );
}

#[tokio::test]
async fn veto_sim_matches_bo1_bo3_and_bo5_signals_exactly_for_each_format() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let dir = tempfile::tempdir().unwrap();
    for name in [
        "panda-17900.json.gz",
        "panda-20708.json.gz",
        "panda-21115.json.gz",
        "panda-20710.json.gz",
        "standings_global_2025_12_01.md.gz",
        "standings_global_2026_06_01.md.gz",
    ] {
        std::fs::copy(root.join("backtest").join(name), dir.path().join(name)).unwrap();
    }
    std::fs::copy(
        root.join("research/liquipedia-cologne-playoffs-postmatch.json"),
        dir.path().join("maps-20710.json"),
    )
    .unwrap();
    let matches_dir = dir.path().join("hltv/matches");
    std::fs::create_dir_all(&matches_dir).unwrap();
    for record in [
        heroic_sharks_bo1_record(),
        vitality_fut_bo3_record(),
        budapest_bo5_record(),
        cologne_qf_furia_bo3_record(),
        cologne_bo5_record(),
    ] {
        std::fs::write(
            matches_dir.join(format!("hltv-match-{}.json", record.match_id)),
            serde_json::to_vec(&record).unwrap(),
        )
        .unwrap();
    }

    let archive = ReplayArchive::load(dir.path()).unwrap();
    let formats: BTreeMap<i64, Format> = archive
        .stages
        .iter()
        .flat_map(|s| &s.matches)
        .map(|m| (m.game.id, m.game.format))
        .collect();

    let mut lab = Lab::new(archive).unwrap();
    lab.signals = ["bo1_veto_sim", "bo3_veto_sim", "bo5_veto_sim", "veto_sim"]
        .map(|name| signals::by_name(name).unwrap())
        .into();
    let outcome = lab.run().await.unwrap();

    let by_signal_match: BTreeMap<(&str, i64), Option<f64>> = outcome
        .predictions
        .iter()
        .map(|p| ((p.signal.as_str(), p.match_id), p.variant))
        .collect();

    let mut counts = BTreeMap::<&str, usize>::new();
    for (&match_id, &format) in &formats {
        let expected_signal = match format {
            Format::Bo1 => "bo1_veto_sim",
            Format::Bo3 => "bo3_veto_sim",
            Format::Bo5 => "bo5_veto_sim",
        };
        let expected = by_signal_match[&(expected_signal, match_id)];
        let actual = by_signal_match[&("veto_sim", match_id)];
        assert_eq!(
            actual, expected,
            "veto_sim must match {expected_signal} exactly on {format:?} Match {match_id}"
        );
        if actual.is_some() {
            *counts.entry(expected_signal).or_default() += 1;
        }
    }
    assert!(counts.get("bo1_veto_sim").copied().unwrap_or(0) > 0);
    assert!(counts.get("bo3_veto_sim").copied().unwrap_or(0) > 0);
    assert!(counts.get("bo5_veto_sim").copied().unwrap_or(0) > 0);

    let row = |name: &str| {
        outcome
            .report
            .candidates
            .iter()
            .find(|c| c.name == name)
            .unwrap()
    };
    let bo1 = row("bo1_veto_sim");
    let bo3 = row("bo3_veto_sim");
    let bo5 = row("bo5_veto_sim");
    let all = row("veto_sim");

    assert_eq!(all.computed_on, bo1.computed_on + bo3.computed_on + bo5.computed_on);
    assert_eq!(all.paired, bo1.paired + bo3.paired + bo5.paired);
    assert!(all.difference.is_some());
    assert!(all.interval.is_some());
    assert!(all.accuracy_interval.is_some());
    assert!(all.added_after_list_fixed);
    let md = outcome.report.markdown();
    assert!(md.contains("`veto_sim`"), "{md}");
    assert!(md.contains("(added after list fixed)"), "{md}");
}
