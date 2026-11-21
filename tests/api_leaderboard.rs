//! Seam 1: scoring and the Leaderboard (spec stories 22, 35, 36, 53-58).

use axum::http::StatusCode;
use chrono::Duration;
use kallbet::domain::{EventId, MatchId, StageKind};
use kallbet::poller;
use kallbet::ports::Clock;
use kallbet::testing::{EventSnapshotBuilder, TestApp, TestClient};
use serde_json::{Value, json};

const EVENT: i64 = 7148;
// Teams are numbered in order of appearance.
const NAVI: i64 = 1;
const FAZE: i64 = 2;
const VITALITY: i64 = 3;
const G2: i64 = 4;
const SPIRIT: i64 = 5;
const MOUZ: i64 = 6;

struct Setup {
    app: TestApp,
    league: i64,
    token: String,
}

/// A League for an Event with two Swiss Matches (101, 102) and a Playoffs
/// Match (201), polled once. Its creator is `alice`.
async fn setup() -> (Setup, TestClient) {
    let app = TestApp::new().await;
    let now = app.clock.now();
    app.source.put_event(
        EventSnapshotBuilder::new(EVENT, "Test Major")
            .stage(StageKind::Swiss, "Stage 1")
            .swiss_match(101, (1, 0, 0), "NAVI", "FaZe", now + Duration::hours(2))
            .swiss_match(102, (1, 0, 0), "Vitality", "G2", now + Duration::hours(3))
            .stage(StageKind::Playoffs, "Playoffs")
            .playoff_match(201, (2, 0), "Spirit", "MOUZ", now + Duration::days(5))
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

    async fn predict(&self, client: &TestClient, game: i64, team: i64) {
        let res = client
            .put(
                &format!("/api/leagues/{}/matches/{game}/prediction", self.league),
                &json!({ "team_id": team }),
            )
            .await;
        assert_eq!(res.status, StatusCode::OK, "{}", res.text());
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

    async fn board(&self, client: &TestClient) -> Vec<Value> {
        let res = client
            .get(&format!("/api/leagues/{}/leaderboard", self.league))
            .await;
        assert_eq!(res.status, StatusCode::OK, "{}", res.text());
        res.json()["rows"].as_array().unwrap().clone()
    }

    async fn card(&self, client: &TestClient, game: i64) -> Value {
        let res = client
            .get(&format!("/api/leagues/{}/matches", self.league))
            .await;
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
}

async fn user_id(client: &TestClient) -> i64 {
    client.get("/api/me").await.json()["id"].as_i64().unwrap()
}

/// (nickname, place, points) of each row.
fn summary(rows: &[Value]) -> Vec<(String, i64, i64)> {
    rows.iter()
        .map(|r| {
            (
                r["nickname"].as_str().unwrap().to_string(),
                r["place"].as_i64().unwrap(),
                r["points"].as_i64().unwrap(),
            )
        })
        .collect()
}

fn row(nickname: &str, place: i64, points: i64) -> (String, i64, i64) {
    (nickname.to_string(), place, points)
}

#[tokio::test]
async fn correct_predictions_earn_points_and_wrong_or_missing_ones_do_not() {
    let (s, alice) = setup().await;
    let bob = s.member("bob").await;
    let carol = s.member("carol").await;
    let dave = s.member("dave").await;
    s.predict(&alice, 101, NAVI).await;
    s.predict(&bob, 101, NAVI).await;
    s.predict(&carol, 101, FAZE).await;
    s.predict(&alice, 102, VITALITY).await;
    // bob misses 102 and dave predicts nothing at all

    s.finish(101, "NAVI").await;
    s.finish(102, "Vitality").await;
    let rows = s.board(&dave).await;

    assert_eq!(
        summary(&rows),
        [
            row("alice", 1, 2),
            row("bob", 2, 1),
            row("carol", 3, 0),
            row("dave", 3, 0)
        ]
    );
}

#[tokio::test]
async fn two_leaders_share_first_place_and_the_next_member_is_third() {
    let (s, alice) = setup().await;
    let bob = s.member("bob").await;
    let carol = s.member("carol").await;
    s.predict(&alice, 101, NAVI).await;
    s.predict(&bob, 101, NAVI).await;
    s.predict(&carol, 101, FAZE).await;

    s.finish(101, "NAVI").await;

    assert_eq!(
        summary(&s.board(&alice).await),
        [row("alice", 1, 1), row("bob", 1, 1), row("carol", 3, 0)]
    );
}

#[tokio::test]
async fn the_leaderboard_updates_after_the_poll_that_delivers_a_result() {
    let (s, alice) = setup().await;
    s.predict(&alice, 101, NAVI).await;
    assert_eq!(summary(&s.board(&alice).await), [row("alice", 1, 0)]);

    s.finish(101, "NAVI").await;

    assert_eq!(summary(&s.board(&alice).await), [row("alice", 1, 1)]);
}

#[tokio::test]
async fn playoff_points_count_only_playoffs_matches() {
    let (s, alice) = setup().await;
    s.predict(&alice, 101, NAVI).await;
    s.predict(&alice, 201, SPIRIT).await;

    s.finish(101, "NAVI").await;
    let rows = s.board(&alice).await;
    assert_eq!(rows[0]["points"], 1);
    assert_eq!(rows[0]["playoff_points"], 0);

    s.finish(201, "Spirit").await;
    let rows = s.board(&alice).await;
    assert_eq!(rows[0]["points"], 2);
    assert_eq!(rows[0]["playoff_points"], 1);
}

#[tokio::test]
async fn the_percentage_of_correct_predictions_excludes_void_matches() {
    let (s, alice) = setup().await;
    s.predict(&alice, 101, NAVI).await;
    s.predict(&alice, 102, G2).await;
    s.predict(&alice, 201, MOUZ).await;

    assert_eq!(s.board(&alice).await[0]["percent_correct"], Value::Null);
    s.finish(101, "NAVI").await;
    s.finish(102, "Vitality").await;
    s.app.source.cancel(EventId(EVENT), MatchId(201));
    s.poll().await;

    let rows = s.board(&alice).await;
    assert_eq!(rows[0]["points"], 1);
    assert_eq!(
        rows[0]["percent_correct"], 50,
        "1 right of 2; the Void Match is not counted"
    );
    assert_eq!(s.card(&alice, 201).await["my_result"], "void");
}

#[tokio::test]
async fn a_match_reported_without_a_winner_counts_for_nothing() {
    let (s, alice) = setup().await;
    s.predict(&alice, 101, NAVI).await;

    s.app
        .source
        .finish_without_winner(EventId(EVENT), MatchId(101));
    s.poll().await;

    let rows = s.board(&alice).await;
    assert_eq!(rows[0]["points"], 0);
    assert_eq!(rows[0]["percent_correct"], Value::Null);
    assert_eq!(s.card(&alice, 101).await["my_result"], "void");
}

#[tokio::test]
async fn a_late_joiner_scores_zero_for_matches_they_could_not_predict() {
    let (s, alice) = setup().await;
    s.predict(&alice, 101, NAVI).await;
    s.finish(101, "NAVI").await;

    let bob = s.member("bob").await;
    s.predict(&alice, 102, VITALITY).await;
    s.predict(&bob, 102, VITALITY).await;
    s.finish(102, "Vitality").await;

    assert_eq!(
        summary(&s.board(&bob).await),
        [row("alice", 1, 2), row("bob", 2, 1)]
    );
    assert_eq!(s.card(&bob, 101).await["my_result"], "missing");
}

#[tokio::test]
async fn a_member_who_left_or_was_kicked_is_not_on_the_leaderboard() {
    let (s, alice) = setup().await;
    let bob = s.member("bob").await;
    let carol = s.member("carol").await;
    s.predict(&alice, 101, NAVI).await;
    s.predict(&bob, 101, NAVI).await;
    s.predict(&carol, 101, NAVI).await;
    s.finish(101, "NAVI").await;
    assert_eq!(s.board(&alice).await.len(), 3);

    let res = bob
        .post_empty(&format!("/api/leagues/{}/leave", s.league))
        .await;
    assert!(res.status.is_success(), "{}", res.text());
    let carol_id = user_id(&carol).await;
    let res = alice
        .delete(&format!("/api/leagues/{}/members/{carol_id}", s.league))
        .await;
    assert!(res.status.is_success(), "{}", res.text());

    assert_eq!(summary(&s.board(&alice).await), [row("alice", 1, 1)]);
}

#[tokio::test]
async fn finished_cards_show_the_score_and_whether_my_prediction_was_right() {
    let (s, alice) = setup().await;
    let bob = s.member("bob").await;
    s.predict(&alice, 101, NAVI).await;
    s.predict(&bob, 101, FAZE).await;
    assert_eq!(s.card(&alice, 101).await["my_result"], "pending");

    s.finish(101, "NAVI").await;

    let mine = s.card(&alice, 101).await;
    assert_eq!(mine["score"], json!({ "team_a": 2, "team_b": 0 }));
    assert_eq!(mine["my_result"], "correct");
    assert_eq!(mine["my_points"], 1);
    let theirs = s.card(&bob, 101).await;
    assert_eq!(theirs["my_result"], "wrong");
    assert_eq!(theirs["my_points"], 0);
}

#[tokio::test]
async fn my_leagues_show_my_place_and_my_points() {
    let (s, alice) = setup().await;
    let bob = s.member("bob").await;
    s.predict(&alice, 101, NAVI).await;
    s.predict(&bob, 101, FAZE).await;
    s.finish(101, "NAVI").await;

    let a = alice.get("/api/leagues").await.json()[0].clone();
    assert_eq!(a["my_place"], 1);
    assert_eq!(a["my_points"], 1);
    let b = bob.get("/api/leagues").await.json()[0].clone();
    assert_eq!(b["my_place"], 2);
    assert_eq!(b["my_points"], 0);
}

#[tokio::test]
async fn the_leaderboard_is_only_for_members() {
    let (s, _alice) = setup().await;
    let stranger = s.app.user("stranger").await;

    let res = stranger
        .get(&format!("/api/leagues/{}/leaderboard", s.league))
        .await;

    assert_eq!(res.status, StatusCode::NOT_FOUND);
}
