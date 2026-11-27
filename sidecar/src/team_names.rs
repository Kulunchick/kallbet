//! The one team-name key every adapter compares by (odds lines, Liquipedia
//! rosters, Valve standings, news, the backtest archive): lower-case
//! alphanumerics without filler words, then one alias table that maps every
//! short or former name to the team's full current name.

/// Words that do not tell teams apart ("Team Vitality" = "Vitality").
const NOISE: [&str; 7] = ["team", "esports", "esport", "gaming", "clan", "club", "gg"];

/// Short or former name (as a key) → the team's key. One table for every
/// source; add a name here, never in an adapter.
const ALIASES: [(&str, &str); 28] = [
    ("nip", "ninjasinpyjamas"),
    ("navi", "natusvincere"),
    ("tl", "liquid"),
    ("fq", "flyquest"),
    ("fly", "flyquest"),
    ("vit", "vitality"),
    ("imp", "imperial"),
    ("gl", "gamerlegion"),
    ("lvg", "lynnvision"),
    ("mongolz", "themongolz"),
    ("col", "complexity"),
    ("c9", "cloud9"),
    ("vp", "virtuspro"),
    ("pvision", "parivision"),
    ("pv", "parivision"),
    ("dendelecs", "dendele"),
    ("1w", "1win"),
    ("theone", "1win"),
    ("chinggis", "chinggiswarriors"),
    ("cw", "chinggiswarriors"),
    ("ef", "eternalfire"),
    ("100t", "100thieves"),
    ("fnc", "fnatic"),
    ("gaimin", "gaimingladiators"),
    // BC.Game played Singapore 2026 under a forced rename (Liquipedia's note).
    ("gosh", "bcgame"),
    ("wc", "wildcard"),
    ("bb", "betboom"),
    ("bbteam", "betboom"),
];

/// The comparison key of a team name.
pub fn team_key(name: &str) -> String {
    let lower = name.to_lowercase();
    let words: Vec<&str> = lower
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .collect();
    let kept: String = words
        .iter()
        .filter(|w| !NOISE.contains(w))
        .copied()
        .collect();
    let key = if kept.is_empty() {
        words.concat()
    } else {
        kept
    };
    ALIASES
        .iter()
        .find(|(alias, _)| *alias == key)
        .map_or(key, |(_, team)| (*team).to_owned())
}

#[cfg(test)]
mod tests {
    use super::team_key;

    #[test]
    fn every_source_spelling_meets_on_one_key() {
        for (a, b) in [
            ("Natus Vincere", "NAVI"),
            ("Ninjas in Pyjamas", "NIP"),
            ("Team Vitality", "Vitality"),
            ("PARIVISION", "PVISION"),
            ("BC.Game Esports", "GOSH"),
            ("Dendele CS", "Dendele"),
            ("1win", "1w Team"),
            ("1win", "1w"),
            ("1win", "THE ONE"),
            ("BetBoom Team", "BB"),
            ("The MongolZ", "MongolZ"),
            ("Team Liquid", "TL"),
        ] {
            assert_eq!(team_key(a), team_key(b), "{a} / {b}");
        }
        assert_ne!(team_key("FaZe"), team_key("FURIA"));
    }
}
