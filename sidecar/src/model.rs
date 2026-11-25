use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
pub type Instant = DateTime<Utc>;
pub type Result<T, E = Box<dyn std::error::Error + Send + Sync>> = std::result::Result<T, E>;
#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    Bo1,
    Bo3,
    Bo5,
}
#[derive(Clone, Deserialize, Serialize)]
pub struct Team {
    pub id: i64,
    pub name: String,
}
#[derive(Clone, Deserialize, Serialize)]
pub struct Match {
    pub event_id: i64,
    pub id: i64,
    pub team_a: Team,
    pub team_b: Team,
    pub format: Format,
    pub scheduled_start: Option<Instant>,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Signal {
    pub map: f64,
    pub series: f64,
}
#[derive(Clone, Serialize)]
pub struct Dossier {
    pub game: Match,
    pub signal: Option<Signal>,
    pub market: Option<f64>,
    pub context: Vec<Value>,
}
#[derive(Clone, Copy, PartialEq, Serialize)]
pub enum Tier {
    Draft,
    PreLock,
}
impl Tier {
    pub fn name(self) -> &'static str {
        match self {
            Self::Draft => "Draft",
            Self::PreLock => "PreLock",
        }
    }
}
#[derive(Serialize, Deserialize, Clone)]
pub struct Prediction {
    pub team_id: i64,
    pub rationale: Value,
}
