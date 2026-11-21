//! Me API: the signed-in User's own profile.
//!
//! The skeleton (#2) only reads it. #3 adds editing the Nickname and the
//! language; #16 adds the notification preferences.

use axum::routing::get;
use axum::{Json, Router};

use crate::auth::CurrentUser;
use crate::domain::User;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/me", get(get_me))
}

async fn get_me(CurrentUser(user): CurrentUser) -> Json<User> {
    Json(user)
}
