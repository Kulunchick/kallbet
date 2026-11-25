//! Point-in-time dossier construction shared by direct and blind replay.
use crate::*;
use chrono::Duration;
use serde_json::Value;

pub fn match_cutoff(game: &Match, actual: Instant) -> Result<Instant> {
    Ok(
        (game.scheduled_start.ok_or("missing Scheduled Start")? - Duration::minutes(15))
            .min(actual),
    )
}

pub fn replay_match(
    game: Match,
    actual: Instant,
    signal: Option<AsOf<Signal>>,
    odds: Option<AsOf<f64>>,
    facts: Vec<AsOf<Value>>,
) -> Result<Dossier> {
    let cutoff = match_cutoff(&game, actual)?;
    Ok(Dossier {
        game,
        signal: signal.map(|v| v.at(cutoff).cloned()).transpose()?,
        market: odds.map(|v| v.at(cutoff).copied()).transpose()?,
        context: facts
            .into_iter()
            .map(|v| v.at(cutoff).cloned())
            .collect::<std::result::Result<_, _>>()?,
    })
}

/// Project the dossier onto anonymous team sides and roster membership. Arbitrary
/// source prose and URLs can contain names even when the roster does not list them.
pub struct BlindAnonymizer {
    teams: [i64; 2],
    aliases: std::collections::BTreeMap<(String, String), String>,
    counts: std::collections::BTreeMap<String, usize>,
}

impl BlindAnonymizer {
    pub fn winner(&self, anonymous: i64) -> Option<i64> {
        match anonymous {
            1 => Some(self.teams[0]),
            2 => Some(self.teams[1]),
            _ => None,
        }
    }

    fn alias(&mut self, category: &str, original: &str) -> String {
        let key = (category.to_owned(), original.to_owned());
        if let Some(id) = self.aliases.get(&key) {
            return id.clone();
        }
        let number = self.counts.entry(category.to_owned()).or_default();
        *number += 1;
        let name = if category == "URL" {
            format!("https://evidence.invalid/{}", *number)
        } else {
            format!("{category}_{:02}", *number)
        };
        self.aliases.insert(key, name.clone());
        name
    }

    fn scrub(&mut self, key: &str, value: &mut Value) {
        match value {
            Value::Object(map) => {
                for (key, value) in map.iter_mut() {
                    self.scrub(key, value)
                }
            }
            Value::Array(items) => {
                for item in items {
                    self.scrub(key, item)
                }
            }
            Value::String(s) => {
                let category = match key {
                    "kind"
                        if matches!(
                            s.as_str(),
                            "liquipedia_roster" | "news_item" | "vrs_roster_diff"
                        ) =>
                    {
                        return;
                    }
                    "match_side" if matches!(s.as_str(), "team_a" | "team_b") => return,
                    _ if s.starts_with("https://") || s.starts_with("http://") => "URL",
                    "page_title" => "EVENT",
                    "revision_timestamp"
                    | "published_at"
                    | "vrs_as_of"
                    | "liquipedia_revision_timestamp" => "DATE",
                    "active_players" | "benched_players" | "unused_substitutes" | "player"
                    | "added_players" | "removed_players" | "vrs_roster" | "active_roster" => {
                        "PLAYER"
                    }
                    "team" | "from_team" | "matched_teams" => "TEAM",
                    _ => "TEXT",
                };
                *s = self.alias(category, s);
            }
            Value::Number(n) => *value = Value::String(self.alias("ID", &n.to_string())),
            _ => {}
        }
    }

    pub fn mask(original: &Dossier) -> (Dossier, Self) {
        let mut dossier = original.clone();
        let game = &mut dossier.game;
        let mut mapping = Self {
            teams: [game.team_a.id, game.team_b.id],
            aliases: Default::default(),
            counts: Default::default(),
        };
        for (team, code, id) in [
            (&mut game.team_a, "TEAM_ALPHA", 1),
            (&mut game.team_b, "TEAM_BETA", 2),
        ] {
            mapping
                .aliases
                .insert(("TEAM".into(), team.name.clone()), code.into());
            team.name = code.into();
            team.id = id;
        }
        game.event_id = 0;
        game.id = 0;
        game.scheduled_start = None;
        // Reserve the active-ten aliases before substitutes and staff.
        for fact in &dossier.context {
            if let Some(players) = fact["active_players"].as_array() {
                for name in players.iter().filter_map(Value::as_str) {
                    mapping.alias("PLAYER", name);
                }
            }
            if let (Some(side), Some(team)) = (fact["match_side"].as_str(), fact["team"].as_str()) {
                let alias = match side {
                    "team_a" => "TEAM_ALPHA",
                    "team_b" => "TEAM_BETA",
                    _ => continue,
                };
                mapping
                    .aliases
                    .insert(("TEAM".into(), team.into()), alias.into());
            }
        }
        dossier.context.retain_mut(|value| {
            // Only typed context is accepted. An arbitrary object's keys can themselves contain identifiers.
            let Ok(fact) = serde_json::from_value::<ContextFact>(value.clone()) else {
                return false;
            };
            *value = fact.to_value().expect("serializable context");
            mapping.scrub("", value);
            true
        });
        (dossier, mapping)
    }
}
mod panel;
mod runner;
pub use runner::run;

pub async fn cli() -> Result<()> {
    let mut args = std::env::args().skip(2);
    let dir = args
        .next()
        .ok_or("usage: backtest DIRECTORY [--collect-llm]")?;
    let mut live = false;
    let mut blind = false;
    for arg in args {
        match arg.as_str() {
            "--collect-llm" => live = true,
            "--blind-singapore" => blind = true,
            _ => return Err("unknown backtest option".into()),
        }
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&run(std::path::Path::new(&dir), live, blind).await?)?
    );
    Ok(())
}
