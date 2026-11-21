use async_trait::async_trait;

use crate::domain::{Language, TelegramId, UserId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NotificationKind {
    /// "You have not predicted yet", 30 minutes before Lock.
    Reminder,
    /// A Match's Predictions were revealed.
    Reveal,
    /// The daily digest after a match day.
    DailyDigest,
}

/// One message to one User, already rendered in their language.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notification {
    pub user_id: UserId,
    pub telegram_id: TelegramId,
    /// The language `text` is written in (the User's choice).
    pub language: Language,
    pub kind: NotificationKind,
    pub text: String,
}

/// What became of a message the port did not fail on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Delivery {
    /// Handed to Telegram.
    Sent,
    /// Cannot be delivered, and trying again will not help: the User has not
    /// started the bot (or blocked it), the chat does not exist, or no bot is
    /// configured. Not an error; the caller counts the notification as handled.
    Undeliverable,
}

/// A failure that may pass (network, Telegram 5xx or rate limit). The caller
/// tries again on a later poller pass while the notification is still timely.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("could not deliver notification: {0}")]
pub struct NotifyError(pub String);

/// Sends a localised message to a User via Telegram. Deciding who gets what,
/// preferences and dedupe happen before this port, not behind it.
#[async_trait]
pub trait Notifier: Send + Sync {
    async fn send(&self, notification: Notification) -> Result<Delivery, NotifyError>;
}

/// The notifier used until the Telegram bot is wired in: logs the message.
pub struct LogNotifier;

#[async_trait]
impl Notifier for LogNotifier {
    async fn send(&self, notification: Notification) -> Result<Delivery, NotifyError> {
        tracing::info!(
            user_id = %notification.user_id,
            kind = ?notification.kind,
            "notification (not sent, no bot configured): {}",
            notification.text
        );
        Ok(Delivery::Undeliverable)
    }
}
