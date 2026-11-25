//! Seam 2: what each provider printed or answered, as saved in tests/fixtures/llm/,
//! fed to the pure envelope parsers. Provenance of every fixture: tests/fixtures/llm/README.md.
use kallbet_ai::{
    LlmError,
    adapters::llm::{Api, Finished, antigravity, claude, codex, http},
};
use serde_json::{Value, json};

fn fixture(name: &str) -> String {
    std::fs::read_to_string(format!(
        "{}/tests/fixtures/llm/{name}",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
}
fn finished(code: i32, stdout: &str, stderr: &str) -> Finished {
    Finished {
        code: Some(code),
        stdout: stdout.into(),
        stderr: stderr.into(),
    }
}
fn tiny() -> Value {
    serde_json::from_str(&fixture("tiny-schema.json")).unwrap()
}

#[test]
fn claude_success_reads_structured_output() {
    let out = finished(0, &fixture("claude-success.json"), "");
    assert_eq!(claude::parse(&out, &tiny()), Ok(json!({"sum": 5})));
}
#[test]
fn claude_weekly_limit_is_quota() {
    let out = finished(1, &fixture("claude-quota.json"), "");
    assert_eq!(claude::parse(&out, &tiny()), Err(LlmError::QuotaExceeded));
}
#[test]
fn claude_session_limit_text_is_quota_without_status() {
    // The same real envelope as the CLI prints for its other limit windows.
    let mut v: Value = serde_json::from_str(&fixture("claude-quota.json")).unwrap();
    v["api_error_status"] = Value::Null;
    v["api_error"] = Value::Null;
    v["result"] = json!("You've hit your session limit · resets 7pm (UTC)");
    let out = finished(1, &v.to_string(), "");
    assert_eq!(claude::parse(&out, &tiny()), Err(LlmError::QuotaExceeded));
}
#[test]
fn claude_signed_out_is_auth() {
    let out = finished(1, &fixture("claude-signedout.json"), "");
    assert_eq!(claude::parse(&out, &tiny()), Err(LlmError::Auth));
}
#[test]
fn claude_rejected_schema_is_invalid_output() {
    let out = finished(1, "", &fixture("claude-badschema.stderr.txt"));
    assert_eq!(claude::parse(&out, &tiny()), Err(LlmError::InvalidOutput));
}
#[test]
fn claude_answer_not_matching_the_schema_is_invalid_output() {
    let out = finished(0, &fixture("claude-success.json"), "");
    let schema = json!({"type":"object","properties":{"sum":{"type":"string"}},"required":["sum"]});
    assert_eq!(claude::parse(&out, &schema), Err(LlmError::InvalidOutput));
}

#[test]
fn codex_success_reads_the_last_agent_message() {
    let out = finished(
        0,
        &fixture("codex-success.jsonl"),
        "Reading additional input from stdin...\n",
    );
    assert_eq!(codex::parse(&out, &tiny()), Ok(json!({"sum": 5})));
}
#[test]
fn codex_usage_limit_is_quota() {
    let out = finished(1, &fixture("codex-quota.jsonl"), "");
    assert_eq!(codex::parse(&out, &tiny()), Err(LlmError::QuotaExceeded));
}
#[test]
fn codex_signed_out_is_auth() {
    let out = finished(1, &fixture("codex-signedout.jsonl"), "");
    assert_eq!(codex::parse(&out, &tiny()), Err(LlmError::Auth));
}
#[test]
fn codex_unreadable_schema_is_invalid_output() {
    let out = finished(1, "", &fixture("codex-badschema.stderr.txt"));
    assert_eq!(codex::parse(&out, &tiny()), Err(LlmError::InvalidOutput));
}
#[test]
fn codex_answer_that_is_not_json_is_invalid_output() {
    let out = finished(
        0,
        &fixture("codex-success.jsonl").replace(r#"{\"sum\":5}"#, "five"),
        "",
    );
    assert_eq!(codex::parse(&out, &tiny()), Err(LlmError::InvalidOutput));
}

#[test]
fn agy_success_reads_structured_output() {
    let out = finished(0, &fixture("agy-success.json"), "");
    assert_eq!(antigravity::parse(&out, &tiny()), Ok(json!({"sum": 5})));
}
#[test]
fn agy_print_timeout_is_timeout() {
    let out = finished(
        3,
        &fixture("agy-timeout.json"),
        &fixture("agy-timeout.stderr.txt"),
    );
    assert_eq!(antigravity::parse(&out, &tiny()), Err(LlmError::Timeout));
}
#[test]
fn agy_exhausted_quota_is_quota() {
    // Derived from the real error envelope; the message is the one agy.exe carries
    // for an exhausted model quota (see the fixtures README).
    let mut v: Value = serde_json::from_str(&fixture("agy-timeout.json")).unwrap();
    v["error"] = json!("You have exhausted your quota on this model.");
    v["finished_reason"] = json!("error");
    let out = finished(1, &v.to_string(), "");
    assert_eq!(
        antigravity::parse(&out, &tiny()),
        Err(LlmError::QuotaExceeded)
    );
}
#[test]
fn agy_unknown_model_is_not_configured() {
    let out = finished(1, &fixture("agy-badmodel.json"), "");
    assert_eq!(
        antigravity::parse(&out, &tiny()),
        Err(LlmError::NotConfigured)
    );
}

#[test]
fn http_rate_limit_and_auth_statuses() {
    let unauthorized = fixture("anthropic-401.json");
    assert_eq!(
        http::parse(Api::Anthropic, 401, &unauthorized, &tiny()),
        Err(LlmError::Auth)
    );
    let unauthorized = fixture("openai-401.json");
    assert_eq!(
        http::parse(Api::Openai, 401, &unauthorized, &tiny()),
        Err(LlmError::Auth)
    );
    let limited = r#"{"type":"error","error":{"type":"rate_limit_error","message":"This request would exceed your account's rate limit. Please try again later."}}"#;
    assert_eq!(
        http::parse(Api::Anthropic, 429, limited, &tiny()),
        Err(LlmError::QuotaExceeded)
    );
    assert_eq!(
        http::parse(Api::Openai, 503, "", &tiny()),
        Err(LlmError::Transient)
    );
    assert_eq!(
        http::parse(Api::Openai, 404, "", &tiny()),
        Err(LlmError::NotConfigured)
    );
    assert_eq!(
        http::parse(Api::Openai, 400, "", &tiny()),
        Err(LlmError::InvalidOutput)
    );
}
#[test]
fn http_reads_structured_text_of_both_apis() {
    let message =
        fixture("anthropic-message.json").replace("Hi! My name is Claude.", r#"{\"sum\":5}"#);
    assert_eq!(
        http::parse(Api::Anthropic, 200, &message, &tiny()),
        Ok(json!({"sum": 5}))
    );
    let completion = fixture("openai-chat-completion.json")
        .replace("Hello! How can I assist you today?", r#"{\"sum\":5}"#);
    assert_eq!(
        http::parse(Api::Openai, 200, &completion, &tiny()),
        Ok(json!({"sum": 5}))
    );
}
#[test]
fn http_plain_text_answer_is_invalid_output() {
    let message = fixture("anthropic-message.json");
    assert_eq!(
        http::parse(Api::Anthropic, 200, &message, &tiny()),
        Err(LlmError::InvalidOutput)
    );
    let completion = fixture("openai-chat-completion.json");
    assert_eq!(
        http::parse(Api::Openai, 200, &completion, &tiny()),
        Err(LlmError::InvalidOutput)
    );
}
