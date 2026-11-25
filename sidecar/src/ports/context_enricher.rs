use crate::*;
#[async_trait::async_trait]
pub trait ContextEnricher: Send + Sync {
    async fn facts(&self, game: &Match, cutoff: Instant) -> Result<Vec<AsOf<serde_json::Value>>>;
}
