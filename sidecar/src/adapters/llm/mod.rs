//! Ticket #27: subscription LLM runtimes behind the `LlmRuntime` port.
//! Leaf providers (`claude`, `codex`, `agy` CLIs and the HTTP adapter) are named
//! in the config's `llm.providers`; a bot's `provider_chain` lists names in the
//! order `FallbackLlm` tries them. See docs/agents/codebase.md, "LLM Runtimes".
pub mod antigravity;
pub mod classify;
pub mod claude;
pub mod codex;
mod fallback;
pub mod http;
mod process;
pub mod prompt;
pub mod schema;

pub use fallback::*;
pub use http::{Api, HttpRuntime};
pub use process::*;

use crate::*;
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
    sync::Arc,
};

/// The only sites a run may read when a bot enables live web search.
pub const WEB_DOMAINS: [&str; 2] = ["hltv.org", "liquipedia.net"];

/// `llm` in the sidecar config file.
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LlmConfig {
    /// Provider name (used in `provider_chain`) → provider.
    #[serde(default)]
    pub providers: BTreeMap<String, ProviderConfig>,
    /// Bot keys allowed to read `WEB_DOMAINS` in a live PreLock run.
    #[serde(default)]
    pub web_search_bots: BTreeSet<String>,
    /// How long a provider is skipped after a quota, transient or timeout failure.
    #[serde(default = "default_cooldown")]
    pub cooldown_minutes: i64,
}
impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            providers: BTreeMap::new(),
            web_search_bots: BTreeSet::new(),
            cooldown_minutes: default_cooldown(),
        }
    }
}
fn default_cooldown() -> i64 {
    30
}

#[derive(Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum ProviderConfig {
    Claude(CliConfig),
    Codex(CliConfig),
    Antigravity(CliConfig),
    Http(HttpConfig),
}

#[derive(Clone, Deserialize)]
pub struct CliConfig {
    /// Path or name on PATH. Default: `claude`, `codex`, `agy`. On Windows an npm
    /// `.cmd` shim is not found by name; give the full path of the executable.
    pub binary: Option<PathBuf>,
    pub model: Option<String>,
    #[serde(default = "default_cli_timeout")]
    pub timeout_seconds: u64,
}
impl CliConfig {
    pub fn binary_or(&self, name: &str) -> PathBuf {
        self.binary.clone().unwrap_or_else(|| name.into())
    }
    pub fn timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.timeout_seconds)
    }
}
fn default_cli_timeout() -> u64 {
    300
}

#[derive(Clone, Deserialize)]
pub struct HttpConfig {
    pub api: Api,
    /// Up to the version segment: `https://api.anthropic.com/v1`,
    /// `https://api.openai.com/v1`, or a proxy's equivalent. Unset: not configured.
    pub base_url: Option<String>,
    pub model: String,
    /// Name of the environment variable holding the key. Unset: no key is sent
    /// (an OAuth proxy that holds the subscription itself).
    pub api_key_env: Option<String>,
    #[serde(default = "default_http_timeout")]
    pub timeout_seconds: u64,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
}
fn default_http_timeout() -> u64 {
    180
}
fn default_max_tokens() -> u32 {
    4096
}

/// What a run may reach besides the dossier.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ToolAccess {
    None,
    /// Read pages on `WEB_DOMAINS` only.
    WhitelistedWeb,
}
/// Live runs may grant web access; a backtest never does, whatever the config says.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mode {
    Live,
    Backtest,
}
#[derive(Clone)]
pub struct ToolPolicy {
    pub mode: Mode,
    pub web_search_bots: BTreeSet<String>,
}
impl ToolPolicy {
    pub fn access(&self, bot: &BotProfile, tier: Tier) -> ToolAccess {
        let live = self.mode == Mode::Live;
        if live && tier == Tier::PreLock && self.web_search_bots.contains(&bot.bot_key) {
            ToolAccess::WhitelistedWeb
        } else {
            ToolAccess::None
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cli {
    Claude,
    Codex,
    Antigravity,
}
impl Cli {
    pub fn command(
        self,
        config: &CliConfig,
        prompt: &str,
        schema: &Value,
        access: ToolAccess,
    ) -> Invocation {
        match self {
            Cli::Claude => claude::command(config, prompt, schema, access),
            Cli::Codex => codex::command(config, prompt, schema, access),
            Cli::Antigravity => antigravity::command(config, prompt, schema, access),
        }
    }
    pub fn parse(self, out: &Finished, schema: &Value) -> Result<Value, LlmError> {
        match self {
            Cli::Claude => claude::parse(out, schema),
            Cli::Codex => codex::parse(out, schema),
            Cli::Antigravity => antigravity::parse(out, schema),
        }
    }
}

/// A subscription CLI as an `LlmRuntime`.
pub struct CliRuntime {
    pub cli: Cli,
    pub config: CliConfig,
    pub policy: ToolPolicy,
    pub runner: Arc<dyn ProcessRunner>,
}
#[async_trait]
impl LlmRuntime for CliRuntime {
    async fn evaluate(
        &self,
        bot: &BotProfile,
        dossier: &Dossier,
        tier: Tier,
        role: &RolePrompt,
    ) -> Result<Option<Value>, LlmError> {
        let access = self.policy.access(bot, tier);
        let text = prompt::render(dossier, tier, role, access);
        let invocation = self.cli.command(&self.config, &text, &role.schema, access);
        let finished = self.runner.run(&invocation).await.map_err(|e| match e {
            RunError::NotFound => LlmError::NotConfigured,
            RunError::TimedOut => LlmError::Timeout,
            RunError::Io(_) => LlmError::Transient,
            RunError::OverBudget => LlmError::QuotaExceeded,
        })?;
        self.cli.parse(&finished, &role.schema).map(Some)
    }
}

struct Unconfigured;
#[async_trait]
impl LlmRuntime for Unconfigured {
    async fn evaluate(
        &self,
        _: &BotProfile,
        _: &Dossier,
        _: Tier,
        _: &RolePrompt,
    ) -> Result<Option<Value>, LlmError> {
        Err(LlmError::NotConfigured)
    }
}

/// Every configured provider by name, each with its one breaker. As an
/// `LlmRuntime` it runs the calling bot's `provider_chain` as a `FallbackLlm`;
/// `chain` builds a chain from any list of names (a debate seat's own chain).
pub struct Providers {
    providers: BTreeMap<String, Arc<Provider>>,
    cooldown: chrono::Duration,
    clock: Arc<dyn Clock>,
    tools: ToolPolicy,
}
impl Providers {
    pub fn new(
        config: &LlmConfig,
        mode: Mode,
        clock: Arc<dyn Clock>,
        runner: Arc<dyn ProcessRunner>,
    ) -> Self {
        let policy = ToolPolicy {
            mode,
            web_search_bots: config.web_search_bots.clone(),
        };
        let web: BTreeSet<String> = config
            .providers
            .iter()
            .filter(|(_, p)| !matches!(p, ProviderConfig::Http(_)))
            .map(|(name, _)| name.clone())
            .collect();
        let runtimes = config.providers.iter().map(|(name, provider)| {
            let cli = |cli, config: &CliConfig| -> Arc<dyn LlmRuntime> {
                Arc::new(CliRuntime {
                    cli,
                    config: config.clone(),
                    policy: policy.clone(),
                    runner: runner.clone(),
                })
            };
            let runtime = match provider {
                ProviderConfig::Claude(c) => cli(Cli::Claude, c),
                ProviderConfig::Codex(c) => cli(Cli::Codex, c),
                ProviderConfig::Antigravity(c) => cli(Cli::Antigravity, c),
                ProviderConfig::Http(h) => Arc::new(HttpRuntime::new(h.clone())),
            };
            (name.clone(), runtime)
        });
        let mut providers = Self::from_runtimes(
            runtimes,
            chrono::Duration::minutes(config.cooldown_minutes),
            clock,
        );
        providers.tools = policy;
        for (name, provider) in &mut providers.providers {
            if web.contains(name) {
                Arc::get_mut(provider)
                    .expect("providers are not shared yet")
                    .web = true;
            }
        }
        providers
    }
    /// Providers from ready runtimes (fakes in tests, or adapters built elsewhere).
    pub fn from_runtimes(
        runtimes: impl IntoIterator<Item = (String, Arc<dyn LlmRuntime>)>,
        cooldown: chrono::Duration,
        clock: Arc<dyn Clock>,
    ) -> Self {
        let providers = runtimes
            .into_iter()
            .map(|(name, runtime)| {
                let breaker = Breaker::new(cooldown, clock.clone());
                (
                    name.clone(),
                    Arc::new(Provider {
                        name,
                        runtime,
                        breaker,
                        web: false,
                    }),
                )
            })
            .collect();
        Self {
            providers,
            cooldown,
            clock,
            // Ready-made runtimes decide their own tools; none is assumed here.
            tools: ToolPolicy {
                mode: Mode::Backtest,
                web_search_bots: BTreeSet::new(),
            },
        }
    }
    /// What a run of `bot` at `tier` may reach besides the dossier, by the policy
    /// these providers were built with (ticket #29 checks evidence URLs by it).
    pub fn access(&self, bot: &BotProfile, tier: Tier) -> ToolAccess {
        self.tools.access(bot, tier)
    }
    /// What the run of `bot` at `tier` that `provider` answered could reach:
    /// web access needs both the policy and a provider that has tools (not the
    /// HTTP adapter, which sends none). An unknown provider had none.
    pub fn access_of(&self, provider: &str, bot: &BotProfile, tier: Tier) -> ToolAccess {
        match self.providers.get(provider) {
            Some(p) if p.web => self.tools.access(bot, tier),
            _ => ToolAccess::None,
        }
    }
    /// The chain over `names`; a name with no provider fails as `NotConfigured`.
    pub fn chain(&self, names: &[String]) -> FallbackLlm {
        FallbackLlm::new(
            names
                .iter()
                .map(|name| {
                    self.providers.get(name).cloned().unwrap_or_else(|| {
                        Arc::new(Provider {
                            name: name.clone(),
                            runtime: Arc::new(Unconfigured),
                            breaker: Breaker::new(self.cooldown, self.clock.clone()),
                            web: false,
                        })
                    })
                })
                .collect(),
        )
    }
    pub fn breaker(&self, name: &str) -> Option<BreakerState> {
        self.providers.get(name).map(|p| p.breaker.state())
    }
}
#[async_trait]
impl LlmRuntime for Providers {
    async fn evaluate(
        &self,
        bot: &BotProfile,
        dossier: &Dossier,
        tier: Tier,
        prompt: &RolePrompt,
    ) -> Result<Option<Value>, LlmError> {
        self.chain(&bot.provider_chain)
            .evaluate(bot, dossier, tier, prompt)
            .await
    }
}

/// The live runtime: real processes, the given clock for the breakers.
pub fn build(config: &LlmConfig, clock: Arc<dyn Clock>) -> Arc<Providers> {
    Arc::new(Providers::new(
        config,
        Mode::Live,
        clock,
        Arc::new(TokioRunner),
    ))
}
