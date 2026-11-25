//! Ticket #29: the `HybridAnchored` decision policy and the Two-Tier Adversarial
//! Debate behind the `LlmRuntime` port. See docs/agents/codebase.md, "HybridAnchored
//! and the Debate".
//!
//! `TieredDebateLlm` gives a run one fast agent or the three-seat debate
//! (`tier::plan`); the debate runs the Market Advocate and the Devil's Advocate
//! side by side and hands the dossier and both briefs to the Chief Arbiter. Every
//! seat is its own `FallbackLlm` chain over the providers of ticket #27, and every
//! turn is kept in the audit file's `debate_turns`.
mod policy;
mod roles;
mod tier;
mod turns;

pub use policy::*;
pub use roles::brief_schema;
pub use tier::*;
pub use turns::DebateTurn;

use crate::{
    adapters::llm::{Providers, schema},
    *,
};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{Value, json};
use std::{collections::BTreeMap, sync::Arc};
use turns::TurnLog;

/// `debate` in the sidecar config file: per bot key, when it debates and which
/// providers sit where. A bot without an entry gets the defaults.
#[derive(Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DebateConfig {
    #[serde(default)]
    pub bots: BTreeMap<String, BotDebate>,
}

#[derive(Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BotDebate {
    #[serde(default)]
    pub mode: DebateMode,
    /// `p_base` at or beyond which (on either side) a Match is a mismatch.
    /// Default 0.80; above 0.5, at most 1.
    pub lopsided: Option<f64>,
    /// The single fast agent's chain. Default: the bot's `provider_chain`.
    pub fast: Option<Vec<String>>,
    /// The seats' chains. Default: the bot's `provider_chain`, turned by one
    /// place per seat, so that `[claude, codex, antigravity]` seats Claude as
    /// Market Advocate, Codex as Devil's Advocate and Antigravity as Chief
    /// Arbiter, each falling back to the other two.
    pub advocate: Option<Vec<String>>,
    pub devil: Option<Vec<String>>,
    pub arbiter: Option<Vec<String>>,
}

pub const DEFAULT_LOPSIDED: f64 = 0.80;

struct Settings {
    mode: DebateMode,
    lopsided: f64,
    fast: Vec<String>,
    advocate: Vec<String>,
    devil: Vec<String>,
    arbiter: Vec<String>,
}
impl DebateConfig {
    fn settings(&self, bot: &BotProfile) -> Settings {
        let own = self.bots.get(&bot.bot_key).cloned().unwrap_or_default();
        let turned = |by: usize| {
            let mut chain = bot.provider_chain.clone();
            if !chain.is_empty() {
                chain.rotate_left(by % bot.provider_chain.len());
            }
            chain
        };
        Settings {
            mode: own.mode,
            lopsided: own.lopsided.unwrap_or(DEFAULT_LOPSIDED),
            fast: own.fast.unwrap_or_else(|| turned(0)),
            advocate: own.advocate.unwrap_or_else(|| turned(0)),
            devil: own.devil.unwrap_or_else(|| turned(1)),
            arbiter: own.arbiter.unwrap_or_else(|| turned(2)),
        }
    }
}

/// One run: who asks, about what, and the key its turns are kept under.
struct Run<'a> {
    bot: &'a BotProfile,
    dossier: &'a Dossier,
    tier: Tier,
    hash: String,
}

pub struct TieredDebateLlm {
    providers: Arc<Providers>,
    config: DebateConfig,
    log: TurnLog,
}
impl TieredDebateLlm {
    /// The answer this seat already gave for this run, if any.
    async fn answered(&self, run: &Run<'_>, prompt: &RolePrompt) -> Option<Value> {
        let found = self
            .log
            .answered(&run.bot.bot_key, run.tier, &run.hash, &prompt.role)
            .await
            .unwrap_or_else(|e| {
                tracing::warn!(error = %e, "debate: reading the audit failed");
                None
            });
        found.filter(|v| schema::valid(v, &prompt.schema))
    }
    /// Asks the seat's chain and records the turn. `Ok(None)`: the chain is empty.
    async fn ask(
        &self,
        run: &Run<'_>,
        chain: &[String],
        prompt: &RolePrompt,
    ) -> Result<Option<Value>, LlmError> {
        let started = std::time::Instant::now();
        let outcome = self
            .providers
            .chain(chain)
            .answer(run.bot, run.dossier, run.tier, prompt)
            .await;
        let (provider, latency, status, brief) = match &outcome {
            Ok(None) => return Ok(None),
            Ok(Some(answer)) => (
                answer.provider.clone(),
                answer.elapsed,
                "ok",
                Some(answer.value.clone()),
            ),
            Err(e) => (chain.join(","), started.elapsed(), turns::status(*e), None),
        };
        let turn = DebateTurn {
            bot: run.bot.bot_key.clone(),
            match_id: run.dossier.game.id,
            tier: run.tier.name().into(),
            input_hash: run.hash.clone(),
            role: prompt.role.clone(),
            provider,
            latency_ms: latency.as_millis() as i64,
            status: status.into(),
            brief,
        };
        if let Err(e) = self.log.record(&turn).await {
            // A turn that cannot be audited is not used.
            tracing::error!(error = %e, "debate: writing the audit failed");
            return Err(LlmError::Transient);
        }
        Ok(outcome?.map(|answer| answer.value))
    }
    async fn seat(
        &self,
        run: &Run<'_>,
        chain: &[String],
        prompt: &RolePrompt,
        answered: Option<Value>,
    ) -> Result<Option<Value>, LlmError> {
        match answered {
            Some(brief) => Ok(Some(brief)),
            None => self.ask(run, chain, prompt).await,
        }
    }
}

#[async_trait]
impl LlmRuntime for TieredDebateLlm {
    async fn evaluate(
        &self,
        bot: &BotProfile,
        dossier: &Dossier,
        tier: Tier,
        prompt: &RolePrompt,
    ) -> Result<Option<Value>, LlmError> {
        let settings = self.config.settings(bot);
        // Nothing to anchor on: the policy makes no Prediction, so no call is spent.
        let Some(p_base) = policy::base(dossier) else {
            return Ok(None);
        };
        let flags = risk_flags(dossier);
        let run = Run {
            bot,
            dossier,
            tier,
            hash: Audit::input_hash(dossier).map_err(|_| LlmError::InvalidOutput)?,
        };
        let baseline = (
            "Baseline".to_string(),
            json!({
                "p_base_team_a": (p_base * 1e4).round() / 1e4,
                "risk_flags": flags,
            }),
        );
        let with_baseline = |mut prompt: RolePrompt| {
            prompt.attachments.push(baseline.clone());
            prompt
        };
        let format = dossier.game.format;
        if plan(
            settings.mode,
            tier,
            p_base,
            format,
            &flags,
            settings.lopsided,
        ) == Pass::Fast
        {
            let prompt = with_baseline(RolePrompt {
                role: roles::PREDICT.into(),
                ..prompt.clone()
            });
            let answered = self.answered(&run, &prompt).await;
            return self.seat(&run, &settings.fast, &prompt, answered).await;
        }
        if settings.arbiter.is_empty() {
            return Ok(None);
        }
        let asked = with_baseline(prompt.clone());
        // The arbiter's schema is the caller's, so look for its answer first.
        let verdict = roles::arbiter(&asked, Value::Null, Value::Null);
        if let Some(answer) = self.answered(&run, &verdict).await {
            return Ok(Some(answer));
        }
        let advocate = with_baseline(roles::advocate());
        let devil = with_baseline(roles::devil());
        let advocate_answered = self.answered(&run, &advocate).await;
        let devil_answered = self.answered(&run, &devil).await;
        let (advocate_brief, devil_brief) = tokio::join!(
            biased;
            self.seat(&run, &settings.advocate, &advocate, advocate_answered),
            self.seat(&run, &settings.devil, &devil, devil_answered),
        );
        // A seat whose whole chain failed leaves no brief; the arbiter still decides.
        let brief =
            |seat: Result<Option<Value>, LlmError>| seat.ok().flatten().unwrap_or(Value::Null);
        let verdict = roles::arbiter(&asked, brief(advocate_brief), brief(devil_brief));
        self.ask(&run, &settings.arbiter, &verdict).await
    }
}

/// The LLM runtime of the sidecar: `providers` (ticket #27) behind the tiers.
pub fn build(
    config: &DebateConfig,
    providers: Arc<Providers>,
    audit: &Audit,
) -> Result<Arc<dyn LlmRuntime>> {
    for (bot, settings) in &config.bots {
        if settings.lopsided.is_some_and(|p| !(p > 0.5 && p <= 1.0)) {
            return Err(
                format!("debate.bots.{bot}.lopsided must be above 0.5 and at most 1").into(),
            );
        }
    }
    Ok(Arc::new(TieredDebateLlm {
        providers,
        config: config.clone(),
        log: TurnLog {
            db: audit.db.clone(),
        },
    }))
}
