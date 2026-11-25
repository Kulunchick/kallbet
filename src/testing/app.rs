//! The Seam 1 harness: the real router, in-process, on in-memory SQLite,
//! with the fake ports.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use axum::Router;
use axum::body::Body;
use axum::http::{HeaderMap, Method, Request, StatusCode, header};
use http_body_util::BodyExt;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use tower::ServiceExt;

use super::{FakeClock, RecordingNotifier, ScriptedSource, TEST_BOT_SECRET};
use crate::config::Config;
use crate::ports::{Clock, MatchSource};
use sqlx::SqlitePool;
use crate::state::AppState;
use crate::{app, db};

/// A whole KallBet in memory. Each test builds its own; nothing is shared.
pub struct TestApp {
    /// Starts at a fixed instant; move it with `advance` / `set`.
    pub clock: Arc<FakeClock>,
    /// The scripted HLTV behind the `MatchSource` port (unless the builder
    /// was given another source).
    pub source: Arc<ScriptedSource>,
    pub notifier: Arc<RecordingNotifier>,
    /// The state the router runs on, for driving non-HTTP entry points (the
    /// poller) against the same database and fakes.
    pub state: AppState,
    router: Router,
}

impl TestApp {
    /// The default test app: dev login on, no built SPA.
    pub async fn new() -> TestApp {
        TestApp::builder().build().await
    }

    pub fn builder() -> TestAppBuilder {
        TestAppBuilder {
            config: Config::for_tests(),
            source: None,
        }
    }

    /// A client without a session.
    pub fn anonymous(&self) -> TestClient {
        TestClient {
            router: self.router.clone(),
            cookies: Mutex::new(BTreeMap::new()),
            bearer: None,
        }
    }

    /// A client that sends `Authorization: Bearer <token>` and holds no
    /// session: how a Bot User (or the sidecar with the shared bot secret)
    /// talks to the API.
    pub fn bearer(&self, token: &str) -> TestClient {
        TestClient {
            bearer: Some(token.to_string()),
            ..self.anonymous()
        }
    }

    /// A client authenticated as the Bot User with this bot key, registered
    /// (or updated) through `POST /api/auth/bot` with [`TEST_BOT_SECRET`].
    pub async fn bot(&self, bot_key: &str, nickname: &str) -> TestClient {
        let res = self
            .bearer(TEST_BOT_SECRET)
            .post(
                "/api/auth/bot",
                &json!({ "bot_key": bot_key, "nickname": nickname }),
            )
            .await;
        assert_eq!(
            res.status,
            StatusCode::OK,
            "bot auth as {bot_key} failed: {}",
            res.text()
        );
        self.bearer(res.json()["token"].as_str().expect("a token"))
    }

    /// A client signed in as the User with this Nickname, created on first
    /// use (through the dev login bypass, which must be enabled).
    pub async fn user(&self, nickname: &str) -> TestClient {
        let client = self.anonymous();
        let res = client
            .post("/api/dev/login", &json!({ "nickname": nickname }))
            .await;
        assert_eq!(
            res.status,
            StatusCode::OK,
            "dev login as {nickname} failed: {}",
            res.text()
        );
        client
    }
}

pub struct TestAppBuilder {
    config: Config,
    source: Option<SourceChoice>,
}

type MakeSource = Box<dyn FnOnce(SqlitePool, Arc<dyn Clock>) -> Arc<dyn MatchSource>>;

enum SourceChoice {
    Ready(Arc<dyn MatchSource>),
    Make(MakeSource),
}

impl TestAppBuilder {
    pub fn dev_login(mut self, enabled: bool) -> Self {
        self.config.dev_login = enabled;
        self
    }

    /// Serve the SPA from this directory.
    pub fn web_dir(mut self, dir: &Path) -> Self {
        self.config.web_dir = PathBuf::from(dir);
        self
    }

    /// Any other change to the test configuration.
    pub fn config(mut self, change: impl FnOnce(&mut Config)) -> Self {
        change(&mut self.config);
        self
    }

    /// Put this behind the `MatchSource` port instead of `TestApp::source`
    /// (e.g. a `FallbackSource` composed of several `ScriptedSource`s).
    pub fn source(mut self, source: Arc<dyn MatchSource>) -> Self {
        self.source = Some(SourceChoice::Ready(source));
        self
    }

    /// Like [`source`](Self::source), for a source that needs the test's
    /// database and clock (a `FallbackSource`).
    pub fn source_with(
        mut self,
        make: impl FnOnce(SqlitePool, Arc<dyn Clock>) -> Arc<dyn MatchSource> + 'static,
    ) -> Self {
        self.source = Some(SourceChoice::Make(Box::new(make)));
        self
    }

    pub async fn build(self) -> TestApp {
        let clock = Arc::new(FakeClock::new());
        let source = Arc::new(ScriptedSource::new());
        let notifier = Arc::new(RecordingNotifier::new());
        let db = db::connect_in_memory()
            .await
            .expect("in-memory database with migrations applied");
        let chosen: Arc<dyn MatchSource> = match self.source {
            None => source.clone(),
            Some(SourceChoice::Ready(s)) => s,
            Some(SourceChoice::Make(make)) => make(db.clone(), clock.clone()),
        };
        let state = AppState {
            db,
            clock: clock.clone(),
            source: chosen,
            notifier: notifier.clone(),
            config: Arc::new(self.config),
        };
        TestApp {
            clock,
            source,
            notifier,
            router: app::build_router(state.clone()),
            state,
        }
    }
}

/// Talks to the router like a browser would: it keeps the cookies it is
/// given, so a client stays signed in (or out) across requests.
pub struct TestClient {
    router: Router,
    cookies: Mutex<BTreeMap<String, String>>,
    /// Sent as `Authorization: Bearer ...` when set.
    bearer: Option<String>,
}

impl TestClient {
    pub async fn get(&self, path: &str) -> TestResponse {
        self.request(Method::GET, path, None::<&()>).await
    }

    pub async fn post(&self, path: &str, body: &impl Serialize) -> TestResponse {
        self.request(Method::POST, path, Some(body)).await
    }

    /// A POST without a body.
    pub async fn post_empty(&self, path: &str) -> TestResponse {
        self.request(Method::POST, path, None::<&()>).await
    }

    pub async fn put(&self, path: &str, body: &impl Serialize) -> TestResponse {
        self.request(Method::PUT, path, Some(body)).await
    }

    pub async fn patch(&self, path: &str, body: &impl Serialize) -> TestResponse {
        self.request(Method::PATCH, path, Some(body)).await
    }

    pub async fn delete(&self, path: &str) -> TestResponse {
        self.request(Method::DELETE, path, None::<&()>).await
    }

    /// The value of a cookie this client holds.
    pub fn cookie(&self, name: &str) -> Option<String> {
        self.cookies.lock().unwrap().get(name).cloned()
    }

    /// Hands this client a cookie, as if the browser had received it.
    pub fn set_cookie(&self, name: &str, value: &str) {
        self.cookies
            .lock()
            .unwrap()
            .insert(name.to_string(), value.to_string());
    }

    pub async fn request(
        &self,
        method: Method,
        path: &str,
        body: Option<&impl Serialize>,
    ) -> TestResponse {
        let mut request = Request::builder().method(method).uri(path);
        let cookie_header = self
            .cookies
            .lock()
            .unwrap()
            .iter()
            .map(|(name, value)| format!("{name}={value}"))
            .collect::<Vec<_>>()
            .join("; ");
        if !cookie_header.is_empty() {
            request = request.header(header::COOKIE, cookie_header);
        }
        if let Some(token) = &self.bearer {
            request = request.header(header::AUTHORIZATION, format!("Bearer {token}"));
        }
        let request = match body {
            Some(body) => request
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(body).unwrap())),
            None => request.body(Body::empty()),
        }
        .unwrap();

        let response = self.router.clone().oneshot(request).await.unwrap();
        let (parts, body) = response.into_parts();
        self.remember_cookies(&parts.headers);
        TestResponse {
            status: parts.status,
            headers: parts.headers,
            body: body.collect().await.unwrap().to_bytes().to_vec(),
        }
    }

    fn remember_cookies(&self, headers: &HeaderMap) {
        let mut cookies = self.cookies.lock().unwrap();
        for set_cookie in headers.get_all(header::SET_COOKIE) {
            let Ok(set_cookie) = set_cookie.to_str() else {
                continue;
            };
            let mut attributes = set_cookie.split(';').map(str::trim);
            let Some((name, value)) = attributes.next().and_then(|pair| pair.split_once('='))
            else {
                continue;
            };
            let removed = value.is_empty()
                || attributes.any(|a| a.eq_ignore_ascii_case("max-age=0"));
            if removed {
                cookies.remove(name);
            } else {
                cookies.insert(name.to_string(), value.to_string());
            }
        }
    }
}

pub struct TestResponse {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: Vec<u8>,
}

impl TestResponse {
    /// The body as JSON. Panics (showing the body) if it is not JSON.
    pub fn json(&self) -> Value {
        self.json_as()
    }

    pub fn json_as<T: DeserializeOwned>(&self) -> T {
        serde_json::from_slice(&self.body).unwrap_or_else(|e| {
            panic!(
                "response ({}) is not the expected JSON: {e}: {}",
                self.status,
                self.text()
            )
        })
    }

    pub fn text(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }

    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(name).and_then(|v| v.to_str().ok())
    }

    /// The `error.code` of an error response, e.g. `"match_locked"`.
    pub fn error_code(&self) -> String {
        self.json()["error"]["code"]
            .as_str()
            .unwrap_or_else(|| panic!("not an error response: {}", self.text()))
            .to_string()
    }
}
