use crate::{
    adapters::{baseline::*, http::HttpPlatform},
    *,
};
use std::sync::Arc;
impl Sidecar {
    pub async fn from_config(config: Config, secret: String) -> Result<Self> {
        let audit = Audit::open(&config.audit_file).await?;
        let mut signal = vrs::Standings::default();
        for path in config.vrs_files {
            let markdown = std::fs::read_to_string(path)?;
            let observed_at = signal.add(&markdown)?;
            audit.record_vrs(&markdown, observed_at).await?;
        }
        let ports = Ports {
            platform: Arc::new(HttpPlatform::new(&config.platform_url, secret)?),
            clock: Arc::new(SystemClock),
            signal: Arc::new(signal),
            odds: adapters::odds::build(),
            context: adapters::context::build(&config.context, &audit, Arc::new(SystemClock)),
            llm: adapters::debate::build(adapters::llm::build()),
            policy: Arc::new(ConfiguredPolicy),
            pickem: adapters::pickem::build(),
        };
        let sidecar = Sidecar {
            bots: config.bots,
            audit,
            ports,
        };

        Ok(sidecar)
    }
}
struct ConfiguredPolicy;
impl DecisionPolicy for ConfiguredPolicy {
    fn decide(
        &self,
        bot: &BotProfile,
        d: &Dossier,
        tier: Tier,
        adjustment: Option<LlmAdjustment>,
    ) -> Result<Option<Prediction>> {
        match bot.policy {
            Policy::VrsOnly => VrsOnly.decide(bot, d, tier, adjustment),
        }
    }
}
