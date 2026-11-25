use crate::*;
pub trait DecisionPolicy: Send + Sync {
    /// Policies opt in to running an LLM for this profile.
    fn wants_adjustment(&self, _bot: &BotProfile) -> bool {
        false
    }

    fn decide(
        &self,
        bot: &BotProfile,
        dossier: &Dossier,
        tier: Tier,
        adjustment: Option<LlmAdjustment>,
    ) -> Result<Option<Prediction>>;
}
