//! Events API: list Events, resolve an Event by HLTV link.
//! Both go through the `MatchSource` port.

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;

use crate::auth::session::CurrentUser;
use crate::domain::league::parse_hltv_event_link;
use crate::domain::{Event, EventId, EventSnapshot};
use crate::error::{ApiError, ApiResult};
use crate::ports::SourceError;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/events", get(list))
        .route("/events/resolve", post(resolve))
}

/// A Data Source failure as the API reports it: 502 `source_unavailable`,
/// except an Event the source does not know, which is 422 `event_not_found`.
pub(crate) fn source_error(error: SourceError) -> ApiError {
    match error {
        SourceError::NotFound => ApiError::unprocessable("event_not_found"),
        other => {
            tracing::warn!("match source failed: {other}");
            ApiError::new(StatusCode::BAD_GATEWAY, "source_unavailable")
        }
    }
}

/// Fetches one Event's snapshot through the port.
pub(crate) async fn fetch(state: &AppState, id: EventId) -> ApiResult<EventSnapshot> {
    state.source.fetch_event(id).await.map_err(source_error)
}

async fn list(
    State(state): State<AppState>,
    CurrentUser(_user): CurrentUser,
) -> ApiResult<Json<Vec<Event>>> {
    let events = state.source.list_events().await.map_err(source_error)?;
    Ok(Json(events))
}

#[derive(Deserialize)]
struct Resolve {
    link: String,
}

async fn resolve(
    State(state): State<AppState>,
    CurrentUser(_user): CurrentUser,
    Json(body): Json<Resolve>,
) -> ApiResult<Json<Event>> {
    let id =
        parse_hltv_event_link(&body.link).ok_or(ApiError::unprocessable("invalid_hltv_link"))?;
    Ok(Json(fetch(&state, id).await?.event))
}
