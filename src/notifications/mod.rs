//! Notifications: decides which reminders, Reveal announcements and daily
//! digests are due, renders them in the User's language and sends them
//! through the `Notifier` port.
//!
//! [`after_poll`] runs on every poller pass for every active League, so
//! everything here is idempotent: each notification has a dedupe key, and the
//! notification log (`store::notification_log`) remembers the keys that were
//! handled. A notification is handled once it was delivered, or found
//! undeliverable for good (the User never started the bot, or no bot is
//! configured). A passing failure leaves no entry, so the next pass tries
//! again for as long as the notification is still timely.
//!
//! What is due is decided in [`due`] from the League's `LeagueMatches` (the
//! effective Match view); the rendering is in [`messages`].

mod due;
pub mod messages;

use chrono::{DateTime, Utc};

use crate::domain::user::NotificationPrefs;
use crate::league_matches::LeagueMatches;
use crate::ports::{Notification, NotificationKind};
use crate::state::AppState;
use crate::store::leagues::League;
use crate::store::{notification_log, users};

pub use due::{DIGEST_GRACE, REMINDER_LEAD, REVEAL_ANNOUNCE_WINDOW, STALE_AFTER_START};

/// Sends whatever is due for `league` at `now`. Called by the poller.
pub async fn after_poll(
    state: &AppState,
    league: &League,
    matches: &LeagueMatches,
    now: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    if league.archived_at.is_some() {
        return Ok(());
    }
    for item in due::due(league, matches, now) {
        deliver(state, league, item, now).await?;
    }
    Ok(())
}

fn wanted(kind: NotificationKind, prefs: NotificationPrefs) -> bool {
    match kind {
        NotificationKind::Reminder => prefs.reminder,
        NotificationKind::Reveal => prefs.reveal,
        NotificationKind::DailyDigest => prefs.digest,
    }
}

async fn deliver(
    state: &AppState,
    league: &League,
    item: due::Due,
    now: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    if notification_log::contains(&state.db, &item.key).await? {
        return Ok(());
    }
    let Some(user) = users::find_by_id(&state.db, item.user).await? else {
        return Ok(());
    };
    // A Bot User has no Telegram chat: nothing is ever sent to one, whatever
    // its switches say (they are off from its registration on).
    if user.is_bot || !wanted(item.kind, user.notifications) {
        return Ok(());
    }
    let notification = Notification {
        user_id: user.id,
        telegram_id: user.telegram_id,
        language: user.language,
        kind: item.kind,
        text: messages::render(user.language, item.message, &item.values),
    };
    match state.notifier.send(notification).await {
        Ok(_) => notification_log::record(&state.db, &item.key, league.id, user.id, now).await?,
        Err(e) => tracing::warn!("notification {} not delivered, will retry: {e}", item.key),
    }
    Ok(())
}
