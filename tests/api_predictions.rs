//! Seam 1: Predictions and Lock (ADR-0003).

use axum::http::StatusCode;
use chrono::{DateTime, Duration, Utc};
use kallbet::domain::{EventId, MatchId, StageKind};
use kallbet::poller;
use kallbet::ports::Clock;
use kallbet::testing::{EventSnapshotBuilder, TestApp, TestClient, TestResponse};
use serde_json::{Value, json};

const EVENT: i64 = 7148;
const NAVI_FAZE: i64 = 101;
// Teams are numbered in order of appearance.
const NAVI: i64 = 1;
const FAZE: i64 = 2;
const VITALITY: i64 = 3;

struct Setup {
    app: TestApp,
    alice: TestClient,
    bob: TestClient,
    league: i64,
    /// Scheduled Start of NAVI vs FaZe.
    start: DateTime<Utc>,
}

/// A League with alice and bob, NAVI vs FaZe two hours from now, polled.
async fn setup() -> Setup {
    let app = TestApp::new().await;
    let now = app.clock.now();
    let start = now + Duration::hours(2);
    app.source.put_event(
        EventSnapshotBuilder::new(EVENT, "Test Major")
            .stage(StageKind::Swiss, "Stage 1")
            .swiss_match(NAVI_FAZE, (1, 0, 0), "NAVI", "FaZe", start)
            .swiss_match(102, (1, 0, 0), "Vitality", "G2", start + Duration::hours(1))
            .build(now),
    );
    let alice = app.user("alice").await;
    let bob = app.user("bob").await;
    let res = alice
        .post(
            "/api/leagues",
            &json!({ "event_id": EVENT, "name": "Friends" }),
        )
        .await;
    let league = res.json();
    let token = league["invite_token"].as_str().unwrap();
    bob.post_empty(&format!("/api/invites/{token}/join")).await;
    poller::tick(&app.state).await;
    Setup {
        league: league["id"].as_i64().unwrap(),
        app,
        alice,
        bob,
        start,
    }
}

impl Setup {
    async fn predict(&self, client: &TestClient, game: i64, team: i64) -> TestResponse {
        client
            .put(
                &format!("/api/leagues/{}/matches/{game}/prediction", self.league),
                &json!({ "team_id": team }),
            )
            .await
    }

    async fn card(&self, client: &TestClient, game: i64) -> Value {
        let res = client
            .get(&format!("/api/leagues/{}/matches", self.league))
            .await;
        assert_eq!(res.status, StatusCode::OK, "{}", res.text());
        res.json()["stages"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|s| s["groups"].as_array().unwrap())
            .flat_map(|g| g["matches"].as_array().unwrap())
            .find(|m| m["id"] == game)
            .unwrap()
            .clone()
    }

    async fn tick_after(&self, by: Duration) {
        self.app.clock.advance(by);
        poller::tick(&self.app.state).await;
    }
}

fn assert_locked(res: &TestResponse) {
    assert_eq!(res.status, StatusCode::CONFLICT, "{}", res.text());
    assert_eq!(res.error_code(), "match_locked");
}

#[tokio::test]
async fn a_member_predicts_and_sees_their_prediction() {
    let s = setup().await;

    let res = s.predict(&s.alice, NAVI_FAZE, NAVI).await;

    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    assert_eq!(res.json()["team_id"], NAVI);
    assert_eq!(s.card(&s.alice, NAVI_FAZE).await["my_prediction"], NAVI);
    // Bob sees only his own Prediction, which he has not made.
    assert_eq!(
        s.card(&s.bob, NAVI_FAZE).await["my_prediction"],
        Value::Null
    );
}

#[tokio::test]
async fn a_prediction_can_be_changed_before_lock() {
    let s = setup().await;
    s.predict(&s.alice, NAVI_FAZE, NAVI).await;
    s.app.clock.advance(Duration::minutes(104)); // 16 minutes before the start

    let res = s.predict(&s.alice, NAVI_FAZE, FAZE).await;

    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    assert_eq!(s.card(&s.alice, NAVI_FAZE).await["my_prediction"], FAZE);
}

#[tokio::test]
async fn a_prediction_is_rejected_after_lock_even_without_a_poll() {
    let s = setup().await;
    s.predict(&s.alice, NAVI_FAZE, NAVI).await;
    s.app.clock.advance(Duration::minutes(105)); // exactly 15 minutes before

    assert_locked(&s.predict(&s.alice, NAVI_FAZE, FAZE).await);
    assert_locked(&s.predict(&s.bob, NAVI_FAZE, FAZE).await);
    // Nothing changed.
    assert_eq!(s.card(&s.alice, NAVI_FAZE).await["my_prediction"], NAVI);
}

#[tokio::test]
async fn the_card_counts_down_to_lock_and_stores_the_lock_time() {
    let s = setup().await;

    let open = s.card(&s.alice, NAVI_FAZE).await;
    assert_eq!(open["lock"]["locked"], false);
    assert_eq!(open["lock"]["locked_at"], Value::Null);
    assert_eq!(
        open["lock"]["locks_at"],
        json!(s.start - Duration::minutes(15))
    );

    s.tick_after(Duration::minutes(110)).await;
    let locked = s.card(&s.alice, NAVI_FAZE).await;
    assert_eq!(locked["lock"]["locked"], true);
    assert_eq!(
        locked["lock"]["locked_at"],
        json!(s.start - Duration::minutes(15))
    );
}

#[tokio::test]
async fn a_postponement_before_lock_moves_the_lock() {
    let s = setup().await;
    s.app.source.postpone(
        EventId(EVENT),
        MatchId(NAVI_FAZE),
        s.start + Duration::hours(3),
    );
    s.tick_after(Duration::minutes(5)).await;

    s.app.clock.set(s.start); // the old Scheduled Start

    let card = s.card(&s.alice, NAVI_FAZE).await;
    assert_eq!(card["lock"]["locked"], false);
    assert_eq!(
        card["lock"]["locks_at"],
        json!(s.start + Duration::hours(3) - Duration::minutes(15))
    );
    assert_eq!(
        s.predict(&s.alice, NAVI_FAZE, NAVI).await.status,
        StatusCode::OK
    );
    s.app
        .clock
        .set(s.start + Duration::hours(3) - Duration::minutes(15));
    assert_locked(&s.predict(&s.alice, NAVI_FAZE, FAZE).await);
}

#[tokio::test]
async fn a_lock_stays_after_a_later_postponement() {
    let s = setup().await;
    s.tick_after(Duration::minutes(106)).await; // locked 14 minutes before

    s.app.source.postpone(
        EventId(EVENT),
        MatchId(NAVI_FAZE),
        s.start + Duration::hours(3),
    );
    s.tick_after(Duration::minutes(1)).await;

    let card = s.card(&s.alice, NAVI_FAZE).await;
    assert_eq!(card["lock"]["locked"], true);
    assert_eq!(
        card["lock"]["locked_at"],
        json!(s.start - Duration::minutes(15))
    );
    assert_eq!(card["scheduled_start"], json!(s.start + Duration::hours(3)));
    assert_locked(&s.predict(&s.alice, NAVI_FAZE, NAVI).await);
}

#[tokio::test]
async fn a_lock_that_was_due_before_a_postponement_arrives_stays() {
    let s = setup().await;
    // Nobody looked between the Lock time and the poll that brings the
    // postponement: the Lock still happened.
    s.app.clock.advance(Duration::minutes(110));
    s.app.source.postpone(
        EventId(EVENT),
        MatchId(NAVI_FAZE),
        s.start + Duration::hours(3),
    );
    poller::tick(&s.app.state).await;

    assert_locked(&s.predict(&s.alice, NAVI_FAZE, NAVI).await);
}

#[tokio::test]
async fn an_early_actual_start_locks_immediately() {
    let s = setup().await;
    s.app.clock.advance(Duration::minutes(30));
    let now = s.app.clock.now();
    s.app.source.start(EventId(EVENT), MatchId(NAVI_FAZE), now);
    s.tick_after(Duration::minutes(1)).await;

    let card = s.card(&s.alice, NAVI_FAZE).await;
    assert_eq!(card["status"], "live");
    assert_eq!(card["lock"]["locked"], true);
    assert_eq!(card["lock"]["locked_at"], json!(now));
    assert_locked(&s.predict(&s.alice, NAVI_FAZE, NAVI).await);
}

#[tokio::test]
async fn the_poller_stamps_the_actual_start_when_it_first_sees_a_match_live() {
    let s = setup().await;
    s.app.clock.advance(Duration::minutes(30));
    s.app
        .source
        .update_match(EventId(EVENT), MatchId(NAVI_FAZE), |m| {
            m.status = kallbet::domain::MatchStatus::Live;
            m.actual_start = None;
        });
    s.tick_after(Duration::minutes(5)).await;
    let first_seen = s.app.clock.now();
    s.tick_after(Duration::minutes(1)).await;

    let card = s.card(&s.alice, NAVI_FAZE).await;
    assert_eq!(card["actual_start"], json!(first_seen));
    assert_eq!(card["lock"]["locked_at"], json!(first_seen));
}

#[tokio::test]
async fn lock_is_per_match() {
    let s = setup().await;
    s.tick_after(Duration::minutes(110)).await;

    assert_locked(&s.predict(&s.alice, NAVI_FAZE, NAVI).await);
    assert_eq!(
        s.predict(&s.alice, 102, VITALITY).await.status,
        StatusCode::OK
    );
}

#[tokio::test]
async fn the_team_must_play_in_the_match() {
    let s = setup().await;

    let res = s.predict(&s.alice, NAVI_FAZE, VITALITY).await;

    assert_eq!(res.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(res.error_code(), "team_not_in_match");
}

#[tokio::test]
async fn only_members_predict_and_only_on_their_events_matches() {
    let s = setup().await;
    let mallory = s.app.user("mallory").await;

    let res = s.predict(&mallory, NAVI_FAZE, NAVI).await;
    assert_eq!(res.status, StatusCode::NOT_FOUND);

    let res = s.predict(&s.alice, 999, NAVI).await;
    assert_eq!(res.status, StatusCode::NOT_FOUND);
    assert_eq!(res.error_code(), "not_found");

    let res = s
        .app
        .anonymous()
        .put(
            &format!("/api/leagues/{}/matches/{NAVI_FAZE}/prediction", s.league),
            &json!({ "team_id": NAVI }),
        )
        .await;
    assert_eq!(res.status, StatusCode::UNAUTHORIZED);
}
