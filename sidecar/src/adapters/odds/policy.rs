use crate::{
    BotProfile, DecisionPolicy, Dossier, LlmAdjustment, Prediction, Result, Tier,
    adapters::baseline::VrsOnly,
};
use serde_json::json;

pub struct MarketOnly;

impl DecisionPolicy for MarketOnly {
    fn decide(
        &self,
        bot: &BotProfile,
        dossier: &Dossier,
        tier: Tier,
        _: Option<LlmAdjustment>,
    ) -> Result<Option<Prediction>> {
        let Some(market) = dossier.market else {
            return VrsOnly.decide(bot, dossier, tier, None);
        };
        let (team_id, probability) = if market >= 0.5 {
            (dossier.game.team_a.id, market)
        } else {
            (dossier.game.team_b.id, 1.0 - market)
        };
        Ok(Some(Prediction {
            team_id,
            rationale: json!({
                "p_win": probability,
                "override_market": false,
                "preliminary": tier == Tier::Draft,
                "summary": {
                    "en": "Prediction based on the market probability.",
                    "ru": "Прогноз на основе рыночной вероятности.",
                    "uk": "Прогноз на основі ринкової ймовірності."
                },
                "factors": []
            }),
        }))
    }
}
