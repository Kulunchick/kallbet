//! The `kallbet-ai` command line.
//!
//! ```text
//! kallbet-ai [run] [--once]   the periodic loop (the default); --once = one tick, then exit
//! kallbet-ai health           exit 0 when the heartbeat file says the loop is alive
//! kallbet-ai backtest ...     retrospective benchmark (ticket #31)
//! kallbet-ai lab ...          Signal Lab: Candidate Signals under the fixed protocol (ticket #34)
//! kallbet-ai hltv-collect ... offline historical HLTV match collector (ticket #36)
//! ```
use super::{
    logging,
    redact::{Redactor, stderr_sink},
    runner::{self, LoopSettings},
    settings::Settings,
};
use crate::{adapters::baseline::SystemClock, *};
use std::process::ExitCode;
use tracing::{info, warn};

pub const USAGE: &str = "usage: kallbet-ai [run [--once] | health | backtest ... | lab ... | hltv-collect ...]\n\
configuration: SIDECAR_CONFIG (JSON file), KALLBET_BOT_SECRET and the SIDECAR_* variables;\n\
see docs/sidecar.md";

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Run { once: bool },
    Health,
    Backtest(Vec<String>),
    Lab(Vec<String>),
    HltvCollect(Vec<String>),
    Help,
}
pub fn parse(args: &[String]) -> Result<Command> {
    let mut args = args.iter().map(String::as_str);
    match args.next() {
        None => Ok(Command::Run { once: false }),
        Some("run") => match (args.next(), args.next()) {
            (None, _) => Ok(Command::Run { once: false }),
            (Some("--once"), None) => Ok(Command::Run { once: true }),
            _ => Err("run takes only --once".into()),
        },
        Some("health") => Ok(Command::Health),
        Some("backtest") => Ok(Command::Backtest(args.map(String::from).collect())),
        Some("lab") => Ok(Command::Lab(args.map(String::from).collect())),
        Some("hltv-collect") => Ok(Command::HltvCollect(args.map(String::from).collect())),
        Some("help" | "--help" | "-h") => Ok(Command::Help),
        Some(other) => Err(format!("unknown command {other:?}").into()),
    }
}

/// The process entry point: loads `.env` for local runs, runs the command and
/// returns the exit code. Errors are printed through the redactor.
pub async fn main_with(args: Vec<String>) -> ExitCode {
    // A local run keeps its variables in `.env`; the container has none.
    let _ = dotenvy::dotenv();
    let redactor = Redactor::default();
    redactor.learn_environment(std::env::vars());
    match execute(args, &redactor).await {
        Ok(code) => code,
        Err(e) => {
            eprintln!("kallbet-ai: {}", redactor.redact(&e.to_string()));
            ExitCode::FAILURE
        }
    }
}

async fn execute(args: Vec<String>, redactor: &Redactor) -> Result<ExitCode> {
    let command = parse(&args).map_err(|e| format!("{e}\n{USAGE}"))?;
    match command {
        Command::Help => {
            println!("{USAGE}");
            Ok(ExitCode::SUCCESS)
        }
        Command::Health => health(&Settings::from_env()?),
        Command::Backtest(args) => backtest(&args, redactor).await,
        Command::Lab(args) => lab(&args, redactor).await,
        Command::HltvCollect(args) => hltv_collect(&args, redactor).await,
        Command::Run { once } => run(Settings::from_env()?, once, redactor).await,
    }
}

/// Offline historical HLTV match collector (ticket #36); logs go through the redactor.
async fn hltv_collect(args: &[String], redactor: &Redactor) -> Result<ExitCode> {
    let filter = std::env::var("SIDECAR_LOG").unwrap_or_else(|_| "info".into());
    let subscriber = logging::subscriber(
        redactor.clone(),
        stderr_sink(),
        &filter,
        super::settings::LogFormat::Text,
    )?;
    tracing::subscriber::set_global_default(subscriber)?;
    println!("{}", crate::adapters::hltv_research::cli::run(args).await?);
    Ok(ExitCode::SUCCESS)
}

/// The retrospective benchmark (ticket #31); logs go to stderr through the redactor.
async fn backtest(args: &[String], redactor: &Redactor) -> Result<ExitCode> {
    let filter = std::env::var("SIDECAR_LOG").unwrap_or_else(|_| "info".into());
    let subscriber = logging::subscriber(
        redactor.clone(),
        stderr_sink(),
        &filter,
        super::settings::LogFormat::Text,
    )?;
    tracing::subscriber::set_global_default(subscriber)?;
    println!("{}", crate::adapters::backtest::cli::run(args).await?);
    Ok(ExitCode::SUCCESS)
}

/// The Signal Lab (ticket #34); logs go to stderr through the redactor.
async fn lab(args: &[String], redactor: &Redactor) -> Result<ExitCode> {
    let filter = std::env::var("SIDECAR_LOG").unwrap_or_else(|_| "info".into());
    let subscriber = logging::subscriber(
        redactor.clone(),
        stderr_sink(),
        &filter,
        super::settings::LogFormat::Text,
    )?;
    tracing::subscriber::set_global_default(subscriber)?;
    println!("{}", crate::adapters::lab::cli::run(args).await?);
    Ok(ExitCode::SUCCESS)
}

fn health(settings: &Settings) -> Result<ExitCode> {
    let health = match &settings.health_file {
        Some(path) => super::Health::new(path),
        None => settings.health_for(&Config::load(&settings.config_path)?),
    };
    match health
        .read()
        .map_err(|e| e.to_string())
        .and_then(|beat| beat.verdict(SystemClock.now(), &settings.limits()))
    {
        Ok(line) => {
            println!("{line}");
            Ok(ExitCode::SUCCESS)
        }
        Err(reason) => {
            println!("unhealthy: {reason}");
            Ok(ExitCode::FAILURE)
        }
    }
}

async fn run(settings: Settings, once: bool, redactor: &Redactor) -> Result<ExitCode> {
    let secret = settings.require_bot_secret()?.to_owned();
    let config = Config::load(&settings.config_path)?;
    redactor.learn(&secret);
    redactor.learn_invites(config.bots.iter().flat_map(|b| &b.invite_links));
    let subscriber = logging::subscriber(
        redactor.clone(),
        stderr_sink(),
        &settings.log_filter,
        settings.log_format,
    )?;
    tracing::subscriber::set_global_default(subscriber)?;

    let health = settings.health_for(&config);
    info!(
        version = env!("CARGO_PKG_VERSION"),
        platform = %config.platform_url,
        audit_file = %config.audit_file.display(),
        bots = %config.bots.iter().map(|b| b.bot_key.as_str()).collect::<Vec<_>>().join(","),
        tick_seconds = settings.tick_interval.as_secs(),
        once,
        "kallbet-ai starting"
    );
    if let Some(reason) = insecure_platform(&config.platform_url) {
        warn!("{reason}");
    }
    let clock: std::sync::Arc<dyn Clock> = std::sync::Arc::new(SystemClock);
    let mut sidecar = Sidecar::from_config(config, secret, clock.clone()).await?;
    if once {
        let submitted = sidecar.tick().await?;
        info!(submitted, "single tick ok");
        return Ok(ExitCode::SUCCESS);
    }
    let summary = runner::run(
        &mut sidecar,
        LoopSettings {
            interval: settings.tick_interval,
            tick_timeout: settings.tick_timeout,
        },
        clock.as_ref(),
        &health,
        runner::shutdown_signal(),
    )
    .await;
    info!(ticks = summary.ticks, "stopped");
    Ok(ExitCode::SUCCESS)
}

/// The shared secret travels in a header: over plain `http` it must stay on a
/// loopback address or a private (single-label) host name such as a Compose
/// service.
pub fn insecure_platform(url: &str) -> Option<String> {
    let rest = url.strip_prefix("http://")?;
    let host = rest.split(['/', ':']).next().unwrap_or("");
    let private = host == "localhost" || host == "[::1]" || !host.contains('.') || {
        host.parse::<std::net::Ipv4Addr>()
            .is_ok_and(|ip| ip.is_loopback() || ip.is_private())
    };
    (!private).then(|| {
        format!("platform_url uses plain http to {host}: the Bot secret would cross the network unencrypted; use https")
    })
}
