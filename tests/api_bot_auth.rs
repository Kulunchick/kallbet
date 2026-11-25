//! Seam 1: Bot User authentication (spec #21, ticket #22; ADR-0004's
//! amendment). A sidecar holding the shared bot secret registers a Bot User
//! by its bot key and gets a Bearer token that is that Bot User's
//! `CurrentUser`.

use axum::http::StatusCode;
use kallbet::testing::{TEST_BOT_SECRET, TestApp, TestClient};
use serde_json::{Value, json};

fn oracle() -> Value {
    json!({ "bot_key": "oracle", "nickname": "Oracle" })
}

/// Registers through the endpoint with the right secret and returns the
/// response body.
async fn register(app: &TestApp, body: &Value) -> Value {
    let res = app
        .bearer(TEST_BOT_SECRET)
        .post("/api/auth/bot", body)
        .await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    res.json()
}

async fn me(client: &TestClient) -> Value {
    let res = client.get("/api/me").await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    res.json()
}

#[tokio::test]
async fn without_a_configured_secret_bot_auth_is_unavailable() {
    let app = TestApp::builder()
        .config(|c| c.bot_secret = None)
        .build()
        .await;

    for client in [app.anonymous(), app.bearer(TEST_BOT_SECRET), app.bearer("")] {
        let res = client.post("/api/auth/bot", &oracle()).await;
        assert_eq!(
            res.status,
            StatusCode::SERVICE_UNAVAILABLE,
            "{}",
            res.text()
        );
        assert_eq!(res.error_code(), "bot_auth_unavailable");
    }
}

#[tokio::test]
async fn a_wrong_or_missing_secret_is_unauthorized() {
    let app = TestApp::new().await;

    for client in [
        app.anonymous(),
        app.bearer("not-the-secret"),
        app.bearer(&format!("{TEST_BOT_SECRET}x")),
        app.bearer(&TEST_BOT_SECRET[1..]),
        app.bearer(""),
    ] {
        let res = client.post("/api/auth/bot", &oracle()).await;
        assert_eq!(res.status, StatusCode::UNAUTHORIZED, "{}", res.text());
        assert_eq!(res.error_code(), "unauthorized");
    }
    // Nothing was registered: the Nickname is still free for a human.
    let human = app.user("Oracle").await;
    assert_eq!(me(&human).await["is_bot"], false);
}

#[tokio::test]
async fn the_secret_is_checked_before_the_body() {
    let app = TestApp::new().await;

    let res = app
        .bearer("not-the-secret")
        .post("/api/auth/bot", &json!({ "nonsense": true }))
        .await;

    assert_eq!(res.status, StatusCode::UNAUTHORIZED, "{}", res.text());
}

#[tokio::test]
async fn registering_a_bot_key_creates_a_bot_user_with_a_bearer_token() {
    let app = TestApp::new().await;

    let body = register(
        &app,
        &json!({
            "bot_key": "oracle",
            "nickname": "Oracle",
            "avatar_url": "https://example.test/oracle.png",
            "language": "uk",
        }),
    )
    .await;

    let user = &body["user"];
    assert_eq!(user["is_bot"], true, "{body}");
    assert_eq!(user["nickname"], "Oracle");
    assert_eq!(user["avatar_url"], "https://example.test/oracle.png");
    assert_eq!(user["language"], "uk");
    // Telegram notifications are off: nobody to message.
    assert_eq!(
        user["notifications"],
        json!({ "reminder": false, "reveal": false, "digest": false })
    );
    // The token is that Bot User's `CurrentUser`.
    let bot = app.bearer(body["token"].as_str().unwrap());
    assert_eq!(&me(&bot).await, user);
}

#[tokio::test]
async fn a_human_user_is_not_a_bot() {
    let app = TestApp::new().await;
    assert_eq!(me(&app.user("alice").await).await["is_bot"], false);
}

#[tokio::test]
async fn registering_the_same_bot_key_again_reuses_the_bot_user_and_updates_its_profile() {
    let app = TestApp::new().await;
    let first = register(&app, &oracle()).await;

    let second = register(
        &app,
        &json!({
            "bot_key": "oracle",
            "nickname": "Oracle v2",
            "avatar_url": "https://example.test/v2.png",
            "language": "ru",
        }),
    )
    .await;

    assert_eq!(second["user"]["id"], first["user"]["id"]);
    assert_eq!(second["user"]["nickname"], "Oracle v2");
    assert_eq!(second["user"]["avatar_url"], "https://example.test/v2.png");
    assert_eq!(second["user"]["language"], "ru");
    assert_eq!(second["user"]["is_bot"], true);
    let bot = app.bearer(second["token"].as_str().unwrap());
    assert_eq!(me(&bot).await, second["user"]);

    // Unchanged input changes nothing; a profile field left out keeps its
    // value, except the avatar, which the profile then does not have.
    let third = register(
        &app,
        &json!({ "bot_key": "oracle", "nickname": "Oracle v2" }),
    )
    .await;
    assert_eq!(third["user"]["id"], first["user"]["id"]);
    assert_eq!(third["user"]["nickname"], "Oracle v2");
    assert_eq!(third["user"]["language"], "ru");
    assert_eq!(third["user"]["avatar_url"], Value::Null);
}

#[tokio::test]
async fn each_registration_issues_a_new_token_and_revokes_the_previous_one() {
    let app = TestApp::new().await;
    let first = register(&app, &oracle()).await;
    let second = register(&app, &oracle()).await;

    assert_ne!(first["token"], second["token"]);
    let old = app.bearer(first["token"].as_str().unwrap());
    assert_eq!(old.get("/api/me").await.status, StatusCode::UNAUTHORIZED);
    let new = app.bearer(second["token"].as_str().unwrap());
    assert_eq!(me(&new).await["id"], first["user"]["id"]);
}

#[tokio::test]
async fn different_bot_keys_are_different_bot_users() {
    let app = TestApp::new().await;
    let oracle = app.bot("oracle", "Oracle").await;
    let market = app.bot("market", "Market Favourite").await;

    let (oracle, market) = (me(&oracle).await, me(&market).await);
    assert_ne!(oracle["id"], market["id"]);
    assert_eq!(market["nickname"], "Market Favourite");
    assert_eq!(oracle["nickname"], "Oracle");
}

#[tokio::test]
async fn a_taken_nickname_gets_a_free_variant_and_keeps_it() {
    let app = TestApp::new().await;
    let human = app.user("Oracle").await;

    let first = register(&app, &oracle()).await;
    assert_eq!(first["user"]["nickname"], "Oracle2");
    // Case does not make it free either, and another bot takes the next one.
    let other = register(&app, &json!({ "bot_key": "other", "nickname": "ORACLE" })).await;
    assert_eq!(other["user"]["nickname"], "ORACLE3");

    // Asking again settles on the same Nickname instead of drifting.
    let again = register(&app, &oracle()).await;
    assert_eq!(again["user"]["id"], first["user"]["id"]);
    assert_eq!(again["user"]["nickname"], "Oracle2");
    // The human keeps theirs.
    assert_eq!(me(&human).await["nickname"], "Oracle");
    assert_eq!(me(&human).await["is_bot"], false);

    // A rename onto a taken Nickname is collision-safe too.
    let renamed = register(&app, &json!({ "bot_key": "other", "nickname": "Oracle2" })).await;
    assert_eq!(renamed["user"]["nickname"], "Oracle22");
}

#[tokio::test]
async fn an_invalid_profile_is_rejected_and_registers_nothing() {
    let app = TestApp::new().await;
    let long_key = "k".repeat(65);
    let long_url = format!("https://example.test/{}", "a".repeat(600));

    for (body, code) in [
        (
            json!({ "bot_key": "", "nickname": "Oracle" }),
            "bot_key_invalid",
        ),
        (
            json!({ "bot_key": long_key, "nickname": "Oracle" }),
            "bot_key_invalid",
        ),
        (
            json!({ "bot_key": "has space", "nickname": "Oracle" }),
            "bot_key_invalid",
        ),
        (
            json!({ "bot_key": "oracle", "nickname": "ab" }),
            "nickname_invalid",
        ),
        (
            json!({ "bot_key": "oracle", "nickname": "a nickname far too long to keep" }),
            "nickname_invalid",
        ),
        (
            json!({ "bot_key": "oracle", "nickname": "Oracle", "language": "de" }),
            "language_invalid",
        ),
        (
            json!({ "bot_key": "oracle", "nickname": "Oracle", "avatar_url": "javascript:alert(1)" }),
            "avatar_url_invalid",
        ),
        (
            json!({ "bot_key": "oracle", "nickname": "Oracle", "avatar_url": long_url }),
            "avatar_url_invalid",
        ),
    ] {
        let res = app
            .bearer(TEST_BOT_SECRET)
            .post("/api/auth/bot", &body)
            .await;
        assert_eq!(
            res.status,
            StatusCode::UNPROCESSABLE_ENTITY,
            "{body}: {}",
            res.text()
        );
        assert_eq!(res.error_code(), code, "{body}");
    }
    for body in [
        json!({ "nickname": "Oracle" }),
        json!({ "bot_key": "oracle" }),
        json!([]),
    ] {
        let res = app
            .bearer(TEST_BOT_SECRET)
            .post("/api/auth/bot", &body)
            .await;
        assert_eq!(
            res.status,
            StatusCode::BAD_REQUEST,
            "{body}: {}",
            res.text()
        );
        assert_eq!(res.error_code(), "invalid_body", "{body}");
    }

    // The Nickname was not taken by any of the attempts.
    assert_eq!(
        register(&app, &oracle()).await["user"]["nickname"],
        "Oracle"
    );
}

#[tokio::test]
async fn an_unknown_bearer_token_is_nobody() {
    let app = TestApp::new().await;
    app.bot("oracle", "Oracle").await;

    for client in [
        app.bearer("0000000000000000000000000000000000000000000000000000000000000000"),
        // The shared secret registers Bot Users; it is not one itself.
        app.bearer(TEST_BOT_SECRET),
    ] {
        let res = client.get("/api/me").await;
        assert_eq!(res.status, StatusCode::UNAUTHORIZED, "{}", res.text());
        assert_eq!(res.error_code(), "unauthorized");
    }
}
