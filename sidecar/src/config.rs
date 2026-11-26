use crate::*;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize)]
pub enum Policy {
    VrsOnly,
    MarketOnly,
    HybridAnchored,
    Promoted { name: String, report: PathBuf },
}
#[derive(Clone, Deserialize)]
pub struct BotProfile {
    pub bot_key: String,
    pub nickname: String,
    pub language: Option<String>,
    pub avatar_url: Option<String>,
    pub invite_links: Vec<String>,
    pub policy: Policy,
    #[serde(default)]
    pub provider_chain: Vec<String>,
}
#[derive(Deserialize)]
pub struct Config {
    pub platform_url: String,
    pub audit_file: PathBuf,
    pub vrs_files: Vec<PathBuf>,
    pub bots: Vec<BotProfile>,
    #[serde(default)]
    pub context: adapters::context::ContextConfig,
    #[serde(default)]
    pub llm: adapters::llm::LlmConfig,
    #[serde(default)]
    pub debate: adapters::debate::DebateConfig,
}
/// Invite Links grant League membership: `Debug` shows how many there are, not
/// what they say.
impl std::fmt::Debug for BotProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BotProfile")
            .field("bot_key", &self.bot_key)
            .field("nickname", &self.nickname)
            .field("language", &self.language)
            .field("avatar_url", &self.avatar_url)
            .field("invite_links", &format_args!("[{} redacted]", self.invite_links.len()))
            .field("policy", &self.policy)
            .field("provider_chain", &self.provider_chain)
            .finish()
    }
}
impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("platform_url", &self.platform_url)
            .field("audit_file", &self.audit_file)
            .field("vrs_files", &self.vrs_files)
            .field("bots", &self.bots)
            .finish_non_exhaustive()
    }
}
impl Config {
    pub fn load(path: impl AsRef<std::path::Path>) -> Result<Self> {
        let config: Self = serde_json::from_reader(std::fs::File::open(path)?)?;
        let mut keys = std::collections::HashSet::new();
        for bot in &config.bots {
            if !keys.insert(&bot.bot_key) {
                return Err("duplicate bot_key".into());
            }
        }
        Ok(config)
    }
}
