//! Seam 1: the Swiss Pick'em (validation, Lock, hiding, scoring, Leaderboard).

use axum::http::StatusCode;
use chrono::{DateTime, Duration, Utc};
use kallbet::domain::{EventId, MatchId, StageKind};
use kallbet::poller;
use kallbet::ports::Clock;
use kallbet::testing::{EventSnapshotBuilder, TestApp, TestClient, TestResponse};
use serde_json::{Value, json};

const EVENT: i64 = 7148;

struct Setup {
    app: TestApp,
    alice: TestClient,
    bob: TestClient,
    league: i64,
    /// Scheduled Start of the first round.
    start: DateTime<Utc>,
}

/// Teams "T1".."T16" (ids 1..16, in order of appearance). Round 1 pairs them
/// (match ids 101..108); further Matches (ids 201..) have scripted records.
async fn setup() -> Setup {
    let app = TestApp::new().await;
    let now = app.clock.now();
    let start = now + Duration::hours(2);
    let mut event =
        EventSnapshotBuilder::new(EVENT, "Test Major").stage(StageKind::Swiss, "Stage 1");
    for i in 0..8 {
        event = event.swiss_match(
            101 + i,
            (1, 0, 0),
            &format!("T{}", 2 * i + 1),
            &format!("T{}", 2 * i + 2),
            start,
        );
    }
    // Later rounds: T1 goes 3-0, T7 goes 3-1, T16 goes 0-3.
    let later = start + Duration::days(1);
    event = event
        .swiss_match(201, (2, 1, 0), "T1", "T3", later)
        .swiss_match(202, (3, 2, 0), "T1", "T5", later)
        .swiss_match(203, (2, 1, 0), "T7", "T9", later)
        .swiss_match(204, (3, 2, 0), "T7", "T11", later)
        .swiss_match(205, (4, 2, 1), "T7", "T13", later)
        .swiss_match(206, (2, 0, 1), "T16", "T14", later)
        .swiss_match(207, (3, 0, 2), "T16", "T12", later)
        // A second Stage, to check Stages are told apart.
        .stage(StageKind::Playoffs, "Playoffs")
        .playoff_match(301, (0, 0), "T1", "T7", later + Duration::days(5));
    app.source.put_event(event.build(now));
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

/// A Pick'em body, by team number.
fn picks(three_nil: [i64; 2], zero_three: [i64; 2], advance: [i64; 6]) -> Value {
    json!({ "three_nil": three_nil, "zero_three": zero_three, "advance": advance })
}

fn valid() -> Value {
    picks([1, 4], [16, 2], [7, 3, 5, 6, 9, 10])
}

impl Setup {
    fn url(&self) -> String {
        format!("/api/leagues/{}/stages/1/pickem", self.league)
    }

    async fn pick(&self, client: &TestClient, body: &Value) -> TestResponse {
        client.put(&self.url(), body).await
    }

    async fn view(&self, client: &TestClient) -> Value {
        let res = client.get(&self.url()).await;
        assert_eq!(res.status, StatusCode::OK, "{}", res.text());
        res.json()
    }
}

#[tokio::test]
async fn a_member_makes_a_pickem_and_sees_it_back() {
    let s = setup().await;

    let res = s.pick(&s.alice, &valid()).await;

    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    let view = s.view(&s.alice).await;
    assert_eq!(view["my_pickem"]["three_nil"], json!([1, 4]));
    assert_eq!(view["my_pickem"]["zero_three"], json!([16, 2]));
    assert_eq!(view["my_pickem"]["advance"], json!([7, 3, 5, 6, 9, 10]));
    // The Stage's teams: all 16 of round 1, none of the playoff-only ones.
    assert_eq!(view["teams"].as_array().unwrap().len(), 16);
    // Bob has none yet.
    assert_eq!(s.view(&s.bob).await["my_pickem"], Value::Null);
}

fn assert_rejected(res: &TestResponse, status: StatusCode, code: &str) {
    assert_eq!(res.status, status, "{}", res.text());
    assert_eq!(res.error_code(), code);
}

const CHANGED: fn() -> Value = || picks([2, 3], [15, 1], [7, 4, 5, 6, 9, 10]);

#[tokio::test]
async fn invalid_pickems_are_rejected_with_an_error_code() {
    let s = setup().await;
    let unprocessable = StatusCode::UNPROCESSABLE_ENTITY;

    // Too few 3-0 picks.
    let body = json!({ "three_nil": [1], "zero_three": [16, 2], "advance": [7, 3, 5, 6, 9, 10] });
    assert_rejected(&s.pick(&s.alice, &body).await, unprocessable, "pickem_wrong_count");
    // Seven advancing teams.
    let mut seven = valid();
    seven["advance"] = json!([7, 3, 5, 6, 9, 10, 11]);
    assert_rejected(&s.pick(&s.alice, &seven).await, unprocessable, "pickem_wrong_count");
    // The same team twice within a list, and across lists.
    let body = picks([1, 1], [16, 2], [7, 3, 5, 6, 9, 10]);
    assert_rejected(&s.pick(&s.alice, &body).await, unprocessable, "pickem_duplicate_team");
    let body = picks([1, 4], [16, 2], [7, 3, 5, 6, 9, 1]);
    assert_rejected(&s.pick(&s.alice, &body).await, unprocessable, "pickem_duplicate_team");
    // A team that is not in the Stage (99 does not exist).
    let body = picks([1, 4], [16, 2], [7, 3, 5, 6, 9, 99]);
    assert_rejected(&s.pick(&s.alice, &body).await, unprocessable, "pickem_team_not_in_stage");

    // None of them was saved.
    assert_eq!(s.view(&s.alice).await["my_pickem"], Value::Null);
}

#[tokio::test]
async fn a_pickem_can_be_changed_until_the_first_match_starts() {
    let s = setup().await;
    s.pick(&s.alice, &valid()).await;
    // 1 minute before the first round starts: still open (a Match's own Lock
    // is 15 minutes earlier, the Pick'em's is the start).
    s.app.clock.set(s.start - Duration::minutes(1));

    let res = s.pick(&s.alice, &CHANGED()).await;

    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    let view = s.view(&s.alice).await;
    assert_eq!(view["my_pickem"]["three_nil"], json!([2, 3]));
    assert_eq!(view["lock"]["locked"], false);
}

#[tokio::test]
async fn edits_are_rejected_once_the_first_match_has_started() {
    let s = setup().await;
    s.pick(&s.alice, &valid()).await;
    s.app.clock.set(s.start);
    poller::tick(&s.app.state).await;

    assert_rejected(&s.pick(&s.alice, &CHANGED()).await, StatusCode::CONFLICT, "pickem_locked");
    // Bob cannot make a late one either.
    assert_rejected(&s.pick(&s.bob, &valid()).await, StatusCode::CONFLICT, "pickem_locked");
    let view = s.view(&s.alice).await;
    assert_eq!(view["my_pickem"]["three_nil"], json!([1, 4]));
    assert_eq!(view["lock"]["locked"], true);
}

#[tokio::test]
async fn an_early_start_locks_at_once_and_a_lock_survives_a_postponement() {
    let s = setup().await;
    let event = EventId(EVENT);
    // Round 1 starts live before its Scheduled Start.
    s.app.clock.advance(Duration::minutes(30));
    s.app.source.start(event, MatchId(101), s.app.clock.now());
    poller::tick(&s.app.state).await;
    assert_rejected(&s.pick(&s.alice, &valid()).await, StatusCode::CONFLICT, "pickem_locked");

    // Postponing every other round-1 Match afterwards does not reopen it.
    for i in 102..=108 {
        s.app.source.postpone(event, MatchId(i), s.start + Duration::days(3));
    }
    s.app.clock.advance(Duration::minutes(10));
    poller::tick(&s.app.state).await;
    assert_rejected(&s.pick(&s.alice, &valid()).await, StatusCode::CONFLICT, "pickem_locked");
}

#[tokio::test]
async fn other_members_picks_are_hidden_until_lock_even_when_everyone_has_picked() {
    let s = setup().await;
    s.pick(&s.alice, &valid()).await;
    s.pick(&s.bob, &CHANGED()).await;

    // Everyone has picked, yet nothing about the others is returned, and the
    // Pick'em is not Locked (ADR-0003).
    let before = s.view(&s.bob).await;
    assert_eq!(before["pickems"], Value::Null);
    assert_eq!(before["lock"]["locked"], false);
    assert!(!serde_json::to_string(&before).unwrap().contains("alice"));
    assert_eq!(s.pick(&s.bob, &CHANGED()).await.status, StatusCode::OK);

    // At Lock they all appear.
    s.app.clock.set(s.start);
    let after = s.view(&s.bob).await;
    let all = after["pickems"].as_array().unwrap();
    assert_eq!(all.len(), 2);
    let alice = all.iter().find(|p| p["nickname"] == "alice").unwrap();
    assert_eq!(alice["three_nil"], json!([1, 4]));
}

#[tokio::test]
async fn the_poller_stores_the_lock_even_if_nobody_looks() {
    let s = setup().await;
    let event = EventId(EVENT);
    // The Scheduled Start arrives; the poller runs, nobody asks.
    s.app.clock.set(s.start + Duration::minutes(1));
    poller::tick(&s.app.state).await;
    // Then every round-1 Match is postponed far into the future.
    for i in 101..=108 {
        s.app.source.postpone(event, MatchId(i), s.start + Duration::days(3));
    }
    s.app.clock.advance(Duration::minutes(10));
    poller::tick(&s.app.state).await;

    assert_rejected(&s.pick(&s.alice, &valid()).await, StatusCode::CONFLICT, "pickem_locked");
}

/// Plays Swiss Matches so that T1 ends 3-0, T7 3-1 and T16 0-3.
fn play_the_stage(s: &Setup) {
    let event = EventId(EVENT);
    let results = [
        (101, "T1", 2),
        (104, "T7", 2),
        (108, "T15", 2),
        (201, "T1", 2),
        (202, "T1", 2),
        (203, "T7", 2),
        (204, "T11", 2),
        (205, "T7", 2),
        (206, "T14", 2),
        (207, "T12", 2),
    ];
    for (id, winner, wins) in results {
        s.app.source.finish(event, MatchId(id), winner, (wins, 0));
    }
}

async fn leaderboard(s: &Setup, client: &TestClient) -> Vec<(i64, String, i64)> {
    let res = client
        .get(&format!("/api/leagues/{}/pickem-leaderboard", s.league))
        .await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    res.json()["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| {
            (
                e["place"].as_i64().unwrap(),
                e["nickname"].as_str().unwrap().to_string(),
                e["points"].as_i64().unwrap(),
            )
        })
        .collect()
}

async fn join(s: &Setup, nickname: &str) -> TestClient {
    let user = s.app.user(nickname).await;
    let league = s.alice.get(&format!("/api/leagues/{}", s.league)).await.json();
    let token = league["invite_token"].as_str().unwrap().to_string();
    user.post_empty(&format!("/api/invites/{token}/join")).await;
    user
}

#[tokio::test]
async fn scoring_gives_one_point_per_correct_pick_by_the_final_records() {
    let s = setup().await;
    let carol = join(&s, "carol").await;
    let _dave = join(&s, "dave").await;
    // alice: T1 3-0 right, T16 0-3 right, T7 advancing right (3 Points).
    s.pick(&s.alice, &valid()).await;
    // bob: T7 as 3-0 is wrong (it went 3-1), T16 0-3 is right, T1 as merely
    // "advancing" is wrong (it went 3-0): 1 Point.
    s.pick(&s.bob, &picks([7, 2], [16, 15], [1, 3, 5, 6, 9, 10])).await;
    // carol: the same score as bob (only T16 0-3) through different picks.
    carol.put(&s.url(), &picks([3, 4], [16, 13], [5, 6, 9, 10, 11, 12])).await;
    // Before any result everyone has 0 and shares first place.
    assert_eq!(
        leaderboard(&s, &s.alice).await,
        vec![
            (1, "alice".into(), 0),
            (1, "bob".into(), 0),
            (1, "carol".into(), 0),
            (1, "dave".into(), 0)
        ]
    );

    s.app.clock.set(s.start);
    play_the_stage(&s);
    poller::tick(&s.app.state).await;

    assert_eq!(
        leaderboard(&s, &s.bob).await,
        vec![
            (1, "alice".into(), 3),
            (2, "bob".into(), 1),
            (2, "carol".into(), 1),
            (4, "dave".into(), 0)
        ]
    );
}

#[tokio::test]
async fn a_void_match_scores_nothing() {
    let s = setup().await;
    s.pick(&s.alice, &valid()).await;
    s.app.clock.set(s.start);
    play_the_stage(&s);
    // T1's third win is Void: T1 is not settled, so only T16 and T7 count.
    s.app.source.cancel(EventId(EVENT), MatchId(202));
    poller::tick(&s.app.state).await;

    assert_eq!(leaderboard(&s, &s.alice).await[0], (1, "alice".into(), 2));
}

#[tokio::test]
async fn deleting_a_league_with_pickems_works() {
    let s = setup().await;
    s.pick(&s.alice, &valid()).await;
    let res = s
        .alice
        .post(
            &format!("/api/leagues/{}/delete", s.league),
            &json!({ "confirm_name": "Friends" }),
        )
        .await;
    assert_eq!(res.status, StatusCode::NO_CONTENT, "{}", res.text());
    assert_eq!(s.alice.get(&s.url()).await.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn only_the_leagues_own_stages_have_a_pickem_and_only_for_members() {
    let s = setup().await;
    // Stage 2 is the Playoffs (its Pick'em is tested in api_pickem_playoffs);
    // stage 9 does not exist.
    for stage in [9] {
        let url = format!("/api/leagues/{}/stages/{stage}/pickem", s.league);
        assert_eq!(s.alice.get(&url).await.status, StatusCode::NOT_FOUND);
        assert_eq!(s.alice.put(&url, &valid()).await.status, StatusCode::NOT_FOUND);
    }
    // A non-Member sees nothing, and a signed-out visitor is refused.
    let outsider = s.app.user("carol").await;
    assert_eq!(outsider.get(&s.url()).await.status, StatusCode::NOT_FOUND);
    assert_eq!(outsider.put(&s.url(), &valid()).await.status, StatusCode::NOT_FOUND);
    assert_eq!(s.app.anonymous().get(&s.url()).await.status, StatusCode::UNAUTHORIZED);
}
