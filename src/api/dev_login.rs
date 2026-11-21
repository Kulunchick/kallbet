//! Dev-only login bypass (ADR-0004): sign in as any Nickname without
//! Telegram. Only routed when `KALLBET_DEV_LOGIN=true` (see `api::router`);
//! the Seam 1 harness uses it to act as a signed-in User.

use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use axum_extra::extract::CookieJar;
use serde::Deserialize;

use crate::auth::session;
use crate::domain::{Language, User};
use crate::error::{ApiError, ApiResult};
use crate::state::AppState;
use crate::store::users::{self, NewUser};

pub fn router() -> Router<AppState> {
    Router::new().route("/dev/login", post(dev_login))
}

#[derive(Deserialize)]
struct DevLogin {
    nickname: String,
}

async fn dev_login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(body): Json<DevLogin>,
) -> ApiResult<(CookieJar, Json<User>)> {
    let nickname = body.nickname.trim();
    if nickname.is_empty() {
        return Err(ApiError::unprocessable("nickname_invalid"));
    }

    let user = match users::find_by_nickname(&state.db, nickname).await? {
        Some(user) => user,
        None => {
            let new_user = NewUser {
                telegram_id: users::next_dev_telegram_id(&state.db).await?,
                nickname: nickname.to_string(),
                avatar_url: None,
                language: Language::default(),
            };
            users::create(&state.db, &new_user, state.clock.now()).await?
        }
    };

    let cookie = session::start(&state, user.id).await?;
    Ok((jar.add(cookie), Json(user)))
}
