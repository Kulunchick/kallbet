use serde::Serialize;

use super::{Language, TelegramId, UserId};

/// A person signed in to KallBet via Telegram, identified by a site-wide
/// unique Nickname.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct User {
    pub id: UserId,
    #[serde(skip)]
    pub telegram_id: TelegramId,
    pub nickname: String,
    pub avatar_url: Option<String>,
    pub language: Language,
}
