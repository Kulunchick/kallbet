//! Who is calling: server-side sessions, the [`CurrentUser`] extractor, the
//! verification of Telegram Login payloads, and the Bearer tokens of Bot
//! Users.

pub mod bot;
pub mod session;
pub mod telegram;

pub use session::CurrentUser;
