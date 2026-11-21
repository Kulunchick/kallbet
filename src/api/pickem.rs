//! Pick'em API: get and upsert a Member's Pick'em per Stage.
//!
//! Stub reserved by the walking skeleton (#2) so that parallel tickets fill in
//! their own file instead of all editing the parent `mod.rs`. Owner: #14, #15.

use axum::Router;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
}
