mod support;
use chrono::Duration;
use kallbet::{domain::StageKind, poller, ports::Clock, testing::EventSnapshotBuilder};
use serde_json::{Value, json};
use support::Harness;

#[tokio::test]
async fn restart_reuses_draft_and_prelock_updates_after_human_reveal() {
    let h = Harness::new().await;
    let app = &h.app;
    let now = "2026-11-24T00:00:00Z".parse().unwrap();
    app.clock.set(now);
    let start = now + Duration::hours(4);
    app.source.put_event(
        EventSnapshotBuilder::new(99, "Major")
            .stage(StageKind::Swiss, "Stage 1")
            .swiss_match(42, (1, 0, 0), "MOUZ", "Vitality", start)
            .build(now),
    );
    let human = app.user("Alice").await;
    let league = human
        .post("/api/leagues", &json!({"name":"Friends","event_id":99}))
        .await
        .json();
    let id = league["id"].as_i64().unwrap();
    let invite = format!(
        "{}/join/{}",
        h.base,
        league["invite_token"].as_str().unwrap()
    );
    poller::tick(&app.state).await;
    let mut sidecar = h.sidecar(&invite).await;
    assert_eq!(sidecar.tick().await.unwrap(), 1);
    assert!(h.scripts.llm_calls.lock().unwrap().is_empty());
    drop(sidecar);
    let mut sidecar = h.sidecar(&invite).await;
    assert_eq!(sidecar.tick().await.unwrap(), 0);
    let path = format!("/api/leagues/{id}/matches");
    assert!(
        human
            .put(&format!("{path}/42/prediction"), &json!({"team_id":2}))
            .await
            .status
            .is_success()
    );
    app.clock.advance(Duration::minutes(5));
    let pick = |view: Value| {
        view["stages"][0]["groups"][0]["matches"][0]["predictions"]["revealed"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["is_bot"] == true)
            .unwrap()
            .clone()
    };
    let draft = pick(human.get(&path).await.json());
    assert_eq!(draft["team_id"], 1);
    assert_eq!(draft["rationale"]["preliminary"], true);
    app.clock.set(start - Duration::minutes(22));
    assert_eq!(sidecar.tick().await.unwrap(), 1);
    assert_eq!(sidecar.tick().await.unwrap(), 0);
    let cutoff = app.clock.now();
    *h.scripts.market.lock().unwrap() =
        Some(kallbet_ai::AsOf::new(0.6, cutoff, cutoff + Duration::minutes(1)).unwrap());
    assert!(
        sidecar
            .tick()
            .await
            .unwrap_err()
            .is::<kallbet_ai::TemporalLeakError>()
    );
    assert!(h.scripts.llm_calls.lock().unwrap().is_empty());
    let final_pick = pick(human.get(&path).await.json());
    assert_eq!(final_pick["rationale"]["preliminary"], false);
    assert!(final_pick["rationale"]["p_win"].as_f64().unwrap() > 0.5);
    assert_eq!(sidecar.audit.submissions().await.unwrap().len(), 2);
    assert_eq!(app.clock.now(), start - Duration::minutes(22));
}
