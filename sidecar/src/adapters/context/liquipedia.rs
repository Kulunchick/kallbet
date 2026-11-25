//! Point-in-time Liquipedia historical-revision roster parser (Seam 2).

use super::names::teams_match;
use super::wikitext::{Template, find_templates, strip_comments};
use crate::*;
use serde::Deserialize;

pub const USER_AGENT: &str = "KallBet/0.1 (https://kallbet.kulunchick.tech)";
pub const DEFAULT_API_URL: &str = "https://liquipedia.net/counterstrike/api.php";

pub fn attribution() -> Attribution {
    Attribution {
        source: "Liquipedia".into(),
        license: "CC-BY-SA 3.0".into(),
        license_url: "https://creativecommons.org/licenses/by-sa/3.0/".into(),
    }
}

pub fn revision_source_url(page_title: &str, revid: i64) -> String {
    let encoded_title = page_title.replace(' ', "_");
    format!("https://liquipedia.net/counterstrike/index.php?title={encoded_title}&oldid={revid}")
}

#[derive(Deserialize)]
struct QueryResponse {
    query: Option<QueryBody>,
}

#[derive(Deserialize)]
struct QueryBody {
    #[serde(default)]
    pages: Vec<WikiPage>,
}

#[derive(Deserialize)]
struct WikiPage {
    title: String,
    #[serde(default)]
    missing: bool,
    #[serde(default)]
    revisions: Vec<WikiRevision>,
}

#[derive(Deserialize)]
struct WikiRevision {
    revid: i64,
    timestamp: Instant,
    slots: WikiSlots,
}

#[derive(Deserialize)]
struct WikiSlots {
    main: WikiSlotMain,
}

#[derive(Deserialize)]
struct WikiSlotMain {
    content: String,
}

/// Parses a MediaWiki `action=query&prop=revisions&formatversion=2` JSON payload and
/// selects the latest revision strictly before `cutoff`.
///
/// Returns `Ok(None)` if the page is missing or every returned revision has `timestamp >= cutoff`.
pub fn parse_revision_rosters(body: &str, cutoff: Instant) -> Result<Option<AsOf<EventRosters>>> {
    let response: QueryResponse = serde_json::from_str(body.trim_start_matches('\u{feff}'))?;
    let Some(query) = response.query else {
        return Err("missing MediaWiki query object".into());
    };
    let Some(page) = query.pages.into_iter().next() else {
        return Ok(None);
    };
    if page.missing {
        return Ok(None);
    }
    let Some(rev) = page
        .revisions
        .into_iter()
        .filter(|r| r.timestamp < cutoff)
        .max_by_key(|r| r.timestamp)
    else {
        return Ok(None);
    };

    let teams = parse_participants_wikitext(&rev.slots.main.content);
    let event = EventRosters {
        source_url: revision_source_url(&page.title, rev.revid),
        page_title: page.title,
        revid: rev.revid,
        revision_timestamp: rev.timestamp,
        attribution: attribution(),
        teams,
    };
    Ok(Some(AsOf::new(event, rev.timestamp, cutoff)?))
}

/// Extracts participating teams, active lineups, coaches, stand-ins, and notes
/// from `{{TeamParticipants}}` templates in Liquipedia wikitext.
pub fn parse_participants_wikitext(wikitext: &str) -> Vec<TeamRoster> {
    let clean = strip_comments(wikitext);
    let mut rosters = Vec::new();
    for block in find_templates(&clean, "TeamParticipants") {
        for arg in &block.args {
            for opponent in find_templates(&arg.value, "Opponent") {
                if let Some(roster) = parse_opponent(&opponent) {
                    rosters.push(roster);
                }
            }
        }
    }
    rosters
}

fn parse_opponent(opponent: &Template) -> Option<TeamRoster> {
    let team = opponent.positional().next()?.trim().to_string();
    if team.is_empty() {
        return None;
    }

    let mut active_players = Vec::new();
    let mut stand_ins = Vec::new();
    let mut benched_players = Vec::new();
    let mut unused_substitutes = Vec::new();
    let mut coaches = Vec::new();

    if let Some(players_raw) = opponent.get("players") {
        for person in find_templates(players_raw, "Person") {
            let Some(handle) = person
                .positional()
                .next()
                .map(str::trim)
                .filter(|s| !s.is_empty())
            else {
                continue;
            };
            let role = person.get("role").map(|s| s.to_ascii_lowercase());
            let status = person.get("status").map(|s| s.to_ascii_lowercase());
            let played = person.get("played").map(|s| s.to_ascii_lowercase());
            let emergency_sub = person
                .get("csub")
                .is_some_and(|v| v.eq_ignore_ascii_case("true"));
            let from_team = person
                .get("team")
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string);

            if role.as_deref() == Some("coach") {
                coaches.push(CoachEntry {
                    player: handle.to_string(),
                    emergency_sub,
                    former: status.as_deref() == Some("former"),
                });
            } else if role.as_deref() == Some("sub") || status.as_deref() == Some("sub") {
                let is_playing_sub = played.as_deref() == Some("true")
                    || (role.as_deref() == Some("sub") && played.as_deref() != Some("false"));
                if is_playing_sub {
                    active_players.push(handle.to_string());
                    stand_ins.push(StandInEntry {
                        player: handle.to_string(),
                        from_team,
                    });
                } else {
                    unused_substitutes.push(handle.to_string());
                }
            } else if played.as_deref() == Some("false") || status.as_deref() == Some("former") {
                benched_players.push(handle.to_string());
            } else {
                active_players.push(handle.to_string());
            }
        }
    }

    let (notes, aliases) = opponent
        .get("notes")
        .map(parse_notes_and_aliases)
        .unwrap_or_default();

    Some(TeamRoster {
        team,
        aliases,
        active_players,
        stand_ins,
        benched_players,
        unused_substitutes,
        coaches,
        notes,
    })
}

fn parse_notes_and_aliases(raw: &str) -> (Vec<RosterNote>, Vec<String>) {
    let mut notes = Vec::new();
    let mut aliases = Vec::new();
    collect_note_leaves(raw, &mut notes, &mut aliases);
    (notes, aliases)
}

fn collect_note_leaves(raw: &str, notes: &mut Vec<RosterNote>, aliases: &mut Vec<String>) {
    let templates = find_templates(raw, "Notes");
    if templates.is_empty() {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            extract_note(trimmed, notes, aliases);
        }
        return;
    }
    for t in templates {
        for pos in t.positional() {
            let nested = find_templates(pos, "Notes");
            if nested.is_empty() {
                extract_note(pos, notes, aliases);
            } else {
                collect_note_leaves(pos, notes, aliases);
            }
        }
    }
}

fn extract_note(raw: &str, notes: &mut Vec<RosterNote>, aliases: &mut Vec<String>) {
    let source_url = find_templates(raw, "cite web")
        .into_iter()
        .find_map(|cite| {
            cite.get("url")
                .map(str::trim)
                .filter(|u| u.starts_with("http://") || u.starts_with("https://"))
                .map(str::to_string)
        });

    // Extract forced tournament rename aliases like "[[1win Team]] will compete as '''The One'''".
    if raw.to_ascii_lowercase().contains("compete as") {
        let mut rest = raw;
        while let Some(start) = rest.find("[[") {
            let after = &rest[start + 2..];
            let Some(end) = after.find("]]") else {
                break;
            };
            let link = &after[..end];
            let target = link.split('|').next().unwrap_or(link).trim();
            if !target.is_empty()
                && !aliases.iter().any(|existing| existing == target)
                && !target.contains(':')
            {
                aliases.push(target.to_string());
            }
            rest = &after[end + 2..];
        }
    }

    let text = strip_ref_and_wiki_markup(raw);
    if !text.is_empty() {
        notes.push(RosterNote { text, source_url });
    }
}

fn strip_ref_and_wiki_markup(raw: &str) -> String {
    // Strip <ref>...</ref> blocks before normalizing remaining markup.
    let mut without_refs = String::with_capacity(raw.len());
    let mut rest = raw;
    while let Some(i) = rest.find("<ref") {
        without_refs.push_str(&rest[..i]);
        if let Some(end) = rest[i..].find("</ref>") {
            rest = &rest[i + end + 6..];
        } else if let Some(end) = rest[i..].find("/>") {
            rest = &rest[i + end + 2..];
        } else {
            rest = "";
            break;
        }
    }
    without_refs.push_str(rest);

    // Replace [[Target|Display]] -> Display and [[Target]] -> Target.
    let mut out = String::with_capacity(without_refs.len());
    let mut s = without_refs.as_str();
    while let Some(i) = s.find("[[") {
        out.push_str(&s[..i]);
        let after = &s[i + 2..];
        let Some(end) = after.find("]]") else {
            out.push_str(&s[i..]);
            s = "";
            break;
        };
        let inner = &after[..end];
        let display = inner.rsplit('|').next().unwrap_or(inner);
        out.push_str(display);
        s = &after[end + 2..];
    }
    out.push_str(s);

    // Replace {{player|name|...}} with the player's handle.
    let mut clean = String::with_capacity(out.len());
    let mut cursor = 0;
    while let Some(i) = out[cursor..].find("{{") {
        let at = cursor + i;
        clean.push_str(&out[cursor..at]);
        if let Some(t) = super::wikitext::read_template(&out, at) {
            if t.is("player")
                && let Some(name) = t.positional().next()
            {
                clean.push_str(name);
            }
            cursor = t.end;
        } else {
            clean.push_str("{{");
            cursor = at + 2;
        }
    }
    clean.push_str(&out[cursor..]);

    clean
        .replace("'''", "")
        .replace("''", "")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Finds a team in parsed Liquipedia Event rosters by canonical name or extracted alias.
pub fn find_team_roster<'a>(event: &'a EventRosters, team_name: &str) -> Option<&'a TeamRoster> {
    event
        .teams
        .iter()
        .find(|roster| teams_match(team_name, &roster.team, &roster.aliases))
}

/// Builds a `TeamRosterFact` for one side (`"team_a"` or `"team_b"`) of a Match.
pub fn build_roster_fact(
    event: &EventRosters,
    roster: &TeamRoster,
    match_side: &str,
) -> TeamRosterFact {
    TeamRosterFact {
        team: roster.team.clone(),
        match_side: match_side.to_string(),
        page_title: event.page_title.clone(),
        revid: event.revid,
        revision_timestamp: event.revision_timestamp,
        source_url: event.source_url.clone(),
        attribution: event.attribution.clone(),
        active_players: roster.active_players.clone(),
        stand_ins: roster.stand_ins.clone(),
        benched_players: roster.benched_players.clone(),
        unused_substitutes: roster.unused_substitutes.clone(),
        coaches: roster.coaches.clone(),
        has_stand_in: !roster.stand_ins.is_empty(),
        notes: roster.notes.clone(),
    }
}
