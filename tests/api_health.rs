//! The example Seam 1 test: the router is driven in-process against in-memory
//! SQLite with the fake ports. Copy this shape for new API areas.

use axum::http::StatusCode;
use kallbet::testing::TestApp;
use serde_json::json;

#[tokio::test]
async fn health_reports_ok_once_migrations_are_applied() {
    let app = TestApp::new().await;

    let res = app.anonymous().get("/api/health").await;

    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.json(), json!({ "status": "ok" }));
}

#[tokio::test]
async fn unknown_api_path_is_a_json_404_not_the_spa() {
    let app = TestApp::new().await;

    let res = app.anonymous().get("/api/no-such-thing").await;

    assert_eq!(res.status, StatusCode::NOT_FOUND);
    assert_eq!(res.json(), json!({ "error": { "code": "not_found" } }));
}
