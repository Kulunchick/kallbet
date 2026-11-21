//! The HLTV fetch layer against a real headless Chrome and the real HLTV.
//! Ignored by default (network, ~1 minute). Run it by hand:
//!
//! ```text
//! docker run -d --rm --name chrome --init --shm-size 2g -p 127.0.0.1:9222:9222 chromedp/headless-shell:stable
//! KALLBET_CHROME_URL=http://127.0.0.1:9222 cargo test --test hltv_live -- --ignored --nocapture
//! ```
//!
//! Keep it rare: every run loads several HLTV pages.

use std::sync::Arc;

use kallbet::domain::{EventId, StageKind};
use kallbet::ports::{MatchSource, SystemClock};
use kallbet::sources::hltv::{ChromePages, HltvSource, LogoCache};

#[tokio::test]
#[ignore = "needs KALLBET_CHROME_URL and the network"]
async fn a_stage_event_is_fetched_as_its_major_through_chrome() {
    let endpoint = std::env::var("KALLBET_CHROME_URL").expect("KALLBET_CHROME_URL");
    let logo_dir = tempfile::tempdir().unwrap();
    let hltv = HltvSource::new(Arc::new(ChromePages::new(endpoint)), Arc::new(SystemClock))
        .with_logos(LogoCache::new(logo_dir.path()));

    // IEM Cologne Major 2026 Stage 1 -> the Major (8301): four HLTV pages
    // and the live list.
    let major = hltv.fetch_event(EventId(9028)).await.unwrap();
    println!(
        "{} ({}): {:?}, {} matches, {} teams",
        major.event.name,
        major.event.id,
        major.stages.iter().map(|s| &s.name).collect::<Vec<_>>(),
        major.matches.len(),
        major.teams.len()
    );
    assert_eq!(major.event.id, EventId(8301));
    assert_eq!(major.stages.last().unwrap().kind, StageKind::Playoffs);
    assert!(major.matches.len() > 100);

    let logos = std::fs::read_dir(logo_dir.path())
        .unwrap()
        .filter_map(Result::ok)
        .filter(|f| !f.file_name().to_string_lossy().ends_with(".source"))
        .count();
    println!("{logos} logos stored");
    assert!(logos > 20, "{logos} logos");
}
