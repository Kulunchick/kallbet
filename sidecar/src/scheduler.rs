use crate::audit::Decision;
use crate::*;
use chrono::Duration;
use sha2::{Digest, Sha256};
use std::sync::Arc;
pub(crate) fn digest(data: &[u8]) -> String {
    hex::encode(Sha256::digest(data))
}
pub struct Ports {
    pub platform: Arc<dyn LeaguePlatform>,
    pub clock: Arc<dyn Clock>,
    pub signal: Arc<dyn SignalEstimator>,
    pub odds: Arc<dyn OddsSource>,
    pub context: Arc<dyn ContextEnricher>,
    pub llm: Arc<dyn LlmRuntime>,
    pub policy: Arc<dyn DecisionPolicy>,
    pub pickem: Arc<dyn PickemStep>,
}
pub struct Sidecar {
    pub bots: Vec<BotProfile>,
    pub audit: Audit,
    pub ports: Ports,
}
impl Sidecar {
    pub async fn tick(&mut self) -> Result<usize> {
        let now = self.ports.clock.now();
        let mut submitted = 0;
        let mut error = None;
        for bot in &self.bots {
            match self.tick_bot(bot, now).await {
                Ok(n) => submitted += n,
                Err(e) => error = Some(e),
            }
        }
        match error {
            Some(e) => Err(e),
            None => Ok(submitted),
        }
    }
    async fn tick_bot(&self, bot: &BotProfile, now: Instant) -> Result<usize> {
        let platform = self.ports.platform.as_ref();
        let token = platform.login(bot).await?;
        for invite in &bot.invite_links {
            platform.join(&token, invite).await?;
        }
        let mut submitted = 0;
        for league in platform.leagues(&token).await? {
            self.ports
                .pickem
                .tick(platform, &token, bot, league, now, &self.audit)
                .await?;
            for (game, has_pick) in platform.matches(&token, league).await? {
                let until = game.scheduled_start.map(|start| start - now);
                if until.is_some_and(|d| d <= Duration::minutes(15)) {
                    continue;
                }
                let tier = if until.is_some_and(|d| d <= Duration::minutes(25)) {
                    Tier::PreLock
                } else if !has_pick {
                    Tier::Draft
                } else {
                    continue;
                };
                submitted += self.predict(bot, &token, league, game, tier, now).await?;
            }
        }
        Ok(submitted)
    }
    async fn predict(
        &self,
        bot: &BotProfile,
        token: &str,
        league: i64,
        game: Match,
        tier: Tier,
        now: Instant,
    ) -> Result<usize> {
        let signal = self.ports.signal.signal(&game, now).await?;
        let market = self.ports.odds.quote(&game, now).await?;
        let context = self.ports.context.facts(&game, now).await?;
        let mut facts = context
            .iter()
            .map(|c| c.at(now).cloned())
            .collect::<Result<Vec<_>, _>>()?;
        facts.sort_by_cached_key(serde_json::Value::to_string);
        let dossier = Dossier {
            game,
            signal: signal.as_ref().map(|s| s.at(now).cloned()).transpose()?,
            market: market
                .as_ref()
                .map(|m| m.at(now).map(|p| (p * 10000.).round() / 10000.))
                .transpose()?,
            context: facts,
        };
        let id = dossier.game.id;
        let (run_id, state) = self
            .audit
            .begin(&bot.bot_key, league, &dossier, tier, now)
            .await?;
        let prediction = match state {
            Decision::Complete => return Ok(0),
            Decision::Submit(pick) => pick,
            Decision::Pending => {
                let adjustment = self
                    .ports
                    .llm
                    .evaluate(bot, &dossier, tier, &RolePrompt::adjustment())
                    .await?
                    .map(serde_json::from_value)
                    .transpose()
                    .map_err(|_| LlmError::InvalidOutput)?;
                let pick = self.ports.policy.decide(bot, &dossier, tier, adjustment)?;
                self.audit.decide(run_id, pick.as_ref()).await?;
                let Some(pick) = pick else { return Ok(0) };
                pick
            }
        };
        self.ports
            .platform
            .submit(token, league, id, &prediction)
            .await?;
        self.audit.submitted(run_id, now, &prediction).await?;
        Ok(1)
    }
}
