//! Seam 1: League-scoped Overrides (#13, spec stories 71-79).

use axum::http::StatusCode;
use chrono::Duration;
use kallbet::domain::{EventId, MatchId, StageKind, UserId};
use kallbet::poller;
use kallbet::ports::Clock;
use kallbet::testing::{EventSnapshotBuilder, TestApp, TestClient};
use serde_json::{Value, json};

const EVENT: i64 = 7148;
const NAVI: i64 = 1;
const FAZE: i64 = 2;
const SPIRIT: i64 = 3;

struct Setup {
    app: TestApp,
    /// alice's League (she is its creator and League Admin).
    league: i64,
    token: String,
    alice: TestClient,
}

/// An Event with one Swiss Match (101, NAVI vs FaZe, in 2 hours), a
/// semifinal (201, NAVI vs Spirit) and the final (301, NAVI vs FaZe), polled
/// once, and alice's League on it.
async fn setup() -> Setup {
    let app = TestApp::new().await;
    let now = app.clock.now();
    app.source.put_event(
        EventSnapshotBuilder::new(EVENT, "Test Major")
            .stage(StageKind::Swiss, "Stage 1")
            .swiss_match(101, (1, 0, 0), "NAVI", "FaZe", now + Duration::hours(2))
            .stage(StageKind::Playoffs, "Playoffs")
            .playoff_match(201, (1, 0), "NAVI", "Spirit", now + Duration::days(5))
            .playoff_match(301, (0, 0), "NAVI", "FaZe", now + Duration::days(6))
            .build(now),
    );
    let alice = app.user("alice").await;
    let (league, token) = create_league(&alice, "Friends").await;
    poller::tick(&app.state).await;
    Setup {
        app,
        league,
        token,
        alice,
    }
}

async fn create_league(client: &TestClient, name: &str) -> (i64, String) {
    let res = client
        .post("/api/leagues", &json!({ "event_id": EVENT, "name": name }))
        .await;
    assert_eq!(res.status, StatusCode::CREATED, "{}", res.text());
    let league = res.json();
    (
        league["id"].as_i64().unwrap(),
        league["invite_token"].as_str().unwrap().to_string(),
    )
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

    /// A second League on the same Event, created by `nickname`.
    async fn other_league(&self, nickname: &str) -> (TestClient, i64) {
        let client = self.app.user(nickname).await;
        let (league, _) = create_league(&client, "Rivals").await;
        (client, league)
    }

    async fn poll(&self) {
        self.app.clock.advance(Duration::minutes(6));
        poller::tick(&self.app.state).await;
    }

    async fn set(&self, client: &TestClient, league: i64, game: i64, body: Value) -> Value {
        let res = client
            .post(
                &format!("/api/leagues/{league}/matches/{game}/overrides"),
                &body,
            )
            .await;
        assert_eq!(res.status, StatusCode::CREATED, "{}", res.text());
        res.json()
    }
}

async fn matches(client: &TestClient, league: i64) -> Value {
    let res = client.get(&format!("/api/leagues/{league}/matches")).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    res.json()
}

fn all_matches(view: &Value) -> Vec<Value> {
    view["stages"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|s| s["groups"].as_array().unwrap())
        .flat_map(|g| g["matches"].as_array().unwrap())
        .cloned()
        .collect()
}

fn find_match(view: &Value, id: i64) -> Option<Value> {
    all_matches(view).into_iter().find(|m| m["id"] == id)
}

async fn card(client: &TestClient, league: i64, id: i64) -> Value {
    let view = matches(client, league).await;
    find_match(&view, id).unwrap_or_else(|| panic!("Match {id} not in the view: {view}"))
}

#[tokio::test]
async fn a_scheduled_start_override_moves_the_lock_in_its_league_only() {
    let s = setup().await;
    let (bob, rivals) = s.other_league("bob").await;
    let later = s.app.clock.now() + Duration::hours(5);

    s.set(
        &s.alice,
        s.league,
        101,
        json!({ "field": "scheduled_start", "scheduled_start": later }),
    )
    .await;

    let mine = card(&s.alice, s.league, 101).await;
    assert_eq!(mine["scheduled_start"], json!(later));
    assert_eq!(
        mine["lock"]["locks_at"],
        json!(later - Duration::minutes(15))
    );
    let theirs = card(&bob, rivals, 101).await;
    assert_eq!(
        theirs["scheduled_start"],
        json!(s.app.clock.now() + Duration::hours(2)),
        "League B is unaffected: {theirs}"
    );

    // 1h50 later League B has Locked, League A is still open.
    s.app.clock.advance(Duration::minutes(110));
    assert_eq!(card(&bob, rivals, 101).await["lock"]["locked"], true);
    assert_eq!(card(&s.alice, s.league, 101).await["lock"]["locked"], false);
}

#[tokio::test]
async fn a_lock_that_happened_stays_when_an_override_postpones_the_match() {
    let s = setup().await;
    s.app.clock.advance(Duration::minutes(106)); // 14 minutes before the start
    assert_eq!(card(&s.alice, s.league, 101).await["lock"]["locked"], true);

    let later = s.app.clock.now() + Duration::hours(5);
    s.set(
        &s.alice,
        s.league,
        101,
        json!({ "field": "scheduled_start", "scheduled_start": later }),
    )
    .await;

    let game = card(&s.alice, s.league, 101).await;
    assert_eq!(game["scheduled_start"], json!(later));
    assert_eq!(game["lock"]["locked"], true, "ADR-0003: {game}");
    let res = s
        .alice
        .put(
            &format!("/api/leagues/{}/matches/101/prediction", s.league),
            &json!({ "team_id": NAVI }),
        )
        .await;
    assert_eq!(res.error_code(), "match_locked");
}

#[tokio::test]
async fn an_override_keeps_winning_over_later_source_updates_until_it_is_removed() {
    let s = setup().await;
    let mine = s.app.clock.now() + Duration::hours(5);
    s.set(
        &s.alice,
        s.league,
        101,
        json!({ "field": "scheduled_start", "scheduled_start": mine }),
    )
    .await;

    let theirs = s.app.clock.now() + Duration::hours(3);
    s.app.source.postpone(EventId(EVENT), MatchId(101), theirs);
    s.poll().await;
    assert_eq!(
        card(&s.alice, s.league, 101).await["scheduled_start"],
        json!(mine)
    );

    let res = s
        .alice
        .delete(&format!(
            "/api/leagues/{}/matches/101/overrides/scheduled_start",
            s.league
        ))
        .await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    assert_eq!(
        card(&s.alice, s.league, 101).await["scheduled_start"],
        json!(theirs),
        "the Data Source's value applies again"
    );
}

async fn leaderboard(client: &TestClient, league: i64) -> Vec<(String, i64)> {
    let res = client
        .get(&format!("/api/leagues/{league}/leaderboard"))
        .await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    res.json()["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| {
            (
                r["nickname"].as_str().unwrap().to_string(),
                r["points"].as_i64().unwrap(),
            )
        })
        .collect()
}

#[tokio::test]
async fn result_and_void_overrides_recalculate_the_leaderboard_of_their_league_only() {
    let s = setup().await;
    let bob = s.member("bob").await;
    let (carol, rivals) = s.other_league("carol").await;
    for (client, league, team) in [
        (&s.alice, s.league, NAVI),
        (&bob, s.league, FAZE),
        (&carol, rivals, NAVI),
    ] {
        let res = client
            .put(
                &format!("/api/leagues/{league}/matches/101/prediction"),
                &json!({ "team_id": team }),
            )
            .await;
        assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    }
    s.app
        .source
        .finish(EventId(EVENT), MatchId(101), "NAVI", (2, 0));
    s.app.clock.advance(Duration::hours(3));
    poller::tick(&s.app.state).await;
    assert_eq!(
        leaderboard(&s.alice, s.league).await,
        vec![("alice".into(), 1), ("bob".into(), 0)]
    );

    s.set(
        &s.alice,
        s.league,
        101,
        json!({ "field": "result", "winner": FAZE }),
    )
    .await;
    assert_eq!(
        leaderboard(&s.alice, s.league).await,
        vec![("bob".into(), 1), ("alice".into(), 0)]
    );
    let game = card(&s.alice, s.league, 101).await;
    assert_eq!(game["winner"], FAZE);
    assert_eq!(game["my_result"], "wrong");
    assert_eq!(game["overridden"], json!(["result"]));

    s.set(
        &s.alice,
        s.league,
        101,
        json!({ "field": "result", "void": true }),
    )
    .await;
    assert_eq!(
        leaderboard(&s.alice, s.league).await,
        vec![("alice".into(), 0), ("bob".into(), 0)]
    );
    assert_eq!(card(&s.alice, s.league, 101).await["my_result"], "void");

    assert_eq!(
        leaderboard(&carol, rivals).await,
        vec![("carol".into(), 1)],
        "the other League keeps the Data Source's result"
    );
}

/// A7: a winner is always one of the effective Match's two teams. A result
/// Override whose winner a later teams Override took out of the Match is not
/// applied: the result is absent, not a win for a team that did not play.
#[tokio::test]
async fn a_result_override_for_a_team_a_teams_override_removed_is_not_applied() {
    let s = setup().await;
    let bob = s.member("bob").await;
    s.set(&s.alice, s.league, 201, json!({ "field": "result", "winner": SPIRIT })).await;
    s.set(
        &s.alice,
        s.league,
        201,
        json!({ "field": "teams", "team_a": NAVI, "team_b": FAZE }),
    )
    .await;

    let game = card(&bob, s.league, 201).await;
    assert_eq!(game["team_b"]["id"], FAZE);
    assert_eq!(game["winner"], Value::Null, "Spirit does not play this Match: {game}");
    assert_ne!(game["status"], "finished", "{game}");
}

/// A7: the same holds for base data: a stored winner who is not one of the
/// Match's teams (inconsistent source data) is an absent result.
#[tokio::test]
async fn a_source_winner_who_does_not_play_the_match_is_an_absent_result() {
    let s = setup().await;
    s.app.source.update_match(EventId(EVENT), MatchId(101), |m| {
        m.status = kallbet::domain::MatchStatus::Finished;
        m.winner = Some(kallbet::domain::TeamId(SPIRIT));
    });
    s.poll().await;

    let game = card(&s.alice, s.league, 101).await;
    assert_eq!(game["winner"], Value::Null, "{game}");
    assert_ne!(game["my_result"], "void", "{game}");
}

#[tokio::test]
async fn a_winner_override_on_the_final_archives_that_league_at_once() {
    let s = setup().await;
    let (bob, rivals) = s.other_league("bob").await;

    s.set(
        &s.alice,
        s.league,
        301,
        json!({ "field": "result", "winner": NAVI }),
    )
    .await;

    let mine = s
        .alice
        .get(&format!("/api/leagues/{}", s.league))
        .await
        .json();
    assert!(!mine["archived_at"].is_null(), "{mine}");
    let theirs = bob.get(&format!("/api/leagues/{rivals}")).await.json();
    assert!(theirs["archived_at"].is_null());
}

#[tokio::test]
async fn archived_leagues_reject_overrides() {
    let s = setup().await;
    s.set(
        &s.alice,
        s.league,
        301,
        json!({ "field": "result", "winner": NAVI }),
    )
    .await;

    let base = format!("/api/leagues/{}", s.league);
    let set = s
        .alice
        .post(
            &format!("{base}/matches/201/overrides"),
            &json!({ "field": "hidden" }),
        )
        .await;
    let remove = s
        .alice
        .delete(&format!("{base}/matches/301/overrides/result"))
        .await;
    let add = s
        .alice
        .post(
            &format!("{base}/matches"),
            &json!({ "stage": 1, "team_a": NAVI, "team_b": SPIRIT,
                     "scheduled_start": s.app.clock.now() + Duration::days(1) }),
        )
        .await;
    for res in [set, remove, add] {
        assert_eq!(res.status, StatusCode::CONFLICT, "{}", res.text());
        assert_eq!(res.error_code(), "league_archived");
    }
}

#[tokio::test]
async fn only_league_admins_create_overrides() {
    let s = setup().await;
    let bob = s.member("bob").await;
    let (_, rivals) = s.other_league("carol").await;

    let res = bob
        .post(
            &format!("/api/leagues/{}/matches/101/overrides", s.league),
            &json!({ "field": "hidden" }),
        )
        .await;
    assert_eq!(res.status, StatusCode::FORBIDDEN);
    let res = bob
        .post(
            &format!("/api/leagues/{}/matches", s.league),
            &json!({ "stage": 1, "team_a": NAVI, "team_b": SPIRIT,
                     "scheduled_start": s.app.clock.now() + Duration::days(1) }),
        )
        .await;
    assert_eq!(res.status, StatusCode::FORBIDDEN);
    let res = s
        .alice
        .post(
            &format!("/api/leagues/{rivals}/matches/101/overrides"),
            &json!({ "field": "hidden" }),
        )
        .await;
    assert_eq!(
        res.status,
        StatusCode::NOT_FOUND,
        "not a Member of that League"
    );
}

#[tokio::test]
async fn invalid_overrides_are_rejected() {
    let s = setup().await;
    let url = format!("/api/leagues/{}/matches/101/overrides", s.league);
    for (body, code) in [
        (
            json!({ "field": "result", "winner": SPIRIT }),
            "team_not_in_match",
        ),
        (
            json!({ "field": "teams", "team_a": NAVI, "team_b": NAVI }),
            "same_team",
        ),
        (
            json!({ "field": "teams", "team_a": NAVI, "team_b": 999 }),
            "team_not_in_event",
        ),
        (json!({ "field": "result" }), "override_invalid"),
        (json!({ "field": "colour" }), "override_invalid"),
    ] {
        let res = s.alice.post(&url, &body).await;
        assert_eq!(res.status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
        assert_eq!(res.error_code(), code, "{body}");
    }
    let res = s
        .alice
        .post(
            &format!("/api/leagues/{}/matches/999/overrides", s.league),
            &json!({ "field": "hidden" }),
        )
        .await;
    assert_eq!(res.status, StatusCode::NOT_FOUND);
}

async fn user_id(client: &TestClient) -> UserId {
    UserId(client.get("/api/me").await.json()["id"].as_i64().unwrap())
}

#[tokio::test]
async fn a_hidden_match_disappears_for_members_and_from_scoring_and_notifications() {
    let s = setup().await;
    let bob = s.member("bob").await;
    let (carol, rivals) = s.other_league("carol").await;
    s.set(&s.alice, s.league, 101, json!({ "field": "hidden" }))
        .await;

    assert!(find_match(&matches(&bob, s.league).await, 101).is_none());
    assert!(find_match(&matches(&carol, rivals).await, 101).is_some());
    let res = bob
        .put(
            &format!("/api/leagues/{}/matches/101/prediction", s.league),
            &json!({ "team_id": NAVI }),
        )
        .await;
    assert_eq!(res.status, StatusCode::NOT_FOUND);

    // Inside the reminder window of 101 (Lock at +1h45).
    s.app.clock.advance(Duration::minutes(80));
    poller::tick(&s.app.state).await;
    assert!(s.app.notifier.sent_to(user_id(&bob).await).is_empty());
    assert!(
        !s.app.notifier.sent_to(user_id(&carol).await).is_empty(),
        "the other League is reminded"
    );

    // Members see it listed as hidden; unhiding brings it back.
    let list = bob
        .get(&format!("/api/leagues/{}/overrides", s.league))
        .await
        .json();
    assert_eq!(list["hidden"][0]["id"], 101, "{list}");
    let res = s
        .alice
        .delete(&format!(
            "/api/leagues/{}/matches/101/overrides/hidden",
            s.league
        ))
        .await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    assert!(find_match(&matches(&bob, s.league).await, 101).is_some());
}

#[tokio::test]
async fn a_match_added_by_hand_can_be_predicted_locks_and_scores_in_its_league_only() {
    let s = setup().await;
    let bob = s.member("bob").await;
    let (carol, rivals) = s.other_league("carol").await;
    let start = s.app.clock.now() + Duration::hours(3);

    let res = s
        .alice
        .post(
            &format!("/api/leagues/{}/matches", s.league),
            &json!({ "stage": 1, "team_a": FAZE, "team_b": SPIRIT, "scheduled_start": start,
                     "format": "bo1",
                     "placement": { "kind": "swiss", "round": 1, "wins": 0, "losses": 0 } }),
        )
        .await;
    assert_eq!(res.status, StatusCode::CREATED, "{}", res.text());
    let id = res.json()["id"].as_i64().unwrap();
    assert!(id < 0, "a local id: {id}");

    let game = card(&bob, s.league, id).await;
    assert_eq!(game["team_a"]["name"], "FaZe");
    assert_eq!(game["team_b"]["name"], "Spirit");
    assert_eq!(game["format"], "bo1");
    assert_eq!(game["added_by_hand"], true);
    assert!(game["source"].is_null());
    assert!(find_match(&matches(&carol, rivals).await, id).is_none());

    let res = bob
        .put(
            &format!("/api/leagues/{}/matches/{id}/prediction", s.league),
            &json!({ "team_id": SPIRIT }),
        )
        .await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    s.poll().await; // a poll does not remove it
    assert_eq!(card(&bob, s.league, id).await["my_prediction"], SPIRIT);

    s.app.clock.advance(Duration::hours(3));
    let res = s
        .alice
        .put(
            &format!("/api/leagues/{}/matches/{id}/prediction", s.league),
            &json!({ "team_id": FAZE }),
        )
        .await;
    assert_eq!(res.error_code(), "match_locked");

    s.set(
        &s.alice,
        s.league,
        id,
        json!({ "field": "result", "winner": SPIRIT }),
    )
    .await;
    assert_eq!(
        leaderboard(&bob, s.league).await,
        vec![("bob".into(), 1), ("alice".into(), 0)]
    );
}

#[tokio::test]
async fn members_see_the_override_history_with_author_and_time() {
    let s = setup().await;
    let bob = s.member("bob").await;
    let first = s.app.clock.now();
    let later = first + Duration::hours(5);
    s.set(
        &s.alice,
        s.league,
        101,
        json!({ "field": "scheduled_start", "scheduled_start": later }),
    )
    .await;
    s.app.clock.advance(Duration::minutes(1));
    s.alice
        .delete(&format!(
            "/api/leagues/{}/matches/101/overrides/scheduled_start",
            s.league
        ))
        .await;

    let res = bob
        .get(&format!("/api/leagues/{}/overrides", s.league))
        .await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    let entries = res.json()["entries"].clone();
    assert_eq!(entries.as_array().unwrap().len(), 2, "{entries}");
    let removal = &entries[0];
    assert_eq!(removal["field"], "scheduled_start");
    assert_eq!(removal["cleared"], true);
    assert_eq!(removal["created_at"], json!(first + Duration::minutes(1)));
    let set = &entries[1];
    assert_eq!(set["author"]["nickname"], "alice");
    assert_eq!(set["created_at"], json!(first));
    assert_eq!(set["scheduled_start"], json!(later));
    assert_eq!(set["match"]["team_a"]["name"], "NAVI");

    let outsider = s.app.user("mallory").await;
    let res = outsider
        .get(&format!("/api/leagues/{}/overrides", s.league))
        .await;
    assert_eq!(res.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_teams_override_fixes_the_teams_and_predictions_of_a_team_no_longer_playing_do_not_count()
{
    let s = setup().await;
    let bob = s.member("bob").await;
    s.member("dave").await; // has not predicted, so no Reveal (and Lock) yet
    let (carol, rivals) = s.other_league("carol").await;
    for (client, league, team) in [
        (&s.alice, s.league, FAZE),
        (&bob, s.league, NAVI),
        (&carol, rivals, FAZE),
    ] {
        let res = client
            .put(
                &format!("/api/leagues/{league}/matches/101/prediction"),
                &json!({ "team_id": team }),
            )
            .await;
        assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    }

    // The Data Sources had FaZe where Spirit plays.
    s.set(
        &s.alice,
        s.league,
        101,
        json!({ "field": "teams", "team_a": NAVI, "team_b": SPIRIT }),
    )
    .await;

    let game = card(&s.alice, s.league, 101).await;
    assert_eq!(game["team_b"]["name"], "Spirit");
    assert!(game["my_prediction"].is_null(), "{game}");
    assert_eq!(game["predictions"]["count"], 1);
    let theirs = card(&carol, rivals, 101).await;
    assert_eq!(theirs["team_b"]["name"], "FaZe");
    assert_eq!(theirs["my_prediction"], FAZE);

    let res = s
        .alice
        .put(
            &format!("/api/leagues/{}/matches/101/prediction", s.league),
            &json!({ "team_id": SPIRIT }),
        )
        .await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());

    // The source's result keeps its side: its team B (FaZe) won, which is
    // Spirit in this League.
    s.app
        .source
        .finish(EventId(EVENT), MatchId(101), "FaZe", (2, 1));
    s.app.clock.advance(Duration::hours(3));
    poller::tick(&s.app.state).await;
    let game = card(&s.alice, s.league, 101).await;
    assert_eq!(game["winner"], SPIRIT);
    assert_eq!(game["my_result"], "correct");
    assert_eq!(leaderboard(&carol, rivals).await, vec![("carol".into(), 1)]);
}

#[tokio::test]
async fn data_source_warnings_are_for_league_admins_only() {
    let s = setup().await;
    let bob = s.member("bob").await;
    let url = format!("/api/leagues/{}/data-warnings", s.league);

    assert_eq!(bob.get(&url).await.status, StatusCode::FORBIDDEN);
    let res = s.alice.get(&url).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    let body = res.json();
    assert!(body["conflicts"].as_array().unwrap().is_empty(), "{body}");
    assert!(body["unmatched"].as_array().unwrap().is_empty(), "{body}");
}

#[tokio::test]
async fn deleting_a_league_deletes_its_overrides_and_hand_added_matches() {
    let s = setup().await;
    let res = s
        .alice
        .post(
            &format!("/api/leagues/{}/matches", s.league),
            &json!({ "stage": 1, "team_a": FAZE, "team_b": SPIRIT,
                     "scheduled_start": s.app.clock.now() + Duration::hours(3) }),
        )
        .await;
    assert_eq!(res.status, StatusCode::CREATED, "{}", res.text());
    let id = res.json()["id"].as_i64().unwrap();
    let res = s
        .alice
        .put(
            &format!("/api/leagues/{}/matches/{id}/prediction", s.league),
            &json!({ "team_id": FAZE }),
        )
        .await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    s.set(&s.alice, s.league, 101, json!({ "field": "hidden" }))
        .await;

    let res = s
        .alice
        .post(
            &format!("/api/leagues/{}/delete", s.league),
            &json!({ "confirm_name": "Friends" }),
        )
        .await;
    assert_eq!(res.status, StatusCode::NO_CONTENT, "{}", res.text());

    let (league, _) = create_league(&s.alice, "Friends again").await;
    let view = matches(&s.alice, league).await;
    assert!(find_match(&view, id).is_none());
    assert!(find_match(&view, 101).is_some());
}
