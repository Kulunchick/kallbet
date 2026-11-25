use crate::*;
#[async_trait::async_trait]
pub trait OddsSource: Send + Sync {
    async fn quote(&self, game: &Match, cutoff: Instant) -> Result<Option<AsOf<f64>>>;
}
