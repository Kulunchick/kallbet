//! Error text shared by the CLIs and the HTTP APIs, mapped to an `LlmError`.
//! The phrases come from the saved fixtures and from the strings the installed
//! `claude`, `codex` and `agy` binaries carry (tests/fixtures/llm/README.md).
use crate::LlmError;
use serde_json::Value;

const QUOTA: &[&str] = &[
    "usage limit",
    "usage_limit_reached",
    "rate limit",
    "rate_limit",
    "too many requests",
    "quota",
    "resource_exhausted",
    "out of credits",
    "out of extra usage",
];
const AUTH: &[&str] = &[
    "not logged in",
    "please run /login",
    "unauthorized",
    "unauthenticated",
    "authentication",
    "missing bearer",
    "invalid api key",
    "invalid x-api-key",
    "permission_denied",
];
const TIMEOUT: &[&str] = &["timed out", "timeout", "deadline_exceeded"];

/// Classifies a provider's error message; `None` when it names no known cause.
pub fn text(message: &str) -> Option<LlmError> {
    let lower = message.to_lowercase();
    let has = |words: &[&str]| words.iter().any(|w| lower.contains(w));
    // "You've hit your weekly limit" / "You’ve hit your limit" (straight or curly quote).
    let hit_limit = lower.contains("hit your") && lower.contains("limit");
    if hit_limit || has(QUOTA) || status_in(&lower, 429) {
        Some(LlmError::QuotaExceeded)
    } else if has(AUTH) || status_in(&lower, 401) || status_in(&lower, 403) {
        Some(LlmError::Auth)
    } else if has(TIMEOUT) {
        Some(LlmError::Timeout)
    } else if lower.contains("overloaded")
        || lower.contains("at capacity")
        || status_in(&lower, 529)
    {
        Some(LlmError::Transient)
    } else {
        None
    }
}

/// An HTTP status as a provider reports it.
pub fn status(code: u16) -> Option<LlmError> {
    match code {
        200..=299 => None,
        429 => Some(LlmError::QuotaExceeded),
        401 | 403 => Some(LlmError::Auth),
        404 => Some(LlmError::NotConfigured),
        408 => Some(LlmError::Timeout),
        400 | 422 => Some(LlmError::InvalidOutput),
        _ => Some(LlmError::Transient),
    }
}

/// `code` as a whole number in `text`: "status 429", "(429)", "429 Too Many";
/// not the digits inside a request id such as `a6028182429f`.
fn status_in(text: &str, code: u16) -> bool {
    let code = code.to_string();
    text.match_indices(&code).any(|(i, _)| {
        let before = text[..i].chars().next_back();
        let after = text[i + code.len()..].chars().next();
        !before.is_some_and(|c| c.is_ascii_alphanumeric())
            && !after.is_some_and(|c| c.is_ascii_alphanumeric())
    })
}

/// Text answers that hold JSON, with or without a Markdown code fence.
pub fn json_text(text: &str) -> Option<Value> {
    let trimmed = text.trim();
    let unfenced = trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```"))
        .and_then(|t| t.strip_suffix("```"))
        .unwrap_or(trimmed);
    serde_json::from_str(unfenced.trim()).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn whole_status_numbers_only() {
        assert_eq!(
            text("API Error: Request rejected (429)"),
            Some(LlmError::QuotaExceeded)
        );
        assert_eq!(text("cf-ray: a6028182429f"), None);
        assert_eq!(
            text("unexpected status 401 Unauthorized"),
            Some(LlmError::Auth)
        );
        assert_eq!(text("529 Overloaded"), Some(LlmError::Transient));
    }
    #[test]
    fn subscription_limits() {
        for m in [
            "You've hit your weekly limit · resets 11am (UTC)",
            "You’ve hit your usage limit. Upgrade to Pro",
            "You have exhausted your quota on this model.",
            "Usage limit reached",
        ] {
            assert_eq!(text(m), Some(LlmError::QuotaExceeded), "{m}");
        }
    }
    #[test]
    fn fenced_json() {
        assert_eq!(
            json_text("```json\n{\"a\":1}\n```"),
            Some(serde_json::json!({"a":1}))
        );
        assert_eq!(json_text("Hi!"), None);
    }
}
