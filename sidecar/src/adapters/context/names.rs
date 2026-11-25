//! Team name normalisation and token-boundary matching across Liquipedia, VRS and RSS feeds.

use std::collections::BTreeSet;

const TEAM_NOISE: &[&str] = &["team", "esports", "esport", "gaming", "clan", "club", "gg"];

fn words(name: &str) -> Vec<String> {
    name.chars()
        .map(|c| if c.is_alphanumeric() { c } else { ' ' })
        .collect::<String>()
        .to_lowercase()
        .split_whitespace()
        .map(str::to_string)
        .collect()
}

/// Canonical comparison key for a team name across data sources.
pub fn team_key(name: &str) -> String {
    let all = words(name);
    let kept: Vec<&String> = all
        .iter()
        .filter(|w| !TEAM_NOISE.contains(&w.as_str()))
        .collect();
    let key = if kept.is_empty() {
        all.concat()
    } else {
        kept.into_iter().map(String::as_str).collect()
    };
    match key.as_str() {
        "pvision" => "parivision".into(),
        "dendelecs" => "dendele".into(),
        "nip" => "ninjasinpyjamas".into(),
        "navi" => "natusvincere".into(),
        "bb" => "betboom".into(),
        _ => key,
    }
}

/// Checks whether two team names (or any alias of either) represent the same team.
pub fn teams_match(a: &str, b: &str, aliases: &[String]) -> bool {
    let target = team_key(a);
    if target.is_empty() {
        return false;
    }
    if team_key(b) == target {
        return true;
    }
    aliases.iter().any(|alias| team_key(alias) == target)
}

/// Search phrases used to match team mentions in RSS headlines and descriptions.
pub fn team_search_terms(name: &str, aliases: &[String]) -> Vec<String> {
    let mut terms = BTreeSet::new();
    let mut add_variants = |raw: &str| {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return;
        }
        terms.insert(trimmed.to_lowercase());
        let all = words(trimmed);
        let kept: Vec<&str> = all
            .iter()
            .map(String::as_str)
            .filter(|w| !TEAM_NOISE.contains(w))
            .collect();
        if !kept.is_empty() {
            let core = kept.join(" ");
            if core.len() >= 2 {
                terms.insert(core);
            }
        }
        match team_key(trimmed).as_str() {
            "ninjasinpyjamas" => {
                terms.insert("nip".into());
                terms.insert("ninjas in pyjamas".into());
            }
            "natusvincere" => {
                terms.insert("navi".into());
                terms.insert("natus vincere".into());
            }
            "parivision" => {
                terms.insert("parivision".into());
                terms.insert("pvision".into());
            }
            "betboom" => {
                terms.insert("betboom".into());
                terms.insert("bb team".into());
            }
            "dendele" => {
                terms.insert("dendele".into());
            }
            "bcgame" => {
                terms.insert("bc.game".into());
                terms.insert("bc game".into());
            }
            _ => {}
        }
    };
    add_variants(name);
    for alias in aliases {
        add_variants(alias);
    }
    terms.into_iter().collect()
}

/// Checks whether `haystack` contains `term` on ASCII/Unicode alphanumeric token boundaries.
pub fn contains_word_phrase(haystack: &str, term: &str) -> bool {
    let term_words = words(term);
    if term_words.is_empty() {
        return false;
    }
    let hay_words = words(haystack);
    if hay_words.len() < term_words.len() {
        return false;
    }
    hay_words
        .windows(term_words.len())
        .any(|window| window == term_words.as_slice())
}
