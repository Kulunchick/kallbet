//! Antigravity subscription through `agy` print mode. `agy` has no switch that
//! removes tools, so a session-only `--settings` denies them (Deny wins over
//! Allow in its permissions). `-p` takes the prompt as its value, so the prompt
//! travels in argv (`--print=<prompt>`), not on stdin.
use super::{CliConfig, Finished, Invocation, ToolAccess, WEB_DOMAINS, classify, schema};
use crate::LlmError;
use serde_json::{Value, json};

const DENY_ALWAYS: &[&str] = &[
    "command(*)",
    "read_file(*)",
    "write_file(*)",
    "execute_url(*)",
    "mcp(*)",
    "search_web(*)",
];

pub fn command(config: &CliConfig, prompt: &str, schema: &Value, access: ToolAccess) -> Invocation {
    let mut deny: Vec<String> = DENY_ALWAYS.iter().map(|s| s.to_string()).collect();
    let mut allow: Vec<String> = vec![];
    match access {
        ToolAccess::None => deny.push("read_url(*)".into()),
        ToolAccess::WhitelistedWeb => {
            allow.extend(WEB_DOMAINS.iter().map(|d| format!("read_url({d})")));
        }
    }
    let settings = json!({
        "toolPermission": "strict",
        "enableTerminalSandbox": true,
        "permissions": {"allow": allow, "deny": deny}
    });
    let mut args = vec![
        format!("--print={prompt}"),
        "--output-format".into(),
        "json".into(),
        "--json-schema".into(),
        schema.to_string(),
        "--disable-slash-commands".into(),
        "--sandbox".into(),
        "--print-timeout".into(),
        format!("{}s", config.timeout_seconds),
        "--settings".into(),
        settings.to_string(),
    ];
    if let Some(model) = &config.model {
        args.extend(["--model".into(), model.clone()]);
    }
    Invocation {
        program: config.binary_or("agy"),
        args,
        stdin: None,
        files: vec![],
        // The outer limit leaves agy's own --print-timeout room to report first.
        timeout: config.timeout() + std::time::Duration::from_secs(15),
    }
}

/// Reads the `--output-format json` envelope and the `AGY_ERROR:` line on stderr.
pub fn parse(out: &Finished, schema: &Value) -> Result<Value, LlmError> {
    let envelope = serde_json::from_str::<Value>(out.stdout.trim()).unwrap_or(Value::Null);
    if envelope["status"] == "SUCCESS" {
        let answer = match &envelope["structured_output"] {
            Value::Null => envelope["response"].as_str().and_then(classify::json_text),
            v => Some(v.clone()),
        };
        return match answer {
            Some(v) if schema::valid(&v, schema) => Ok(v),
            _ => Err(LlmError::InvalidOutput),
        };
    }
    if envelope["finished_reason"] == "print_timeout" {
        return Err(LlmError::Timeout);
    }
    let structured = out
        .stderr
        .lines()
        .find_map(|l| l.strip_prefix("AGY_ERROR:"))
        .and_then(|j| serde_json::from_str::<Value>(j.trim()).ok());
    if let Some(e) = structured
        .as_ref()
        .and_then(|e| status(e["status"].as_str()?))
    {
        return Err(e);
    }
    let message = envelope["error"]
        .as_str()
        .map(str::to_owned)
        .unwrap_or_else(|| out.stderr.clone());
    if message.contains("invalid model selection") {
        return Err(LlmError::NotConfigured);
    }
    if let Some(e) = classify::text(&message) {
        return Err(e);
    }
    if message.to_lowercase().contains("schema") {
        return Err(LlmError::InvalidOutput);
    }
    Err(LlmError::Transient)
}

/// gRPC-style status names agy reports in `AGY_ERROR`.
fn status(name: &str) -> Option<LlmError> {
    Some(match name {
        "DEADLINE_EXCEEDED" => LlmError::Timeout,
        "RESOURCE_EXHAUSTED" => LlmError::QuotaExceeded,
        "UNAUTHENTICATED" | "PERMISSION_DENIED" => LlmError::Auth,
        "UNAVAILABLE" | "INTERNAL" | "ABORTED" => LlmError::Transient,
        _ => return None,
    })
}
