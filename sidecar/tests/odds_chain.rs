use async_trait::async_trait;
use chrono::Duration;
use kallbet_ai::{
    Audit, Clock, Format, Instant, Match, OddsSource, Team, Tier,
    adapters::odds::{FallbackOdds, FetchError, OddsConfig, OddsTransport},
};
use reqwest::Url;
use std::sync::{Arc, Mutex};

const PINNACLE: &[u8] =
    include_bytes!("fixtures/odds/oddspapi-odds-by-tournaments-pgl-singapore-pinnacle.json");
const POLYMARKET: &[u8] =
    include_bytes!("fixtures/odds/polymarket-event-cs2-hero-faze-2026-11-26-open.json");

struct FixedClock(Instant);

impl Clock for FixedClock {
    fn now(&self) -> Instant {
        self.0
    }
}

#[derive(Default)]
struct PinnacleFeed {
    batches: Mutex<Vec<String>>,
}

#[async_trait]
impl OddsTransport for PinnacleFeed {
    async fn get(&self, url: Url) -> Result<Vec<u8>, FetchError> {
        assert_eq!(url.path(), "/v4/odds-by-tournaments");
        let params: std::collections::HashMap<_, _> = url.query_pairs().collect();
        assert_eq!(params.get("bookmaker").unwrap(), "pinnacle+30");
        self.batches
            .lock()
            .unwrap()
            .push(params["tournamentIds"].to_string());
        Ok(PINNACLE.to_vec())
    }
}

fn game(id: i64, a: &str, b: &str) -> Match {
    Match {
        id,
        event_id: 99,
        team_a: Team {
            id: 1,
            name: a.into(),
        },
        team_b: Team {
            id: 2,
            name: b.into(),
        },
        format: Format::Bo3,
        scheduled_start: Some("2026-11-26T06:00:00Z".parse().unwrap()),
    }
}

#[tokio::test]
async fn one_event_batch_is_shared_by_matches_and_survives_a_restart_until_ttl() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("sidecar.db");
    let feed = Arc::new(PinnacleFeed::default());
    let config = OddsConfig {
        oddspapi_key: Some("test-key".into()),
        oddspapi_tournaments: [(99, vec![400014, 51216])].into(),
        ..OddsConfig::default()
    };
    let now: Instant = "2026-11-25T17:20:00Z".parse().unwrap();
    let a = game(1, "M80", "Eternal Fire");
    let b = game(2, "FaZe", "Heroic");
    let audit = Audit::open(&path).await.unwrap();
    let chain = FallbackOdds::new(&audit, config.clone(), feed.clone());
    let (_, quote) = chain.live_quote(&a, now, Tier::Draft, &FixedClock(now)).await.unwrap();
    assert!(quote.unwrap().at(now).unwrap() < &0.5);
    drop(chain);
    drop(audit);

    let audit = Audit::open(&path).await.unwrap();
    let chain = FallbackOdds::new(&audit, config, feed.clone());
    let still_fresh = now + Duration::hours(5);
    let (_, quote) = chain
        .live_quote(&b, still_fresh, Tier::Draft, &FixedClock(still_fresh))
        .await
        .unwrap();
    assert!(quote.unwrap().at(still_fresh).unwrap() < &0.5);
    assert_eq!(*feed.batches.lock().unwrap(), ["400014,51216"]);
    let expired = now + Duration::hours(6);
    let (_, quote) = chain
        .live_quote(&b, expired, Tier::Draft, &FixedClock(expired))
        .await
        .unwrap();
    assert!(quote.is_some());
    assert_eq!(
        *feed.batches.lock().unwrap(),
        ["400014,51216", "400014,51216"]
    );
}

#[derive(Default)]
struct FailingPinnacle {
    attempts: Mutex<usize>,
}

#[async_trait]
impl OddsTransport for FailingPinnacle {
    async fn get(&self, url: Url) -> Result<Vec<u8>, FetchError> {
        match url.host_str().unwrap() {
            "api.oddspapi.io" => {
                let mut attempts = self.attempts.lock().unwrap();
                *attempts += 1;
                if *attempts <= 3 {
                    Err(FetchError::Unavailable)
                } else {
                    Ok(PINNACLE.to_vec())
                }
            }
            "gamma-api.polymarket.com" => Ok(POLYMARKET.to_vec()),
            _ => panic!("unexpected odds source"),
        }
    }
}

#[tokio::test]
async fn three_failures_open_the_breaker_then_a_successful_probe_restores_pinnacle() {
    let dir = tempfile::tempdir().unwrap();
    let audit = Audit::open(&dir.path().join("audit.db")).await.unwrap();
    let feed = Arc::new(FailingPinnacle::default());
    let config = OddsConfig {
        oddspapi_key: Some("test-key".into()),
        oddspapi_tournaments: [(99, vec![400014])].into(),
        polymarket_tags: [(99, "pgl-major-singapore-2026".into())].into(),
        ..OddsConfig::default()
    };
    let chain = FallbackOdds::new(&audit, config, feed.clone());
    let now: Instant = "2026-11-25T17:20:00Z".parse().unwrap();
    let game = game(1, "Heroic", "FaZe");
    for offset in [0, 1, 2, 3, 16] {
        let at = now + Duration::minutes(offset);
        let (_, quote) = chain.live_quote(&game, at, Tier::Draft, &FixedClock(at)).await.unwrap();
        assert!((quote.unwrap().at(at).unwrap() - 0.545).abs() < 1e-12);
    }
    assert_eq!(*feed.attempts.lock().unwrap(), 3);
    let retry = now + Duration::minutes(17);
    let (_, quote) = chain
        .live_quote(&game, retry, Tier::Draft, &FixedClock(retry))
        .await
        .unwrap();
    assert!((quote.unwrap().at(retry).unwrap() - 0.537051381).abs() < 1e-9);
    assert_eq!(*feed.attempts.lock().unwrap(), 4);
}

#[tokio::test]
async fn oddspapi_keeps_headroom_in_the_monthly_budget_across_events_and_restarts() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("audit.db");
    let audit = Audit::open(&path).await.unwrap();
    let feed = Arc::new(PinnacleFeed::default());
    let config = OddsConfig {
        oddspapi_key: Some("test-key".into()),
        oddspapi_tournaments: (1..=201).map(|id| (id, vec![400014])).collect(),
        kalshi_enabled: false,
        ..OddsConfig::default()
    };
    let chain = FallbackOdds::new(&audit, config.clone(), feed.clone());
    let now: Instant = "2026-11-25T17:20:00Z".parse().unwrap();
    let mut game = game(1, "Heroic", "FaZe");
    for id in 1..=200 {
        game.event_id = id;
        let at = now + Duration::seconds(id);
        let (_, quote) = chain.live_quote(&game, at, Tier::PreLock, &FixedClock(at)).await.unwrap();
        assert!(quote.is_some());
    }
    drop(chain);
    drop(audit);

    let audit = Audit::open(&path).await.unwrap();
    let chain = FallbackOdds::new(&audit, config, feed.clone());
    game.event_id = 201;
    let at = now + Duration::seconds(201);
    let (_, quote) = chain.live_quote(&game, at, Tier::PreLock, &FixedClock(at)).await.unwrap();
    assert!(quote.is_none());
    assert_eq!(feed.batches.lock().unwrap().len(), 200);
    let next_month = "2026-12-01T00:00:00Z".parse().unwrap();
    game.scheduled_start = Some("2026-12-01T06:00:00Z".parse().unwrap());
    let (_, quote) = chain
        .live_quote(&game, next_month, Tier::PreLock, &FixedClock(next_month))
        .await
        .unwrap();
    assert!(
        quote.is_none(),
        "the saved November line is outside the Match window"
    );
    assert_eq!(feed.batches.lock().unwrap().len(), 201);
}

#[tokio::test]
async fn the_pre_lock_pass_refetches_a_batch_older_than_its_maximum_age() {
    let dir = tempfile::tempdir().unwrap();
    let audit = Audit::open(&dir.path().join("audit.db")).await.unwrap();
    let feed = Arc::new(PinnacleFeed::default());
    let config = OddsConfig {
        oddspapi_key: Some("test-key".into()),
        oddspapi_tournaments: [(99, vec![400014])].into(),
        kalshi_enabled: false,
        ..OddsConfig::default()
    };
    assert_eq!(config.prelock_max_age, Duration::minutes(10));
    let chain = FallbackOdds::new(&audit, config, feed.clone());
    let game = game(1, "Heroic", "FaZe");
    let start = game.scheduled_start.unwrap();
    let ask = |minutes_before: i64, tier: Tier| {
        let at = start - Duration::minutes(minutes_before);
        let chain = &chain;
        let game = &game;
        async move {
            let (_, quote) = chain
                .live_quote(game, at, tier, &FixedClock(at))
                .await
                .unwrap();
            assert!(quote.is_some());
        }
    };
    ask(36, Tier::Draft).await;
    // 11 minutes later: a Draft still reuses the batch (six-hour TTL), the
    // PreLock pass wants the closing line and fetches again.
    ask(25, Tier::Draft).await;
    assert_eq!(feed.batches.lock().unwrap().len(), 1);
    ask(25, Tier::PreLock).await;
    assert_eq!(feed.batches.lock().unwrap().len(), 2);
    // That batch is fresh enough for the rest of the window.
    for minutes_before in [24, 20, 16] {
        ask(minutes_before, Tier::PreLock).await;
    }
    assert_eq!(feed.batches.lock().unwrap().len(), 2);
}

/// OddsPapi and Polymarket, counting the OddsPapi requests.
#[derive(Default)]
struct BothFeeds {
    oddspapi: Mutex<usize>,
}
#[async_trait]
impl OddsTransport for BothFeeds {
    async fn get(&self, url: Url) -> Result<Vec<u8>, FetchError> {
        match url.host_str().unwrap() {
            "api.oddspapi.io" => {
                *self.oddspapi.lock().unwrap() += 1;
                Ok(PINNACLE.to_vec())
            }
            "gamma-api.polymarket.com" => Ok(POLYMARKET.to_vec()),
            _ => panic!("unexpected odds source"),
        }
    }
}

#[tokio::test]
async fn drafts_leave_the_last_fifty_oddspapi_requests_to_the_pre_lock_pass() {
    let dir = tempfile::tempdir().unwrap();
    let audit = Audit::open(&dir.path().join("audit.db")).await.unwrap();
    let feed = Arc::new(BothFeeds::default());
    let config = OddsConfig {
        oddspapi_key: Some("test-key".into()),
        oddspapi_tournaments: (1..=152).map(|id| (id, vec![400014])).collect(),
        polymarket_tags: (1..=152)
            .map(|id| (id, "pgl-major-singapore-2026".to_string()))
            .collect(),
        kalshi_enabled: false,
        ..OddsConfig::default()
    };
    let chain = FallbackOdds::new(&audit, config, feed.clone());
    let now: Instant = "2026-11-25T17:20:00Z".parse().unwrap();
    let mut game = game(1, "Heroic", "FaZe");
    for id in 1..=150 {
        game.event_id = id;
        let at = now + Duration::seconds(id);
        chain
            .live_quote(&game, at, Tier::Draft, &FixedClock(at))
            .await
            .unwrap();
    }
    assert_eq!(*feed.oddspapi.lock().unwrap(), 150);
    // The guard refuses the 151st Draft batch: the chain falls through to the
    // next Odds Source, which still prices the Match.
    game.event_id = 151;
    let at = now + Duration::seconds(151);
    let (_, quote) = chain
        .live_quote(&game, at, Tier::Draft, &FixedClock(at))
        .await
        .unwrap();
    assert!(quote.is_some(), "Polymarket answered");
    assert_eq!(*feed.oddspapi.lock().unwrap(), 150);
    // The PreLock pass still has its share.
    game.event_id = 152;
    let at = now + Duration::seconds(152);
    chain
        .live_quote(&game, at, Tier::PreLock, &FixedClock(at))
        .await
        .unwrap();
    assert_eq!(*feed.oddspapi.lock().unwrap(), 151);
}

#[tokio::test]
async fn the_raw_fixture_behind_a_quote_is_kept_once_per_content() {
    let dir = tempfile::tempdir().unwrap();
    let audit = Audit::open(&dir.path().join("audit.db")).await.unwrap();
    let feed = Arc::new(PinnacleFeed::default());
    let config = OddsConfig {
        oddspapi_key: Some("test-key".into()),
        oddspapi_tournaments: [(99, vec![400014])].into(),
        kalshi_enabled: false,
        ..OddsConfig::default()
    };
    let chain = FallbackOdds::new(&audit, config, feed.clone());
    let game = game(1, "Heroic", "FaZe");
    let start = game.scheduled_start.unwrap();
    // Two fetches of the same batch (the PreLock refresh), three quotes.
    for minutes_before in [60, 25, 24] {
        let at = start - Duration::minutes(minutes_before);
        let (_, quote) = chain
            .live_quote(&game, at, Tier::PreLock, &FixedClock(at))
            .await
            .unwrap();
        assert!(quote.is_some());
    }
    assert_eq!(feed.batches.lock().unwrap().len(), 2);
    let kept = audit.raw_records("odds:oddspapi").await.unwrap();
    assert_eq!(kept.len(), 1, "one record, not one per batch or quote");
    // It is the fixture object of that Match, as OddsPapi sent it, not the batch.
    let record: serde_json::Value = serde_json::from_slice(&kept[0].1).unwrap();
    let names = [&record["participant1Name"], &record["participant2Name"]];
    assert!(names.contains(&&serde_json::json!("Heroic")), "{names:?}");
    let batch: Vec<serde_json::Value> = serde_json::from_slice(PINNACLE).unwrap();
    assert!(batch.contains(&record));
    assert!(kept[0].1.len() < PINNACLE.len() / 4);
}
