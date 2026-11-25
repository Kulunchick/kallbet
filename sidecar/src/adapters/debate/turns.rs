//! The `debate_turns` audit table: one append-only row per seat that ran, with
//! the provider that answered, how long it took, and its brief. A later run of
//! the same bot, tier and dossier reads its answers from here instead of asking
//! the subscriptions again.
use crate::*;
use serde_json::Value;
use sqlx::SqlitePool;

/// A recorded seat of a run, as the audit reader returns it.
#[derive(Debug, Clone, PartialEq)]
pub struct DebateTurn {
    pub bot: String,
    pub match_id: i64,
    /// `Draft` or `PreLock`.
    pub tier: String,
    /// The dossier hash of the run that asked (`dossier_runs.input_hash`).
    pub input_hash: String,
    /// What the answer is reused under: the dossier with the market reduced to
    /// its bucket and favourite. Empty for turns recorded before it existed.
    pub reuse_key: String,
    /// `advocate`, `devil`, `arbiter`, or `predict` for the single fast agent.
    pub role: String,
    /// The provider that answered; for a failed seat, the chain that was tried.
    pub provider: String,
    pub latency_ms: i64,
    /// `ok`, or the kind of failure the seat's whole chain ended with.
    pub status: String,
    /// The seat's structured answer; `None` for a failed seat.
    pub brief: Option<Value>,
}

type Row = (
    String,
    i64,
    String,
    String,
    String,
    String,
    String,
    i64,
    String,
    Option<String>,
);

impl Audit {
    /// Every recorded turn, oldest first.
    pub async fn debate_turns(&self) -> Result<Vec<DebateTurn>> {
        let rows: Vec<Row> = sqlx::query_as(
            "SELECT bot, match_id, tier, input_hash, reuse_key, role, provider, latency_ms, status, brief
             FROM debate_turns ORDER BY id",
        )
        .fetch_all(&self.db)
        .await?;
        rows.into_iter()
            .map(|r| {
                Ok(DebateTurn {
                    bot: r.0,
                    match_id: r.1,
                    tier: r.2,
                    input_hash: r.3,
                    reuse_key: r.4,
                    role: r.5,
                    provider: r.6,
                    latency_ms: r.7,
                    status: r.8,
                    brief: r.9.as_deref().map(serde_json::from_str).transpose()?,
                })
            })
            .collect()
    }
}

pub(super) fn status(error: LlmError) -> &'static str {
    match error {
        LlmError::QuotaExceeded => "quota_exceeded",
        LlmError::Transient => "transient",
        LlmError::Timeout => "timeout",
        LlmError::Auth => "auth",
        LlmError::InvalidOutput => "invalid_output",
        LlmError::NotConfigured => "not_configured",
    }
}

pub(super) struct TurnLog {
    pub db: SqlitePool,
}
impl TurnLog {
    /// The latest answer a seat gave under this reuse key, if it gave one,
    /// and the provider that gave it.
    pub async fn answered(
        &self,
        bot: &str,
        tier: Tier,
        key: &str,
        role: &str,
    ) -> Result<Option<(String, Value)>> {
        let row: Option<(String, Option<String>)> = sqlx::query_as(
            "SELECT provider, brief FROM debate_turns
             WHERE bot=? AND tier=? AND reuse_key=? AND role=? AND status='ok'
             ORDER BY id DESC LIMIT 1",
        )
        .bind(bot)
        .bind(tier.name())
        .bind(key)
        .bind(role)
        .fetch_optional(&self.db)
        .await?;
        let Some((provider, Some(brief))) = row else {
            return Ok(None);
        };
        Ok(Some((provider, serde_json::from_str(&brief)?)))
    }
    pub async fn record(&self, turn: &DebateTurn) -> Result<()> {
        sqlx::query(
            "INSERT INTO debate_turns(bot,match_id,tier,input_hash,reuse_key,role,provider,latency_ms,status,brief)
             VALUES(?,?,?,?,?,?,?,?,?,?)",
        )
        .bind(&turn.bot)
        .bind(turn.match_id)
        .bind(&turn.tier)
        .bind(&turn.input_hash)
        .bind(&turn.reuse_key)
        .bind(&turn.role)
        .bind(&turn.provider)
        .bind(turn.latency_ms)
        .bind(&turn.status)
        .bind(turn.brief.as_ref().map(Value::to_string))
        .execute(&self.db)
        .await?;
        Ok(())
    }
}
