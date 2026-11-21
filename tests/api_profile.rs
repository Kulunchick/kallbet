use axum::http::StatusCode;
use kallbet::ports::Clock;
use kallbet::testing::{TelegramLogin, TestApp};
use serde_json::json;

#[tokio::test]
async fn a_user_can_change_their_nickname() {
    let app = TestApp::new().await;
    let alice = app.user("alice").await;

    let res = alice.patch("/api/me", &json!({ "nickname": "Alice the Great" })).await;

    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.json()["nickname"], "Alice the Great");
    assert_eq!(alice.get("/api/me").await.json()["nickname"], "Alice the Great");
}

#[tokio::test]
async fn the_nickname_is_trimmed() {
    let app = TestApp::new().await;
    let alice = app.user("alice").await;

    let res = alice.patch("/api/me", &json!({ "nickname": "  Ali  " })).await;

    assert_eq!(res.json()["nickname"], "Ali");
}

#[tokio::test]
async fn nicknames_of_3_to_20_characters_are_accepted() {
    let app = TestApp::new().await;
    let alice = app.user("alice").await;

    for nickname in ["abc", "Кіт", "Олександр Петренко 1", "12345678901234567890"] {
        let res = alice.patch("/api/me", &json!({ "nickname": nickname })).await;
        assert_eq!(res.status, StatusCode::OK, "{nickname}");
    }
}

#[tokio::test]
async fn nicknames_shorter_than_3_or_longer_than_20_characters_are_rejected() {
    let app = TestApp::new().await;
    let alice = app.user("alice").await;

    for nickname in ["", "  ", "ab", " a ", "123456789012345678901", "Олександр Петренко 12"] {
        let res = alice.patch("/api/me", &json!({ "nickname": nickname })).await;
        assert_eq!(res.status, StatusCode::UNPROCESSABLE_ENTITY, "{nickname:?}");
        assert_eq!(res.error_code(), "nickname_invalid", "{nickname:?}");
    }
    assert_eq!(alice.get("/api/me").await.json()["nickname"], "alice");
}

#[tokio::test]
async fn nicknames_with_control_characters_are_rejected() {
    let app = TestApp::new().await;
    let alice = app.user("alice").await;

    let res = alice.patch("/api/me", &json!({ "nickname": "bad\nname" })).await;

    assert_eq!(res.error_code(), "nickname_invalid");
}

#[tokio::test]
async fn a_nickname_taken_by_someone_else_is_a_conflict_whatever_its_case() {
    let app = TestApp::new().await;
    app.user("Bobby").await;
    let alice = app.user("alice").await;

    for nickname in ["Bobby", "bobby", "BOBBY"] {
        let res = alice.patch("/api/me", &json!({ "nickname": nickname })).await;
        assert_eq!(res.status, StatusCode::CONFLICT, "{nickname}");
        assert_eq!(res.error_code(), "nickname_taken", "{nickname}");
    }
    assert_eq!(alice.get("/api/me").await.json()["nickname"], "alice");
}

#[tokio::test]
async fn a_user_can_change_the_case_of_their_own_nickname() {
    let app = TestApp::new().await;
    let alice = app.user("alice").await;

    let res = alice.patch("/api/me", &json!({ "nickname": "ALICE" })).await;

    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.json()["nickname"], "ALICE");
}

#[tokio::test]
async fn a_released_nickname_can_be_taken() {
    let app = TestApp::new().await;
    let alice = app.user("alice").await;
    let bob = app.user("bobby").await;
    bob.patch("/api/me", &json!({ "nickname": "robert" })).await;

    let res = alice.patch("/api/me", &json!({ "nickname": "bobby" })).await;

    assert_eq!(res.status, StatusCode::OK);
}

#[tokio::test]
async fn the_language_is_changed_and_remembered_across_sessions() {
    let app = TestApp::new().await;
    let first = app.anonymous();
    first
        .post("/api/auth/telegram", &TelegramLogin::new(app.clock.now(), 7, "Olena").body())
        .await;

    let res = first.patch("/api/me", &json!({ "language": "uk" })).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.json()["language"], "uk");

    let later = app.anonymous();
    let signed_in = later
        .post("/api/auth/telegram", &TelegramLogin::new(app.clock.now(), 7, "Olena").body())
        .await;
    assert_eq!(signed_in.json()["language"], "uk");
    assert_eq!(later.get("/api/me").await.json()["language"], "uk");
}

#[tokio::test]
async fn an_unsupported_language_is_rejected() {
    let app = TestApp::new().await;
    let alice = app.user("alice").await;

    let res = alice.patch("/api/me", &json!({ "language": "de" })).await;

    assert_eq!(res.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(res.error_code(), "language_invalid");
    assert_eq!(alice.get("/api/me").await.json()["language"], "en");
}

#[tokio::test]
async fn an_update_changes_only_the_given_fields() {
    let app = TestApp::new().await;
    let alice = app.user("alice").await;
    alice.patch("/api/me", &json!({ "language": "ru" })).await;

    let res = alice.patch("/api/me", &json!({ "nickname": "alicia" })).await;

    let me = res.json();
    assert_eq!(me["nickname"], "alicia");
    assert_eq!(me["language"], "ru");
}

#[tokio::test]
async fn a_failed_update_changes_nothing() {
    let app = TestApp::new().await;
    app.user("bobby").await;
    let alice = app.user("alice").await;

    let res = alice
        .patch("/api/me", &json!({ "nickname": "bobby", "language": "uk" }))
        .await;

    assert_eq!(res.status, StatusCode::CONFLICT);
    assert_eq!(alice.get("/api/me").await.json()["language"], "en");
}

#[tokio::test]
async fn editing_the_profile_requires_a_session() {
    let app = TestApp::new().await;

    let res = app.anonymous().patch("/api/me", &json!({ "nickname": "mallory" })).await;

    assert_eq!(res.status, StatusCode::UNAUTHORIZED);
}
