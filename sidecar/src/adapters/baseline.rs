use crate::*;
use async_trait::async_trait;
use serde_json::{Value, json};
pub struct Noop;
#[async_trait]
impl OddsSource for Noop {
    async fn quote(&self, _: &Match, _: Instant) -> Result<Option<AsOf<f64>>> {
        Ok(None)
    }
}
#[async_trait]
impl ContextEnricher for Noop {
    async fn facts(&self, _: &Match, _: Instant) -> Result<Vec<AsOf<Value>>> {
        Ok(vec![])
    }
}
#[async_trait]
impl LlmRuntime for Noop {
    async fn evaluate(
        &self,
        _: &BotProfile,
        _: &Dossier,
        _: Tier,
        _: &RolePrompt,
    ) -> Result<Option<Value>, LlmError> {
        Ok(None)
    }
}
#[async_trait]
impl PickemStep for Noop {
    async fn tick(
        &self,
        _: &dyn LeaguePlatform,
        _: &str,
        _: &BotProfile,
        _: i64,
        _: Instant,
        _: &Audit,
    ) -> Result<()> {
        Ok(())
    }
}
#[async_trait]
impl SignalEstimator for vrs::Standings {
    async fn signal(&self, m: &Match, cutoff: Instant) -> Result<Option<AsOf<Signal>>> {
        self.estimate(&m.team_a.name, &m.team_b.name, m.format, cutoff)
    }
}
pub struct SystemClock;
impl Clock for SystemClock {
    fn now(&self) -> Instant {
        chrono::Utc::now()
    }
}
pub struct VrsOnly;
impl DecisionPolicy for VrsOnly {
    fn decide(
        &self,
        _: &BotProfile,
        d: &Dossier,
        tier: Tier,
        _: Option<LlmAdjustment>,
    ) -> Result<Option<Prediction>> {
        let Some(s) = &d.signal else { return Ok(None) };
        let (winner, p) = if s.series >= 0.5 {
            (&d.game.team_a, s.series)
        } else {
            (&d.game.team_b, 1. - s.series)
        };
        Ok(Some(Prediction {
            team_id: winner.id,
            rationale: json!({
                "p_win": p,
                "override_market": false,
                "preliminary": tier == Tier::Draft,
                "summary": {
                    "en": "Prediction based on the Valve Regional Standings.",
                    "ru": "Прогноз по рейтингу Valve.",
                    "uk": "Прогноз за рейтингом Valve."
                },
                "factors": []
            }),
        }))
    }
}
