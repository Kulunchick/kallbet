//! Seam 1: the LLM runtimes inside a real sidecar tick against the KallBet
//! TestApp. The bot's provider chain is Claude then Codex; the CLIs answer through
//! a scripted runner with saved real output, and the breakers share the FakeClock.
mod support;
use async_trait::async_trait;
use chrono::Duration;
use kallbet::{domain::StageKind, poller, ports::Clock as _, testing::EventSnapshotBuilder};
use kallbet_ai::{adapters::llm::*, *};
use serde_json::{Value, json};
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};
use support::Harness;

fn fixture(name: &str) -> String {
    std::fs::read_to_string(format!(
        "{}/tests/fixtures/llm/{name}",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
}
/// The real codex success stream with an adjustment as its agent message.
fn codex_adjustment() -> String {
    let adjustment = json!({"delta": 0, "factors": [], "summary": {"en": "No verified news.", "ru": "Новостей нет.", "uk": "Новин немає."}});
    fixture("codex-success.jsonl")
        .lines()
        .map(|line| {
            let mut event: Value = serde_json::from_str(line).unwrap();
            if event["type"] == "item.completed" {
                event["item"]["text"] = json!(adjustment.to_string());
            }
            event.to_string()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

struct Runner {
    replies: Mutex<VecDeque<(String, Finished)>>,
    started: Mutex<Vec<String>>,
}
#[async_trait]
impl ProcessRunner for Runner {
    async fn run(&self, i: &Invocation) -> Result<Finished, RunError> {
        let program = i.program.display().to_string();
        self.started.lock().unwrap().push(program.clone());
        let mut replies = self.replies.lock().unwrap();
        let at = replies
            .iter()
            .position(|(p, _)| *p == program)
            .expect("scripted reply");
        Ok(replies.remove(at).unwrap().1)
    }
}
struct ServerClock(Arc<kallbet::testing::FakeClock>);
impl Clock for ServerClock {
    fn now(&self) -> Instant {
        self.0.now()
    }
}

#[tokio::test]
async fn claude_quota_fails_over_to_codex_and_cools_down_across_matches() {
    let h = Harness::new().await;
    let now = "2026-11-24T00:00:00Z".parse().unwrap();
    h.app.clock.set(now);
    let start = now + Duration::hours(4);
    h.app.source.put_event(
        EventSnapshotBuilder::new(99, "Major")
            .stage(StageKind::Swiss, "Stage 1")
            .swiss_match(42, (1, 0, 0), "MOUZ", "Vitality", start)
            .swiss_match(43, (1, 0, 1), "FaZe", "Spirit", start)
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

    let finished = |code, stdout: String| Finished {
        code: Some(code),
        stdout,
        stderr: String::new(),
    };
    let runner = Arc::new(Runner {
        replies: Mutex::new(VecDeque::from([
            ("claude".into(), finished(1, fixture("claude-quota.json"))),
            ("codex".into(), finished(0, codex_adjustment())),
            ("codex".into(), finished(0, codex_adjustment())),
        ])),
        started: Mutex::default(),
    });
    let config: LlmConfig = serde_json::from_value(json!({
        "providers": {"claude": {"kind": "claude"}, "codex": {"kind": "codex"}}
    }))
    .unwrap();
    let mut sidecar = h.sidecar(&invite).await;
    sidecar.bots[0].provider_chain = vec!["claude".into(), "codex".into()];
    sidecar.ports.llm = Arc::new(Providers::new(
        &config,
        Mode::Live,
        Arc::new(ServerClock(h.app.clock.clone())),
        runner.clone(),
    ));

    assert_eq!(sidecar.tick().await.unwrap(), 2, "both Matches predicted");
    // Claude's weekly limit is seen once; for the second Match it is cooling down.
    assert_eq!(
        *runner.started.lock().unwrap(),
        ["claude", "codex", "codex"]
    );
    assert_eq!(sidecar.audit.submissions().await.unwrap().len(), 2);
}
