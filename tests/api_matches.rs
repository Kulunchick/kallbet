//! Seam 1: the poller storing Matches, and the Matches view of a League.

use axum::http::StatusCode;
use chrono::{DateTime, Duration, Utc};
use kallbet::domain::{EventId, MatchId, StageKind};
use kallbet::poller;
use kallbet::ports::Clock;
use kallbet::testing::{EventSnapshotBuilder, TestApp, TestClient};
use serde_json::{Value, json};

const EVENT: i64 = 7148;

fn major(now: DateTime<Utc>) -> EventSnapshotBuilder {
    EventSnapshotBuilder::new(EVENT, "Test Major")
        .stage(StageKind::Swiss, "Stage 1")
        .swiss_match(101, (1, 0, 0), "NAVI", "FaZe", now + Duration::hours(2))
        .swiss_match(102, (1, 0, 0), "Vitality", "G2", now + Duration::hours(3))
        .stage(StageKind::Playoffs, "Playoffs")
        .playoff_match(201, (2, 0), "Spirit", "MOUZ", now + Duration::days(5))
        .playoff_slot(301, (0, 0), now + Duration::days(7))
}

async fn create_league(client: &TestClient) -> i64 {
    let res = client
        .post(
            "/api/leagues",
            &json!({ "event_id": EVENT, "name": "Friends" }),
        )
        .await;
    assert_eq!(res.status, StatusCode::CREATED, "{}", res.text());
    res.json()["id"].as_i64().unwrap()
}

async fn matches(client: &TestClient, league: i64) -> Value {
    let res = client.get(&format!("/api/leagues/{league}/matches")).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    res.json()
}

fn match_ids(group: &Value) -> Vec<i64> {
    group["matches"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["id"].as_i64().unwrap())
        .collect()
}

fn find_match(view: &Value, id: i64) -> Value {
    view["stages"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|s| s["groups"].as_array().unwrap())
        .flat_map(|g| g["matches"].as_array().unwrap())
        .find(|m| m["id"] == id)
        .unwrap_or_else(|| panic!("Match {id} not in the view: {view}"))
        .clone()
}

#[tokio::test]
async fn a_poll_brings_in_the_matches_grouped_by_stage_and_record() {
    let app = TestApp::new().await;
    app.source
        .put_event(major(app.clock.now()).build(app.clock.now()));
    let alice = app.user("alice").await;
    let league = create_league(&alice).await;

    poller::tick(&app.state).await;
    let view = matches(&alice, league).await;

    assert_eq!(view["event"]["name"], "Test Major");
    let stages = view["stages"].as_array().unwrap();
    assert_eq!(stages.len(), 2);
    assert_eq!(stages[0]["kind"], "swiss");
    assert_eq!(stages[0]["name"], "Stage 1");
    let swiss = stages[0]["groups"].as_array().unwrap();
    assert_eq!(swiss.len(), 1);
    assert_eq!(swiss[0]["kind"], "record");
    assert_eq!(swiss[0]["wins"], 0);
    assert_eq!(swiss[0]["losses"], 0);
    assert_eq!(match_ids(&swiss[0]), [101, 102]);

    assert_eq!(stages[1]["kind"], "playoffs");
    let playoffs = stages[1]["groups"].as_array().unwrap();
    assert_eq!(playoffs[0]["kind"], "playoff_round");
    assert_eq!(playoffs[0]["rounds_before_final"], 2);
    assert_eq!(match_ids(&playoffs[0]), [201]);
    assert_eq!(playoffs[1]["rounds_before_final"], 0);
    assert_eq!(match_ids(&playoffs[1]), [301]);

    let navi_faze = find_match(&view, 101);
    assert_eq!(navi_faze["team_a"]["name"], "NAVI");
    assert_eq!(navi_faze["team_b"]["name"], "FaZe");
    assert_eq!(navi_faze["format"], "bo3");
    assert_eq!(navi_faze["status"], "upcoming");
    assert_eq!(
        navi_faze["scheduled_start"],
        json!(app.clock.now() + Duration::hours(2))
    );
    assert_eq!(navi_faze["source"], "hltv");
    assert_eq!(navi_faze["updated_at"], json!(app.clock.now()));
    assert_eq!(navi_faze["my_prediction"], Value::Null);
    let tbd = find_match(&view, 301);
    assert_eq!(tbd["team_a"], Value::Null);
}

/// Fetches of the test Event since the start of the test (the League creation
/// fetches it once too).
fn polls(app: &TestApp) -> usize {
    app.source.fetch_event_calls(EventId(EVENT))
}

#[tokio::test]
async fn only_events_with_a_league_are_polled() {
    let app = TestApp::new().await;
    let now = app.clock.now();
    app.source.put_event(major(now).build(now));
    app.source.put_event(
        EventSnapshotBuilder::new(9000, "Nobody plays this")
            .stage(StageKind::Swiss, "Stage 1")
            .swiss_match(901, (1, 0, 0), "A", "B", now + Duration::minutes(30))
            .build(now),
    );

    poller::tick(&app.state).await;
    assert_eq!(app.source.total_fetch_event_calls(), 0);

    let alice = app.user("alice").await;
    create_league(&alice).await;
    let before = polls(&app);
    for _ in 0..10 {
        app.clock.advance(Duration::minutes(5));
        poller::tick(&app.state).await;
    }

    assert_eq!(polls(&app) - before, 10);
    assert_eq!(app.source.fetch_event_calls(EventId(9000)), 0);
}

#[tokio::test]
async fn matches_are_polled_every_five_minutes_when_nothing_is_close() {
    let app = TestApp::new().await;
    app.source
        .put_event(major(app.clock.now()).build(app.clock.now()));
    let alice = app.user("alice").await;
    create_league(&alice).await;
    let before = polls(&app);

    poller::tick(&app.state).await;
    assert_eq!(polls(&app) - before, 1, "a new Event is polled at once");
    for _ in 0..4 {
        app.clock.advance(Duration::minutes(1));
        poller::tick(&app.state).await;
    }
    assert_eq!(polls(&app) - before, 1);
    app.clock.advance(Duration::minutes(1));
    poller::tick(&app.state).await;
    assert_eq!(polls(&app) - before, 2);
}

#[tokio::test]
async fn matches_are_polled_every_minute_when_one_starts_within_45_minutes() {
    let app = TestApp::new().await;
    app.source
        .put_event(major(app.clock.now()).build(app.clock.now()));
    let alice = app.user("alice").await;
    create_league(&alice).await;
    poller::tick(&app.state).await;

    // Match 101 starts at +2h: 45 minutes before is +1h15m.
    app.clock.advance(Duration::minutes(75));
    poller::tick(&app.state).await;
    let before = polls(&app);
    for _ in 0..3 {
        app.clock.advance(Duration::minutes(1));
        poller::tick(&app.state).await;
    }
    assert_eq!(polls(&app) - before, 3);
}

#[tokio::test]
async fn matches_are_polled_every_minute_while_one_is_live() {
    let app = TestApp::new().await;
    let now = app.clock.now();
    app.source.put_event(major(now).build(now));
    app.source.start(EventId(EVENT), MatchId(201), now);
    let alice = app.user("alice").await;
    create_league(&alice).await;
    poller::tick(&app.state).await;
    let before = polls(&app);

    for _ in 0..3 {
        app.clock.advance(Duration::minutes(1));
        poller::tick(&app.state).await;
    }

    assert_eq!(polls(&app) - before, 3);
}

#[tokio::test]
async fn the_event_list_is_polled_every_six_hours() {
    let app = TestApp::new().await;
    app.source
        .put_event(major(app.clock.now()).build(app.clock.now()));

    poller::tick(&app.state).await;
    assert_eq!(app.source.list_events_calls(), 1);
    app.clock.advance(Duration::hours(6) - Duration::minutes(1));
    poller::tick(&app.state).await;
    assert_eq!(app.source.list_events_calls(), 1);
    app.clock.advance(Duration::minutes(1));
    poller::tick(&app.state).await;
    assert_eq!(app.source.list_events_calls(), 2);
}

#[tokio::test]
async fn a_newly_paired_swiss_round_appears_after_a_poll() {
    let app = TestApp::new().await;
    let now = app.clock.now();
    app.source.put_event(major(now).build(now));
    let alice = app.user("alice").await;
    let league = create_league(&alice).await;
    poller::tick(&app.state).await;

    app.source
        .finish(EventId(EVENT), MatchId(101), "NAVI", (2, 1));
    app.source
        .finish(EventId(EVENT), MatchId(102), "G2", (0, 2));
    // Round 2 is paired: the Data Source now lists two more Matches.
    app.source.update_event(EventId(EVENT), |snapshot| {
        let round_two = EventSnapshotBuilder::new(EVENT, "Test Major")
            .stage(StageKind::Swiss, "Stage 1")
            .swiss_match(111, (2, 1, 0), "NAVI", "G2", now + Duration::hours(6))
            .swiss_match(112, (2, 0, 1), "FaZe", "Vitality", now + Duration::hours(7))
            .build(now);
        for mut m in round_two.matches {
            // Same teams, same ids as in the first snapshot.
            for team in [&mut m.team_a, &mut m.team_b] {
                let name = &round_two
                    .teams
                    .iter()
                    .find(|t| Some(t.id) == *team)
                    .unwrap()
                    .name;
                *team = snapshot.team_named(name).map(|t| t.id);
            }
            snapshot.matches.push(m);
        }
    });
    app.clock.advance(Duration::minutes(5));
    poller::tick(&app.state).await;

    let view = matches(&alice, league).await;
    let groups = view["stages"][0]["groups"].as_array().unwrap();
    let buckets: Vec<(i64, i64)> = groups
        .iter()
        .map(|g| (g["wins"].as_i64().unwrap(), g["losses"].as_i64().unwrap()))
        .collect();
    assert_eq!(buckets, [(0, 0), (1, 0), (0, 1)]);
    assert_eq!(match_ids(&groups[1]), [111]);
    assert_eq!(find_match(&view, 111)["team_a"]["name"], "NAVI");
    assert_eq!(find_match(&view, 112)["team_b"]["name"], "Vitality");
    let finished = find_match(&view, 101);
    assert_eq!(finished["status"], "finished");
    assert_eq!(finished["score"], json!({ "team_a": 2, "team_b": 1 }));
}

#[tokio::test]
async fn only_members_see_the_matches() {
    let app = TestApp::new().await;
    app.source
        .put_event(major(app.clock.now()).build(app.clock.now()));
    let alice = app.user("alice").await;
    let mallory = app.user("mallory").await;
    let league = create_league(&alice).await;

    let res = mallory.get(&format!("/api/leagues/{league}/matches")).await;
    assert_eq!(res.status, StatusCode::NOT_FOUND);
    let res = app
        .anonymous()
        .get(&format!("/api/leagues/{league}/matches"))
        .await;
    assert_eq!(res.status, StatusCode::UNAUTHORIZED);
}
