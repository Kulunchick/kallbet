//! Configuration, read from the environment (and `.env` in development,
//! loaded by `main`). Secrets only ever come from here.
//!
//! Adding a setting: add a field, read it in [`Config::from_lookup`], give it
//! a value in [`Config::for_tests`], and document it in `.env.example`.

use std::fmt;
use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Clone)]
pub struct Config {
    /// `KALLBET_BIND`, default `127.0.0.1:8095`.
    pub bind_addr: SocketAddr,
    /// `DATABASE_URL`, default `sqlite://kallbet.db`.
    pub database_url: String,
    /// `KALLBET_WEB_DIR`: the built SPA, default `web/dist`.
    pub web_dir: PathBuf,
    /// `KALLBET_DEV_LOGIN`: mounts the dev-only login bypass. Off unless set
    /// to `true`; never set it in production.
    pub dev_login: bool,
    /// `BOT_TOKEN`: the Telegram bot token (secret).
    pub bot_token: Option<String>,
    /// `BOT_USERNAME`: the bot's @username (without the @), which the
    /// Telegram Login Widget needs to render.
    pub bot_username: Option<String>,
    /// `PANDASCORE_TOKEN` (secret).
    pub pandascore_token: Option<String>,
    /// `KALLBET_CHROME_URL`: the DevTools endpoint of the headless Chrome the
    /// HLTV adapter loads pages with (`http://chrome:9222` in compose). Unset
    /// means the HLTV adapter is skipped.
    pub chrome_url: Option<String>,
    /// `KALLBET_LOGO_DIR`: where downloaded team logos are kept, default
    /// `logos`.
    pub logo_dir: PathBuf,
    /// `KALLBET_BREAKER_FAILURES`: consecutive failures after which a Data
    /// Source's circuit breaker opens, default 3.
    pub breaker_failures: u32,
    /// `KALLBET_BREAKER_OPEN_MINUTES`: how long an open breaker skips its
    /// Data Source before one more try (half-open), default 15.
    pub breaker_open_minutes: i64,
    /// `KALLBET_MATCH_WINDOW_HOURS`: how far apart (±) two Data Sources'
    /// start times may be for their records to count as the same Match,
    /// default 6.
    pub match_window_hours: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("invalid configuration: {key}: {problem}")]
pub struct ConfigError {
    pub key: &'static str,
    pub problem: String,
}

impl Config {
    pub fn from_env() -> Result<Config, ConfigError> {
        Config::from_lookup(|key| std::env::var(key).ok())
    }

    /// Builds the configuration from any key → value lookup. Blank values
    /// count as unset.
    pub fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> Result<Config, ConfigError> {
        let get = |key: &str| {
            lookup(key)
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
        };

        let bind_addr = match get("KALLBET_BIND") {
            None => SocketAddr::from(([127, 0, 0, 1], 8095)),
            Some(v) => v.parse().map_err(|e| ConfigError {
                key: "KALLBET_BIND",
                problem: format!("{e}"),
            })?,
        };
        let dev_login = match get("KALLBET_DEV_LOGIN").as_deref() {
            None | Some("false") => false,
            Some("true") => true,
            Some(other) => {
                return Err(ConfigError {
                    key: "KALLBET_DEV_LOGIN",
                    problem: format!("expected `true` or `false`, got `{other}`"),
                });
            }
        };

        let positive = |key: &'static str, default: i64| -> Result<i64, ConfigError> {
            match get(key) {
                None => Ok(default),
                Some(v) => match v.parse::<i64>() {
                    Ok(n) if n > 0 => Ok(n),
                    _ => Err(ConfigError {
                        key,
                        problem: format!("expected a positive whole number, got `{v}`"),
                    }),
                },
            }
        };
        let breaker_failures = positive("KALLBET_BREAKER_FAILURES", 3)?
            .try_into()
            .map_err(|_| ConfigError {
                key: "KALLBET_BREAKER_FAILURES",
                problem: "too large".into(),
            })?;
        let breaker_open_minutes = positive("KALLBET_BREAKER_OPEN_MINUTES", 15)?;
        let match_window_hours = positive("KALLBET_MATCH_WINDOW_HOURS", 6)?;

        Ok(Config {
            breaker_failures,
            breaker_open_minutes,
            match_window_hours,
            bind_addr,
            database_url: get("DATABASE_URL").unwrap_or_else(|| "sqlite://kallbet.db".into()),
            web_dir: get("KALLBET_WEB_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("web/dist")),
            dev_login,
            bot_token: get("BOT_TOKEN"),
            bot_username: get("BOT_USERNAME").map(|u| u.trim_start_matches('@').to_string()),
            pandascore_token: get("PANDASCORE_TOKEN"),
            chrome_url: get("KALLBET_CHROME_URL"),
            logo_dir: get("KALLBET_LOGO_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("logos")),
        })
    }

    /// The configuration the Seam 1 harness starts from.
    pub fn for_tests() -> Config {
        Config {
            bind_addr: SocketAddr::from(([127, 0, 0, 1], 0)),
            database_url: "sqlite::memory:".into(),
            web_dir: PathBuf::from("web/dist"),
            dev_login: true,
            bot_token: Some(crate::testing::TEST_BOT_TOKEN.into()),
            bot_username: None,
            pandascore_token: None,
            chrome_url: None,
            logo_dir: PathBuf::from("logos"),
            breaker_failures: 3,
            breaker_open_minutes: 15,
            match_window_hours: 6,
        }
    }
}

/// Where a URL points (`scheme://host:port`), without user info, path or
/// query, any of which may carry credentials. `…` marks that something was
/// dropped.
pub fn url_origin(url: &str) -> String {
    let (scheme, rest) = url.split_once("://").unwrap_or(("", url));
    let authority_end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let authority = &rest[..authority_end];
    let host = authority.rsplit_once('@').map_or(authority, |(_, host)| host);
    let dropped = host.len() != rest.len();
    let prefix = if scheme.is_empty() { String::new() } else { format!("{scheme}://") };
    format!("{prefix}{host}{}", if dropped { "/…" } else { "" })
}

fn redact(secret: &Option<String>) -> &'static str {
    match secret {
        Some(_) => "<set>",
        None => "<unset>",
    }
}

impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Config")
            .field("bind_addr", &self.bind_addr)
            .field("database_url", &self.database_url)
            .field("web_dir", &self.web_dir)
            .field("dev_login", &self.dev_login)
            .field("bot_token", &redact(&self.bot_token))
            .field("bot_username", &self.bot_username)
            .field("pandascore_token", &redact(&self.pandascore_token))
            .field("chrome_url", &self.chrome_url.as_deref().map(url_origin))
            .field("logo_dir", &self.logo_dir)
            .field("breaker_failures", &self.breaker_failures)
            .field("breaker_open_minutes", &self.breaker_open_minutes)
            .field("match_window_hours", &self.match_window_hours)
            .finish()
    }
}
