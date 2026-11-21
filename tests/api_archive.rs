//! Seam 1: archiving a League after the final (spec stories 80, 81).

use axum::http::StatusCode;
use chrono::Duration;
use kallbet::domain::{EventId, MatchId, StageKind};
use kallbet::poller;
use kallbet::ports::Clock;
use kallbet::testing::{EventSnapshotBuilder, TestApp, TestClient};
use serde_json::{Value, json};

const EVENT: i64 = 7148;

struct Setup {
    app: TestApp,
    league: i64,
    token: String,
}

/// A League for an Event with one Swiss Match (101), a semifinal (201) and the
/// final (301), polled once. Its creator is `alice`.
async fn setup() -> (Setup, TestClient) {
    let app = TestApp::new().await;
    let now = app.clock.now();
    app.source.put_event(
        EventSnapshotBuilder::new(EVENT, "Test Major")
            .stage(StageKind::Swiss, "Stage 1")
            .swiss_match(101, (1, 0, 0), "NAVI", "FaZe", now + Duration::days(2))
            .stage(StageKind::Playoffs, "Playoffs")
            .playoff_match(201, (1, 0), "NAVI", "Spirit", now + Duration::days(5))
            .playoff_match(301, (0, 0), "NAVI", "FaZe", now + Duration::days(6))
            .build(now),
    );
    let alice = app.user("alice").await;
    let res = alice
        .post(
            "/api/leagues",
            &json!({ "event_id": EVENT, "name": "Friends" }),
        )
        .await;
    assert_eq!(res.status, StatusCode::CREATED, "{}", res.text());
    let league = res.json();
    let setup = Setup {
        league: league["id"].as_i64().unwrap(),
        token: league["invite_token"].as_str().unwrap().to_string(),
        app,
    };
    poller::tick(&setup.app.state).await;
    (setup, alice)
}

impl Setup {
    async fn member(&self, nickname: &str) -> TestClient {
        let client = self.app.user(nickname).await;
        let res = client
            .post_empty(&format!("/api/invites/{}/join", self.token))
            .await;
        assert!(res.status.is_success(), "{}", res.text());
        client
    }

    /// The poll that delivers a result.
    async fn finish(&self, game: i64, winner: &str) {
        self.app
            .source
            .finish(EventId(EVENT), MatchId(game), winner, (2, 0));
        self.poll().await;
    }

    async fn poll(&self) {
        self.app.clock.advance(Duration::minutes(6));
        poller::tick(&self.app.state).await;
    }

    async fn league_json(&self, client: &TestClient) -> Value {
        client
            .get(&format!("/api/leagues/{}", self.league))
            .await
            .json()
    }
}

#[tokio::test]
async fn the_league_archives_once_the_final_has_a_winner() {
    let (s, alice) = setup().await;
    assert!(s.league_json(&alice).await["archived_at"].is_null());

    s.finish(101, "NAVI").await;
    s.finish(201, "NAVI").await;
    assert!(
        s.league_json(&alice).await["archived_at"].is_null(),
        "only the final archives"
    );

    s.finish(301, "FaZe").await;
    assert!(!s.league_json(&alice).await["archived_at"].is_null());
}

/// An archived League: the final is decided.
async fn archived() -> (Setup, TestClient) {
    let (s, alice) = setup().await;
    s.finish(301, "FaZe").await;
    assert!(!s.league_json(&alice).await["archived_at"].is_null());
    (s, alice)
}

#[tokio::test]
async fn an_archived_league_rejects_predictions() {
    let (s, alice) = archived().await;

    let res = alice
        .put(
            &format!("/api/leagues/{}/matches/201/prediction", s.league),
            &json!({ "team_id": 1 }),
        )
        .await;

    assert_eq!(res.status, StatusCode::CONFLICT);
    assert_eq!(res.error_code(), "league_archived");
}

#[tokio::test]
async fn an_archived_league_rejects_pickems() {
    let (s, alice) = archived().await;

    let res = alice
        .put(
            &format!("/api/leagues/{}/stages/1/pickem", s.league),
            &json!({ "three_nil": [1, 2], "zero_three": [3, 4], "advance": [5, 6, 7, 8, 9, 10] }),
        )
        .await;

    assert_eq!(res.status, StatusCode::CONFLICT);
    assert_eq!(res.error_code(), "league_archived");
}

#[tokio::test]
async fn nobody_can_join_or_be_kicked_from_an_archived_league() {
    let (s, alice) = setup().await;
    let bob = s.member("bob").await;
    let bob_id = bob.get("/api/me").await.json()["id"].as_i64().unwrap();
    s.finish(301, "FaZe").await;

    let carol = s.app.user("carol").await;
    let join = carol
        .post_empty(&format!("/api/invites/{}/join", s.token))
        .await;
    assert_eq!(join.status, StatusCode::CONFLICT);
    assert_eq!(join.error_code(), "league_archived");

    let kick = alice
        .delete(&format!("/api/leagues/{}/members/{bob_id}", s.league))
        .await;
    assert_eq!(kick.status, StatusCode::CONFLICT);
    assert_eq!(kick.error_code(), "league_archived");
}

#[tokio::test]
async fn members_can_still_leave_and_the_creator_can_still_delete_an_archived_league() {
    let (s, alice) = setup().await;
    let bob = s.member("bob").await;
    s.finish(301, "FaZe").await;

    let left = bob
        .post_empty(&format!("/api/leagues/{}/leave", s.league))
        .await;
    assert_eq!(left.status, StatusCode::NO_CONTENT);

    let deleted = alice
        .post(
            &format!("/api/leagues/{}/delete", s.league),
            &json!({ "confirm_name": "Friends" }),
        )
        .await;
    assert_eq!(deleted.status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn archiving_is_stored_once_and_never_undone() {
    let (s, alice) = archived().await;
    let first = s.league_json(&alice).await["archived_at"].clone();

    s.app.source.update_match(EventId(EVENT), MatchId(301), |m| {
        m.status = kallbet::domain::MatchStatus::Upcoming;
        m.winner = None;
    });
    s.poll().await;
    s.poll().await;

    assert_eq!(s.league_json(&alice).await["archived_at"], first);
}

#[tokio::test]
async fn a_void_final_does_not_archive() {
    let (s, alice) = setup().await;
    s.app.source.cancel(EventId(EVENT), MatchId(301));
    s.poll().await;
    assert!(s.league_json(&alice).await["archived_at"].is_null());

    s.app
        .source
        .finish_without_winner(EventId(EVENT), MatchId(301));
    s.poll().await;
    assert!(s.league_json(&alice).await["archived_at"].is_null());
}
