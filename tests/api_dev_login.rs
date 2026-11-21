use axum::http::StatusCode;
use kallbet::testing::TestApp;
use serde_json::json;

#[tokio::test]
async fn anonymous_visitor_has_no_user() {
    let app = TestApp::new().await;

    let res = app.anonymous().get("/api/me").await;

    assert_eq!(res.status, StatusCode::UNAUTHORIZED);
    assert_eq!(res.json(), json!({ "error": { "code": "unauthorized" } }));
}

#[tokio::test]
async fn dev_login_signs_in_as_a_user_with_that_nickname() {
    let app = TestApp::new().await;

    let alice = app.user("alice").await;
    let res = alice.get("/api/me").await;

    assert_eq!(res.status, StatusCode::OK);
    let me = res.json();
    assert_eq!(me["nickname"], "alice");
    assert_eq!(me["language"], "en");
}

#[tokio::test]
async fn dev_login_twice_with_the_same_nickname_is_the_same_user() {
    let app = TestApp::new().await;

    let first = app.user("alice").await.get("/api/me").await.json();
    let second = app.user("Alice").await.get("/api/me").await.json();
    let bob = app.user("bob").await.get("/api/me").await.json();

    assert_eq!(first["id"], second["id"]);
    assert_ne!(first["id"], bob["id"]);
}

#[tokio::test]
async fn dev_login_session_cookie_is_http_only_secure_and_lax() {
    let app = TestApp::new().await;

    let res = app
        .anonymous()
        .post("/api/dev/login", &json!({ "nickname": "alice" }))
        .await;

    assert_eq!(res.status, StatusCode::OK);
    let cookie = res.header("set-cookie").expect("a session cookie is set");
    assert!(cookie.starts_with("kallbet_session="), "{cookie}");
    assert!(cookie.contains("HttpOnly"), "{cookie}");
    assert!(cookie.contains("Secure"), "{cookie}");
    assert!(cookie.contains("SameSite=Lax"), "{cookie}");
}

#[tokio::test]
async fn dev_login_does_not_exist_unless_enabled_by_configuration() {
    let app = TestApp::builder().dev_login(false).build().await;

    let client = app.anonymous();
    let res = client
        .post("/api/dev/login", &json!({ "nickname": "mallory" }))
        .await;

    assert_eq!(res.status, StatusCode::NOT_FOUND);
    assert_eq!(res.header("set-cookie"), None);
    assert_eq!(client.get("/api/me").await.status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn a_session_expires() {
    let app = TestApp::new().await;
    let alice = app.user("alice").await;

    app.clock.advance(chrono::Duration::days(31));

    assert_eq!(alice.get("/api/me").await.status, StatusCode::UNAUTHORIZED);
}
