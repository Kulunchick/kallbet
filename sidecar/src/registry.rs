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
            llm: adapters::debate::build(adapters::llm::build(&config.llm, Arc::new(SystemClock))),
            policy: Arc::new(ConfiguredPolicy),
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
            Policy::MarketOnly => adapters::odds::MarketOnly.decide(bot, d, tier, adjustment),
        }
    }
}
