//! The whole HTTP surface: the JSON API under `/api`, the SPA everywhere else.
//! This router is Seam 1: tests drive it in-process (`crate::testing`).

use axum::Router;
use tower_http::trace::TraceLayer;

use crate::state::AppState;
use crate::{api, spa};

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .nest("/api", api::router(&state.config))
        .fallback_service(spa::service(&state.config.web_dir))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
