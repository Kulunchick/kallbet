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
    /// `PANDASCORE_TOKEN` (secret).
    pub pandascore_token: Option<String>,
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

        Ok(Config {
            bind_addr,
            database_url: get("DATABASE_URL").unwrap_or_else(|| "sqlite://kallbet.db".into()),
            web_dir: get("KALLBET_WEB_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("web/dist")),
            dev_login,
            bot_token: get("BOT_TOKEN"),
            pandascore_token: get("PANDASCORE_TOKEN"),
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
            pandascore_token: None,
        }
    }
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
            .field("pandascore_token", &redact(&self.pandascore_token))
            .finish()
    }
}
