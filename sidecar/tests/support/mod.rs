//! Shared Seam 1 harness. Replace any Ports member before calling tick.
#![allow(dead_code)]
pub mod platform;
use async_trait::async_trait;
use kallbet::{
    ports::Clock as ServerClock,
    testing::{FakeClock, TEST_BOT_SECRET, TestApp},
};
use kallbet_ai::{
    adapters::{baseline::VrsOnly, http::HttpPlatform},
    *,
};
use serde_json::Value;
use std::sync::{Arc, Mutex};
use tempfile::TempDir;
use tokio::task::JoinHandle;

struct SharedClock(Arc<FakeClock>);
impl Clock for SharedClock {
    fn now(&self) -> Instant {
        self.0.now()
    }
}

#[derive(Default)]
pub struct Scripts {
    pub market: Mutex<Option<AsOf<f64>>>,
    pub facts: Mutex<Vec<AsOf<Value>>>,
    pub adjustment: Mutex<Option<Value>>,
    pub llm_errors: Mutex<std::collections::VecDeque<LlmError>>,
    pub picks: Mutex<Option<Value>>,
    pub llm_calls: Mutex<Vec<Tier>>,
    pub stages: Mutex<Vec<(i64, Instant)>>,
    pub signal: Mutex<Option<AsOf<Signal>>>,
    pub prediction: Mutex<Option<Prediction>>,
}
#[async_trait]
impl OddsSource for Scripts {
    async fn quote(&self, _: &Match, _: Instant) -> Result<Option<AsOf<f64>>> {
        Ok(self.market.lock().unwrap().clone())
    }
}
#[async_trait]
impl ContextEnricher for Scripts {
    async fn facts(&self, _: &Match, _: Instant) -> Result<Vec<AsOf<Value>>> {
        Ok(self.facts.lock().unwrap().clone())
    }
}
#[async_trait]
impl SignalEstimator for Scripts {
    async fn signal(&self, _: &Match, _: Instant) -> Result<Option<AsOf<Signal>>> {
        Ok(self.signal.lock().unwrap().clone())
    }
}
#[async_trait]
impl LlmRuntime for Scripts {
    async fn evaluate(
        &self,
        _: &BotProfile,
        _: &Dossier,
        t: Tier,
        _: &RolePrompt,
    ) -> Result<Option<Value>, LlmError> {
        self.llm_calls.lock().unwrap().push(t);
        if let Some(error) = self.llm_errors.lock().unwrap().pop_front() {
            return Err(error);
        }
        Ok(self.adjustment.lock().unwrap().clone())
    }
}
impl DecisionPolicy for Scripts {
    fn decide(
        &self,
        _: &BotProfile,
        _: &Dossier,
        _: Tier,
        _: Option<LlmAdjustment>,
    ) -> Result<Option<Prediction>> {
        Ok(self.prediction.lock().unwrap().clone())
    }
}
#[async_trait]
impl PickemStep for Scripts {
    async fn tick(
        &self,
        _: &dyn LeaguePlatform,
        _: &str,
        _: &BotProfile,
        l: i64,
        n: Instant,
        _: &Audit,
    ) -> Result<Vec<StageFailure>> {
        self.stages.lock().unwrap().push((l, n));
        Ok(vec![])
    }
}

pub struct Harness {
    pub app: TestApp,
    pub base: String,
    pub scripts: Arc<Scripts>,
    dir: TempDir,
    server: JoinHandle<()>,
}
impl Harness {
    pub async fn new() -> Self {
        let app = TestApp::new().await;
        let router = kallbet::app::build_router(app.state.clone());
        let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
            .await
            .unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let server = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
        Self {
            app,
            base,
            dir: tempfile::tempdir().unwrap(),
            scripts: Arc::default(),
            server,
        }
    }
    pub async fn sidecar(&self, invite: &str) -> Sidecar {
        let signal = vrs::Standings::parse([include_str!(
            "../fixtures/vrs/standings_global_2026_11_02.md"
        )])
        .unwrap();
        let bot = BotProfile {
            bot_key: "vrs".into(),
            nickname: "Baseline".into(),
            avatar_url: None,
            language: None,
            provider_chain: vec![],
            policy: Policy::VrsOnly,
            invite_links: vec![invite.into()],
        };
        let ports = Ports {
            platform: Arc::new(HttpPlatform::new(&self.base, TEST_BOT_SECRET.into()).unwrap()),
            clock: Arc::new(SharedClock(self.app.clock.clone())),
            signal: Arc::new(signal),
            odds: self.scripts.clone(),
            context: self.scripts.clone(),
            policy: Arc::new(VrsOnly),
            llm: self.scripts.clone(),
            pickem: self.scripts.clone(),
        };
        Sidecar::new(
            vec![bot],
            Audit::open(&self.dir.path().join("audit.db"))
                .await
                .unwrap(),
            ports,
        )
    }
    /// The server's FakeClock as the sidecar's clock.
    pub fn clock(&self) -> Arc<dyn Clock> {
        Arc::new(SharedClock(self.app.clock.clone()))
    }
    pub async fn tick(&self, sidecar: &mut Sidecar, now: Instant) -> Result<usize> {
        self.app.clock.set(now);
        sidecar.tick().await
    }
}
impl Drop for Harness {
    fn drop(&mut self) {
        self.server.abort();
    }
}
impl PickemSolver for Scripts {
    fn solve(
        &self,
        stage: &AsOf<Value>,
        ratings: &[AsOf<PairwiseProbability>],
        cutoff: Instant,
    ) -> Result<Option<Value>> {
        stage.at(cutoff)?;
        for rating in ratings {
            rating.at(cutoff)?;
        }
        Ok(self.picks.lock().unwrap().clone())
    }
}
