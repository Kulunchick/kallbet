//! `BlindAnonymizer`: what an LLM sees of a Match from before its training cutoff.
//!
//! Team names become `TEAM_ALPHA` / `TEAM_BETA` (other teams `TEAM_03`, ...),
//! player handles `PLAYER_01..10` for the two active rosters (`PLAYER_11`, ...
//! for anyone else), coaches `COACH_01`, ..., the Event and Stage `EVENT`, every
//! date a distance before the Match (`T-36h`), years `YEAR`, month names
//! `MONTH`, and every URL a URL-shaped placeholder (`https://evidence.invalid/1`)
//! so that #29's evidence rule still holds on the masked dossier. The mapping is
//! deterministic per Match and bijective, and `reveal` maps an answer back:
//! placeholder URLs to the real ones, placeholders in the summary to names.
//! Sides are kept (team_a stays TEAM_ALPHA), so the LLM's `delta` and the
//! policy's pick need no translation.
//!
//! `Blind` wraps the sidecar's LLM runtime (the one `debate::build` returns):
//! the debate, its reuse key, its audit and its evidence check all run on the
//! masked dossier, and `HybridAnchored` decides on the real one.
use crate::*;
use serde_json::{Map, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

pub const TEAM_ALPHA: &str = "TEAM_ALPHA";
pub const TEAM_BETA: &str = "TEAM_BETA";
const URL_BASE: &str = "https://evidence.invalid/";
const MONTHS: [&str; 12] = [
    "january",
    "february",
    "march",
    "april",
    "may",
    "june",
    "july",
    "august",
    "september",
    "october",
    "november",
    "december",
];
/// Keys whose values are people.
const PEOPLE: [&str; 9] = [
    "active_players",
    "benched_players",
    "unused_substitutes",
    "player",
    "added_players",
    "removed_players",
    "vrs_roster",
    "active_roster",
    "stand_ins",
];
const TEAMS: [&str; 3] = ["team", "from_team", "matched_teams"];
const DATES: [&str; 4] = [
    "revision_timestamp",
    "published_at",
    "vrs_as_of",
    "liquipedia_revision_timestamp",
];
const IDS: [&str; 2] = ["revid", "liquipedia_revid"];
/// Kept as they are: closed vocabularies and the licence of the source.
const KEEP: [&str; 4] = ["kind", "match_side", "license_url", "license"];

#[derive(Clone, Debug)]
pub struct BlindAnonymizer {
    /// Real name (as written) → placeholder, for every entity met.
    names: BTreeMap<String, String>,
    /// Placeholder → the first real name it stood for (the Match's own team names).
    back: BTreeMap<String, String>,
    /// Placeholder URL → real URL.
    urls: BTreeMap<String, String>,
    /// The real team ids behind TEAM_ALPHA and TEAM_BETA.
    sides: [i64; 2],
    /// Per category, how many placeholders were handed out.
    counts: BTreeMap<&'static str, usize>,
    start: Option<Instant>,
}

impl BlindAnonymizer {
    /// The masked dossier and the mapping back. Facts that would still carry a
    /// real name after masking (free text naming someone the facts do not list
    /// is caught by the residual check against every name met) are dropped.
    pub fn mask(original: &Dossier) -> (Dossier, Self) {
        let game = &original.game;
        let mut map = Self {
            names: BTreeMap::new(),
            back: BTreeMap::new(),
            urls: BTreeMap::new(),
            sides: [game.team_a.id, game.team_b.id],
            counts: BTreeMap::new(),
            start: game.scheduled_start,
        };
        map.insert(&game.team_a.name, TEAM_ALPHA);
        map.insert(&game.team_b.name, TEAM_BETA);
        map.counts.insert("TEAM", 2);
        // First the sides' own names and rosters, so that the two active
        // rosters are PLAYER_01..05 and PLAYER_06..10.
        for side in ["team_a", "team_b"] {
            for fact in original
                .context
                .iter()
                .filter(|f| f["match_side"].as_str() == Some(side))
            {
                if let Some(team) = fact["team"].as_str() {
                    let alias = if side == "team_a" {
                        TEAM_ALPHA
                    } else {
                        TEAM_BETA
                    };
                    if !map.names.contains_key(team) {
                        map.insert(team, alias);
                    }
                }
                for p in fact["active_players"].as_array().into_iter().flatten() {
                    if let Some(p) = p.as_str() {
                        map.placeholder("PLAYER", p);
                    }
                }
            }
        }
        // Then everyone else the facts name, before any free text is rewritten.
        for fact in &original.context {
            map.collect("", fact);
        }
        let masked_game = Match {
            event_id: 0,
            id: opaque_id(game.id),
            team_a: Team {
                id: 1,
                name: TEAM_ALPHA.into(),
            },
            team_b: Team {
                id: 2,
                name: TEAM_BETA.into(),
            },
            format: game.format,
            scheduled_start: None,
        };
        let mut context = Vec::new();
        for fact in &original.context {
            let masked = map.rewrite("", fact);
            if map
                .residue(&strings(&masked).join(
                    "
",
                ))
                .is_none()
            {
                context.push(masked);
            }
        }
        let dossier = Dossier {
            game: masked_game,
            signal: original.signal.clone(),
            market: original.market,
            context,
        };
        (dossier, map)
    }

    /// The real team behind a side placeholder.
    pub fn team(&self, placeholder: &str) -> Option<i64> {
        match placeholder {
            TEAM_ALPHA => Some(self.sides[0]),
            TEAM_BETA => Some(self.sides[1]),
            _ => None,
        }
    }

    /// The first real name, Event title segment or year still present in `text`.
    pub fn residue(&self, text: &str) -> Option<String> {
        let lower = text.to_ascii_lowercase();
        if let Some(year) = years(&lower).into_iter().next() {
            return Some(year);
        }
        self.names
            .keys()
            .find(|name| contains_word(&lower, &name.to_ascii_lowercase()))
            .cloned()
    }

    /// An answer given on the masked dossier, in real terms: placeholder URLs
    /// become the URLs they stand for, placeholders in text the real names.
    pub fn reveal(&self, answer: Value) -> Value {
        match answer {
            Value::String(s) => {
                if let Some(real) = self.urls.get(&s) {
                    return Value::String(real.clone());
                }
                let mut text = s;
                for (url, real) in self.urls.iter().rev() {
                    text = replace_word(&text, url, real);
                }
                // Longest placeholder first: PLAYER_10 before PLAYER_1x prefixes.
                let mut back: Vec<_> = self.back.iter().collect();
                back.sort_by_key(|(alias, _)| std::cmp::Reverse(alias.len()));
                for (alias, real) in back {
                    text = replace_word(&text, alias, real);
                }
                Value::String(text)
            }
            Value::Array(items) => {
                Value::Array(items.into_iter().map(|v| self.reveal(v)).collect())
            }
            Value::Object(map) => Value::Object(
                map.into_iter()
                    .map(|(k, v)| (k, self.reveal(v)))
                    .collect::<Map<_, _>>(),
            ),
            other => other,
        }
    }

    fn insert(&mut self, real: &str, placeholder: &str) {
        self.names.insert(real.to_owned(), placeholder.to_owned());
        self.back
            .entry(placeholder.to_owned())
            .or_insert_with(|| real.to_owned());
    }

    fn placeholder(&mut self, category: &'static str, real: &str) -> String {
        let real = real.trim();
        if let Some(p) = self.names.get(real) {
            return p.clone();
        }
        let n = self.counts.entry(category).or_default();
        *n += 1;
        let p = format!("{category}_{:02}", *n);
        self.insert(real, &p);
        p
    }

    fn url(&mut self, real: &str) -> String {
        if let Some((p, _)) = self.urls.iter().find(|(_, r)| r.as_str() == real) {
            return p.clone();
        }
        let p = format!("{URL_BASE}{}", self.urls.len() + 1);
        self.urls.insert(p.clone(), real.to_owned());
        p
    }

    /// Registers every name a fact holds under a known key.
    fn collect(&mut self, key: &str, value: &Value) {
        match value {
            Value::Object(map) => {
                for (k, v) in map {
                    if k == "coaches" {
                        for c in v.as_array().into_iter().flatten() {
                            if let Some(p) = c["player"].as_str() {
                                self.placeholder("COACH", p);
                            }
                        }
                    } else if k == "page_title" {
                        if let Some(title) = v.as_str() {
                            self.insert(title, "EVENT");
                            for part in title.split('/').filter(|p| p.len() > 2) {
                                if years(part).is_empty() && !self.names.contains_key(part) {
                                    self.names.insert(part.to_owned(), "EVENT".into());
                                }
                            }
                        }
                    } else {
                        self.collect(k, v);
                    }
                }
            }
            Value::Array(items) => items.iter().for_each(|v| self.collect(key, v)),
            Value::String(s) if PEOPLE.contains(&key) => {
                self.placeholder("PLAYER", s);
            }
            Value::String(s) if TEAMS.contains(&key) => {
                self.placeholder("TEAM", s);
            }
            _ => {}
        }
    }

    fn rewrite(&mut self, key: &str, value: &Value) -> Value {
        match value {
            Value::Object(map) => Value::Object(
                map.iter()
                    .map(|(k, v)| (k.clone(), self.rewrite(k, v)))
                    .collect(),
            ),
            Value::Array(items) => {
                Value::Array(items.iter().map(|v| self.rewrite(key, v)).collect())
            }
            Value::Number(_) if IDS.contains(&key) => Value::from(0),
            Value::String(s) if KEEP.contains(&key) => Value::String(s.clone()),
            Value::String(s) if DATES.contains(&key) => Value::String(self.relative(s)),
            Value::String(s) if key == "page_title" => Value::String("EVENT".into()),
            Value::String(s) if s.starts_with("http://") || s.starts_with("https://") => {
                Value::String(self.url(s))
            }
            Value::String(s) if PEOPLE.contains(&key) || TEAMS.contains(&key) => Value::String(
                self.names
                    .get(s.trim())
                    .cloned()
                    .unwrap_or_else(|| self.text(s)),
            ),
            Value::String(s) => Value::String(self.text(s)),
            other => other.clone(),
        }
    }

    /// Free text with every known name, year and month name replaced.
    fn text(&self, s: &str) -> String {
        let mut names: Vec<(&String, &String)> = self.names.iter().collect();
        names.sort_by_key(|(real, _)| std::cmp::Reverse(real.len()));
        let mut text = s.to_owned();
        for (real, alias) in names {
            text = replace_word(&text, real, alias);
        }
        for year in years(&text.to_ascii_lowercase()) {
            text = replace_word(&text, &year, "YEAR");
        }
        for month in MONTHS {
            text = replace_word(&text, month, "MONTH");
        }
        text
    }

    fn relative(&self, s: &str) -> String {
        match (s.parse::<Instant>(), self.start) {
            (Ok(at), Some(start)) => format!("T-{}h", (start - at).num_hours().max(0)),
            _ => "before the Match".into(),
        }
    }
}

/// Every string value in `value` (not its keys).
fn strings(value: &Value) -> Vec<String> {
    match value {
        Value::String(s) => vec![s.clone()],
        Value::Array(items) => items.iter().flat_map(strings).collect(),
        Value::Object(map) => map.values().flat_map(strings).collect(),
        _ => vec![],
    }
}

/// A number standing for the Match that names nothing.
fn opaque_id(id: i64) -> i64 {
    let hex = crate::digest(format!("blind-match-{id}").as_bytes());
    i64::from_str_radix(&hex[..12], 16).unwrap_or(0)
}

/// Four-digit years 1900-2099 standing alone (not inside a longer number or
/// after a decimal point).
fn years(lower: &str) -> Vec<String> {
    let b = lower.as_bytes();
    let mut found = Vec::new();
    for i in 0..b.len().saturating_sub(3) {
        let window = &b[i..i + 4];
        let digits = window.iter().all(u8::is_ascii_digit);
        let before = i == 0 || !(b[i - 1].is_ascii_digit() || b[i - 1] == b'.');
        let after = i + 4 == b.len() || !b[i + 4].is_ascii_digit();
        if digits && before && after && (window.starts_with(b"19") || window.starts_with(b"20")) {
            found.push(lower[i..i + 4].to_owned());
        }
    }
    found
}

fn is_word(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_'
}

/// Whether `needle` occurs in `hay` (both lower case) between word boundaries.
fn contains_word(hay: &str, needle: &str) -> bool {
    !needle.is_empty() && find_word(hay, needle, 0).is_some()
}
fn find_word(hay: &str, needle: &str, from: usize) -> Option<usize> {
    let h = hay.as_bytes();
    let n = needle.as_bytes();
    let mut i = from;
    while let Some(pos) = hay.get(i..)?.find(needle) {
        let at = i + pos;
        let end = at + n.len();
        let starts = at == 0 || !is_word(h[at - 1]) || !is_word(n[0]);
        let ends = end == h.len() || !is_word(h[end]) || !is_word(n[n.len() - 1]);
        if starts && ends {
            return Some(at);
        }
        i = at + 1;
        while !hay.is_char_boundary(i) {
            i += 1;
        }
    }
    None
}
/// Case-insensitive whole-word replacement (ASCII case folding).
fn replace_word(text: &str, needle: &str, with: &str) -> String {
    if needle.trim().is_empty() {
        return text.to_owned();
    }
    let lower = text.to_ascii_lowercase();
    let needle = needle.to_ascii_lowercase();
    let mut out = String::with_capacity(text.len());
    let mut from = 0;
    while let Some(at) = find_word(&lower, &needle, from) {
        out.push_str(&text[from..at]);
        out.push_str(with);
        from = at + needle.len();
    }
    out.push_str(&text[from..]);
    out
}

/// The LLM runtime of the backtest: masks the dossier of every Match in
/// `matches` before the inner runtime sees it, and reveals its answer.
pub struct Blind {
    inner: Arc<dyn LlmRuntime>,
    matches: BTreeSet<i64>,
}
impl Blind {
    pub fn new(inner: Arc<dyn LlmRuntime>, matches: BTreeSet<i64>) -> Self {
        Self { inner, matches }
    }
}
#[async_trait::async_trait]
impl LlmRuntime for Blind {
    async fn evaluate(
        &self,
        bot: &BotProfile,
        dossier: &Dossier,
        tier: Tier,
        prompt: &RolePrompt,
    ) -> Result<Option<Value>, LlmError> {
        if !self.matches.contains(&dossier.game.id) {
            return self.inner.evaluate(bot, dossier, tier, prompt).await;
        }
        let (masked, map) = BlindAnonymizer::mask(dossier);
        let answer = self.inner.evaluate(bot, &masked, tier, prompt).await?;
        Ok(answer.map(|a| map.reveal(a)))
    }
}
