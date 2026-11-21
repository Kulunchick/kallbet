//! Events API: list Events, resolve an Event by HLTV link.
//!
//! Stub reserved by the walking skeleton (#2) so that parallel tickets fill in
//! their own file instead of all editing the parent `mod.rs`. Owner: #4.

use axum::Router;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
}
