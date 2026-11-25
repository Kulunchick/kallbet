use crate::*;
#[async_trait::async_trait]
pub trait OddsSource: Send + Sync {
    /// The fair series-win probability for team A, strictly before cutoff.
    async fn quote(&self, game: &Match, cutoff: Instant) -> Result<Option<AsOf<f64>>>;

    /// Live feeds can finish fetching before fixing the evaluation cutoff.
    /// Replay and scripts retain the cutoff supplied by the caller. `tier`:
    /// the PreLock pass wants the closing line, so a live source refreshes a
    /// batch that a Draft would still reuse.
    async fn live_quote(
        &self,
        game: &Match,
        cutoff: Instant,
        _tier: Tier,
        _: &dyn Clock,
    ) -> Result<(Instant, Option<AsOf<f64>>)> {
        Ok((cutoff, self.quote(game, cutoff).await?))
    }
}
