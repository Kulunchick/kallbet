//! The PandaScore adapter behind `ExternalSource`, against a local stand-in
//! for the REST API (saved fixtures served from 127.0.0.1). No network, no
//! real token.

use std::sync::{Arc, Mutex};

use chrono::{TimeZone, Utc};
use kallbet::domain::{DataSourceKind, EventId, StageKind};
use kallbet::ports::SourceError;
use kallbet::sources::fallback::{EventHint, ExternalSource};
use kallbet::sources::pandascore::PandaScoreSource;
use serde_json::{Value, json};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

fn fixture() -> Value {
    let path = format!(
        "{}/tests/fixtures/pandascore/constructed-swiss-major.json",
        env!("CARGO_MANIFEST_DIR")
    );
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

/// A local API: answers the first route whose pattern occurs in the request
/// line (percent-decoded brackets), 404 otherwise; records every request.
struct Api {
    url: String,
    requests: Arc<Mutex<Vec<String>>>,
}

impl Api {
    async fn serve(routes: Vec<(&'static str, u16, String)>) -> Api {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let requests = Arc::new(Mutex::new(Vec::new()));
        let log = requests.clone();
        tokio::spawn(async move {
            loop {
                let (mut socket, _) = listener.accept().await.unwrap();
                let routes = routes.clone();
                let log = log.clone();
                tokio::spawn(async move {
                    let mut received = Vec::new();
                    let mut chunk = [0u8; 4096];
                    while !received.windows(4).any(|w| w == b"\r\n\r\n") {
                        let n = socket.read(&mut chunk).await.unwrap_or(0);
                        if n == 0 {
                            break;
                        }
                        received.extend_from_slice(&chunk[..n]);
                    }
                    let request = String::from_utf8_lossy(&received)
                        .replace("%5B", "[")
                        .replace("%5D", "]")
                        .replace("%2C", ",");
                    let line = request.lines().next().unwrap_or("").to_string();
                    log.lock().unwrap().push(request.clone());
                    let (status, body) = routes
                        .iter()
                        .find(|(pattern, _, _)| line.contains(pattern))
                        .map(|(_, s, b)| (*s, b.clone()))
                        .unwrap_or((404, r#"{"error":"Not Found"}"#.to_string()));
                    let reply = format!(
                        "HTTP/1.1 {status} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                        body.len()
                    );
                    let _ = socket.write_all(reply.as_bytes()).await;
                });
            }
        });
        Api { url, requests }
    }

    fn requests(&self) -> Vec<String> {
        self.requests.lock().unwrap().clone()
    }
}

/// The Major as the API would serve it: series, tournaments, then matches
/// (`/matches` for a Swiss stage, `/brackets` for the bracket).
async fn major_api() -> Api {
    let doc = fixture();
    let t = doc["tournaments"].as_array().unwrap();
    let list = |i: usize| t[i]["matches"].to_string();
    let tournaments: Vec<Value> = t.iter().map(|x| x["tournament"].clone()).collect();
    Api::serve(vec![
        ("/csgo/series?filter[id]=50", 200, json!([doc["serie"].clone()]).to_string()),
        ("/series/50/tournaments", 200, json!(tournaments).to_string()),
        ("/tournaments/701/matches", 200, list(0)),
        ("/tournaments/702/brackets", 200, list(1)),
    ])
    .await
}

#[tokio::test]
async fn without_a_token_the_adapter_is_not_configured_and_never_calls_out() {
    let api = major_api().await;
    let source = PandaScoreSource::with_base_url(None, &api.url);

    assert_eq!(source.kind(), DataSourceKind::PandaScore);
    assert_eq!(source.list_events().await, Err(SourceError::NotConfigured));
    assert_eq!(source.fetch_event("50").await, Err(SourceError::NotConfigured));
    let hint = EventHint { id: EventId(1), name: "X".into(), starts_at: None, ends_at: None };
    assert_eq!(source.find_events(&hint).await, Err(SourceError::NotConfigured));
    assert!(api.requests().is_empty());
}

#[tokio::test]
async fn an_empty_token_counts_as_no_token() {
    let source = PandaScoreSource::new(Some(String::new()));
    assert_eq!(source.list_events().await, Err(SourceError::NotConfigured));
}

#[tokio::test]
async fn fetching_a_serie_reads_its_stages_and_every_stages_matches() {
    let api = major_api().await;
    let source = PandaScoreSource::with_base_url(Some("secret-token".into()), &api.url);

    let snapshot = source.fetch_event("50").await.unwrap();

    assert_eq!(snapshot.event.name, "Constructed Major 2026");
    let kinds: Vec<_> = snapshot.stages.iter().map(|s| s.kind).collect();
    assert_eq!(kinds, vec![StageKind::Swiss, StageKind::Playoffs]);
    assert_eq!(snapshot.matches.len(), 19);
}

#[tokio::test]
async fn the_token_goes_in_the_authorization_header_never_in_the_url() {
    let api = major_api().await;
    let source = PandaScoreSource::with_base_url(Some("secret-token".into()), &api.url);
    source.fetch_event("50").await.unwrap();

    let requests = api.requests();
    assert!(!requests.is_empty());
    for r in &requests {
        let line = r.lines().next().unwrap();
        assert!(!line.contains("secret-token"), "{line}");
        assert!(
            r.to_lowercase().contains("authorization: bearer secret-token"),
            "{r}"
        );
    }
}

#[tokio::test]
async fn a_serie_the_api_does_not_know_is_not_found() {
    let api = Api::serve(vec![("/csgo/series?filter[id]=", 200, "[]".into())]).await;
    let source = PandaScoreSource::with_base_url(Some("t".into()), &api.url);

    assert_eq!(source.fetch_event("999").await, Err(SourceError::NotFound));
    assert_eq!(source.fetch_event("not-a-number").await, Err(SourceError::NotFound));
}

#[tokio::test]
async fn a_refused_token_or_a_rate_limit_is_unavailable_without_leaking_the_token() {
    for status in [401, 403, 429, 500] {
        let api = Api::serve(vec![("/csgo/series", status, r#"{"error":"nope"}"#.into())]).await;
        let source = PandaScoreSource::with_base_url(Some("secret-token".into()), &api.url);

        let err = source.fetch_event("50").await.unwrap_err();
        let SourceError::Unavailable(message) = err else {
            panic!("status {status} gave {err:?}");
        };
        assert!(message.contains(&status.to_string()), "{message}");
        assert!(!message.contains("secret-token"), "{message}");
    }
}

#[tokio::test]
async fn an_unreachable_server_is_unavailable_without_leaking_the_token() {
    let source = PandaScoreSource::with_base_url(Some("secret-token".into()), "http://127.0.0.1:1");
    let err = source.fetch_event("50").await.unwrap_err();
    let SourceError::Unavailable(message) = err else { panic!("{err:?}") };
    assert!(!message.contains("secret-token"), "{message}");
}

#[tokio::test]
async fn a_reply_that_is_not_json_is_a_parse_error() {
    let api = Api::serve(vec![("/csgo/series", 200, "<html>maintenance</html>".into())]).await;
    let source = PandaScoreSource::with_base_url(Some("t".into()), &api.url);

    assert!(matches!(source.fetch_event("50").await, Err(SourceError::Parse(_))));
}

#[tokio::test]
async fn the_event_list_merges_running_and_upcoming_series_once_each() {
    let doc = fixture();
    let serie = doc["serie"].clone();
    let mut other = serie.clone();
    other["id"] = json!(51);
    other["full_name"] = json!("Another Event 2026");
    let api = Api::serve(vec![
        ("/csgo/series/running", 200, json!([serie.clone()]).to_string()),
        ("/csgo/series/upcoming", 200, json!([serie, other]).to_string()),
    ])
    .await;
    let source = PandaScoreSource::with_base_url(Some("t".into()), &api.url);

    let events = source.list_events().await.unwrap();

    let ids: Vec<_> = events.iter().map(|e| e.external_id.as_str()).collect();
    assert_eq!(ids, vec!["50", "51"]);
}

#[tokio::test]
async fn candidates_are_searched_by_the_dates_of_the_event() {
    let doc = fixture();
    let api = Api::serve(vec![("/csgo/series", 200, json!([doc["serie"].clone()]).to_string())]).await;
    let source = PandaScoreSource::with_base_url(Some("t".into()), &api.url);
    let hint = EventHint {
        id: EventId(8302),
        name: "Constructed Major 2026".into(),
        starts_at: Some(Utc.with_ymd_and_hms(2026, 12, 1, 0, 0, 0).unwrap()),
        ends_at: Some(Utc.with_ymd_and_hms(2026, 12, 7, 0, 0, 0).unwrap()),
    };

    let found = source.find_events(&hint).await.unwrap();

    assert_eq!(found.len(), 1);
    assert_eq!(found[0].external_id, "50");
    let request = &api.requests()[0];
    assert!(request.contains("range[begin_at]=2026-11-24"), "{request}");
}

#[tokio::test]
async fn an_event_never_stored_and_undated_cannot_be_searched_for() {
    let api = major_api().await;
    let source = PandaScoreSource::with_base_url(Some("t".into()), &api.url);
    let hint = EventHint { id: EventId(8302), name: String::new(), starts_at: None, ends_at: None };

    assert_eq!(source.find_events(&hint).await, Ok(vec![]));
    assert!(api.requests().is_empty());
}
