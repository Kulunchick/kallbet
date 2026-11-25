use async_trait::async_trait;
use chrono::Duration;
use kallbet_ai::{
    Audit, Clock, Format, Instant, Match, OddsSource, Team,
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
    let (_, quote) = chain.live_quote(&a, now, &FixedClock(now)).await.unwrap();
    assert!(quote.unwrap().at(now).unwrap() < &0.5);
    drop(chain);
    drop(audit);

    let audit = Audit::open(&path).await.unwrap();
    let chain = FallbackOdds::new(&audit, config, feed.clone());
    let still_fresh = now + Duration::hours(5);
    let (_, quote) = chain
        .live_quote(&b, still_fresh, &FixedClock(still_fresh))
        .await
        .unwrap();
    assert!(quote.unwrap().at(still_fresh).unwrap() < &0.5);
    assert_eq!(*feed.batches.lock().unwrap(), ["400014,51216"]);
    let expired = now + Duration::hours(6);
    let (_, quote) = chain
        .live_quote(&b, expired, &FixedClock(expired))
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
        let (_, quote) = chain.live_quote(&game, at, &FixedClock(at)).await.unwrap();
        assert!((quote.unwrap().at(at).unwrap() - 0.545).abs() < 1e-12);
    }
    assert_eq!(*feed.attempts.lock().unwrap(), 3);
    let retry = now + Duration::minutes(17);
    let (_, quote) = chain
        .live_quote(&game, retry, &FixedClock(retry))
        .await
        .unwrap();
    assert!((quote.unwrap().at(retry).unwrap() - 0.537051381).abs() < 1e-9);
    assert_eq!(*feed.attempts.lock().unwrap(), 4);
}
