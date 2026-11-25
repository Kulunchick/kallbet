use chrono::Duration;
use kallbet_ai::{
    adapters::{
        backtest::{self, BlindAnonymizer, replay_match},
        context::liquipedia,
        llm::{self, ToolAccess},
    },
    *,
};
use serde_json::{Value, json};

fn captured_game() -> Match {
    let prompt = include_str!("fixtures/debate/predict-claude.prompt.txt").replace("\r\n", "\n");
    let raw: Value = serde_json::from_str(
        prompt
            .split_once("Match dossier (JSON):\n")
            .unwrap()
            .1
            .lines()
            .next()
            .unwrap(),
    )
    .unwrap();
    serde_json::from_value(raw["game"].clone()).unwrap()
}

#[test]
fn each_feed_is_rechecked_at_the_earlier_of_both_match_cutoffs() {
    let game = captured_game();
    let scheduled = game.scheduled_start.unwrap();
    for actual in [scheduled - Duration::minutes(20), scheduled] {
        let cutoff = actual.min(scheduled - Duration::minutes(15));
        for observed in [cutoff, cutoff + Duration::seconds(1)] {
            let future = cutoff + Duration::days(1);
            let signal = AsOf::new(
                Signal {
                    map: 0.5,
                    series: 0.5,
                },
                observed,
                future,
            )
            .unwrap();
            let odds = AsOf::new(0.5, observed, future).unwrap();
            for (vrs, price, kind) in [
                (Some(signal), None, None),
                (None, Some(odds), None),
                (None, None, Some("liquipedia_roster")),
                (None, None, Some("news_item")),
            ] {
                let facts = kind
                    .into_iter()
                    .map(|k| AsOf::new(json!({"kind":k}), observed, future).unwrap())
                    .collect();
                let err = replay_match(game.clone(), actual, vrs, price, facts)
                    .err()
                    .unwrap();
                let leak = err.downcast_ref::<TemporalLeakError>().unwrap();
                assert_eq!((leak.observed_at, leak.cutoff), (observed, cutoff));
            }
        }
    }
}

#[test]
fn serialized_blind_prompt_has_only_anonymous_rosters_and_reversible_team_sides() {
    let mut game = captured_game();
    let cutoff = game.scheduled_start.unwrap();
    let document =
        include_str!("fixtures/context/liquipedia/pgl-singapore-2026-rev-before-start.json");
    let snapshot = liquipedia::parse_revision_rosters(document, cutoff)
        .unwrap()
        .unwrap();
    let rev = snapshot.at(cutoff).unwrap();
    let mut facts = vec![];
    let mut hidden = vec![rev.page_title.clone(), "2026".into(), "Singapore".into()];
    for (side, t, r) in [
        ("team_a", &mut game.team_a, &rev.teams[0]),
        ("team_b", &mut game.team_b, &rev.teams[1]),
    ] {
        t.name = r.team.clone();
        hidden.push(t.name.clone());
        hidden.extend(r.active_players.clone());
        facts.push(
            ContextFact::LiquipediaRoster(liquipedia::build_roster_fact(rev, r, side))
                .to_value()
                .unwrap(),
        );
    }
    let id = game.team_b.id;
    let dossier = Dossier {
        game,
        signal: None,
        market: None,
        context: facts,
    };
    let (masked, mapping) = BlindAnonymizer::mask(&dossier);
    let wire = llm::prompt::render(
        &masked,
        Tier::PreLock,
        &RolePrompt::adjustment(),
        ToolAccess::None,
    );
    for token in hidden {
        assert!(!wire.contains(&format!("\"{token}\"")), "{token}");
    }
    for token in ["Singapore", "2026"] {
        assert!(!wire.contains(token));
    }
    for token in [
        "TEAM_ALPHA",
        "TEAM_BETA",
        "PLAYER_01",
        "PLAYER_10",
        "https://evidence.invalid/",
    ] {
        assert!(wire.contains(token), "{token}");
    }
    assert_eq!(mapping.winner(masked.game.team_b.id), Some(id));
    assert_eq!(mapping.winner(3), None);
}

#[tokio::test]
async fn completed_stage_replay_aborts_if_a_match_result_reaches_the_opening_view() {
    let archive = tempfile::tempdir().unwrap();
    let saved: Vec<Value> = serde_json::from_str(include_str!(
        "../../tests/fixtures/pandascore/real-major-stage1-matches.json"
    ))
    .unwrap();
    let played = &saved[0];
    let cutoff = played["begin_at"].as_str().unwrap();
    let snapshot = json!({"observed":"2026-05-01T00:00:00Z","payload":{"stage":{"kind":"swiss"},"teams":[],
        "groups":[{"matches":[{"winner":played["winner_id"]}]}]}});
    let stage = json!({"id":played["tournament_id"],"name":played["tournament"]["name"],"cutoff":cutoff,
        "complete":true,"teams":[],"matches":[],"view":snapshot});
    std::fs::write(
        archive.path().join("stages.json"),
        json!([stage]).to_string(),
    )
    .unwrap();
    let err = backtest::run(archive.path(), false, false)
        .await
        .unwrap_err();
    assert!(err.is::<TemporalLeakError>(), "{err}");
}

#[test]
fn backtest_subcommand_reads_its_archive_without_live_credentials() {
    let result = std::process::Command::new(env!("CARGO_BIN_EXE_kallbet-ai"))
        .args(["backtest", "missing-replay-directory"])
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("stages.json"));
}

#[tokio::test]
async fn real_cologne_bo1_scores_the_vrs_prediction_and_reports_market_abstention() {
    let archive = tempfile::tempdir().unwrap();
    let saved: Vec<Value> = serde_json::from_str(include_str!(
        "../../tests/fixtures/pandascore/real-major-stage1-matches.json"
    ))
    .unwrap();
    let row = saved.iter().find(|r| r["id"] == 1494593).unwrap();
    let a = &row["opponents"][0]["opponent"];
    let b = &row["opponents"][1]["opponent"];
    let stage = json!({"name":"Cologne Stage 1","id":row["tournament_id"],"complete":false,
        "cutoff":row["begin_at"],"view":null,
        "teams":[{"id":a["id"],"rating_name":a["name"]},{"id":b["id"],"rating_name":b["name"]}],
        "matches":[{"game":{"id":row["id"],"event_id":row["serie_id"],"format":"bo1","team_a":a,"team_b":b,"scheduled_start":row["scheduled_at"]},
            "actual":row["begin_at"],"winner":row["winner_id"]}]});
    std::fs::write(
        archive.path().join("stages.json"),
        json!([stage]).to_string(),
    )
    .unwrap();
    std::fs::write(
        archive.path().join("standings_global_2026_06_01.md"),
        include_bytes!("fixtures/backtest/standings_global_2026_06_01.md"),
    )
    .unwrap();
    let report = backtest::run(archive.path(), false, false).await.unwrap();
    assert_eq!(report["stages"][0]["coverage"], json!({"VrsOnly":1}));
    let bo1 = report["scores"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["format"] == "bo1")
        .unwrap();
    assert_eq!(bo1["metrics"]["correct"], 0); // Heroic (1420) lost to Sharks (1354).
    assert!(bo1["metrics"]["brier"].as_f64().unwrap() > 0.25);
}
