use serde::{Deserialize, Serialize};

/// An interface language. The SPA locale folders (`web/src/locales/<code>/`)
/// use the same codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    #[default]
    En,
    Ru,
    Uk,
}

impl Language {
    pub const ALL: [Language; 3] = [Language::En, Language::Ru, Language::Uk];

    pub fn code(self) -> &'static str {
        match self {
            Language::En => "en",
            Language::Ru => "ru",
            Language::Uk => "uk",
        }
    }

    /// Accepts a bare code (`uk`) or a locale tag (`uk-UA`, as Telegram sends).
    pub fn from_code(code: &str) -> Option<Language> {
        let primary = code.split(['-', '_']).next().unwrap_or(code);
        Language::ALL
            .into_iter()
            .find(|l| l.code().eq_ignore_ascii_case(primary))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_codes_and_locale_tags() {
        assert_eq!(Language::from_code("uk"), Some(Language::Uk));
        assert_eq!(Language::from_code("ru-RU"), Some(Language::Ru));
        assert_eq!(Language::from_code("EN"), Some(Language::En));
        assert_eq!(Language::from_code("de"), None);
    }
}
