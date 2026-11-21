//! The Liquipedia fetch layer: rate limits (never sleeping), the request
//! headers, failures, and the adapter inside the fallback chain. The wiki is
//! a stand-in that replays real saved API answers (`tests/fixtures/liquipedia/`).

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::{DateTime, Duration, TimeZone, Utc};
use kallbet::domain::{EventId, StageKind};
use kallbet::poller;
use kallbet::ports::{Clock, SourceError};
use kallbet::sources::fallback::{EventHint, ExternalSource, FallbackSettings, FallbackSource};
use kallbet::sources::liquipedia::limiter::{Class, RateLimiter};
use kallbet::sources::liquipedia::{HttpWiki, LiquipediaSource, Pause, Wiki, WikiFailure};
use kallbet::testing::{EventSnapshotBuilder, FakeClock, ScriptedSource, TestApp};
use serde_json::{Value, json};

const COLOGNE: &str = "Intel Extreme Masters/2026/Cologne";

fn fixture(name: &str) -> String {
    let path = format!("{}/tests/fixtures/liquipedia/{name}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

#[derive(Debug, Clone)]
struct Call {
    at: DateTime<Utc>,
    params: HashMap<String, String>,
}

impl Call {
    fn action(&self) -> &str {
        let p = &self.params;
        match p["action"].as_str() {
            "query" if p.contains_key("list") => p["list"].as_str(),
            other => other,
        }
    }
}

/// The wiki, answering with saved real responses. Records when each request
/// arrived (by the clock the adapter uses).
struct FakeWiki {
    clock: Arc<dyn Clock>,
    calls: Mutex<Vec<Call>>,
    fail: Mutex<Option<WikiFailure>>,
    missing_root: bool,
    /// Answers every plain page query with this instead of the Cologne pages.
    pages: Mutex<Option<String>>,
}

impl FakeWiki {
    fn new(clock: Arc<dyn Clock>) -> Arc<FakeWiki> {
        Arc::new(FakeWiki {
            clock,
            calls: Mutex::new(Vec::new()),
            fail: Mutex::new(None),
            missing_root: false,
            pages: Mutex::new(None),
        })
    }

    fn calls(&self) -> Vec<Call> {
        self.calls.lock().unwrap().clone()
    }

    fn actions(&self) -> Vec<String> {
        self.calls().iter().map(|c| c.action().to_string()).collect()
    }

    fn fail_with(&self, failure: Option<WikiFailure>) {
        *self.fail.lock().unwrap() = failure;
    }
}

#[async_trait]
impl Wiki for FakeWiki {
    async fn get(&self, params: &[(&str, &str)]) -> Result<String, WikiFailure> {
        let params: HashMap<String, String> = params
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        self.calls.lock().unwrap().push(Call {
            at: self.clock.now(),
            params: params.clone(),
        });
        if let Some(failure) = self.fail.lock().unwrap().clone() {
            return Err(failure);
        }
        let file = match params["action"].as_str() {
            "expandtemplates" => "api-teamnames-cologne.json",
            _ => match params.get("list").map(String::as_str) {
                Some("allpages") => "api-allpages-cologne.json",
                Some("search") => "api-search-iem-cologne.json",
                _ if params.contains_key("rvsection") => "api-section0-search-candidates.json",
                _ if self.missing_root => "api-missing-page.json",
                _ => "api-query-cologne-pages.json",
            },
        };
        if file == "api-query-cologne-pages.json"
            && let Some(pages) = self.pages.lock().unwrap().clone()
        {
            return Ok(pages);
        }
        Ok(fixture(file))
    }
}

/// "Waiting" moves the fake clock instead of sleeping.
struct FakePause {
    clock: Arc<FakeClock>,
    waited: Mutex<Vec<Duration>>,
}

#[async_trait]
impl Pause for FakePause {
    async fn pause(&self, for_: Duration) {
        self.waited.lock().unwrap().push(for_);
        self.clock.advance(for_);
    }
}

struct Rig {
    clock: Arc<FakeClock>,
    wiki: Arc<FakeWiki>,
    source: LiquipediaSource,
}

fn rig_with(wiki: impl FnOnce(Arc<dyn Clock>) -> Arc<FakeWiki>) -> Rig {
    let clock = Arc::new(FakeClock::at(Utc.with_ymd_and_hms(2026, 12, 1, 0, 0, 0).unwrap()));
    let wiki = wiki(clock.clone());
    let pause = Arc::new(FakePause {
        clock: clock.clone(),
        waited: Mutex::new(Vec::new()),
    });
    let source = LiquipediaSource::with(wiki.clone(), pause.clone(), clock.clone());
    Rig { clock, wiki, source }
}

fn rig() -> Rig {
    rig_with(FakeWiki::new)
}

// ------------------------------------------------------------- rate limits

#[test]
fn the_limiter_keeps_two_seconds_between_requests_and_thirty_between_heavy_ones() {
    let t0 = Utc.with_ymd_and_hms(2026, 12, 1, 0, 0, 0).unwrap();
    let max = Duration::minutes(10);
    let mut limiter = RateLimiter::new();

    // Published: 1 request per 2 seconds, whatever the kind.
    assert_eq!(limiter.reserve(t0, Class::Normal, max), Some(Duration::zero()));
    assert_eq!(limiter.reserve(t0, Class::Normal, max), Some(Duration::seconds(2)));
    assert_eq!(limiter.reserve(t0, Class::Normal, max), Some(Duration::seconds(4)));
    // Time passing counts: 10 s on, the next slot (6 s) has long passed.
    let t1 = t0 + Duration::seconds(10);
    assert_eq!(limiter.reserve(t1, Class::Normal, max), Some(Duration::zero()));

    // Published: action=parse 1 per 30 seconds (we hold expandtemplates to it).
    assert_eq!(limiter.reserve(t1, Class::Heavy, max), Some(Duration::seconds(2)));
    assert_eq!(limiter.reserve(t1, Class::Heavy, max), Some(Duration::seconds(32)));
    // A normal request goes between them, 2 s after the last booked one.
    assert_eq!(limiter.reserve(t1, Class::Normal, max), Some(Duration::seconds(34)));
}

#[test]
fn a_request_that_would_wait_too_long_is_refused_and_books_nothing() {
    let t0 = Utc.with_ymd_and_hms(2026, 12, 1, 0, 0, 0).unwrap();
    let mut limiter = RateLimiter::new();
    limiter.back_off(t0, Duration::minutes(10));

    assert_eq!(limiter.reserve(t0, Class::Normal, Duration::minutes(1)), None);
    // Nothing was booked: once the back-off is over, the next slot is free.
    let later = t0 + Duration::minutes(10);
    assert_eq!(limiter.reserve(later, Class::Normal, Duration::minutes(1)), Some(Duration::zero()));
}

#[tokio::test]
async fn fetching_an_event_spaces_its_requests_by_the_published_limits_without_sleeping() {
    let r = rig();

    let snapshot = r.source.fetch_event(COLOGNE).await.unwrap();

    assert_eq!(snapshot.matches.len(), 40);
    assert_eq!(r.wiki.actions(), vec!["allpages", "query", "expandtemplates"]);
    let calls = r.wiki.calls();
    for pair in calls.windows(2) {
        assert!(pair[1].at - pair[0].at >= Duration::seconds(2), "{pair:?}");
    }
    // Team names are remembered: the second fetch asks for none.
    let before = r.wiki.calls().len();
    r.source.fetch_event(COLOGNE).await.unwrap();
    let again: Vec<Call> = r.wiki.calls().split_off(before);
    assert_eq!(
        again.iter().map(|c| c.action().to_string()).collect::<Vec<_>>(),
        vec!["allpages", "query"]
    );
    assert!(again[0].at - calls.last().unwrap().at >= Duration::seconds(2));
}

/// An `action=query` answer carrying one page's saved wikitext.
fn query_answer(title: &str, wikitext_file: &str) -> String {
    json!({"batchcomplete": true, "query": {"pages": [{
        "ns": 0, "title": title,
        "revisions": [{"slots": {"main": {"content": fixture(wikitext_file)}}}]
    }]}})
    .to_string()
}

#[tokio::test]
async fn a_heavy_request_waits_thirty_seconds_after_the_previous_heavy_one() {
    let r = rig();
    r.source.fetch_event(COLOGNE).await.unwrap();
    // Another Event, whose teams are new: it needs its own name lookup.
    *r.wiki.pages.lock().unwrap() = Some(query_answer(
        "Dust2.dk/Ligaen/Season 33",
        "ligaen-season33.wikitext",
    ));

    r.source.fetch_event("Dust2.dk/Ligaen/Season 33").await.unwrap();

    let heavy: Vec<_> = r
        .wiki
        .calls()
        .into_iter()
        .filter(|c| c.action() == "expandtemplates")
        .collect();
    assert_eq!(heavy.len(), 2);
    assert!(heavy[1].at - heavy[0].at >= Duration::seconds(30), "{heavy:?}");
}

#[tokio::test]
async fn a_429_backs_off_and_the_next_request_fails_fast_until_it_has_passed() {
    let r = rig();
    r.wiki.fail_with(Some(WikiFailure::RateLimited(Some(Duration::minutes(5)))));

    let first = r.source.fetch_event(COLOGNE).await;
    assert!(matches!(first, Err(SourceError::Unavailable(_))));
    r.wiki.fail_with(None);
    let calls_after_429 = r.wiki.calls().len();

    // Within the back-off nothing reaches the wiki.
    let during = r.source.fetch_event(COLOGNE).await;
    assert!(matches!(during, Err(SourceError::Unavailable(_))));
    assert_eq!(r.wiki.calls().len(), calls_after_429);

    r.clock.advance(Duration::minutes(5));
    assert!(r.source.fetch_event(COLOGNE).await.is_ok());
}

#[tokio::test]
async fn a_403_is_unavailable_and_backs_off_for_a_long_time() {
    let r = rig();
    r.wiki.fail_with(Some(WikiFailure::Refused));

    assert!(matches!(r.source.fetch_event(COLOGNE).await, Err(SourceError::Unavailable(_))));
    r.wiki.fail_with(None);
    r.clock.advance(Duration::minutes(8));
    assert!(matches!(r.source.fetch_event(COLOGNE).await, Err(SourceError::Unavailable(_))));
    r.clock.advance(Duration::minutes(3));
    assert!(r.source.fetch_event(COLOGNE).await.is_ok());
}

// ------------------------------------------------------------ HTTP request

async fn serve_once(body: &str) -> (String, tokio::task::JoinHandle<String>) {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/counterstrike/api.php", listener.local_addr().unwrap());
    let body = body.to_string();
    let handle = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut received = Vec::new();
        let mut chunk = [0u8; 4096];
        while !String::from_utf8_lossy(&received).contains("\r\n\r\n") {
            let n = socket.read(&mut chunk).await.unwrap();
            if n == 0 {
                break;
            }
            received.extend_from_slice(&chunk[..n]);
        }
        let reply = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        socket.write_all(reply.as_bytes()).await.unwrap();
        String::from_utf8_lossy(&received).to_string()
    });
    (url, handle)
}

#[tokio::test]
async fn requests_carry_the_project_user_agent_and_accept_gzip() {
    let (url, request) = serve_once(r#"{"query":{"allpages":[]}}"#).await;
    let wiki = HttpWiki::with_api_url(url);

    let body = wiki
        .get(&[("action", "query"), ("list", "allpages"), ("apprefix", "A B/")])
        .await
        .unwrap();

    let raw = request.await.unwrap();
    let lower = raw.to_lowercase();
    assert_eq!(body, r#"{"query":{"allpages":[]}}"#);
    assert!(raw.starts_with("GET /counterstrike/api.php?format=json&formatversion=2&action=query&list=allpages&apprefix=A+B%2F"), "{raw}");
    assert!(
        lower.contains("user-agent: kallbet/0.1 (https://kallbet.kulunchick.tech)"),
        "{raw}"
    );
    assert!(lower.contains("accept-encoding: gzip"), "{raw}");
}

// ------------------------------------------------------------ the adapter

#[tokio::test]
async fn an_event_missing_on_the_wiki_is_not_found() {
    let r = rig_with(|clock| {
        Arc::new(FakeWiki {
            clock,
            calls: Mutex::new(Vec::new()),
            fail: Mutex::new(None),
            missing_root: true,
            pages: Mutex::new(None),
        })
    });

    let result = r.source.fetch_event("Nonexistent Event zzqx/2026").await;

    assert_eq!(result.unwrap_err(), SourceError::NotFound);
}

#[tokio::test]
async fn the_event_list_is_not_offered_and_that_is_not_a_failure() {
    let r = rig();

    assert_eq!(r.source.list_events().await.unwrap_err(), SourceError::NotConfigured);
    assert!(r.wiki.calls().is_empty());
}

#[tokio::test]
async fn candidates_for_an_event_are_the_search_hits_that_have_a_league_infobox() {
    let r = rig();
    let hint = EventHint {
        id: EventId(8301),
        name: "IEM Cologne Major 2026".into(),
        starts_at: None,
        ends_at: None,
    };

    let found = r.source.find_events(&hint).await.unwrap();

    assert_eq!(r.wiki.actions(), vec!["search", "query"]);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].external_id, COLOGNE);
    assert_eq!(found[0].name, "IEM Cologne Major 2026");
}

#[tokio::test]
async fn an_event_never_stored_has_no_name_to_search_for_and_costs_no_request() {
    let r = rig();
    let hint = EventHint { id: EventId(8301), name: String::new(), starts_at: None, ends_at: None };

    assert!(r.source.find_events(&hint).await.unwrap().is_empty());
    assert!(r.wiki.calls().is_empty());
}

// ------------------------------------------------- inside the fallback chain

fn matches_of(view: &Value) -> Vec<Value> {
    view["stages"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|s| s["groups"].as_array().unwrap())
        .flat_map(|g| g["matches"].as_array().unwrap())
        .cloned()
        .collect()
}

#[tokio::test]
async fn when_hltv_fails_liquipedia_updates_the_stored_match_by_the_hltv_id_it_carries() {
    let hltv = Arc::new(ScriptedSource::new());
    let wiki_cell: Arc<Mutex<Option<Arc<FakeWiki>>>> = Arc::new(Mutex::new(None));
    let cell = wiki_cell.clone();
    let h = hltv.clone();
    let app = TestApp::builder()
        .source_with(move |db, clock| {
            let wiki = FakeWiki::new(clock.clone());
            *cell.lock().unwrap() = Some(wiki.clone());
            // No real waiting; the chain's own clock is the fake one.
            struct NoPause;
            #[async_trait]
            impl Pause for NoPause {
                async fn pause(&self, _: Duration) {}
            }
            let liquipedia = LiquipediaSource::with(wiki, Arc::new(NoPause), clock.clone());
            Arc::new(
                FallbackSource::new(db, clock, FallbackSettings::default())
                    .with_primary(h)
                    .with_fallback(Arc::new(liquipedia)),
            )
        })
        .build()
        .await;
    let june = |d, h, m| Utc.with_ymd_and_hms(2026, 6, d, h, m, 0).unwrap();
    hltv.put_event(
        EventSnapshotBuilder::new(8301, "IEM Cologne Major 2026")
            .dates(june(2, 0, 0), june(21, 23, 0))
            .stage(StageKind::Swiss, "Stage 1")
            .swiss_match(2394777, (1, 0, 0), "M80", "Lynn Vision Gaming", june(2, 10, 30))
            .build(app.clock.now()),
    );
    let alice = app.user("alice").await;
    let res = alice
        .post("/api/leagues", &json!({ "event_id": 8301, "name": "Friends" }))
        .await;
    let league = res.json()["id"].as_i64().unwrap();
    poller::tick(&app.state).await;

    hltv.fail_with(SourceError::Unavailable("cloudflare".into()));
    app.clock.advance(Duration::minutes(5));
    poller::tick(&app.state).await;

    let view = alice.get(&format!("/api/leagues/{league}/matches")).await.json();
    let game = matches_of(&view)
        .into_iter()
        .find(|m| m["id"] == 2394777)
        .expect("the Match keeps its HLTV id");
    assert_eq!(game["source"], "liquipedia");
    assert_eq!(game["status"], "finished");
    assert_eq!(game["team_a"]["name"], "M80", "HLTV names are kept");
    assert_eq!(game["winner"], game["team_a"]["id"], "M80 won on maps 13-8");
    assert_eq!(game["score"], json!({ "team_a": 1, "team_b": 0 }));
    // Liquipedia's other Matches were left out as unmatched, not added.
    assert_eq!(matches_of(&view).len(), 1, "{view}");
}
