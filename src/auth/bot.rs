//! Machine authentication of Bot Users (ADR-0004's amendment).
//!
//! A sidecar presents the server's shared bot secret (`KALLBET_BOT_SECRET`)
//! to `POST /api/auth/bot` and gets a Bearer token per Bot User. The database
//! holds only the token's SHA-256 (`bot_tokens`), like sessions.

use axum::http::HeaderMap;
use axum::http::header::AUTHORIZATION;
use sha2::{Digest, Sha256};

/// The token of an `Authorization: Bearer <token>` header, if there is one.
pub fn bearer(headers: &HeaderMap) -> Option<&str> {
    let value = headers.get(AUTHORIZATION)?.to_str().ok()?;
    let (scheme, token) = value.split_once(' ')?;
    scheme
        .eq_ignore_ascii_case("bearer")
        .then_some(token.trim())
        .filter(|token| !token.is_empty())
}

/// What the database stores of a Bearer token.
pub fn hash(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}

/// A fresh random Bearer token.
pub fn new_token() -> String {
    hex::encode(rand::random::<[u8; 32]>())
}

/// Whether `presented` is the shared bot secret. Compares digests without an
/// early exit, so the time taken tells nothing about where they differ.
pub fn is_secret(presented: &str, secret: &str) -> bool {
    let (a, b) = (
        Sha256::digest(presented.as_bytes()),
        Sha256::digest(secret.as_bytes()),
    );
    a.iter()
        .zip(b.iter())
        .fold(0u8, |diff, (x, y)| diff | (x ^ y))
        == 0
}
