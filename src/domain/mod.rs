//! The domain: the canonical model and the pure policy modules.
//!
//! Nothing in here touches the database, the network or the system clock.
//! Vocabulary follows `GLOSSARY.md`.

pub mod effective_match;
pub mod ids;
pub mod language;
pub mod league;
pub mod lock_reveal;
pub mod model;
pub mod pickem;
pub mod scoring;
pub mod user;

pub use ids::{EventId, MatchId, StageOrdinal, TeamId, TelegramId, UserId};
pub use language::Language;
pub use league::{LeagueId, Role};
pub use model::{
    DataSourceKind, Event, EventSnapshot, Match, MatchFormat, MatchPlacement, MatchScore,
    MatchStatus, Stage, StageKind, Team,
};
pub use user::User;
