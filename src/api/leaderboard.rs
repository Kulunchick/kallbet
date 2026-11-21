//! Leaderboard and Pick'em Leaderboard API.
//!
//! Stub reserved by the walking skeleton (#2) so that parallel tickets fill in
//! their own file instead of all editing the parent `mod.rs`. Owner: #7, #14.

use axum::Router;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
}
