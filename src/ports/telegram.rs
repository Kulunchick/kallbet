//! The production `Notifier`: the Telegram Bot API `sendMessage`.
//!
//! - No `BOT_TOKEN`: the notifier is unconfigured and sends nothing, which is
//!   not an error ([`Delivery::Undeliverable`]).
//! - A User who has not started the bot (or blocked it) cannot be messaged:
//!   Telegram answers 403, and 400 `chat not found` for an id it does not
//!   know. Both are [`Delivery::Undeliverable`]: the caller logs the
//!   notification as handled, so the poller does not retry forever.
//! - Anything that may pass (network, 429 rate limit, 5xx, a wrong token) is a
//!   [`NotifyError`]; the caller leaves no log entry and tries again on a
//!   later pass while the notification is still timely.
//!
//! The bot token is part of the request URL, so errors are logged without it.

use std::time::Duration;

use async_trait::async_trait;
use serde::Deserialize;

use super::{Delivery, Notification, Notifier, NotifyError};

const API_BASE: &str = "https://api.telegram.org";
const TIMEOUT: Duration = Duration::from_secs(10);

pub struct TelegramNotifier {
    token: Option<String>,
    base_url: String,
    client: reqwest::Client,
}

impl TelegramNotifier {
    /// `None` makes the notifier unconfigured: it sends nothing.
    pub fn new(token: Option<String>) -> TelegramNotifier {
        TelegramNotifier::with_base_url(token, API_BASE)
    }

    /// Talks to another server than Telegram's (tests use a local one).
    pub fn with_base_url(token: Option<String>, base_url: &str) -> TelegramNotifier {
        TelegramNotifier {
            token,
            base_url: base_url.trim_end_matches('/').to_string(),
            client: reqwest::Client::builder()
                .timeout(TIMEOUT)
                .build()
                .expect("an HTTP client with default TLS settings"),
        }
    }
}

#[derive(Deserialize)]
struct ApiReply {
    #[serde(default)]
    description: String,
}

/// What an answer of the Bot API means for the caller.
pub fn classify(status: u16, description: &str) -> Result<Delivery, NotifyError> {
    match status {
        200..=299 => Ok(Delivery::Sent),
        // Blocked, or never started the bot.
        403 => Ok(Delivery::Undeliverable),
        // "chat not found": an id Telegram does not know (never retried).
        400 if description.contains("chat not found") => Ok(Delivery::Undeliverable),
        _ => Err(NotifyError(format!(
            "Telegram answered {status}: {description}"
        ))),
    }
}

#[async_trait]
impl Notifier for TelegramNotifier {
    async fn send(&self, notification: Notification) -> Result<Delivery, NotifyError> {
        let Some(token) = &self.token else {
            return Ok(Delivery::Undeliverable);
        };
        let response = self
            .client
            .post(format!("{}/bot{token}/sendMessage", self.base_url))
            .json(&serde_json::json!({
                "chat_id": notification.telegram_id.0,
                "text": notification.text,
                "disable_web_page_preview": true,
            }))
            .send()
            .await
            // The URL holds the token: never put it in an error.
            .map_err(|e| NotifyError(format!("request failed: {}", e.without_url())))?;
        let status = response.status().as_u16();
        let description = response
            .json::<ApiReply>()
            .await
            .map(|r| r.description)
            .unwrap_or_default();
        classify(status, &description)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_answers() {
        assert_eq!(classify(200, ""), Ok(Delivery::Sent));
        assert_eq!(
            classify(403, "Forbidden: bot was blocked by the user"),
            Ok(Delivery::Undeliverable)
        );
        assert_eq!(
            classify(400, "Bad Request: chat not found"),
            Ok(Delivery::Undeliverable)
        );
        assert!(classify(400, "Bad Request: message is too long").is_err());
        assert!(classify(401, "Unauthorized").is_err());
        assert!(classify(429, "Too Many Requests: retry after 5").is_err());
        assert!(classify(502, "").is_err());
    }
}
