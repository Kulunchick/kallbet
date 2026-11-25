use crate::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub struct StagePickem {
    pub event_id: i64,
    pub ordinal: u8,
    pub locks_at: Option<Instant>,
    pub locked: bool,
    pub can_edit: bool,
    pub my_updated_at: Option<Instant>,
    pub teams: Vec<Team>,
    pub opening_matches: Vec<Match>,
    pub stage: serde_json::Value,
}

#[async_trait::async_trait]
pub trait LeaguePlatform: Send + Sync {
    async fn login(&self, bot: &BotProfile) -> Result<String>;
    async fn join(&self, token: &str, invite: &str) -> Result<()>;
    async fn leagues(&self, token: &str) -> Result<Vec<i64>>;
    async fn matches(&self, token: &str, league: i64) -> Result<Vec<(Match, bool)>>;
    async fn submit(&self, token: &str, league: i64, game: i64, pick: &Prediction) -> Result<()>;
    async fn pickem_stages(&self, token: &str, league: i64) -> Result<Vec<StagePickem>> {
        let _ = (token, league);
        Ok(vec![])
    }
    async fn submit_pickem(
        &self,
        token: &str,
        league: i64,
        stage: u8,
        payload: &serde_json::Value,
    ) -> Result<bool> {
        let _ = (token, league, stage, payload);
        Ok(false)
    }
}
