use std::sync::Mutex;

use async_trait::async_trait;

use crate::domain::UserId;
use crate::ports::{Notification, Notifier, NotifyError};

/// A notifier that delivers nothing and remembers everything.
#[derive(Default)]
pub struct RecordingNotifier {
    sent: Mutex<Vec<Notification>>,
    failure: Mutex<Option<NotifyError>>,
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
}

#[async_trait]
impl Notifier for RecordingNotifier {
    async fn send(&self, notification: Notification) -> Result<(), NotifyError> {
        if let Some(error) = self.failure.lock().unwrap().clone() {
            return Err(error);
        }
        self.sent.lock().unwrap().push(notification);
        Ok(())
    }
}
