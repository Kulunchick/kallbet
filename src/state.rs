use std::sync::Arc;

use sqlx::SqlitePool;

use crate::config::Config;
use crate::ports::{Clock, MatchSource, Notifier};

/// Everything a handler or background task can reach: the database, the
/// three ports and the configuration. Cheap to clone.
#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub clock: Arc<dyn Clock>,
    pub source: Arc<dyn MatchSource>,
    pub notifier: Arc<dyn Notifier>,
    pub config: Arc<Config>,
}
