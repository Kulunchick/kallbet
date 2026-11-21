//! Verification of the Telegram Login Widget payload (ADR-0004).
//!
//! The widget hands the page the user's fields plus a `hash`: the hex
//! HMAC-SHA256 of the other fields (sorted by name, one `name=value` per
//! line) keyed by SHA256(bot token). See
//! <https://core.telegram.org/widgets/login#checking-authorization>.

use chrono::{DateTime, Duration, Utc};
use hmac::{Hmac, KeyInit, Mac};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

use crate::domain::TelegramId;

/// How old a payload may be. The widget produces it the moment the User
/// confirms, so a payload older than this is a replay.
pub const MAX_AGE: Duration = Duration::minutes(10);
/// Tolerated clock difference between Telegram and us.
const MAX_FUTURE: Duration = Duration::minutes(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginError {
    /// Missing or wrong hash, or a payload that is not the widget's shape.
    Invalid,
    /// Correctly signed, but `auth_date` is too old (or in the future).
    Expired,
}

/// The Telegram account a verified payload speaks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TelegramProfile {
    pub id: TelegramId,
    pub first_name: String,
    pub last_name: Option<String>,
    pub username: Option<String>,
    pub photo_url: Option<String>,
}

impl TelegramProfile {
    /// The name Telegram shows: first and last name.
    pub fn display_name(&self) -> String {
        match &self.last_name {
            Some(last) => format!("{} {last}", self.first_name),
            None => self.first_name.clone(),
        }
    }
}

/// Checks the payload against `bot_token` at time `now`.
pub fn verify(
    fields: &Map<String, Value>,
    bot_token: &str,
    now: DateTime<Utc>,
) -> Result<TelegramProfile, LoginError> {
    let hash = fields
        .get("hash")
        .and_then(Value::as_str)
        .and_then(|h| hex::decode(h).ok())
        .ok_or(LoginError::Invalid)?;
    let data = data_check_string(fields).ok_or(LoginError::Invalid)?;
    let mut mac = mac(bot_token);
    mac.update(data.as_bytes());
    // Constant-time comparison.
    mac.verify_slice(&hash).map_err(|_| LoginError::Invalid)?;

    let auth_date = fields
        .get("auth_date")
        .and_then(Value::as_i64)
        .and_then(|secs| DateTime::from_timestamp(secs, 0))
        .ok_or(LoginError::Invalid)?;
    if now - auth_date > MAX_AGE || auth_date - now > MAX_FUTURE {
        return Err(LoginError::Expired);
    }

    let text = |name: &str| {
        fields
            .get(name)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
    };
    Ok(TelegramProfile {
        id: TelegramId(fields.get("id").and_then(Value::as_i64).ok_or(LoginError::Invalid)?),
        first_name: text("first_name").unwrap_or_default(),
        last_name: text("last_name"),
        username: text("username"),
        photo_url: text("photo_url"),
    })
}

/// The hex hash Telegram would send for these fields (ignoring any `hash`
/// among them). Public so the test harness can sign payloads.
pub fn sign(fields: &Map<String, Value>, bot_token: &str) -> String {
    let data = data_check_string(fields).expect("scalar fields");
    let mut mac = mac(bot_token);
    mac.update(data.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

fn mac(bot_token: &str) -> Hmac<Sha256> {
    let secret = Sha256::digest(bot_token.as_bytes());
    Hmac::<Sha256>::new_from_slice(&secret).expect("HMAC accepts any key length")
}

/// `name=value` lines of every field except `hash`, sorted by name. `None`
/// if a value is not a string, number or boolean.
fn data_check_string(fields: &Map<String, Value>) -> Option<String> {
    let mut lines = Vec::with_capacity(fields.len());
    for (name, value) in fields.iter().filter(|(name, _)| *name != "hash") {
        let value = match value {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            _ => return None,
        };
        lines.push(format!("{name}={value}"));
    }
    lines.sort();
    Some(lines.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const TOKEN: &str = "123456:TEST-BOT-TOKEN";

    fn fields(hash: &str) -> Map<String, Value> {
        let Value::Object(map) = json!({
            "id": 1001, "first_name": "Alex", "username": "alex",
            "auth_date": 1768046400, "hash": hash,
        }) else {
            unreachable!()
        };
        map
    }

    // The hash was computed independently (Python hmac/hashlib) over
    // "auth_date=1768046400\nfirst_name=Alex\nid=1001\nusername=alex".
    const KNOWN_HASH: &str = "b9326b2abe885af46664778c7675f92d6a697fa12a7b01250d2a38fa05210dfd";

    #[test]
    fn signs_like_telegram_documents() {
        assert_eq!(sign(&fields(""), TOKEN), KNOWN_HASH);
    }

    #[test]
    fn accepts_a_known_good_payload() {
        let now = DateTime::from_timestamp(1768046400 + 30, 0).unwrap();
        let profile = verify(&fields(KNOWN_HASH), TOKEN, now).unwrap();
        assert_eq!(profile.id, TelegramId(1001));
        assert_eq!(profile.username.as_deref(), Some("alex"));
    }
}
