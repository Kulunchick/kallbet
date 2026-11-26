use crate::{
    adapters::{baseline::*, http::HttpPlatform},
    *,
};
use std::sync::Arc;
impl Sidecar {
    /// The production sidecar of `config`. Every port that reads the time reads
    /// `clock` (production: `SystemClock`, the only adapter that reads the wall
    /// clock; tests: their fake).
    pub async fn from_config(config: Config, secret: String, clock: Arc<dyn Clock>) -> Result<Self> {
        let mut promotions = std::collections::HashMap::new();
        for bot in &config.bots {
            if let Policy::Promoted { name, report } = &bot.policy {
                promotions.insert(
                    bot.bot_key.clone(),
                    crate::promotion::Promotion::load(name, report)?,
                );
            }
        }
        let audit = Audit::open(&config.audit_file).await?;
        let signal = Arc::new(
            adapters::vrs_files::WatchedStandings::load(&config.vrs_files, &audit, clock.clone())
                .await?,
        );
        let odds = adapters::odds::build(&audit)?;
        let context = adapters::context::build(&config.context, &audit, clock.clone());
        let ports = Ports {
            pickem: adapters::pickem::new(
                clock.clone(),
                signal.clone(),
                odds.clone(),
                context.clone(),
            ),
            platform: Arc::new(HttpPlatform::new(&config.platform_url, secret)?),
            clock: clock.clone(),
            signal,
            odds,
            context,
            llm: adapters::debate::build(
                &config.debate,
                adapters::llm::build(&config.llm, clock.clone()),
                &audit,
            )?,
            policy: Arc::new(ConfiguredPolicy),
        };
        let mut sidecar = Sidecar::new(config.bots, audit, ports);
        sidecar.promotions = promotions;
        Ok(sidecar)
    }
}
pub(crate) struct ConfiguredPolicy;
impl DecisionPolicy for ConfiguredPolicy {
    fn wants_adjustment(&self, bot: &BotProfile) -> bool {
        match bot.policy {
            Policy::VrsOnly | Policy::MarketOnly | Policy::Promoted { .. } => false,
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
            Policy::Promoted { .. } => Err("missing configured promotion".into()),
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
