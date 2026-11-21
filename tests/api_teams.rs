//! Team logos: downloaded once from the Data Source, then served from our
//! side at `/api/teams/{id}/logo`, never hotlinked. The "Data Source" here
//! is a local HTTP server standing in for HLTV's image CDN.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use axum::Router;
use axum::http::{StatusCode, header};
use axum::routing::get;
use kallbet::domain::{Team, TeamId};
use kallbet::sources::hltv::LogoCache;
use kallbet::testing::TestApp;

const PNG: &[u8] = b"\x89PNG\r\n\x1a\nnot really a png";
const SVG: &[u8] = b"<svg xmlns=\"http://www.w3.org/2000/svg\"/>";

/// Serves `/teamlogo/old.png` and `/teamlogo/new.svg`; counts requests.
async fn image_cdn() -> (String, Arc<AtomicUsize>) {
    let hits = Arc::new(AtomicUsize::new(0));
    let (h1, h2) = (hits.clone(), hits.clone());
    let app = Router::new()
        .route(
            "/teamlogo/old.png",
            get(move || async move {
                h1.fetch_add(1, Ordering::SeqCst);
                ([(header::CONTENT_TYPE, "image/png")], PNG)
            }),
        )
        .route(
            "/teamlogo/new.svg",
            get(move || async move {
                h2.fetch_add(1, Ordering::SeqCst);
                ([(header::CONTENT_TYPE, "image/svg+xml")], SVG)
            }),
        );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (format!("http://{addr}"), hits)
}

fn team(logo_url: String) -> Team {
    Team { id: TeamId(8297), name: "FURIA".into(), logo_url: Some(logo_url) }
}

#[tokio::test]
async fn a_logo_is_downloaded_once_and_served_from_our_side() {
    let (cdn, hits) = image_cdn().await;
    let dir = tempfile::tempdir().unwrap();
    let logos = LogoCache::new(dir.path());
    let furia = team(format!("{cdn}/teamlogo/old.png?w=50&s=abc"));

    logos.ensure(&furia).await.unwrap();
    logos.ensure(&furia).await.unwrap();
    assert_eq!(hits.load(Ordering::SeqCst), 1);

    let logo_dir = dir.path().to_path_buf();
    let app = TestApp::builder().config(|c| c.logo_dir = logo_dir).build().await;
    let res = app.anonymous().get("/api/teams/8297/logo").await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.header("content-type"), Some("image/png"));
    assert_eq!(res.body.as_slice(), PNG);
}

#[tokio::test]
async fn a_new_logo_replaces_the_old_one() {
    let (cdn, hits) = image_cdn().await;
    let dir = tempfile::tempdir().unwrap();
    let logos = LogoCache::new(dir.path());

    logos.ensure(&team(format!("{cdn}/teamlogo/old.png"))).await.unwrap();
    logos.ensure(&team(format!("{cdn}/teamlogo/new.svg"))).await.unwrap();
    assert_eq!(hits.load(Ordering::SeqCst), 2);

    let logo_dir = dir.path().to_path_buf();
    let app = TestApp::builder().config(|c| c.logo_dir = logo_dir).build().await;
    let res = app.anonymous().get("/api/teams/8297/logo").await;
    assert_eq!(res.header("content-type"), Some("image/svg+xml"));
    assert_eq!(res.body.as_slice(), SVG);
}

#[tokio::test]
async fn a_team_without_a_stored_logo_is_not_found() {
    let dir = tempfile::tempdir().unwrap();
    let logo_dir = dir.path().to_path_buf();
    let app = TestApp::builder().config(|c| c.logo_dir = logo_dir).build().await;

    let res = app.anonymous().get("/api/teams/1/logo").await;
    assert_eq!(res.status, StatusCode::NOT_FOUND);
    assert_eq!(res.error_code(), "not_found");
}

#[tokio::test]
async fn a_failed_download_is_an_error_and_stores_nothing() {
    let (cdn, _) = image_cdn().await;
    let dir = tempfile::tempdir().unwrap();
    let logos = LogoCache::new(dir.path());

    assert!(logos.ensure(&team(format!("{cdn}/teamlogo/missing.png"))).await.is_err());

    let logo_dir = dir.path().to_path_buf();
    let app = TestApp::builder().config(|c| c.logo_dir = logo_dir).build().await;
    let res = app.anonymous().get("/api/teams/8297/logo").await;
    assert_eq!(res.status, StatusCode::NOT_FOUND);
}
