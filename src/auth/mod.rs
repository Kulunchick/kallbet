//! Who is calling: server-side sessions and the [`CurrentUser`] extractor.
//! Telegram Login verification (#3) goes in `auth/telegram.rs`.

pub mod session;

pub use session::CurrentUser;
