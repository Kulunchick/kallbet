//! The League Platform port in backtest mode: one virtual League that shows the
//! bots, step by step, exactly what the live server would have shown at that
//! instant (a Match due for its pre-Hard-Lock prediction, or a Stage's Pick'em
//! before its first Match), and records what they submit.
use super::archive::ReplayStage;
use crate::*;
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Mutex,
};

/// The replay's virtual clock, shared by the scheduler, the Pick'em step and
/// the LLM breakers. It moves only when the runner sets it.
pub struct ReplayClock(Mutex<Instant>);
impl ReplayClock {
    pub fn new(at: Instant) -> Self {
        Self(Mutex::new(at))
    }
    pub fn set(&self, at: Instant) {
        *self.0.lock().expect("replay clock") = at;
    }
}
impl Clock for ReplayClock {
    fn now(&self) -> Instant {
        *self.0.lock().expect("replay clock")
    }
}

/// What the League shows at the current step.
#[derive(Clone, Default)]
pub enum Due {
    #[default]
    Nothing,
    /// Matches whose prediction step this is (both teams known, not yet locked).
    Matches(Vec<Match>),
    /// The open Pick'em of this Stage (index into the platform's Stages).
    Stage(usize),
}

#[derive(Clone)]
pub struct Submitted {
    pub bot: String,
    pub match_id: i64,
    pub prediction: Prediction,
}
#[derive(Clone)]
pub struct SubmittedPickem {
    pub bot: String,
    pub stage_id: i64,
    pub payload: Value,
}

pub const LEAGUE: i64 = 1;

pub struct ReplayPlatform {
    stages: Vec<ReplayStage>,
    due: Mutex<Due>,
    /// Bots limited to some Matches (the budgeted LLM track); others see all.
    scope: BTreeMap<String, BTreeSet<i64>>,
    predictions: Mutex<Vec<Submitted>>,
    pickems: Mutex<Vec<SubmittedPickem>>,
}
impl ReplayPlatform {
    pub fn new(stages: Vec<ReplayStage>, scope: BTreeMap<String, BTreeSet<i64>>) -> Self {
        Self {
            stages,
            due: Mutex::default(),
            scope,
            predictions: Mutex::default(),
            pickems: Mutex::default(),
        }
    }
    pub fn show(&self, due: Due) {
        *self.due.lock().expect("replay platform") = due;
    }
    pub fn predictions(&self) -> Vec<Submitted> {
        self.predictions.lock().expect("replay platform").clone()
    }
    pub fn pickems(&self) -> Vec<SubmittedPickem> {
        self.pickems.lock().expect("replay platform").clone()
    }
    fn due(&self) -> Due {
        self.due.lock().expect("replay platform").clone()
    }
}

#[async_trait::async_trait]
impl LeaguePlatform for ReplayPlatform {
    async fn login(&self, bot: &BotProfile) -> Result<String> {
        Ok(bot.bot_key.clone())
    }
    async fn join(&self, _: &str, _: &str) -> Result<()> {
        Ok(())
    }
    async fn leagues(&self, _: &str) -> Result<Vec<i64>> {
        Ok(vec![LEAGUE])
    }
    async fn matches(&self, token: &str, _: i64) -> Result<Vec<(Match, bool)>> {
        let Due::Matches(games) = self.due() else {
            return Ok(vec![]);
        };
        let scope = self.scope.get(token);
        Ok(games
            .into_iter()
            .filter(|g| scope.is_none_or(|ids| ids.contains(&g.id)))
            .map(|g| (g, false))
            .collect())
    }
    async fn submit(&self, token: &str, _: i64, game: i64, pick: &Prediction) -> Result<()> {
        self.predictions
            .lock()
            .expect("replay platform")
            .push(Submitted {
                bot: token.to_owned(),
                match_id: game,
                prediction: pick.clone(),
            });
        Ok(())
    }
    async fn pickem_stages(&self, _: &str, _: i64) -> Result<Vec<StagePickem>> {
        let Due::Stage(index) = self.due() else {
            return Ok(vec![]);
        };
        let stage = &self.stages[index];
        let Some(opening) = &stage.opening else {
            return Ok(vec![]);
        };
        // The view is a Liquipedia revision: it must predate the Stage's first Match.
        let view = AsOf::new(opening.value.clone(), opening.observed_at, stage.cutoff)?;
        let mut view = view.at(stage.cutoff)?.clone();
        view["stage"]["ordinal"] = (index as u64).into();
        let teams = view["teams"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|t| {
                Some(Team {
                    id: t["id"].as_i64()?,
                    name: t["name"].as_str()?.to_owned(),
                })
            })
            .collect();
        Ok(vec![StagePickem {
            event_id: stage.event_id,
            ordinal: index as u8,
            locks_at: Some(stage.cutoff),
            locked: false,
            can_edit: true,
            my_updated_at: None,
            teams,
            opening_matches: stage.opening_matches.clone(),
            stage: view,
        }])
    }
    async fn submit_pickem(
        &self,
        token: &str,
        _: i64,
        ordinal: u8,
        payload: &Value,
    ) -> Result<bool> {
        let stage_id = self.stages[usize::from(ordinal)].id;
        self.pickems
            .lock()
            .expect("replay platform")
            .push(SubmittedPickem {
                bot: token.to_owned(),
                stage_id,
                payload: payload.clone(),
            });
        Ok(true)
    }
}
