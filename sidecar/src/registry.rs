use crate::{
    adapters::{baseline::*, http::HttpPlatform},
    *,
};
use std::sync::Arc;
impl Sidecar {
    pub async fn from_config(config: Config, secret: String) -> Result<Self> {
        let audit = Audit::open(&config.audit_file).await?;
        let mut signal = vrs::Standings::default();
        for path in vrs_paths(&config.vrs_files)? {
            let markdown = std::fs::read_to_string(path)?;
            let observed_at = signal.add(&markdown)?;
            audit.record_vrs(&markdown, observed_at).await?;
        }
        let signal = Arc::new(signal);
        let odds = adapters::odds::build(&audit)?;
        let clock = Arc::new(SystemClock);
        let ports = Ports {
            pickem: adapters::pickem::new(clock.clone(), signal.clone(), odds.clone()),
            platform: Arc::new(HttpPlatform::new(&config.platform_url, secret)?),
            clock,
            signal,
            odds,
            context: adapters::context::build(&config.context, &audit, Arc::new(SystemClock)),
            llm: adapters::debate::build(
                &config.debate,
                adapters::llm::build(&config.llm, Arc::new(SystemClock)),
                &audit,
            )?,
            policy: Arc::new(ConfiguredPolicy),
        };
        let sidecar = Sidecar {
            bots: config.bots,
            audit,
            ports,
            on_failure: OnFailure::Isolate,
        };

        Ok(sidecar)
    }
}
pub(crate) struct ConfiguredPolicy;
impl DecisionPolicy for ConfiguredPolicy {
    fn wants_adjustment(&self, bot: &BotProfile) -> bool {
        match bot.policy {
            Policy::VrsOnly | Policy::MarketOnly => false,
            Policy::HybridAnchored => adapters::debate::HybridAnchored.wants_adjustment(bot),
        }
    }
    fn decide(
        &self,
        bot: &BotProfile,
        d: &Dossier,
        tier: Tier,
        adjustment: Option<LlmAdjustment>,
    ) -> Result<Option<Prediction>> {
        match bot.policy {
            Policy::VrsOnly => VrsOnly.decide(bot, d, tier, adjustment),
            Policy::MarketOnly => adapters::odds::MarketOnly.decide(bot, d, tier, adjustment),
            Policy::HybridAnchored => {
                adapters::debate::HybridAnchored.decide(bot, d, tier, adjustment)
            }
        }
    }
    fn decide_unadjusted(
        &self,
        bot: &BotProfile,
        d: &Dossier,
        tier: Tier,
        error: LlmError,
    ) -> Result<Option<Prediction>> {
        match bot.policy {
            Policy::HybridAnchored => {
                adapters::debate::HybridAnchored.decide_unadjusted(bot, d, tier, error)
            }
            _ => self.decide(bot, d, tier, None),
        }
    }
}

/// `vrs_files` entries are Markdown files or directories of them: a directory
/// contributes its Valve global files (`standings_global_*.md`), so a new Valve snapshot is one more file in a
/// volume and a restart, not a config edit.
fn vrs_paths(entries: &[std::path::PathBuf]) -> Result<Vec<std::path::PathBuf>> {
    let mut paths = Vec::new();
    for entry in entries {
        if entry.is_dir() {
            let mut files = std::fs::read_dir(entry)?
                .map(|f| f.map(|f| f.path()))
                .collect::<std::io::Result<Vec<_>>>()?;
            files.retain(|f| {
                f.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with("standings_global_") && n.ends_with(".md"))
            });
            files.sort();
            paths.extend(files);
        } else {
            paths.push(entry.clone());
        }
    }
    Ok(paths)
}
