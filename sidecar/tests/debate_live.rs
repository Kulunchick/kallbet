//! Runs the debate lifecycle of `debate_support` through the real, signed-in
//! `claude`, `codex` and `agy`: an early Draft by one agent, then the three-seat
//! debate at T-22m. It checks that every CLI accepts the seats'
//! prompts and schemas, and it is how `tests/fixtures/debate/` was captured.
//! Ignored by default: it makes four subscription calls.
//!
//!     cargo test -p kallbet-ai --test debate_live -- --ignored --nocapture
//!
//! With `DEBATE_CAPTURE_DIR=<dir>` set, each run's prompt, stdout and stderr are
//! written there as `<seat>-<cli>.*`.
mod debate_support;
mod support;
use async_trait::async_trait;
use debate_support::*;
use kallbet_ai::adapters::llm::*;
use serde_json::json;
use std::{path::PathBuf, sync::Arc};

struct Recorder {
    dir: Option<PathBuf>,
}
#[async_trait]
impl ProcessRunner for Recorder {
    async fn run(&self, invocation: &Invocation) -> Result<Finished, RunError> {
        let (seat, cli) = (role_of(invocation), program_of(invocation));
        let finished = TokioRunner.run(invocation).await;
        println!("--- {seat} by {cli}: {finished:?}");
        if let (Some(dir), Ok(out)) = (&self.dir, &finished) {
            let prompt = invocation.stdin.clone().unwrap_or_else(|| {
                let print = invocation
                    .args
                    .iter()
                    .find_map(|a| a.strip_prefix("--print="));
                print.unwrap_or_default().to_owned()
            });
            let stdout = if cli == "codex" { "jsonl" } else { "json" };
            std::fs::write(dir.join(format!("{seat}-{cli}.prompt.txt")), prompt).unwrap();
            std::fs::write(dir.join(format!("{seat}-{cli}.{stdout}")), &out.stdout).unwrap();
            if !out.stderr.trim().is_empty() {
                std::fs::write(dir.join(format!("{seat}-{cli}.stderr.txt")), &out.stderr).unwrap();
            }
        }
        finished
    }
}

#[tokio::test]
#[ignore]
async fn the_real_subscriptions_run_the_draft_and_the_debate() {
    let dir = std::env::var_os("DEBATE_CAPTURE_DIR").map(PathBuf::from);
    if let Some(dir) = &dir {
        std::fs::create_dir_all(dir).unwrap();
    }
    {
        let runner = Arc::new(Recorder { dir: dir.clone() });
        let mut l = Lifecycle::new(runner, json!({})).await;
        assert_eq!(l.tick_at(240).await.unwrap(), 1, "the Draft is submitted");
        l.human_reveals().await;
        println!("=== draft pick: {}", l.bot_pick().await);
        l.wiki.publish();
        assert_eq!(
            l.tick_at(22).await.unwrap(),
            1,
            "the debate's pick is submitted"
        );
        println!("=== final pick: {}", l.bot_pick().await);
        let turns = l.sidecar.audit.debate_turns().await.unwrap();
        for turn in &turns {
            println!(
                "=== {} by {} ({} ms, {}): {}",
                turn.role,
                turn.provider,
                turn.latency_ms,
                turn.status,
                turn.brief
                    .as_ref()
                    .map(|b| b.to_string())
                    .unwrap_or_default()
            );
        }
        // Claude drafts and is Market Advocate, Codex Devil's Advocate, Antigravity
        // Chief Arbiter; a seat whose subscription is exhausted falls back in its chain.
        let mut seats: Vec<_> = turns
            .iter()
            .map(|t| (t.role.as_str(), t.status.as_str()))
            .collect();
        seats.sort();
        assert_eq!(
            seats,
            [
                ("advocate", "ok"),
                ("arbiter", "ok"),
                ("devil", "ok"),
                ("predict", "ok")
            ]
        );
    }
}
