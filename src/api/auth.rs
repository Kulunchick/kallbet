//! Auth API: Telegram sign-in, sign-out and what the SPA needs to show the
//! sign-in button.

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::auth::session;
use crate::auth::telegram::{self, LoginError, TelegramProfile};
use crate::domain::user::{nickname_attempt, nickname_from_name};
use crate::domain::{Language, User};
use crate::error::{ApiError, ApiResult};
use crate::state::AppState;
use crate::store::users::{self, NewUser};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/auth/config", get(sign_in_options))
        .route("/auth/telegram", post(sign_in))
        .route("/auth/logout", post(sign_out))
}

#[derive(Serialize)]
struct SignInOptions {
    /// The bot the Login Widget is rendered for; `None` when not configured.
    bot_username: Option<String>,
    /// Whether the dev-only login bypass is mounted.
    dev_login: bool,
}

async fn sign_in_options(State(state): State<AppState>) -> Json<SignInOptions> {
    Json(SignInOptions {
        bot_username: state.config.bot_username.clone(),
        dev_login: state.config.dev_login,
    })
}

#[derive(Deserialize)]
struct SignIn {
    /// The widget's payload, as it handed it to the page.
    auth: Map<String, Value>,
    /// The interface language, used when this creates the User.
    language: Option<String>,
}

async fn sign_in(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(body): Json<SignIn>,
) -> ApiResult<(CookieJar, Json<User>)> {
    let bot_token = state.config.bot_token.as_deref().ok_or(ApiError::new(
        StatusCode::SERVICE_UNAVAILABLE,
        "login_unavailable",
    ))?;
    let profile =
        telegram::verify(&body.auth, bot_token, state.clock.now()).map_err(|e| match e {
            LoginError::Invalid => ApiError::unauthorized_with("invalid_login"),
            LoginError::Expired => ApiError::unauthorized_with("login_expired"),
        })?;

    let language = body
        .language
        .as_deref()
        .and_then(Language::from_code)
        .unwrap_or_default();
    let user = find_or_create(&state, &profile, language).await?;

    let cookie = session::start(&state, user.id).await?;
    Ok((jar.add(cookie), Json(user)))
}

async fn find_or_create(
    state: &AppState,
    profile: &TelegramProfile,
    language: Language,
) -> Result<User, sqlx::Error> {
    if let Some(existing) = users::find_by_telegram_id(&state.db, profile.id).await? {
        return users::update_avatar(&state.db, existing.id, profile.photo_url.as_deref()).await;
    }

    let base = nickname_from_name(&profile.display_name());
    for attempt in 1.. {
        let new_user = NewUser {
            telegram_id: profile.id,
            nickname: nickname_attempt(&base, attempt),
            avatar_url: profile.photo_url.clone(),
            language,
        };
        match users::create(&state.db, &new_user, state.clock.now()).await {
            Err(e) if users::is_unique_violation(&e) => {
                // Taken Nickname: try the next one. Taken Telegram id: a
                // concurrent sign-in of the same account created the User.
                if let Some(existing) = users::find_by_telegram_id(&state.db, profile.id).await? {
                    return Ok(existing);
                }
            }
            result => return result,
        }
    }
    unreachable!("the attempts are unbounded")
}

async fn sign_out(
    State(state): State<AppState>,
    jar: CookieJar,
) -> ApiResult<(CookieJar, StatusCode)> {
    let jar = session::end(&state, jar).await?;
    Ok((jar, StatusCode::NO_CONTENT))
}
