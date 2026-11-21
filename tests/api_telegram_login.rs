use axum::http::StatusCode;
use chrono::Duration;
use kallbet::ports::Clock;
use kallbet::testing::{TelegramLogin, TestApp};
use serde_json::json;

fn alex(app: &TestApp) -> TelegramLogin {
    TelegramLogin::new(app.clock.now(), 1001, "Alex")
        .username("alex")
        .photo_url("https://t.me/i/userpic/320/alex.jpg")
}

#[tokio::test]
async fn first_sign_in_creates_a_user_named_after_the_telegram_name_with_their_avatar() {
    let app = TestApp::new().await;
    let client = app.anonymous();

    let res = client.post("/api/auth/telegram", &alex(&app).last_name("Smith").body()).await;

    assert_eq!(res.status, StatusCode::OK);
    let user = res.json();
    assert_eq!(user["nickname"], "Alex Smith");
    assert_eq!(user["avatar_url"], "https://t.me/i/userpic/320/alex.jpg");
    assert_eq!(user["language"], "en");
    // The session works.
    let me = client.get("/api/me").await;
    assert_eq!(me.status, StatusCode::OK);
    assert_eq!(me.json(), user);
}

#[tokio::test]
async fn the_session_cookie_is_http_only_secure_and_lax() {
    let app = TestApp::new().await;

    let res = app.anonymous().post("/api/auth/telegram", &alex(&app).body()).await;

    let cookie = res.header("set-cookie").expect("a session cookie is set");
    assert!(cookie.starts_with("kallbet_session="), "{cookie}");
    assert!(cookie.contains("HttpOnly"), "{cookie}");
    assert!(cookie.contains("Secure"), "{cookie}");
    assert!(cookie.contains("SameSite=Lax"), "{cookie}");
}

#[tokio::test]
async fn signing_in_again_is_the_same_user_and_keeps_their_edits() {
    let app = TestApp::new().await;
    let first = app.anonymous();
    first.post("/api/auth/telegram", &alex(&app).body()).await;
    first.patch("/api/me", &json!({ "nickname": "Sasha" })).await;

    let second = app.anonymous();
    let res = second.post("/api/auth/telegram", &alex(&app).body()).await;

    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.json()["nickname"], "Sasha");
}

#[tokio::test]
async fn a_payload_with_a_bad_hash_is_rejected_and_starts_no_session() {
    let app = TestApp::new().await;
    let client = app.anonymous();

    let forged = alex(&app).body_signed_with("999:SOME-OTHER-BOT");
    let res = client.post("/api/auth/telegram", &forged).await;

    assert_eq!(res.status, StatusCode::UNAUTHORIZED);
    assert_eq!(res.error_code(), "invalid_login");
    assert_eq!(res.header("set-cookie"), None);
    assert_eq!(client.get("/api/me").await.status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn a_tampered_payload_is_rejected() {
    let app = TestApp::new().await;

    let tampered = alex(&app).body_tampered("id", json!(1));
    let res = app.anonymous().post("/api/auth/telegram", &tampered).await;

    assert_eq!(res.status, StatusCode::UNAUTHORIZED);
    assert_eq!(res.error_code(), "invalid_login");
}

#[tokio::test]
async fn a_payload_without_a_hash_is_rejected() {
    let app = TestApp::new().await;

    let mut body = alex(&app).body();
    body["auth"].as_object_mut().unwrap().remove("hash");
    let res = app.anonymous().post("/api/auth/telegram", &body).await;

    assert_eq!(res.status, StatusCode::UNAUTHORIZED);
    assert_eq!(res.error_code(), "invalid_login");
}

#[tokio::test]
async fn a_correctly_signed_but_stale_payload_is_rejected() {
    let app = TestApp::new().await;
    let login = alex(&app);

    app.clock.advance(Duration::hours(2));
    let res = app.anonymous().post("/api/auth/telegram", &login.body()).await;

    assert_eq!(res.status, StatusCode::UNAUTHORIZED);
    assert_eq!(res.error_code(), "login_expired");
}

#[tokio::test]
async fn a_payload_from_the_future_is_rejected() {
    let app = TestApp::new().await;
    let login = TelegramLogin::new(app.clock.now() + Duration::hours(1), 1001, "Alex");

    let res = app.anonymous().post("/api/auth/telegram", &login.body()).await;

    assert_eq!(res.status, StatusCode::UNAUTHORIZED);
    assert_eq!(res.error_code(), "login_expired");
}

#[tokio::test]
async fn sign_in_is_unavailable_without_a_bot_token() {
    let app = TestApp::builder().config(|c| c.bot_token = None).build().await;

    let res = app.anonymous().post("/api/auth/telegram", &alex(&app).body()).await;

    assert_eq!(res.status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(res.error_code(), "login_unavailable");
}

#[tokio::test]
async fn a_new_user_starts_in_the_language_the_interface_was_showing() {
    let app = TestApp::new().await;

    let res = app
        .anonymous()
        .post("/api/auth/telegram", &alex(&app).language("uk").body())
        .await;

    assert_eq!(res.json()["language"], "uk");
}

#[tokio::test]
async fn the_nickname_is_made_unique_when_the_telegram_name_is_taken() {
    let app = TestApp::new().await;
    app.user("alex").await; // case-insensitive clash with the Telegram name "Alex"

    let first = app.anonymous().post("/api/auth/telegram", &alex(&app).body()).await;
    let other = TelegramLogin::new(app.clock.now(), 1002, "ALEX");
    let second = app.anonymous().post("/api/auth/telegram", &other.body()).await;

    assert_eq!(first.json()["nickname"], "Alex2");
    assert_eq!(second.json()["nickname"], "ALEX3");
}

#[tokio::test]
async fn a_telegram_name_that_is_too_short_or_too_long_still_gives_a_valid_nickname() {
    let app = TestApp::new().await;

    let short = TelegramLogin::new(app.clock.now(), 1, "Al");
    let long = TelegramLogin::new(app.clock.now(), 2, "Maximilian-Alexander-the-Great");
    let short = app.anonymous().post("/api/auth/telegram", &short.body()).await.json();
    let long = app.anonymous().post("/api/auth/telegram", &long.body()).await.json();

    let short = short["nickname"].as_str().unwrap();
    let long = long["nickname"].as_str().unwrap();
    assert!((3..=20).contains(&short.chars().count()), "{short}");
    assert!(short.starts_with("Al"), "{short}");
    assert_eq!(long, "Maximilian-Alexander");
}

#[tokio::test]
async fn logout_invalidates_the_session() {
    let app = TestApp::new().await;
    let client = app.anonymous();
    client.post("/api/auth/telegram", &alex(&app).body()).await;
    // Keep a copy of the session to prove the server forgot it, not just the browser.
    let stolen = app.anonymous();
    let cookie = client_cookie(&client, &app).await;

    let res = client.post_empty("/api/auth/logout").await;

    assert_eq!(res.status, StatusCode::NO_CONTENT);
    assert_eq!(client.get("/api/me").await.status, StatusCode::UNAUTHORIZED);
    let replay = replay(&stolen, &cookie).await;
    assert_eq!(replay, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn logout_without_a_session_is_harmless() {
    let app = TestApp::new().await;

    let res = app.anonymous().post_empty("/api/auth/logout").await;

    assert_eq!(res.status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn the_sign_in_options_tell_the_spa_which_bot_to_show() {
    let app = TestApp::builder()
        .config(|c| c.bot_username = Some("kallbet_bot".into()))
        .dev_login(false)
        .build()
        .await;

    let res = app.anonymous().get("/api/auth/config").await;

    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.json(), json!({ "bot_username": "kallbet_bot", "dev_login": false }));
}

// Helpers to replay a session cookie from outside the client that owns it.
async fn client_cookie(client: &kallbet::testing::TestClient, _app: &TestApp) -> String {
    client.cookie("kallbet_session").expect("signed in")
}

async fn replay(other: &kallbet::testing::TestClient, cookie: &str) -> StatusCode {
    other.set_cookie("kallbet_session", cookie);
    other.get("/api/me").await.status
}
