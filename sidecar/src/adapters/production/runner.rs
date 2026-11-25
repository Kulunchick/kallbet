//! The periodic loop and its shutdown.
use super::health::{Health, Heartbeat};
use crate::*;
use std::{future::Future, time::Duration};
use tracing::{error, info, warn};

/// One step of the sidecar. `Sidecar` is the production implementation; tests
/// script their own.
#[async_trait::async_trait]
pub trait Tick: Send {
    /// How many Predictions and Pick'ems the step submitted.
    async fn tick(&mut self) -> Result<usize>;
}
#[async_trait::async_trait]
impl Tick for Sidecar {
    async fn tick(&mut self) -> Result<usize> {
        Sidecar::tick(self).await
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LoopSettings {
    /// Time between the starts of two ticks (a tick that overruns it makes the
    /// next one start at once, but missed ticks are not queued).
    pub interval: Duration,
    /// A tick running longer is dropped and counted as a failure. Dropping it is
    /// safe: the audit makes every step repeatable, and a child process of an
    /// LLM CLI is killed with its future.
    pub tick_timeout: Duration,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Summary {
    pub ticks: u64,
    pub failed_ticks: u64,
    /// The signal arrived while a tick was running, and the tick was dropped.
    pub interrupted: bool,
}

/// Run until `shutdown` completes. Never returns an error: a failed tick is
/// logged and retried on the next interval (the platform being down for an hour
/// must not stop the sidecar).
pub async fn run(
    ticker: &mut dyn Tick,
    settings: LoopSettings,
    clock: &dyn Clock,
    health: &Health,
    shutdown: impl Future<Output = ()>,
) -> Summary {
    tokio::pin!(shutdown);
    let mut interval = tokio::time::interval(settings.interval);
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let mut summary = Summary {
        ticks: 0,
        failed_ticks: 0,
        interrupted: false,
    };
    let mut consecutive_failures = 0;
    loop {
        tokio::select! {
            biased;
            () = &mut shutdown => break,
            _ = interval.tick() => {}
        }
        let started_at = clock.now();
        let mut beat = Heartbeat {
            started_at,
            finished_at: None,
            consecutive_failures,
            submitted: 0,
        };
        write_beat(health, &beat);
        let began = tokio::time::Instant::now();
        let outcome = tokio::select! {
            biased;
            () = &mut shutdown => {
                warn!("shutdown requested during a tick: dropping it");
                summary.interrupted = true;
                break;
            }
            outcome = tokio::time::timeout(settings.tick_timeout, ticker.tick()) => outcome,
        };
        summary.ticks += 1;
        let elapsed_ms = began.elapsed().as_millis() as u64;
        match outcome {
            Ok(Ok(submitted)) => {
                consecutive_failures = 0;
                beat.submitted = submitted;
                info!(submitted, elapsed_ms, "tick ok");
            }
            Ok(Err(e)) => {
                consecutive_failures += 1;
                summary.failed_ticks += 1;
                error!(error = %e, consecutive_failures, elapsed_ms, "tick failed");
            }
            Err(_) => {
                consecutive_failures += 1;
                summary.failed_ticks += 1;
                error!(
                    timeout_seconds = settings.tick_timeout.as_secs(),
                    consecutive_failures, "tick timed out"
                );
            }
        }
        beat.finished_at = Some(clock.now());
        beat.consecutive_failures = consecutive_failures;
        write_beat(health, &beat);
    }
    info!(
        ticks = summary.ticks,
        failed_ticks = summary.failed_ticks,
        "shutting down"
    );
    summary
}

fn write_beat(health: &Health, beat: &Heartbeat) {
    if let Err(e) = health.write(beat) {
        warn!(error = %e, "could not write the heartbeat file");
    }
}

/// Completes on SIGINT or SIGTERM (Docker's stop signal); on Windows, on Ctrl-C.
pub async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        match (signal(SignalKind::terminate()), signal(SignalKind::interrupt())) {
            (Ok(mut term), Ok(mut int)) => {
                tokio::select! {
                    _ = term.recv() => info!("SIGTERM received"),
                    _ = int.recv() => info!("SIGINT received"),
                }
            }
            _ => {
                error!("cannot install signal handlers; running until killed");
                std::future::pending::<()>().await;
            }
        }
    }
    #[cfg(not(unix))]
    {
        if tokio::signal::ctrl_c().await.is_ok() {
            info!("Ctrl-C received");
        } else {
            std::future::pending::<()>().await;
        }
    }
}
