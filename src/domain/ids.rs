//! Typed ids.
//!
//! `EventId`, `MatchId` and `TeamId` are **canonical ids**: the HLTV ids
//! (ADR-0001). Other Data Sources are linked to them through the external-id
//! mapping table. `UserId` is our own row id. New id types for later
//! aggregates (League, ...) belong next to their model.

use serde::{Deserialize, Serialize};

macro_rules! id {
    ($(#[$doc:meta])* $name:ident($inner:ty)) => {
        $(#[$doc])*
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
            sqlx::Type,
        )]
        #[serde(transparent)]
        #[sqlx(transparent)]
        pub struct $name(pub $inner);

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }
    };
}
#[allow(unused_imports)] // for the id types of later aggregates
pub(crate) use id;

id!(
    /// Canonical id of an Event: its HLTV event id.
    EventId(i64)
);
id!(
    /// Canonical id of a Match: its HLTV match id.
    MatchId(i64)
);
id!(
    /// Canonical id of a team: its HLTV team id.
    TeamId(i64)
);
id!(
    /// A Stage's position within its Event, starting at 1. HLTV has no Stage
    /// ids, so a Stage is identified by `(EventId, StageOrdinal)`.
    StageOrdinal(u8)
);
id!(
    /// Our id of a User.
    UserId(i64)
);
id!(
    /// A User's Telegram user id (also the chat id the bot messages).
    TelegramId(i64)
);
