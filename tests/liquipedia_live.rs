//! The Liquipedia fetch layer against the real API. Ignored by default
//! (network, about a minute because of the published rate limits). Run it by
//! hand, rarely:
//!
//! ```text
//! cargo test --test liquipedia_live -- --ignored --nocapture
//! ```
//!
//! It sends a handful of requests, 2 seconds or more apart, with the
//! project's User-Agent.

use std::sync::Arc;

use kallbet::domain::StageKind;
use kallbet::ports::SystemClock;
use kallbet::sources::fallback::{EventHint, ExternalSource};
use kallbet::sources::liquipedia::LiquipediaSource;

#[tokio::test]
#[ignore = "needs the network"]
async fn a_major_is_fetched_from_the_real_api() {
    let liquipedia = LiquipediaSource::new(Arc::new(SystemClock));

    // Find it the way the chain does, by the name HLTV uses.
    let found = liquipedia
        .find_events(&EventHint {
            id: kallbet::domain::EventId(8301),
            name: "IEM Cologne Major 2026".into(),
            starts_at: None,
            ends_at: None,
        })
        .await
        .unwrap();
    println!("found: {found:?}");
    let event = found
        .iter()
        .find(|e| e.name == "IEM Cologne Major 2026")
        .expect("the Cologne Major among the candidates");

    let snapshot = liquipedia.fetch_event(&event.external_id).await.unwrap();
    println!(
        "{} ({}): stages {:?}, {} matches",
        snapshot.event.name,
        snapshot.event.external_id,
        snapshot.stages.iter().map(|s| &s.name).collect::<Vec<_>>(),
        snapshot.matches.len()
    );
    let with_hltv = snapshot.matches.iter().filter(|m| m.canonical.is_some()).count();
    let named = snapshot
        .matches
        .iter()
        .filter_map(|m| m.team_a.as_ref())
        .filter(|t| t.name != t.external_id)
        .count();
    println!("{with_hltv} matches carry an HLTV id, {named} first teams have a real name");

    assert_eq!(snapshot.stages.last().unwrap().kind, StageKind::Playoffs);
    assert!(snapshot.stages.iter().filter(|s| s.kind == StageKind::Swiss).count() >= 3);
    assert!(snapshot.matches.len() > 100);
    assert!(with_hltv > 100);
    assert!(named > 100);
}
