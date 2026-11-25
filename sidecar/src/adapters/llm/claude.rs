//! Claude subscription through `claude -p` (Claude Code print mode).
//! `--bare` is not used: it reads only ANTHROPIC_API_KEY and never the
//! subscription's OAuth login (`claude --help`, research note 05).
use super::{CliConfig, Finished, Invocation, ToolAccess, WEB_DOMAINS, classify, schema};
use crate::LlmError;
use serde_json::{Value, json};

pub fn command(config: &CliConfig, prompt: &str, schema: &Value, access: ToolAccess) -> Invocation {
    let mut args: Vec<String> = [
        "-p",
        "--output-format",
        "json",
        "--json-schema",
        &schema.to_string(),
        "--no-session-persistence",
        "--strict-mcp-config",
        "--disable-slash-commands",
        // Only these settings apply: no user or local settings (their allow rules
        // could widen tool access); the run's directory is empty, so no project ones.
        "--setting-sources",
        "project",
        // Nobody answers a permission prompt: whatever is not allowed below is denied.
        "--permission-prompts",
        "none",
    ]
    .map(String::from)
    .into();
    match access {
        ToolAccess::None => args.extend(["--tools".into(), String::new()]),
        ToolAccess::WhitelistedWeb => {
            let allow: Vec<String> = WEB_DOMAINS
                .iter()
                .flat_map(|d| {
                    [
                        format!("WebFetch(domain:{d})"),
                        format!("WebFetch(domain:www.{d})"),
                    ]
                })
                .collect();
            args.extend(["--tools".into(), "WebFetch".into()]);
            args.push("--settings".into());
            args.push(json!({"permissions": {"allow": allow, "deny": ["WebSearch"]}}).to_string());
        }
    }
    if let Some(model) = &config.model {
        args.extend(["--model".into(), model.clone()]);
    }
    Invocation {
        program: config.binary_or("claude"),
        args,
        stdin: Some(prompt.into()),
        files: vec![],
        timeout: config.timeout(),
    }
}

/// Reads the `--output-format json` result envelope (stdout), or classifies
/// stderr when the CLI stopped before printing one.
pub fn parse(out: &Finished, schema: &Value) -> Result<Value, LlmError> {
    let Ok(envelope) = serde_json::from_str::<Value>(out.stdout.trim()) else {
        return Err(stderr_error(&out.stderr));
    };
    if envelope["is_error"] == Value::Bool(false) {
        let answer = match &envelope["structured_output"] {
            Value::Null => envelope["result"].as_str().and_then(classify::json_text),
            v => Some(v.clone()),
        };
        return match answer {
            Some(v) if schema::valid(&v, schema) => Ok(v),
            _ => Err(LlmError::InvalidOutput),
        };
    }
    if envelope["api_error"].as_str() == Some("usage_limit_reached") {
        return Err(LlmError::QuotaExceeded);
    }
    if let Some(e) = envelope["api_error_status"]
        .as_u64()
        .and_then(|s| classify::status(s as u16))
    {
        return Err(e);
    }
    let result = envelope["result"].as_str().unwrap_or_default();
    if let Some(e) = classify::text(result) {
        return Err(e);
    }
    let subtype = envelope["subtype"].as_str().unwrap_or_default();
    if subtype.contains("structured_output") || subtype == "error_max_turns" {
        return Err(LlmError::InvalidOutput);
    }
    Err(LlmError::Transient)
}

fn stderr_error(stderr: &str) -> LlmError {
    if stderr.contains("--json-schema") {
        return LlmError::InvalidOutput;
    }
    classify::text(stderr).unwrap_or(LlmError::Transient)
}
