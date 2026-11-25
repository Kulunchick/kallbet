//! The LLM Runtime port (ticket #27). Every leaf provider (the `claude`, `codex`
//! and `agy` CLIs, the OpenAI/Anthropic-compatible HTTP adapter), the
//! `FallbackLlm` chain and the debate composites (#29) implement it.
use crate::*;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// Why a runtime gave no answer. `FallbackLlm` fails over on every kind; it opens
/// the provider's circuit breaker on `QuotaExceeded`, `Transient` and `Timeout`.
#[derive(Debug, thiserror::Error, Clone, Copy, PartialEq, Eq)]
pub enum LlmError {
    /// Rate limit, session or weekly subscription quota, HTTP 429.
    #[error("provider quota exceeded")]
    QuotaExceeded,
    /// Provider error that may pass on its own (5xx, overload, network).
    #[error("transient provider failure")]
    Transient,
    /// The run did not finish within the provider's time limit; the process was killed.
    #[error("provider timed out")]
    Timeout,
    /// Signed out, missing or rejected credentials.
    #[error("provider authentication failed")]
    Auth,
    /// No JSON matching the schema came back, or the provider refused the schema.
    #[error("invalid structured output")]
    InvalidOutput,
    /// The provider is not configured here: unknown name, missing binary, no endpoint.
    #[error("provider not configured")]
    NotConfigured,
}

/// What one LLM call is asked to do: the seat's role, its instructions, the JSON
/// Schema its answer must match, and labelled JSON inputs besides the dossier
/// (for example the two briefs the Chief Arbiter weighs).
#[derive(Clone)]
pub struct RolePrompt {
    pub role: String,
    pub instructions: String,
    pub schema: Value,
    pub attachments: Vec<(String, Value)>,
}
impl RolePrompt {
    /// The single-agent pass the scheduler runs: an `LlmAdjustment`.
    pub fn adjustment() -> Self {
        Self {
            role: "predict".into(),
            instructions: "Estimate how verified late-breaking factors in the dossier should move \
                the probability that team_a wins the series. `delta` is in percentage points for \
                team_a (positive favours team_a). Use only factors the dossier supports, each with \
                its evidence URL; with none, answer delta 0 and no factors. Write a 2-4 sentence \
                summary in English (en), Russian (ru) and Ukrainian (uk)."
                .into(),
            schema: adjustment_schema(),
            attachments: vec![],
        }
    }
}
/// JSON Schema of `LlmAdjustment`, in the strict subset every provider accepts.
pub fn adjustment_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "delta": {"type": "number"},
            "factors": {"type": "array", "items": {
                "type": "object",
                "properties": {
                    "kind": {"type": "string", "enum": [
                        "stand_in", "roster_change_30d", "igl_or_coach_change",
                        "visa_or_travel", "illness_or_emergency", "technical_or_dq"
                    ]},
                    "evidence_url": {"type": "string"}
                },
                "required": ["kind", "evidence_url"],
                "additionalProperties": false
            }},
            "summary": {
                "type": "object",
                "properties": {
                    "en": {"type": "string"},
                    "ru": {"type": "string"},
                    "uk": {"type": "string"}
                },
                "required": ["en", "ru", "uk"],
                "additionalProperties": false
            }
        },
        "required": ["delta", "factors", "summary"],
        "additionalProperties": false
    })
}
#[derive(Clone, Serialize, Deserialize)]
pub struct LlmAdjustment {
    pub delta: f64,
    pub factors: Vec<Value>,
    pub summary: Value,
}

#[async_trait::async_trait]
pub trait LlmRuntime: Send + Sync {
    /// `Ok(Some(v))`: `v` matches `prompt.schema`. `Ok(None)`: this bot uses no LLM
    /// (empty provider chain). The runtime decides tool access from the bot, the
    /// tier and the mode it was built for.
    async fn evaluate(
        &self,
        bot: &BotProfile,
        dossier: &Dossier,
        tier: Tier,
        prompt: &RolePrompt,
    ) -> Result<Option<Value>, LlmError>;
}
