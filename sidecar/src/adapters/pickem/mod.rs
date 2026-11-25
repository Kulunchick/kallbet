//! Ticket #28 owns this adapter family and its registration.
mod playoffs;
mod step;
mod swiss;

use crate::*;
use serde_json::Value;
use std::sync::Arc;

pub const DEFAULT_SIMULATIONS: usize = 25_000;
pub const DEFAULT_SEED: u64 = 0x51C0_2026_0028_CAFE;

fn check_no_mid_stage_leak(
    stage_val: &Value,
    cutoff: Instant,
) -> std::result::Result<(), TemporalLeakError> {
    let leak = || TemporalLeakError {
        observed_at: cutoff,
        cutoff,
    };
    let kind = stage_val["stage"]["kind"].as_str().unwrap_or("");

    let mut leaf_slots = std::collections::HashSet::new();
    if let Some(slots) = stage_val["bracket"]["slots"].as_array() {
        for s in slots {
            if !s["winner"].is_null() {
                return Err(leak());
            }
            let leaf = s["leaf"].as_bool().unwrap_or(false);
            if leaf {
                if let (Some(r), Some(slot)) =
                    (s["rounds_before_final"].as_u64(), s["slot"].as_u64())
                {
                    leaf_slots.insert((r, slot));
                }
            } else if !s["team_a"].is_null() || !s["team_b"].is_null() {
                return Err(leak());
            }
        }
    }

    if let Some(groups) = stage_val["groups"].as_array() {
        for g in groups {
            let wins = g["wins"].as_u64().unwrap_or(0);
            let losses = g["losses"].as_u64().unwrap_or(0);
            let rbf = g["rounds_before_final"].as_u64();
            let Some(matches) = g["matches"].as_array() else {
                continue;
            };
            for m in matches {
                if !m["winner"].is_null()
                    || !m["score"].is_null()
                    || !m["actual_start"].is_null()
                    || matches!(m["status"].as_str(), Some("live" | "finished"))
                {
                    return Err(leak());
                }
                let has_team = !m["team_a"].is_null() || !m["team_b"].is_null();
                if kind == "swiss" && (wins > 0 || losses > 0) && has_team {
                    return Err(leak());
                }
                if kind == "playoffs"
                    && has_team
                    && !leaf_slots.is_empty()
                    && let (Some(r), Some(s)) = (rbf, m["bracket_slot"].as_u64())
                    && !leaf_slots.contains(&(r, s))
                {
                    return Err(leak());
                }
            }
        }
    }
    Ok(())
}

pub struct MonteCarloPickemSolver {
    simulations: usize,
    rng_factory: Arc<dyn Fn() -> Box<dyn RandomSource> + Send + Sync>,
}

impl Default for MonteCarloPickemSolver {
    fn default() -> Self {
        Self::seeded(DEFAULT_SEED)
    }
}

impl MonteCarloPickemSolver {
    pub fn seeded(seed: u64) -> Self {
        Self::with_simulations(seed, DEFAULT_SIMULATIONS)
    }

    pub fn with_simulations(seed: u64, simulations: usize) -> Self {
        Self {
            simulations,
            rng_factory: Arc::new(move || Box::new(SeededRng::new(seed))),
        }
    }

    pub fn with_rng_factory(
        simulations: usize,
        rng_factory: Arc<dyn Fn() -> Box<dyn RandomSource> + Send + Sync>,
    ) -> Self {
        Self {
            simulations,
            rng_factory,
        }
    }

    pub fn solve_with_rng(
        &self,
        stage: &AsOf<Value>,
        ratings: &[AsOf<PairwiseProbability>],
        cutoff: Instant,
        rng: &mut dyn RandomSource,
    ) -> Result<Option<Value>> {
        let stage_val = stage.at(cutoff)?;
        for rating in ratings {
            rating.at(cutoff)?;
        }
        check_no_mid_stage_leak(stage_val, cutoff)?;
        if ratings.is_empty() {
            return Ok(None);
        }
        match stage_val["stage"]["kind"].as_str() {
            Some("swiss") => {
                swiss::solve_swiss(stage_val, ratings, cutoff, self.simulations, rng)
            }
            Some("playoffs") => playoffs::solve_playoffs(stage_val, ratings, cutoff),
            _ => Ok(None),
        }
    }
}

impl PickemSolver for MonteCarloPickemSolver {
    fn solve(
        &self,
        stage: &AsOf<Value>,
        ratings: &[AsOf<PairwiseProbability>],
        cutoff: Instant,
    ) -> Result<Option<Value>> {
        let mut rng = (self.rng_factory)();
        self.solve_with_rng(stage, ratings, cutoff, &mut *rng)
    }
}

pub use step::{SubmittedPickem, submitted_pickems};

pub fn new(
    clock: Arc<dyn Clock>,
    signal: Arc<dyn SignalEstimator>,
    odds: Arc<dyn OddsSource>,
) -> Arc<dyn PickemStep> {
    with_solver(clock, signal, odds, Arc::new(MonteCarloPickemSolver::default()))
}

pub fn with_solver(
    clock: Arc<dyn Clock>,
    signal: Arc<dyn SignalEstimator>,
    odds: Arc<dyn OddsSource>,
    solver: Arc<dyn PickemSolver>,
) -> Arc<dyn PickemStep> {
    Arc::new(step::PickemRunner::new(clock, signal, odds, solver))
}
