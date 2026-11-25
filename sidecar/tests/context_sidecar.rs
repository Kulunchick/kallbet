mod support;

use async_trait::async_trait;
use axum::{Router, extract::State, http::StatusCode, routing::get};
use chrono::Duration;
use kallbet::{
    domain::StageKind,
    poller,
    testing::{EventSnapshotBuilder, TEST_BOT_SECRET},
};
use kallbet_ai::adapters::context::{
    CompositeContextEnricher, ContextConfig, Pause, RssFeedConfig,
};
use kallbet_ai::{
    BotProfile, Config, ContextFact, DecisionPolicy, Dossier, Instant, LlmAdjustment, LlmError,
    LlmRuntime, Prediction, RolePrompt, Sidecar, Tier, adapters::baseline::VrsOnly,
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex};
use support::Harness;

const SINGAPORE_REV: &str =
    include_str!("fixtures/context/liquipedia/pgl-singapore-2026-rev-before-start.json");
const DUST2US_RSS: &str = include_str!("fixtures/context/rss/dust2us-news.xml");
const VRS_NOV: &str = include_str!("fixtures/vrs/standings_global_2026_11_02.md");

struct NoopPause;
#[async_trait]
impl Pause for NoopPause {
    async fn pause(&self, _: Duration) {}
}

#[derive(Default)]
struct RecordingLlm {
    dossiers: Mutex<Vec<Dossier>>,
}
#[async_trait]
impl LlmRuntime for RecordingLlm {
    async fn evaluate(
        &self,
        _: &BotProfile,
        dossier: &Dossier,
        _: Tier,
        _: &RolePrompt,
    ) -> Result<Option<Value>, LlmError> {
        self.dossiers.lock().unwrap().push(dossier.clone());
        Ok(None)
    }
}

struct ContextTestPolicy;

impl DecisionPolicy for ContextTestPolicy {
    fn wants_adjustment(&self, _: &BotProfile) -> bool {
        true
    }

    fn decide(
        &self,
        bot: &BotProfile,
        dossier: &Dossier,
        tier: Tier,
        adjustment: Option<LlmAdjustment>,
    ) -> kallbet_ai::Result<Option<Prediction>> {
        VrsOnly.decide(bot, dossier, tier, adjustment)
    }
}

#[derive(Default)]
struct StubUpstream {
    wiki_hits: Mutex<usize>,
    rss_hits: Mutex<usize>,
    wiki_response: Mutex<(StatusCode, String)>,
    rss_response: Mutex<(StatusCode, String)>,
}

async fn start_upstream(state: Arc<StubUpstream>) -> (String, tokio::task::JoinHandle<()>) {
    let app = Router::new()
        .route(
            "/api.php",
            get(|State(st): State<Arc<StubUpstream>>| async move {
                *st.wiki_hits.lock().unwrap() += 1;
                st.wiki_response.lock().unwrap().clone()
            }),
        )
        .route(
            "/rss/dust2us",
            get(|State(st): State<Arc<StubUpstream>>| async move {
                *st.rss_hits.lock().unwrap() += 1;
                st.rss_response.lock().unwrap().clone()
            }),
        )
        .with_state(state);
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
        .await
        .unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let handle = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (base, handle)
}

#[tokio::test]
async fn sidecar_tick_populates_deterministic_pre_match_context_and_prevents_liquipedia_bursts() {
    let upstream = Arc::new(StubUpstream {
        wiki_hits: Mutex::new(0),
        rss_hits: Mutex::new(0),
        wiki_response: Mutex::new((StatusCode::OK, SINGAPORE_REV.to_string())),
        rss_response: Mutex::new((StatusCode::OK, DUST2US_RSS.to_string())),
    });
    let (up_base, up_server) = start_upstream(upstream.clone()).await;

    let h = Harness::new().await;
    let now: Instant = "2026-11-24T00:00:00Z".parse().unwrap();
    h.app.clock.set(now);
    let start = now + Duration::hours(4);
    h.app.source.put_event(
        EventSnapshotBuilder::new(99, "PGL Major Singapore 2026")
            .stage(StageKind::Swiss, "Stage 1")
            .swiss_match(101, (1, 0, 0), "Vitality", "MOUZ", start)
            .swiss_match(102, (1, 0, 0), "FaZe", "Liquid", start)
            .build(now),
    );
    let human = h.app.user("Alice").await;
    let league = human
        .post("/api/leagues", &json!({"name": "Major", "event_id": 99}))
        .await
        .json();
    let invite = format!(
        "{}/join/{}",
        h.base,
        league["invite_token"].as_str().unwrap()
    );
    poller::tick(&h.app.state).await;

    let mut sidecar = h.sidecar(&invite).await;
    let vrs_ts: Instant = "2026-11-02T23:59:59.999999999Z".parse().unwrap();
    sidecar.audit.record_vrs(VRS_NOV, vrs_ts).await.unwrap();

    let mut pages = BTreeMap::new();
    pages.insert(99, "PGL/2026/Singapore".to_string());
    let ctx_cfg = ContextConfig {
        liquipedia_pages: pages,
        liquipedia_api_url: format!("{up_base}/api.php"),
        rss_feeds: vec![RssFeedConfig::new(format!("{up_base}/rss/dust2us"))],
        ..ContextConfig::default()
    };
    let recorder = Arc::new(RecordingLlm::default());
    sidecar.ports.policy = Arc::new(ContextTestPolicy);
    sidecar.ports.llm = recorder.clone();
    sidecar.ports.context = Arc::new(
        CompositeContextEnricher::from_audit(
            &ctx_cfg,
            &sidecar.audit,
            sidecar.ports.clock.clone(),
            Arc::new(NoopPause),
        )
        .await
        .unwrap(),
    );

    // Tick 1 evaluates both Matches in the same tick: only 1 Liquipedia request & 1 RSS request.
    assert_eq!(h.tick(&mut sidecar, now).await.unwrap(), 2);
    assert_eq!(*upstream.wiki_hits.lock().unwrap(), 1);
    assert_eq!(*upstream.rss_hits.lock().unwrap(), 1);

    let dossiers = recorder.dossiers.lock().unwrap().clone();
    assert_eq!(dossiers.len(), 2);

    let by_match: HashMap<i64, Dossier> = dossiers.into_iter().map(|d| (d.game.id, d)).collect();
    let vit_facts: Vec<ContextFact> = by_match[&101]
        .context
        .iter()
        .map(|v| serde_json::from_value(v.clone()).unwrap())
        .collect();
    assert!(vit_facts.iter().any(|f| matches!(
        f,
        ContextFact::LiquipediaRoster(r) if r.team == "Team Vitality" && r.has_stand_in
    )));
    assert!(vit_facts.iter().any(|f| matches!(
        f,
        ContextFact::VrsRosterDiff(d)
            if d.team == "Team Vitality"
                && d.changed
                && d.added_players == ["apEX"]
                && d.removed_players == ["jL"]
    )));

    let faze_facts: Vec<ContextFact> = by_match[&102]
        .context
        .iter()
        .map(|v| serde_json::from_value(v.clone()).unwrap())
        .collect();
    assert!(faze_facts.iter().any(|f| matches!(
        f,
        ContextFact::NewsItem(n)
            if n.url.contains("faze-liquid-to-each-take-home") && n.published_at < now
    )));

    // Tick 2 one minute later: unchanged pre-Match facts yield identical input_hash,
    // so zero extra LLM calls and zero extra upstream requests happen.
    assert_eq!(
        h.tick(&mut sidecar, now + Duration::minutes(1))
            .await
            .unwrap(),
        0
    );
    assert_eq!(recorder.dossiers.lock().unwrap().len(), 2);
    assert_eq!(*upstream.wiki_hits.lock().unwrap(), 1);
    assert_eq!(*upstream.rss_hits.lock().unwrap(), 1);

    up_server.abort();
}

#[tokio::test]
async fn context_enricher_failures_degrade_gracefully_without_aborting_match_predictions() {
    let upstream = Arc::new(StubUpstream {
        wiki_hits: Mutex::new(0),
        rss_hits: Mutex::new(0),
        wiki_response: Mutex::new((StatusCode::INTERNAL_SERVER_ERROR, "boom".into())),
        rss_response: Mutex::new((
            StatusCode::FORBIDDEN,
            "<html>Just a moment...</html>".into(),
        )),
    });
    let (up_base, up_server) = start_upstream(upstream).await;

    let h = Harness::new().await;
    let now: Instant = "2026-11-24T00:00:00Z".parse().unwrap();
    h.app.clock.set(now);
    let start = now + Duration::hours(4);
    h.app.source.put_event(
        EventSnapshotBuilder::new(99, "PGL Major Singapore 2026")
            .stage(StageKind::Swiss, "Stage 1")
            .swiss_match(101, (1, 0, 0), "MOUZ", "Vitality", start)
            .build(now),
    );
    let human = h.app.user("Alice").await;
    let league = human
        .post("/api/leagues", &json!({"name": "Major", "event_id": 99}))
        .await
        .json();
    let league_id = league["id"].as_i64().unwrap();
    let invite = format!(
        "{}/join/{}",
        h.base,
        league["invite_token"].as_str().unwrap()
    );
    poller::tick(&h.app.state).await;

    let mut sidecar = h.sidecar(&invite).await;
    let mut pages = BTreeMap::new();
    pages.insert(99, "PGL/2026/Singapore".to_string());
    let ctx_cfg = ContextConfig {
        liquipedia_pages: pages,
        liquipedia_api_url: format!("{up_base}/api.php"),
        rss_feeds: vec![RssFeedConfig::new(format!("{up_base}/rss/dust2us"))],
        ..ContextConfig::default()
    };
    sidecar.ports.context = Arc::new(
        CompositeContextEnricher::from_audit(
            &ctx_cfg,
            &sidecar.audit,
            sidecar.ports.clock.clone(),
            Arc::new(NoopPause),
        )
        .await
        .unwrap(),
    );

    // Despite both Liquipedia and RSS failing, the Match Prediction still succeeds.
    assert_eq!(h.tick(&mut sidecar, now).await.unwrap(), 1);
    assert_eq!(sidecar.audit.submissions().await.unwrap().len(), 1);

    // Human predicts and sees the bot's revealed pick on the Match card.
    assert!(
        human
            .put(
                &format!("/api/leagues/{league_id}/matches/101/prediction"),
                &json!({"team_id": 2})
            )
            .await
            .status
            .is_success()
    );
    h.app.clock.advance(Duration::minutes(5));
    let view = human
        .get(&format!("/api/leagues/{league_id}/matches"))
        .await
        .json();
    let revealed = view["stages"][0]["groups"][0]["matches"][0]["predictions"]["revealed"]
        .as_array()
        .unwrap();
    assert!(revealed.iter().any(|p| p["is_bot"] == true));

    up_server.abort();
}

#[tokio::test]
async fn sidecar_from_config_loads_context_settings_and_runs_tick() {
    let upstream = Arc::new(StubUpstream {
        wiki_hits: Mutex::new(0),
        rss_hits: Mutex::new(0),
        wiki_response: Mutex::new((StatusCode::OK, SINGAPORE_REV.to_string())),
        rss_response: Mutex::new((StatusCode::OK, DUST2US_RSS.to_string())),
    });
    let (up_base, up_server) = start_upstream(upstream.clone()).await;

    let h = Harness::new().await;
    let now = chrono::Utc::now();
    h.app.clock.set(now);
    h.app.source.put_event(
        EventSnapshotBuilder::new(99, "PGL Major Singapore 2026")
            .stage(StageKind::Swiss, "Stage 1")
            .swiss_match(101, (1, 0, 0), "MOUZ", "Vitality", now + Duration::hours(4))
            .build(now),
    );
    let human = h.app.user("Alice").await;
    let league = human
        .post("/api/leagues", &json!({"name": "Major", "event_id": 99}))
        .await
        .json();
    let invite = format!(
        "{}/join/{}",
        h.base,
        league["invite_token"].as_str().unwrap()
    );
    poller::tick(&h.app.state).await;

    let dir = tempfile::tempdir().unwrap();
    let audit_path = dir.path().join("sidecar.db");
    let cfg_path = dir.path().join("sidecar.json");
    let cfg_json = json!({
        "platform_url": h.base,
        "audit_file": audit_path,
        "vrs_files": [concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/vrs/standings_global_2026_11_02.md")],
        "context": {
            "liquipedia_pages": { "99": "PGL/2026/Singapore" },
            "liquipedia_api_url": format!("{up_base}/api.php"),
            "rss_feeds": [{ "url": format!("{up_base}/rss/dust2us") }]
        },
        "bots": [{
            "bot_key": "ctx-bot",
            "nickname": "ContextBot",
            "invite_links": [invite],
            "policy": "VrsOnly",
            "provider_chain": []
        }]
    });
    std::fs::write(&cfg_path, cfg_json.to_string()).unwrap();

    let config = Config::load(&cfg_path).unwrap();
    assert_eq!(
        config.context.liquipedia_pages.get(&99).map(String::as_str),
        Some("PGL/2026/Singapore")
    );
    let mut sidecar = Sidecar::from_config(config, TEST_BOT_SECRET.into())
        .await
        .unwrap();
    sidecar.ports.odds = h.scripts.clone();

    assert_eq!(sidecar.tick().await.unwrap(), 1);
    assert_eq!(*upstream.wiki_hits.lock().unwrap(), 1);
    assert_eq!(*upstream.rss_hits.lock().unwrap(), 1);

    up_server.abort();
}
