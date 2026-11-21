//! Me API: the signed-in User's own profile.
//!
//! #3 reads it and edits the Nickname and the language; #16 adds the
//! notification preferences (`notifications: {reminder, reveal, digest}`).

use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;

use crate::auth::CurrentUser;
use crate::domain::user::validate_nickname;
use crate::domain::{Language, User};
use crate::error::{ApiError, ApiResult};
use crate::state::AppState;
use crate::store::users::{self, NotificationPrefsChange};

pub fn router() -> Router<AppState> {
    Router::new().route("/me", get(get_me).patch(update_me))
}

async fn get_me(CurrentUser(user): CurrentUser) -> Json<User> {
    Json(user)
}

#[derive(Deserialize)]
struct UpdateMe {
    nickname: Option<String>,
    language: Option<String>,
    /// Partial: switches left out keep their value.
    notifications: Option<NotificationPrefsChange>,
}

async fn update_me(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(body): Json<UpdateMe>,
) -> ApiResult<Json<User>> {
    let nickname = body
        .nickname
        .as_deref()
        .map(|n| validate_nickname(n).map_err(|_| ApiError::unprocessable("nickname_invalid")))
        .transpose()?;
    let language = body
        .language
        .as_deref()
        .map(|l| Language::from_code(l).ok_or(ApiError::unprocessable("language_invalid")))
        .transpose()?;

    match users::update_profile(&state.db, user.id, nickname.as_deref(), language).await {
        Ok(updated) => match body.notifications {
            Some(change) => Ok(Json(
                users::update_notification_prefs(&state.db, user.id, &change).await?,
            )),
            None => Ok(Json(updated)),
        },
        Err(e) if users::is_unique_violation(&e) => Err(ApiError::conflict("nickname_taken")),
        Err(e) => Err(e.into()),
    }
}
