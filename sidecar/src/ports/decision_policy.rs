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

    /// The policy wanted an LLM adjustment but none came (every chain, or the
    /// Arbiter, failed with `error`): decide from the inputs alone. The
    /// Prediction is still submitted; a later tick may ask the LLM again.
    fn decide_unadjusted(
        &self,
        bot: &BotProfile,
        dossier: &Dossier,
        tier: Tier,
        error: LlmError,
    ) -> Result<Option<Prediction>> {
        let _ = error;
        self.decide(bot, dossier, tier, None)
    }
}
