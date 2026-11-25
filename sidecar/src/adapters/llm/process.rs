//! Running a CLI once: the `ProcessRunner` port and its tokio implementation.
//! Tests replace the runner with a scripted one, so no test starts a real CLI.
use async_trait::async_trait;
use std::{
    path::PathBuf,
    process::Stdio,
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};
use tokio::io::AsyncWriteExt;

/// One non-interactive CLI run, built by a pure command builder.
#[derive(Clone, Debug, PartialEq)]
pub struct Invocation {
    pub program: PathBuf,
    pub args: Vec<String>,
    /// Written to stdin, which is then closed. `None`: stdin is closed at once
    /// (`codex exec` and `claude -p` wait on an open stdin).
    pub stdin: Option<String>,
    /// Files written into the run's own empty working directory before the start
    /// (`codex --output-schema` takes a path). Names are relative to that directory.
    pub files: Vec<(String, String)>,
    /// The process is killed when it runs longer.
    pub timeout: Duration,
}
/// What a finished process left behind.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Finished {
    pub code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}
#[derive(Debug, Clone, PartialEq)]
pub enum RunError {
    /// The program does not exist (not installed, wrong path).
    NotFound,
    /// Killed after `Invocation::timeout`.
    TimedOut,
    Io(String),
}

#[async_trait]
pub trait ProcessRunner: Send + Sync {
    async fn run(&self, invocation: &Invocation) -> Result<Finished, RunError>;
}

/// Starts the process with tokio in a fresh empty directory (so file tools see
/// nothing), kills it on timeout, and removes the directory afterwards.
pub struct TokioRunner;

static RUNS: AtomicU64 = AtomicU64::new(0);

#[async_trait]
impl ProcessRunner for TokioRunner {
    async fn run(&self, invocation: &Invocation) -> Result<Finished, RunError> {
        let dir = std::env::temp_dir().join(format!(
            "kallbet-ai-llm-{}-{}",
            std::process::id(),
            RUNS.fetch_add(1, Ordering::Relaxed)
        ));
        let result = run_in(&dir, invocation).await;
        let _ = tokio::fs::remove_dir_all(&dir).await;
        result
    }
}

async fn run_in(dir: &std::path::Path, invocation: &Invocation) -> Result<Finished, RunError> {
    let io = |e: std::io::Error| RunError::Io(e.to_string());
    tokio::fs::create_dir_all(dir).await.map_err(io)?;
    for (name, contents) in &invocation.files {
        tokio::fs::write(dir.join(name), contents)
            .await
            .map_err(io)?;
    }
    let mut command = tokio::process::Command::new(&invocation.program);
    command
        .args(&invocation.args)
        .current_dir(dir)
        .stdin(if invocation.stdin.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    let mut child = command.spawn().map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => RunError::NotFound,
        _ => io(e),
    })?;
    if let (Some(text), Some(mut stdin)) = (&invocation.stdin, child.stdin.take()) {
        // A child that exits without reading its input is not an error here.
        let _ = stdin.write_all(text.as_bytes()).await;
    }
    // Dropping the future on timeout drops the child, and kill_on_drop kills it.
    match tokio::time::timeout(invocation.timeout, child.wait_with_output()).await {
        Err(_) => Err(RunError::TimedOut),
        Ok(Err(e)) => Err(io(e)),
        Ok(Ok(output)) => Ok(Finished {
            code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        }),
    }
}
