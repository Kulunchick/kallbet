//! Runs the real, signed-in `claude`, `codex` and `agy` through the adapters, once
//! with no tools and once with the whitelisted web access, to check that each CLI
//! accepts the flags the command builders produce. Ignored by default: it spends
//! a little subscription quota and needs the CLIs installed and logged in.
//!
//!     cargo test -p kallbet-ai --test llm_live -- --ignored --nocapture
use kallbet_ai::{adapters::llm::*, *};
use serde_json::json;
use std::sync::Arc;

fn bot() -> BotProfile {
    BotProfile {
        bot_key: "live".into(),
        nickname: "live".into(),
        language: None,
        avatar_url: None,
        invite_links: vec![],
        policy: Policy::VrsOnly,
        provider_chain: vec![],
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
            id: 1,
            team_a: team(1, "TEAM_ALPHA"),
            team_b: team(2, "TEAM_BETA"),
            format: Format::Bo3,
            scheduled_start: None,
        },
        signal: None,
        market: None,
        context: vec![],
    }
}

#[tokio::test]
#[ignore]
async fn each_cli_answers_with_the_schema() {
    let prompt = RolePrompt {
        role: "smoke".into(),
        instructions: "Ignore the dossier. What is 2+3? Put it in `sum`.".into(),
        schema: json!({"type":"object","properties":{"sum":{"type":"integer"}},"required":["sum"],"additionalProperties":false}),
        attachments: vec![],
    };
    let mut failures = vec![];
    for cli in [Cli::Claude, Cli::Codex, Cli::Antigravity] {
        for (tier, web) in [
            (Tier::Draft, vec![]),
            (Tier::PreLock, vec!["live".to_string()]),
        ] {
            let runtime = CliRuntime {
                cli,
                config: serde_json::from_value(json!({"timeout_seconds": 180})).unwrap(),
                policy: ToolPolicy {
                    mode: Mode::Live,
                    web_search_bots: web.into_iter().collect(),
                },
                runner: Arc::new(TokioRunner),
            };
            let result = runtime.evaluate(&bot(), &dossier(), tier, &prompt).await;
            println!("{cli:?} {}: {result:?}", tier.name());
            if result != Ok(Some(json!({"sum": 5}))) {
                failures.push(format!("{cli:?} {}", tier.name()));
            }
        }
    }
    assert!(failures.is_empty(), "{failures:?}");
}

#[tokio::test]
#[ignore]
async fn whitelisted_web_refuses_other_sites() {
    let prompt = RolePrompt {
        role: "smoke".into(),
        instructions: "Ignore the dossier. Read https://example.com/ and put the text of its <h1> in `heading`. \
            If you cannot read it, put DENIED in `heading`. Do not guess."
            .into(),
        schema: json!({"type":"object","properties":{"heading":{"type":"string"}},"required":["heading"],"additionalProperties":false}),
        attachments: vec![],
    };
    let mut leaks = vec![];
    for cli in [Cli::Claude, Cli::Codex, Cli::Antigravity] {
        let runtime = CliRuntime {
            cli,
            config: serde_json::from_value(json!({"timeout_seconds": 240})).unwrap(),
            policy: ToolPolicy {
                mode: Mode::Live,
                web_search_bots: ["live".to_string()].into_iter().collect(),
            },
            runner: Arc::new(TokioRunner),
        };
        let result = runtime
            .evaluate(&bot(), &dossier(), Tier::PreLock, &prompt)
            .await;
        println!("{cli:?}: {result:?}");
        if let Ok(Some(v)) = &result
            && v["heading"]
                .as_str()
                .is_some_and(|h| h.contains("Example Domain"))
        {
            leaks.push(format!("{cli:?}"));
        }
    }
    assert!(
        leaks.is_empty(),
        "read a site outside the whitelist: {leaks:?}"
    );
}
