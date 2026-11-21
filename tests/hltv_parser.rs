//! Seam 2: the HLTV parser on saved pages (`tests/fixtures/hltv/`). No network.

use chrono::{DateTime, TimeZone, Utc};
use kallbet::domain::{Event, EventId};
use kallbet::sources::hltv::parse;

fn fixture(name: &str) -> String {
    let path = format!("{}/tests/fixtures/hltv/{name}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

fn ms(millis: i64) -> DateTime<Utc> {
    Utc.timestamp_millis_opt(millis).unwrap()
}

#[test]
fn the_event_list_has_ongoing_and_big_upcoming_events_with_hltv_ids() {
    let events = parse::event_list(&fixture("events-list.html")).unwrap();

    let major = events.iter().find(|e| e.id == EventId(8302)).unwrap();
    assert_eq!(
        major,
        &Event {
            id: EventId(8302),
            name: "PGL Major Singapore 2026".into(),
            starts_at: Some(ms(1796209200000)),
            ends_at: Some(ms(1797159600000)),
        }
    );
    let ongoing = events.iter().find(|e| e.id == EventId(9469)).unwrap();
    assert_eq!(ongoing.name, "NODWIN Clutch Series 14");

    // Each Event once, although HLTV repeats ongoing ones in two tabs.
    assert_eq!(events.iter().filter(|e| e.id == EventId(9469)).count(), 1);
}

#[test]
fn the_event_list_leaves_out_stages_of_a_listed_major() {
    let events = parse::event_list(&fixture("events-list.html")).unwrap();
    let ids: Vec<EventId> = events.iter().map(|e| e.id).collect();

    // PGL Major Singapore 2026 Stage 1 and 2 belong to the Major (8302).
    assert!(!ids.contains(&EventId(9303)));
    assert!(!ids.contains(&EventId(9304)));
    // IEM Kraków 2027 Stage 1 belongs to IEM Kraków 2027 (8318).
    assert!(!ids.contains(&EventId(9489)));
    assert!(ids.contains(&EventId(8318)));
}
