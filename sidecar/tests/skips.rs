mod support;
use chrono::Duration;
use kallbet::{domain::StageKind, poller, ports::Clock, testing::EventSnapshotBuilder};
use serde_json::json;
use support::Harness;

#[tokio::test]
async fn unchanged_unrated_pair_is_audited_once_even_without_a_prediction() {
    let h = Harness::new().await;
    h.app.clock.set("2026-11-24T00:00:00Z".parse().unwrap());
    let now = h.app.clock.now();
    h.app.source.put_event(
        EventSnapshotBuilder::new(99, "Major")
            .stage(StageKind::Swiss, "Stage 1")
            .swiss_match(
                42,
                (1, 0, 0),
                "unrated",
                "Vitality",
                now + Duration::hours(4),
            )
            .build(now),
    );
    let human = h.app.user("Alice").await;
    let league = human
        .post("/api/leagues", &json!({"name":"Friends","event_id":99}))
        .await
        .json();
    let invite = format!(
        "{}/join/{}",
        h.base,
        league["invite_token"].as_str().unwrap()
    );
    poller::tick(&h.app.state).await;
    let mut s = h.sidecar(&invite).await;
    for _ in 0..2 {
        assert_eq!(s.tick().await.unwrap(), 0);
    }
    assert_eq!(h.scripts.llm_calls.lock().unwrap().len(), 1);
    assert!(s.audit.submissions().await.unwrap().is_empty());
}
