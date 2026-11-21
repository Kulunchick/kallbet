use serde::Serialize;

use super::{Language, TelegramId, UserId};

pub const NICKNAME_MIN: usize = 3;
pub const NICKNAME_MAX: usize = 20;

/// The Nickname is not 3-20 characters, or holds control characters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NicknameInvalid;

/// Trims and checks a Nickname typed by a User.
pub fn validate_nickname(raw: &str) -> Result<String, NicknameInvalid> {
    let nickname = raw.trim();
    let length = nickname.chars().count();
    if !(NICKNAME_MIN..=NICKNAME_MAX).contains(&length) || nickname.chars().any(char::is_control) {
        return Err(NicknameInvalid);
    }
    Ok(nickname.to_string())
}

/// A valid Nickname made from a Telegram name: tidied, cut to the maximum
/// and padded to the minimum.
pub fn nickname_from_name(name: &str) -> String {
    let tidy: String = name
        .chars()
        .filter(|c| !c.is_control())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let mut nickname: String = tidy.chars().take(NICKNAME_MAX).collect();
    nickname = nickname.trim_end().to_string();
    if nickname.is_empty() {
        nickname = "Player".into();
    }
    while nickname.chars().count() < NICKNAME_MIN {
        nickname.push('1');
    }
    nickname
}

/// The `attempt`-th choice for a Nickname (attempt 1 is `base` itself):
/// `base`, `base2`, `base3`, ..., shortened to stay within the maximum.
pub fn nickname_attempt(base: &str, attempt: u32) -> String {
    if attempt <= 1 {
        return base.to_string();
    }
    let suffix = attempt.to_string();
    let room = NICKNAME_MAX - suffix.len();
    let head: String = base.chars().take(room).collect();
    format!("{}{suffix}", head.trim_end())
}

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
