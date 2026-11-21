//! Server-side sessions in an HttpOnly, Secure, SameSite=Lax cookie.
//!
//! The cookie holds a random token; the database holds only its SHA-256, so
//! a leaked database does not leak usable sessions.

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::{Cookie, SameSite};
use chrono::Duration;
use sha2::{Digest, Sha256};

use crate::domain::{User, UserId};
use crate::error::ApiError;
use crate::state::AppState;
use crate::store::{sessions, users};

pub const SESSION_COOKIE: &str = "kallbet_session";

const SESSION_DAYS: i64 = 30;

fn hash(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}

fn cookie(value: String) -> Cookie<'static> {
    Cookie::build((SESSION_COOKIE, value))
        .path("/")
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Lax)
        .build()
}

/// Starts a session for `user_id` and returns the cookie to set.
pub async fn start(state: &AppState, user_id: UserId) -> Result<Cookie<'static>, sqlx::Error> {
    let token = hex::encode(rand::random::<[u8; 32]>());
    let now = state.clock.now();
    let expires_at = now + Duration::days(SESSION_DAYS);
    sessions::insert(&state.db, &hash(&token), user_id, now, expires_at).await?;

    let mut cookie = cookie(token);
    cookie.set_max_age(time_duration(Duration::days(SESSION_DAYS)));
    Ok(cookie)
}

/// Ends the caller's session, if any, and returns the jar with the cookie
/// removed (sign-out).
pub async fn end(state: &AppState, jar: CookieJar) -> Result<CookieJar, sqlx::Error> {
    if let Some(current) = jar.get(SESSION_COOKIE) {
        sessions::delete(&state.db, &hash(current.value())).await?;
    }
    Ok(jar.remove(cookie(String::new())))
}

fn time_duration(d: Duration) -> time::Duration {
    time::Duration::seconds(d.num_seconds())
}

/// The signed-in User. Use it as a handler argument to require a session:
/// without a valid one the request is answered with 401 `unauthorized`.
#[derive(Debug, Clone)]
pub struct CurrentUser(pub User);

impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, ApiError> {
        let jar = CookieJar::from_headers(&parts.headers);
        let token = jar.get(SESSION_COOKIE).ok_or(ApiError::unauthorized())?;
        let user_id = sessions::find_user(&state.db, &hash(token.value()), state.clock.now())
            .await?
            .ok_or(ApiError::unauthorized())?;
        let user = users::find_by_id(&state.db, user_id)
            .await?
            .ok_or(ApiError::unauthorized())?;
        Ok(CurrentUser(user))
    }
}
