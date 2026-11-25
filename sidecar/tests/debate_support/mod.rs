//! The debate lifecycle scenario shared by `debate_sidecar.rs` (replays saved CLI
//! output) and `debate_live.rs` (runs the real CLIs and saves their output).
//!
//! Vitality meet MOUZ in a Bo3 with the market at 52/48 for Vitality and the
//! ratings leaning slightly to MOUZ (anchor: 50.6% Vitality). When the
//! Draft is made, Liquipedia is unreachable, so the dossier has no roster facts.
//! Before the pre-Hard-Lock window it answers with the saved revision of the
//! Major's page, in which Vitality field apEX as a stand-in for the benched jL.
#![allow(dead_code)]
use crate::support::Harness;
use async_trait::async_trait;
use axum::{Router, extract::State, http::StatusCode, routing::get};
use chrono::Duration;
use kallbet::{
    domain::StageKind,
    poller,
    ports::Clock as _,
    testing::{EventSnapshotBuilder, TestClient},
};
use kallbet_ai::{
    adapters::{
        context::{CompositeContextEnricher, ContextConfig, Pause},
        debate::{self, HybridAnchored},
        llm::*,
    },
    *,
};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

pub const SINGAPORE_REV: &str =
    include_str!("../fixtures/context/liquipedia/pgl-singapore-2026-rev-before-start.json");
pub const VRS_NOV: &str = include_str!("../fixtures/vrs/standings_global_2026_11_02.md");
pub const MATCH: i64 = 101;
pub const VITALITY: i64 = 1;
pub const MOUZ: i64 = 2;

struct NoPause;
#[async_trait]
impl Pause for NoPause {
    async fn pause(&self, _: Duration) {}
}

/// A stand-in for Liquipedia's API: down until `publish`, then the saved revision.
#[derive(Default)]
pub struct Wiki {
    up: Mutex<bool>,
}
impl Wiki {
    pub fn publish(&self) {
        *self.up.lock().unwrap() = true;
    }
}

/// The seat a CLI run was started for, read from the prompt it was given
/// (stdin for `claude` and `codex`, `--print=` for `agy`).
pub fn role_of(invocation: &Invocation) -> String {
    let prompt = invocation.stdin.clone().unwrap_or_else(|| {
        let print = invocation
            .args
            .iter()
            .find_map(|a| a.strip_prefix("--print="));
        print.unwrap_or_default().to_owned()
    });
    let rest = prompt.split("You are the `").nth(1).unwrap_or_default();
    rest.split('`').next().unwrap_or_default().to_owned()
}
pub fn program_of(invocation: &Invocation) -> String {
    let stem = invocation.program.file_stem().unwrap_or_default();
    stem.to_string_lossy().into_owned()
}

pub struct Lifecycle {
    pub h: Harness,
    pub sidecar: Sidecar,
    pub human: TestClient,
    pub league: i64,
    pub start: Instant,
    pub wiki: Arc<Wiki>,
    pub providers: Arc<Providers>,
    server: tokio::task::JoinHandle<()>,
}
impl Drop for Lifecycle {
    fn drop(&mut self) {
        self.server.abort();
    }
}
impl Lifecycle {
    /// The League, its one Match four hours ahead, and a `HybridAnchored` bot whose
    /// three subscriptions are the CLIs `runner` runs. `debate` is the `debate`
    /// section of the sidecar config.
    pub async fn new(runner: Arc<dyn ProcessRunner>, debate: Value) -> Self {
        let wiki = Arc::new(Wiki::default());
        let router = Router::new()
            .route(
                "/api.php",
                get(|State(wiki): State<Arc<Wiki>>| async move {
                    if *wiki.up.lock().unwrap() {
                        (StatusCode::OK, SINGAPORE_REV.to_string())
                    } else {
                        (StatusCode::SERVICE_UNAVAILABLE, String::new())
                    }
                }),
            )
            .with_state(wiki.clone());
        let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
            .await
            .unwrap();
        let wiki_url = format!("http://{}/api.php", listener.local_addr().unwrap());
        let server = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });

        let h = Harness::new().await;
        let now: Instant = "2026-11-24T00:00:00Z".parse().unwrap();
        h.app.clock.set(now);
        let start = now + Duration::hours(4);
        h.app.source.put_event(
            EventSnapshotBuilder::new(99, "PGL Major Singapore 2026")
                .stage(StageKind::Swiss, "Stage 1")
                .swiss_match(MATCH, (1, 0, 0), "Vitality", "MOUZ", start)
                .build(now),
        );
        let human = h.app.user("Alice").await;
        let league = human
            .post("/api/leagues", &json!({"name": "Friends", "event_id": 99}))
            .await
            .json();
        let invite = format!(
            "{}/join/{}",
            h.base,
            league["invite_token"].as_str().unwrap()
        );
        poller::tick(&h.app.state).await;

        // The market makes Vitality a 52/48 favourite; the ratings lean to MOUZ.
        // Blended 80/20 in logit space that anchors Vitality at 50.6%.
        let seen = now - Duration::hours(1);
        *h.scripts.market.lock().unwrap() = Some(AsOf::new(0.52, seen, now).unwrap());
        let signal = Signal {
            map: 0.466,
            series: 0.45,
        };
        *h.scripts.signal.lock().unwrap() = Some(AsOf::new(signal, seen, now).unwrap());

        let mut sidecar = h.sidecar(&invite).await;
        let vrs_day: Instant = "2026-11-02T23:59:59.999999999Z".parse().unwrap();
        sidecar.audit.record_vrs(VRS_NOV, vrs_day).await.unwrap();
        sidecar.bots[0].bot_key = "oracle".into();
        sidecar.bots[0].nickname = "Oracle".into();
        sidecar.bots[0].policy = Policy::HybridAnchored;
        sidecar.bots[0].provider_chain =
            vec!["claude".into(), "codex".into(), "antigravity".into()];
        sidecar.ports.policy = Arc::new(HybridAnchored);
        sidecar.ports.signal = h.scripts.clone();
        let context = ContextConfig {
            liquipedia_pages: [(99, "PGL/2026/Singapore".to_string())].into(),
            liquipedia_api_url: wiki_url,
            ..ContextConfig::default()
        };
        sidecar.ports.context = Arc::new(
            CompositeContextEnricher::from_audit(
                &context,
                &sidecar.audit,
                sidecar.ports.clock.clone(),
                Arc::new(NoPause),
            )
            .await
            .unwrap(),
        );
        let llm: LlmConfig = serde_json::from_value(json!({"providers": {
            "claude": {"kind": "claude"},
            "codex": {"kind": "codex"},
            "antigravity": {"kind": "antigravity"}
        }}))
        .unwrap();
        let providers = Arc::new(Providers::new(
            &llm,
            Mode::Live,
            sidecar.ports.clock.clone(),
            runner,
        ));
        let debate = serde_json::from_value(debate).unwrap();
        sidecar.ports.llm = debate::build(&debate, providers.clone(), &sidecar.audit).unwrap();
        Self {
            league: league["id"].as_i64().unwrap(),
            h,
            sidecar,
            human,
            start,
            wiki,
            providers,
            server,
        }
    }
    /// One sidecar tick at `minutes` before the Scheduled Start.
    pub async fn tick_at(&mut self, minutes: i64) -> Result<usize> {
        let at = self.start - Duration::minutes(minutes);
        self.h.tick(&mut self.sidecar, at).await
    }
    /// The Match card as Alice sees it.
    pub async fn card(&self) -> Value {
        let view = self
            .human
            .get(&format!("/api/leagues/{}/matches", self.league))
            .await
            .json();
        view["stages"][0]["groups"][0]["matches"][0].clone()
    }
    /// The bot's revealed Prediction on Alice's Match card.
    pub async fn bot_pick(&self) -> Value {
        let card = self.card().await;
        let revealed = card["predictions"]["revealed"].as_array().unwrap().clone();
        revealed.into_iter().find(|p| p["is_bot"] == true).unwrap()
    }
    /// Alice predicts; her final edit window passes and the Match is revealed early.
    pub async fn human_reveals(&self) {
        let path = format!("/api/leagues/{}/matches/{MATCH}/prediction", self.league);
        let put = self.human.put(&path, &json!({"team_id": VITALITY})).await;
        assert!(put.status.is_success());
        self.h.app.clock.advance(Duration::minutes(5));
        assert!(self.h.app.clock.now() < self.start - Duration::hours(3));
    }
}
