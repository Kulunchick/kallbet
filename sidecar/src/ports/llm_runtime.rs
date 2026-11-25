use crate::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, thiserror::Error, Clone)]
pub enum LlmError {
    #[error("provider quota exceeded")]
    QuotaExceeded,
    #[error("transient provider failure")]
    Transient,
    #[error("provider authentication failed")]
    Auth,
    #[error("invalid structured output")]
    InvalidOutput,
}
/// The pipeline asks for an adjustment. Debate seats can supply a role-specific schema.
pub struct RolePrompt {
    pub role: String,
    pub schema: Value,
}
impl RolePrompt {
    pub fn adjustment() -> Self {
        Self {
            role: "predict".into(),
            schema: serde_json::json!({"type":"object","properties":{"delta":{"type":"number"},"factors":{"type":"array"},"summary":{"type":"object"}},"required":["delta","factors","summary"]}),
        }
    }
}
#[derive(Clone, Serialize, Deserialize)]
pub struct LlmAdjustment {
    pub delta: f64,
    pub factors: Vec<Value>,
    pub summary: Value,
}

#[async_trait::async_trait]
pub trait LlmRuntime: Send + Sync {
    async fn evaluate(
        &self,
        bot: &BotProfile,
        dossier: &Dossier,
        tier: Tier,
        prompt: &RolePrompt,
    ) -> Result<Option<Value>, LlmError>;
}
