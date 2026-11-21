//! Seam 2: the HLTV adapter behind `MatchSource`, fed saved pages instead of
//! a browser. No network.

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use kallbet::domain::{DataSourceKind, EventId, MatchId, MatchStatus, StageKind};
use kallbet::ports::{MatchSource, SourceError};
use kallbet::sources::hltv::{HltvSource, PageLoader};
use kallbet::testing::FakeClock;

/// Serves the saved page whose URL starts with the given prefix and records
/// every URL asked for.
#[derive(Default)]
struct SavedPages {
    pages: Vec<(&'static str, &'static str)>,
    loaded: Mutex<Vec<String>>,
    fail: Mutex<Option<SourceError>>,
}

impl SavedPages {
    fn cologne() -> SavedPages {
        SavedPages {
            pages: vec![
                ("https://www.hltv.org/events/8301/", "event-8301-cologne-major-parent.html"),
                ("https://www.hltv.org/events/9028/", "event-9028-cologne-stage1.html"),
                ("https://www.hltv.org/events/9029/", "event-9029-cologne-stage2.html"),
                ("https://www.hltv.org/events/9469/", "event-9469-swiss-plus-playoffs.html"),
                ("https://www.hltv.org/matches", "matches.html"),
                ("https://www.hltv.org/events", "events-list.html"),
            ],
            ..SavedPages::default()
        }
    }

    fn loaded(&self) -> Vec<String> {
        self.loaded.lock().unwrap().clone()
    }
}

#[async_trait]
impl PageLoader for SavedPages {
    async fn load(&self, url: &str) -> Result<String, SourceError> {
        self.loaded.lock().unwrap().push(url.to_string());
        if let Some(e) = self.fail.lock().unwrap().clone() {
            return Err(e);
        }
        let (_, file) = self
            .pages
            .iter()
            .find(|(prefix, _)| url.starts_with(prefix))
            .unwrap_or_else(|| panic!("no saved page for {url}"));
        let path = format!("{}/tests/fixtures/hltv/{file}", env!("CARGO_MANIFEST_DIR"));
        Ok(std::fs::read_to_string(path).unwrap())
    }
}

fn source(pages: &Arc<SavedPages>) -> HltvSource {
    let clock = Arc::new(FakeClock::new());
    HltvSource::new(pages.clone(), clock)
}

#[tokio::test]
async fn a_stage_event_id_yields_the_whole_major() {
    let pages = Arc::new(SavedPages::cologne());
    let hltv = source(&pages);

    let major = hltv.fetch_event(EventId(9028)).await.unwrap();

    assert_eq!(major.event.id, EventId(8301));
    assert_eq!(major.source, DataSourceKind::Hltv);
    let kinds: Vec<StageKind> = major.stages.iter().map(|s| s.kind).collect();
    assert_eq!(
        kinds,
        vec![StageKind::Swiss, StageKind::Swiss, StageKind::Swiss, StageKind::Playoffs]
    );
    assert_eq!(major.fetched_at, Utc.with_ymd_and_hms(2026, 1, 10, 12, 0, 0).unwrap());
}

#[tokio::test]
async fn finished_stage_pages_are_loaded_once() {
    let pages = Arc::new(SavedPages::cologne());
    let hltv = source(&pages);

    hltv.fetch_event(EventId(8301)).await.unwrap();
    hltv.fetch_event(EventId(8301)).await.unwrap();

    let loads = |prefix: &str| pages.loaded().iter().filter(|u| u.starts_with(prefix)).count();
    assert_eq!(loads("https://www.hltv.org/events/8301/"), 2);
    assert_eq!(loads("https://www.hltv.org/events/9028/"), 1);
    assert_eq!(loads("https://www.hltv.org/events/9029/"), 1);
}

#[tokio::test]
async fn the_live_list_marks_matches_being_played() {
    let pages = Arc::new(SavedPages::cologne());
    let hltv = source(&pages);

    let event = hltv.fetch_event(EventId(9469)).await.unwrap();

    assert_eq!(event.match_by_id(MatchId(2400366)).unwrap().status, MatchStatus::Live);
    assert!(pages.loaded().iter().any(|u| u == "https://www.hltv.org/matches"));
}

#[tokio::test]
async fn the_event_list_comes_from_the_events_page() {
    let pages = Arc::new(SavedPages::cologne());
    let hltv = source(&pages);

    let events = hltv.list_events().await.unwrap();

    assert!(events.iter().any(|e| e.id == EventId(8302)));
    assert_eq!(pages.loaded(), vec!["https://www.hltv.org/events".to_string()]);
}

#[tokio::test]
async fn a_failing_browser_is_an_unavailable_source() {
    let pages = Arc::new(SavedPages::cologne());
    *pages.fail.lock().unwrap() = Some(SourceError::Unavailable("chrome is down".into()));
    let hltv = source(&pages);

    let err = hltv.fetch_event(EventId(8301)).await.unwrap_err();
    assert!(matches!(err, SourceError::Unavailable(_)), "{err:?}");
    let err = hltv.list_events().await.unwrap_err();
    assert!(matches!(err, SourceError::Unavailable(_)), "{err:?}");
}
