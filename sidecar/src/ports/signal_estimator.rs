use crate::*;
#[async_trait::async_trait]
pub trait SignalEstimator: Send + Sync {
    async fn signal(&self, game: &Match, cutoff: Instant) -> Result<Option<AsOf<Signal>>>;
}
