//! Who is calling: server-side sessions, the [`CurrentUser`] extractor and
//! the verification of Telegram Login payloads.

pub mod session;
pub mod telegram;

pub use session::CurrentUser;
