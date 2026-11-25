//! What the environment says about how to run. Pure: `from_lookup` takes the
//! lookup function, so tests never touch the real environment.
use super::health::Limits;
use crate::*;
use chrono::Duration as Span;
use std::{path::PathBuf, time::Duration};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogFormat {
    Text,
    Json,
}

#[derive(Clone)]
pub struct Settings {
    pub config_path: PathBuf,
    pub bot_secret: super::Secret,
    pub tick_interval: Duration,
    pub tick_timeout: Duration,
    pub log_filter: String,
    pub log_format: LogFormat,
    pub health_file: Option<PathBuf>,
    pub health_max_age: Span,
    pub health_max_failures: u32,
}
impl std::fmt::Debug for Settings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Settings")
            .field("config_path", &self.config_path)
            .field("bot_secret", &self.bot_secret)
            .field("tick_interval", &self.tick_interval)
            .field("tick_timeout", &self.tick_timeout)
            .field("log_filter", &self.log_filter)
            .field("log_format", &self.log_format)
            .field("health_file", &self.health_file)
            .finish_non_exhaustive()
    }
}

fn number(
    get: &dyn Fn(&str) -> Option<String>,
    name: &str,
    default: u64,
    range: std::ops::RangeInclusive<u64>,
) -> Result<u64> {
    let Some(raw) = get(name) else {
        return Ok(default);
    };
    match raw.trim().parse::<u64>() {
        Ok(n) if range.contains(&n) => Ok(n),
        _ => Err(format!(
            "{name} must be a whole number from {} to {}",
            range.start(),
            range.end()
        )
        .into()),
    }
}

impl Settings {
    /// The variables `health` needs: only where the heartbeat is.
    pub fn from_lookup(get: &dyn Fn(&str) -> Option<String>) -> Result<Self> {
        let get_nonempty = |name: &str| get(name).filter(|v| !v.trim().is_empty());
        let config_path = get_nonempty("SIDECAR_CONFIG")
            .ok_or("SIDECAR_CONFIG is not set: give it the path of the JSON configuration file")?;
        let bot_secret = get_nonempty("KALLBET_BOT_SECRET").unwrap_or_default();
        let log_format = match get_nonempty("SIDECAR_LOG_FORMAT").as_deref() {
            None | Some("text") => LogFormat::Text,
            Some("json") => LogFormat::Json,
            Some(_) => return Err("SIDECAR_LOG_FORMAT must be text or json".into()),
        };
        Ok(Self {
            config_path: config_path.into(),
            bot_secret: super::Secret::new(bot_secret),
            tick_interval: Duration::from_secs(number(get, "SIDECAR_TICK_SECONDS", 60, 5..=3600)?),
            tick_timeout: Duration::from_secs(number(
                get,
                "SIDECAR_TICK_TIMEOUT_SECONDS",
                900,
                30..=7200,
            )?),
            log_filter: get_nonempty("SIDECAR_LOG").unwrap_or_else(|| "info".into()),
            log_format,
            health_file: get_nonempty("SIDECAR_HEALTH_FILE").map(PathBuf::from),
            health_max_age: Span::seconds(number(
                get,
                "SIDECAR_HEALTH_MAX_AGE_SECONDS",
                1800,
                60..=86_400,
            )? as i64),
            health_max_failures: number(get, "SIDECAR_HEALTH_MAX_FAILURES", 10, 1..=1000)? as u32,
        })
    }
    pub fn from_env() -> Result<Self> {
        Self::from_lookup(&|name| std::env::var(name).ok())
    }
    /// `run` needs the Bot secret; `health` does not.
    pub fn require_bot_secret(&self) -> Result<&str> {
        let secret = self.bot_secret.expose();
        if secret.is_empty() {
            return Err("KALLBET_BOT_SECRET is not set: use the same value as the server's".into());
        }
        Ok(secret)
    }
    pub fn health_for(&self, config: &Config) -> super::Health {
        match &self.health_file {
            Some(path) => super::Health::new(path),
            None => super::Health::beside(&config.audit_file),
        }
    }
    pub fn limits(&self) -> Limits {
        Limits {
            max_age: self.health_max_age,
            max_failures: self.health_max_failures,
        }
    }
}
