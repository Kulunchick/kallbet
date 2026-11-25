//! The periodic loop: cadence, failure handling, shutdown, heartbeat. Tokio's
//! paused clock stands in for real time, so nothing here waits.
use chrono::{Duration as Span, Utc};
use kallbet_ai::{
    Clock, Instant, Result,
    adapters::production::{
        Health, Heartbeat,
        health::Limits,
        runner::{self, LoopSettings, Tick},
    },
};
use std::{
    collections::VecDeque,
    sync::Arc,
    time::Duration,
};
use tokio::sync::Notify;

struct Scripted {
    steps: VecDeque<Result<usize, &'static str>>,
    started: Arc<Notify>,
    hang_after_script: bool,
    calls: usize,
}
impl Scripted {
    fn new(steps: Vec<Result<usize, &'static str>>) -> Self {
        Self {
            steps: steps.into(),
            started: Arc::default(),
            hang_after_script: false,
            calls: 0,
        }
    }
}
#[async_trait::async_trait]
impl Tick for Scripted {
    async fn tick(&mut self) -> Result<usize> {
        self.calls += 1;
        self.started.notify_one();
        match self.steps.pop_front() {
            Some(step) => step.map_err(Into::into),
            None if self.hang_after_script => std::future::pending().await,
            None => Ok(0),
        }
    }
}
struct WallClock;
impl Clock for WallClock {
    fn now(&self) -> Instant {
        Utc::now()
    }
}
const SETTINGS: LoopSettings = LoopSettings {
    interval: Duration::from_secs(60),
    tick_timeout: Duration::from_secs(300),
};
fn health() -> (tempfile::TempDir, Health) {
    let dir = tempfile::tempdir().unwrap();
    let health = Health::new(dir.path().join("sidecar.health"));
    (dir, health)
}

#[tokio::test(start_paused = true)]
async fn ticks_once_per_interval_until_the_shutdown_signal() {
    let (_dir, health) = health();
    let mut ticker = Scripted::new(vec![Ok(1), Ok(0), Ok(2)]);
    // The first tick is at once, then one per 60 s: ticks at 0, 60 and 120 s, and
    // the signal at 150 s arrives between the third and the fourth.
    let summary = runner::run(
        &mut ticker,
        SETTINGS,
        &WallClock,
        &health,
        tokio::time::sleep(Duration::from_secs(150)),
    )
    .await;
    assert_eq!(summary.ticks, 3);
    assert_eq!(summary.failed_ticks, 0);
    assert!(!summary.interrupted);
    let beat = health.read().unwrap();
    assert_eq!(beat.submitted, 2);
    assert_eq!(beat.consecutive_failures, 0);
    assert!(beat.finished_at.is_some());
}

#[tokio::test(start_paused = true)]
async fn a_failing_tick_does_not_stop_the_loop_and_is_counted() {
    let (_dir, health) = health();
    let mut ticker = Scripted::new(vec![Err("platform down"), Err("platform down"), Ok(3)]);
    let summary = runner::run(
        &mut ticker,
        SETTINGS,
        &WallClock,
        &health,
        tokio::time::sleep(Duration::from_secs(150)),
    )
    .await;
    assert_eq!((summary.ticks, summary.failed_ticks), (3, 2));
    // The third tick succeeded: the failure streak is over.
    assert_eq!(health.read().unwrap().consecutive_failures, 0);
}

#[tokio::test(start_paused = true)]
async fn the_failure_streak_is_visible_in_the_heartbeat() {
    let (_dir, health) = health();
    let mut ticker = Scripted::new(vec![Err("down"), Err("down")]);
    ticker.hang_after_script = true;
    let summary = runner::run(
        &mut ticker,
        SETTINGS,
        &WallClock,
        &health,
        tokio::time::sleep(Duration::from_secs(90)),
    )
    .await;
    assert_eq!(summary.failed_ticks, 2);
    let beat = health.read().unwrap();
    assert_eq!(beat.consecutive_failures, 2);
}

#[tokio::test(start_paused = true)]
async fn a_signal_during_a_tick_drops_the_tick_and_returns() {
    let (_dir, health) = health();
    let mut ticker = Scripted::new(vec![]);
    ticker.hang_after_script = true;
    let started = ticker.started.clone();
    let summary = runner::run(&mut ticker, SETTINGS, &WallClock, &health, async move {
        started.notified().await;
    })
    .await;
    assert!(summary.interrupted);
    assert_eq!(summary.ticks, 0);
    // The heartbeat records the tick as begun and not finished.
    let beat = health.read().unwrap();
    assert!(beat.finished_at.is_none());
}

#[tokio::test(start_paused = true)]
async fn a_tick_over_its_timeout_is_dropped_and_counts_as_a_failure() {
    let (_dir, health) = health();
    let mut ticker = Scripted::new(vec![]);
    ticker.hang_after_script = true;
    let settings = LoopSettings {
        interval: Duration::from_secs(60),
        tick_timeout: Duration::from_secs(100),
    };
    let summary = runner::run(
        &mut ticker,
        settings,
        &WallClock,
        &health,
        tokio::time::sleep(Duration::from_secs(250)),
    )
    .await;
    assert!(summary.failed_ticks >= 1);
    assert!(ticker.calls >= 1);
    assert!(health.read().unwrap().consecutive_failures >= 1);
}

#[tokio::test(start_paused = true)]
async fn a_shutdown_that_is_already_signalled_runs_no_tick() {
    let (_dir, health) = health();
    let mut ticker = Scripted::new(vec![Ok(1)]);
    let summary = runner::run(&mut ticker, SETTINGS, &WallClock, &health, async {}).await;
    assert_eq!(summary.ticks, 0);
    assert_eq!(ticker.calls, 0);
}

#[test]
fn the_health_verdict_flags_a_stale_or_failing_loop() {
    let now = "2026-11-25T12:00:00Z".parse::<Instant>().unwrap();
    let limits = Limits {
        max_age: Span::minutes(30),
        max_failures: 5,
    };
    let beat = |finished: Span, failures| Heartbeat {
        started_at: now - finished - Span::seconds(1),
        finished_at: Some(now - finished),
        consecutive_failures: failures,
        submitted: 0,
    };
    assert!(beat(Span::minutes(2), 0).verdict(now, &limits).is_ok());
    assert!(beat(Span::minutes(29), 4).verdict(now, &limits).is_ok());
    assert!(beat(Span::minutes(31), 0).verdict(now, &limits).is_err());
    let failing = beat(Span::minutes(1), 5).verdict(now, &limits).unwrap_err();
    assert!(failing.contains("5 ticks failed in a row"), "{failing}");
    // A tick still running counts by when it started.
    let running = Heartbeat {
        started_at: now - Span::minutes(10),
        finished_at: None,
        consecutive_failures: 0,
        submitted: 0,
    };
    assert!(running.verdict(now, &limits).is_ok());
}

#[test]
fn the_heartbeat_file_round_trips_and_sits_beside_the_audit_file() {
    let health = Health::beside(std::path::Path::new("/data/sidecar.db"));
    assert_eq!(health.path, std::path::Path::new("/data/sidecar.health"));
    let (_dir, health) = health();
    let beat = Heartbeat {
        started_at: Utc::now(),
        finished_at: None,
        consecutive_failures: 3,
        submitted: 7,
    };
    health.write(&beat).unwrap();
    assert_eq!(health.read().unwrap(), beat);
}
