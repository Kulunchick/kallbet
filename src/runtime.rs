//! Production wiring: which adapter stands behind each port, and which
//! background tasks run. `main` calls these; tests never do.
//!
//! One function per concern, so the tickets that replace a placeholder each
//! edit their own function.

use std::sync::Arc;

use crate::config::Config;
use crate::ports::{
    Clock, LogNotifier, MatchSource, Notifier, SystemClock, UnconfiguredSource,
};
use crate::state::AppState;

pub fn clock() -> Arc<dyn Clock> {
    Arc::new(SystemClock)
}

/// The Data Source chain. Placeholder until #9 wires the `FallbackSource`
/// (HLTV → PandaScore → Liquipedia).
pub fn match_source(_config: &Config) -> Arc<dyn MatchSource> {
    Arc::new(UnconfiguredSource)
}

/// The Telegram notifier. Placeholder until #16 wires the bot.
pub fn notifier(_config: &Config) -> Arc<dyn Notifier> {
    Arc::new(LogNotifier)
}

/// Starts the background tasks. Nothing yet: #5 spawns the poller here.
pub fn spawn_background_tasks(_state: &AppState) {}
