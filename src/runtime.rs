//! Production wiring: which adapter stands behind each port, and which
//! background tasks run. `main` calls these; tests never do.
//!
//! One function per concern, so the tickets that replace a placeholder each
//! edit their own function.

use std::sync::Arc;

use crate::config::Config;
use crate::poller;
use crate::ports::{Clock, MatchSource, Notifier, SystemClock, TelegramNotifier};
use crate::sources::fallback::{FallbackSettings, FallbackSource};
use crate::sources::hltv::{ChromePages, HltvSource, LogoCache};
use crate::sources::liquipedia::LiquipediaSource;
use crate::sources::pandascore::PandaScoreSource;
use crate::state::AppState;
use sqlx::SqlitePool;

pub fn clock() -> Arc<dyn Clock> {
    Arc::new(SystemClock)
}

/// The Data Source chain (#9): HLTV -> PandaScore -> Liquipedia behind a
/// `FallbackSource`. HLTV is left out when `KALLBET_CHROME_URL` is unset;
/// PandaScore (#10) and Liquipedia (#11) report `NotConfigured` themselves
/// and are skipped. An HLTV fetch loads one page at a time and can take about
/// a minute for a Major; its breaker keeps a failing HLTV from costing that
/// on every poll.
pub fn match_source(
    config: &Config,
    db: &SqlitePool,
    clock: Arc<dyn Clock>,
) -> Arc<dyn MatchSource> {
    let mut chain = FallbackSource::new(
        db.clone(),
        clock.clone(),
        FallbackSettings::from_config(config),
    );
    match &config.chrome_url {
        Some(url) => {
            let hltv = HltvSource::new(Arc::new(ChromePages::new(url.clone())), clock)
                .with_logos(LogoCache::new(config.logo_dir.clone()));
            chain = chain.with_primary(Arc::new(hltv));
        }
        None => tracing::warn!("KALLBET_CHROME_URL is unset: the HLTV Data Source is skipped"),
    }
    Arc::new(
        chain
            .with_fallback(Arc::new(PandaScoreSource::new(
                config.pandascore_token.clone(),
            )))
            .with_fallback(Arc::new(LiquipediaSource::new())),
    )
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
