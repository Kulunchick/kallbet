//! The text of every notification, in every language, in one place.
//!
//! There is no backend i18n beyond this: a catalog per language of
//! `key -> template`, where `{name}` marks a value filled in by [`render`].
//! Templates avoid plural forms on purpose (numbers follow a label such as
//! "Points today: 2"), so one template per language is enough. The tests
//! check that all languages define the same keys with the same placeholders.
//!
//! Messages are sent as plain text (no Telegram parse mode), so League and
//! team names need no escaping.

use crate::domain::Language;

/// "You have not predicted {match} yet." `{league} {match} {minutes}`
pub const REMINDER: &str = "reminder";
/// Predictions for a Match are revealed, with the split.
/// `{league} {match} {team_a} {count_a} {count_b} {team_b}`
pub const REVEAL: &str = "reveal";
/// The daily digest. `{league} {date} {gained} {place} {total}`
pub const DIGEST: &str = "digest";

type Catalog = &'static [(&'static str, &'static str)];

const EN: Catalog = &[
    (
        REMINDER,
        "⏰ {league}: {match} locks in {minutes} min, and you have not predicted it yet.",
    ),
    (
        REVEAL,
        "👀 {league}: Predictions for {match} are revealed. {team_a} {count_a} : {count_b} {team_b}",
    ),
    (
        DIGEST,
        "📊 {league}: match day {date}. Points today: {gained}. Place: {place}, {total} in total.",
    ),
];

const RU: Catalog = &[
    (
        REMINDER,
        "⏰ {league}: матч {match} закроется для прогнозов через {minutes} мин, а вы ещё не сделали прогноз.",
    ),
    (
        REVEAL,
        "👀 {league}: прогнозы на матч {match} раскрыты. {team_a} {count_a} : {count_b} {team_b}",
    ),
    (
        DIGEST,
        "📊 {league}: игровой день {date}. Очки за день: {gained}. Место: {place}, всего очков: {total}.",
    ),
];

const UK: Catalog = &[
    (
        REMINDER,
        "⏰ {league}: матч {match} закриється для прогнозів через {minutes} хв, а ви ще не зробили прогноз.",
    ),
    (
        REVEAL,
        "👀 {league}: прогнози на матч {match} розкрито. {team_a} {count_a} : {count_b} {team_b}",
    ),
    (
        DIGEST,
        "📊 {league}: ігровий день {date}. Очки за день: {gained}. Місце: {place}, усього очок: {total}.",
    ),
];

fn catalog(language: Language) -> Catalog {
    match language {
        Language::En => EN,
        Language::Ru => RU,
        Language::Uk => UK,
    }
}

/// The template of `key` in `language`.
pub fn template(language: Language, key: &str) -> Option<&'static str> {
    catalog(language)
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, text)| *text)
}

/// The keys a language defines.
pub fn keys(language: Language) -> Vec<&'static str> {
    catalog(language).iter().map(|(k, _)| *k).collect()
}

/// The text of message `key` in `language`, with each `{name}` replaced by
/// its value from `values`. A missing key falls back to English, and a
/// missing English one to the key itself (the tests rule both out).
pub fn render(language: Language, key: &str, values: &[(&str, String)]) -> String {
    let mut text = template(language, key)
        .or_else(|| template(Language::En, key))
        .unwrap_or(key)
        .to_string();
    for (name, value) in values {
        text = text.replace(&format!("{{{name}}}"), value);
    }
    text
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    fn placeholders(template: &str) -> BTreeSet<String> {
        template
            .split('{')
            .skip(1)
            .filter_map(|rest| rest.split_once('}').map(|(name, _)| name.to_string()))
            .collect()
    }

    #[test]
    fn every_language_defines_the_same_message_keys() {
        let english: BTreeSet<_> = keys(Language::En).into_iter().collect();
        assert!(!english.is_empty());
        for language in Language::ALL {
            let own: BTreeSet<_> = keys(language).into_iter().collect();
            assert_eq!(own, english, "keys of {}", language.code());
            assert_eq!(
                keys(language).len(),
                own.len(),
                "{} defines a key twice",
                language.code()
            );
        }
    }

    #[test]
    fn every_language_uses_the_same_placeholders_for_a_key() {
        for key in keys(Language::En) {
            let english = placeholders(template(Language::En, key).unwrap());
            for language in Language::ALL {
                let own = placeholders(template(language, key).unwrap());
                assert_eq!(
                    own,
                    english,
                    "placeholders of `{key}` in {}",
                    language.code()
                );
            }
        }
    }

    #[test]
    fn render_fills_every_placeholder() {
        let text = render(
            Language::Uk,
            REMINDER,
            &[
                ("league", "Друзі".into()),
                ("match", "NAVI vs FaZe".into()),
                ("minutes", "30".into()),
            ],
        );
        assert!(text.contains("Друзі") && text.contains("NAVI vs FaZe") && text.contains("30"));
        assert!(!text.contains('{'), "{text}");
    }
}
