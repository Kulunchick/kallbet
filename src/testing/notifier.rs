use std::sync::Mutex;

use async_trait::async_trait;

use crate::domain::UserId;
use crate::ports::{Delivery, Notification, Notifier, NotifyError};

/// A notifier that delivers nothing and remembers everything.
#[derive(Default)]
pub struct RecordingNotifier {
    sent: Mutex<Vec<Notification>>,
    failure: Mutex<Option<NotifyError>>,
    unreachable: Mutex<Vec<UserId>>,
    attempts: Mutex<Vec<Notification>>,
}

impl RecordingNotifier {
    pub fn new() -> RecordingNotifier {
        RecordingNotifier::default()
    }

    /// Everything sent so far, in order.
    pub fn sent(&self) -> Vec<Notification> {
        self.sent.lock().unwrap().clone()
    }

    pub fn sent_to(&self, user_id: UserId) -> Vec<Notification> {
        self.sent()
            .into_iter()
            .filter(|n| n.user_id == user_id)
            .collect()
    }

    /// Forgets what was sent so far.
    pub fn clear(&self) {
        self.sent.lock().unwrap().clear();
    }

    /// Makes every send fail (and record nothing) until [`Self::recover`].
    pub fn fail_with(&self, error: NotifyError) {
        *self.failure.lock().unwrap() = Some(error);
    }

    pub fn recover(&self) {
        *self.failure.lock().unwrap() = None;
    }

    /// Makes `user` unreachable, like a User who has not started the bot:
    /// sends to them answer `Delivery::Undeliverable` and are not recorded
    /// in [`Self::sent`] (but in [`Self::undeliverable`]).
    pub fn make_unreachable(&self, user: UserId) {
        self.unreachable.lock().unwrap().push(user);
    }

    /// Messages that were tried for unreachable Users.
    pub fn undeliverable(&self) -> Vec<Notification> {
        self.attempts.lock().unwrap().clone()
    }
}

#[async_trait]
impl Notifier for RecordingNotifier {
    async fn send(&self, notification: Notification) -> Result<Delivery, NotifyError> {
        if let Some(error) = self.failure.lock().unwrap().clone() {
            return Err(error);
        }
        if self
            .unreachable
            .lock()
            .unwrap()
            .contains(&notification.user_id)
        {
            self.attempts.lock().unwrap().push(notification);
            return Ok(Delivery::Undeliverable);
        }
        self.sent.lock().unwrap().push(notification);
        Ok(Delivery::Sent)
    }
}
