//! Bot auth API: the machine sign-in of Bot Users (ADR-0004's amendment).
//!
//! `POST /api/auth/bot` with `Authorization: Bearer <KALLBET_BOT_SECRET>` and
//! `{"bot_key", "nickname", "avatar_url"?, "language"?}` → 200
//! `{"user": User, "token": "..."}`. The token is the Bot User's Bearer
//! token: every endpoint that takes `CurrentUser` accepts it.
//!
//! Errors: 503 `bot_auth_unavailable` (no secret configured), 401
//! `unauthorized` (missing or wrong secret), 400 `invalid_body`, 422
//! `bot_key_invalid` / `nickname_invalid` / `language_invalid` /
//! `avatar_url_invalid`. No human ever sees these codes, so the SPA has no
//! texts for them.

use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::post;
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqliteConnection;

use crate::auth::bot;
use crate::db;
use crate::domain::user::{nickname_attempt, validate_nickname};
use crate::domain::{Language, User, UserId};
use crate::error::{ApiError, ApiResult};
use crate::state::AppState;
use crate::store::{bots, users};

pub fn router() -> Router<AppState> {
    Router::new().route("/auth/bot", post(sign_in))
}

/// The profile a sidecar registers a Bot User with.
#[derive(Deserialize)]
struct SignIn {
    /// The bot's stable name in the sidecar's configuration: the same key
    /// always means the same Bot User.
    bot_key: String,
    nickname: String,
    /// Left out or `null`: the Bot User has no avatar.
    avatar_url: Option<String>,
    /// Left out: English for a new Bot User, unchanged for a known one.
    language: Option<String>,
}

#[derive(Serialize)]
struct BotSession {
    user: User,
    /// The Bot User's Bearer token. Sent only here; the database keeps its
    /// hash.
    token: String,
}

const BOT_KEY_MAX: usize = 64;
const AVATAR_URL_MAX: usize = 512;

/// Registers a Bot User under a bot key, or updates the one already
/// registered under it, and returns its Bearer token. Each call issues a new
/// token and revokes the previous one of that bot key.
///
/// The secret is checked before anything of the body is looked at.
async fn sign_in(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<Json<BotSession>> {
    let secret = state.config.bot_secret.as_deref().ok_or(ApiError::new(
        StatusCode::SERVICE_UNAVAILABLE,
        "bot_auth_unavailable",
    ))?;
    if !bot::bearer(&headers).is_some_and(|presented| bot::is_secret(presented, secret)) {
        return Err(ApiError::unauthorized());
    }

    let body: SignIn =
        serde_json::from_slice(&body).map_err(|_| ApiError::bad_request("invalid_body"))?;
    let key = body.bot_key.as_str();
    if key.is_empty() || key.len() > BOT_KEY_MAX || !key.chars().all(|c| c.is_ascii_graphic()) {
        return Err(ApiError::unprocessable("bot_key_invalid"));
    }
    let nickname = validate_nickname(&body.nickname)
        .map_err(|_| ApiError::unprocessable("nickname_invalid"))?;
    let language = body
        .language
        .as_deref()
        .map(|l| Language::from_code(l).ok_or(ApiError::unprocessable("language_invalid")))
        .transpose()?;
    let avatar_url = body.avatar_url.as_deref().map(str::trim);
    if avatar_url.is_some_and(|url| !is_avatar_url(url)) {
        return Err(ApiError::unprocessable("avatar_url_invalid"));
    }

    let token = bot::new_token();
    let profile = Profile {
        bot_key: key,
        nickname: &nickname,
        avatar_url,
        language,
    };
    let mut tx = db::begin_write(&state.db).await?;
    let now = state.clock.now();
    let registered = register(&mut tx, &profile, &bot::hash(&token), now)
        .await
        .map_err(ApiError::from);
    let user_id = db::finish(tx, registered).await?;
    let user = users::find_by_id(&state.db, user_id)
        .await?
        .ok_or_else(|| ApiError::internal("the registered Bot User is gone"))?;
    Ok(Json(BotSession { user, token }))
}

/// An absolute http(s) URL of sane length without whitespace: all the SPA
/// ever does with it is load it as an image.
fn is_avatar_url(url: &str) -> bool {
    url.len() <= AVATAR_URL_MAX
        && ["https://", "http://"]
            .iter()
            .any(|scheme| url.len() > scheme.len() && url.starts_with(scheme))
        && !url.chars().any(|c| c.is_whitespace() || c.is_control())
}

struct Profile<'a> {
    bot_key: &'a str,
    nickname: &'a str,
    avatar_url: Option<&'a str>,
    language: Option<Language>,
}

/// Creates the Bot User of the bot key, or updates its profile, and makes
/// `token_hash` its token. A Nickname another User has gets the first free
/// variant (`Oracle`, `Oracle2`, ...); a variant the bot already holds counts
/// as free, so registering again settles on the same Nickname.
async fn register(
    db: &mut SqliteConnection,
    profile: &Profile<'_>,
    token_hash: &str,
    now: DateTime<Utc>,
) -> Result<UserId, sqlx::Error> {
    let existing = bots::find_by_key(&mut *db, profile.bot_key).await?;
    let mut attempt = 1;
    let nickname = loop {
        let candidate = nickname_attempt(profile.nickname, attempt);
        if !bots::nickname_taken(&mut *db, &candidate, existing).await? {
            break candidate;
        }
        attempt += 1;
    };
    let user = match existing {
        Some(user) => {
            bots::update_profile(
                &mut *db,
                user,
                &nickname,
                profile.avatar_url,
                profile.language,
            )
            .await?;
            user
        }
        None => {
            let language = profile.language.unwrap_or_default();
            bots::create(&mut *db, &nickname, profile.avatar_url, language, now).await?
        }
    };
    bots::set_token(&mut *db, profile.bot_key, user, token_hash, now).await?;
    Ok(user)
}
