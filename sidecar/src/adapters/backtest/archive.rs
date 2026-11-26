//! The replay archive: saved real responses of completed Stages, read from one
//! directory (plain or gzip-compressed files, see `tests/fixtures/backtest/README.md`).
//!
//! What each file is, is read from its content, not trusted from its name:
//! PandaScore tournament Match lists (results, formats, start times: the ground
//! truth), Liquipedia revisions (a Stage page as it stood before the Stage: the
//! Pick'em view; an Event page before a Match: the rosters), Valve global
//! standings, OddsPapi settled fixture lists and their historical odds.
//! Only the Liquipedia file name carries meaning: `lp-<tournament>` is the Stage
//! revision of that PandaScore tournament.
use super::{Dated, match_cutoff};
use crate::adapters::context::wikitext::{Template, find_templates, strip_comments};
use crate::*;
use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Read,
    path::Path,
};

/// One completed Match: what the pipeline may see (`game`) and what happened.
#[derive(Clone)]
pub struct ReplayMatch {
    pub game: Match,
    pub actual_start: Instant,
    pub winner: i64,
    /// The Swiss round from PandaScore's name ("Round 3: ..."), if any.
    pub round: Option<u32>,
}
impl ReplayMatch {
    pub fn cutoff(&self) -> Instant {
        match_cutoff(
            self.game.scheduled_start.unwrap_or(self.actual_start),
            self.actual_start,
        )
    }
    pub fn loser(&self) -> i64 {
        if self.winner == self.game.team_a.id {
            self.game.team_b.id
        } else {
            self.game.team_a.id
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum StageKind {
    Swiss,
    Playoffs,
    Groups,
    Other,
}

#[derive(Clone)]
pub struct ReplayStage {
    /// PandaScore tournament id.
    pub id: i64,
    /// PandaScore serie id, used as the Event id of every Match.
    pub event_id: i64,
    /// "IEM Cologne Major 2026".
    pub event: String,
    /// "Stage 1", "Playoffs".
    pub name: String,
    pub kind: StageKind,
    pub teams: Vec<Team>,
    /// Finished Matches with a winner, in Match cutoff order.
    pub matches: Vec<ReplayMatch>,
    /// Matches of the Stage that are not scored: forfeits, unfinished, no times.
    pub unscored: usize,
    /// (winner, loser) of every finished Match with a winner, forfeits included:
    /// the records Pick'em Points are counted from.
    pub results: Vec<(i64, i64)>,
    /// Every Match of the Stage is finished or cancelled.
    pub complete: bool,
    /// Start of the Stage's first Match: earliest Scheduled Start or actual start.
    pub cutoff: Instant,
    /// The Pick'em view the server would show before the Stage, dated by the
    /// Liquipedia revision it was read from. `None`: no usable revision saved.
    pub opening: Option<Dated<Value>>,
    /// The opening Matches of that view, with format and Scheduled Start.
    pub opening_matches: Vec<Match>,
    /// Why there is no opening view, when there is none.
    pub opening_problem: Option<String>,
}
impl ReplayStage {
    pub fn title(&self) -> String {
        format!("{} / {}", self.event, self.name)
    }
}

/// A Liquipedia revision of a page, as saved from `prop=revisions`.
#[derive(Clone)]
pub struct WikiRevision {
    pub title: String,
    pub timestamp: Instant,
    /// The whole MediaWiki response, for `liquipedia::parse_revision_rosters`.
    pub body: String,
}

#[derive(Default)]
pub struct ReplayArchive {
    pub stages: Vec<ReplayStage>,
    /// Valve global standings Markdown, in the form `vrs::Standings` reads.
    pub standings: Vec<String>,
    /// OddsPapi settled `/v4/fixtures` responses.
    pub fixture_lists: Vec<Vec<u8>>,
    /// OddsPapi `/v4/historical-odds` responses by fixture id.
    pub histories: BTreeMap<String, Vec<u8>>,
    /// Polymarket closed events (gamma `/events` pages, concatenated).
    pub polymarket_events: Vec<Value>,
    /// Polymarket `prices-history` of each event's first moneyline token, by event slug.
    pub polymarket_histories: BTreeMap<String, Value>,
    /// Kalshi settled markets (`/markets` pages, concatenated).
    pub kalshi_markets: Vec<Value>,
    /// Kalshi minute candlesticks by market ticker.
    pub kalshi_candles: BTreeMap<String, Value>,
    /// Liquipedia Event page revisions (rosters), by page title, oldest first.
    pub event_pages: BTreeMap<String, Vec<WikiRevision>>,
    /// Liquipedia root title of each Event (PandaScore serie id).
    pub event_titles: BTreeMap<i64, String>,
    /// SHA-256 over every file name and content, in name order.
    pub digest: String,
    pub files: usize,
}

/// Reads a file, inflating it when its name ends in `.gz`.
pub fn read_file(path: &Path) -> Result<Vec<u8>> {
    let raw = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    if path.extension().is_some_and(|e| e == "gz") {
        let mut out = Vec::new();
        flate2::read::GzDecoder::new(&raw[..]).read_to_end(&mut out)?;
        Ok(out)
    } else {
        Ok(raw)
    }
}

impl ReplayArchive {
    /// Every team and person the archive names: the Stages' teams (PandaScore
    /// names and acronyms) and, from every saved Liquipedia Event revision, the
    /// teams, players and coaches. The anonymizer masks those that free text
    /// mentions.
    pub fn known_names(&self) -> Vec<(String, &'static str)> {
        let mut known = std::collections::BTreeSet::new();
        for stage in &self.stages {
            for team in &stage.teams {
                known.insert((team.name.clone(), "TEAM"));
            }
        }
        for revision in self.event_pages.values().flatten() {
            let at = revision.timestamp + chrono::Duration::seconds(1);
            let Ok(Some(rosters)) =
                crate::adapters::context::liquipedia::parse_revision_rosters(&revision.body, at)
            else {
                continue;
            };
            let Ok(event) = rosters.at(at) else { continue };
            for team in &event.teams {
                known.insert((team.team.clone(), "TEAM"));
                for player in team
                    .active_players
                    .iter()
                    .chain(&team.benched_players)
                    .chain(&team.unused_substitutes)
                {
                    known.insert((player.clone(), "PLAYER"));
                }
                for coach in &team.coaches {
                    known.insert((coach.player.clone(), "COACH"));
                }
            }
        }
        known.into_iter().collect()
    }
}

impl ReplayArchive {
    pub fn load(dir: &Path) -> Result<Self> {
        let mut names: Vec<_> = std::fs::read_dir(dir)
            .map_err(|e| format!("replay archive {}: {e}", dir.display()))?
            .map(|f| f.map(|f| f.path()))
            .collect::<std::io::Result<_>>()?;
        names.retain(|p| p.is_file());
        let catalog = dir.join("catalog");
        if catalog.is_dir() {
            for entry in std::fs::read_dir(&catalog)? {
                let entry = entry?;
                if entry.file_type()?.is_file() {
                    names.push(entry.path());
                }
            }
        }
        names.sort();
        let mut archive = Self::default();
        let mut digest = Sha256::new();
        let mut matches: BTreeMap<i64, Vec<Value>> = BTreeMap::new();
        let mut stage_pages: BTreeMap<i64, WikiRevision> = BTreeMap::new();
        for path in names {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            let stem = name.trim_end_matches(".gz");
            if !(stem.ends_with(".json") || stem.ends_with(".md")) {
                continue;
            }
            let bytes = read_file(&path)?;
            let is_catalog = path.parent() == Some(catalog.as_path());
            if is_catalog {
                digest.update(b"catalog/");
            }
            digest.update(stem.as_bytes());
            digest.update(&bytes);
            archive.files += 1;
            if is_catalog {
                continue;
            }
            if stem.ends_with(".md") {
                if stem.starts_with("standings_global_") {
                    archive.standings.push(String::from_utf8(bytes)?);
                }
                continue;
            }
            let value: Value =
                serde_json::from_slice(&bytes).map_err(|e| format!("{name}: not JSON ({e})"))?;
            let first = &value[0];
            if stem.starts_with("panda-") && value.as_array().is_some_and(Vec::is_empty) {
                continue;
            }
            if first["slug"].is_string() && first["markets"].is_array() {
                archive
                    .polymarket_events
                    .extend(value.as_array().cloned().unwrap_or_default());
            } else if value["history"].is_array() {
                let slug = stem
                    .trim_end_matches(".json")
                    .strip_prefix("polymarket-history-")
                    .ok_or_else(|| format!("{name}: a price history is named by its event slug"))?;
                archive.polymarket_histories.insert(slug.to_owned(), value);
            } else if value["markets"].is_array() && value.get("cursor").is_some() {
                archive
                    .kalshi_markets
                    .extend(value["markets"].as_array().cloned().unwrap_or_default());
            } else if value["candlesticks"].is_array() {
                let ticker = value["ticker"]
                    .as_str()
                    .ok_or("candlesticks without ticker")?;
                archive.kalshi_candles.insert(ticker.to_owned(), value);
            } else if first["fixtureId"].is_string() {
                archive.fixture_lists.push(bytes);
            } else if value["fixtureId"].is_string() && value["bookmakers"].is_object() {
                let id = value["fixtureId"].as_str().unwrap_or_default().to_owned();
                archive.histories.insert(id, bytes);
            } else if first["tournament_id"].is_i64() && first["opponents"].is_array() {
                let id = first["tournament_id"].as_i64().unwrap_or_default();
                matches
                    .entry(id)
                    .or_default()
                    .extend(value.as_array().cloned().unwrap_or_default());
            } else if let Some(page) = value["query"]["pages"][0].as_object() {
                let roster_event = stem
                    .strip_prefix("roster-")
                    .and_then(|s| s.split_once('-'))
                    .and_then(|(id, _)| id.parse::<i64>().ok());
                if roster_event.is_some()
                    && page
                        .get("revisions")
                        .is_none_or(|r| r.as_array().is_some_and(Vec::is_empty))
                {
                    continue;
                }
                let revision = &page["revisions"][0];
                let (Some(title), Some(timestamp)) = (
                    page["title"].as_str(),
                    revision["timestamp"].as_str().and_then(|t| t.parse().ok()),
                ) else {
                    return Err(format!("{name}: no revision").into());
                };
                let revision = WikiRevision {
                    title: title.to_owned(),
                    timestamp,
                    body: String::from_utf8(bytes)?,
                };
                if let Some(event) = roster_event {
                    archive.event_titles.insert(event, title.to_owned());
                }
                match stem
                    .strip_prefix("lp-")
                    .and_then(|s| s.trim_end_matches(".json").parse::<i64>().ok())
                {
                    Some(tournament) => {
                        stage_pages.insert(tournament, revision);
                    }
                    None => archive
                        .event_pages
                        .entry(revision.title.clone())
                        .or_default()
                        .push(revision),
                }
            } else {
                return Err(format!("{name}: unknown kind of saved response").into());
            }
        }
        for revisions in archive.event_pages.values_mut() {
            revisions.sort_by_key(|r| r.timestamp);
        }
        archive.digest = hex::encode(digest.finalize());
        for (id, rows) in matches {
            let stage = stage_from_pandascore(id, &rows, stage_pages.get(&id))?;
            if let Some(page) = stage_pages.get(&id) {
                // `PGL/2026/Singapore/Stage 1` belongs to the Event page `PGL/2026/Singapore`;
                // a Playoffs bracket may live on the Event page itself.
                let root = match page.title.rsplit_once('/') {
                    Some((root, last)) if last.contains("Stage") || last == "Playoffs" => root,
                    _ => page.title.as_str(),
                };
                archive.event_titles.insert(stage.event_id, root.to_owned());
            }
            archive.stages.push(stage);
        }
        archive.stages.sort_by_key(|s| (s.cutoff, s.id));
        Ok(archive)
    }
}

fn instant(v: &Value) -> Option<Instant> {
    v.as_str()?.parse().ok()
}

fn stage_from_pandascore(
    id: i64,
    rows: &[Value],
    page: Option<&WikiRevision>,
) -> Result<ReplayStage> {
    let first = rows.first().ok_or("empty PandaScore Match list")?;
    let name = first["tournament"]["name"]
        .as_str()
        .unwrap_or("")
        .to_owned();
    let event = format!(
        "{} {}",
        first["league"]["name"].as_str().unwrap_or(""),
        first["serie"]["full_name"].as_str().unwrap_or("")
    )
    .trim()
    .to_owned();
    let event_id = first["serie_id"].as_i64().ok_or("missing serie_id")?;
    let kind = if name.to_lowercase().contains("playoff") {
        StageKind::Playoffs
    } else if name.to_lowercase().contains("group") {
        StageKind::Groups
    } else if name.to_lowercase().contains("swiss")
        || (event.to_lowercase().contains("major")
            && matches!(name.as_str(), "Elimination Stage" | "Opening Stage" | "Stage 1" | "Stage 2" | "Stage 3"))
    {
        StageKind::Swiss
    } else {
        StageKind::Other
    };
    let mut teams: BTreeMap<i64, Team> = BTreeMap::new();
    let mut acronyms: BTreeMap<i64, String> = BTreeMap::new();
    for row in rows {
        for o in row["opponents"].as_array().into_iter().flatten() {
            let t = &o["opponent"];
            if let (Some(tid), Some(n)) = (t["id"].as_i64(), t["name"].as_str()) {
                teams.insert(
                    tid,
                    Team {
                        id: tid,
                        name: n.to_owned(),
                    },
                );
                if let Some(a) = t["acronym"].as_str() {
                    acronyms.insert(tid, a.to_owned());
                }
            }
        }
    }
    let mut cutoff: Option<Instant> = None;
    let mut matches = Vec::new();
    let mut unscored = 0;
    let mut complete = true;
    let mut opening_rows = Vec::new();
    let mut results = Vec::new();
    for row in rows {
        let status = row["status"].as_str().unwrap_or("");
        complete &= matches!(status, "finished" | "canceled");
        let scheduled = instant(&row["scheduled_at"]);
        let actual = instant(&row["begin_at"]);
        if let Some(earliest) = [scheduled, actual].into_iter().flatten().min()
            && status != "canceled"
        {
            cutoff = Some(cutoff.map_or(earliest, |c| c.min(earliest)));
        }
        let format = match row["number_of_games"].as_i64() {
            Some(1) => Some(Format::Bo1),
            Some(3) => Some(Format::Bo3),
            Some(5) => Some(Format::Bo5),
            _ => None,
        };
        let pair: Vec<Team> = row["opponents"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|o| teams.get(&o["opponent"]["id"].as_i64()?).cloned())
            .collect();
        let round = row["name"]
            .as_str()
            .and_then(|n| n.strip_prefix("Round "))
            .and_then(|n| n.split(':').next())
            .and_then(|n| n.trim().parse().ok());
        let (Some(format), [a, b]) = (format, &pair[..]) else {
            unscored += 1;
            continue;
        };
        let game = Match {
            event_id,
            id: row["id"].as_i64().ok_or("missing Match id")?,
            team_a: a.clone(),
            team_b: b.clone(),
            format,
            scheduled_start: scheduled,
        };
        opening_rows.push((
            round,
            row["name"].as_str().unwrap_or("").to_owned(),
            game.clone(),
        ));
        let winner = row["winner_id"].as_i64();
        if status == "finished"
            && let Some(w) = winner.filter(|w| *w == a.id || *w == b.id)
        {
            results.push((w, if w == a.id { b.id } else { a.id }));
        }
        match (status, row["forfeit"].as_bool(), winner, scheduled, actual) {
            ("finished", Some(false), Some(w), Some(_), Some(actual)) if w == a.id || w == b.id => {
                matches.push(ReplayMatch {
                    game,
                    actual_start: actual,
                    winner: w,
                    round,
                })
            }
            _ => unscored += 1,
        }
    }
    matches.sort_by_key(|m| (m.cutoff(), m.game.id));
    let cutoff = cutoff.ok_or("no Match of the Stage has a start time")?;
    let teams: Vec<Team> = teams.into_values().collect();
    let mut stage = ReplayStage {
        id,
        event_id,
        event,
        name,
        kind,
        teams,
        matches,
        unscored,
        results,
        complete,
        cutoff,
        opening: None,
        opening_matches: vec![],
        opening_problem: None,
    };
    match page {
        None => stage.opening_problem = Some("no Liquipedia Stage revision saved".into()),
        Some(page) => match opening_view(&stage, &acronyms, page, &opening_rows) {
            Ok((view, games)) => {
                stage.opening = Some(Dated {
                    observed_at: page.timestamp,
                    value: view,
                });
                stage.opening_matches = games;
            }
            Err(e) => stage.opening_problem = Some(e.to_string()),
        },
    }
    Ok(stage)
}

/// The shared team key (`crate::team_names::team_key`): PandaScore names and
/// acronyms, Liquipedia template keys and Valve's names meet on it.
pub use crate::team_names::team_key;

/// The Liquipedia team template key (`{{TeamOpponent|key}}`, `|team3=key`).
fn opponent_key(value: &str) -> Option<String> {
    let t = find_templates(value, "TeamOpponent").into_iter().next()?;
    t.positional().next().map(|k| k.trim().to_owned())
}

/// The pre-Stage Pick'em view, in the shape the server's Pick'em GET gives the
/// sidecar: the Stage's teams and its opening pairings only (Swiss Round 1 or
/// the Playoffs first round), plus the initial seeds when the page lists them.
fn opening_view(
    stage: &ReplayStage,
    acronyms: &BTreeMap<i64, String>,
    page: &WikiRevision,
    rows: &[(Option<u32>, String, Match)],
) -> Result<(Value, Vec<Match>)> {
    let body: Value = serde_json::from_str(&page.body)?;
    let content = body["query"]["pages"][0]["revisions"][0]["slots"]["main"]["content"]
        .as_str()
        .ok_or("Liquipedia revision has no content")?;
    let content = strip_comments(content);
    let mut ids: BTreeMap<String, i64> = BTreeMap::new();
    for t in &stage.teams {
        ids.insert(team_key(&t.name), t.id);
        if let Some(a) = acronyms.get(&t.id) {
            ids.entry(team_key(a)).or_insert(t.id);
        }
    }
    let id_of = |key: &str| {
        ids.get(&team_key(key))
            .copied()
            .ok_or_else(|| format!("Liquipedia team key {key:?} matches no PandaScore team"))
    };
    let pair_of = |m: &Template| -> Result<(i64, i64)> {
        let a = m
            .get("opponent1")
            .and_then(opponent_key)
            .ok_or("Match without opponent1")?;
        let b = m
            .get("opponent2")
            .and_then(opponent_key)
            .ok_or("Match without opponent2")?;
        Ok((id_of(&a)?, id_of(&b)?))
    };
    // The PandaScore record of an opening pair gives its format and Scheduled Start.
    let game_of = |(a, b): (i64, i64)| -> Result<Match> {
        rows.iter()
            .map(|r| &r.2)
            .filter(|g| {
                (g.team_a.id, g.team_b.id) == (a, b) || (g.team_a.id, g.team_b.id) == (b, a)
            })
            .min_by_key(|g| g.scheduled_start)
            .cloned()
            .ok_or_else(|| format!("opening pair {a} vs {b} not in PandaScore").into())
    };
    let team = |id: i64| {
        let t = stage.teams.iter().find(|t| t.id == id);
        json!({"id": id, "name": t.map(|t| t.name.as_str()).unwrap_or("")})
    };
    match stage.kind {
        StageKind::Groups | StageKind::Other => Err("no Pick'em model for this Stage format".into()),
        StageKind::Swiss => {
            let section = content
                .split("====Round 1====")
                .nth(1)
                .or_else(|| content.split("==== Round 1 ====").nth(1))
                .ok_or("no Round 1 section")?;
            let section = section.split("====Round 2====").next().unwrap_or(section);
            let pairs = find_templates(section, "Match")
                .iter()
                .map(pair_of)
                .collect::<Result<Vec<_>>>()?;
            let mut seen = BTreeSet::new();
            if pairs.len() != 8
                || !pairs
                    .iter()
                    .all(|(a, b)| seen.insert(*a) && seen.insert(*b))
            {
                return Err(format!("Round 1 has {} distinct pairs, not 8", pairs.len()).into());
            }
            let games = pairs
                .iter()
                .map(|p| game_of(*p))
                .collect::<Result<Vec<_>>>()?;
            let format = |g: &Match| serde_json::to_value(g.format).unwrap_or(Value::Null);
            let mut view = json!({
                "stage": {"kind": "swiss", "name": stage.name},
                "teams": seen.iter().map(|id| team(*id)).collect::<Vec<_>>(),
                "groups": [{
                    "kind": "record", "wins": 0, "losses": 0,
                    "matches": games.iter().map(|g| json!({
                        "team_a": team(g.team_a.id), "team_b": team(g.team_b.id), "format": format(g),
                    })).collect::<Vec<_>>(),
                }],
            });
            // Initial seeds: `{{TeamOpponent|key|tiebreaker=16}}` (2026) or
            // `|team1=key|temp_tie1=16` (2024-2025), highest first.
            let mut seeds: Vec<(i64, i64)> = find_templates(&content, "TeamOpponent")
                .iter()
                .filter_map(|t| {
                    let rank = t.get("tiebreaker")?.trim().parse().ok()?;
                    Some((rank, id_of(t.positional().next()?).ok()?))
                })
                .collect();
            if seeds.len() != 16 {
                seeds = (1..=16)
                    .filter_map(|i| {
                        let key = field(&content, &format!("team{i}"))?;
                        let rank = field(&content, &format!("temp_tie{i}"))?.parse().ok()?;
                        Some((rank, id_of(&key).ok()?))
                    })
                    .collect();
            }
            if seeds.len() == 16 && seeds.iter().map(|s| s.1).collect::<BTreeSet<_>>() == seen {
                seeds.sort_by_key(|s| -s.0);
                view["seeds"] = json!(seeds.iter().map(|s| s.1).collect::<Vec<_>>());
            }
            Ok((view, games))
        }
        StageKind::Playoffs => {
            let pairs = (1..=4)
                .map(|i| {
                    let start = content
                        .find(&format!("|R1M{i}={{{{Match"))
                        .ok_or(format!("no R1M{i}"))?;
                    let m = find_templates(&content[start..], "Match")
                        .into_iter()
                        .next()
                        .ok_or("unreadable bracket Match")?;
                    pair_of(&m)
                })
                .collect::<Result<Vec<_>>>()?;
            let games = pairs
                .iter()
                .map(|p| game_of(*p))
                .collect::<Result<Vec<_>>>()?;
            let final_format = rows
                .iter()
                .find(|r| r.1.to_lowercase().starts_with("grand final"))
                .map(|r| r.2.format)
                .unwrap_or(Format::Bo3);
            let mut slots: Vec<Value> = games
                .iter()
                .enumerate()
                .map(|(i, g)| {
                    json!({"rounds_before_final": 2, "slot": i, "leaf": true,
                        "team_a": g.team_a.id, "team_b": g.team_b.id, "format": g.format})
                })
                .collect();
            for slot in 0..2 {
                slots.push(
                    json!({"rounds_before_final": 1, "slot": slot, "leaf": false,
                    "team_a": null, "team_b": null, "format": "bo3"}),
                );
            }
            slots.push(json!({"rounds_before_final": 0, "slot": 0, "leaf": false,
                "team_a": null, "team_b": null, "format": final_format}));
            let mut teams: Vec<i64> = pairs.iter().flat_map(|(a, b)| [*a, *b]).collect();
            teams.sort();
            let view = json!({
                "stage": {"kind": "playoffs", "name": stage.name},
                "teams": teams.iter().map(|id| team(*id)).collect::<Vec<_>>(),
                "bracket": {"ready": true, "slots": slots},
            });
            Ok((view, games))
        }
    }
}

/// `|name=value` of an old-style standings table (value up to the next `|` or line end).
fn field(content: &str, name: &str) -> Option<String> {
    let start = content.find(&format!("|{name}="))? + name.len() + 2;
    let rest = &content[start..];
    let end = rest.find(['|', '\n']).unwrap_or(rest.len());
    Some(rest[..end].trim().to_owned()).filter(|s| !s.is_empty())
}
