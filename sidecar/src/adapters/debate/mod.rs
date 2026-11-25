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
    adapters::llm::{Providers, ToolAccess, schema},
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
    /// The step, in percentage points, to which the market probability is
    /// rounded in the key LLM answers are reused under. Default 1; above 0.
    /// A market move that stays in its step (and on the same favourite) costs no
    /// call: the stored adjustment is applied to the fresh anchor.
    pub market_step_pp: Option<f64>,
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
pub const DEFAULT_MARKET_STEP_PP: f64 = 1.0;

struct Settings {
    mode: DebateMode,
    lopsided: f64,
    market_step_pp: f64,
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
            market_step_pp: own.market_step_pp.unwrap_or(DEFAULT_MARKET_STEP_PP),
            fast: own.fast.unwrap_or_else(|| turned(0)),
            advocate: own.advocate.unwrap_or_else(|| turned(0)),
            devil: own.devil.unwrap_or_else(|| turned(1)),
            arbiter: own.arbiter.unwrap_or_else(|| turned(2)),
        }
    }
}

/// The key an LLM answer is reused under. It covers what the dossier hash covers
/// (the Match with its format, the rating estimate, every context fact), but of
/// the market only the step it falls in and which team it favours: the precise
/// line moves every few minutes, and the model's adjustment does not depend on
/// a fraction of a point. The policy still anchors on the precise line.
fn reuse_key(dossier: &Dossier, step_pp: f64) -> Result<String> {
    let market = dossier.market.map(|p| {
        json!({
            "step_pp": step_pp,
            "bucket": (p * 100.0 / step_pp).round() as i64,
            "favourite": if p >= 0.5 { "team_a" } else { "team_b" },
        })
    });
    let inputs = json!({
        "game": dossier.game,
        "signal": dossier.signal,
        "market": market,
        "context": dossier.context,
    });
    Ok(crate::digest(&serde_json::to_vec(&inputs)?))
}

/// One run: who asks, about what, and the keys its turns are kept under.
struct Run<'a> {
    bot: &'a BotProfile,
    dossier: &'a Dossier,
    tier: Tier,
    /// The dossier hash, for the audit.
    hash: String,
    /// `reuse_key`, by which answers are found again.
    key: String,
}

pub struct TieredDebateLlm {
    providers: Arc<Providers>,
    config: DebateConfig,
    log: TurnLog,
}
impl TieredDebateLlm {
    /// The answer this seat already gave under this run's reuse key, if any.
    async fn answered(&self, run: &Run<'_>, prompt: &RolePrompt) -> Option<Value> {
        let found = self
            .log
            .answered(&run.bot.bot_key, run.tier, &run.key, &prompt.role)
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
            reuse_key: run.key.clone(),
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
            key: reuse_key(dossier, settings.market_step_pp)
                .map_err(|_| LlmError::InvalidOutput)?,
        };
        // What a factor of the answer may cite: this dossier is what the model is
        // sent, and the providers know whether this run may read the web.
        let web = self.providers.access(bot, tier) == ToolAccess::WhitelistedWeb;
        let evidence = Evidence::of(dossier, web);
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
        let predict = with_baseline(RolePrompt {
            role: roles::PREDICT.into(),
            ..prompt.clone()
        });
        let asked = with_baseline(prompt.clone());
        // The arbiter answers the caller's schema, whatever the briefs are.
        let verdict = roles::arbiter(&asked, Value::Null, Value::Null);
        let format = dossier.game.format;
        let pass = plan(
            settings.mode,
            tier,
            p_base,
            format,
            &flags,
            settings.lopsided,
        );
        // A run that was answered is not asked again. In `tiered` mode that holds
        // whichever pass answered: a line moving inside its step can carry `p_base`
        // across the mismatch threshold, and that is no reason for a new call.
        let finals: &[&RolePrompt] = match (settings.mode, pass) {
            (DebateMode::Always, _) => &[&verdict],
            (DebateMode::Never, _) => &[&predict],
            (DebateMode::Tiered, Pass::Fast) => &[&predict, &verdict],
            (DebateMode::Tiered, Pass::Debate) => &[&verdict, &predict],
        };
        for asked_before in finals {
            if let Some(answer) = self.answered(&run, asked_before).await {
                return Ok(Some(evidence.filter(answer)));
            }
        }
        let answer = match pass {
            Pass::Fast => self.ask(&run, &settings.fast, &predict).await?,
            Pass::Debate => {
                if settings.arbiter.is_empty() {
                    return Ok(None);
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
                let brief = |seat: Result<Option<Value>, LlmError>| {
                    seat.ok().flatten().unwrap_or(Value::Null)
                };
                let verdict = roles::arbiter(&asked, brief(advocate_brief), brief(devil_brief));
                self.ask(&run, &settings.arbiter, &verdict).await?
            }
        };
        // The audit keeps the model's answer as given; the policy gets only the
        // factors whose evidence this run could have read.
        Ok(answer.map(|answer| evidence.filter(answer)))
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
        if settings
            .market_step_pp
            .is_some_and(|step| !(step > 0.0 && step <= 50.0))
        {
            return Err(
                format!("debate.bots.{bot}.market_step_pp must be above 0 and at most 50").into(),
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
