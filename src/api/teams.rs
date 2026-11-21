//! Teams API: team logos, served from our side (never hotlinked). The HLTV
//! adapter downloads them once (`crate::sources::hltv::LogoCache`).

use axum::Router;
use axum::extract::{Path, State};
use axum::http::header;
use axum::response::{IntoResponse, Response};
use axum::routing::get;

use crate::domain::TeamId;
use crate::error::{ApiError, ApiResult};
use crate::sources::hltv::logos;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/teams/{id}/logo", get(logo))
}

async fn logo(State(state): State<AppState>, Path(id): Path<i64>) -> ApiResult<Response> {
    let (bytes, content_type) = logos::stored(&state.config.logo_dir, TeamId(id))
        .await
        .ok_or(ApiError::not_found())?;
    Ok((
        [
            (header::CONTENT_TYPE, content_type),
            (header::CACHE_CONTROL, "public, max-age=86400"),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
            // An SVG from a Data Source must not run scripts on our origin.
            (header::CONTENT_SECURITY_POLICY, "default-src 'none'; style-src 'unsafe-inline'"),
        ],
        bytes,
    )
        .into_response())
}
