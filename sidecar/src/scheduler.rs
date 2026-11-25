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
/// What a tick does when one of its steps fails.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OnFailure {
    /// Live: a failure of one Match, one League, one Stage or one Invite Link
    /// is logged and recorded in the audit (`tick_failures`), and the tick goes
    /// on with the next one. When the LLM gives no answer, the policy decides
    /// from its anchor and that Prediction is submitted. Only a failure to sign
    /// in or to list the bot's Leagues, and a `TemporalLeakError`, end the tick.
    #[default]
    Isolate,
    /// Backtest: the first failure ends the tick with that error, and a missing
    /// LLM answer is never replaced by the anchor's pick.
    Abort,
}
pub struct Sidecar {
    pub bots: Vec<BotProfile>,
    pub audit: Audit,
    pub ports: Ports,
    pub on_failure: OnFailure,
}
/// Where in a tick a failure happened.
#[derive(Clone, Copy, Default)]
struct At {
    league: Option<i64>,
    match_id: Option<i64>,
    stage: Option<u8>,
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
    /// Logs and records a failed step, unless it must end the tick: a leak
    /// always does, and with `OnFailure::Abort` every failure does.
    async fn failed(
        &self,
        bot: &BotProfile,
        now: Instant,
        step: &str,
        at: At,
        e: Box<dyn std::error::Error + Send + Sync>,
    ) -> Result<()> {
        if self.on_failure == OnFailure::Abort || e.is::<TemporalLeakError>() {
            return Err(e);
        }
        tracing::error!(
            bot = %bot.bot_key, step, league = ?at.league, match_id = ?at.match_id,
            stage = ?at.stage, error = %e, "tick step failed; going on with the next one"
        );
        let failure = Failure {
            at: now,
            bot: bot.bot_key.clone(),
            step: step.into(),
            league: at.league,
            match_id: at.match_id,
            stage: at.stage,
            error: e.to_string(),
        };
        if let Err(e) = self.audit.record_failure(&failure).await {
            tracing::error!(error = %e, "could not record a failed step in the audit");
        }
        Ok(())
    }
    async fn tick_bot(&self, bot: &BotProfile, now: Instant) -> Result<usize> {
        let platform = self.ports.platform.as_ref();
        let token = platform.login(bot).await?;
        for invite in &bot.invite_links {
            if let Err(e) = platform.join(&token, invite).await {
                self.failed(bot, now, "join", At::default(), e).await?;
            }
        }
        let mut submitted = 0;
        for league in platform.leagues(&token).await? {
            let at = At {
                league: Some(league),
                ..At::default()
            };
            let pickem = self
                .ports
                .pickem
                .tick(platform, &token, bot, league, now, &self.audit)
                .await;
            match pickem {
                Ok(failures) => {
                    for StageFailure { stage, error } in failures {
                        let at = At {
                            stage: Some(stage),
                            ..at
                        };
                        self.failed(bot, now, "pickem", at, error).await?;
                    }
                }
                Err(e) => self.failed(bot, now, "pickem", at, e).await?,
            }
            let matches = match platform.matches(&token, league).await {
                Ok(matches) => matches,
                Err(e) => {
                    self.failed(bot, now, "matches", at, e).await?;
                    continue;
                }
            };
            for (game, has_pick) in matches {
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
                let at = At {
                    match_id: Some(game.id),
                    ..at
                };
                match self.predict(bot, &token, league, game, tier, now).await {
                    Ok(n) => submitted += n,
                    Err(e) => self.failed(bot, now, "predict", at, e).await?,
                }
            }
        }
        Ok(submitted)
    }
    /// The LLM's adjustment, or why there is none.
    async fn adjustment(
        &self,
        bot: &BotProfile,
        dossier: &Dossier,
        tier: Tier,
    ) -> Result<Option<LlmAdjustment>, LlmError> {
        if !self.ports.policy.wants_adjustment(bot) {
            return Ok(None);
        }
        self.ports
            .llm
            .evaluate(bot, dossier, tier, &RolePrompt::adjustment())
            .await?
            .map(serde_json::from_value)
            .transpose()
            .map_err(|_| LlmError::InvalidOutput)
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
        let (now, market) = self
            .ports
            .odds
            .live_quote(&game, now, self.ports.clock.as_ref())
            .await?;
        if game
            .scheduled_start
            .is_some_and(|start| start - now <= Duration::minutes(15))
        {
            return Ok(0);
        }
        let signal = self.ports.signal.signal(&game, now).await?;
        let context = self.ports.context.facts(&game, now).await?;
        let mut facts = context
            .iter()
            .map(|c| c.at(now).cloned())
            .collect::<Result<Vec<_>, _>>()?;
        facts.sort_by_cached_key(serde_json::Value::to_string);
        let dossier = Dossier {
            game,
            signal: signal.as_ref().map(|s| s.at(now).cloned()).transpose()?,
            market: market.as_ref().map(|m| m.at(now).copied()).transpose()?,
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
            Decision::Pending => match self.adjustment(bot, &dossier, tier).await {
                Ok(adjustment) => {
                    let pick = self.ports.policy.decide(bot, &dossier, tier, adjustment)?;
                    self.audit.decide(run_id, pick.as_ref()).await?;
                    let Some(pick) = pick else { return Ok(0) };
                    pick
                }
                Err(e) if self.on_failure == OnFailure::Abort => return Err(e.into()),
                Err(e) => {
                    let run = Unadjusted {
                        bot,
                        token,
                        league,
                        dossier: &dossier,
                        tier,
                        now,
                        run_id,
                    };
                    return self.submit_unadjusted(run, e).await;
                }
            },
        };
        self.ports
            .platform
            .submit(token, league, id, &prediction)
            .await?;
        self.audit.submitted(run_id, now, &prediction).await?;
        Ok(1)
    }
    /// Every LLM chain (or the Arbiter) failed: the anchor's Prediction is
    /// submitted, once per run, and its Rationale says so. It is not the run's
    /// decision, so a later tick with the same dossier asks the LLM again.
    async fn submit_unadjusted(&self, run: Unadjusted<'_>, error: LlmError) -> Result<usize> {
        let Unadjusted {
            bot,
            token,
            league,
            dossier,
            tier,
            now,
            run_id,
        } = run;
        tracing::warn!(
            bot = %bot.bot_key, match_id = dossier.game.id, tier = tier.name(), error = %error,
            "no LLM adjustment available; deciding from the anchor"
        );
        if self.audit.unadjusted(run_id).await? {
            return Ok(0);
        }
        let decided = self
            .ports
            .policy
            .decide_unadjusted(bot, dossier, tier, error)?;
        let Some(pick) = decided else {
            return Ok(0);
        };
        self.ports
            .platform
            .submit(token, league, dossier.game.id, &pick)
            .await?;
        self.audit
            .submitted_unadjusted(run_id, now, &error.to_string(), &pick)
            .await?;
        Ok(1)
    }
}
/// A run whose LLM gave no answer.
struct Unadjusted<'a> {
    bot: &'a BotProfile,
    token: &'a str,
    league: i64,
    dossier: &'a Dossier,
    tier: Tier,
    now: Instant,
    run_id: i64,
}
