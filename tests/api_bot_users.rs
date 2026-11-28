//! Seam 1: Bot Users in a League (spec #21, ticket #22; ADR-0003's
//! amendment). Humans Reveal early among themselves, a Bot User neither holds
//! that up nor takes part in the human final edit window, and it may still
//! predict until Hard Lock while it is shown no revealed Prediction.

use axum::http::StatusCode;
use chrono::{DateTime, Duration, Utc};
use kallbet::domain::{EventId, MatchId, StageKind, UserId};
use kallbet::poller;
use kallbet::ports::{Clock, NotificationKind};
use kallbet::testing::{EventSnapshotBuilder, TestApp, TestClient, TestResponse};
use serde_json::{Value, json};

const EVENT: i64 = 7148;
const NAVI_FAZE: i64 = 101;
const VITALITY_G2: i64 = 102;
// Teams are numbered in order of appearance.
const NAVI: i64 = 1;
const FAZE: i64 = 2;
const VITALITY: i64 = 3;

struct Setup {
    app: TestApp,
    alice: TestClient,
    bob: TestClient,
    /// The Bot User "Oracle", a Member.
    oracle: TestClient,
    league: i64,
    token: String,
    /// Scheduled Start of NAVI vs FaZe.
    start: DateTime<Utc>,
}

/// NAVI vs FaZe four hours from now and Vitality vs G2 a day later.
fn event(app: &TestApp) -> DateTime<Utc> {
    let now = app.clock.now();
    let start = now + Duration::hours(4);
    app.source.put_event(
        EventSnapshotBuilder::new(EVENT, "Test Major")
            .stage(StageKind::Swiss, "Stage 1")
            .swiss_match(NAVI_FAZE, (1, 0, 0), "NAVI", "FaZe", start)
            .swiss_match(
                VITALITY_G2,
                (1, 0, 0),
                "Vitality",
                "G2",
                start + Duration::days(1),
            )
            .build(now),
    );
    start
}

async fn join(client: &TestClient, token: &str) {
    let res = client
        .post_empty(&format!("/api/invites/{token}/join"))
        .await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
}

/// A League of alice (creator) and bob, both human, and the Bot User Oracle,
/// which joined through the Invite Link; polled.
async fn setup() -> Setup {
    let app = TestApp::new().await;
    let start = event(&app);
    let alice = app.user("alice").await;
    let bob = app.user("bob").await;
    let oracle = app.bot("oracle", "Oracle").await;
    let res = alice
        .post(
            "/api/leagues",
            &json!({ "event_id": EVENT, "name": "Friends" }),
        )
        .await;
    assert_eq!(res.status, StatusCode::CREATED, "{}", res.text());
    let league = res.json();
    let token = league["invite_token"].as_str().unwrap().to_string();
    join(&bob, &token).await;
    join(&oracle, &token).await;
    poller::tick(&app.state).await;
    Setup {
        league: league["id"].as_i64().unwrap(),
        token,
        app,
        alice,
        bob,
        oracle,
        start,
    }
}

impl Setup {
    async fn predict(&self, client: &TestClient, game: i64, team: i64) -> TestResponse {
        let res = self.try_predict(client, game, team).await;
        assert_eq!(res.status, StatusCode::OK, "{}", res.text());
        res
    }

    async fn try_predict(&self, client: &TestClient, game: i64, team: i64) -> TestResponse {
        try_predict_in(client, self.league, game, team).await
    }

    async fn card(&self, client: &TestClient, game: i64) -> Value {
        card_in(client, self.league, game).await
    }

    /// alice and bob predict NAVI vs FaZe now and their final edit window
    /// runs out: the humans' early Reveal. Returns its instant.
    async fn humans_reveal_early(&self) -> DateTime<Utc> {
        self.predict(&self.alice, NAVI_FAZE, NAVI).await;
        self.predict(&self.bob, NAVI_FAZE, FAZE).await;
        let revealed_at = self.app.clock.now() + Duration::minutes(5);
        self.app.clock.set(revealed_at);
        let card = self.card(&self.alice, NAVI_FAZE).await;
        assert_eq!(
            card["predictions"]["revealed_at"],
            json!(revealed_at),
            "{card}"
        );
        revealed_at
    }

    /// Moves the clock to `minutes` before the Scheduled Start of NAVI vs
    /// FaZe.
    fn minutes_before_start(&self, minutes: i64) {
        self.app.clock.set(self.start - Duration::minutes(minutes));
    }
}

async fn try_predict_in(client: &TestClient, league: i64, game: i64, team: i64) -> TestResponse {
    client
        .put(
            &format!("/api/leagues/{league}/matches/{game}/prediction"),
            &json!({ "team_id": team }),
        )
        .await
}

async fn matches_text(client: &TestClient, league: i64) -> String {
    let res = client.get(&format!("/api/leagues/{league}/matches")).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    res.text()
}

async fn card_in(client: &TestClient, league: i64, game: i64) -> Value {
    let view: Value = serde_json::from_str(&matches_text(client, league).await).unwrap();
    view["stages"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|s| s["groups"].as_array().unwrap())
        .flat_map(|g| g["matches"].as_array().unwrap())
        .find(|m| m["id"] == game)
        .unwrap()
        .clone()
}

async fn user_id(client: &TestClient) -> i64 {
    client.get("/api/me").await.json()["id"].as_i64().unwrap()
}

/// The revealed Predictions of a card as (nickname, team) pairs, sorted.
fn revealed(card: &Value) -> Vec<(String, i64)> {
    let mut pairs: Vec<(String, i64)> = card["predictions"]["revealed"]
        .as_array()
        .unwrap_or_else(|| panic!("not revealed: {card}"))
        .iter()
        .map(|p| {
            (
                p["nickname"].as_str().unwrap().to_string(),
                p["team_id"].as_i64().unwrap(),
            )
        })
        .collect();
    pairs.sort();
    pairs
}

fn pairs(list: &[(&str, i64)]) -> Vec<(String, i64)> {
    list.iter().map(|(n, t)| (n.to_string(), *t)).collect()
}

/// The revealed Prediction of the Member with this Nickname.
fn pick_of<'a>(card: &'a Value, nickname: &str) -> &'a Value {
    card["predictions"]["revealed"]
        .as_array()
        .unwrap_or_else(|| panic!("not revealed: {card}"))
        .iter()
        .find(|p| p["nickname"] == nickname)
        .unwrap_or_else(|| panic!("no revealed Prediction of {nickname}: {card}"))
}

// --- Membership -----------------------------------------------------------

#[tokio::test]
async fn a_bot_user_joins_through_an_invite_link_and_is_kicked_like_any_member() {
    let s = setup().await;
    let oracle = user_id(&s.oracle).await;

    let league = s.oracle.get(&format!("/api/leagues/{}", s.league)).await;
    assert_eq!(league.status, StatusCode::OK, "{}", league.text());
    let members = league.json()["members"].clone();
    assert!(
        members
            .as_array()
            .unwrap()
            .iter()
            .any(|m| m["user_id"] == oracle),
        "{members}"
    );
    // Joining again changes nothing (a sidecar re-joins on every restart).
    join(&s.oracle, &s.token).await;
    assert_eq!(
        s.card(&s.alice, NAVI_FAZE).await["predictions"]["eligible"],
        3
    );

    let res = s
        .alice
        .delete(&format!("/api/leagues/{}/members/{oracle}", s.league))
        .await;
    assert!(res.status.is_success(), "{}", res.text());

    let res = s
        .oracle
        .get(&format!("/api/leagues/{}/matches", s.league))
        .await;
    assert_eq!(res.status, StatusCode::NOT_FOUND);
    let res = s.try_predict(&s.oracle, NAVI_FAZE, NAVI).await;
    assert_eq!(res.status, StatusCode::NOT_FOUND);
    assert_eq!(
        s.card(&s.alice, NAVI_FAZE).await["predictions"]["eligible"],
        2
    );
}

#[tokio::test]
async fn a_bot_user_has_no_power_over_a_league_it_is_only_a_member_of() {
    let s = setup().await;
    let bob = user_id(&s.bob).await;

    let res = s
        .oracle
        .delete(&format!("/api/leagues/{}/members/{bob}", s.league))
        .await;
    assert_eq!(res.status, StatusCode::FORBIDDEN);
}

// --- Human quorum for the early Reveal ------------------------------------

#[tokio::test]
async fn humans_reveal_early_without_waiting_for_a_bot_user() {
    let s = setup().await;
    s.predict(&s.alice, NAVI_FAZE, NAVI).await;
    let before = s.card(&s.alice, NAVI_FAZE).await;
    assert_eq!(before["lock"]["edit_window"], false);

    // bob is the last human: his Prediction opens the final edit window.
    s.predict(&s.bob, NAVI_FAZE, FAZE).await;
    let end = s.app.clock.now() + Duration::minutes(5);
    let during = s.card(&s.bob, NAVI_FAZE).await;
    assert_eq!(during["lock"]["edit_window"], true, "{during}");
    assert_eq!(during["lock"]["locks_at"], json!(end));
    assert_eq!(during["predictions"]["revealed"], Value::Null);

    s.app.clock.set(end);
    for client in [&s.alice, &s.bob] {
        let card = s.card(client, NAVI_FAZE).await;
        assert_eq!(card["lock"]["locked"], true, "{card}");
        assert_eq!(card["lock"]["locked_at"], json!(end));
        assert_eq!(card["predictions"]["revealed_at"], json!(end));
        assert_eq!(revealed(&card), pairs(&[("alice", NAVI), ("bob", FAZE)]));
    }
}

#[tokio::test]
async fn a_bot_user_that_predicted_does_not_reveal_before_every_human_has() {
    let s = setup().await;
    s.predict(&s.oracle, NAVI_FAZE, NAVI).await;
    s.predict(&s.alice, NAVI_FAZE, NAVI).await;
    s.app.clock.advance(Duration::minutes(30));

    // bob has not predicted: still open, nothing revealed.
    let card = s.card(&s.alice, NAVI_FAZE).await;
    assert_eq!(card["lock"]["locked"], false, "{card}");
    assert_eq!(card["lock"]["edit_window"], false);
    assert_eq!(card["predictions"]["revealed"], Value::Null);
    assert_eq!(card["predictions"]["count"], 2);
    assert_eq!(card["predictions"]["eligible"], 3);
}

#[tokio::test]
async fn a_bot_users_prediction_never_opens_the_final_edit_window() {
    let s = setup().await;
    s.predict(&s.alice, NAVI_FAZE, NAVI).await;
    s.predict(&s.bob, VITALITY_G2, VITALITY).await;

    // Oracle is the last Member without a Prediction for NAVI vs FaZe, but
    // bob, a human, has not predicted it: no window, and alice can still
    // change her mind long after.
    s.predict(&s.oracle, NAVI_FAZE, FAZE).await;
    s.app.clock.advance(Duration::minutes(30));

    let card = s.card(&s.alice, NAVI_FAZE).await;
    assert_eq!(card["lock"]["edit_window"], false, "{card}");
    assert_eq!(card["lock"]["can_predict"], true);
    assert_eq!(card["lock"]["locked"], false);
    s.predict(&s.alice, NAVI_FAZE, FAZE).await;
}

#[tokio::test]
async fn the_last_human_gets_the_final_edit_window_even_when_a_bot_user_predicts_after() {
    let s = setup().await;
    s.predict(&s.alice, NAVI_FAZE, NAVI).await;
    s.predict(&s.bob, NAVI_FAZE, FAZE).await;
    let end = s.app.clock.now() + Duration::minutes(5);

    // Oracle predicts inside bob's window: it neither takes the window over
    // nor is held back by it.
    s.app.clock.advance(Duration::minutes(2));
    s.predict(&s.oracle, NAVI_FAZE, NAVI).await;
    s.predict(&s.oracle, NAVI_FAZE, FAZE).await;

    let alice = s.card(&s.alice, NAVI_FAZE).await;
    assert_eq!(alice["lock"]["can_predict"], false, "{alice}");
    assert_eq!(alice["lock"]["locks_at"], json!(end));
    assert_eq!(
        s.try_predict(&s.alice, NAVI_FAZE, FAZE).await.error_code(),
        "match_locked"
    );
    let bob = s.card(&s.bob, NAVI_FAZE).await;
    assert_eq!(bob["lock"]["can_predict"], true, "{bob}");
    assert_eq!(bob["lock"]["edit_window"], true);
    s.predict(&s.bob, NAVI_FAZE, NAVI).await;

    s.app.clock.set(end);
    let card = s.card(&s.alice, NAVI_FAZE).await;
    assert_eq!(card["predictions"]["revealed_at"], json!(end), "{card}");
    assert_eq!(
        revealed(&card),
        pairs(&[("Oracle", FAZE), ("alice", NAVI), ("bob", NAVI)])
    );
}

#[tokio::test]
async fn kicking_the_last_human_who_has_not_predicted_reveals_despite_a_silent_bot_user() {
    let s = setup().await;
    s.predict(&s.alice, NAVI_FAZE, NAVI).await;
    let bob = user_id(&s.bob).await;

    let res = s
        .alice
        .delete(&format!("/api/leagues/{}/members/{bob}", s.league))
        .await;
    assert!(res.status.is_success(), "{}", res.text());

    let card = s.card(&s.alice, NAVI_FAZE).await;
    assert_eq!(card["lock"]["locked"], true, "{card}");
    assert_eq!(revealed(&card), pairs(&[("alice", NAVI)]));
}

#[tokio::test]
async fn a_league_of_only_bot_users_stays_open_until_hard_lock() {
    let app = TestApp::new().await;
    let start = event(&app);
    let oracle = app.bot("oracle", "Oracle").await;
    let market = app.bot("market", "Market Favourite").await;
    let res = oracle
        .post(
            "/api/leagues",
            &json!({ "event_id": EVENT, "name": "Benchmark" }),
        )
        .await;
    assert_eq!(res.status, StatusCode::CREATED, "{}", res.text());
    let league = res.json()["id"].as_i64().unwrap();
    join(&market, res.json()["invite_token"].as_str().unwrap()).await;
    poller::tick(&app.state).await;

    // Every Member has predicted, and no human is there to Reveal early.
    for (bot, team) in [(&oracle, NAVI), (&market, FAZE)] {
        let res = try_predict_in(bot, league, NAVI_FAZE, team).await;
        assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    }
    app.clock.advance(Duration::hours(1));
    poller::tick(&app.state).await;

    for bot in [&oracle, &market] {
        let card = card_in(bot, league, NAVI_FAZE).await;
        assert_eq!(card["lock"]["locked"], false, "{card}");
        assert_eq!(card["lock"]["can_predict"], true);
        assert_eq!(card["lock"]["edit_window"], false);
        assert_eq!(
            card["lock"]["locks_at"],
            json!(start - Duration::minutes(15))
        );
        assert_eq!(card["predictions"]["revealed"], Value::Null);
    }
    let res = try_predict_in(&market, league, NAVI_FAZE, NAVI).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());

    // Hard Lock: Scheduled Start minus 15 minutes.
    app.clock.set(start - Duration::minutes(15));
    let res = try_predict_in(&oracle, league, NAVI_FAZE, FAZE).await;
    assert_eq!(res.status, StatusCode::CONFLICT);
    assert_eq!(res.error_code(), "match_locked");
    let card = card_in(&oracle, league, NAVI_FAZE).await;
    assert_eq!(card["lock"]["locked"], true, "{card}");
    assert_eq!(card["lock"]["can_predict"], false);
    assert_eq!(
        revealed(&card),
        pairs(&[("Market Favourite", NAVI), ("Oracle", NAVI)])
    );
}

// --- The pre-Hard-Lock exception ------------------------------------------

#[tokio::test]
async fn after_the_human_early_reveal_only_a_bot_user_can_still_predict_until_hard_lock() {
    let s = setup().await;
    s.predict(&s.oracle, NAVI_FAZE, NAVI).await; // the early draft
    s.humans_reveal_early().await; // more than three hours before the start

    // Humans are Locked (ADR-0003).
    for human in [&s.alice, &s.bob] {
        let res = s.try_predict(human, NAVI_FAZE, FAZE).await;
        assert_eq!(res.status, StatusCode::CONFLICT);
        assert_eq!(res.error_code(), "match_locked");
    }

    // 22 minutes before the Scheduled Start the Bot User updates its pick.
    s.minutes_before_start(22);
    let res = s.predict(&s.oracle, NAVI_FAZE, FAZE).await;
    assert_eq!(res.json()["team_id"], FAZE);
    assert_eq!(res.json()["user_id"], user_id(&s.oracle).await);
    assert_eq!(s.card(&s.oracle, NAVI_FAZE).await["my_prediction"], FAZE);
    // Humans stay Locked and see the Bot User's new pick.
    let card = s.card(&s.alice, NAVI_FAZE).await;
    assert_eq!(card["lock"]["locked"], true);
    assert_eq!(card["lock"]["can_predict"], false);
    assert_eq!(
        revealed(&card),
        pairs(&[("Oracle", FAZE), ("alice", NAVI), ("bob", FAZE)])
    );
    assert_eq!(
        s.try_predict(&s.alice, NAVI_FAZE, FAZE).await.error_code(),
        "match_locked"
    );

    // One minute before Hard Lock it still can; at Hard Lock no more.
    s.minutes_before_start(16);
    s.predict(&s.oracle, NAVI_FAZE, NAVI).await;
    s.minutes_before_start(15);
    let res = s.try_predict(&s.oracle, NAVI_FAZE, FAZE).await;
    assert_eq!(res.status, StatusCode::CONFLICT);
    assert_eq!(res.error_code(), "match_locked");
    let card = s.card(&s.oracle, NAVI_FAZE).await;
    assert_eq!(card["my_prediction"], NAVI);
    assert_eq!(card["lock"]["can_predict"], false);
}

#[tokio::test]
async fn a_bot_user_can_place_its_first_prediction_after_the_human_early_reveal() {
    let s = setup().await;
    s.humans_reveal_early().await;
    assert_eq!(
        revealed(&s.card(&s.alice, NAVI_FAZE).await),
        pairs(&[("alice", NAVI), ("bob", FAZE)])
    );

    s.minutes_before_start(25);
    s.predict(&s.oracle, NAVI_FAZE, FAZE).await;

    let card = s.card(&s.bob, NAVI_FAZE).await;
    assert_eq!(
        revealed(&card),
        pairs(&[("Oracle", FAZE), ("alice", NAVI), ("bob", FAZE)])
    );
    assert_eq!(card["predictions"]["count"], 3);
}

#[tokio::test]
async fn a_bot_user_that_joins_after_the_human_early_reveal_can_predict_until_hard_lock() {
    let s = setup().await;
    s.humans_reveal_early().await;
    s.app.clock.advance(Duration::minutes(1));
    let market = s.app.bot("market", "Market Favourite").await;
    let carol = s.app.user("carol").await;
    join(&market, &s.token).await;
    join(&carol, &s.token).await;

    // The human who joined after the Lock is not Eligible and cannot predict.
    let res = s.try_predict(&carol, NAVI_FAZE, NAVI).await;
    assert_eq!(res.error_code(), "match_locked");
    // The Bot User's Lock is the Hard Lock, which has not come.
    s.predict(&market, NAVI_FAZE, NAVI).await;

    let card = s.card(&carol, NAVI_FAZE).await;
    assert_eq!(
        revealed(&card),
        pairs(&[("Market Favourite", NAVI), ("alice", NAVI), ("bob", FAZE)])
    );
    assert_eq!(card["predictions"]["eligible"], 4, "{card}");
}

#[tokio::test]
async fn a_bot_user_that_joins_after_hard_lock_is_not_eligible() {
    let s = setup().await;
    s.minutes_before_start(10);
    poller::tick(&s.app.state).await;
    let market = s.app.bot("market", "Market Favourite").await;
    join(&market, &s.token).await;

    let res = s.try_predict(&market, NAVI_FAZE, NAVI).await;
    assert_eq!(res.error_code(), "match_locked");
    assert_eq!(
        s.card(&market, NAVI_FAZE).await["predictions"]["eligible"],
        3
    );
}

#[tokio::test]
async fn an_early_actual_start_is_hard_lock() {
    let s = setup().await;
    s.humans_reveal_early().await;
    s.app.clock.advance(Duration::minutes(30));
    s.predict(&s.oracle, NAVI_FAZE, NAVI).await;

    // The Match starts hours ahead of its schedule.
    let now = s.app.clock.now();
    s.app.source.start(EventId(EVENT), MatchId(NAVI_FAZE), now);
    s.app.clock.advance(Duration::minutes(6));
    poller::tick(&s.app.state).await;

    let res = s.try_predict(&s.oracle, NAVI_FAZE, FAZE).await;
    assert_eq!(res.status, StatusCode::CONFLICT);
    assert_eq!(res.error_code(), "match_locked");
    assert_eq!(
        s.card(&s.oracle, NAVI_FAZE).await["lock"]["locked_at"],
        json!(now)
    );
}

#[tokio::test]
async fn a_cancellation_is_hard_lock() {
    let s = setup().await;
    s.humans_reveal_early().await;
    s.predict(&s.oracle, NAVI_FAZE, NAVI).await;

    s.app.source.cancel(EventId(EVENT), MatchId(NAVI_FAZE));
    s.app.clock.advance(Duration::minutes(6));
    poller::tick(&s.app.state).await;

    let res = s.try_predict(&s.oracle, NAVI_FAZE, FAZE).await;
    assert_eq!(res.status, StatusCode::CONFLICT);
    assert_eq!(res.error_code(), "match_locked");
}

#[tokio::test]
async fn hard_lock_is_never_undone_by_a_postponement() {
    let s = setup().await;
    s.humans_reveal_early().await;
    s.predict(&s.oracle, NAVI_FAZE, NAVI).await;
    s.minutes_before_start(14);
    poller::tick(&s.app.state).await;
    // By now the Bot User has been shown the humans' Predictions.
    assert_eq!(
        revealed(&s.card(&s.oracle, NAVI_FAZE).await),
        pairs(&[("Oracle", NAVI), ("alice", NAVI), ("bob", FAZE)])
    );

    s.app.source.postpone(
        EventId(EVENT),
        MatchId(NAVI_FAZE),
        s.start + Duration::days(2),
    );
    s.app.clock.advance(Duration::minutes(6));
    poller::tick(&s.app.state).await;

    let res = s.try_predict(&s.oracle, NAVI_FAZE, FAZE).await;
    assert_eq!(res.status, StatusCode::CONFLICT, "{}", res.text());
    assert_eq!(res.error_code(), "match_locked");
    let card = s.card(&s.oracle, NAVI_FAZE).await;
    assert_eq!(card["lock"]["locked"], true, "{card}");
    assert_eq!(card["lock"]["can_predict"], false);
    assert_eq!(
        card["lock"]["locked_at"],
        json!(s.start - Duration::minutes(15))
    );
    assert_eq!(card["predictions"]["revealed"].as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn hard_lock_reopens_on_postponement_when_bot_did_not_predict() {
    let s = setup().await;
    s.humans_reveal_early().await;
    let humans_locked_at = s.card(&s.alice, NAVI_FAZE).await["lock"]["locked_at"].clone();
    s.minutes_before_start(14);
    poller::tick(&s.app.state).await;
    assert_eq!(
        s.try_predict(&s.oracle, NAVI_FAZE, FAZE)
            .await
            .error_code(),
        "match_locked"
    );

    let moved = s.start + Duration::days(2);
    s.app
        .source
        .postpone(EventId(EVENT), MatchId(NAVI_FAZE), moved);
    s.app.clock.advance(Duration::minutes(5));
    poller::tick(&s.app.state).await;

    // Humans stay Locked and Revealed; the Bot User's Hard Lock reopened and
    // it is blind to revealed Picks again until the new Hard Lock.
    assert_eq!(
        s.try_predict(&s.alice, NAVI_FAZE, FAZE)
            .await
            .error_code(),
        "match_locked"
    );
    assert_eq!(
        s.card(&s.alice, NAVI_FAZE).await["lock"]["locked_at"],
        humans_locked_at
    );

    let oracle_card = s.card(&s.oracle, NAVI_FAZE).await;
    assert_eq!(oracle_card["lock"]["locked"], false, "{oracle_card}");
    assert_eq!(oracle_card["lock"]["can_predict"], true, "{oracle_card}");
    assert_eq!(
        oracle_card["lock"]["locks_at"],
        json!(moved - Duration::minutes(15))
    );
    assert_eq!(oracle_card["predictions"]["revealed"], Value::Null);
    s.predict(&s.oracle, NAVI_FAZE, FAZE).await;

    // Once the Bot User has predicted and the new Hard Lock passes, another
    // postponement keeps it Locked.
    s.app.clock.set(moved - Duration::minutes(14));
    poller::tick(&s.app.state).await;
    s.app
        .source
        .postpone(EventId(EVENT), MatchId(NAVI_FAZE), moved + Duration::days(1));
    s.app.clock.advance(Duration::minutes(5));
    poller::tick(&s.app.state).await;
    assert_eq!(
        s.try_predict(&s.oracle, NAVI_FAZE, NAVI)
            .await
            .error_code(),
        "match_locked"
    );
}

#[tokio::test]
async fn a_postponement_before_hard_lock_gives_a_bot_user_the_new_deadline() {
    let s = setup().await;
    s.humans_reveal_early().await;
    let later = s.start + Duration::days(2);
    s.app
        .source
        .postpone(EventId(EVENT), MatchId(NAVI_FAZE), later);
    s.app.clock.advance(Duration::minutes(6));
    poller::tick(&s.app.state).await;

    // Past the old Hard Lock, before the new one.
    s.app.clock.set(s.start);
    s.predict(&s.oracle, NAVI_FAZE, NAVI).await;
    let card = s.card(&s.oracle, NAVI_FAZE).await;
    assert_eq!(
        card["lock"]["locks_at"],
        json!(later - Duration::minutes(15))
    );
    assert_eq!(card["predictions"]["revealed"], Value::Null);
    // The humans' Lock stays (ADR-0003).
    assert_eq!(
        s.try_predict(&s.alice, NAVI_FAZE, FAZE).await.error_code(),
        "match_locked"
    );

    s.app.clock.set(later - Duration::minutes(15));
    assert_eq!(
        s.try_predict(&s.oracle, NAVI_FAZE, FAZE).await.error_code(),
        "match_locked"
    );
}

#[tokio::test]
async fn a_bot_user_can_only_pick_a_team_of_the_match() {
    let s = setup().await;
    s.humans_reveal_early().await;

    let res = s.try_predict(&s.oracle, NAVI_FAZE, VITALITY).await;
    assert_eq!(res.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(res.error_code(), "team_not_in_match");
}

// --- A Bot User is blind until Hard Lock ----------------------------------

#[tokio::test]
async fn before_hard_lock_a_bot_user_is_shown_no_revealed_prediction() {
    let s = setup().await;
    s.predict(&s.oracle, NAVI_FAZE, NAVI).await;
    let revealed_at = s.humans_reveal_early().await;
    let (alice, bob) = (user_id(&s.alice).await, user_id(&s.bob).await);
    // The humans do see everything.
    let human = s.card(&s.bob, NAVI_FAZE).await;
    assert_eq!(human["predictions"]["revealed_at"], json!(revealed_at));
    assert_eq!(
        revealed(&human),
        pairs(&[("Oracle", NAVI), ("alice", NAVI), ("bob", FAZE)])
    );

    for minutes in [200, 22, 16] {
        s.minutes_before_start(minutes);
        let body = matches_text(&s.oracle, s.league).await;
        let card = card_in(&s.oracle, s.league, NAVI_FAZE).await;
        assert_eq!(card["predictions"]["revealed"], Value::Null, "{card}");
        assert_eq!(card["predictions"]["revealed_at"], Value::Null);
        assert_eq!(card["my_prediction"], NAVI);
        // To the Bot User the Match is open until Hard Lock.
        assert_eq!(card["lock"]["can_predict"], true);
        assert_eq!(card["lock"]["locked"], false);
        assert_eq!(card["lock"]["locked_at"], Value::Null);
        assert_eq!(card["lock"]["edit_window"], false);
        assert_eq!(
            card["lock"]["locks_at"],
            json!(s.start - Duration::minutes(15))
        );
        // No human's Prediction is anywhere in the response: not their ids,
        // not their Nicknames.
        for needle in [
            "\"alice\"".to_string(),
            "\"bob\"".to_string(),
            format!("\"user_id\":{alice}"),
            format!("\"user_id\":{bob}"),
        ] {
            assert!(!body.contains(&needle), "{needle} in {body}");
        }
        // Nor does its own Prediction response carry one.
        let res = s.predict(&s.oracle, NAVI_FAZE, NAVI).await;
        assert!(!res.text().contains(&format!("\"user_id\":{alice}")));
        assert!(!res.text().contains(&format!("\"user_id\":{bob}")));
    }

    // At Hard Lock it can no longer change its pick, and sees the others'.
    s.minutes_before_start(15);
    let card = s.card(&s.oracle, NAVI_FAZE).await;
    assert_eq!(card["lock"]["can_predict"], false, "{card}");
    assert_eq!(card["lock"]["locked"], true);
    assert_eq!(
        card["lock"]["locked_at"],
        json!(s.start - Duration::minutes(15))
    );
    assert_eq!(card["predictions"]["revealed_at"], json!(revealed_at));
    assert_eq!(
        revealed(&card),
        pairs(&[("Oracle", NAVI), ("alice", NAVI), ("bob", FAZE)])
    );
    assert_eq!(
        s.try_predict(&s.oracle, NAVI_FAZE, FAZE).await.error_code(),
        "match_locked"
    );
}

#[tokio::test]
async fn a_bot_user_is_blind_to_another_bot_users_prediction_too() {
    let s = setup().await;
    let market = s.app.bot("market", "Market Favourite").await;
    join(&market, &s.token).await;
    s.predict(&market, NAVI_FAZE, FAZE).await;
    s.humans_reveal_early().await;
    let market_id = user_id(&market).await;

    let body = matches_text(&s.oracle, s.league).await;
    assert!(!body.contains("Market Favourite"), "{body}");
    assert!(
        !body.contains(&format!("\"user_id\":{market_id}")),
        "{body}"
    );
}

// --- What humans see of a Bot User ----------------------------------------

#[tokio::test]
async fn a_revealed_bot_users_prediction_is_marked_and_preliminary_until_hard_lock() {
    let s = setup().await;
    s.predict(&s.oracle, NAVI_FAZE, NAVI).await;
    s.humans_reveal_early().await;

    let card = s.card(&s.alice, NAVI_FAZE).await;
    let oracle = pick_of(&card, "Oracle");
    assert_eq!(oracle["is_bot"], true, "{card}");
    assert_eq!(oracle["preliminary"], true);
    for human in ["alice", "bob"] {
        let pick = pick_of(&card, human);
        assert_eq!(pick["is_bot"], false, "{card}");
        assert_eq!(pick["preliminary"], false);
    }

    // From Hard Lock on the Bot User's pick is final.
    s.minutes_before_start(15);
    let card = s.card(&s.alice, NAVI_FAZE).await;
    let oracle = pick_of(&card, "Oracle");
    assert_eq!(oracle["is_bot"], true, "{card}");
    assert_eq!(oracle["preliminary"], false);
}

#[tokio::test]
async fn before_reveal_a_bot_users_prediction_is_hidden_from_humans() {
    let s = setup().await;
    s.predict(&s.oracle, NAVI_FAZE, FAZE).await;
    let oracle = user_id(&s.oracle).await;

    for human in [&s.alice, &s.bob] {
        let body = matches_text(human, s.league).await;
        assert!(!body.contains("Oracle"), "{body}");
        assert!(!body.contains(&format!("\"user_id\":{oracle}")), "{body}");
        let card = card_in(human, s.league, NAVI_FAZE).await;
        assert_eq!(card["predictions"]["revealed"], Value::Null);
        assert_eq!(card["predictions"]["count"], 1);
    }
}

// --- Notifications --------------------------------------------------------

#[tokio::test]
async fn a_bot_user_is_never_sent_a_notification() {
    let s = setup().await;
    let oracle = UserId(user_id(&s.oracle).await);
    let alice = UserId(user_id(&s.alice).await);
    // Even with every switch turned on by the bot itself.
    let res = s
        .oracle
        .patch(
            "/api/me",
            &json!({ "notifications": { "reminder": true, "reveal": true, "digest": true } }),
        )
        .await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());

    // A Reveal announcement (alice and bob Reveal early; Oracle is Eligible).
    s.predict(&s.oracle, NAVI_FAZE, NAVI).await;
    s.humans_reveal_early().await;
    poller::tick(&s.app.state).await;
    assert!(
        !s.app.notifier.sent_to(alice).is_empty(),
        "the humans are told"
    );

    // A daily digest (NAVI vs FaZe, the only Match of its day, finishes).
    s.app.clock.set(s.start + Duration::hours(1));
    s.app
        .source
        .finish(EventId(EVENT), MatchId(NAVI_FAZE), "NAVI", (2, 0));
    poller::tick(&s.app.state).await;
    // A reminder (nobody has predicted Vitality vs G2, 20 minutes to Lock).
    s.app
        .clock
        .set(s.start + Duration::days(1) - Duration::minutes(35));
    poller::tick(&s.app.state).await;

    let kinds: Vec<_> = s
        .app
        .notifier
        .sent_to(alice)
        .iter()
        .map(|n| n.kind)
        .collect();
    for kind in [
        NotificationKind::Reveal,
        NotificationKind::DailyDigest,
        NotificationKind::Reminder,
    ] {
        assert!(kinds.contains(&kind), "alice got no {kind:?}: {kinds:?}");
    }
    assert_eq!(s.app.notifier.sent_to(oracle).len(), 0);
    assert!(
        s.app
            .notifier
            .undeliverable()
            .iter()
            .all(|n| n.user_id != oracle),
        "nothing was even attempted"
    );
}

// --- Archive, and the Bearer token on other endpoints ----------------------

#[tokio::test]
async fn an_archived_league_rejects_a_bot_users_prediction() {
    let app = TestApp::new().await;
    let now = app.clock.now();
    // A Swiss Match days away and a final that is about to be decided.
    app.source.put_event(
        EventSnapshotBuilder::new(EVENT, "Test Major")
            .stage(StageKind::Swiss, "Stage 1")
            .swiss_match(
                NAVI_FAZE,
                (1, 0, 0),
                "NAVI",
                "FaZe",
                now + Duration::days(9),
            )
            .stage(StageKind::Playoffs, "Playoffs")
            .playoff_match(301, (0, 0), "NAVI", "FaZe", now + Duration::days(1))
            .build(now),
    );
    let alice = app.user("alice").await;
    let oracle = app.bot("oracle", "Oracle").await;
    let res = alice
        .post(
            "/api/leagues",
            &json!({ "event_id": EVENT, "name": "Friends" }),
        )
        .await;
    let league = res.json()["id"].as_i64().unwrap();
    join(&oracle, res.json()["invite_token"].as_str().unwrap()).await;
    poller::tick(&app.state).await;
    let res = try_predict_in(&oracle, league, NAVI_FAZE, NAVI).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());

    app.source
        .finish(EventId(EVENT), MatchId(301), "FaZe", (2, 0));
    app.clock.advance(Duration::minutes(6));
    poller::tick(&app.state).await;

    // NAVI vs FaZe is far from its Hard Lock, but the League is read-only.
    let res = try_predict_in(&oracle, league, NAVI_FAZE, FAZE).await;
    assert_eq!(res.status, StatusCode::CONFLICT, "{}", res.text());
    assert_eq!(res.error_code(), "league_archived");
    assert_eq!(
        card_in(&oracle, league, NAVI_FAZE).await["my_prediction"],
        NAVI
    );
}

#[tokio::test]
async fn a_bot_users_bearer_token_is_accepted_wherever_a_session_is() {
    let s = setup().await;
    let league = s.league;
    let paths = [
        "/api/me".to_string(),
        "/api/leagues".to_string(),
        format!("/api/leagues/{league}"),
        format!("/api/leagues/{league}/matches"),
        format!("/api/leagues/{league}/leaderboard"),
        format!("/api/leagues/{league}/pickem-leaderboard"),
        format!("/api/leagues/{league}/stages/1/pickem"),
        format!("/api/leagues/{league}/overrides"),
        format!("/api/invites/{}", s.token),
    ];

    for path in &paths {
        let res = s.oracle.get(path).await;
        assert_eq!(res.status, StatusCode::OK, "{path}: {}", res.text());
        // The same request without any credentials is refused.
        let res = s.app.anonymous().get(path).await;
        assert_eq!(res.status, StatusCode::UNAUTHORIZED, "{path}");
    }
    // It is on the Leaderboard like any Member.
    let rows = s
        .alice
        .get(&format!("/api/leagues/{league}/leaderboard"))
        .await
        .json()["rows"]
        .clone();
    assert!(
        rows.as_array()
            .unwrap()
            .iter()
            .any(|r| r["nickname"] == "Oracle"),
        "{rows}"
    );
}

#[tokio::test]
async fn a_session_cookie_decides_alone_when_a_request_carries_both() {
    let s = setup().await;
    let token = s
        .app
        .bearer(kallbet::testing::TEST_BOT_SECRET)
        .post(
            "/api/auth/bot",
            &json!({ "bot_key": "oracle", "nickname": "Oracle" }),
        )
        .await
        .json()["token"]
        .as_str()
        .unwrap()
        .to_string();

    // alice's session and the Bot User's token: the request is alice's.
    let both = s.app.bearer(&token);
    both.set_cookie(
        "kallbet_session",
        &s.alice.cookie("kallbet_session").unwrap(),
    );
    assert_eq!(both.get("/api/me").await.json()["nickname"], "alice");
    // A session that is not valid is not rescued by the token.
    let stale = s.app.bearer(&token);
    stale.set_cookie("kallbet_session", "not-a-session");
    assert_eq!(stale.get("/api/me").await.status, StatusCode::UNAUTHORIZED);
}
