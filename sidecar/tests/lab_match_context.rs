use kallbet_ai::adapters::{
    backtest::{
        ReplayArchive, ReplayHistory,
        match_context::{Environment, Region},
    },
    lab::{
        Lab,
        signals::match_context::{rest_schedule_load::team_load, series_format::format_rates, stakes::swiss_record},
    },
};
use kallbet_ai::{ResultsHistory, team_names::team_key};
use std::path::Path;

const SIGNALS: [&str; 5] = [
    "rest_schedule_load",
    "stakes",
    "lan_online",
    "series_format",
    "region_travel",
];

#[tokio::test]
async fn context_candidates_are_reported_by_the_lab() {
    let archive =
        ReplayArchive::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/backtest"))
            .unwrap();
    let mut lab = Lab::new(archive).unwrap();

    lab.signals.retain(|s| SIGNALS.contains(&s.name()));
    let run = lab.run().await.unwrap();
    for name in SIGNALS {
        let candidate = run
            .report
            .candidates
            .iter()
            .find(|c| c.name == name)
            .unwrap_or_else(|| panic!("missing {name}"));
        assert!(!candidate.reason.is_empty());
        if candidate.paired > 0 {
            assert!(candidate.difference.is_some(), "{name}");
            assert!(candidate.interval.is_some(), "{name}");
        }
    }
    for name in ["rest_schedule_load", "stakes", "series_format"] {
        assert!(
            run.report
                .candidates
                .iter()
                .any(|c| c.name == name && c.paired > 0),
            "{name}"
        );
    }
}


fn archive() -> ReplayArchive {
    ReplayArchive::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/backtest"))
        .unwrap()
}

/// Cologne 2026 Stage 1 (stage 20708). Hand-checked against the archive's
/// results: Swiss records are the results of rounds before the Match's own.
#[tokio::test]
async fn swiss_records_use_only_earlier_rounds() {
    let archive = archive();
    let history = ReplayHistory::new(&archive);
    let stage = archive.stages.iter().find(|s| s.id == 20708).unwrap();
    // (match id, round, team A, team B, A record, B record)
    let cases = [
        (1508629, 3, "NRG", "FlyQuest", (1, 1), (1, 1)),
        (1508637, 4, "TYLOO", "Sharks", (1, 2), (1, 2)),
        (1508639, 4, "GamerLegion", "BIG", (2, 1), (2, 1)),
        (1508644, 5, "TYLOO", "Lynn Vision", (2, 2), (2, 2)),
    ];
    for (id, round, a, b, want_a, want_b) in cases {
        let m = stage.matches.iter().find(|m| m.game.id == id).unwrap();
        assert_eq!(m.round, Some(round));
        for (team, want) in [(a, want_a), (b, want_b)] {
            let slice = history.team_matches(team, m.cutoff()).await.unwrap();
            let past = slice.at(m.cutoff()).unwrap();
            assert_eq!(
                swiss_record(&team_key(team), &past, stage.id, round, m.cutoff()),
                Some(want),
                "{team} before round {round}"
            );
        }
    }
}

#[tokio::test]
async fn rest_and_load_of_hand_checked_matches() {
    let archive = archive();
    let history = ReplayHistory::new(&archive);
    let stage = archive.stages.iter().find(|s| s.id == 20708).unwrap();
    // GamerLegion's round 2: round 1 ended 14:03:43, round 2 starts 18:12:05, same day.
    let m = stage.matches.iter().find(|m| m.game.id == 1508622).unwrap();
    let slice = history.team_matches("GamerLegion", m.cutoff()).await.unwrap();
    let load = team_load(&slice.at(m.cutoff()).unwrap(), m.cutoff(), m.actual_start).unwrap();
    assert!((load.rest_hours - 14_902.0 / 3_600.0).abs() < 1e-9);
    assert_eq!((load.matches, load.maps, load.second_match), (1, 1, true));
    // BetBoom's Bo3 on June 3: two Bo1 Matches the day before, the last ending 20:14:42.
    let m = stage.matches.iter().find(|m| m.game.id == 1508633).unwrap();
    let slice = history.team_matches("BetBoom Team", m.cutoff()).await.unwrap();
    let load = team_load(&slice.at(m.cutoff()).unwrap(), m.cutoff(), m.actual_start).unwrap();
    assert!((load.rest_hours - 67_114.0 / 3_600.0).abs() < 1e-9);
    assert_eq!((load.matches, load.maps, load.second_match), (2, 2, false));
}

#[tokio::test]
async fn series_format_rates_split_bo1_from_bo3() {
    let archive = archive();
    let history = ReplayHistory::new(&archive);
    let stage = archive.stages.iter().find(|s| s.id == 20708).unwrap();
    // TYLOO before round 5: Bo1 L, L; Bo3 W, W (this Event only) -> Laplace-smoothed 1/4 and 3/4.
    let m = stage.matches.iter().find(|m| m.game.id == 1508644).unwrap();
    let slice = history.team_matches("TYLOO", m.cutoff()).await.unwrap();
    let past: Vec<_> = slice
        .at(m.cutoff())
        .unwrap()
        .into_iter()
        .filter(|h| h.event_id == stage.event_id)
        .collect();
    let (bo1, bo3) = format_rates(&team_key("TYLOO"), &past).unwrap();
    assert!((bo1 - 0.25).abs() < 1e-12 && (bo3 - 0.75).abs() < 1e-12);
}

#[test]
fn lan_region_and_unknown_events() {
    let archive = archive();
    let find = |stage_id, id| {
        let stage = archive.stages.iter().find(|s| s.id == stage_id).unwrap();
        let m = stage.matches.iter().find(|m| m.game.id == id).unwrap();
        (archive.match_context.view(stage, m), m.cutoff())
    };
    // Cologne 2026: a LAN in Europe; Sinners are Europe-qualified, FlyQuest Asia-Pacific.
    let (c, cutoff) = find(20708, 1494591);
    assert_eq!(c.environment(cutoff).unwrap().value, Environment::Lan);
    assert_eq!(c.region(cutoff).unwrap().value, Region::Europe);
    assert_eq!(c.team_region("Sinners", cutoff).unwrap().value, Region::Europe);
    assert_eq!(c.team_region("FlyQuest", cutoff).unwrap().value, Region::AsiaPacific);
    // An Event whose type is unknown at the cutoff stays uncovered (Copenhagen 2024 Swiss).
    let (c, cutoff) = find(13056, 930780);
    assert!(c.environment(cutoff).is_none() && c.region(cutoff).is_none());
}
