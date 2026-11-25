mod support;

use async_trait::async_trait;
use chrono::Duration;
use kallbet::{
    domain::{EventId, MatchFormat, MatchId, StageKind, TeamId},
    poller,
    testing::{EventSnapshotBuilder, TEST_BOT_SECRET},
};
use kallbet_ai::{
    AsOf, Config, Instant, Match as AiMatch, OddsSource, Result as AiResult, Sidecar,
    TemporalLeakError,
    adapters::pickem::{self, MonteCarloPickemSolver},
};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use support::Harness;

#[derive(Default)]
struct PairOdds {
    by_pair: Mutex<HashMap<(String, String), AsOf<f64>>>,
}

impl PairOdds {
    fn set(&self, team_a: &str, team_b: &str, p_a: f64, observed_at: Instant, cutoff: Instant) {
        let mut map = self.by_pair.lock().unwrap();
        map.insert(
            (team_a.to_lowercase(), team_b.to_lowercase()),
            AsOf::new(p_a, observed_at, cutoff).unwrap(),
        );
    }
}

#[async_trait]
impl OddsSource for PairOdds {
    async fn quote(&self, game: &AiMatch, _cutoff: Instant) -> AiResult<Option<AsOf<f64>>> {
        let map = self.by_pair.lock().unwrap();
        let a = game.team_a.name.to_lowercase();
        let b = game.team_b.name.to_lowercase();
        if let Some(q) = map.get(&(a.clone(), b.clone())) {
            return Ok(Some(q.clone()));
        }
        if let Some(q) = map.get(&(b, a)) {
            let obs = q.observed_at();
            let future = obs + Duration::days(365);
            let p_rev = 1.0 - *q.at(future).unwrap();
            return Ok(Some(AsOf::new(p_rev, obs, future).unwrap()));
        }
        Ok(None)
    }
}

const SWISS_16: [&str; 16] = [
    "MOUZ",
    "Vitality",
    "FURIA",
    "Aurora",
    "Spirit",
    "BETBOOM",
    "Falcons",
    "FUT",
    "Legacy",
    "Nemiga",
    "G2",
    "1win",
    "Natus Vincere",
    "Astralis",
    "Ninjas in Pyjamas",
    "PARIVISION",
];

#[tokio::test]
async fn swiss_stage_submits_refreshes_once_in_prelock_window_stops_after_lock_and_scores_points() {
    let h = Harness::new().await;
    let app = &h.app;
    let t0: Instant = "2026-11-24T00:00:00Z".parse().unwrap();
    app.clock.set(t0);
    let start = t0 + Duration::hours(4);

    let mut builder = EventSnapshotBuilder::new(8302, "PGL Major Singapore 2026")
        .stage(StageKind::Swiss, "Stage 1")
        .format(MatchFormat::Bo1);
    for i in 0..8 {
        builder = builder.swiss_match(
            101 + i as i64,
            (1, 0, 0),
            SWISS_16[i],
            SWISS_16[i + 8],
            start,
        );
    }
    app.source.put_event(builder.build(t0));

    let alice = app.user("Alice").await;
    let league = alice
        .post("/api/leagues", &json!({ "name": "Major Pickem", "event_id": 8302 }))
        .await
        .json();
    let league_id = league["id"].as_i64().unwrap();
    let invite_token = league["invite_token"].as_str().unwrap();
    let invite = format!("{}/join/{invite_token}", h.base);
    poller::tick(&app.state).await;

    let odds = Arc::new(PairOdds::default());
    let mut sidecar = h.sidecar(&invite).await;
    sidecar.ports.odds = odds.clone();
    // Use 4,000 simulations in the HTTP integration loop for fast test execution while exercising full Buchholz MC.
    let solver = Arc::new(MonteCarloPickemSolver::with_simulations(2026, 4_000));
    sidecar.ports.pickem = pickem::with_solver(
        sidecar.ports.signal.clone(),
        sidecar.ports.odds.clone(),
        solver,
    );

    // 1. Initial submission at T - 4h (from VRS ratings).
    h.tick(&mut sidecar, t0).await.unwrap();

    // Human Alice submits and triggers human early Lock (after 5m edit window); bot can still refresh later!
    let human_pickem = json!({
        "three_nil": [9, 10],
        "zero_three": [1, 2],
        "advance": [11, 12, 13, 14, 15, 16]
    });
    assert!(
        alice
            .put(
                &format!("/api/leagues/{league_id}/stages/1/pickem"),
                &human_pickem
            )
            .await
            .status
            .is_success()
    );
    app.clock.advance(Duration::minutes(6));

    let view_after_early = alice
        .get(&format!("/api/leagues/{league_id}/stages/1/pickem"))
        .await
        .json();
    let bot_initial = view_after_early["pickems"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["is_bot"] == true)
        .expect("bot should have submitted initial Swiss Pick'em")
        .clone();
    assert_eq!(bot_initial["shape"], "swiss");
    assert_eq!(bot_initial["preliminary"], true);
    assert_eq!(bot_initial["updated_at"], json!(t0));
    assert!(
        !bot_initial["zero_three"]
            .as_array()
            .unwrap()
            .contains(&json!(1)),
        "top-seeded MOUZ (1) should not initially be in zero_three"
    );

    // 2. Outside T-30m window (at T - 2h), even if odds shift, Pick'em is NOT refreshed yet.
    let t_minus_2h = start - Duration::hours(2);
    for opp in &SWISS_16[1..] {
        odds.set("MOUZ", opp, 0.02, t_minus_2h - Duration::minutes(5), start);
    }
    h.tick(&mut sidecar, t_minus_2h).await.unwrap();
    let check_2h = alice
        .get(&format!("/api/leagues/{league_id}/stages/1/pickem"))
        .await
        .json();
    let bot_at_2h = check_2h["pickems"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["is_bot"] == true)
        .unwrap();
    assert_eq!(bot_at_2h["updated_at"], json!(t0));

    // 3. Inside T-30m window (at T - 20m), changed upstream odds trigger ONE refresh:
    // MOUZ's collapsed 2% win rate moves MOUZ into `zero_three`.
    let t_minus_20m = start - Duration::minutes(20);
    h.tick(&mut sidecar, t_minus_20m).await.unwrap();
    let check_20m = alice
        .get(&format!("/api/leagues/{league_id}/stages/1/pickem"))
        .await
        .json();
    let bot_refreshed = check_20m["pickems"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["is_bot"] == true)
        .unwrap()
        .clone();
    assert_eq!(bot_refreshed["updated_at"], json!(t_minus_20m));
    assert!(
        bot_refreshed["zero_three"]
            .as_array()
            .unwrap()
            .contains(&json!(1)),
        "refreshed Swiss Pick'em should place collapsed MOUZ (1) into zero_three, got {bot_refreshed}"
    );

    // 4. At T - 10m (still before Stage Lock), even if odds change again, bot does NOT refresh a second time.
    let t_minus_10m = start - Duration::minutes(10);
    for opp in &SWISS_16[1..] {
        odds.set("MOUZ", opp, 0.99, t_minus_10m - Duration::minutes(1), start);
    }
    h.tick(&mut sidecar, t_minus_10m).await.unwrap();
    let check_10m = alice
        .get(&format!("/api/leagues/{league_id}/stages/1/pickem"))
        .await
        .json();
    let bot_at_10m = check_10m["pickems"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["is_bot"] == true)
        .unwrap();
    assert_eq!(bot_at_10m["updated_at"], json!(t_minus_20m));

    // 5. At Stage Lock (start) and after (start + 5m), no submission occurs (even for a newly added bot).
    sidecar.bots.push(kallbet_ai::BotProfile {
        bot_key: "late-bot".into(),
        nickname: "LateBot".into(),
        language: None,
        avatar_url: None,
        invite_links: vec![invite.clone()],
        policy: kallbet_ai::Policy::VrsOnly,
        provider_chain: vec![],
    });
    h.tick(&mut sidecar, start + Duration::minutes(5))
        .await
        .unwrap();
    let check_locked = alice
        .get(&format!("/api/leagues/{league_id}/stages/1/pickem"))
        .await
        .json();
    let revealed = check_locked["pickems"].as_array().unwrap();
    assert!(
        !revealed.iter().any(|p| p["nickname"] == "LateBot"),
        "late bot must not submit after Stage Lock"
    );

    // 6. Settle Swiss Stage outcomes so the bot earns Pick'em Points on the Pick'em Leaderboard!
    // Make MOUZ (team 1, in bot's zero_three) finish 0-3, and Vitality (2) & FURIA (3) finish 3-0 / 3-1.
    let mut settled = EventSnapshotBuilder::new(8302, "PGL Major Singapore 2026")
        .stage(StageKind::Swiss, "Stage 1")
        .format(MatchFormat::Bo1);
    for i in 0..8 {
        settled = settled.swiss_match(
            101 + i as i64,
            (1, 0, 0),
            SWISS_16[i],
            SWISS_16[i + 8],
            start,
        );
    }
    settled = settled
        .swiss_match(201, (2, 0, 1), "MOUZ", "Nemiga", start + Duration::hours(5))
        .format(MatchFormat::Bo3)
        .swiss_match(202, (3, 0, 2), "MOUZ", "PARIVISION", start + Duration::hours(10))
        .swiss_match(203, (2, 1, 0), "Vitality", "Aurora", start + Duration::hours(5))
        .swiss_match(204, (3, 2, 0), "Vitality", "Spirit", start + Duration::hours(10));
    app.source.put_event(settled.build(start + Duration::hours(12)));
    let ev = EventId(8302);
    // MOUZ loses all 3 -> 0-3
    app.source.finish(ev, MatchId(101), "Legacy", (1, 0));
    app.source.finish(ev, MatchId(201), "Nemiga", (1, 0));
    app.source.finish(ev, MatchId(202), "PARIVISION", (2, 0));
    // Vitality wins all 3 -> 3-0
    app.source.finish(ev, MatchId(102), "Vitality", (1, 0));
    app.source.finish(ev, MatchId(203), "Vitality", (2, 0));
    app.source.finish(ev, MatchId(204), "Vitality", (2, 0));

    app.clock.set(start + Duration::hours(13));
    poller::tick(&app.state).await;

    let lb = alice
        .get(&format!("/api/leagues/{league_id}/pickem-leaderboard"))
        .await
        .json();
    let bot_entry = lb["entries"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["nickname"] == "Baseline")
        .unwrap();
    assert!(
        bot_entry["points"].as_u64().unwrap() >= 1,
        "bot should earn Pick'em Points on the Pick'em Leaderboard, got {lb}"
    );
    let audited = pickem::submitted_pickems(&sidecar.audit).await.unwrap();
    assert_eq!(audited.len(), 2, "initial submission + 1 pre-Stage refresh recorded in audit DB");
    assert_ne!(audited[0].input_hash, audited[1].input_hash);
}

#[tokio::test]
async fn playoffs_stage_waits_until_ready_submits_refreshes_stops_after_lock_and_scores_points() {
    let h = Harness::new().await;
    let app = &h.app;
    let t0: Instant = "2026-11-24T00:00:00Z".parse().unwrap();
    app.clock.set(t0);
    let start = t0 + Duration::hours(4);
    let semis = start + Duration::days(1);
    let final_time = start + Duration::days(2);

    // Initially QF4 (match 4) is still TBD -> bracket.ready == false!
    let unready = EventSnapshotBuilder::new(9001, "Major Playoffs")
        .stage(StageKind::Playoffs, "Playoffs")
        .playoff_match(1, (2, 0), "MOUZ", "PARIVISION", start)
        .playoff_match(2, (2, 1), "Spirit", "G2", start)
        .playoff_match(3, (2, 2), "FURIA", "9z", start)
        .playoff_slot(4, (2, 3), start)
        .playoff_slot(5, (1, 0), semis)
        .playoff_slot(6, (1, 1), semis)
        .format(MatchFormat::Bo5)
        .playoff_slot(7, (0, 0), final_time);
    app.source.put_event(unready.build(t0));

    let alice = app.user("Alice").await;
    let league = alice
        .post(
            "/api/leagues",
            &json!({ "name": "Playoffs Club", "event_id": 9001 }),
        )
        .await
        .json();
    let league_id = league["id"].as_i64().unwrap();
    let invite_token = league["invite_token"].as_str().unwrap();
    let invite = format!("{}/join/{invite_token}", h.base);
    poller::tick(&app.state).await;

    let odds = Arc::new(PairOdds::default());
    let mut sidecar = h.sidecar(&invite).await;
    sidecar.ports.odds = odds.clone();
    sidecar.ports.pickem = pickem::new(sidecar.ports.signal.clone(), sidecar.ports.odds.clone());

    // 1. While bracket.ready == false, Sidecar::tick waits and submits nothing.
    h.tick(&mut sidecar, t0).await.unwrap();
    let unready_view = alice
        .get(&format!("/api/leagues/{league_id}/stages/1/pickem"))
        .await
        .json();
    assert_eq!(unready_view["bracket"]["ready"], false);
    assert_eq!(unready_view["submitted"], 0);

    // 2. Fill QF4 ("Vitality" vs "Falcons") so bracket.ready becomes true at T - 3h.
    let t_ready = t0 + Duration::hours(1);
    let ready_event = EventSnapshotBuilder::new(9001, "Major Playoffs")
        .stage(StageKind::Playoffs, "Playoffs")
        .playoff_match(1, (2, 0), "MOUZ", "PARIVISION", start)
        .playoff_match(2, (2, 1), "Spirit", "G2", start)
        .playoff_match(3, (2, 2), "FURIA", "9z", start)
        .playoff_match(4, (2, 3), "Vitality", "Falcons", start)
        .playoff_slot(5, (1, 0), semis)
        .playoff_slot(6, (1, 1), semis)
        .format(MatchFormat::Bo5)
        .playoff_slot(7, (0, 0), final_time);
    app.clock.set(t_ready);
    app.source.put_event(ready_event.build(t_ready));
    poller::tick(&app.state).await;

    h.tick(&mut sidecar, t_ready).await.unwrap();

    // Human Alice submits so human early Lock reveals everyone's Pick'em to Alice.
    let alice_picks = json!({
        "picks": [
            { "rounds_before_final": 2, "slot": 0, "team_id": 2 },
            { "rounds_before_final": 2, "slot": 1, "team_id": 4 },
            { "rounds_before_final": 2, "slot": 2, "team_id": 6 },
            { "rounds_before_final": 2, "slot": 3, "team_id": 8 },
            { "rounds_before_final": 1, "slot": 0, "team_id": 2 },
            { "rounds_before_final": 1, "slot": 1, "team_id": 8 },
            { "rounds_before_final": 0, "slot": 0, "team_id": 8 }
        ]
    });
    assert!(
        alice
            .put(
                &format!("/api/leagues/{league_id}/stages/1/pickem"),
                &alice_picks
            )
            .await
            .status
            .is_success()
    );
    app.clock.advance(Duration::minutes(6));

    let view_initial = alice
        .get(&format!("/api/leagues/{league_id}/stages/1/pickem"))
        .await
        .json();
    let bot_initial = view_initial["pickems"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["is_bot"] == true)
        .expect("bot should submit Playoffs Pick'em once bracket is ready")
        .clone();
    assert_eq!(bot_initial["shape"], "playoffs");
    assert_eq!(bot_initial["updated_at"], json!(t_ready));
    let final_winner = |p: &Value| -> i64 {
        p["picks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["rounds_before_final"] == 0 && s["slot"] == 0)
            .unwrap()["team_id"]
            .as_i64()
            .unwrap()
    };
    // MOUZ (id 1, VRS #1 with 1960 pts) is the VRS favourite to win the Final initially.
    assert_eq!(final_winner(&bot_initial), 1);

    // 3. At T - 25m with UNCHANGED inputs, bot does NOT re-submit.
    let t_minus_25m = start - Duration::minutes(25);
    h.tick(&mut sidecar, t_minus_25m).await.unwrap();
    let view_unchanged = alice
        .get(&format!("/api/leagues/{league_id}/stages/1/pickem"))
        .await
        .json();
    let bot_unchanged = view_unchanged["pickems"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["is_bot"] == true)
        .unwrap();
    assert_eq!(bot_unchanged["updated_at"], json!(t_ready));

    // 4. At T - 20m, market odds surge Falcons (id 8) to 95% against everyone -> bot refreshes once!
    let t_minus_20m = start - Duration::minutes(20);
    for opp in [
        "MOUZ",
        "PARIVISION",
        "Spirit",
        "G2",
        "FURIA",
        "9z",
        "Vitality",
    ] {
        odds.set("Falcons", opp, 0.95, t_minus_20m - Duration::minutes(1), start);
    }
    h.tick(&mut sidecar, t_minus_20m).await.unwrap();
    let view_refreshed = alice
        .get(&format!("/api/leagues/{league_id}/stages/1/pickem"))
        .await
        .json();
    let bot_refreshed = view_refreshed["pickems"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["is_bot"] == true)
        .unwrap()
        .clone();
    assert_eq!(bot_refreshed["updated_at"], json!(t_minus_20m));
    assert_eq!(final_winner(&bot_refreshed), 8);

    // 5. Settle the Playoffs bracket (MOUZ, Spirit, FURIA, Falcons win QFs; MOUZ & Falcons win SFs; Falcons wins Final).
    let ev = EventId(9001);
    app.source.finish(ev, MatchId(1), "MOUZ", (2, 0));
    app.source.finish(ev, MatchId(2), "Spirit", (2, 0));
    app.source.finish(ev, MatchId(3), "FURIA", (2, 0));
    app.source.finish(ev, MatchId(4), "Falcons", (2, 0));
    let set_teams = |a: i64, b: i64| {
        move |m: &mut kallbet::domain::Match| {
            m.team_a = Some(TeamId(a));
            m.team_b = Some(TeamId(b));
        }
    };
    app.source.update_match(ev, MatchId(5), set_teams(1, 3));
    app.source.update_match(ev, MatchId(6), set_teams(5, 8));
    app.source.finish(ev, MatchId(5), "MOUZ", (2, 1));
    app.source.finish(ev, MatchId(6), "Falcons", (2, 0));
    app.source.update_match(ev, MatchId(7), set_teams(1, 8));
    app.source.finish(ev, MatchId(7), "Falcons", (3, 1));

    app.clock.set(final_time + Duration::hours(5));
    poller::tick(&app.state).await;

    // Tick after Stage Lock / archive does not error or overwrite.
    h.tick(&mut sidecar, final_time + Duration::hours(5))
        .await
        .unwrap();

    // Bot predicted all 7 slots accurately -> 7 Points on the Pick'em Leaderboard!
    let lb = alice
        .get(&format!("/api/leagues/{league_id}/pickem-leaderboard"))
        .await
        .json();
    let bot_entry = lb["entries"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["nickname"] == "Baseline")
        .unwrap();
    assert_eq!(bot_entry["points"], 7);
    assert_eq!(bot_entry["place"], 1);
}

#[tokio::test]
async fn pickem_tick_rejects_post_cutoff_odds_and_from_config_wires_pickem_step() {
    let h = Harness::new().await;
    let app = &h.app;
    let t0: Instant = "2026-11-24T00:00:00Z".parse().unwrap();
    app.clock.set(t0);
    let start = t0 + Duration::hours(4);
    let semis = start + Duration::days(1);

    let event = EventSnapshotBuilder::new(9050, "Mini Playoffs")
        .stage(StageKind::Playoffs, "Playoffs")
        .playoff_match(1, (1, 0), "MOUZ", "Vitality", start)
        .playoff_match(2, (1, 1), "FURIA", "Spirit", start)
        .playoff_slot(3, (0, 0), semis);
    app.source.put_event(event.build(t0));

    let alice = app.user("Alice").await;
    let league = alice
        .post("/api/leagues", &json!({ "name": "Leak Check", "event_id": 9050 }))
        .await
        .json();
    let league_id = league["id"].as_i64().unwrap();
    let invite_token = league["invite_token"].as_str().unwrap();
    let invite = format!("{}/join/{invite_token}", h.base);
    poller::tick(&app.state).await;

    // Verify Sidecar::from_config wires PickemStep out-of-the-box!
    let dir = tempfile::tempdir().unwrap();
    let cfg_path = dir.path().join("sidecar.json");
    std::fs::write(
        &cfg_path,
        json!({
            "platform_url": h.base,
            "audit_file": dir.path().join("audit.db"),
            "vrs_files": [concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/vrs/standings_global_2026_11_02.md")],
            "bots": [{
                "bot_key": "cfg-bot",
                "nickname": "ConfigBot",
                "invite_links": [invite],
                "policy": "VrsOnly",
                "provider_chain": []
            }]
        })
        .to_string(),
    )
    .unwrap();
    let config = Config::load(&cfg_path).unwrap();
    let mut cfg_sidecar = Sidecar::from_config(config, TEST_BOT_SECRET.into())
        .await
        .unwrap();
    // Replace clock with shared fake clock at T - 10m (after Match Hard Lock at T - 15m, so ONLY PickemStep runs!).
    let mut harness_sidecar = h.sidecar(&format!("{}/join/{invite_token}", h.base)).await;
    cfg_sidecar.ports.clock = harness_sidecar.ports.clock.clone();
    h.tick(&mut cfg_sidecar, start - Duration::minutes(10))
        .await
        .unwrap();

    let view = alice
        .get(&format!("/api/leagues/{league_id}/stages/1/pickem"))
        .await
        .json();
    assert_eq!(
        view["submitted"], 1,
        "Sidecar::from_config must submit Pick'em even at T-10m when Matches are already Hard Locked"
    );

    // Inject post-cutoff odds (observed_at == eval_time) at T - 10m (where only PickemStep runs) -> TemporalLeakError!
    let eval_time = start - Duration::minutes(10);
    harness_sidecar.ports.pickem = pickem::new(
        harness_sidecar.ports.signal.clone(),
        harness_sidecar.ports.odds.clone(),
    );
    *h.scripts.market.lock().unwrap() =
        Some(AsOf::new(0.65, eval_time, eval_time + Duration::minutes(1)).unwrap());
    let err = h.tick(&mut harness_sidecar, eval_time).await.unwrap_err();
    assert!(err.is::<TemporalLeakError>(), "expected TemporalLeakError, got {err}");
}
