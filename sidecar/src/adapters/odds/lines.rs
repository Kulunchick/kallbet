use crate::{AsOf, Instant, Match, Result};
use chrono::Duration;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize)]
pub(super) struct Line {
    pub teams: [String; 2],
    pub start: Instant,
    pub observed_at: Option<Instant>,
    #[serde(with = "cached_probabilities")]
    pub probabilities: [f64; 2],
}

mod cached_probabilities {
    use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error};

    // Round-trip the exact probabilities through SQLite so replaying a cache
    // hit preserves the dossier hash even for a quote near an even split.
    pub fn serialize<S: Serializer>(prices: &[f64; 2], serializer: S) -> Result<S::Ok, S::Error> {
        prices.map(|price| price.to_string()).serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<[f64; 2], D::Error> {
        let [a, b] = <[String; 2]>::deserialize(deserializer)?;
        Ok([
            a.parse().map_err(D::Error::custom)?,
            b.parse().map_err(D::Error::custom)?,
        ])
    }
}

pub(super) fn timestamp(value: &Value) -> Option<Instant> {
    value.as_str()?.parse().ok()
}

pub(super) fn normalize(a: f64, b: f64) -> Option<[f64; 2]> {
    if !(0.0 < a && a < 1.0 && 0.0 < b && b < 1.0) {
        return None;
    }
    let p = a / (a + b);
    Some([p, 1.0 - p])
}

pub(super) fn team_key(name: &str) -> String {
    let lower = name.to_lowercase();
    let words: Vec<_> = lower.split(|c: char| !c.is_alphanumeric()).collect();
    let key: String = words
        .iter()
        .filter(|word| {
            !["team", "esports", "esport", "gaming", "clan", "club", "gg"].contains(word)
        })
        .copied()
        .collect();
    let key = if key.is_empty() { words.concat() } else { key };
    match key.as_str() {
        "nip" => "ninjasinpyjamas".into(),
        "natusvincere" => "navi".into(),
        "pvision" => "parivision".into(),
        "gosh" => "bcgame".into(),
        "dendelecs" => "dendele".into(),
        _ => key,
    }
}

pub(super) fn best<'a, T>(
    game: &Match,
    items: impl IntoIterator<Item = &'a T>,
    fields: impl Fn(&T) -> (&[String; 2], Instant),
) -> Option<(&'a T, usize)> {
    let start = game.scheduled_start?;
    let a = team_key(&game.team_a.name);
    let b = team_key(&game.team_b.name);
    if a.is_empty() || b.is_empty() || a == b {
        return None;
    }
    let mut nearest = None;
    let mut distance = Duration::MAX;
    for item in items {
        let (teams, scheduled) = fields(item);
        let first = team_key(&teams[0]);
        let second = team_key(&teams[1]);
        let side = if a == first && b == second {
            0
        } else if a == second && b == first {
            1
        } else {
            continue;
        };
        let delta = (start - scheduled).abs();
        if delta > Duration::hours(6) {
            continue;
        }
        if delta < distance {
            distance = delta;
            nearest = Some((item, side));
        } else if delta == distance {
            nearest = None;
        }
    }
    nearest
}

pub(super) fn quote(
    lines: &[Line],
    game: &Match,
    captured_at: Option<Instant>,
    cutoff: Instant,
) -> Result<Option<AsOf<f64>>> {
    let Some((line, side)) = best(game, lines, |line| (&line.teams, line.start)) else {
        return Ok(None);
    };
    let observed_at = line
        .observed_at
        .or(captured_at)
        .ok_or("missing quote observation time")?;
    let quote = AsOf::new(line.probabilities[side], observed_at, cutoff)?;
    // The provider's Match may have started earlier than KallBet's Match.
    Ok((cutoff < line.start).then_some(quote))
}
