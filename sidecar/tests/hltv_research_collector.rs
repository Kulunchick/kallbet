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
use flate2::read::GzDecoder;
use kallbet_ai::{
    Clock, Instant,
    adapters::hltv_research::{
        cli,
        collector::{
            CollectorConfig, CollectorError, HltvCollector, Pause, ReqwestHttp, UaMode,
            ios_user_agent,
        },
    },
};
use std::{
    io::Read,
    path::Path,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
use tokio::net::TcpListener;

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
            min_interval: Duration::from_secs(5),
            ua_mode: UaMode::FixedPerRun {
                major: 17,
                minor: 2,
                patch: 1,
            },
        },
        Arc::new(ReqwestHttp::new()),
        clock,
        pause.clone(),
    );

    let url = format!("{base}/matches/2394774/heroic-vs-sharks");
    let first = collector.collect_match(2394774, &url).await.unwrap();
    assert!(!first.from_cache);
    assert_eq!(first.record.match_id, 2394774);
    assert_eq!(first.record.veto.len(), 7);
    assert_eq!(first.record.maps.len(), 1);
    assert_eq!(first.record.overall_stats.len(), 2);
    assert_eq!(state.hits.load(Ordering::SeqCst), 1);
    assert_eq!(
        state.user_agents.lock().unwrap().as_slice(),
        &["hltv/2.13.2 (org.hltv; build:1; iOS 17.2.1) Alamofire/4.9.1"]
    );
    assert!(dir.path().join("hltv-match-2394774.html.gz").exists());
    assert!(dir.path().join("hltv-match-2394774.json").exists());
    assert!(dir.path().join("sources.tsv").exists());

    // Second call must hit local cache only: zero extra HTTP requests and zero pause.
    let second = collector.collect_match(2394774, &url).await.unwrap();
    assert!(second.from_cache);
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
            min_interval: Duration::from_secs(5),
            ua_mode: UaMode::FixedPerRun {
                major: 16,
                minor: 5,
                patch: 3,
            },
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
    assert_eq!(pause.waits(), vec![Duration::from_secs(5)]);
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
                min_interval: Duration::from_secs(5),
                ua_mode: UaMode::FixedPerRun {
                    major: 17,
                    minor: 2,
                    patch: 1,
                },
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
                .join(format!("hltv-match-{id}-blocked.html"))
                .exists(),
            "blocked response must be saved for audit"
        );

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
    assert!(dir.path().join("hltv-match-2394774.json").exists());

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
