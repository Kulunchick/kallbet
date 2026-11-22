//! Seam 1: Reveal (ADR-0003). Predictions stay hidden until every Eligible
//! Member has predicted, which Reveals and Locks the Match, or until Lock.

use axum::http::StatusCode;
use chrono::{DateTime, Duration, Utc};
use kallbet::domain::StageKind;
use kallbet::poller;
use kallbet::ports::Clock;
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
    carol: TestClient,
    league: i64,
    token: String,
    /// Scheduled Start of NAVI vs FaZe.
    start: DateTime<Utc>,
}

/// A League of alice (creator), bob and carol; NAVI vs FaZe two hours from
/// now, Vitality vs G2 an hour later; polled.
async fn setup() -> Setup {
    let app = TestApp::new().await;
    let now = app.clock.now();
    let start = now + Duration::hours(2);
    app.source.put_event(
        EventSnapshotBuilder::new(EVENT, "Test Major")
            .stage(StageKind::Swiss, "Stage 1")
            .swiss_match(NAVI_FAZE, (1, 0, 0), "NAVI", "FaZe", start)
            .swiss_match(
                VITALITY_G2,
                (1, 0, 0),
                "Vitality",
                "G2",
                start + Duration::hours(1),
            )
            .build(now),
    );
    let alice = app.user("alice").await;
    let bob = app.user("bob").await;
    let carol = app.user("carol").await;
    let res = alice
        .post(
            "/api/leagues",
            &json!({ "event_id": EVENT, "name": "Friends" }),
        )
        .await;
    let league = res.json();
    let token = league["invite_token"].as_str().unwrap().to_string();
    for c in [&bob, &carol] {
        c.post_empty(&format!("/api/invites/{token}/join")).await;
    }
    poller::tick(&app.state).await;
    Setup {
        league: league["id"].as_i64().unwrap(),
        token,
        app,
        alice,
        bob,
        carol,
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
        client
            .put(
                &format!("/api/leagues/{}/matches/{game}/prediction", self.league),
                &json!({ "team_id": team }),
            )
            .await
    }

    async fn matches_text(&self, client: &TestClient) -> String {
        let res = client
            .get(&format!("/api/leagues/{}/matches", self.league))
            .await;
        assert_eq!(res.status, StatusCode::OK, "{}", res.text());
        res.text()
    }

    async fn card(&self, client: &TestClient, game: i64) -> Value {
        card_in(client, self.league, game).await
    }

    async fn join(&self, client: &TestClient) {
        let res = client
            .post_empty(&format!("/api/invites/{}/join", self.token))
            .await;
        assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    }

    async fn tick_after(&self, by: Duration) {
        self.app.clock.advance(by);
        poller::tick(&self.app.state).await;
    }
}

async fn card_in(client: &TestClient, league: i64, game: i64) -> Value {
    let res = client.get(&format!("/api/leagues/{league}/matches")).await;
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

#[tokio::test]
async fn before_reveal_a_card_shows_only_the_count() {
    let s = setup().await;
    s.predict(&s.alice, NAVI_FAZE, NAVI).await;
    s.predict(&s.bob, NAVI_FAZE, FAZE).await;

    for client in [&s.alice, &s.bob, &s.carol] {
        let card = s.card(client, NAVI_FAZE).await;
        assert_eq!(card["predictions"]["count"], 2, "{card}");
        assert_eq!(card["predictions"]["eligible"], 3);
        assert_eq!(card["predictions"]["revealed"], Value::Null);
        assert_eq!(card["predictions"]["revealed_at"], Value::Null);
        assert_eq!(card["lock"]["locked"], false);
    }
}

#[tokio::test]
async fn only_the_final_voter_can_edit_until_the_original_window_expires() {
    let s = setup().await;
    s.predict(&s.alice, NAVI_FAZE, NAVI).await;
    s.predict(&s.bob, NAVI_FAZE, FAZE).await;
    s.app.clock.advance(Duration::minutes(10));
    let now = s.app.clock.now();

    s.predict(&s.carol, NAVI_FAZE, NAVI).await;
    let end = now + Duration::minutes(5);
    for (client, can_predict) in [(&s.alice, false), (&s.bob, false), (&s.carol, true)] {
        let card = s.card(client, NAVI_FAZE).await;
        assert_eq!(card["lock"]["can_predict"], can_predict);
        assert_eq!(card["lock"]["locks_at"], json!(end));
        assert_eq!(card["predictions"]["revealed"], Value::Null);
    }
    assert_eq!(
        s.try_predict(&s.alice, NAVI_FAZE, FAZE).await.status,
        StatusCode::CONFLICT
    );
    s.app.clock.advance(Duration::minutes(4));
    s.predict(&s.carol, NAVI_FAZE, FAZE).await;
    s.app.clock.advance(Duration::minutes(1));

    for client in [&s.alice, &s.bob, &s.carol] {
        let card = s.card(client, NAVI_FAZE).await;
        assert_eq!(card["lock"]["locked"], true, "{card}");
        assert_eq!(card["lock"]["locked_at"], json!(end));
        assert_eq!(card["predictions"]["revealed_at"], json!(end));
        assert_eq!(card["predictions"]["count"], 3);
        assert_eq!(card["predictions"]["eligible"], 3);
        assert_eq!(
            revealed(&card),
            pairs(&[("alice", NAVI), ("bob", FAZE), ("carol", FAZE)])
        );
    }
    // Nobody may change their Prediction after seeing the others.
    for client in [&s.alice, &s.bob, &s.carol] {
        let res = s.try_predict(client, NAVI_FAZE, FAZE).await;
        assert_eq!(res.status, StatusCode::CONFLICT);
        assert_eq!(res.error_code(), "match_locked");
    }
    // The other Match is untouched.
    let other = s.card(&s.alice, VITALITY_G2).await;
    assert_eq!(other["lock"]["locked"], false);
    assert_eq!(other["predictions"]["revealed"], Value::Null);
}

#[tokio::test]
async fn without_full_participation_reveal_happens_at_lock() {
    let s = setup().await;
    s.predict(&s.alice, NAVI_FAZE, NAVI).await;
    s.predict(&s.bob, NAVI_FAZE, FAZE).await;

    s.tick_after(Duration::minutes(110)).await; // 10 minutes before the start

    let card = s.card(&s.carol, NAVI_FAZE).await;
    let lock = json!(s.start - Duration::minutes(15));
    assert_eq!(card["lock"]["locked_at"], lock);
    assert_eq!(card["predictions"]["revealed_at"], lock);
    assert_eq!(card["predictions"]["count"], 2);
    assert_eq!(card["predictions"]["eligible"], 3);
    assert_eq!(revealed(&card), pairs(&[("alice", NAVI), ("bob", FAZE)]));
}

#[tokio::test]
async fn reveal_at_lock_needs_no_poll() {
    let s = setup().await;
    s.predict(&s.alice, NAVI_FAZE, NAVI).await;
    s.app.clock.advance(Duration::minutes(105)); // exactly at Lock

    let card = s.card(&s.bob, NAVI_FAZE).await;
    assert_eq!(revealed(&card), pairs(&[("alice", NAVI)]));
}

#[tokio::test]
async fn a_member_who_left_does_not_count_and_does_not_hold_up_reveal() {
    let s = setup().await;
    s.predict(&s.alice, NAVI_FAZE, NAVI).await;
    s.predict(&s.carol, NAVI_FAZE, FAZE).await;

    let res = s
        .carol
        .post_empty(&format!("/api/leagues/{}/leave", s.league))
        .await;
    assert!(res.status.is_success(), "{}", res.text());
    let card = s.card(&s.alice, NAVI_FAZE).await;
    assert_eq!(card["predictions"]["count"], 1, "{card}");
    assert_eq!(card["predictions"]["eligible"], 2);

    s.predict(&s.bob, NAVI_FAZE, NAVI).await;
    s.app.clock.advance(Duration::minutes(5));

    let card = s.card(&s.alice, NAVI_FAZE).await;
    assert_eq!(card["lock"]["locked"], true);
    assert_eq!(revealed(&card), pairs(&[("alice", NAVI), ("bob", NAVI)]));
}

#[tokio::test]
async fn kicking_the_last_member_who_has_not_predicted_reveals() {
    let s = setup().await;
    s.predict(&s.alice, NAVI_FAZE, NAVI).await;
    s.predict(&s.bob, NAVI_FAZE, FAZE).await;
    let carol = user_id(&s.carol).await;

    let res = s
        .alice
        .delete(&format!("/api/leagues/{}/members/{carol}", s.league))
        .await;
    assert!(res.status.is_success(), "{}", res.text());

    let card = s.card(&s.bob, NAVI_FAZE).await;
    assert_eq!(card["predictions"]["eligible"], 2);
    assert_eq!(card["lock"]["locked"], true);
    assert_eq!(revealed(&card), pairs(&[("alice", NAVI), ("bob", FAZE)]));
}

#[tokio::test]
async fn a_member_who_joined_after_lock_does_not_count() {
    let s = setup().await;
    s.predict(&s.alice, NAVI_FAZE, NAVI).await;
    s.tick_after(Duration::minutes(110)).await; // NAVI vs FaZe is Locked
    let dave = s.app.user("dave").await;

    s.join(&dave).await;

    let locked = s.card(&dave, NAVI_FAZE).await;
    assert_eq!(locked["predictions"]["eligible"], 3, "{locked}");
    assert_eq!(locked["predictions"]["count"], 1);
    assert_eq!(revealed(&locked), pairs(&[("alice", NAVI)]));
    // He does count for the Match that is still open.
    let open = s.card(&dave, VITALITY_G2).await;
    assert_eq!(open["predictions"]["eligible"], 4);
}

#[tokio::test]
async fn a_member_who_left_and_rejoined_after_lock_does_not_count() {
    let s = setup().await;
    s.predict(&s.carol, NAVI_FAZE, FAZE).await;
    s.carol
        .post_empty(&format!("/api/leagues/{}/leave", s.league))
        .await;
    s.tick_after(Duration::minutes(110)).await;

    s.join(&s.carol).await;

    let card = s.card(&s.alice, NAVI_FAZE).await;
    assert_eq!(card["predictions"]["eligible"], 2, "{card}");
    assert_eq!(card["predictions"]["count"], 0);
    assert_eq!(revealed(&card), pairs(&[]));
}

#[tokio::test]
async fn a_reveal_is_never_undone() {
    let s = setup().await;
    for (c, team) in [(&s.alice, NAVI), (&s.bob, FAZE), (&s.carol, NAVI)] {
        s.predict(c, NAVI_FAZE, team).await;
    }
    s.app.clock.advance(Duration::minutes(5));
    let revealed_at = s.card(&s.alice, NAVI_FAZE).await["predictions"]["revealed_at"].clone();
    assert_ne!(revealed_at, Value::Null);

    // A new Member joins, the Match is postponed far away, time passes.
    s.app.clock.advance(Duration::minutes(1));
    let dave = s.app.user("dave").await;
    s.join(&dave).await;
    s.app.source.postpone(
        kallbet::domain::EventId(EVENT),
        kallbet::domain::MatchId(NAVI_FAZE),
        s.start + Duration::days(2),
    );
    s.tick_after(Duration::minutes(30)).await;
    s.tick_after(Duration::hours(3)).await;

    let card = s.card(&dave, NAVI_FAZE).await;
    assert_eq!(card["predictions"]["revealed_at"], revealed_at, "{card}");
    assert_eq!(card["lock"]["locked"], true);
    assert_eq!(card["predictions"]["eligible"], 3);
    assert_eq!(
        revealed(&card),
        pairs(&[("alice", NAVI), ("bob", FAZE), ("carol", NAVI)])
    );
    let res = s.try_predict(&dave, NAVI_FAZE, NAVI).await;
    assert_eq!(res.error_code(), "match_locked");
}

#[tokio::test]
/// Lock and Reveal state is per League (#5, #6): two Leagues on the same Event
/// whose Eligible Members differ.
async fn lock_and_reveal_are_per_league() {
    let s = setup().await;
    // A second League on the same Event, of alice and bob only.
    let res = s
        .alice
        .post("/api/leagues", &json!({ "event_id": EVENT, "name": "Duo" }))
        .await;
    let duo = res.json();
    let duo_id = duo["id"].as_i64().unwrap();
    let token = duo["invite_token"].as_str().unwrap();
    s.bob
        .post_empty(&format!("/api/invites/{token}/join"))
        .await;

    for league in [s.league, duo_id] {
        for c in [&s.alice, &s.bob] {
            let res = c
                .put(
                    &format!("/api/leagues/{league}/matches/{NAVI_FAZE}/prediction"),
                    &json!({ "team_id": NAVI }),
                )
                .await;
            assert_eq!(res.status, StatusCode::OK, "{}", res.text());
        }
    }

    s.app.clock.advance(Duration::minutes(5));
    // Duo: both Eligible Members predicted, so it Revealed and Locked early.
    let duo_card = card_in(&s.alice, duo_id, NAVI_FAZE).await;
    assert_eq!(duo_card["lock"]["locked"], true, "{duo_card}");
    assert_eq!(
        revealed(&duo_card),
        pairs(&[("alice", NAVI), ("bob", NAVI)])
    );
    // Friends still waits for carol, who can still predict there.
    let friends_card = s.card(&s.alice, NAVI_FAZE).await;
    assert_eq!(friends_card["lock"]["locked"], false, "{friends_card}");
    assert_eq!(friends_card["predictions"]["revealed"], Value::Null);
    assert_eq!(friends_card["predictions"]["count"], 2);
    let res = s
        .alice
        .put(
            &format!("/api/leagues/{}/matches/{NAVI_FAZE}/prediction", s.league),
            &json!({ "team_id": FAZE }),
        )
        .await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    let res = s
        .alice
        .put(
            &format!("/api/leagues/{duo_id}/matches/{NAVI_FAZE}/prediction"),
            &json!({ "team_id": FAZE }),
        )
        .await;
    assert_eq!(res.error_code(), "match_locked");
}

#[tokio::test]
async fn before_reveal_no_response_carries_another_members_prediction() {
    let s = setup().await;
    // Bob picks a team nobody else picks; alice and carol stay out of it.
    let res = s.predict(&s.bob, VITALITY_G2, VITALITY).await;
    let bob = user_id(&s.bob).await;
    assert_eq!(res.json()["user_id"], bob);
    s.predict(&s.alice, NAVI_FAZE, NAVI).await;

    for client in [&s.alice, &s.carol] {
        let body = s.matches_text(client).await;
        let view: Value = serde_json::from_str(&body).unwrap();
        let card = view["stages"][0]["groups"][0]["matches"]
            .as_array()
            .unwrap()
            .iter()
            .find(|m| m["id"] == VITALITY_G2)
            .unwrap()
            .clone();
        assert_eq!(card["my_prediction"], Value::Null);
        assert_eq!(card["predictions"]["count"], 1);
        assert_eq!(card["predictions"]["revealed"], Value::Null);
        // Bob's Prediction is nowhere in the response: not his id, not his
        // Nickname.
        assert!(!body.contains("\"bob\""), "{body}");
        assert!(!body.contains(&format!("\"user_id\":{bob}")), "{body}");
    }
    // Other Members' Prediction responses do not carry it either.
    let res = s.predict(&s.carol, VITALITY_G2, VITALITY + 1).await;
    assert!(!res.text().contains(&format!("\"user_id\":{bob}")));
}

#[tokio::test]
async fn the_normal_deadline_can_end_the_final_edit_window_early() {
    let s = setup().await;
    s.predict(&s.alice, NAVI_FAZE, NAVI).await;
    s.predict(&s.bob, NAVI_FAZE, FAZE).await;

    let lock = s.start - Duration::minutes(15);
    s.app.clock.set(lock - Duration::minutes(2));
    s.predict(&s.carol, NAVI_FAZE, NAVI).await;
    let card = s.card(&s.carol, NAVI_FAZE).await;
    assert_eq!(card["lock"]["locks_at"], json!(lock));
    assert_eq!(card["predictions"]["revealed"], Value::Null);

    s.app.clock.set(lock);
    let result = s.try_predict(&s.carol, NAVI_FAZE, FAZE).await;
    assert_eq!(result.error_code(), "match_locked");
    let card = s.card(&s.carol, NAVI_FAZE).await;
    assert_eq!(card["predictions"]["revealed_at"], json!(lock));
}
