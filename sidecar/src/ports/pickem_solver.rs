use crate::*;
#[async_trait::async_trait]
pub trait PickemStep: Send + Sync {
    async fn tick(
        &self,
        platform: &dyn LeaguePlatform,
        token: &str,
        bot: &BotProfile,
        league: i64,
        now: Instant,
        audit: &Audit,
    ) -> Result<()>;
}
pub struct PairwiseProbability {
    pub first: i64,
    pub second: i64,
    pub map: f64,
}
/// #28 implements the deterministic solver and its League-level scheduling adapter.
pub trait PickemSolver: Send + Sync {
    fn solve(
        &self,
        stage: &AsOf<serde_json::Value>,
        ratings: &[AsOf<PairwiseProbability>],
        cutoff: Instant,
    ) -> Result<Option<serde_json::Value>>;
}
