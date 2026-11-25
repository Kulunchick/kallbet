//! Seam 1: a bot registers once and keeps its token; when the server refuses
//! it (another registration under the same bot key rotated it), the bot
//! registers again in the same tick.
mod support;
use async_trait::async_trait;
use chrono::Duration;
use kallbet::{domain::StageKind, poller, testing::EventSnapshotBuilder};
use kallbet_ai::*;
use serde_json::json;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use support::Harness;

/// The real HTTP adapter, counting registrations.
struct Counting {
    inner: Arc<dyn LeaguePlatform>,
    logins: AtomicUsize,
}
#[async_trait]
impl LeaguePlatform for Counting {
    async fn login(&self, bot: &BotProfile) -> Result<String> {
        self.logins.fetch_add(1, Ordering::SeqCst);
        self.inner.login(bot).await
    }
    async fn join(&self, token: &str, invite: &str) -> Result<Join> {
        self.inner.join(token, invite).await
    }
    async fn leagues(&self, token: &str) -> Result<Vec<i64>> {
        self.inner.leagues(token).await
    }
    async fn matches(&self, token: &str, league: i64) -> Result<Vec<(Match, bool)>> {
        self.inner.matches(token, league).await
    }
    async fn submit(&self, token: &str, league: i64, game: i64, pick: &Prediction) -> Result<()> {
        self.inner.submit(token, league, game, pick).await
    }
}

#[tokio::test]
async fn a_bot_keeps_its_token_and_registers_again_only_when_it_is_refused() {
    let h = Harness::new().await;
    let now: Instant = "2026-11-24T00:00:00Z".parse().unwrap();
    h.app.clock.set(now);
    h.app.source.put_event(
        EventSnapshotBuilder::new(99, "Major")
            .stage(StageKind::Swiss, "Stage 1")
            .swiss_match(42, (1, 0, 0), "MOUZ", "Vitality", now + Duration::hours(4))
            .swiss_match(43, (1, 0, 1), "FaZe", "Spirit", now + Duration::hours(6))
            .build(now),
    );
    let human = h.app.user("Alice").await;
    let league = human
        .post("/api/leagues", &json!({"name":"Friends","event_id":99}))
        .await
        .json();
    poller::tick(&h.app.state).await;
    let invite = format!(
        "{}/join/{}",
        h.base,
        league["invite_token"].as_str().unwrap()
    );
    let mut sidecar = h.sidecar(&invite).await;
    let platform = Arc::new(Counting {
        inner: sidecar.ports.platform.clone(),
        logins: AtomicUsize::new(0),
    });
    sidecar.ports.platform = platform.clone();

    assert_eq!(h.tick(&mut sidecar, now).await.unwrap(), 2);
    for minutes in 1..=3 {
        h.tick(&mut sidecar, now + Duration::minutes(minutes))
            .await
            .unwrap();
    }
    assert_eq!(platform.logins.load(Ordering::SeqCst), 1, "one registration for four ticks");

    // Someone registers the same bot key: the kept token stops working.
    h.app.bot("vrs", "Baseline").await;
    let later = now + Duration::minutes(10);
    h.tick(&mut sidecar, later).await.unwrap();
    assert_eq!(platform.logins.load(Ordering::SeqCst), 2);
    assert!(
        sidecar.audit.failures().await.unwrap().is_empty(),
        "a refused token is not a failed step"
    );
    h.tick(&mut sidecar, later + Duration::minutes(1))
        .await
        .unwrap();
    assert_eq!(platform.logins.load(Ordering::SeqCst), 2);
}
