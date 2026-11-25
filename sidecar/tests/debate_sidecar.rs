//! Seam 1: `HybridAnchored` and the Two-Tier Debate inside real sidecar ticks
//! against the KallBet TestApp. The three subscriptions answer through a runner
//! that replays the saved output of the real CLIs (`tests/fixtures/debate/`,
//! captured by `debate_live.rs` on this same scenario).
mod debate_support;
mod support;
use async_trait::async_trait;
use axum::{Json, Router, routing::post};
use chrono::Duration;
use debate_support::*;
use kallbet::{
    domain::StageKind,
    poller,
    testing::{EventSnapshotBuilder, TEST_BOT_SECRET},
};
use kallbet_ai::{
    adapters::llm::{BreakerState, Finished, Invocation, ProcessRunner, RunError},
    *,
};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};
use support::Harness;

fn saved(name: &str) -> Option<String> {
    let path = format!(
        "{}/tests/fixtures/debate/{name}",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(path).ok()
}

/// Answers each CLI run with what that CLI really printed for that seat.
#[derive(Default)]
struct Replay {
    /// `cli:seat` of every run, in the order they were started.
    started: Mutex<Vec<String>>,
}
impl Replay {
    fn started(&self) -> Vec<String> {
        self.started.lock().unwrap().clone()
    }
}
#[async_trait]
impl ProcessRunner for Replay {
    async fn run(&self, invocation: &Invocation) -> Result<Finished, RunError> {
        let (seat, cli) = (role_of(invocation), program_of(invocation));
        self.started.lock().unwrap().push(format!("{cli}:{seat}"));
        let stdout = saved(&format!("{seat}-{cli}.json"))
            .or_else(|| saved(&format!("{seat}-{cli}.jsonl")))
            .unwrap_or_else(|| panic!("no saved run of {cli} as {seat}"));
        let stderr = saved(&format!("{seat}-{cli}.stderr.txt")).unwrap_or_default();
        Ok(Finished {
            // `agy` left with 3 when its quota was exhausted; the others with 0.
            code: Some(if stderr.is_empty() { 0 } else { 3 }),
            stdout,
            stderr,
        })
    }
}

#[tokio::test]
async fn an_early_draft_then_the_debate_before_hard_lock_overturns_a_close_favourite() {
    let replay = Arc::new(Replay::default());
    let mut l = Lifecycle::new(replay.clone(), json!({})).await;
    let card = l.card().await;
    assert_eq!(card["team_a"]["name"], "Vitality");
    let (vitality, mouz) = (card["team_a"]["id"].clone(), card["team_b"]["id"].clone());

    // Four hours ahead: the Draft is one agent's call, on the first subscription.
    assert_eq!(l.tick_at(240).await.unwrap(), 1);
    assert_eq!(replay.started(), ["claude:predict"]);
    assert!(
        l.card().await["predictions"]["revealed"].is_null(),
        "nothing of the bot shows before Reveal"
    );

    // Alice predicts and her edit window passes: an early Reveal, three hours ahead.
    l.human_reveals().await;
    let draft = l.bot_pick().await;
    assert_eq!(draft["team_id"], vitality);
    assert_eq!(
        draft["preliminary"], true,
        "the bot may still change its pick"
    );
    assert_eq!(draft["rationale"]["preliminary"], true);
    assert_eq!(draft["rationale"]["override_market"], false);
    assert_eq!(draft["rationale"]["factors"], json!([]));
    // 0.8 logit(0.52) + 0.2 logit(0.45) = 0.0239, i.e. 50.6% for Vitality.
    let p = draft["rationale"]["p_win"].as_f64().unwrap();
    assert!((p - 0.506).abs() < 5e-4, "{p}");

    // Later ticks before the window find the pick in place and ask nobody.
    for minutes in [180, 60, 26] {
        assert_eq!(l.tick_at(minutes).await.unwrap(), 0);
    }
    assert_eq!(replay.started().len(), 1);

    // T-22m: the roster page now shows Vitality with a stand-in. The run is a
    // debate: Claude and Codex argue, Antigravity is out of quota as Arbiter and
    // its seat falls back to Claude.
    l.wiki.publish();
    assert_eq!(l.tick_at(22).await.unwrap(), 1);
    assert_eq!(
        replay.started()[1..],
        [
            "claude:advocate",
            "codex:devil",
            "agy:arbiter",
            "claude:arbiter"
        ]
    );
    let pick = l.bot_pick().await;
    assert_eq!(
        pick["team_id"], mouz,
        "the 52/48 favourite is overturned: {pick}"
    );
    let rationale = &pick["rationale"];
    assert_eq!(rationale["override_market"], true);
    assert_eq!(rationale["preliminary"], false, "the final analysis");
    assert_eq!(
        pick["preliminary"], true,
        "not yet Hard Locked for the server"
    );
    // The Arbiter's -1.5 points, backed by the roster revision, on 50.6%.
    let p = rationale["p_win"].as_f64().unwrap();
    assert!((p - 0.509).abs() < 5e-4, "{p}");
    assert_eq!(
        rationale["factors"],
        json!([{
            "kind": "stand_in",
            "evidence_url": "https://liquipedia.net/counterstrike/index.php?title=PGL/2026/Singapore&oldid=3473596"
        }])
    );
    // The summary reaches Members in all three interface languages.
    assert!(
        rationale["summary"]["en"]
            .as_str()
            .unwrap()
            .contains("apEX")
    );
    assert!(
        rationale["summary"]["ru"]
            .as_str()
            .unwrap()
            .contains("Vitality играют с apEX")
    );
    assert!(
        rationale["summary"]["uk"]
            .as_str()
            .unwrap()
            .contains("Vitality грають з apEX")
    );

    // Every tick until Hard Lock: no further call, no further write.
    for minutes in [21, 20, 19, 18, 17, 16] {
        assert_eq!(l.tick_at(minutes).await.unwrap(), 0, "T-{minutes}m");
    }
    assert_eq!(l.tick_at(15).await.unwrap(), 0);
    assert_eq!(replay.started().len(), 5, "no redundant LLM call");
    let locked = l.bot_pick().await;
    assert_eq!(locked["team_id"], mouz);
    assert_eq!(locked["preliminary"], false, "Hard Locked");

    // The audit keeps the Draft's one turn and all three seats of the debate.
    let turns = l.sidecar.audit.debate_turns().await.unwrap();
    let mut seats: Vec<_> = turns
        .iter()
        .map(|t| {
            (
                t.tier.as_str(),
                t.role.as_str(),
                t.provider.as_str(),
                t.status.as_str(),
            )
        })
        .collect();
    seats.sort();
    assert_eq!(
        seats,
        [
            ("Draft", "predict", "claude", "ok"),
            ("PreLock", "advocate", "claude", "ok"),
            ("PreLock", "arbiter", "claude", "ok"),
            ("PreLock", "devil", "codex", "ok"),
        ]
    );
    let devil = turns.iter().find(|t| t.role == "devil").unwrap();
    assert_eq!(devil.brief.as_ref().unwrap()["delta"], -2);
    assert!(matches!(
        l.providers.breaker("antigravity"),
        Some(BreakerState::Open {
            cause: LlmError::QuotaExceeded,
            ..
        })
    ));
    assert_eq!(l.sidecar.audit.submissions().await.unwrap().len(), 2);
}

/// A runner for scenarios whose LLM is scripted elsewhere: no CLI may start.
struct NoCli;
#[async_trait]
impl ProcessRunner for NoCli {
    async fn run(&self, invocation: &Invocation) -> Result<Finished, RunError> {
        panic!("unexpected CLI run: {}", program_of(invocation));
    }
}

#[tokio::test]
async fn a_shift_the_model_cannot_substantiate_never_reaches_the_match_card() {
    let mut l = Lifecycle::new(Arc::new(NoCli), json!({})).await;
    // A solid 70/30 favourite, and a model that wants it overturned.
    let seen = l.start - Duration::hours(5);
    let cutoff = l.start - Duration::hours(4);
    *l.h.scripts.market.lock().unwrap() = Some(AsOf::new(0.70, seen, cutoff).unwrap());
    let rating = Signal {
        map: 0.64,
        series: 0.70,
    };
    *l.h.scripts.signal.lock().unwrap() = Some(AsOf::new(rating, seen, cutoff).unwrap());
    l.sidecar.ports.llm = l.h.scripts.clone();
    let summary =
        json!({"en": "MOUZ will win this.", "ru": "MOUZ победят.", "uk": "MOUZ переможуть."});
    *l.h.scripts.adjustment.lock().unwrap() = Some(json!({
        "delta": -45,
        "factors": [{"kind": "stand_in", "evidence_url": ""}],
        "summary": summary
    }));
    let card = l.card().await;
    let vitality = card["team_a"]["id"].clone();

    assert_eq!(l.tick_at(240).await.unwrap(), 1);
    l.human_reveals().await;
    let pick = l.bot_pick().await;
    assert_eq!(pick["team_id"], vitality, "no evidence, no shift: {pick}");
    let rationale = &pick["rationale"];
    assert!((rationale["p_win"].as_f64().unwrap() - 0.70).abs() < 1e-9);
    assert_eq!(rationale["override_market"], false);
    assert_eq!(rationale["factors"], json!([]));
    assert_ne!(
        rationale["summary"]["en"], "MOUZ will win this.",
        "the text arguing for the discarded shift is not published"
    );

    // With evidence the same demand is cut to eight points: 70% becomes 62%.
    l.wiki.publish();
    let url = "https://www.hltv.org/news/46047/tyloo-bench-moseyuh";
    *l.h.scripts.adjustment.lock().unwrap() = Some(json!({
        "delta": -45,
        "factors": [{"kind": "stand_in", "evidence_url": url}],
        "summary": summary
    }));
    assert_eq!(l.tick_at(22).await.unwrap(), 1);
    let pick = l.bot_pick().await;
    assert_eq!(pick["team_id"], vitality, "{pick}");
    let rationale = &pick["rationale"];
    assert!((rationale["p_win"].as_f64().unwrap() - 0.62).abs() < 1e-9);
    assert_eq!(
        rationale["factors"],
        json!([{"kind": "stand_in", "evidence_url": url}])
    );
    assert_eq!(rationale["summary"]["en"], "MOUZ will win this.");
    let asked: Vec<_> =
        l.h.scripts
            .llm_calls
            .lock()
            .unwrap()
            .iter()
            .map(|t| t.name())
            .collect();
    assert_eq!(asked, ["Draft", "PreLock"]);
}

#[tokio::test]
async fn a_configured_hybrid_bot_debates_through_its_providers() {
    // An Anthropic-compatible endpoint (an OAuth proxy) that answers every seat.
    async fn answer(Json(body): Json<Value>) -> Json<Value> {
        let prompt = body["messages"][0]["content"].as_str().unwrap();
        let seat = prompt.split('`').nth(1).unwrap();
        let text = match seat {
            "advocate" | "devil" => {
                json!({"position": format!("{seat} speaking"), "claims": [], "delta": 0})
            }
            _ => json!({"delta": 0, "factors": [], "summary": {
                "en": format!("Decided by the {seat}."), "ru": "Решено.", "uk": "Вирішено."
            }}),
        };
        Json(json!({
            "type": "message", "role": "assistant", "stop_reason": "end_turn",
            "content": [{"type": "text", "text": text.to_string()}]
        }))
    }
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
        .await
        .unwrap();
    let proxy = format!("http://{}/v1", listener.local_addr().unwrap());
    let router = Router::new().route("/v1/messages", post(answer));
    let server = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });

    let h = Harness::new().await;
    let now = chrono::Utc::now();
    h.app.clock.set(now);
    h.app.source.put_event(
        EventSnapshotBuilder::new(99, "Major")
            .stage(StageKind::Swiss, "Stage 1")
            .swiss_match(42, (1, 0, 0), "MOUZ", "Vitality", now + Duration::hours(4))
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

    // The config file as an operator writes it.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("sidecar.json");
    let config = json!({
        "platform_url": h.base,
        "audit_file": dir.path().join("sidecar.db"),
        "vrs_files": [concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/vrs/standings_global_2026_11_02.md")],
        "llm": {"providers": {"proxy": {
            "kind": "http", "api": "anthropic", "base_url": proxy, "model": "claude-opus-5-5"
        }}},
        "debate": {"bots": {"oracle": {"mode": "always"}}},
        "bots": [{
            "bot_key": "oracle",
            "nickname": "Oracle",
            "invite_links": [invite],
            "policy": "HybridAnchored",
            "provider_chain": ["proxy"]
        }]
    });
    std::fs::write(&path, config.to_string()).unwrap();
    let mut sidecar = Sidecar::from_config(Config::load(&path).unwrap(), TEST_BOT_SECRET.into())
        .await
        .unwrap();
    sidecar.ports.odds = h.scripts.clone();

    assert_eq!(sidecar.tick().await.unwrap(), 1);
    let roles: Vec<_> = sidecar
        .audit
        .debate_turns()
        .await
        .unwrap()
        .into_iter()
        .map(|t| (t.role, t.provider))
        .collect();
    assert_eq!(roles.len(), 3, "`always`: the Draft is a debate");
    assert!(roles.iter().all(|(_, provider)| provider == "proxy"));
    assert_eq!(roles[2].0, "arbiter");
    let pick = &sidecar.audit.submissions().await.unwrap()[0];
    assert_eq!(pick.rationale["summary"]["en"], "Decided by the arbiter.");
    // No market line in this scenario: the anchor is the rating alone.
    assert_eq!(pick.rationale["override_market"], false);
    server.abort();
}
