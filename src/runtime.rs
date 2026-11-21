//! Production wiring: which adapter stands behind each port, and which
//! background tasks run. `main` calls these; tests never do.
//!
//! One function per concern, so the tickets that replace a placeholder each
//! edit their own function.

use std::sync::Arc;

use crate::config::Config;
use crate::poller;
use crate::ports::{
    Clock, MatchSource, Notifier, SystemClock, TelegramNotifier, UnconfiguredSource,
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

/// The Telegram bot (`sendMessage`). Without `BOT_TOKEN` it sends nothing.
pub fn notifier(config: &Config) -> Arc<dyn Notifier> {
    Arc::new(TelegramNotifier::new(config.bot_token.clone()))
}

/// Starts the background tasks: the poller, one [`poller::tick`] every
/// [`poller::TICK_EVERY`]. Must be called inside the Tokio runtime.
pub fn spawn_background_tasks(state: &AppState) {
    let state = state.clone();
    tokio::spawn(async move {
        let mut every = tokio::time::interval(poller::TICK_EVERY);
        every.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        loop {
            every.tick().await;
            poller::tick(&state).await;
        }
    });
}
