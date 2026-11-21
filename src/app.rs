//! The whole HTTP surface: the JSON API under `/api`, the SPA everywhere else.
//! This router is Seam 1: tests drive it in-process (`crate::testing`).

use axum::Router;
use axum::http::Request;
use tower_http::trace::TraceLayer;

use crate::state::AppState;
use crate::{api, spa};

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .nest("/api", api::router(&state.config))
        .fallback_service(spa::service(&state.config.web_dir))
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
                tracing::info_span!(
                    "request",
                    method = %request.method(),
                    path = %loggable_path(request.uri().path()),
                )
            }),
        )
        .with_state(state)
}

/// A request path as it may appear in logs: the secret after `invites/` or
/// `join/` (an Invite Link token) is replaced by `…`. The query is never
/// logged (it is not passed in), since it can carry such a path too
/// (`/login?next=/join/<token>`).
pub fn loggable_path(path: &str) -> String {
    let mut out = Vec::new();
    let mut secret_next = false;
    for segment in path.split('/') {
        if secret_next && !segment.is_empty() {
            out.push("…");
        } else {
            out.push(segment);
        }
        secret_next = matches!(segment, "invites" | "join");
    }
    out.join("/")
}

#[cfg(test)]
mod tests {
    use super::loggable_path;

    #[test]
    fn invite_tokens_are_not_logged() {
        assert_eq!(loggable_path("/api/invites/abc123/join"), "/api/invites/…/join");
        assert_eq!(loggable_path("/api/invites/abc123"), "/api/invites/…");
        assert_eq!(loggable_path("/join/abc123"), "/join/…");
        assert_eq!(loggable_path("/api/leagues/4/matches"), "/api/leagues/4/matches");
    }
}
