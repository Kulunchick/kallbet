use axum::http::StatusCode;
use kallbet::domain::{EventId, StageKind};
use kallbet::ports::{Clock, SourceError};
use kallbet::testing::{EventSnapshotBuilder, TestApp, TestClient};
use serde_json::{Value, json};

const EVENT: i64 = 7148;

async fn app_with_event() -> TestApp {
    let app = TestApp::new().await;
    app.source.put_event(
        EventSnapshotBuilder::new(EVENT, "Test Major")
            .stage(StageKind::Swiss, "Stage 1")
            .build(app.clock.now()),
    );
    app
}

async fn create(client: &TestClient, name: &str) -> Value {
    let res = client
        .post("/api/leagues", &json!({ "event_id": EVENT, "name": name }))
        .await;
    assert_eq!(res.status, StatusCode::CREATED, "{}", res.text());
    res.json()
}

fn nicknames(league: &Value) -> Vec<String> {
    league["members"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["nickname"].as_str().unwrap().to_string())
        .collect()
}

fn token(league: &Value) -> String {
    league["invite_token"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn events_are_listed_through_the_match_source() {
    let app = app_with_event().await;
    let alice = app.user("alice").await;

    let res = alice.get("/api/events").await;

    assert_eq!(res.status, StatusCode::OK);
    let events = res.json();
    assert_eq!(events[0]["id"], EVENT);
    assert_eq!(events[0]["name"], "Test Major");
    assert_eq!(app.source.list_events_calls(), 1);
}

#[tokio::test]
async fn events_need_a_session() {
    let app = app_with_event().await;
    assert_eq!(
        app.anonymous().get("/api/events").await.status,
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn a_failing_source_is_reported_as_unavailable() {
    let app = app_with_event().await;
    app.source
        .fail_with(SourceError::Unavailable("down".into()));
    let alice = app.user("alice").await;

    let res = alice.get("/api/events").await;

    assert_eq!(res.status, StatusCode::BAD_GATEWAY);
    assert_eq!(res.error_code(), "source_unavailable");
}

#[tokio::test]
async fn an_hltv_link_resolves_to_its_event() {
    let app = app_with_event().await;
    let alice = app.user("alice").await;

    let res = alice
        .post(
            "/api/events/resolve",
            &json!({ "link": "https://www.hltv.org/events/7148/test-major-2026?foo=1" }),
        )
        .await;

    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    assert_eq!(res.json()["id"], EVENT);
    assert_eq!(app.source.fetch_event_calls(EventId(EVENT)), 1);
}

#[tokio::test]
async fn an_invalid_link_gets_an_error_code() {
    let app = app_with_event().await;
    let alice = app.user("alice").await;

    for link in [
        "not a link",
        "https://example.com/events/7148/x",
        "https://www.hltv.org/matches/7148/x",
        "https://www.hltv.org/events/abc/x",
        "https://www.hltv.org.evil.com/events/7148/x",
    ] {
        let res = alice
            .post("/api/events/resolve", &json!({ "link": link }))
            .await;
        assert_eq!(res.status, StatusCode::UNPROCESSABLE_ENTITY, "{link}");
        assert_eq!(res.error_code(), "invalid_hltv_link", "{link}");
    }
    assert_eq!(app.source.total_fetch_event_calls(), 0);
}

#[tokio::test]
async fn a_link_to_an_unknown_event_is_event_not_found() {
    let app = app_with_event().await;
    let alice = app.user("alice").await;

    let res = alice
        .post(
            "/api/events/resolve",
            &json!({ "link": "https://www.hltv.org/events/1/nothing" }),
        )
        .await;

    assert_eq!(res.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(res.error_code(), "event_not_found");
}

#[tokio::test]
async fn creating_a_league_makes_the_creator_its_admin_member() {
    let app = app_with_event().await;
    let alice = app.user("alice").await;

    let league = create(&alice, "  Friends  ").await;

    assert_eq!(league["name"], "Friends");
    assert_eq!(league["event"]["name"], "Test Major");
    assert_eq!(league["my_role"], "admin");
    assert_eq!(league["is_creator"], true);
    assert_eq!(league["members"][0]["nickname"], "alice");
    assert_eq!(league["members"][0]["role"], "admin");
    assert!(league["invite_token"].as_str().unwrap().len() >= 32);
}

#[tokio::test]
async fn league_names_and_events_are_validated() {
    let app = app_with_event().await;
    let alice = app.user("alice").await;

    let res = alice
        .post("/api/leagues", &json!({ "event_id": EVENT, "name": "   " }))
        .await;
    assert_eq!(res.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(res.error_code(), "league_name_invalid");

    let long = "x".repeat(61);
    let res = alice
        .post("/api/leagues", &json!({ "event_id": EVENT, "name": long }))
        .await;
    assert_eq!(res.error_code(), "league_name_invalid");

    let res = alice
        .post("/api/leagues", &json!({ "event_id": 999, "name": "ok" }))
        .await;
    assert_eq!(res.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(res.error_code(), "event_not_found");
}

#[tokio::test]
async fn my_leagues_lists_each_league_with_its_event() {
    let app = app_with_event().await;
    let alice = app.user("alice").await;
    let bob = app.user("bob").await;
    create(&alice, "One").await;
    create(&alice, "Two").await;
    create(&bob, "Bobs").await;

    let mine = alice.get("/api/leagues").await.json();

    let names: Vec<_> = mine
        .as_array()
        .unwrap()
        .iter()
        .map(|l| l["name"].as_str().unwrap())
        .collect();
    assert_eq!(names.len(), 2);
    assert!(names.contains(&"One") && names.contains(&"Two"));
    assert_eq!(mine[0]["event"]["name"], "Test Major");
    assert_eq!(mine[0]["my_role"], "admin");
}

#[tokio::test]
async fn non_members_cannot_see_a_league() {
    let app = app_with_event().await;
    let alice = app.user("alice").await;
    let mallory = app.user("mallory").await;
    let id = create(&alice, "Secret").await["id"].as_i64().unwrap();

    assert_eq!(
        mallory.get(&format!("/api/leagues/{id}")).await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        mallory.get("/api/leagues/9999").await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        mallory
            .post_empty(&format!("/api/leagues/{id}/leave"))
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        mallory
            .post_empty(&format!("/api/leagues/{id}/invite/reissue"))
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(mallory.get("/api/leagues").await.json(), json!([]));
    assert_eq!(
        app.anonymous()
            .get(&format!("/api/leagues/{id}"))
            .await
            .status,
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn joining_through_the_invite_link_makes_a_member() {
    let app = app_with_event().await;
    let alice = app.user("alice").await;
    let bob = app.user("bob").await;
    let league = create(&alice, "Friends").await;
    let id = league["id"].as_i64().unwrap();
    let token = token(&league);

    let preview = bob.get(&format!("/api/invites/{token}")).await;
    assert_eq!(preview.status, StatusCode::OK);
    assert_eq!(preview.json()["name"], "Friends");
    assert_eq!(preview.json()["event"]["name"], "Test Major");

    let res = bob.post_empty(&format!("/api/invites/{token}/join")).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    assert_eq!(res.json()["id"], id);

    let seen = bob.get(&format!("/api/leagues/{id}")).await.json();
    assert_eq!(seen["my_role"], "member");
    assert_eq!(nicknames(&seen), ["alice", "bob"]);
    // Only admins hold the Invite Link.
    assert!(seen.get("invite_token").is_none_or(|t| t.is_null()));
}

#[tokio::test]
async fn joining_twice_does_not_duplicate_the_member() {
    let app = app_with_event().await;
    let alice = app.user("alice").await;
    let bob = app.user("bob").await;
    let league = create(&alice, "Friends").await;
    let token = token(&league);

    bob.post_empty(&format!("/api/invites/{token}/join")).await;
    let again = bob.post_empty(&format!("/api/invites/{token}/join")).await;

    assert_eq!(again.status, StatusCode::OK);
    let seen = alice
        .get(&format!("/api/leagues/{}", league["id"]))
        .await
        .json();
    assert_eq!(nicknames(&seen), ["alice", "bob"]);
}

#[tokio::test]
async fn a_user_can_join_after_the_event_has_started() {
    let app = app_with_event().await;
    let alice = app.user("alice").await;
    let bob = app.user("bob").await;
    let league = create(&alice, "Late").await;
    app.clock.advance(chrono::Duration::days(20));

    let res = bob
        .post_empty(&format!("/api/invites/{}/join", token(&league)))
        .await;

    assert_eq!(res.status, StatusCode::OK);
}

#[tokio::test]
async fn an_unknown_invite_is_invalid() {
    let app = app_with_event().await;
    let bob = app.user("bob").await;

    let res = bob.post_empty("/api/invites/nope/join").await;
    assert_eq!(res.status, StatusCode::NOT_FOUND);
    assert_eq!(res.error_code(), "invite_invalid");
    assert_eq!(
        bob.get("/api/invites/nope").await.error_code(),
        "invite_invalid"
    );
}

#[tokio::test]
async fn a_reissued_link_invalidates_the_old_one() {
    let app = app_with_event().await;
    let alice = app.user("alice").await;
    let bob = app.user("bob").await;
    let league = create(&alice, "Friends").await;
    let id = league["id"].as_i64().unwrap();
    let old = token(&league);

    let res = alice
        .post_empty(&format!("/api/leagues/{id}/invite/reissue"))
        .await;
    assert_eq!(res.status, StatusCode::OK);
    let new = token(&res.json());
    assert_ne!(new, old);

    let stale = bob.post_empty(&format!("/api/invites/{old}/join")).await;
    assert_eq!(stale.status, StatusCode::NOT_FOUND);
    assert_eq!(stale.error_code(), "invite_invalid");
    assert_eq!(
        bob.post_empty(&format!("/api/invites/{new}/join"))
            .await
            .status,
        StatusCode::OK
    );
}

#[tokio::test]
async fn only_admins_can_reissue_the_link() {
    let app = app_with_event().await;
    let alice = app.user("alice").await;
    let bob = app.user("bob").await;
    let league = create(&alice, "Friends").await;
    let id = league["id"].as_i64().unwrap();
    bob.post_empty(&format!("/api/invites/{}/join", token(&league)))
        .await;

    let res = bob
        .post_empty(&format!("/api/leagues/{id}/invite/reissue"))
        .await;

    assert_eq!(res.status, StatusCode::FORBIDDEN);
    assert_eq!(res.error_code(), "forbidden");
}

#[tokio::test]
async fn a_user_can_be_in_several_leagues_for_the_same_event() {
    let app = app_with_event().await;
    let alice = app.user("alice").await;
    let bob = app.user("bob").await;
    let first = create(&alice, "First").await;
    let second = create(&alice, "Second").await;
    for league in [&first, &second] {
        bob.post_empty(&format!("/api/invites/{}/join", token(league)))
            .await;
    }

    let mine = bob.get("/api/leagues").await.json();

    assert_eq!(mine.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn leaving_removes_the_membership() {
    let app = app_with_event().await;
    let alice = app.user("alice").await;
    let bob = app.user("bob").await;
    let league = create(&alice, "Friends").await;
    let id = league["id"].as_i64().unwrap();
    let token = token(&league);
    bob.post_empty(&format!("/api/invites/{token}/join")).await;

    let res = bob.post_empty(&format!("/api/leagues/{id}/leave")).await;

    assert_eq!(res.status, StatusCode::NO_CONTENT);
    assert_eq!(
        bob.get(&format!("/api/leagues/{id}")).await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(bob.get("/api/leagues").await.json(), json!([]));
    let seen = alice.get(&format!("/api/leagues/{id}")).await.json();
    assert_eq!(nicknames(&seen), ["alice"]);

    // Leaving twice: no longer a Member, so the League does not exist for bob.
    assert_eq!(
        bob.post_empty(&format!("/api/leagues/{id}/leave"))
            .await
            .status,
        StatusCode::NOT_FOUND
    );

    // A User who left can join again.
    assert_eq!(
        bob.post_empty(&format!("/api/invites/{token}/join"))
            .await
            .status,
        StatusCode::OK
    );
    let seen = alice.get(&format!("/api/leagues/{id}")).await.json();
    assert_eq!(nicknames(&seen), ["alice", "bob"]);
}

#[tokio::test]
async fn the_creator_cannot_leave() {
    let app = app_with_event().await;
    let alice = app.user("alice").await;
    let id = create(&alice, "Friends").await["id"].as_i64().unwrap();

    let res = alice.post_empty(&format!("/api/leagues/{id}/leave")).await;

    assert_eq!(res.status, StatusCode::CONFLICT);
    assert_eq!(res.error_code(), "creator_cannot_leave");
}
