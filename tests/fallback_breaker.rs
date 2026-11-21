//! The fallback chain's circuit breaker (A10): what counts towards opening
//! it, and the single probe once it is half-open. Driven through the
//! `MatchSource` port of a `FallbackSource` composed of fakes; time is the
//! test's `FakeClock`.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use async_trait::async_trait;
use chrono::Duration;
use kallbet::domain::{DataSourceKind, Event, EventId, EventSnapshot, StageKind};
use kallbet::ports::{Clock, MatchSource, SourceError};
use kallbet::sources::fallback::{FallbackSettings, FallbackSource};
use kallbet::testing::{EventSnapshotBuilder, ScriptedSource, TestApp};
use tokio::sync::Notify;

const EVENT: i64 = 7148;

/// HLTV as the primary: its Event list works while its Event pages fail
/// (`failing`), as when HLTV changes the markup of one page type. With
/// `gate` set, the first fetch that gets through waits for `release`.
struct Hltv {
    inner: ScriptedSource,
    failing: AtomicBool,
    fetches: AtomicUsize,
    gate: AtomicBool,
    release: Notify,
}

impl Hltv {
    fn new() -> Hltv {
        Hltv {
            inner: ScriptedSource::of(DataSourceKind::Hltv),
            failing: AtomicBool::new(false),
            fetches: AtomicUsize::new(0),
            gate: AtomicBool::new(false),
            release: Notify::new(),
        }
    }

    fn fetches(&self) -> usize {
        self.fetches.load(Ordering::SeqCst)
    }
}

#[async_trait]
impl MatchSource for Hltv {
    async fn list_events(&self) -> Result<Vec<Event>, SourceError> {
        self.inner.list_events().await
    }

    async fn fetch_event(&self, id: EventId) -> Result<EventSnapshot, SourceError> {
        self.fetches.fetch_add(1, Ordering::SeqCst);
        if self.failing.load(Ordering::SeqCst) {
            return Err(SourceError::Parse("standings row missing".into()));
        }
        // Only the first call after the gate is set waits.
        if self.gate.swap(false, Ordering::SeqCst) {
            self.release.notified().await;
        }
        self.inner.fetch_event(id).await
    }
}

async fn chain() -> (TestApp, Arc<Hltv>) {
    let hltv = Arc::new(Hltv::new());
    let primary = hltv.clone();
    let app = TestApp::builder()
        .source_with(move |db, clock| {
            Arc::new(FallbackSource::new(db, clock, FallbackSettings::default()).with_primary(primary))
        })
        .build()
        .await;
    hltv.inner.put_event(
        EventSnapshotBuilder::new(EVENT, "Test Major")
            .stage(StageKind::Swiss, "Stage 1")
            .swiss_match(101, (1, 0, 0), "NAVI", "FaZe", app.clock.now() + Duration::hours(2))
            .build(app.clock.now()),
    );
    (app, hltv)
}

#[tokio::test]
async fn a_working_event_list_does_not_hide_failing_event_pages_from_the_breaker() {
    let (app, hltv) = chain().await;
    let source = app.state.source.clone();
    hltv.failing.store(true, Ordering::SeqCst);

    assert!(source.fetch_event(EventId(EVENT)).await.is_err());
    assert!(source.fetch_event(EventId(EVENT)).await.is_err());
    assert!(source.list_events().await.is_ok(), "the Event list still parses");
    assert!(source.fetch_event(EventId(EVENT)).await.is_err());
    assert_eq!(hltv.fetches(), 3);

    let _ = source.fetch_event(EventId(EVENT)).await;
    assert_eq!(hltv.fetches(), 3, "three Event page failures in a row open the breaker");
}

#[tokio::test]
async fn a_half_open_breaker_lets_exactly_one_call_through() {
    let (app, hltv) = chain().await;
    let source = app.state.source.clone();
    hltv.failing.store(true, Ordering::SeqCst);
    for _ in 0..3 {
        assert!(source.fetch_event(EventId(EVENT)).await.is_err());
    }
    app.clock.advance(Duration::minutes(15));
    hltv.failing.store(false, Ordering::SeqCst);
    hltv.gate.store(true, Ordering::SeqCst);

    // The probe is in flight (held at the gate) when a second call arrives.
    let probe = source.fetch_event(EventId(EVENT));
    let second = async {
        while hltv.fetches() < 4 {
            tokio::task::yield_now().await;
        }
        let result = source.fetch_event(EventId(EVENT)).await;
        hltv.release.notify_one();
        result
    };
    let (probe, second) = tokio::join!(probe, second);

    assert!(probe.is_ok(), "{probe:?}");
    assert!(second.is_err(), "refused while the probe runs: {second:?}");
    assert_eq!(hltv.fetches(), 4, "only the probe reached HLTV");
    assert!(source.fetch_event(EventId(EVENT)).await.is_ok(), "closed after the probe");
}
