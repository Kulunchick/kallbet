//! The one error type API handlers return.
//!
//! Every error response is `{"error": {"code": "<snake_case_code>"}}`. The
//! code is stable API: the SPA shows the localised text for
//! `errors.<code>` (namespace `common`, or the area's own namespace), so the
//! server never sends human-readable error text.

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiError {
    pub status: StatusCode,
    pub code: &'static str,
}

pub type ApiResult<T> = Result<T, ApiError>;

impl ApiError {
    pub const fn new(status: StatusCode, code: &'static str) -> ApiError {
        ApiError { status, code }
    }

    /// 400 with a specific code, e.g. `invalid_hltv_link`.
    pub const fn bad_request(code: &'static str) -> ApiError {
        ApiError::new(StatusCode::BAD_REQUEST, code)
    }

    /// 401: no valid session.
    pub const fn unauthorized() -> ApiError {
        ApiError::new(StatusCode::UNAUTHORIZED, "unauthorized")
    }

    /// 401 with a specific code, e.g. `invalid_login`.
    pub const fn unauthorized_with(code: &'static str) -> ApiError {
        ApiError::new(StatusCode::UNAUTHORIZED, code)
    }

    /// 403: signed in, but not allowed.
    pub const fn forbidden() -> ApiError {
        ApiError::new(StatusCode::FORBIDDEN, "forbidden")
    }

    /// 404. Also the answer for things the User may not know exist.
    pub const fn not_found() -> ApiError {
        ApiError::new(StatusCode::NOT_FOUND, "not_found")
    }

    /// 409 with a specific code, e.g. `nickname_taken`, `match_locked`.
    pub const fn conflict(code: &'static str) -> ApiError {
        ApiError::new(StatusCode::CONFLICT, code)
    }

    /// 422 with a specific code, e.g. `nickname_invalid`.
    pub const fn unprocessable(code: &'static str) -> ApiError {
        ApiError::new(StatusCode::UNPROCESSABLE_ENTITY, code)
    }

    /// 500. Logs the cause; the client only sees `internal`.
    pub fn internal(cause: impl std::fmt::Display) -> ApiError {
        tracing::error!("internal error: {cause}");
        ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, "internal")
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(json!({ "error": { "code": self.code } }))).into_response()
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(e: sqlx::Error) -> ApiError {
        ApiError::internal(e)
    }
}
