mod support;

use async_trait::async_trait;
use chrono::Duration;
use kallbet::{domain::StageKind, poller, ports::Clock as _, testing::EventSnapshotBuilder};
use kallbet_ai::{
    AsOf, BotProfile, Clock, DecisionPolicy, Dossier, Instant, LlmAdjustment, Policy, Prediction,
    Result, Sidecar, TemporalLeakError, Tier,
    adapters::odds::{FallbackOdds, FetchError, MarketOnly, OddsConfig, OddsTransport},
};
use reqwest::Url;
use serde_json::json;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicI64, Ordering},
};
use support::Harness;

const POLYMARKET: &[u8] =
    include_bytes!("fixtures/odds/polymarket-event-cs2-hero-faze-2026-11-26-open.json");
const KALSHI: &[u8] = include_bytes!("fixtures/odds/kalshi-events-KXCS2GAME-open.json");

#[derive(Default)]
struct Feeds {
    hosts: Mutex<Vec<String>>,
}

#[async_trait]
impl OddsTransport for Feeds {
    async fn get(&self, url: Url) -> Result<Vec<u8>, FetchError> {
        let host = url.host_str().unwrap();
        self.hosts.lock().unwrap().push(host.into());
        match host {
            "api.oddspapi.io" => Err(FetchError::RateLimited),
            "gamma-api.polymarket.com" => Ok(POLYMARKET.to_vec()),
            "api.elections.kalshi.com" => {
                assert_eq!(
                    url.query_pairs()
                        .find(|(key, _)| key == "series_ticker")
                        .unwrap()
                        .1,
                    "KXCS2GAME"
                );
                if let Some((_, cursor)) = url.query_pairs().find(|(key, _)| key == "cursor") {
                    // Use the saved first page and derive an empty terminal page.
                    let mut page: serde_json::Value = serde_json::from_slice(KALSHI).unwrap();
                    assert_eq!(page["cursor"], cursor.as_ref());
                    page["events"].as_array_mut().unwrap().clear();
                    page["cursor"] = json!("");
                    Ok(serde_json::to_vec(&page).unwrap())
                } else {
                    Ok(KALSHI.to_vec())
                }
            }
            _ => panic!("unexpected odds host"),
        }
    }
}

async fn market_bot(
    h: &Harness,
    now: Instant,
    teams: (&str, &str),
    config: OddsConfig,
    feeds: Arc<dyn OddsTransport>,
) -> (Sidecar, String) {
    h.app.clock.set(now);
    h.app.source.put_event(
        EventSnapshotBuilder::new(99, "Major")
            .stage(StageKind::Swiss, "Stage 1")
            .swiss_match(
                42,
                (1, 0, 0),
                teams.0,
                teams.1,
                "2026-11-26T06:00:00Z".parse().unwrap(),
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
    let mut sidecar = h.sidecar(&invite).await;
    sidecar.bots[0].policy = Policy::MarketOnly;
    sidecar.ports.policy = Arc::new(MarketOnly);
    sidecar.ports.odds = Arc::new(FallbackOdds::new(&sidecar.audit, config, feeds));
    (sidecar, format!("/api/leagues/{}/matches", league["id"]))
}

#[tokio::test]
async fn market_bot_submits_the_polymarket_favourite_when_pinnacle_is_rate_limited() {
    let h = Harness::new().await;
    let feed = Arc::new(Feeds::default());
    let start: Instant = "2026-11-26T06:00:00Z".parse().unwrap();
    let now = start - chrono::Duration::minutes(23);
    let (mut sidecar, path) = market_bot(
        &h,
        now,
        ("FaZe", "Heroic"),
        OddsConfig {
            oddspapi_key: Some("test-key".into()),
            oddspapi_tournaments: [(99, vec![400014])].into(),
            polymarket_tags: [(99, "pgl-major-singapore-2026".into())].into(),
            ..OddsConfig::default()
        },
        feed.clone(),
    )
    .await;
    assert_eq!(h.tick(&mut sidecar, now).await.unwrap(), 1);
    assert_eq!(
        h.tick(&mut sidecar, now + chrono::Duration::minutes(1))
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        *feed.hosts.lock().unwrap(),
        ["api.oddspapi.io", "gamma-api.polymarket.com"]
    );
    assert!(h.scripts.llm_calls.lock().unwrap().is_empty());
    let bot = h.app.bot("vrs", "Baseline").await;
    let view = bot.get(&path).await.json();
    assert_eq!(
        view["stages"][0]["groups"][0]["matches"][0]["my_prediction"],
        2
    );
    let predictions = sidecar.audit.submissions().await.unwrap();
    assert!((predictions[0].rationale["p_win"].as_f64().unwrap() - 0.545).abs() < 1e-12);
    assert_eq!(predictions[0].rationale["preliminary"], false);
    assert_eq!(predictions[0].rationale["override_market"], false);
}

#[tokio::test]
async fn market_bot_uses_valve_standings_if_neither_public_feed_has_a_line() {
    let h = Harness::new().await;
    let now = "2026-11-25T17:20:00Z".parse().unwrap();
    let feed = Arc::new(Feeds::default());
    let (mut sidecar, path) = market_bot(
        &h,
        now,
        ("Vitality", "MOUZ"),
        OddsConfig {
            oddspapi_key: Some("test-key-with-no-event-mapping".into()),
            polymarket_tags: [(99, "pgl-major-singapore-2026".into())].into(),
            ..OddsConfig::default()
        },
        feed.clone(),
    )
    .await;
    assert_eq!(h.tick(&mut sidecar, now).await.unwrap(), 1);
    assert_eq!(
        *feed.hosts.lock().unwrap(),
        [
            "gamma-api.polymarket.com",
            "api.elections.kalshi.com",
            "api.elections.kalshi.com"
        ]
    );
    let bot = h.app.bot("vrs", "Baseline").await;
    let view = bot.get(&path).await.json();
    assert_eq!(
        view["stages"][0]["groups"][0]["matches"][0]["my_prediction"],
        2
    );
    let predictions = sidecar.audit.submissions().await.unwrap();
    assert_eq!(
        predictions[0].rationale["summary"]["en"],
        "Prediction based on the Valve Regional Standings."
    );
    assert_eq!(predictions[0].rationale["preliminary"], true);
    assert!(h.scripts.llm_calls.lock().unwrap().is_empty());
}

#[tokio::test]
async fn market_bot_preserves_a_favourite_that_is_very_close_to_even() {
    let h = Harness::new().await;
    let now: Instant = "2026-11-25T17:20:00Z".parse().unwrap();
    let (mut sidecar, path) = market_bot(
        &h,
        now,
        ("Vitality", "MOUZ"),
        OddsConfig::default(),
        Arc::new(Feeds::default()),
    )
    .await;
    *h.scripts.market.lock().unwrap() =
        Some(AsOf::new(0.49999, now - chrono::Duration::seconds(1), now).unwrap());
    sidecar.ports.odds = h.scripts.clone();
    assert_eq!(h.tick(&mut sidecar, now).await.unwrap(), 1);
    let bot = h.app.bot("vrs", "Baseline").await;
    let view = bot.get(&path).await.json();
    assert_eq!(
        view["stages"][0]["groups"][0]["matches"][0]["my_prediction"],
        2
    );
}

struct MarketPolicyWithLlm;

impl DecisionPolicy for MarketPolicyWithLlm {
    fn wants_adjustment(&self, _: &BotProfile) -> bool {
        true
    }

    fn decide(
        &self,
        bot: &BotProfile,
        dossier: &Dossier,
        tier: Tier,
        adjustment: Option<LlmAdjustment>,
    ) -> Result<Option<Prediction>> {
        MarketOnly.decide(bot, dossier, tier, adjustment)
    }
}

#[tokio::test]
async fn market_moves_only_reopen_the_dossier_when_the_rounded_probability_changes() {
    let h = Harness::new().await;
    let start: Instant = "2026-11-26T06:00:00Z".parse().unwrap();
    let now = start - Duration::minutes(24);
    let (mut sidecar, _) = market_bot(
        &h,
        now,
        ("FaZe", "Heroic"),
        OddsConfig::default(),
        Arc::new(Feeds::default()),
    )
    .await;
    sidecar.ports.odds = h.scripts.clone();
    sidecar.ports.policy = Arc::new(MarketPolicyWithLlm);

    for (offset, probability, expected_submission, expected_runs) in
        [(0, 0.60001, 1, 1), (1, 0.60004, 0, 1), (2, 0.60016, 1, 2)]
    {
        let tick = now + Duration::minutes(offset);
        *h.scripts.market.lock().unwrap() =
            Some(AsOf::new(probability, tick - Duration::seconds(1), tick).unwrap());
        assert_eq!(
            h.tick(&mut sidecar, tick).await.unwrap(),
            expected_submission
        );
        assert_eq!(h.scripts.llm_calls.lock().unwrap().len(), expected_runs);
        assert_eq!(
            sidecar.audit.submissions().await.unwrap().len(),
            expected_runs
        );
    }

    let predictions = sidecar.audit.submissions().await.unwrap();
    for (prediction, expected) in predictions.iter().zip([0.60001, 0.60016]) {
        assert!((prediction.rationale["p_win"].as_f64().unwrap() - expected).abs() < 1e-12);
    }
}

// Model elapsed time during live ingestion while keeping platform time fixed
// for the entire tick. The first read is the harness's exact tick time.
struct AdvancingClock(Arc<kallbet::testing::FakeClock>, AtomicI64);

impl Clock for AdvancingClock {
    fn now(&self) -> Instant {
        self.0.now() + chrono::Duration::nanoseconds(self.1.fetch_add(1, Ordering::Relaxed))
    }
}

#[tokio::test]
async fn market_bot_accepts_a_kalshi_quote_captured_in_the_tick_and_rejects_cutoff_equality() {
    let h = Harness::new().await;
    let feed = Arc::new(Feeds::default());
    let start: Instant = "2026-11-26T06:00:00Z".parse().unwrap();
    let now = start - chrono::Duration::minutes(24);
    let (mut sidecar, path) = market_bot(
        &h,
        now,
        ("FaZe", "Heroic"),
        OddsConfig::default(),
        feed.clone(),
    )
    .await;
    let error = h.tick(&mut sidecar, now).await.unwrap_err();
    let temporal = error.downcast::<TemporalLeakError>().unwrap();
    assert_eq!(temporal.cutoff, now);
    assert_eq!(temporal.observed_at, now);
    assert!(sidecar.audit.submissions().await.unwrap().is_empty());

    // Refetch after expiration. The new response is observed after the tick
    // starts and strictly before the final injected-clock evaluation cutoff.
    sidecar.ports.clock = Arc::new(AdvancingClock(h.app.clock.clone(), AtomicI64::new(0)));
    let later = now + chrono::Duration::minutes(2);
    assert_eq!(h.tick(&mut sidecar, later).await.unwrap(), 1);
    assert_eq!(*feed.hosts.lock().unwrap(), ["api.elections.kalshi.com"; 4]);
    let bot = h.app.bot("vrs", "Baseline").await;
    let view = bot.get(&path).await.json();
    assert_eq!(
        view["stages"][0]["groups"][0]["matches"][0]["my_prediction"],
        2
    );
    let predictions = sidecar.audit.submissions().await.unwrap();
    assert!((predictions[0].rationale["p_win"].as_f64().unwrap() - 0.542288557).abs() < 1e-9);
}
