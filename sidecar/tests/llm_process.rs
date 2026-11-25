//! The real process runner against local stand-in programs (never a real CLI).
//! The timeout test gives a hanging program a 300 ms limit: that is the behaviour
//! under test, not a sleep. (Paused tokio time does not help here: the pipe
//! readers run on the blocking pool, which stops the clock from auto-advancing.)
use kallbet_ai::adapters::llm::{Invocation, ProcessRunner, RunError, TokioRunner};
use std::time::Duration;

fn invocation(program: &str, args: &[&str]) -> Invocation {
    Invocation {
        program: program.into(),
        args: args.iter().map(|a| a.to_string()).collect(),
        stdin: None,
        files: vec![],
        timeout: Duration::from_secs(30),
    }
}
/// A program that never exits on its own.
fn hanging() -> Invocation {
    if cfg!(windows) {
        invocation(
            "powershell",
            &[
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "Start-Sleep -Seconds 3600",
            ],
        )
    } else {
        invocation("sleep", &["3600"])
    }
}

#[tokio::test]
async fn a_run_past_its_timeout_is_killed() {
    let mut i = hanging();
    i.timeout = Duration::from_millis(300);
    let started = std::time::Instant::now();
    assert_eq!(TokioRunner.run(&i).await, Err(RunError::TimedOut));
    assert!(
        started.elapsed() < Duration::from_secs(20),
        "returned without waiting for the child"
    );
}

#[tokio::test]
async fn a_missing_binary_is_not_found() {
    let i = invocation("kallbet-ai-test-no-such-cli", &["-p"]);
    assert_eq!(TokioRunner.run(&i).await, Err(RunError::NotFound));
}

#[tokio::test]
async fn stdin_and_files_reach_the_process_in_its_own_directory() {
    // `sort` echoes its sorted input on Windows and Unix alike; the file is read back
    // by name from the run's working directory.
    let mut i = invocation("sort", &[]);
    i.stdin = Some("prompt line\n".into());
    let out = TokioRunner.run(&i).await.unwrap();
    assert_eq!(out.code, Some(0));
    assert_eq!(out.stdout.trim(), "prompt line");

    let mut i = invocation("sort", &["schema.json"]);
    i.files = vec![("schema.json".into(), "{\"type\":\"object\"}\n".into())];
    let out = TokioRunner.run(&i).await.unwrap();
    assert_eq!(out.stdout.trim(), "{\"type\":\"object\"}");
}
