//! Optional in-memory LeaguePlatform for replay scenarios. Seam 1 uses real HTTP by default.
use async_trait::async_trait;
use kallbet_ai::*;
use std::{collections::BTreeMap, sync::Mutex};

#[derive(Default)]
pub struct PlatformScript {
    pub leagues: BTreeMap<i64, Vec<(Match, bool)>>,
    pub joins: Mutex<Vec<String>>,
    pub submissions: Mutex<Vec<Prediction>>,
}
#[async_trait]
impl LeaguePlatform for PlatformScript {
    async fn login(&self, bot: &BotProfile) -> Result<String> {
        Ok(bot.bot_key.clone())
    }
    async fn join(&self, _: &str, invite: &str) -> Result<Join> {
        self.joins.lock().unwrap().push(invite.to_string());
        Ok(Join::Member)
    }
    async fn leagues(&self, _: &str) -> Result<Vec<i64>> {
        Ok(self.leagues.keys().copied().collect())
    }
    async fn matches(&self, _: &str, league: i64) -> Result<Vec<(Match, bool)>> {
        Ok(self.leagues.get(&league).cloned().unwrap_or_default())
    }
    async fn submit(&self, _: &str, _: i64, _: i64, pick: &Prediction) -> Result<()> {
        self.submissions.lock().unwrap().push(pick.clone());
        Ok(())
    }
}
