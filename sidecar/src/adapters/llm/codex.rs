//! Codex subscription through `codex exec`. The prompt goes on stdin (`-`), the
//! schema as a file (`--output-schema` takes a path), events come back as JSONL.
use super::{CliConfig, Finished, Invocation, ToolAccess, WEB_DOMAINS, classify, schema};
use crate::LlmError;
use serde_json::Value;

pub const SCHEMA_FILE: &str = "schema.json";

pub fn command(config: &CliConfig, prompt: &str, schema: &Value, access: ToolAccess) -> Invocation {
    let mut args: Vec<String> = [
        "exec",
        "--json",
        "--ephemeral",
        "--skip-git-repo-check",
        // No user config.toml (its MCP servers, tools and profiles); the login still applies.
        "--ignore-user-config",
        "--ignore-rules",
        "--sandbox",
        "read-only",
        "-c",
        "mcp_servers={}",
        "--output-schema",
        SCHEMA_FILE,
    ]
    .map(String::from)
    .into();
    match access {
        ToolAccess::None => args.extend(["-c".into(), r#"web_search="disabled""#.into()]),
        ToolAccess::WhitelistedWeb => {
            let domains: Vec<String> = WEB_DOMAINS.iter().map(|d| format!("\"{d}\"")).collect();
            args.extend([
                "-c".into(),
                r#"web_search="live""#.into(),
                "-c".into(),
                format!(
                    "tools.web_search={{allowed_domains=[{}]}}",
                    domains.join(",")
                ),
            ]);
        }
    }
    if let Some(model) = &config.model {
        args.extend(["--model".into(), model.clone()]);
    }
    args.push("-".into());
    Invocation {
        program: config.binary_or("codex"),
        args,
        stdin: Some(prompt.into()),
        files: vec![(SCHEMA_FILE.into(), schema.to_string())],
        timeout: config.timeout(),
    }
}

/// `codex exec` may exit 0 after a failed turn, so the event stream decides:
/// `turn.failed` is an error, otherwise the last `agent_message` is the answer.
pub fn parse(out: &Finished, schema: &Value) -> Result<Value, LlmError> {
    let mut answer = None;
    let mut failure = None;
    for line in out.stdout.lines() {
        let Ok(event) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        match event["type"].as_str() {
            Some("turn.failed") => {
                failure = Some(
                    event["error"]["message"]
                        .as_str()
                        .unwrap_or_default()
                        .to_owned(),
                )
            }
            Some("item.completed") if event["item"]["type"] == "agent_message" => {
                answer = event["item"]["text"].as_str().map(str::to_owned);
            }
            _ => {}
        }
    }
    if let Some(message) = failure {
        return Err(classify::text(&message).unwrap_or(LlmError::Transient));
    }
    match answer {
        Some(text) => match classify::json_text(&text) {
            Some(v) if schema::valid(&v, schema) => Ok(v),
            _ => Err(LlmError::InvalidOutput),
        },
        None if out.stderr.contains("output schema") => Err(LlmError::InvalidOutput),
        None => Err(classify::text(&out.stderr).unwrap_or(LlmError::Transient)),
    }
}
