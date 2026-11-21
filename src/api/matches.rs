//! Matches API: a League's effective Matches grouped by Stage and round, with Lock/Reveal state.
//!
//! Stub reserved by the walking skeleton (#2) so that parallel tickets fill in
//! their own file instead of all editing the parent `mod.rs`. Owner: #5, #6.

use axum::Router;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
}
