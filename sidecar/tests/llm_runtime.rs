//! The composed runtime as the pipeline sees it: `Providers` built from a config
//! file's `llm` section, each bot's `provider_chain` run as a `FallbackLlm`. The
//! CLIs answer through a scripted process runner with saved real output; the HTTP
//! adapter talks to a local stand-in server. The breaker clock is set by hand.
use async_trait::async_trait;
use axum::{Json, Router, extract::State, http::HeaderMap, http::StatusCode, routing::post};
use kallbet_ai::{adapters::llm::*, *};
use serde_json::{Value, json};
use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
};

fn fixture(name: &str) -> String {
    std::fs::read_to_string(format!(
        "{}/tests/fixtures/llm/{name}",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
}
fn done(code: i32, stdout: String) -> Result<Finished, RunError> {
    Ok(Finished {
        code: Some(code),
        stdout,
        stderr: String::new(),
    })
}
fn agy_quota() -> String {
    let mut v: Value = serde_json::from_str(&fixture("agy-timeout.json")).unwrap();
    v["error"] = json!("You have exhausted your quota on this model.");
    v["finished_reason"] = json!("error");
    v.to_string()
}
fn anthropic_answer() -> String {
    fixture("anthropic-message.json").replace("Hi! My name is Claude.", r#"{\"sum\":5}"#)
}

struct ManualClock(Mutex<Instant>);
impl Clock for ManualClock {
    fn now(&self) -> Instant {
        *self.0.lock().unwrap()
    }
}
impl ManualClock {
    fn advance(&self, minutes: i64) {
        *self.0.lock().unwrap() += chrono::Duration::minutes(minutes);
    }
}

/// Answers each program from its own queue and records every invocation.
#[derive(Default)]
struct ScriptedRunner {
    scripts: Mutex<HashMap<String, VecDeque<Result<Finished, RunError>>>>,
    calls: Mutex<Vec<Invocation>>,
}
impl ScriptedRunner {
    fn script(&self, program: &str, result: Result<Finished, RunError>) {
        self.scripts
            .lock()
            .unwrap()
            .entry(program.into())
            .or_default()
            .push_back(result);
    }
    fn programs(&self) -> Vec<String> {
        self.calls
            .lock()
            .unwrap()
            .iter()
            .map(|i| i.program.display().to_string())
            .collect()
    }
}
#[async_trait]
impl ProcessRunner for ScriptedRunner {
    async fn run(&self, invocation: &Invocation) -> Result<Finished, RunError> {
        self.calls.lock().unwrap().push(invocation.clone());
        let program = invocation.program.display().to_string();
        self.scripts
            .lock()
            .unwrap()
            .get_mut(&program)
            .and_then(VecDeque::pop_front)
            .unwrap_or(Err(RunError::NotFound))
    }
}

/// A local Anthropic- or OpenAI-compatible endpoint answering from a queue.
#[derive(Default)]
struct StandIn {
    replies: Mutex<VecDeque<(u16, String)>>,
    requests: Mutex<Vec<(HeaderMap, Value)>>,
}
async fn serve(stand_in: Arc<StandIn>) -> String {
    async fn answer(
        State(s): State<Arc<StandIn>>,
        headers: HeaderMap,
        Json(body): Json<Value>,
    ) -> (StatusCode, String) {
        s.requests.lock().unwrap().push((headers, body));
        let (status, body) = s
            .replies
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or((500, String::new()));
        (StatusCode::from_u16(status).unwrap(), body)
    }
    let app = Router::new()
        .route("/v1/messages", post(answer))
        .route("/v1/chat/completions", post(answer))
        .with_state(stand_in);
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
        .await
        .unwrap();
    let base = format!("http://{}/v1", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    base
}

struct World {
    providers: Providers,
    runner: Arc<ScriptedRunner>,
    clock: Arc<ManualClock>,
    http: Arc<StandIn>,
}
async fn world(mode: Mode) -> World {
    let http = Arc::new(StandIn::default());
    let base = serve(http.clone()).await;
    // The `llm` section exactly as an operator writes it in the sidecar config file.
    let config: Config = serde_json::from_value(json!({
        "platform_url": "http://localhost:8095", "audit_file": "x.db", "vrs_files": [], "bots": [],
        "llm": {
            "providers": {
                "claude": {"kind": "claude", "binary": "fake-claude", "timeout_seconds": 60},
                "codex": {"kind": "codex", "binary": "fake-codex"},
                "antigravity": {"kind": "antigravity", "binary": "fake-agy", "model": "gemini-3.8-flash-medium"},
                "proxy": {"kind": "http", "api": "anthropic", "base_url": base, "model": "claude-opus-5-5",
                          "api_key_env": "CARGO_PKG_NAME"},
                "no-endpoint": {"kind": "http", "api": "openai", "model": "gpt"},
                "no-key": {"kind": "http", "api": "openai", "base_url": "http://127.0.0.1:9/v1", "model": "gpt",
                           "api_key_env": "KALLBET_AI_TEST_UNSET_KEY"},
                "missing-binary": {"kind": "codex", "binary": "not-installed-codex"}
            },
            "web_search_bots": ["oracle"],
            "cooldown_minutes": 30
        }
    }))
    .unwrap();
    let clock = Arc::new(ManualClock(Mutex::new(
        "2026-11-25T12:00:00Z".parse().unwrap(),
    )));
    let runner = Arc::new(ScriptedRunner::default());
    let providers = Providers::new(&config.llm, mode, clock.clone(), runner.clone());
    World {
        providers,
        runner,
        clock,
        http,
    }
}
fn bot(key: &str, chain: &[&str]) -> BotProfile {
    BotProfile {
        bot_key: key.into(),
        nickname: key.into(),
        language: None,
        avatar_url: None,
        invite_links: vec![],
        policy: Policy::VrsOnly,
        provider_chain: chain.iter().map(|s| s.to_string()).collect(),
    }
}
fn dossier() -> Dossier {
    let team = |id, name: &str| Team {
        id,
        name: name.into(),
    };
    Dossier {
        game: Match {
            event_id: 1,
            id: 42,
            team_a: team(1, "Vitality"),
            team_b: team(2, "FaZe"),
            format: Format::Bo3,
            scheduled_start: Some("2026-11-25T15:00:00Z".parse().unwrap()),
        },
        signal: Some(Signal {
            map: 0.6,
            series: 0.65,
        }),
        market: Some(0.7),
        context: vec![json!({"kind": "stand_in", "team": "FaZe"})],
    }
}
fn sum_prompt() -> RolePrompt {
    RolePrompt {
        role: "test".into(),
        instructions: "Add the numbers.".into(),
        schema: serde_json::from_str(&fixture("tiny-schema.json")).unwrap(),
        attachments: vec![],
    }
}
const CHAIN: [&str; 4] = ["claude", "codex", "antigravity", "proxy"];

#[tokio::test]
async fn quota_and_timeout_fail_over_claude_codex_antigravity_to_http() {
    let w = world(Mode::Live).await;
    w.runner
        .script("fake-claude", done(1, fixture("claude-quota.json")));
    w.runner.script("fake-codex", Err(RunError::TimedOut));
    w.runner.script("fake-agy", done(1, agy_quota()));
    w.http
        .replies
        .lock()
        .unwrap()
        .push_back((200, anthropic_answer()));

    let chain = w.providers.chain(&CHAIN.map(String::from));
    let answer = chain
        .answer(
            &bot("oracle", &CHAIN),
            &dossier(),
            Tier::Draft,
            &sum_prompt(),
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(answer.provider, "proxy");
    assert_eq!(answer.value, json!({"sum": 5}));
    assert_eq!(
        w.runner.programs(),
        ["fake-claude", "fake-codex", "fake-agy"]
    );

    // What the stand-in received: a structured-output Messages request, key from the env var.
    let (headers, body) = w.http.requests.lock().unwrap()[0].clone();
    assert_eq!(headers["x-api-key"], "kallbet-ai");
    assert_eq!(headers["anthropic-version"], "2023-06-01");
    assert_eq!(body["model"], "claude-opus-5-5");
    assert_eq!(body["output_config"]["format"]["type"], "json_schema");
    assert_eq!(
        body["output_config"]["format"]["schema"],
        sum_prompt().schema
    );
    let text = body["messages"][0]["content"].as_str().unwrap();
    assert!(
        text.contains("Add the numbers.") && text.contains("Vitality") && text.contains("FaZe")
    );
}

#[tokio::test]
async fn the_bots_own_chain_through_the_port() {
    let w = world(Mode::Live).await;
    w.runner
        .script("fake-codex", done(0, fixture("codex-success.jsonl")));
    let runtime: &dyn LlmRuntime = &w.providers;
    let value = runtime
        .evaluate(
            &bot("oracle", &["codex", "claude"]),
            &dossier(),
            Tier::Draft,
            &sum_prompt(),
        )
        .await
        .unwrap();
    assert_eq!(value, Some(json!({"sum": 5})));
    assert_eq!(w.runner.programs(), ["fake-codex"]);
    let empty = runtime
        .evaluate(&bot("vrs", &[]), &dossier(), Tier::Draft, &sum_prompt())
        .await
        .unwrap();
    assert_eq!(empty, None, "a bot without a provider chain uses no LLM");
}

#[tokio::test]
async fn cooling_providers_are_skipped_then_half_open_recovers() {
    let w = world(Mode::Live).await;
    let b = bot("oracle", &CHAIN);
    w.runner
        .script("fake-claude", done(1, fixture("claude-quota.json")));
    w.runner
        .script("fake-codex", done(1, fixture("codex-quota.jsonl")));
    w.runner
        .script("fake-agy", done(3, fixture("agy-timeout.json")));
    w.http
        .replies
        .lock()
        .unwrap()
        .extend([(200, anthropic_answer()), (200, anthropic_answer())]);
    let chain = w.providers.chain(&CHAIN.map(String::from));
    chain
        .answer(&b, &dossier(), Tier::Draft, &sum_prompt())
        .await
        .unwrap();
    assert!(matches!(
        w.providers.breaker("claude"),
        Some(BreakerState::Open {
            cause: LlmError::QuotaExceeded,
            ..
        })
    ));
    assert!(matches!(
        w.providers.breaker("antigravity"),
        Some(BreakerState::Open {
            cause: LlmError::Timeout,
            ..
        })
    ));

    // Within the cool-down no subscription CLI is started again.
    w.clock.advance(29);
    let again = chain
        .answer(&b, &dossier(), Tier::Draft, &sum_prompt())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(again.provider, "proxy");
    assert_eq!(w.runner.calls.lock().unwrap().len(), 3);

    // After it, one trial goes to Claude, which has its quota back.
    w.clock.advance(1);
    w.runner
        .script("fake-claude", done(0, fixture("claude-success.json")));
    let recovered = chain
        .answer(&b, &dossier(), Tier::Draft, &sum_prompt())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(recovered.provider, "claude");
    assert_eq!(w.providers.breaker("claude"), Some(BreakerState::Closed));
    assert_eq!(
        w.runner.programs().last().map(String::as_str),
        Some("fake-claude")
    );
}

#[tokio::test]
async fn failed_half_open_trial_reopens_for_a_new_cool_down() {
    let w = world(Mode::Live).await;
    let b = bot("oracle", &["claude"]);
    let chain = w.providers.chain(&["claude".into()]);
    w.runner
        .script("fake-claude", done(1, fixture("claude-quota.json")));
    assert_eq!(
        chain
            .answer(&b, &dossier(), Tier::Draft, &sum_prompt())
            .await
            .unwrap_err(),
        LlmError::QuotaExceeded
    );
    w.clock.advance(30);
    w.runner
        .script("fake-claude", done(1, fixture("claude-quota.json")));
    assert_eq!(
        chain
            .answer(&b, &dossier(), Tier::Draft, &sum_prompt())
            .await
            .unwrap_err(),
        LlmError::QuotaExceeded
    );
    assert_eq!(w.runner.calls.lock().unwrap().len(), 2);
    w.clock.advance(29);
    assert_eq!(
        chain
            .answer(&b, &dossier(), Tier::Draft, &sum_prompt())
            .await
            .unwrap_err(),
        LlmError::QuotaExceeded,
        "still cooling down: the cause is reported without a run"
    );
    assert_eq!(w.runner.calls.lock().unwrap().len(), 2);
}

#[tokio::test]
async fn unconfigured_providers_are_skipped_cleanly() {
    let w = world(Mode::Live).await;
    let names = [
        "unknown-name",
        "missing-binary",
        "no-endpoint",
        "no-key",
        "codex",
    ];
    w.runner
        .script("fake-codex", done(0, fixture("codex-success.jsonl")));
    let chain = w.providers.chain(&names.map(String::from));
    let answer = chain
        .answer(
            &bot("oracle", &names),
            &dossier(),
            Tier::Draft,
            &sum_prompt(),
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(answer.provider, "codex");
    for name in ["missing-binary", "no-endpoint", "no-key"] {
        assert_eq!(
            w.providers.breaker(name),
            Some(BreakerState::Closed),
            "{name}"
        );
    }
    let nothing = w
        .providers
        .chain(&["unknown-name".into(), "no-endpoint".into()]);
    assert_eq!(
        nothing
            .answer(&bot("oracle", &[]), &dossier(), Tier::Draft, &sum_prompt())
            .await
            .unwrap_err(),
        LlmError::NotConfigured
    );
}

#[tokio::test]
async fn invalid_output_and_auth_fail_over_without_cooling_down() {
    let w = world(Mode::Live).await;
    w.runner
        .script("fake-claude", done(1, fixture("claude-signedout.json")));
    w.runner.script(
        "fake-codex",
        done(
            0,
            fixture("codex-success.jsonl").replace(r#"{\"sum\":5}"#, "five"),
        ),
    );
    w.http
        .replies
        .lock()
        .unwrap()
        .push_back((200, anthropic_answer()));
    let names = ["claude", "codex", "proxy"];
    let answer = w
        .providers
        .chain(&names.map(String::from))
        .answer(
            &bot("oracle", &names),
            &dossier(),
            Tier::Draft,
            &sum_prompt(),
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(answer.provider, "proxy");
    assert_eq!(w.providers.breaker("claude"), Some(BreakerState::Closed));
    assert_eq!(w.providers.breaker("codex"), Some(BreakerState::Closed));
}

#[tokio::test]
async fn http_quota_opens_the_proxy_breaker() {
    let w = world(Mode::Live).await;
    let limited = r#"{"type":"error","error":{"type":"rate_limit_error","message":"This request would exceed your account's rate limit. Please try again later."}}"#;
    w.http
        .replies
        .lock()
        .unwrap()
        .push_back((429, limited.into()));
    let err = w
        .providers
        .chain(&["proxy".into()])
        .answer(
            &bot("oracle", &["proxy"]),
            &dossier(),
            Tier::Draft,
            &sum_prompt(),
        )
        .await
        .unwrap_err();
    assert_eq!(err, LlmError::QuotaExceeded);
    assert!(matches!(
        w.providers.breaker("proxy"),
        Some(BreakerState::Open { .. })
    ));
}

/// Which `--tools` a claude run got.
async fn claude_tools(mode: Mode, bot_key: &str, tier: Tier) -> String {
    let w = world(mode).await;
    w.runner
        .script("fake-claude", done(0, fixture("claude-success.json")));
    w.providers
        .evaluate(&bot(bot_key, &["claude"]), &dossier(), tier, &sum_prompt())
        .await
        .unwrap();
    let calls = w.runner.calls.lock().unwrap();
    let args = &calls[0].args;
    let i = args.iter().position(|a| a == "--tools").unwrap();
    args[i + 1].clone()
}
#[tokio::test]
async fn web_access_only_for_an_opted_in_bot_in_a_live_pre_lock_run() {
    assert_eq!(
        claude_tools(Mode::Live, "oracle", Tier::PreLock).await,
        "WebFetch"
    );
    assert_eq!(claude_tools(Mode::Live, "oracle", Tier::Draft).await, "");
    assert_eq!(claude_tools(Mode::Live, "other", Tier::PreLock).await, "");
    assert_eq!(
        claude_tools(Mode::Backtest, "oracle", Tier::PreLock).await,
        "",
        "backtests never browse"
    );
}
#[tokio::test]
async fn the_prompt_tells_the_tool_rule() {
    let w = world(Mode::Live).await;
    w.runner
        .script("fake-codex", done(0, fixture("codex-success.jsonl")));
    w.runner
        .script("fake-codex", done(0, fixture("codex-success.jsonl")));
    for tier in [Tier::Draft, Tier::PreLock] {
        w.providers
            .evaluate(&bot("oracle", &["codex"]), &dossier(), tier, &sum_prompt())
            .await
            .unwrap();
    }
    let calls = w.runner.calls.lock().unwrap();
    assert!(calls[0].stdin.as_ref().unwrap().contains("Use no tools"));
    assert!(
        calls[1]
            .stdin
            .as_ref()
            .unwrap()
            .contains("hltv.org and liquipedia.net only")
    );
}
