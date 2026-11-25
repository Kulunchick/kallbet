use crate::*;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Clone, Deserialize)]
pub enum Policy {
    VrsOnly,
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
