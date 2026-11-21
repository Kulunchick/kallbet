use axum::http::StatusCode;
use kallbet::domain::StageKind;
use kallbet::ports::Clock;
use kallbet::testing::{EventSnapshotBuilder, TestApp, TestClient};
use serde_json::{Value, json};

const EVENT: i64 = 7148;

/// A League with `owner` as creator, `coadmin` appointed, and `member`, `other` as plain Members.
struct Setup {
    id: i64,
    owner: TestClient,
    coadmin: TestClient,
    member: TestClient,
    other: TestClient,
    stranger: TestClient,
    ids: Ids,
}

struct Ids {
    owner: i64,
    coadmin: i64,
    member: i64,
    other: i64,
}

async fn user_id(client: &TestClient) -> i64 {
    client.get("/api/me").await.json()["id"].as_i64().unwrap()
}

async fn setup() -> Setup {
    let app = TestApp::new().await;
    app.source.put_event(
        EventSnapshotBuilder::new(EVENT, "Test Major")
            .stage(StageKind::Swiss, "Stage 1")
            .build(app.clock.now()),
    );
    let owner = app.user("owner").await;
    let coadmin = app.user("coadmin").await;
    let member = app.user("member").await;
    let other = app.user("other").await;
    let stranger = app.user("stranger").await;
    let res = owner
        .post("/api/leagues", &json!({ "event_id": EVENT, "name": "Friends" }))
        .await;
    assert_eq!(res.status, StatusCode::CREATED);
    let league: Value = res.json();
    let id = league["id"].as_i64().unwrap();
    let token = league["invite_token"].as_str().unwrap().to_string();
    for c in [&coadmin, &member, &other] {
        let res = c.post_empty(&format!("/api/invites/{token}/join")).await;
        assert_eq!(res.status, StatusCode::OK);
    }
    let ids = Ids {
        owner: user_id(&owner).await,
        coadmin: user_id(&coadmin).await,
        member: user_id(&member).await,
        other: user_id(&other).await,
    };
    let res = owner
        .put(
            &format!("/api/leagues/{id}/members/{}/role", ids.coadmin),
            &json!({ "role": "admin" }),
        )
        .await;
    assert_eq!(res.status, StatusCode::NO_CONTENT, "{}", res.text());
    Setup { id, owner, coadmin, member, other, stranger, ids }
}

fn kick_path(s: &Setup, user: i64) -> String {
    format!("/api/leagues/{}/members/{user}", s.id)
}

fn role_path(s: &Setup, user: i64) -> String {
    format!("/api/leagues/{}/members/{user}/role", s.id)
}

async fn nicknames(client: &TestClient, id: i64) -> Vec<String> {
    let league: Value = client.get(&format!("/api/leagues/{id}")).await.json();
    league["members"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["nickname"].as_str().unwrap().to_string())
        .collect()
}

#[tokio::test]
async fn a_member_cannot_kick() {
    let s = setup().await;
    let res = s.member.delete(&kick_path(&s, s.ids.other)).await;
    assert_eq!(res.status, StatusCode::FORBIDDEN);
    assert_eq!(res.error_code(), "forbidden");
    assert!(nicknames(&s.owner, s.id).await.contains(&"other".to_string()));
}

#[tokio::test]
async fn a_non_member_gets_not_found_on_every_management_endpoint() {
    let s = setup().await;
    let target = s.ids.member;
    assert_eq!(s.stranger.delete(&kick_path(&s, target)).await.status, StatusCode::NOT_FOUND);
    assert_eq!(
        s.stranger.put(&role_path(&s, target), &json!({ "role": "admin" })).await.status,
        StatusCode::NOT_FOUND
    );
    let res = s
        .stranger
        .post(&format!("/api/leagues/{}/delete", s.id), &json!({ "confirm_name": "Friends" }))
        .await;
    assert_eq!(res.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_league_admin_kicks_a_member_who_then_loses_access() {
    let s = setup().await;
    let res = s.coadmin.delete(&kick_path(&s, s.ids.member)).await;
    assert_eq!(res.status, StatusCode::NO_CONTENT, "{}", res.text());

    assert!(!nicknames(&s.owner, s.id).await.contains(&"member".to_string()));
    let res = s.member.get(&format!("/api/leagues/{}", s.id)).await;
    assert_eq!(res.status, StatusCode::NOT_FOUND);
    assert!(s.member.get("/api/leagues").await.json().as_array().unwrap().is_empty());
}

#[tokio::test]
async fn the_creator_kicks_a_co_admin() {
    let s = setup().await;
    let res = s.owner.delete(&kick_path(&s, s.ids.coadmin)).await;
    assert_eq!(res.status, StatusCode::NO_CONTENT);
    assert_eq!(s.coadmin.get(&format!("/api/leagues/{}", s.id)).await.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn the_creator_cannot_be_kicked() {
    let s = setup().await;
    for admin in [&s.coadmin, &s.owner] {
        let res = admin.delete(&kick_path(&s, s.ids.owner)).await;
        assert_eq!(res.status, StatusCode::CONFLICT);
        assert_eq!(res.error_code(), "creator_protected");
    }
    assert!(nicknames(&s.member, s.id).await.contains(&"owner".to_string()));
}

#[tokio::test]
async fn kicking_someone_who_is_not_a_member_is_not_found() {
    let s = setup().await;
    let stranger_id = user_id(&s.stranger).await;
    let res = s.owner.delete(&kick_path(&s, stranger_id)).await;
    assert_eq!(res.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn an_admin_cannot_kick_themselves() {
    let s = setup().await;
    let res = s.coadmin.delete(&kick_path(&s, s.ids.coadmin)).await;
    assert_eq!(res.status, StatusCode::CONFLICT);
    assert_eq!(res.error_code(), "use_leave");
}

#[tokio::test]
async fn a_kicked_member_can_come_back_through_the_invite_link() {
    let s = setup().await;
    s.owner.delete(&kick_path(&s, s.ids.member)).await;
    let league: Value = s.owner.get(&format!("/api/leagues/{}", s.id)).await.json();
    let token = league["invite_token"].as_str().unwrap();
    let res = s.member.post_empty(&format!("/api/invites/{token}/join")).await;
    assert_eq!(res.status, StatusCode::OK);
    assert!(nicknames(&s.owner, s.id).await.contains(&"member".to_string()));
}

#[tokio::test]
async fn members_cannot_change_roles() {
    let s = setup().await;
    let res = s.member.put(&role_path(&s, s.ids.other), &json!({ "role": "admin" })).await;
    assert_eq!(res.status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn the_creator_and_co_admins_appoint_and_remove_co_admins() {
    let s = setup().await;
    // A co-admin appoints another.
    let res = s.coadmin.put(&role_path(&s, s.ids.member), &json!({ "role": "admin" })).await;
    assert_eq!(res.status, StatusCode::NO_CONTENT);
    let league: Value = s.member.get(&format!("/api/leagues/{}", s.id)).await.json();
    assert_eq!(league["my_role"], "admin");
    assert!(league.get("invite_token").is_some());
    // The creator removes one; a co-admin removes another.
    let res = s.owner.put(&role_path(&s, s.ids.member), &json!({ "role": "member" })).await;
    assert_eq!(res.status, StatusCode::NO_CONTENT);
    let res = s.coadmin.put(&role_path(&s, s.ids.coadmin), &json!({ "role": "member" })).await;
    assert_eq!(res.status, StatusCode::NO_CONTENT);
    let league: Value = s.coadmin.get(&format!("/api/leagues/{}", s.id)).await.json();
    assert_eq!(league["my_role"], "member");
}

#[tokio::test]
async fn the_creator_cannot_be_demoted() {
    let s = setup().await;
    for admin in [&s.coadmin, &s.owner] {
        let res = admin.put(&role_path(&s, s.ids.owner), &json!({ "role": "member" })).await;
        assert_eq!(res.status, StatusCode::CONFLICT);
        assert_eq!(res.error_code(), "creator_protected");
    }
    let league: Value = s.owner.get(&format!("/api/leagues/{}", s.id)).await.json();
    assert_eq!(league["my_role"], "admin");
}

#[tokio::test]
async fn an_unknown_role_or_target_is_rejected() {
    let s = setup().await;
    let res = s.owner.put(&role_path(&s, s.ids.member), &json!({ "role": "owner" })).await;
    assert!(res.status.is_client_error());
    let stranger_id = user_id(&s.stranger).await;
    let res = s.owner.put(&role_path(&s, stranger_id), &json!({ "role": "admin" })).await;
    assert_eq!(res.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn only_the_creator_can_delete_and_only_when_confirmed() {
    let s = setup().await;
    let path = format!("/api/leagues/{}/delete", s.id);
    let body = json!({ "confirm_name": "Friends" });

    for client in [&s.member, &s.coadmin] {
        let res = client.post(&path, &body).await;
        assert_eq!(res.status, StatusCode::FORBIDDEN);
    }
    // A wrong confirmation deletes nothing.
    let res = s.owner.post(&path, &json!({ "confirm_name": "friends?" })).await;
    assert_eq!(res.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(res.error_code(), "confirmation_mismatch");
    assert_eq!(s.owner.get(&format!("/api/leagues/{}", s.id)).await.status, StatusCode::OK);

    let res = s.owner.post(&path, &body).await;
    assert_eq!(res.status, StatusCode::NO_CONTENT, "{}", res.text());
    for client in [&s.owner, &s.coadmin, &s.member, &s.other] {
        assert_eq!(client.get(&format!("/api/leagues/{}", s.id)).await.status, StatusCode::NOT_FOUND);
        assert!(client.get("/api/leagues").await.json().as_array().unwrap().is_empty());
    }
}

#[tokio::test]
async fn a_deleted_leagues_invite_link_stops_working_and_the_event_stays_usable() {
    let s = setup().await;
    let league: Value = s.owner.get(&format!("/api/leagues/{}", s.id)).await.json();
    let token = league["invite_token"].as_str().unwrap().to_string();
    let res = s
        .owner
        .post(&format!("/api/leagues/{}/delete", s.id), &json!({ "confirm_name": "Friends" }))
        .await;
    assert_eq!(res.status, StatusCode::NO_CONTENT);
    let res = s.stranger.get(&format!("/api/invites/{token}")).await;
    assert_eq!(res.error_code(), "invite_invalid");
    let again = s
        .owner
        .post("/api/leagues", &json!({ "event_id": EVENT, "name": "Again" }))
        .await;
    assert_eq!(again.status, StatusCode::CREATED);
}
