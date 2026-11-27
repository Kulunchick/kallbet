//! Ranking collector tests against an in-memory transport (no network, no sleeping).

use chrono::{NaiveDate, TimeZone, Utc};
use kallbet_ai::{
    Clock, Instant,
    adapters::hltv_research::{
        cli,
        collector::{CollectorError, HttpResponse, Pause, ResearchHttp},
        ranking::{RankingConfig, collect_rankings, mondays, ranking_url, validate_ranking_url},
    },
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};

fn day(s: &str) -> NaiveDate {
    s.parse().unwrap()
}

struct FakeClock(Mutex<Instant>);
impl Clock for FakeClock {
    fn now(&self) -> Instant {
        *self.0.lock().unwrap()
    }
}
struct RecordingPause {
    clock: Arc<FakeClock>,
    waits: Mutex<Vec<Duration>>,
}
#[async_trait::async_trait]
impl Pause for RecordingPause {
    async fn sleep(&self, d: Duration) {
        self.waits.lock().unwrap().push(d);
        *self.clock.0.lock().unwrap() += chrono::Duration::from_std(d).unwrap();
    }
}

struct Scripted {
    responses: Mutex<HashMap<String, (u16, Option<String>)>>,
    urls: Mutex<Vec<String>>,
}
#[async_trait::async_trait]
impl ResearchHttp for Scripted {
    async fn get(&self, url: &str, _ua: &str) -> Result<HttpResponse, CollectorError> {
        self.urls.lock().unwrap().push(url.to_owned());
        let (status, location) = self.responses.lock().unwrap().get(url).cloned().unwrap_or((404, None));
        Ok(HttpResponse { status, cf_mitigated: None, location, body: format!("<html>ranking {url}</html>") })
    }
}

fn setup(responses: &[(&str, u16, Option<&str>)]) -> (Arc<Scripted>, Arc<FakeClock>, Arc<RecordingPause>) {
    let http = Arc::new(Scripted {
        responses: Mutex::new(
            responses.iter().map(|(u, s, l)| (u.to_string(), (*s, l.map(str::to_owned)))).collect(),
        ),
        urls: Mutex::new(Vec::new()),
    });
    let clock = Arc::new(FakeClock(Mutex::new(Utc.with_ymd_and_hms(2026, 11, 27, 12, 0, 0).unwrap())));
    let pause = Arc::new(RecordingPause { clock: clock.clone(), waits: Mutex::new(Vec::new()) });
    (http, clock, pause)
}

fn config(dir: &std::path::Path) -> RankingConfig {
    RankingConfig { out_dir: dir.join("ranking"), min_interval: Duration::from_secs(6), ios: (17, 2, 1), max_requests: None }
}

#[test]
fn urls_and_mondays_follow_hltv_publication_dates() {
    assert_eq!(ranking_url("https://www.hltv.org/", day("2023-09-25")), "https://www.hltv.org/ranking/teams/2023/september/25");
    let all = mondays(day("2023-09-25"), day("2026-11-23"));
    assert_eq!((all.first().copied(), all.last().copied()), (Some(day("2023-09-25")), Some(day("2026-11-23"))));
    assert_eq!(all.len(), 166);
    assert_eq!(mondays(day("2023-09-26"), day("2023-10-08")), vec![day("2023-10-02")]);
    assert!(validate_ranking_url("https://www.hltv.org/matches/1/x").is_err());
    assert!(validate_ranking_url("https://www.hltv.org/ranking/teams/2024/may/6").is_ok());
}

#[tokio::test]
async fn caches_spaces_requests_and_records_provenance() {
    let dir = tempfile::tempdir().unwrap();
    let a = "https://www.hltv.org/ranking/teams/2024/january/1";
    let b = "https://www.hltv.org/ranking/teams/2024/january/8";
    let (http, clock, pause) = setup(&[(a, 200, None), (b, 200, None)]);
    let dates = [day("2024-01-01"), day("2024-01-08")];
    let r = collect_rankings(&config(dir.path()), "https://www.hltv.org", &dates, http.clone(), clock.clone(), pause.clone()).await.unwrap();
    assert_eq!((r.fetched_ok, r.requests_made), (2, 2));
    assert_eq!(pause.waits.lock().unwrap().as_slice(), &[Duration::from_secs(6)]);
    let r = collect_rankings(&config(dir.path()), "https://www.hltv.org", &dates, http.clone(), clock, pause).await.unwrap();
    assert_eq!((r.cached_hits, r.requests_made), (2, 0));
    assert_eq!(http.urls.lock().unwrap().len(), 2);
    let tsv = std::fs::read_to_string(dir.path().join("ranking/sources.tsv")).unwrap();
    assert!(tsv.starts_with("file\trequest\tfetched_at\tsha256\n"));
    assert!(tsv.contains("hltv-ranking-2024-01-08.html.gz\thttps://www.hltv.org/ranking/teams/2024/january/8\t"));
}

#[tokio::test]
async fn a_403_stops_the_run_records_it_and_blocks_later_runs() {
    let dir = tempfile::tempdir().unwrap();
    let a = "https://www.hltv.org/ranking/teams/2024/january/1";
    let (http, clock, pause) = setup(&[(a, 403, None)]);
    let dates = [day("2024-01-01"), day("2024-01-08")];
    let r = collect_rankings(&config(dir.path()), "https://www.hltv.org", &dates, http.clone(), clock.clone(), pause.clone()).await.unwrap();
    assert_eq!(r.requests_made, 1);
    assert_eq!(r.stopped.as_ref().unwrap().status, 403);
    assert!(dir.path().join("ranking/stop.json").is_file());
    assert!(dir.path().join("ranking/blocked/hltv-ranking-2024-01-01-blocked.html").is_file());
    assert!(!dir.path().join("ranking/hltv-ranking-2024-01-01.html.gz").exists());
    let again = collect_rankings(&config(dir.path()), "https://www.hltv.org", &dates, http.clone(), clock, pause).await;
    assert!(matches!(again, Err(CollectorError::AlreadyStopped)));
    assert_eq!(http.urls.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn same_origin_ranking_redirects_are_counted_and_others_refused() {
    let dir = tempfile::tempdir().unwrap();
    let a = "https://www.hltv.org/ranking/teams/2024/january/1";
    let (http, clock, pause) = setup(&[(a, 301, Some("/ranking/teams/2024/january/1/")), ("https://www.hltv.org/ranking/teams/2024/january/1/", 200, None)]);
    let r = collect_rankings(&config(dir.path()), "https://www.hltv.org", &[day("2024-01-01")], http, clock, pause).await.unwrap();
    assert_eq!((r.fetched_ok, r.requests_made), (1, 2));
    let (http, clock, pause) = setup(&[(a, 301, Some("https://evil.example/ranking/teams/x"))]);
    let dir2 = tempfile::tempdir().unwrap();
    let e = collect_rankings(&config(dir2.path()), "https://www.hltv.org", &[day("2024-01-01")], http, clock, pause).await;
    assert!(matches!(e, Err(CollectorError::DisallowedPath(_))));
}

#[test]
fn ranking_cli_needs_one_connection_and_six_seconds() {
    let args = |extra: &[&str]| -> Vec<String> {
        ["--ranking-from", "2023-09-25", "--ranking-until", "2023-10-02", "--out", "x", "--ios-version", "17.2.1"]
            .iter().chain(extra).map(|s| s.to_string()).collect()
    };
    assert!(cli::parse(&args(&["--min-interval-secs", "6"])).is_ok());
    assert!(cli::parse(&args(&["--min-interval-secs", "5"])).is_err());
    assert!(cli::parse(&args(&["--min-interval-secs", "6", "--concurrency", "2"])).is_err());
}

#[test]
fn parses_the_saved_ranking_page() {
    use flate2::read::GzDecoder;
    use std::io::Read;
    let raw = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/research/hltv-ranking-2024-01-01.html.gz")).unwrap();
    let mut html = String::new();
    GzDecoder::new(&raw[..]).read_to_string(&mut html).unwrap();
    let page = kallbet_ai::adapters::hltv_research::ranking::parse_ranking(&html).unwrap();
    assert_eq!(page.date, day("2024-01-01"));
    assert_eq!(page.teams.len(), 220);
    let first = &page.teams[0];
    assert_eq!((first.rank, first.team_id, first.name.as_str(), first.points), (1, 9565, "Vitality", 980));
    assert!(page.teams.windows(2).all(|w| w[0].rank <= w[1].rank && w[0].points >= w[1].points));
    assert!(kallbet_ai::adapters::hltv_research::ranking::parse_ranking("<html>Just a moment...<title>Just a moment...</title></html>").is_err());
}
