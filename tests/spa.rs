use axum::http::StatusCode;
use kallbet::testing::TestApp;

async fn app_with_built_spa() -> (TestApp, tempfile::TempDir) {
    let dist = tempfile::tempdir().unwrap();
    std::fs::write(dist.path().join("index.html"), "<html>KallBET</html>").unwrap();
    std::fs::create_dir(dist.path().join("assets")).unwrap();
    std::fs::write(dist.path().join("assets/app.js"), "console.log(1)").unwrap();
    let app = TestApp::builder().web_dir(dist.path()).build().await;
    (app, dist)
}

#[tokio::test]
async fn serves_the_built_spa_and_its_assets() {
    let (app, _dist) = app_with_built_spa().await;

    let index = app.anonymous().get("/").await;
    assert_eq!(index.status, StatusCode::OK);
    assert_eq!(index.text(), "<html>KallBET</html>");

    let asset = app.anonymous().get("/assets/app.js").await;
    assert_eq!(asset.status, StatusCode::OK);
    assert_eq!(asset.text(), "console.log(1)");
}

#[tokio::test]
async fn client_side_routes_fall_back_to_the_spa_shell() {
    let (app, _dist) = app_with_built_spa().await;

    let res = app.anonymous().get("/leagues/42").await;

    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.text(), "<html>KallBET</html>");
}
