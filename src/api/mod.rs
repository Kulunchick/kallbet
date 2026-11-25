//! The JSON API, mounted under `/api`. One file per area; each exposes
//! `router()` with its full paths (relative to `/api`), merged here.
//!
//! Every area in the spec's API outline already has a file, so a ticket
//! normally fills in its area's `router()` and does not edit this one.

use axum::Router;

use crate::config::Config;
use crate::error::ApiError;
use crate::state::AppState;

pub mod auth;
pub mod bot_auth;
pub mod dev_login;
pub mod events;
pub mod health;
pub mod leaderboard;
pub mod leagues;
pub mod matches;
pub mod me;
pub mod overrides;
pub mod pickem;
pub mod predictions;
pub mod teams;

pub fn router(config: &Config) -> Router<AppState> {
    let mut router = Router::new()
        .merge(auth::router())
        .merge(bot_auth::router())
        .merge(events::router())
        .merge(health::router())
        .merge(leaderboard::router())
        .merge(leagues::router())
        .merge(matches::router())
        .merge(me::router())
        .merge(overrides::router())
        .merge(pickem::router())
        .merge(predictions::router())
        .merge(teams::router());

    // The dev-only login bypass is not even routed unless explicitly enabled.
    if config.dev_login {
        router = router.merge(dev_login::router());
    }

    router.fallback(|| async { ApiError::not_found() })
}
