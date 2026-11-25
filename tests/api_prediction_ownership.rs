//! Seam 1: no API path lets anyone create or change another Member's
//! Prediction, not a League Admin and not the creator (user story 79, #12),
//! and not a Bot User's Bearer token (spec #21, #22).

use axum::http::StatusCode;
use chrono::Duration;
use kallbet::domain::StageKind;
use kallbet::poller;
use kallbet::ports::Clock;
use kallbet::testing::{EventSnapshotBuilder, TestApp, TestClient};
use serde_json::{Value, json};

const EVENT: i64 = 7148;
const NAVI_FAZE: i64 = 101;
const NAVI: i64 = 1;
const FAZE: i64 = 2;

async fn user_id(client: &TestClient) -> i64 {
    client.get("/api/me").await.json()["id"].as_i64().unwrap()
}

async fn my_prediction(client: &TestClient, league: i64) -> Value {
    let res = client.get(&format!("/api/leagues/{league}/matches")).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    res.json()["stages"][0]["groups"][0]["matches"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["id"] == NAVI_FAZE)
        .unwrap()["my_prediction"]
        .clone()
}

#[tokio::test]
async fn neither_an_admin_nor_the_creator_can_predict_for_another_member() {
    let app = TestApp::new().await;
    let now = app.clock.now();
    app.source.put_event(
        EventSnapshotBuilder::new(EVENT, "Test Major")
            .stage(StageKind::Swiss, "Stage 1")
            .swiss_match(
                NAVI_FAZE,
                (1, 0, 0),
                "NAVI",
                "FaZe",
                now + Duration::hours(2),
            )
            .build(now),
    );
    let creator = app.user("alice").await;
    let admin = app.user("bob").await;
    let carol = app.user("carol").await; // has predicted
    let dave = app.user("dave").await; // has not, and keeps the Match open
    let res = creator
        .post(
            "/api/leagues",
            &json!({ "event_id": EVENT, "name": "Friends" }),
        )
        .await;
    let league = res.json()["id"].as_i64().unwrap();
    let token = res.json()["invite_token"].as_str().unwrap().to_string();
    for c in [&admin, &carol, &dave] {
        c.post_empty(&format!("/api/invites/{token}/join")).await;
    }
    let admin_id = user_id(&admin).await;
    let res = creator
        .put(
            &format!("/api/leagues/{league}/members/{admin_id}/role"),
            &json!({ "role": "admin" }),
        )
        .await;
    assert!(res.status.is_success(), "{}", res.text());
    poller::tick(&app.state).await;
    let base = format!("/api/leagues/{league}/matches/{NAVI_FAZE}/prediction");
    let res = carol.put(&base, &json!({ "team_id": NAVI })).await;
    assert_eq!(res.status, StatusCode::OK);
    let carol_id = user_id(&carol).await;
    let dave_id = user_id(&dave).await;

    for writer in [&creator, &admin] {
        for target in [carol_id, dave_id] {
            // Naming the target in the body or the query only writes for the
            // caller.
            for body in [
                json!({ "team_id": FAZE, "user_id": target }),
                json!({ "team_id": FAZE, "user": target, "member_id": target }),
            ] {
                let res = writer.put(&base, &body).await;
                assert_eq!(res.status, StatusCode::OK, "{}", res.text());
                assert_ne!(res.json()["user_id"], target);
            }
            let res = writer
                .put(
                    &format!("{base}?user_id={target}"),
                    &json!({ "team_id": FAZE }),
                )
                .await;
            assert_ne!(res.json()["user_id"], target);

            // There is no route that takes another Member.
            for path in [
                format!("{base}/{target}"),
                format!("/api/leagues/{league}/members/{target}/prediction"),
                format!("/api/leagues/{league}/members/{target}/predictions/{NAVI_FAZE}"),
                format!("/api/leagues/{league}/predictions/{target}"),
            ] {
                let body = json!({ "team_id": FAZE });
                for res in [
                    writer.put(&path, &body).await,
                    writer.post(&path, &body).await,
                    writer.patch(&path, &body).await,
                ] {
                    assert!(
                        res.status == StatusCode::NOT_FOUND
                            || res.status == StatusCode::METHOD_NOT_ALLOWED,
                        "{path}: {}",
                        res.status
                    );
                }
            }
            for method_res in [
                writer
                    .post(&base, &json!({ "team_id": FAZE, "user_id": target }))
                    .await,
                writer
                    .patch(&base, &json!({ "team_id": FAZE, "user_id": target }))
                    .await,
            ] {
                assert_eq!(method_res.status, StatusCode::METHOD_NOT_ALLOWED);
            }
        }
    }

    assert_eq!(my_prediction(&carol, league).await, NAVI);
    assert_eq!(my_prediction(&dave, league).await, Value::Null);
}

/// Spec #21, ticket #22: a Bot User's Bearer token writes only that Bot
/// User's own Prediction, and a human cannot write a Bot User's.
#[tokio::test]
async fn a_bearer_token_predicts_only_for_its_own_bot_user() {
    let app = TestApp::new().await;
    let now = app.clock.now();
    app.source.put_event(
        EventSnapshotBuilder::new(EVENT, "Test Major")
            .stage(StageKind::Swiss, "Stage 1")
            .swiss_match(
                NAVI_FAZE,
                (1, 0, 0),
                "NAVI",
                "FaZe",
                now + Duration::hours(2),
            )
            .build(now),
    );
    let creator = app.user("alice").await;
    let carol = app.user("carol").await; // has predicted
    let dave = app.user("dave").await; // has not, and keeps the Match open
    let oracle = app.bot("oracle", "Oracle").await;
    let market = app.bot("market", "Market Favourite").await; // has predicted
    let res = creator
        .post(
            "/api/leagues",
            &json!({ "event_id": EVENT, "name": "Friends" }),
        )
        .await;
    let league = res.json()["id"].as_i64().unwrap();
    let token = res.json()["invite_token"].as_str().unwrap().to_string();
    for c in [&carol, &dave, &oracle, &market] {
        let res = c.post_empty(&format!("/api/invites/{token}/join")).await;
        assert!(res.status.is_success(), "{}", res.text());
    }
    poller::tick(&app.state).await;
    let base = format!("/api/leagues/{league}/matches/{NAVI_FAZE}/prediction");
    for c in [&carol, &market] {
        let res = c.put(&base, &json!({ "team_id": NAVI })).await;
        assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    }
    let oracle_id = user_id(&oracle).await;
    let (carol_id, dave_id, market_id) = (
        user_id(&carol).await,
        user_id(&dave).await,
        user_id(&market).await,
    );

    // The Bot User names a human or another Bot User: it only ever writes
    // its own Prediction.
    for target in [carol_id, dave_id, market_id] {
        for body in [
            json!({ "team_id": FAZE, "user_id": target }),
            json!({ "team_id": FAZE, "user": target, "member_id": target, "bot_key": "market" }),
        ] {
            let res = oracle.put(&base, &body).await;
            assert_eq!(res.status, StatusCode::OK, "{}", res.text());
            assert_eq!(res.json()["user_id"], oracle_id);
        }
        let res = oracle
            .put(
                &format!("{base}?user_id={target}"),
                &json!({ "team_id": FAZE }),
            )
            .await;
        assert_eq!(res.json()["user_id"], oracle_id);
        for path in [
            format!("{base}/{target}"),
            format!("/api/leagues/{league}/members/{target}/prediction"),
            format!("/api/leagues/{league}/predictions/{target}"),
        ] {
            let body = json!({ "team_id": FAZE });
            for res in [
                oracle.put(&path, &body).await,
                oracle.post(&path, &body).await,
                oracle.patch(&path, &body).await,
            ] {
                assert!(
                    res.status == StatusCode::NOT_FOUND
                        || res.status == StatusCode::METHOD_NOT_ALLOWED,
                    "{path}: {}",
                    res.status
                );
            }
        }
    }
    // Nor can a human, the creator included, write a Bot User's.
    for body in [
        json!({ "team_id": FAZE, "user_id": market_id }),
        json!({ "team_id": FAZE, "bot_key": "market" }),
    ] {
        let res = creator.put(&base, &body).await;
        assert_eq!(res.status, StatusCode::OK, "{}", res.text());
        assert_ne!(res.json()["user_id"], market_id);
    }

    assert_eq!(my_prediction(&oracle, league).await, FAZE);
    assert_eq!(my_prediction(&market, league).await, NAVI);
    assert_eq!(my_prediction(&carol, league).await, NAVI);
    assert_eq!(my_prediction(&dave, league).await, Value::Null);
}

/// A token is a Bot User's and nothing more: an unknown one, or the shared
/// bot secret itself, predicts for nobody.
#[tokio::test]
async fn a_bearer_token_that_is_no_bot_users_cannot_predict() {
    let app = TestApp::new().await;
    let now = app.clock.now();
    app.source.put_event(
        EventSnapshotBuilder::new(EVENT, "Test Major")
            .stage(StageKind::Swiss, "Stage 1")
            .swiss_match(
                NAVI_FAZE,
                (1, 0, 0),
                "NAVI",
                "FaZe",
                now + Duration::hours(2),
            )
            .build(now),
    );
    let creator = app.user("alice").await;
    let res = creator
        .post(
            "/api/leagues",
            &json!({ "event_id": EVENT, "name": "Friends" }),
        )
        .await;
    let league = res.json()["id"].as_i64().unwrap();
    poller::tick(&app.state).await;
    let base = format!("/api/leagues/{league}/matches/{NAVI_FAZE}/prediction");

    for client in [
        app.bearer(kallbet::testing::TEST_BOT_SECRET),
        app.bearer("f00f00f00f00f00f00f00f00f00f00f00f00f00f00f00f00f00f00f00f00f00f"),
        app.anonymous(),
    ] {
        let res = client.put(&base, &json!({ "team_id": FAZE })).await;
        assert_eq!(res.status, StatusCode::UNAUTHORIZED, "{}", res.text());
    }
    assert_eq!(my_prediction(&creator, league).await, Value::Null);
}
