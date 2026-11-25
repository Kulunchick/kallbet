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

/// A platform failure the scheduler acts on.
#[derive(Debug, thiserror::Error, Clone, Copy, PartialEq, Eq)]
pub enum PlatformError {
    /// 401: the bot's token was refused (another registration under the same
    /// bot key rotated it). The scheduler registers again and retries once.
    #[error("the bot token was refused (401)")]
    Unauthorized,
}

/// What joining through an Invite Link came to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Join {
    /// The bot is a Member (it was already, or it just joined).
    Member,
    /// 404 `invite_invalid`: the link is unknown or was reissued (or it is not
    /// an Invite Link at all). It will not work again.
    InviteInvalid,
    /// 409 `league_archived`: the League is over.
    LeagueArchived,
}

#[async_trait::async_trait]
pub trait LeaguePlatform: Send + Sync {
    async fn login(&self, bot: &BotProfile) -> Result<String>;
    async fn join(&self, token: &str, invite: &str) -> Result<Join>;
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
