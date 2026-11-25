use crate::*;
#[async_trait::async_trait]
pub trait LeaguePlatform: Send + Sync {
    async fn login(&self, bot: &BotProfile) -> Result<String>;
    async fn join(&self, token: &str, invite: &str) -> Result<()>;
    async fn leagues(&self, token: &str) -> Result<Vec<i64>>;
    async fn matches(&self, token: &str, league: i64) -> Result<Vec<(Match, bool)>>;
    async fn submit(&self, token: &str, league: i64, game: i64, pick: &Prediction) -> Result<()>;
}
