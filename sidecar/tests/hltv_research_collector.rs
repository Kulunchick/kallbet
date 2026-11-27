//! Collector tests against a local stand-in HTTP server (ticket #36).
//! No external network requests; no wall-clock sleeping.

use axum::{
    Router,
    extract::State,
    http::{HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    routing::get,
};
use chrono::{TimeZone, Utc};
use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use kallbet_ai::{
    Clock, Instant,
    adapters::{
        baseline::SystemClock,
        hltv_research::{
            archive::HltvArchiveManifest,
            cli,
            collector::{
                CollectorConfig, CollectorError, DEFAULT_CONCURRENCY, DEFAULT_DAILY_BUDGET,
                HltvCollector, Pause, ReqwestHttp, TokioPause, UaMode, discover_archive_targets,
                ios_user_agent,
            },
        },
    },
};
use std::{
    collections::HashMap,
    io::{Read, Write},
    path::Path,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
use tokio::{net::TcpListener, sync::Notify};

fn fixture(name: &str) -> String {
    let path = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/research")).join(name);
    let raw = std::fs::read(&path).unwrap();
    if name.ends_with(".gz") {
        let mut out = String::new();
        GzDecoder::new(&raw[..]).read_to_string(&mut out).unwrap();
        out
    } else {
        String::from_utf8(raw).unwrap()
    }
}

struct FakeClock(Mutex<Instant>);
impl FakeClock {
    fn new(at: Instant) -> Self {
        Self(Mutex::new(at))
    }
    fn advance(&self, d: chrono::Duration) {
        *self.0.lock().unwrap() += d;
    }
}
impl Clock for FakeClock {
    fn now(&self) -> Instant {
        *self.0.lock().unwrap()
    }
}

struct RecordingPause {
    clock: Arc<FakeClock>,
    waits: Mutex<Vec<Duration>>,
}
impl RecordingPause {
    fn new(clock: Arc<FakeClock>) -> Self {
        Self {
            clock,
            waits: Mutex::new(Vec::new()),
        }
    }
    fn waits(&self) -> Vec<Duration> {
        self.waits.lock().unwrap().clone()
    }
}
#[async_trait::async_trait]
impl Pause for RecordingPause {
    async fn sleep(&self, duration: Duration) {
        self.waits.lock().unwrap().push(duration);
        self.clock
            .advance(chrono::Duration::from_std(duration).unwrap());
    }
}

#[derive(Clone)]
struct MockState {
    hits: Arc<AtomicUsize>,
    user_agents: Arc<Mutex<Vec<String>>>,
    bo1_html: Arc<String>,
    bo3_html: Arc<String>,
    challenge_html: Arc<String>,
}

async fn start_server() -> (String, MockState) {
    let state = MockState {
        hits: Arc::new(AtomicUsize::new(0)),
        user_agents: Arc::new(Mutex::new(Vec::new())),
        bo1_html: Arc::new(fixture("hltv-match-2394774-heroic-vs-sharks.html.gz")),
        bo3_html: Arc::new(fixture("hltv-match-2394898-vitality-vs-fut.html.gz")),
        challenge_html: Arc::new(fixture("hltv-mapstats-230770-403-challenge.html")),
    };

    let app = Router::new()
        .route(
            "/matches/2394774/heroic-vs-sharks",
            get(|State(s): State<MockState>, headers: HeaderMap| async move {
                s.hits.fetch_add(1, Ordering::SeqCst);
                if let Some(ua) = headers.get("user-agent").and_then(|v| v.to_str().ok()) {
                    s.user_agents.lock().unwrap().push(ua.to_owned());
                }
                (StatusCode::OK, (*s.bo1_html).clone())
            }),
        )
        .route(
            "/matches/2394774/match",
            get(|State(s): State<MockState>| async move {
                s.hits.fetch_add(1, Ordering::SeqCst);
                let mut headers = HeaderMap::new();
                headers.insert(
                    "location",
                    HeaderValue::from_static("/matches/2394774/heroic-vs-sharks"),
                );
                (StatusCode::MOVED_PERMANENTLY, headers, "").into_response()
            }),
        )
        .route(
            "/matches/2394898/vitality-vs-fut",
            get(|State(s): State<MockState>, headers: HeaderMap| async move {
                s.hits.fetch_add(1, Ordering::SeqCst);
                if let Some(ua) = headers.get("user-agent").and_then(|v| v.to_str().ok()) {
                    s.user_agents.lock().unwrap().push(ua.to_owned());
                }
                (StatusCode::OK, (*s.bo3_html).clone())
            }),
        )
        .route(
            "/matches/2394898/match",
            get(|State(s): State<MockState>| async move {
                s.hits.fetch_add(1, Ordering::SeqCst);
                let mut headers = HeaderMap::new();
                headers.insert(
                    "location",
                    HeaderValue::from_static("/matches/2394898/vitality-vs-fut"),
                );
                (StatusCode::MOVED_PERMANENTLY, headers, "").into_response()
            }),
        )
        .route(
            "/matches/999004/redirect-to-stats",
            get(|State(s): State<MockState>| async move {
                s.hits.fetch_add(1, Ordering::SeqCst);
                let mut headers = HeaderMap::new();
                headers.insert(
                    "location",
                    HeaderValue::from_static("/stats/matches/mapstatsid/230770/x"),
                );
                (StatusCode::MOVED_PERMANENTLY, headers, "").into_response()
            }),
        )
        .route(
            "/matches/999005/match",
            get(|State(s): State<MockState>| async move {
                s.hits.fetch_add(1, Ordering::SeqCst);
                (StatusCode::OK, "<html><body><p>Match forfeited</p></body></html>")
            }),
        )
        .route(
            "/matches/999001/challenge-403",
            get(|State(s): State<MockState>| async move {
                s.hits.fetch_add(1, Ordering::SeqCst);
                let mut headers = HeaderMap::new();
                headers.insert("cf-mitigated", HeaderValue::from_static("challenge"));
                (StatusCode::FORBIDDEN, headers, (*s.challenge_html).clone()).into_response()
            }),
        )
        .route(
            "/matches/999002/rate-limited-429",
            get(|State(s): State<MockState>| async move {
                s.hits.fetch_add(1, Ordering::SeqCst);
                (StatusCode::TOO_MANY_REQUESTS, "Too Many Requests").into_response()
            }),
        )
        .route(
            "/matches/999003/challenge-200",
            get(|State(s): State<MockState>| async move {
                s.hits.fetch_add(1, Ordering::SeqCst);
                (StatusCode::OK, (*s.challenge_html).clone()).into_response()
            }),
        )
        .with_state(state.clone());

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (format!("http://{addr}"), state)
}

#[test]
fn ios_user_agent_matches_operator_pattern() {
    let ua = ios_user_agent(17, 2, 1).unwrap();
    assert_eq!(ua, "hltv/2.13.2 (org.hltv; build:1; iOS 17.2.1) Alamofire/4.9.1");
    assert!(ios_user_agent(13, 0, 0).is_err());
    assert!(ios_user_agent(18, 0, 0).is_err());
    assert!(ios_user_agent(15, 10, 0).is_err());
    assert!(ios_user_agent(15, 0, 10).is_err());
}

#[tokio::test]
async fn fetches_parses_and_caches_match_without_hitting_server_twice() {
    let (base, state) = start_server().await;
    let dir = tempfile::tempdir().unwrap();
    let clock = Arc::new(FakeClock::new(
        Utc.with_ymd_and_hms(2026, 11, 26, 12, 0, 0).unwrap(),
    ));
    let pause = Arc::new(RecordingPause::new(clock.clone()));

    let mut collector = HltvCollector::new(
        CollectorConfig {
            out_dir: dir.path().to_path_buf(),
            min_interval: Duration::from_secs(9),
            ua_mode: UaMode::FixedPerRun {
                major: 17,
                minor: 2,
                patch: 1,
            },
            daily_budget: DEFAULT_DAILY_BUDGET,
            concurrency: DEFAULT_CONCURRENCY,
        },
        Arc::new(ReqwestHttp::new()),
        clock,
        pause.clone(),
    );

    let url = format!("{base}/matches/2394774/heroic-vs-sharks");
    let first = collector.collect_match(2394774, &url).await.unwrap();
    assert!(!first.from_cache);
    assert_eq!(first.requests_used, 1);
    assert_eq!(first.record.match_id, 2394774);
    assert_eq!(first.record.veto.len(), 7);
    assert_eq!(first.record.maps.len(), 1);
    assert_eq!(first.record.overall_stats.len(), 2);
    assert_eq!(state.hits.load(Ordering::SeqCst), 1);
    assert_eq!(
        state.user_agents.lock().unwrap().as_slice(),
        &["hltv/2.13.2 (org.hltv; build:1; iOS 17.2.1) Alamofire/4.9.1"]
    );
    assert!(dir.path().join("matches/hltv-match-2394774.html.gz").exists());
    assert!(dir.path().join("matches/hltv-match-2394774.json").exists());
    assert!(dir.path().join("sources.tsv").exists());
    assert!(dir.path().join("manifest.json").exists());

    // Second call must hit local cache only: zero extra HTTP requests and zero pause.
    let second = collector.collect_match(2394774, &url).await.unwrap();
    assert!(second.from_cache);
    assert_eq!(second.requests_used, 0);
    assert_eq!(second.record, first.record);
    assert_eq!(state.hits.load(Ordering::SeqCst), 1);
    assert!(pause.waits().is_empty());
}

#[tokio::test]
async fn enforces_rate_limit_between_network_requests_without_sleeping() {
    let (base, state) = start_server().await;
    let dir = tempfile::tempdir().unwrap();
    let clock = Arc::new(FakeClock::new(
        Utc.with_ymd_and_hms(2026, 11, 26, 12, 0, 0).unwrap(),
    ));
    let pause = Arc::new(RecordingPause::new(clock.clone()));

    let mut collector = HltvCollector::new(
        CollectorConfig {
            out_dir: dir.path().to_path_buf(),
            min_interval: Duration::from_secs(9),
            ua_mode: UaMode::FixedPerRun {
                major: 16,
                minor: 5,
                patch: 3,
            },
            daily_budget: DEFAULT_DAILY_BUDGET,
            concurrency: DEFAULT_CONCURRENCY,
        },
        Arc::new(ReqwestHttp::new()),
        clock,
        pause.clone(),
    );

    collector
        .collect_match(2394774, &format!("{base}/matches/2394774/heroic-vs-sharks"))
        .await
        .unwrap();
    collector
        .collect_match(2394898, &format!("{base}/matches/2394898/vitality-vs-fut"))
        .await
        .unwrap();

    assert_eq!(state.hits.load(Ordering::SeqCst), 2);
    assert_eq!(pause.waits(), vec![Duration::from_secs(9)]);
    let uas = state.user_agents.lock().unwrap().clone();
    assert_eq!(uas.len(), 2);
    assert_eq!(uas[0], uas[1]);
    assert_eq!(
        uas[0],
        "hltv/2.13.2 (org.hltv; build:1; iOS 16.5.3) Alamofire/4.9.1"
    );
}

#[tokio::test]
async fn stops_immediately_on_403_429_or_cloudflare_challenge_and_refuses_further_requests() {
    for (id, path, expected_status) in [
        (999001, "/matches/999001/challenge-403", 403u16),
        (999002, "/matches/999002/rate-limited-429", 429u16),
        (999003, "/matches/999003/challenge-200", 200u16),
    ] {
        let (base, state) = start_server().await;
        let dir = tempfile::tempdir().unwrap();
        let clock = Arc::new(FakeClock::new(
            Utc.with_ymd_and_hms(2026, 11, 26, 12, 0, 0).unwrap(),
        ));
        let pause = Arc::new(RecordingPause::new(clock.clone()));
        let mut collector = HltvCollector::new(
            CollectorConfig {
                out_dir: dir.path().to_path_buf(),
                min_interval: Duration::from_secs(9),
                ua_mode: UaMode::FixedPerRun {
                    major: 17,
                    minor: 2,
                    patch: 1,
                },
                daily_budget: DEFAULT_DAILY_BUDGET,
                concurrency: DEFAULT_CONCURRENCY,
            },
            Arc::new(ReqwestHttp::new()),
            clock,
            pause,
        );

        let err = collector
            .collect_match(id, &format!("{base}{path}"))
            .await
            .expect_err("must trigger stop rule");
        match err {
            CollectorError::Stopped { status, .. } => assert_eq!(status, expected_status),
            other => panic!("expected Stopped, got {other:?}"),
        }
        assert_eq!(state.hits.load(Ordering::SeqCst), 1, "zero retries allowed");
        assert!(
            dir.path()
                .join(format!("blocked/hltv-match-{id}-blocked.html"))
                .exists(),
            "blocked response must be saved for audit"
        );
        assert!(collector.manifest().last_stop.is_some());

        // Any follow-up request on the halted collector is refused without hitting the server.
        let follow = collector
            .collect_match(2394774, &format!("{base}/matches/2394774/heroic-vs-sharks"))
            .await
            .expect_err("halted collector must refuse further requests");
        assert!(matches!(follow, CollectorError::AlreadyStopped), "{follow:?}");
        assert_eq!(state.hits.load(Ordering::SeqCst), 1);
    }
}

#[tokio::test]
async fn rejects_non_matches_paths_and_counts_redirect_hops_in_daily_budget() {
    let (base, state) = start_server().await;
    let dir = tempfile::tempdir().unwrap();
    let clock = Arc::new(FakeClock::new(
        Utc.with_ymd_and_hms(2026, 11, 26, 12, 0, 0).unwrap(),
    ));
    let pause = Arc::new(RecordingPause::new(clock.clone()));
    let mut collector = HltvCollector::new(
        CollectorConfig {
            out_dir: dir.path().to_path_buf(),
            min_interval: Duration::from_secs(9),
            ua_mode: UaMode::FixedPerRun {
                major: 17,
                minor: 2,
                patch: 1,
            },
            daily_budget: 10,
            concurrency: DEFAULT_CONCURRENCY,
        },
        Arc::new(ReqwestHttp::new()),
        clock,
        pause,
    );

    // /stats/* is explicitly disallowed with zero network requests.
    let err = collector
        .collect_match(230770, &format!("{base}/stats/matches/mapstatsid/230770/x"))
        .await
        .expect_err("/stats/* must be refused");
    assert!(matches!(err, CollectorError::DisallowedPath(_)), "{err:?}");
    assert_eq!(state.hits.load(Ordering::SeqCst), 0);

    // Redirect from /matches/* to /stats/* is also refused before following.
    let err = collector
        .collect_match(999004, &format!("{base}/matches/999004/redirect-to-stats"))
        .await
        .expect_err("redirect to /stats/* must be refused");
    assert!(matches!(err, CollectorError::DisallowedPath(_)), "{err:?}");
    assert_eq!(state.hits.load(Ordering::SeqCst), 1);
    assert_eq!(collector.spent_today(), 1);

    // Valid 301 redirect inside /matches/* charges both hops (2 requests).
    let out = collector
        .collect_match(2394774, &format!("{base}/matches/2394774/match"))
        .await
        .unwrap();
    assert_eq!(out.requests_used, 2);
    assert_eq!(out.final_url, format!("{base}/matches/2394774/heroic-vs-sharks"));
    assert_eq!(collector.spent_today(), 3);
}

#[tokio::test]
async fn batch_discovers_liquipedia_hltv_ids_enforces_daily_budget_and_resumes_cleanly() {
    let (base, state) = start_server().await;
    let archive = tempfile::tempdir().unwrap();
    let out_dir = archive.path().join("hltv");

    // Write a gzipped Liquipedia revision holding two HLTV match IDs + one walkover ID.
    let lp_json = r#"{"query":{"pages":[{"title":"Test/Event","revisions":[{"timestamp":"2026-06-22T00:00:00Z","slots":{"main":{"content":"{{Match|hltv=2394774}}{{Match|hltv=2394898}}{{Match|hltv=999005}}"}}}]}]}}"#;
    let mut enc = GzEncoder::new(Vec::new(), Compression::default());
    enc.write_all(lp_json.as_bytes()).unwrap();
    std::fs::write(archive.path().join("roster-1-2026-06-22.json.gz"), enc.finish().unwrap())
        .unwrap();

    let targets = discover_archive_targets(archive.path(), &out_dir, &base).unwrap();
    assert_eq!(targets.len(), 3);
    assert_eq!(targets[0].0, 999005);
    assert_eq!(targets[1].0, 2394774);
    assert_eq!(targets[2].0, 2394898);

    // Run 1: daily budget = 3 (covers walkover 1 req + redirected 2394774 2 reqs = 3, stops before 2394898)
    let clock = Arc::new(FakeClock::new(
        Utc.with_ymd_and_hms(2026, 11, 26, 12, 0, 0).unwrap(),
    ));
    let pause = Arc::new(RecordingPause::new(clock.clone()));
    let mut c1 = HltvCollector::new(
        CollectorConfig {
            out_dir: out_dir.clone(),
            min_interval: Duration::from_secs(9),
            ua_mode: UaMode::FixedPerRun {
                major: 17,
                minor: 2,
                patch: 1,
            },
            daily_budget: 3,
            concurrency: DEFAULT_CONCURRENCY,
        },
        Arc::new(ReqwestHttp::new()),
        clock.clone(),
        pause.clone(),
    );
    let r1 = c1.collect_batch(&targets, None).await.unwrap();
    assert_eq!(r1.fetched_ok, 1);
    assert_eq!(r1.parse_errors, 1);
    assert_eq!(r1.spent_today, 3);
    assert!(r1.stopped_by_budget);
    assert_eq!(state.hits.load(Ordering::SeqCst), 3);
    // Raw walkover HTML is cached so Run 2 will never fetch it again.
    assert!(out_dir.join("matches/hltv-match-999005.html.gz").exists());

    // Run 2 (resume same day with higher budget): skips cached 999005 & 2394774, fetches only 2394898!
    let mut c2 = HltvCollector::new(
        CollectorConfig {
            out_dir: out_dir.clone(),
            min_interval: Duration::from_secs(9),
            ua_mode: UaMode::FixedPerRun {
                major: 17,
                minor: 2,
                patch: 1,
            },
            daily_budget: 10,
            concurrency: DEFAULT_CONCURRENCY,
        },
        Arc::new(ReqwestHttp::new()),
        clock,
        pause,
    );
    let r2 = c2.collect_batch(&targets, None).await.unwrap();
    assert_eq!(r2.cached_hits, 1);
    assert_eq!(r2.parse_errors, 1);
    assert_eq!(r2.fetched_ok, 1);
    assert_eq!(r2.requests_made, 2); // only 2394898 (301 + 200)
    assert_eq!(r2.spent_today, 5);
    assert!(!r2.stopped_by_budget);
    assert_eq!(state.hits.load(Ordering::SeqCst), 5);

    let manifest: HltvArchiveManifest =
        serde_json::from_slice(&std::fs::read(out_dir.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(manifest.matches.len(), 2);
    assert_eq!(manifest.parse_errors.len(), 1);
    assert_eq!(manifest.daily_requests.get("2026-11-26"), Some(&5));
}

#[tokio::test]
async fn cli_command_collects_from_local_stand_in_and_guards_remote_hltv_without_feature() {
    let (base, state) = start_server().await;
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().to_string_lossy().into_owned();
    let url = format!("{base}/matches/2394774/heroic-vs-sharks");

    let summary = cli::run(&[
        "--match-id".into(),
        "2394774".into(),
        "--url".into(),
        url,
        "--out".into(),
        out.clone(),
        "--min-interval-secs".into(),
        "2".into(),
    ])
    .await
    .expect("local stand-in cli run");
    assert!(summary.contains("2394774"), "{summary}");
    assert_eq!(state.hits.load(Ordering::SeqCst), 1);
    assert!(dir.path().join("matches/hltv-match-2394774.json").exists());

    // Remote HLTV enforces >= 3 s interval per connection and accepts --concurrency.
    let too_fast = cli::parse(&[
        "--match-id".into(),
        "2394774".into(),
        "--url".into(),
        "https://www.hltv.org/matches/2394774/heroic-vs-sharks".into(),
        "--out".into(),
        out.clone(),
        "--min-interval-secs".into(),
        "2".into(),
    ])
    .expect_err("remote HLTV requires >= 3 s interval");
    assert!(too_fast.to_string().contains("at least 3"), "{too_fast}");
    assert!(
        cli::parse(&[
            "--archive".into(),
            out.clone(),
            "--min-interval-secs".into(),
            "3".into(),
            "--concurrency".into(),
            "4".into(),
        ])
        .is_ok()
    );

    let zero_conc = cli::parse(&[
        "--archive".into(),
        out.clone(),
        "--concurrency".into(),
        "0".into(),
    ])
    .expect_err("--concurrency 0 must be rejected");
    assert!(zero_conc.to_string().contains("concurrency"), "{zero_conc}");

    let parsed = cli::parse(&[
        "--archive".into(),
        out.clone(),
        "--concurrency".into(),
        "2".into(),
        "--min-interval-secs".into(),
        "4".into(),
        "--daily-budget".into(),
        "4000".into(),
        "--ios-version".into(),
        "17.2.1".into(),
    ])
    .expect("operator concurrent remote options must parse");
    assert_eq!(parsed.concurrency, 2);
    assert_eq!(parsed.min_interval_secs, 4);
    assert_eq!(parsed.daily_budget, 4000);

    // Default iOS version is deterministically seeded from the injected Clock.
    let fixed_clock = FakeClock::new(Utc.with_ymd_and_hms(2026, 11, 26, 12, 0, 0).unwrap());
    let seeded_a = cli::parse_with_clock(&["--archive".into(), out.clone()], &fixed_clock).unwrap();
    let seeded_b = cli::parse_with_clock(&["--archive".into(), out.clone()], &fixed_clock).unwrap();
    assert_eq!(
        (seeded_a.ios_major, seeded_a.ios_minor, seeded_a.ios_patch),
        (seeded_b.ios_major, seeded_b.ios_minor, seeded_b.ios_patch),
    );
    assert!(ios_user_agent(seeded_a.ios_major, seeded_a.ios_minor, seeded_a.ios_patch).is_ok());
    let shifted_clock = FakeClock::new(
        Utc.with_ymd_and_hms(2026, 11, 26, 12, 0, 0).unwrap() + chrono::Duration::nanoseconds(71),
    );
    let seeded_c = cli::parse_with_clock(&["--archive".into(), out.clone()], &shifted_clock).unwrap();
    assert_ne!(
        (seeded_a.ios_major, seeded_a.ios_minor, seeded_a.ios_patch),
        (seeded_c.ios_major, seeded_c.ios_minor, seeded_c.ios_patch),
    );

    // Without `--features hltv-impersonate`, pointing at remote hltv.org must be refused
    // rather than sending a mismatched default rustls fingerprint.
    #[cfg(not(feature = "hltv-impersonate"))]
    {
        let err = cli::run(&[
            "--match-id".into(),
            "2394774".into(),
            "--url".into(),
            "https://www.hltv.org/matches/2394774/heroic-vs-sharks".into(),
            "--out".into(),
            out,
        ])
        .await
        .expect_err("remote hltv.org requires hltv-impersonate feature");
        assert!(
            err.to_string().contains("hltv-impersonate"),
            "unexpected error: {err}"
        );
    }
}

#[derive(Clone)]
struct ConcurrentServerState {
    in_flight: Arc<AtomicUsize>,
    max_in_flight: Arc<AtomicUsize>,
    per_match_hits: Arc<Mutex<HashMap<i64, usize>>>,
    user_agents: Arc<Mutex<Vec<String>>>,
    bo1_html: Arc<String>,
    bo3_html: Arc<String>,
    slow_started: Arc<AtomicUsize>,
    slow_completed: Arc<AtomicUsize>,
    slow_started_notify: Arc<Notify>,
    hits_429: Arc<AtomicUsize>,
}

impl ConcurrentServerState {
    fn new() -> Self {
        Self {
            in_flight: Arc::new(AtomicUsize::new(0)),
            max_in_flight: Arc::new(AtomicUsize::new(0)),
            per_match_hits: Arc::new(Mutex::new(HashMap::new())),
            user_agents: Arc::new(Mutex::new(Vec::new())),
            bo1_html: Arc::new(fixture("hltv-match-2394774-heroic-vs-sharks.html.gz")),
            bo3_html: Arc::new(fixture("hltv-match-2394898-vitality-vs-fut.html.gz")),
            slow_started: Arc::new(AtomicUsize::new(0)),
            slow_completed: Arc::new(AtomicUsize::new(0)),
            slow_started_notify: Arc::new(Notify::new()),
            hits_429: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn record_hit(&self, match_id: i64, headers: &HeaderMap) {
        *self.per_match_hits.lock().unwrap().entry(match_id).or_insert(0) += 1;
        if let Some(ua) = headers.get("user-agent").and_then(|v| v.to_str().ok()) {
            self.user_agents.lock().unwrap().push(ua.to_owned());
        }
    }

    fn enter_in_flight(&self) {
        let cur = self.in_flight.fetch_add(1, Ordering::SeqCst) + 1;
        let mut prev = self.max_in_flight.load(Ordering::SeqCst);
        while cur > prev {
            match self
                .max_in_flight
                .compare_exchange(prev, cur, Ordering::SeqCst, Ordering::SeqCst)
            {
                Ok(_) => break,
                Err(actual) => prev = actual,
            }
        }
    }
}

async fn start_concurrent_server() -> (String, ConcurrentServerState) {
    let state = ConcurrentServerState::new();

    async fn serve_match(
        s: ConcurrentServerState,
        match_id: i64,
        use_bo3: bool,
        headers: HeaderMap,
    ) -> impl IntoResponse {
        s.record_hit(match_id, &headers);
        s.enter_in_flight();
        tokio::time::sleep(Duration::from_millis(80)).await;
        s.in_flight.fetch_sub(1, Ordering::SeqCst);
        let body = if use_bo3 {
            (*s.bo3_html).clone()
        } else {
            (*s.bo1_html).clone()
        };
        (StatusCode::OK, body)
    }

    let app = Router::new()
        .route(
            "/matches/2394774/heroic-vs-sharks",
            get(|State(s): State<ConcurrentServerState>, h: HeaderMap| {
                serve_match(s, 2394774, false, h)
            }),
        )
        .route(
            "/matches/2394898/vitality-vs-fut",
            get(|State(s): State<ConcurrentServerState>, h: HeaderMap| {
                serve_match(s, 2394898, true, h)
            }),
        )
        .route(
            "/matches/2394775/match-c",
            get(|State(s): State<ConcurrentServerState>, h: HeaderMap| {
                serve_match(s, 2394775, false, h)
            }),
        )
        .route(
            "/matches/2394776/match-d",
            get(|State(s): State<ConcurrentServerState>, h: HeaderMap| {
                serve_match(s, 2394776, true, h)
            }),
        )
        .route(
            "/matches/999010/slow-in-flight",
            get(|State(s): State<ConcurrentServerState>, h: HeaderMap| async move {
                s.record_hit(999010, &h);
                s.slow_started.fetch_add(1, Ordering::SeqCst);
                s.slow_started_notify.notify_one();
                tokio::time::sleep(Duration::from_secs(5)).await;
                s.slow_completed.fetch_add(1, Ordering::SeqCst);
                (StatusCode::OK, (*s.bo1_html).clone())
            }),
        )
        .route(
            "/matches/999002/rate-limited-429",
            get(|State(s): State<ConcurrentServerState>, h: HeaderMap| async move {
                // Wait until slow-in-flight is actively running on the other connection.
                let _ = tokio::time::timeout(
                    Duration::from_secs(2),
                    s.slow_started_notify.notified(),
                )
                .await;
                s.record_hit(999002, &h);
                s.hits_429.fetch_add(1, Ordering::SeqCst);
                (StatusCode::TOO_MANY_REQUESTS, "Too Many Requests - global stop")
            }),
        )
        .with_state(state.clone());

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (format!("http://{addr}"), state)
}

#[tokio::test]
async fn concurrent_batch_respects_concurrency_per_connection_interval_and_no_duplicate_fetch() {
    let (base, state) = start_concurrent_server().await;
    let dir = tempfile::tempdir().unwrap();

    // Include duplicate match IDs in the target list to prove deduplication under concurrency.
    let targets = vec![
        (2394774, format!("{base}/matches/2394774/heroic-vs-sharks")),
        (2394774, format!("{base}/matches/2394774/heroic-vs-sharks")),
        (2394898, format!("{base}/matches/2394898/vitality-vs-fut")),
        (2394775, format!("{base}/matches/2394775/match-c")),
        (2394898, format!("{base}/matches/2394898/vitality-vs-fut")),
        (2394776, format!("{base}/matches/2394776/match-d")),
    ];

    let mut collector = HltvCollector::new(
        CollectorConfig {
            out_dir: dir.path().to_path_buf(),
            min_interval: Duration::from_millis(250),
            ua_mode: UaMode::FixedPerRun {
                major: 17,
                minor: 2,
                patch: 1,
            },
            daily_budget: 20,
            concurrency: 2,
        },
        Arc::new(ReqwestHttp::new()),
        Arc::new(SystemClock),
        Arc::new(TokioPause),
    );

    let start = std::time::Instant::now();
    let report = collector.collect_batch(&targets, None).await.unwrap();
    let elapsed = start.elapsed();

    assert_eq!(report.fetched_ok, 4);
    assert_eq!(report.requests_made, 4);
    assert_eq!(report.parse_errors, 0);
    assert!(!report.stopped_by_budget);
    assert!(report.stopped_by_challenge.is_none());

    // Concurrency == 2: two connections overlap in flight, never exceeding 2.
    assert_eq!(state.max_in_flight.load(Ordering::SeqCst), 2);
    // Per-connection interval of 250ms applies between request 1 and 2 on each connection.
    assert!(
        elapsed >= Duration::from_millis(220),
        "expected per-connection interval >= 250ms, elapsed={elapsed:?}"
    );

    // Zero duplicate fetches despite duplicate targets.
    let hits = state.per_match_hits.lock().unwrap().clone();
    assert_eq!(hits.len(), 4);
    for id in [2394774, 2394898, 2394775, 2394776] {
        assert_eq!(hits.get(&id), Some(&1), "match {id} fetched more than once");
    }

    // One fixed iOS User-Agent across all connections.
    let uas = state.user_agents.lock().unwrap().clone();
    assert_eq!(uas.len(), 4);
    assert!(uas
        .iter()
        .all(|ua| ua == "hltv/2.13.2 (org.hltv; build:1; iOS 17.2.1) Alamofire/4.9.1"));

    // Resuming on a fresh collector hits cache only (0 network requests).
    let mut resumed = HltvCollector::new(
        CollectorConfig {
            out_dir: dir.path().to_path_buf(),
            min_interval: Duration::from_millis(250),
            ua_mode: UaMode::FixedPerRun {
                major: 17,
                minor: 2,
                patch: 1,
            },
            daily_budget: 20,
            concurrency: 2,
        },
        Arc::new(ReqwestHttp::new()),
        Arc::new(SystemClock),
        Arc::new(TokioPause),
    );
    let r2 = resumed.collect_batch(&targets, None).await.unwrap();
    assert_eq!(r2.cached_hits, 4);
    assert_eq!(r2.fetched_ok, 0);
    assert_eq!(r2.requests_made, 0);
    assert_eq!(state.user_agents.lock().unwrap().len(), 4);
}

#[tokio::test]
async fn global_stop_on_429_cancels_in_flight_and_queued_requests_without_retry() {
    let (base, state) = start_concurrent_server().await;
    let dir = tempfile::tempdir().unwrap();

    let targets = vec![
        (999010, format!("{base}/matches/999010/slow-in-flight")),
        (999002, format!("{base}/matches/999002/rate-limited-429")),
        (2394774, format!("{base}/matches/2394774/heroic-vs-sharks")),
        (2394898, format!("{base}/matches/2394898/vitality-vs-fut")),
        (2394775, format!("{base}/matches/2394775/match-c")),
        (2394776, format!("{base}/matches/2394776/match-d")),
    ];

    let mut collector = HltvCollector::new(
        CollectorConfig {
            out_dir: dir.path().to_path_buf(),
            min_interval: Duration::from_millis(20),
            ua_mode: UaMode::FixedPerRun {
                major: 17,
                minor: 2,
                patch: 1,
            },
            daily_budget: 20,
            concurrency: 2,
        },
        Arc::new(ReqwestHttp::new()),
        Arc::new(SystemClock),
        Arc::new(TokioPause),
    );

    let start = std::time::Instant::now();
    let report = collector.collect_batch(&targets, None).await.unwrap();
    let elapsed = start.elapsed();

    // Must cancel the 5-second in-flight sibling immediately instead of waiting for it.
    assert!(
        elapsed < Duration::from_secs(2),
        "global stop should cancel in-flight requests immediately, took {elapsed:?}"
    );
    assert!(collector.is_stopped());

    let stop = report
        .stopped_by_challenge
        .expect("429 on one connection must trigger global stop");
    assert_eq!(stop.status, 429);
    assert_eq!(stop.match_id, 999002);
    assert_eq!(stop.saved_file, "blocked/hltv-match-999002-blocked.html");
    let saved = std::fs::read_to_string(dir.path().join(&stop.saved_file)).unwrap();
    assert_eq!(saved, "Too Many Requests - global stop");

    // Manifest on disk records the exact stop event.
    let manifest: HltvArchiveManifest =
        serde_json::from_slice(&std::fs::read(dir.path().join("manifest.json")).unwrap()).unwrap();
    assert_eq!(manifest.last_stop, Some(stop));

    // Zero retries on the 429, in-flight request never completed, and zero queued targets hit.
    assert_eq!(state.hits_429.load(Ordering::SeqCst), 1);
    assert_eq!(state.slow_started.load(Ordering::SeqCst), 1);
    assert_eq!(state.slow_completed.load(Ordering::SeqCst), 0);
    let hits = state.per_match_hits.lock().unwrap().clone();
    for queued_id in [2394774, 2394898, 2394775, 2394776] {
        assert_eq!(
            hits.get(&queued_id),
            None,
            "queued match {queued_id} must never be fetched after global stop"
        );
    }

    // Any subsequent request on the stopped collector is refused with zero hits.
    let err = collector
        .collect_match(2394774, &format!("{base}/matches/2394774/heroic-vs-sharks"))
        .await
        .expect_err("halted collector must refuse further requests");
    assert!(matches!(err, CollectorError::AlreadyStopped), "{err:?}");
    assert_eq!(state.per_match_hits.lock().unwrap().get(&2394774), None);
}
