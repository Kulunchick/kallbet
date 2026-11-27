//! Name normalisation for linking records across Data Sources.

use std::collections::BTreeSet;

/// Words that Data Sources add to or drop from team names at will.
const TEAM_NOISE: &[&str] = &["team", "esports", "esport", "gaming", "clan", "club", "gg"];

/// Words that say nothing about which Event is meant.
const EVENT_NOISE: &[&str] = &["cs2", "cs", "counter", "strike", "the", "of", "csgo"];

fn words(name: &str) -> Vec<String> {
    name.chars()
        .map(|c| if c.is_alphanumeric() { c } else { ' ' })
        .collect::<String>()
        .to_lowercase()
        .split_whitespace()
        .map(str::to_string)
        .collect()
}

/// A team name reduced to what every source agrees on: lowercase letters and
/// digits, without filler words ("Team Vitality", "Vitality" -> "vitality";
/// "G2 Esports" -> "g2"). A name made only of filler words is kept whole.
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
    // The Singapore Major feeds used alternate names for these rosters.
    match key.as_str() {
        "pvision" => "parivision".into(),
        "dendelecs" => "dendele".into(),
        "1w" | "theone" => "1win".into(),
        "bb" | "bbteam" => "betboom".into(),
        "gosh" => "bcgame".into(),
        _ => key,
    }
}

/// The distinctive words of an Event name.
pub fn event_words(name: &str) -> BTreeSet<String> {
    words(name)
        .into_iter()
        .filter(|w| !EVENT_NOISE.contains(&w.as_str()))
        .collect()
}

/// Share of distinctive words two Event names have in common (0 to 1).
pub fn event_similarity(a: &str, b: &str) -> f64 {
    let (a, b) = (event_words(a), event_words(b));
    let union = a.union(&b).count();
    if union == 0 {
        return 0.0;
    }
    a.intersection(&b).count() as f64 / union as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn team_names_agree_after_dropping_filler_words() {
        assert_eq!(team_key("Team Vitality"), team_key("Vitality"));
        assert_eq!(team_key("G2 Esports"), team_key("G2"));
        assert_eq!(team_key("FaZe Clan"), team_key("FaZe"));
        assert_eq!(team_key("1w Team"), team_key("1win"));
        assert_eq!(team_key("1w"), team_key("1win"));
        assert_eq!(team_key("THE ONE"), team_key("1win"));
        assert_eq!(team_key("BB Team"), team_key("BETBOOM"));
        assert_eq!(team_key("BB"), team_key("BETBOOM"));
        assert_eq!(team_key("GOSH"), team_key("BC.Game"));
        assert_eq!(team_key("Ninjas in Pyjamas"), "ninjasinpyjamas");
        assert_ne!(team_key("NAVI"), team_key("Natus Vincere"));
    }
}
