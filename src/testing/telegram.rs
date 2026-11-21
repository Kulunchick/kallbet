//! Builds Telegram Login Widget payloads for tests, signed with the test bot
//! token (or any other), so the real verification is exercised.

use chrono::{DateTime, Utc};
use serde_json::{Map, Value, json};

use super::TEST_BOT_TOKEN;
use crate::auth::telegram;

/// A widget login payload for one Telegram account. `body()` is what the
/// SPA posts to `POST /api/auth/telegram`.
pub struct TelegramLogin {
    fields: Map<String, Value>,
    language: Option<&'static str>,
}

impl TelegramLogin {
    /// A payload made at `now` for Telegram user `id` named `first_name`.
    pub fn new(now: DateTime<Utc>, id: i64, first_name: &str) -> TelegramLogin {
        let mut fields = Map::new();
        fields.insert("id".into(), json!(id));
        fields.insert("first_name".into(), json!(first_name));
        fields.insert("auth_date".into(), json!(now.timestamp()));
        TelegramLogin {
            fields,
            language: None,
        }
    }

    pub fn last_name(self, value: &str) -> Self {
        self.field("last_name", json!(value))
    }

    pub fn username(self, value: &str) -> Self {
        self.field("username", json!(value))
    }

    pub fn photo_url(self, value: &str) -> Self {
        self.field("photo_url", json!(value))
    }

    /// The interface language the SPA was showing, sent along with the login.
    pub fn language(mut self, code: &'static str) -> Self {
        self.language = Some(code);
        self
    }

    /// Any field, set before signing.
    pub fn field(mut self, name: &str, value: Value) -> Self {
        self.fields.insert(name.into(), value);
        self
    }

    /// The request body, signed with the test bot token.
    pub fn body(&self) -> Value {
        self.body_signed_with(TEST_BOT_TOKEN)
    }

    /// The request body, signed with another bot's token (a forgery).
    pub fn body_signed_with(&self, bot_token: &str) -> Value {
        let mut auth = self.fields.clone();
        let hash = telegram::sign(&auth, bot_token);
        auth.insert("hash".into(), json!(hash));
        self.wrap(auth)
    }

    /// The signed body with one field changed afterwards (a tampered payload).
    pub fn body_tampered(&self, name: &str, value: Value) -> Value {
        let mut body = self.body();
        body["auth"][name] = value;
        body
    }

    fn wrap(&self, auth: Map<String, Value>) -> Value {
        let mut body = json!({ "auth": auth });
        if let Some(language) = self.language {
            body["language"] = json!(language);
        }
        body
    }
}
