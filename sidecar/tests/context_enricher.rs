use async_trait::async_trait;
use axum::{
    Router,
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    routing::get,
};
use chrono::Duration;
use kallbet_ai::adapters::context::{
    CompositeContextEnricher, ContextConfig, Pause, RssFeedConfig,
};
use kallbet_ai::{Audit, Clock, ContextEnricher, ContextFact, Format, Instant, Match, Team};
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex};

const SINGAPORE_REV: &str =
    include_str!("fixtures/context/liquipedia/pgl-singapore-2026-rev-before-start.json");
const HLTV_RSS: &str = include_str!("fixtures/context/rss/hltv-news.xml");
const DUST2US_RSS: &str = include_str!("fixtures/context/rss/dust2us-news.xml");
const VRS_NOV: &str = include_str!("fixtures/vrs/standings_global_2026_11_02.md");

struct TestClock(Mutex<Instant>);
impl TestClock {
    fn new(at: Instant) -> Arc<Self> {
        Arc::new(Self(Mutex::new(at)))
    }
    fn advance(&self, delta: Duration) {
        *self.0.lock().unwrap() += delta;
    }
}
impl Clock for TestClock {
    fn now(&self) -> Instant {
        *self.0.lock().unwrap()
    }
}

struct AdvancingPause {
    clock: Arc<TestClock>,
    waits: Mutex<Vec<Duration>>,
}
impl AdvancingPause {
    fn new(clock: Arc<TestClock>) -> Arc<Self> {
        Arc::new(Self {
            clock,
            waits: Mutex::new(Vec::new()),
        })
    }
    fn waits(&self) -> Vec<Duration> {
        self.waits.lock().unwrap().clone()
    }
}
#[async_trait]
impl Pause for AdvancingPause {
    async fn pause(&self, duration: Duration) {
        self.waits.lock().unwrap().push(duration);
        self.clock.advance(duration);
    }
}

#[derive(Clone)]
struct RecordedWikiRequest {
    params: HashMap<String, String>,
    user_agent: Option<String>,
    accept_encoding: Option<String>,
}

#[derive(Default)]
struct ServerState {
    wiki_requests: Mutex<Vec<RecordedWikiRequest>>,
    wiki_status: Mutex<Option<(StatusCode, Option<String>)>>,
    wiki_body: Mutex<String>,
    rss_hits: Mutex<HashMap<String, usize>>,
    rss_bodies: Mutex<HashMap<String, (StatusCode, String)>>,
}

async fn start_stub_server(state: Arc<ServerState>) -> (String, tokio::task::JoinHandle<()>) {
    let app = Router::new()
        .route(
            "/api.php",
            get(
                |State(st): State<Arc<ServerState>>,
                 headers: HeaderMap,
                 Query(params): Query<HashMap<String, String>>| async move {
                    st.wiki_requests.lock().unwrap().push(RecordedWikiRequest {
                        params,
                        user_agent: headers
                            .get("user-agent")
                            .and_then(|v| v.to_str().ok())
                            .map(str::to_string),
                        accept_encoding: headers
                            .get("accept-encoding")
                            .and_then(|v| v.to_str().ok())
                            .map(str::to_string),
                    });
                    if let Some((code, retry)) = st.wiki_status.lock().unwrap().clone() {
                        let mut resp_headers = HeaderMap::new();
                        if let Some(r) = retry {
                            resp_headers.insert("retry-after", r.parse().unwrap());
                        }
                        return (code, resp_headers, "error".to_string());
                    }
                    let body = st.wiki_body.lock().unwrap().clone();
                    (StatusCode::OK, HeaderMap::new(), body)
                },
            ),
        )
        .route(
            "/rss/{name}",
            get(
                |State(st): State<Arc<ServerState>>,
                 axum::extract::Path(name): axum::extract::Path<String>| async move {
                    *st.rss_hits.lock().unwrap().entry(name.clone()).or_default() += 1;
                    let (code, body) = st
                        .rss_bodies
                        .lock()
                        .unwrap()
                        .get(&name)
                        .cloned()
                        .unwrap_or((StatusCode::NOT_FOUND, String::new()));
                    (code, body)
                },
            ),
        )
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
        .await
        .unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let handle = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (base, handle)
}

fn sample_match(event_id: i64, id: i64, team_a: &str, team_b: &str) -> Match {
    Match {
        event_id,
        id,
        team_a: Team {
            id: 1,
            name: team_a.into(),
        },
        team_b: Team {
            id: 2,
            name: team_b.into(),
        },
        format: Format::Bo3,
        scheduled_start: Some("2026-11-24T04:00:00Z".parse().unwrap()),
    }
}

#[tokio::test]
async fn past_cutoff_queries_historical_revision_with_compliant_headers_and_caches_in_sqlite() {
    let state = Arc::new(ServerState::default());
    *state.wiki_body.lock().unwrap() = SINGAPORE_REV.to_string();
    state
        .rss_bodies
        .lock()
        .unwrap()
        .insert("hltv".into(), (StatusCode::OK, HLTV_RSS.to_string()));
    state
        .rss_bodies
        .lock()
        .unwrap()
        .insert("dust2us".into(), (StatusCode::OK, DUST2US_RSS.to_string()));

    let (base, server) = start_stub_server(state.clone()).await;
    let dir = tempfile::tempdir().unwrap();
    let audit = Audit::open(&dir.path().join("audit.db")).await.unwrap();
    let vrs_ts: Instant = "2026-11-02T23:59:59.999999999Z".parse().unwrap();
    audit.record_vrs(VRS_NOV, vrs_ts).await.unwrap();

    // Live wall clock is Nov 25, while cutoff is in the past (Nov 24 00:00Z).
    let now: Instant = "2026-11-25T16:00:00Z".parse().unwrap();
    let past_cutoff: Instant = "2026-11-24T00:00:00Z".parse().unwrap();
    let clock = TestClock::new(now);
    let pause = AdvancingPause::new(clock.clone());

    let mut pages = BTreeMap::new();
    pages.insert(99, "PGL/2026/Singapore".to_string());
    let config = ContextConfig {
        liquipedia_pages: pages,
        liquipedia_api_url: format!("{base}/api.php"),
        rss_feeds: vec![
            RssFeedConfig::new(format!("{base}/rss/hltv")),
            RssFeedConfig::new(format!("{base}/rss/dust2us")),
        ],
        ..ContextConfig::default()
    };

    let enricher = CompositeContextEnricher::from_audit(&config, &audit, clock.clone(), pause)
        .await
        .unwrap();

    let match1 = sample_match(99, 101, "Team Vitality", "MOUZ");
    let match2 = sample_match(99, 102, "FaZe Clan", "Team Liquid");

    let facts1 = enricher.facts(&match1, past_cutoff).await.unwrap();
    let facts2 = enricher.facts(&match2, past_cutoff).await.unwrap();

    // Liquipedia was queried ONCE across both Matches (SQLite cache hit for match2).
    let wiki_reqs = state.wiki_requests.lock().unwrap().clone();
    assert_eq!(wiki_reqs.len(), 1);
    let req = &wiki_reqs[0];
    assert_eq!(req.params.get("action").map(String::as_str), Some("query"));
    assert_eq!(
        req.params.get("prop").map(String::as_str),
        Some("revisions")
    );
    assert_eq!(req.params.get("rvdir").map(String::as_str), Some("older"));
    assert_eq!(
        req.params.get("rvstart").map(String::as_str),
        Some("2026-11-23T23:59:59Z")
    );
    assert_eq!(
        req.params.get("titles").map(String::as_str),
        Some("PGL/2026/Singapore")
    );
    assert_eq!(
        req.params.get("formatversion").map(String::as_str),
        Some("2")
    );
    assert_eq!(
        req.user_agent.as_deref(),
        Some("KallBet/0.1 (https://kallbet.kulunchick.tech)")
    );
    assert!(
        req.accept_encoding
            .as_deref()
            .is_some_and(|enc| enc.to_ascii_lowercase().contains("gzip")),
        "expected gzip Accept-Encoding, got {:?}",
        req.accept_encoding
    );

    // RSS feeds were also fetched ONCE each across both Matches.
    assert_eq!(state.rss_hits.lock().unwrap().get("hltv"), Some(&1));
    assert_eq!(state.rss_hits.lock().unwrap().get("dust2us"), Some(&1));

    // Match 1 (Vitality vs MOUZ): 2 Liquipedia rosters + 2 VRS diffs, Vitality stand-in & diff flagged.
    let parsed1: Vec<ContextFact> = facts1
        .iter()
        .map(|f| serde_json::from_value(f.at(past_cutoff).unwrap().clone()).unwrap())
        .collect();
    assert!(parsed1.iter().any(|f| matches!(
        f,
        ContextFact::LiquipediaRoster(r) if r.team == "Team Vitality" && r.has_stand_in
    )));
    assert!(parsed1.iter().any(|f| matches!(
        f,
        ContextFact::VrsRosterDiff(d)
            if d.team == "Team Vitality"
                && d.changed
                && d.added_players == ["apEX"]
                && d.removed_players == ["jL"]
    )));

    // Match 2 (FaZe vs Liquid at cutoff 2026-11-24T00:00Z):
    // dust2us has "FaZe, Liquid to each take home over $90,000..." at 2026-11-23T23:49:00Z (< cutoff),
    // while Nov 24/25 articles MUST be excluded.
    let parsed2: Vec<ContextFact> = facts2
        .iter()
        .map(|f| serde_json::from_value(f.at(past_cutoff).unwrap().clone()).unwrap())
        .collect();
    let news2: Vec<_> = parsed2
        .iter()
        .filter_map(|f| match f {
            ContextFact::NewsItem(n) => Some(n),
            _ => None,
        })
        .collect();
    assert!(
        news2
            .iter()
            .any(|n| n.url.contains("faze-liquid-to-each-take-home"))
    );
    assert!(news2.iter().all(|n| n.published_at < past_cutoff));

    // A fresh enricher instance opening the same SQLite audit DB reuses the SQLite cache.
    let enricher2 = CompositeContextEnricher::from_audit(
        &config,
        &audit,
        clock.clone(),
        AdvancingPause::new(clock),
    )
    .await
    .unwrap();
    let _ = enricher2.facts(&match1, past_cutoff).await.unwrap();
    assert_eq!(state.wiki_requests.lock().unwrap().len(), 1);
    assert_eq!(state.rss_hits.lock().unwrap().get("hltv"), Some(&1));

    server.abort();
}

#[tokio::test]
async fn rate_limiter_spaces_liquipedia_requests_and_backs_off_on_429() {
    let state = Arc::new(ServerState::default());
    *state.wiki_body.lock().unwrap() = SINGAPORE_REV.to_string();
    let (base, server) = start_stub_server(state.clone()).await;

    let dir = tempfile::tempdir().unwrap();
    let audit = Audit::open(&dir.path().join("audit.db")).await.unwrap();
    let now: Instant = "2026-11-24T00:00:00Z".parse().unwrap();
    let clock = TestClock::new(now);
    let pause = AdvancingPause::new(clock.clone());

    let mut pages = BTreeMap::new();
    pages.insert(1, "PGL/2026/Singapore".to_string());
    pages.insert(2, "PGL/2026/Singapore/Other".to_string());
    let config = ContextConfig {
        liquipedia_pages: pages,
        liquipedia_api_url: format!("{base}/api.php"),
        ..ContextConfig::default()
    };

    let enricher =
        CompositeContextEnricher::from_audit(&config, &audit, clock.clone(), pause.clone())
            .await
            .unwrap();

    // Two distinct Liquipedia pages queried back-to-back must wait >= 2s before the second call.
    let _ = enricher
        .facts(&sample_match(1, 1, "MOUZ", "Vitality"), now)
        .await
        .unwrap();
    let _ = enricher
        .facts(&sample_match(2, 2, "MOUZ", "Vitality"), now)
        .await
        .unwrap();
    assert_eq!(state.wiki_requests.lock().unwrap().len(), 2);
    assert_eq!(pause.waits(), vec![Duration::seconds(2)]);

    // Simulate HTTP 429 with Retry-After: 120; subsequent request during backoff fails fast
    // (> 60s max wait) and degrades gracefully without hanging or sending a burst.
    *state.wiki_status.lock().unwrap() = Some((StatusCode::TOO_MANY_REQUESTS, Some("120".into())));
    let mut pages3 = config.liquipedia_pages.clone();
    pages3.insert(3, "PGL/2026/Singapore/Third".to_string());
    pages3.insert(4, "PGL/2026/Singapore/Fourth".to_string());
    let config3 = ContextConfig {
        liquipedia_pages: pages3,
        ..config
    };
    let enricher_429 =
        CompositeContextEnricher::from_audit(&config3, &audit, clock.clone(), pause.clone())
            .await
            .unwrap();
    // Event 3 triggers the 429 -> returns Ok(empty) gracefully.
    let res3 = enricher_429
        .facts(&sample_match(3, 3, "MOUZ", "Vitality"), clock.now())
        .await
        .unwrap();
    assert!(res3.is_empty());
    let reqs_after_429 = state.wiki_requests.lock().unwrap().len();
    // Event 4 immediately after is skipped by the backoff (0 extra HTTP calls) and returns Ok(empty).
    let res4 = enricher_429
        .facts(&sample_match(4, 4, "MOUZ", "Vitality"), clock.now())
        .await
        .unwrap();
    assert!(res4.is_empty());
    assert_eq!(state.wiki_requests.lock().unwrap().len(), reqs_after_429);

    server.abort();
}

#[tokio::test]
async fn enricher_degrades_gracefully_when_liquipedia_or_one_rss_feed_fails() {
    let state = Arc::new(ServerState::default());
    *state.wiki_status.lock().unwrap() = Some((StatusCode::INTERNAL_SERVER_ERROR, None));
    state.rss_bodies.lock().unwrap().insert(
        "broken".into(),
        (
            StatusCode::FORBIDDEN,
            "<html>Just a moment...</html>".into(),
        ),
    );
    state
        .rss_bodies
        .lock()
        .unwrap()
        .insert("dust2us".into(), (StatusCode::OK, DUST2US_RSS.to_string()));
    let (base, server) = start_stub_server(state.clone()).await;

    let dir = tempfile::tempdir().unwrap();
    let audit = Audit::open(&dir.path().join("audit.db")).await.unwrap();
    let cutoff: Instant = "2026-11-25T12:00:00Z".parse().unwrap();
    let clock = TestClock::new(cutoff);
    let pause = AdvancingPause::new(clock.clone());

    let mut pages = BTreeMap::new();
    pages.insert(99, "PGL/2026/Singapore".to_string());
    let config = ContextConfig {
        liquipedia_pages: pages,
        liquipedia_api_url: format!("{base}/api.php"),
        rss_feeds: vec![
            RssFeedConfig::new(format!("{base}/rss/broken")),
            RssFeedConfig::new(format!("{base}/rss/dust2us")),
        ],
        ..ContextConfig::default()
    };

    let enricher = CompositeContextEnricher::from_audit(&config, &audit, clock, pause)
        .await
        .unwrap();
    let facts = enricher
        .facts(&sample_match(99, 101, "FaZe Clan", "Team Liquid"), cutoff)
        .await
        .expect("upstream failures must degrade gracefully");

    // Liquipedia 500 + broken RSS 403 were skipped, while Dust2.us still returned matching news!
    assert!(!facts.is_empty());
    assert!(facts.iter().all(|f| f.observed_at() < cutoff));

    server.abort();
}

#[tokio::test]
async fn earlier_historical_cutoff_never_reuses_newer_cached_revision() {
    let state = Arc::new(ServerState::default());
    *state.wiki_body.lock().unwrap() = SINGAPORE_REV.to_string();
    let (base, server) = start_stub_server(state.clone()).await;

    let dir = tempfile::tempdir().unwrap();
    let audit = Audit::open(&dir.path().join("audit.db")).await.unwrap();
    let now: Instant = "2026-11-25T12:00:00Z".parse().unwrap();
    let clock = TestClock::new(now);
    let pause = AdvancingPause::new(clock.clone());

    let mut pages = BTreeMap::new();
    pages.insert(99, "PGL/2026/Singapore".to_string());
    let config = ContextConfig {
        liquipedia_pages: pages,
        liquipedia_api_url: format!("{base}/api.php"),
        ..ContextConfig::default()
    };

    let enricher = CompositeContextEnricher::from_audit(&config, &audit, clock, pause)
        .await
        .unwrap();
    let game = sample_match(99, 101, "Team Vitality", "MOUZ");

    // First query at Nov 24 caches the Nov 23 19:49:30Z revision.
    let later_cutoff: Instant = "2026-11-24T00:00:00Z".parse().unwrap();
    let later_facts = enricher.facts(&game, later_cutoff).await.unwrap();
    assert!(!later_facts.is_empty());
    assert_eq!(state.wiki_requests.lock().unwrap().len(), 1);

    // Second query at an earlier cutoff (Nov 20) MUST NOT serve the Nov 23 cached revision.
    // It queries Liquipedia with rvstart=2026-11-19T23:59:59Z; since our stub still returns
    // the Nov 23 revision, strict AsOf filtering excludes it and returns empty.
    let earlier_cutoff: Instant = "2026-11-20T00:00:00Z".parse().unwrap();
    let earlier_facts = enricher.facts(&game, earlier_cutoff).await.unwrap();
    assert!(earlier_facts.is_empty());
    let reqs = state.wiki_requests.lock().unwrap().clone();
    assert_eq!(reqs.len(), 2);
    assert_eq!(
        reqs[1].params.get("rvstart").map(String::as_str),
        Some("2026-11-19T23:59:59Z")
    );
    assert_eq!(reqs[1].params.get("rvdir").map(String::as_str), Some("older"));

    server.abort();
}

#[tokio::test]
async fn an_unchanged_page_or_feed_fetched_again_is_kept_once() {
    let state = Arc::new(ServerState::default());
    *state.wiki_body.lock().unwrap() = SINGAPORE_REV.to_string();
    state
        .rss_bodies
        .lock()
        .unwrap()
        .insert("hltv".into(), (StatusCode::OK, HLTV_RSS.to_string()));
    let (base, server) = start_stub_server(state.clone()).await;
    let dir = tempfile::tempdir().unwrap();
    let audit = Audit::open(&dir.path().join("audit.db")).await.unwrap();
    let now: Instant = "2026-11-25T16:00:00Z".parse().unwrap();
    let clock = TestClock::new(now);
    let pause = AdvancingPause::new(clock.clone());
    let config = ContextConfig {
        liquipedia_pages: [(99, "PGL/2026/Singapore".to_string())].into(),
        liquipedia_api_url: format!("{base}/api.php"),
        rss_feeds: vec![RssFeedConfig::new(format!("{base}/rss/hltv"))],
        ..ContextConfig::default()
    };
    let enricher = CompositeContextEnricher::from_audit(&config, &audit, clock.clone(), pause)
        .await
        .unwrap();
    let game = sample_match(99, 101, "Team Vitality", "MOUZ");
    for _ in 0..2 {
        enricher.facts(&game, clock.now()).await.unwrap();
        // Past both cache TTLs: the next call fetches again.
        clock.advance(Duration::minutes(20));
    }
    assert_eq!(state.wiki_requests.lock().unwrap().len(), 2);
    assert_eq!(state.rss_hits.lock().unwrap()["hltv"], 2);
    let revisions = audit
        .raw_records("liquipedia:revision:PGL/2026/Singapore")
        .await
        .unwrap();
    assert_eq!(revisions.len(), 1);
    assert_eq!(revisions[0].1, SINGAPORE_REV.as_bytes());
    let feeds = audit
        .raw_records(&format!("rss:xml:{base}/rss/hltv"))
        .await
        .unwrap();
    assert_eq!(feeds.len(), 1);
    let reopened = Audit::open(&dir.path().join("audit.db")).await.unwrap();
    assert!(reopened.news_before(now).await.unwrap().is_empty());
    let archived = reopened.news_before(clock.now()).await.unwrap();
    let expected = kallbet_ai::adapters::context::rss::parse_rss_items(
        HLTV_RSS,
        &format!("{base}/rss/hltv"),
        clock.now(),
    )
    .unwrap();
    assert!(!expected.is_empty());
    assert_eq!(archived.len(), expected.len());
    assert!(
        archived
            .iter()
            .all(|item| item.observed_at() < now + Duration::minutes(20))
    );
    server.abort();
}
