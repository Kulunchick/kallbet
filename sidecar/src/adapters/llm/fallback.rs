//! `FallbackLlm`: a Chain of Responsibility over providers, each behind its own
//! cool-down circuit breaker. A breaker is per provider, not per chain, so a
//! Claude quota seen by one bot or debate seat skips Claude for every chain.
use crate::*;
use async_trait::async_trait;
use serde_json::Value;
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BreakerState {
    Closed,
    /// Skipped until `until`, then one trial request (half-open).
    Open {
        until: Instant,
        cause: LlmError,
    },
    /// One trial request is running; others skip the provider meanwhile.
    HalfOpen,
}

/// Opens on `QuotaExceeded`, `Transient` and `Timeout` for `cooldown`. After the
/// cool-down one request goes through: success closes it, another such failure
/// opens it for a new cool-down. Any answer (also `InvalidOutput` or `Auth`)
/// proves the provider is up and closes it.
pub struct Breaker {
    state: Mutex<BreakerState>,
    cooldown: chrono::Duration,
    clock: Arc<dyn Clock>,
}
impl Breaker {
    pub fn new(cooldown: chrono::Duration, clock: Arc<dyn Clock>) -> Self {
        Self {
            state: Mutex::new(BreakerState::Closed),
            cooldown,
            clock,
        }
    }
    pub fn state(&self) -> BreakerState {
        *self.state.lock().unwrap()
    }
    /// `Err(cause)` while the provider must be skipped. `Ok(true)`: this
    /// request is the half-open trial.
    fn admit(&self) -> Result<bool, LlmError> {
        let mut state = self.state.lock().unwrap();
        match *state {
            BreakerState::Closed => Ok(false),
            BreakerState::Open { until, cause } if self.clock.now() < until => Err(cause),
            BreakerState::Open { .. } => {
                *state = BreakerState::HalfOpen;
                Ok(true)
            }
            BreakerState::HalfOpen => Err(LlmError::Transient),
        }
    }
    /// The trial was dropped before it reported (its tick timed out or the
    /// process is shutting down): open again for a new cool-down, as a timeout,
    /// so the provider is tried after it instead of staying half-open.
    fn abandon(&self) {
        let mut state = self.state.lock().unwrap();
        if *state == BreakerState::HalfOpen {
            *state = BreakerState::Open {
                until: self.clock.now() + self.cooldown,
                cause: LlmError::Timeout,
            };
        }
    }
    fn record(&self, outcome: Result<(), LlmError>) {
        let mut state = self.state.lock().unwrap();
        *state = match outcome {
            Err(cause @ (LlmError::QuotaExceeded | LlmError::Transient | LlmError::Timeout)) => {
                BreakerState::Open {
                    until: self.clock.now() + self.cooldown,
                    cause,
                }
            }
            _ => BreakerState::Closed,
        };
    }
}

/// Reopens the breaker if a half-open trial is dropped before it reports.
struct Trial<'a> {
    breaker: &'a Breaker,
    armed: bool,
}
impl Drop for Trial<'_> {
    fn drop(&mut self) {
        if self.armed {
            self.breaker.abandon();
        }
    }
}

/// A named provider with its breaker, shared by every chain that lists it.
pub struct Provider {
    pub name: String,
    pub runtime: Arc<dyn LlmRuntime>,
    pub breaker: Breaker,
}

/// The answer of a chain and who gave it (for the debate audit of #29).
#[derive(Clone, Debug)]
pub struct Answer {
    pub provider: String,
    pub value: Value,
    pub elapsed: std::time::Duration,
}

pub struct FallbackLlm {
    links: Vec<Arc<Provider>>,
}
impl FallbackLlm {
    pub fn new(links: Vec<Arc<Provider>>) -> Self {
        Self { links }
    }
    /// Names of the providers, in the order they are tried.
    pub fn names(&self) -> Vec<&str> {
        self.links.iter().map(|l| l.name.as_str()).collect()
    }
    /// Tries each provider in order until one answers. `Ok(None)` for an empty
    /// chain. Errors: `NotConfigured` when every provider is unconfigured, else the
    /// last failure (or the cause of a cool-down) met on the way.
    pub async fn answer(
        &self,
        bot: &BotProfile,
        dossier: &Dossier,
        tier: Tier,
        prompt: &RolePrompt,
    ) -> Result<Option<Answer>, LlmError> {
        if self.links.is_empty() {
            return Ok(None);
        }
        let mut last = LlmError::NotConfigured;
        for link in &self.links {
            let mut trial = match link.breaker.admit() {
                Ok(armed) => Trial {
                    breaker: &link.breaker,
                    armed,
                },
                Err(cause) => {
                    tracing::info!(provider = %link.name, cause = %cause, "llm: provider is cooling down; skipped");
                    last = cause;
                    continue;
                }
            };
            let started = std::time::Instant::now();
            let outcome = link.runtime.evaluate(bot, dossier, tier, prompt).await;
            // It reports below, so the guard has nothing left to do.
            trial.armed = false;
            match outcome {
                Ok(Some(value)) => {
                    link.breaker.record(Ok(()));
                    return Ok(Some(Answer {
                        provider: link.name.clone(),
                        value,
                        elapsed: started.elapsed(),
                    }));
                }
                Ok(None) => {
                    link.breaker.record(Ok(()));
                    return Ok(None);
                }
                Err(e) => {
                    link.breaker.record(Err(e));
                    if e != LlmError::NotConfigured {
                        tracing::warn!(provider = %link.name, error = %e, "llm: provider failed; trying the next one");
                        last = e;
                    }
                }
            }
        }
        Err(last)
    }
}

#[async_trait]
impl LlmRuntime for FallbackLlm {
    async fn evaluate(
        &self,
        bot: &BotProfile,
        dossier: &Dossier,
        tier: Tier,
        prompt: &RolePrompt,
    ) -> Result<Option<Value>, LlmError> {
        Ok(self
            .answer(bot, dossier, tier, prompt)
            .await?
            .map(|a| a.value))
    }
}
