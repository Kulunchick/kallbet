//! The pure command builders: what each CLI would be started with.
use kallbet_ai::adapters::llm::{CliConfig, Invocation, ToolAccess, antigravity, claude, codex};
use serde_json::{Value, json};
use std::time::Duration;

fn config() -> CliConfig {
    serde_json::from_value(json!({"model": "m1", "timeout_seconds": 120})).unwrap()
}
fn schema() -> Value {
    json!({"type":"object","properties":{"sum":{"type":"integer"}},"required":["sum"],"additionalProperties":false})
}
/// The value following `flag`, for each time it appears.
fn values<'a>(i: &'a Invocation, flag: &str) -> Vec<&'a str> {
    i.args
        .windows(2)
        .filter(|w| w[0] == flag)
        .map(|w| w[1].as_str())
        .collect()
}
fn has(i: &Invocation, arg: &str) -> bool {
    i.args.iter().any(|a| a == arg)
}

#[test]
fn claude_print_mode_with_schema_and_no_tools() {
    let i = claude::command(&config(), "PROMPT", &schema(), ToolAccess::None);
    assert_eq!(i.program.to_str(), Some("claude"));
    assert!(has(&i, "-p"));
    assert!(
        !has(&i, "--bare"),
        "--bare would drop the subscription login"
    );
    assert_eq!(values(&i, "--output-format"), ["json"]);
    assert_eq!(
        serde_json::from_str::<Value>(values(&i, "--json-schema")[0]).unwrap(),
        schema()
    );
    assert_eq!(values(&i, "--tools"), [""]);
    assert_eq!(values(&i, "--permission-prompts"), ["none"]);
    assert_eq!(values(&i, "--setting-sources"), ["project"]);
    for flag in [
        "--strict-mcp-config",
        "--disable-slash-commands",
        "--no-session-persistence",
    ] {
        assert!(has(&i, flag), "{flag}");
    }
    assert_eq!(values(&i, "--model"), ["m1"]);
    assert_eq!(i.stdin.as_deref(), Some("PROMPT"));
    assert!(!i.args.iter().any(|a| a.contains("PROMPT")));
    assert_eq!(i.timeout, Duration::from_secs(120));
    assert!(!has(&i, "--dangerously-skip-permissions"));
}
#[test]
fn claude_whitelisted_web_reads_only_two_sites() {
    let i = claude::command(&config(), "P", &schema(), ToolAccess::WhitelistedWeb);
    assert_eq!(values(&i, "--tools"), ["WebFetch"]);
    let settings: Value = serde_json::from_str(values(&i, "--settings")[0]).unwrap();
    let allow: Vec<&str> = settings["permissions"]["allow"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert_eq!(
        allow,
        [
            "WebFetch(domain:hltv.org)",
            "WebFetch(domain:www.hltv.org)",
            "WebFetch(domain:liquipedia.net)",
            "WebFetch(domain:www.liquipedia.net)"
        ]
    );
    assert_eq!(settings["permissions"]["deny"], json!(["WebSearch"]));
    assert_eq!(values(&i, "--permission-prompts"), ["none"]);
}

#[test]
fn codex_exec_read_only_with_schema_file_and_search_off() {
    let i = codex::command(&config(), "PROMPT", &schema(), ToolAccess::None);
    assert_eq!(i.program.to_str(), Some("codex"));
    assert_eq!(i.args[0], "exec");
    for flag in [
        "--json",
        "--ephemeral",
        "--skip-git-repo-check",
        "--ignore-user-config",
        "--ignore-rules",
    ] {
        assert!(has(&i, flag), "{flag}");
    }
    assert_eq!(values(&i, "--sandbox"), ["read-only"]);
    let file = values(&i, "--output-schema")[0];
    let (_, contents) = i.files.iter().find(|(name, _)| name == file).unwrap();
    assert_eq!(serde_json::from_str::<Value>(contents).unwrap(), schema());
    assert_eq!(
        values(&i, "-c"),
        [r#"mcp_servers={}"#, r#"web_search="disabled""#]
    );
    assert_eq!(
        i.args.last().map(String::as_str),
        Some("-"),
        "prompt from stdin"
    );
    assert_eq!(i.stdin.as_deref(), Some("PROMPT"));
    assert!(!has(&i, "--dangerously-bypass-approvals-and-sandbox"));
}
#[test]
fn codex_whitelisted_web_search() {
    let i = codex::command(&config(), "P", &schema(), ToolAccess::WhitelistedWeb);
    assert_eq!(
        values(&i, "-c"),
        [
            r#"mcp_servers={}"#,
            r#"web_search="live""#,
            r#"tools.web_search={allowed_domains=["hltv.org","liquipedia.net"]}"#
        ]
    );
    assert_eq!(values(&i, "--sandbox"), ["read-only"]);
}

#[test]
fn agy_print_mode_denies_every_tool() {
    let i = antigravity::command(&config(), "PROMPT", &schema(), ToolAccess::None);
    assert_eq!(i.program.to_str(), Some("agy"));
    assert_eq!(i.args[0], "--print=PROMPT");
    assert_eq!(i.stdin, None);
    assert_eq!(values(&i, "--output-format"), ["json"]);
    assert_eq!(
        serde_json::from_str::<Value>(values(&i, "--json-schema")[0]).unwrap(),
        schema()
    );
    assert!(has(&i, "--disable-slash-commands") && has(&i, "--sandbox"));
    assert_eq!(values(&i, "--print-timeout"), ["120s"]);
    assert!(
        i.timeout > Duration::from_secs(120),
        "agy reports its own timeout first"
    );
    let settings: Value = serde_json::from_str(values(&i, "--settings")[0]).unwrap();
    assert_eq!(settings["toolPermission"], "strict");
    let deny = settings["permissions"]["deny"].as_array().unwrap();
    for rule in [
        "command(*)",
        "read_file(*)",
        "write_file(*)",
        "read_url(*)",
        "execute_url(*)",
        "mcp(*)",
        "search_web(*)",
    ] {
        assert!(deny.contains(&json!(rule)), "{rule}");
    }
    assert_eq!(settings["permissions"]["allow"], json!([]));
    assert!(!has(&i, "--dangerously-skip-permissions"));
}
#[test]
fn agy_whitelisted_web_reads_only_two_sites() {
    let i = antigravity::command(&config(), "P", &schema(), ToolAccess::WhitelistedWeb);
    let settings: Value = serde_json::from_str(values(&i, "--settings")[0]).unwrap();
    assert_eq!(
        settings["permissions"]["allow"],
        json!(["read_url(hltv.org)", "read_url(liquipedia.net)"])
    );
    let deny = settings["permissions"]["deny"].as_array().unwrap();
    assert!(!deny.contains(&json!("read_url(*)")));
    assert!(deny.contains(&json!("search_web(*)")) && deny.contains(&json!("command(*)")));
}
#[test]
fn configured_binary_path_is_used() {
    let config: CliConfig =
        serde_json::from_value(json!({"binary": "C:/tools/claude.exe"})).unwrap();
    let i = claude::command(&config, "P", &schema(), ToolAccess::None);
    assert_eq!(i.program.to_str(), Some("C:/tools/claude.exe"));
    assert_eq!(values(&i, "--model"), Vec::<&str>::new());
    assert_eq!(i.timeout, Duration::from_secs(300));
}
