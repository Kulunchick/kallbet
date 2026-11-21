//! Pure HLTV parser: rendered HTML in, canonical model out. No I/O.
//!
//! HLTV splits a Major into several HLTV events: a parent event (holding the
//! last Swiss Stage and the Playoffs) plus one event per earlier Stage. The
//! canonical Event is the parent; [`EventPage::parent`] and
//! [`EventPage::stage_events`] say which pages [`snapshot`] needs.
//!
//! Anything unexpected is an error, never silently empty data.

use std::collections::HashMap;

use chrono::{DateTime, TimeZone, Utc};
use scraper::{ElementRef, Html, Selector};
use serde_json::Value;

use crate::domain::{
    DataSourceKind, Event, EventId, EventSnapshot, Match, MatchFormat, MatchId, MatchPlacement,
    MatchScore, MatchStatus, Stage, StageKind, StageOrdinal, Team, TeamId,
};
use crate::ports::SourceError;

/// One parsed HLTV event page (`/events/<id>/<slug>`).
#[derive(Debug, Clone, PartialEq)]
pub struct EventPage {
    /// This HLTV event alone (for a Stage event: only that Stage's dates).
    pub event: Event,
    pub slug: String,
    /// HLTV marks the event as finished: its page will not change any more.
    pub ended: bool,
    /// The other events HLTV links from this one (a Major's parent and Stages).
    pub linked: Vec<LinkedEvent>,
    pub stages: Vec<PageStage>,
    /// Teams found on the page, with their HLTV names and logos.
    pub teams: Vec<Team>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkedEvent {
    pub id: EventId,
    pub slug: String,
    /// The label HLTV gives the link, e.g. `Stage 1` or `Stage 3 & Playoffs`.
    pub title: String,
}

/// A Stage as found on one page. `Match::stage` is set by [`snapshot`].
#[derive(Debug, Clone, PartialEq)]
pub struct PageStage {
    pub kind: StageKind,
    /// HLTV's name for a bracket; `None` for a Swiss Stage, which HLTV does
    /// not name on its own page.
    pub name: Option<String>,
    pub matches: Vec<Match>,
}

impl EventPage {
    /// The Major this Stage event belongs to, if it is one.
    pub fn parent(&self) -> Option<&LinkedEvent> {
        self.linked
            .iter()
            .find(|l| is_part_slug_of(&self.slug, &l.slug))
    }

    /// The Stage events of this Major, in HLTV's order.
    pub fn stage_events(&self) -> Vec<&LinkedEvent> {
        self.linked
            .iter()
            .filter(|l| is_part_slug_of(&l.slug, &self.slug))
            .collect()
    }
}

/// `iem-cologne-major-2026-stage-1` is part of `iem-cologne-major-2026`.
fn is_part_slug_of(slug: &str, parent: &str) -> bool {
    slug.strip_prefix(parent)
        .is_some_and(|rest| rest.starts_with('-'))
}

/// Parses one event page.
pub fn event_page(html: &str) -> Result<EventPage, SourceError> {
    let doc = Html::parse_document(html);
    let url = meta(&doc, "og:url");
    let Some((id, slug)) = url.as_deref().and_then(event_href) else {
        if is_challenge(&doc) {
            return Err(SourceError::Unavailable(
                "hltv: Cloudflare challenge instead of the page".into(),
            ));
        }
        // HLTV's "Page not found" page (served for an unknown event id).
        if doc
            .select(&sel(".error-status h1"))
            .any(|h| text_of(h) == "404")
        {
            return Err(SourceError::NotFound);
        }
        return Err(parse_error("not an HLTV event page (no og:url)".into()));
    };
    let name = meta(&doc, "og:title")
        .ok_or_else(|| parse_error(format!("event {id} has no name")))?;
    let dates: Vec<DateTime<Utc>> = doc
        .select(&sel("td.eventdate [data-unix]"))
        .map(|d| unix_ms(d.value().attr("data-unix").unwrap_or_default()))
        .collect::<Result<_, _>>()?;
    let ended = doc
        .select(&sel(".event-hub-indicator.event-ended"))
        .next()
        .is_some();

    let mut linked = Vec::new();
    for a in doc.select(&sel("a.linked-event")) {
        let href = a.value().attr("href").unwrap_or_default();
        let (lid, lslug) = event_href(href)
            .ok_or_else(|| parse_error(format!("linked event without an id: {href:?}")))?;
        let title = first_text(a, ".linked-event-title")
            .ok_or_else(|| parse_error(format!("linked event {lid} has no title")))?;
        linked.push(LinkedEvent { id: lid, slug: lslug, title });
    }

    let mut teams = Vec::new();
    let mut stages = Vec::new();
    reject_unsupported_groups(&doc)?;
    if let Some(swiss) = swiss_stage(&doc, &mut teams)? {
        stages.push(swiss);
    }
    stages.extend(bracket_stages(&doc, &mut teams)?);

    Ok(EventPage {
        event: Event {
            id,
            name,
            starts_at: dates.first().copied(),
            ends_at: dates.get(1).copied(),
        },
        slug,
        ended,
        linked,
        stages,
        teams,
    })
}

/// Joins a Major's parent page and its Stage pages (or a lone event's page)
/// into the canonical snapshot. `live` are the Matches the live list shows
/// as being played (see [`live_matches`]).
pub fn snapshot(
    main: &EventPage,
    stage_pages: &[EventPage],
    live: &[MatchId],
    fetched_at: DateTime<Utc>,
) -> Result<EventSnapshot, SourceError> {
    if let Some(parent) = main.parent() {
        return Err(parse_error(format!(
            "event {} is a Stage of event {}; start from that one",
            main.event.id, parent.id
        )));
    }
    // (name, stage) in Event order; the Playoffs go last.
    let mut named: Vec<(String, PageStage)> = Vec::new();
    let mut pages: Vec<&EventPage> = Vec::new();
    let stage_events = main.stage_events();
    for link in &stage_events {
        let page = stage_pages
            .iter()
            .find(|p| p.event.id == link.id)
            .ok_or_else(|| parse_error(format!("the page of Stage event {} is missing", link.id)))?;
        if page.stages.is_empty() {
            return Err(parse_error(format!("Stage event {} has no Stage", link.id)));
        }
        for stage in &page.stages {
            let name = stage.name.clone().unwrap_or_else(|| link.title.clone());
            named.push((name, stage.clone()));
        }
        pages.push(page);
    }
    let own_swiss_name = if stage_events.is_empty() {
        "Swiss Stage".to_string()
    } else {
        format!("Stage {}", stage_events.len() + 1)
    };
    let (playoffs, others): (Vec<&PageStage>, Vec<&PageStage>) = main
        .stages
        .iter()
        .partition(|s| s.kind == StageKind::Playoffs);
    for stage in others.into_iter().chain(playoffs) {
        let name = stage.name.clone().unwrap_or_else(|| own_swiss_name.clone());
        named.push((name, stage.clone()));
    }
    pages.push(main);
    if named.is_empty() {
        return Err(parse_error(format!(
            "event {} shows no Swiss Stage and no bracket",
            main.event.id
        )));
    }

    let mut stages = Vec::new();
    let mut matches: Vec<Match> = Vec::new();
    for (i, (name, page_stage)) in named.into_iter().enumerate() {
        let ordinal = StageOrdinal(i as u8 + 1);
        stages.push(Stage { ordinal, kind: page_stage.kind, name });
        for mut m in page_stage.matches {
            if matches.iter().any(|other| other.id == m.id) {
                return Err(parse_error(format!("match {} appears twice", m.id)));
            }
            m.stage = ordinal;
            if m.status == MatchStatus::Upcoming && live.contains(&m.id) {
                m.status = MatchStatus::Live;
            }
            matches.push(m);
        }
    }

    let mut teams: Vec<Team> = Vec::new();
    for page in &pages {
        for team in &page.teams {
            if !teams.iter().any(|t| t.id == team.id) {
                teams.push(team.clone());
            }
        }
    }
    for m in &matches {
        for team in m.teams().into_iter().flatten() {
            if !teams.iter().any(|t| t.id == team) {
                return Err(parse_error(format!("team {team} of match {} is unknown", m.id)));
            }
        }
    }

    Ok(EventSnapshot {
        event: Event {
            id: main.event.id,
            name: main.event.name.clone(),
            starts_at: pages.iter().filter_map(|p| p.event.starts_at).min(),
            ends_at: pages.iter().filter_map(|p| p.event.ends_at).max(),
        },
        stages,
        teams,
        matches,
        source: DataSourceKind::Hltv,
        fetched_at,
    })
}

/// A classic group table (`Group A`, ...) is a format this parser has never
/// seen on a real page, so it refuses it rather than ignore its Matches.
fn reject_unsupported_groups(doc: &Html) -> Result<(), SourceError> {
    for header in doc.select(&sel(".groups-container .table-header.group-name")) {
        let text = text_of(header);
        if text != "Group Swiss" {
            return Err(parse_error(format!("unsupported group format {text:?}")));
        }
    }
    Ok(())
}

/// One Swiss-table row's view of a Match (`a.bottom-row`), from the side of
/// the row's team.
struct SwissRow {
    team: TeamId,
    /// The name the row shows, which can be an older name of the team.
    shown_name: String,
    scores: Option<(u8, u8)>,
    start: DateTime<Utc>,
    format_cell: String,
}

fn swiss_stage(doc: &Html, teams: &mut Vec<Team>) -> Result<Option<PageStage>, SourceError> {
    let visuals: Vec<ElementRef> = doc.select(&sel(".swiss-visual-container")).collect();
    let visual = match visuals.as_slice() {
        [] => return Ok(None),
        [one] => *one,
        _ => return Err(parse_error("more than one Swiss Stage on a page".into())),
    };

    // The standings table: team ids, names and logos, and per team the rows
    // of its Matches with time, format and score.
    let mut rows: HashMap<MatchId, Vec<SwissRow>> = HashMap::new();
    let top_rows: Vec<ElementRef> = doc.select(&sel(".groups-container div.top-row")).collect();
    if top_rows.is_empty() {
        return Err(parse_error("Swiss Stage without a standings table".into()));
    }
    for top in top_rows {
        let link = top
            .select(&sel("a[href^=\"/team/\"]"))
            .next()
            .ok_or_else(|| parse_error("Swiss standings row without a team link".into()))?;
        let team = team_href(link.value().attr("href").unwrap_or_default())?;
        let name = text_of(link);
        if name.is_empty() {
            return Err(parse_error(format!("team {team} has no name")));
        }
        let logo = top
            .select(&sel(".logo-container img.night-only"))
            .chain(top.select(&sel(".logo-container img")))
            .find_map(|img| img.value().attr("src"))
            .map(absolute_url);
        add_team(teams, Team { id: team, name, logo_url: logo });

        let cell = top
            .parent()
            .and_then(ElementRef::wrap)
            .ok_or_else(|| parse_error("Swiss standings row outside a cell".into()))?;
        for row in cell.select(&sel("a.bottom-row")) {
            let href = row.value().attr("href").unwrap_or_default();
            let id = match_href(href)?;
            let shown_name = first_text(row, ".group-team.team-1 .text-ellipsis")
                .ok_or_else(|| parse_error(format!("match {id}: row without a team name")))?;
            let start = row
                .select(&sel(".group-left [data-unix]"))
                .next()
                .and_then(|d| d.value().attr("data-unix"))
                .ok_or_else(|| parse_error(format!("match {id}: row without a time")))?;
            let left = first_text(row, ".score-left");
            let right = first_text(row, ".score-right");
            let scores = match (left, right) {
                (Some(l), Some(r)) => Some((score(&l)?, score(&r)?)),
                _ => None,
            };
            rows.entry(id).or_default().push(SwissRow {
                team,
                shown_name,
                scores,
                start: unix_ms(start)?,
                format_cell: first_text(row, ".group-right").unwrap_or_default(),
            });
        }
    }

    let mut matches = Vec::new();
    for (column, col) in visual.select(&sel(".swiss-visual-column")).enumerate() {
        let round = column as u8 + 1;
        for wrapper in col.select(&sel(".swiss-visual-matchups-wrapper")) {
            let title = first_text(wrapper, ".swiss-visual-matchups-title").unwrap_or_default();
            for matchup in wrapper.select(&sel(".swiss-visual-matchup")) {
                // HLTV draws future pairings as empty boxes without a match.
                let Some(json) = matchup.value().attr("data-match-details-popup-json") else {
                    continue;
                };
                if rows.is_empty() {
                    // Matches in the chart but not one standings row: the
                    // rows' markup changed.
                    return Err(parse_error("Swiss standings rows list no Match".into()));
                }
                let (wins, losses) = record(&title)?;
                if let Some(m) = swiss_match(json, matchup, round, wins, losses, &rows)? {
                    matches.push(m);
                }
            }
        }
    }
    Ok(Some(PageStage { kind: StageKind::Swiss, name: None, matches }))
}

fn swiss_match(
    json: &str,
    matchup: ElementRef<'_>,
    round: u8,
    wins: u8,
    losses: u8,
    rows: &HashMap<MatchId, Vec<SwissRow>>,
) -> Result<Option<Match>, SourceError> {
    let json: Value = serde_json::from_str(json)
        .map_err(|e| parse_error(format!("Swiss matchup JSON: {e}")))?;
    let id = json
        .pointer("/ref/matchId/matchId")
        .and_then(Value::as_i64)
        .map(MatchId)
        .ok_or_else(|| parse_error("Swiss matchup without a match id".into()))?;
    let finished = !json.pointer("/ref/resultId").unwrap_or(&Value::Null).is_null();
    if round != wins + losses + 1 {
        return Err(parse_error(format!(
            "match {id}: record {wins}-{losses} in Swiss round {round}"
        )));
    }

    let (a, b) = match rows.get(&id).map(Vec::as_slice) {
        Some([a, b]) if a.team != b.team => (a, b),
        // In the chart but in neither team's rows: HLTV shows a Match of a
        // live round in the chart first. Its teams, time and format are only
        // in the rows, so it is left out of this snapshot (the stored
        // Match, if any, stays as it was).
        None => return Ok(None),
        _ => {
            return Err(parse_error(format!(
                "match {id} is not in the rows of both its teams"
            )));
        }
    };
    // Order the teams as the Swiss chart draws them.
    let drawn: Vec<String> = matchup
        .select(&sel(".swiss-visual-team"))
        .filter_map(|t| {
            t.select(&sel("img"))
                .find_map(|img| img.value().attr("title"))
                .map(str::to_string)
        })
        .collect();
    let (a, b) = if drawn.first() == Some(&b.shown_name) || drawn.get(1) == Some(&a.shown_name) {
        (b, a)
    } else {
        (a, b)
    };

    let format = match a.format_cell.as_str() {
        "bo1" => MatchFormat::Bo1,
        "bo3" => MatchFormat::Bo3,
        "bo5" => MatchFormat::Bo5,
        // A played Bo1 shows its map ("d2", "inf", ...) instead.
        _ if finished => MatchFormat::Bo1,
        other => return Err(parse_error(format!("match {id}: unknown format {other:?}"))),
    };

    let (status, winner, score) = if finished {
        let (sa, sb) = a
            .scores
            .ok_or_else(|| parse_error(format!("finished match {id} has no score")))?;
        if b.scores != Some((sb, sa)) {
            return Err(parse_error(format!("match {id}: the two rows disagree on the score")));
        }
        let winner = match sa.cmp(&sb) {
            std::cmp::Ordering::Greater => a.team,
            std::cmp::Ordering::Less => b.team,
            std::cmp::Ordering::Equal => {
                return Err(parse_error(format!("finished match {id} is a draw")));
            }
        };
        let score = if format == MatchFormat::Bo1 {
            // The rows show rounds; a Bo1 is one map.
            MatchScore {
                team_a: u8::from(winner == a.team),
                team_b: u8::from(winner == b.team),
            }
        } else {
            MatchScore { team_a: sa, team_b: sb }
        };
        (MatchStatus::Finished, Some(winner), Some(score))
    } else {
        (MatchStatus::Upcoming, None, None)
    };

    Ok(Some(Match {
        id,
        stage: StageOrdinal(0),
        placement: MatchPlacement::Swiss { round, wins, losses },
        team_a: Some(a.team),
        team_b: Some(b.team),
        format,
        scheduled_start: Some(a.start),
        actual_start: None,
        status,
        winner,
        score,
    }))
}

/// `"1:0"` -> (1, 0).
fn record(title: &str) -> Result<(u8, u8), SourceError> {
    title
        .split_once(':')
        .and_then(|(w, l)| Some((w.trim().parse().ok()?, l.trim().parse().ok()?)))
        .ok_or_else(|| parse_error(format!("bad Swiss record {title:?}")))
}

fn score(text: &str) -> Result<u8, SourceError> {
    text.trim()
        .parse()
        .map_err(|_| parse_error(format!("bad score {text:?}")))
}

/// The brackets of a page (`data-slotted-bracket-json`). The single-
/// elimination bracket with the most rounds is the Playoffs; any other
/// bracket (a double elimination, a decider) is a Group Stage.
fn bracket_stages(doc: &Html, teams: &mut Vec<Team>) -> Result<Vec<PageStage>, SourceError> {
    let mut brackets: Vec<Value> = Vec::new();
    for el in doc.select(&sel("[data-slotted-bracket-json]")) {
        let raw = el.value().attr("data-slotted-bracket-json").unwrap_or_default();
        brackets.push(
            serde_json::from_str(raw).map_err(|e| parse_error(format!("bracket JSON: {e}")))?,
        );
    }
    let round_count = |b: &Value| -> Option<usize> {
        (b.get("type")?.as_str()? == "singleElimination")
            .then(|| b.get("rounds")?.as_array().map(Vec::len))
            .flatten()
    };
    let playoffs = brackets
        .iter()
        .enumerate()
        .filter_map(|(i, b)| round_count(b).map(|n| (i, n)))
        .max_by_key(|&(i, n)| (n, std::cmp::Reverse(i)))
        .map(|(i, _)| i);

    let mut stages = Vec::new();
    for (i, bracket) in brackets.iter().enumerate() {
        let name = bracket
            .get("name")
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| parse_error("bracket without a name".into()))?;
        let mut matches = Vec::new();
        if Some(i) == playoffs {
            let rounds = bracket["rounds"].as_array().cloned().unwrap_or_default();
            for (r, round) in rounds.iter().enumerate() {
                let rounds_before_final = (rounds.len() - 1 - r) as u8;
                for (slot, matchup) in slots(round).into_iter().enumerate() {
                    let placement = MatchPlacement::Playoffs {
                        rounds_before_final,
                        slot: slot as u8,
                    };
                    if let Some(m) = bracket_match(matchup, placement, teams)? {
                        matches.push(m);
                    }
                }
            }
            stages.push(PageStage { kind: StageKind::Playoffs, name: Some(name), matches });
        } else {
            let mut found = Vec::new();
            collect_matchups(bracket, &mut found);
            for matchup in found {
                if let Some(m) = bracket_match(matchup, MatchPlacement::Unplaced, teams)? {
                    matches.push(m);
                }
            }
            stages.push(PageStage { kind: StageKind::Groups, name: Some(name), matches });
        }
    }
    Ok(stages)
}

/// A round's `slot1`, `slot2`, ... matchups, top to bottom.
fn slots(round: &Value) -> Vec<&Value> {
    let mut slots: Vec<(u32, &Value)> = round
        .as_object()
        .into_iter()
        .flatten()
        .filter_map(|(k, v)| Some((k.strip_prefix("slot")?.parse().ok()?, v.get("matchup")?)))
        .collect();
    slots.sort_by_key(|(n, _)| *n);
    slots.into_iter().map(|(_, v)| v).collect()
}

fn collect_matchups<'a>(value: &'a Value, out: &mut Vec<&'a Value>) {
    match value {
        Value::Object(map) => {
            if let Some(m) = map.get("matchup") {
                out.push(m);
            }
            for (k, v) in map {
                if k != "matchup" {
                    collect_matchups(v, out);
                }
            }
        }
        Value::Array(items) => items.iter().for_each(|v| collect_matchups(v, out)),
        _ => {}
    }
}

/// A bracket slot's Match; `None` while HLTV has not created it yet.
fn bracket_match(
    matchup: &Value,
    placement: MatchPlacement,
    teams: &mut Vec<Team>,
) -> Result<Option<Match>, SourceError> {
    let m = &matchup["match"];
    if m.is_null() {
        return Ok(None);
    }
    let id = m["matchId"]
        .as_i64()
        .map(MatchId)
        .ok_or_else(|| parse_error("bracket match without an id".into()))?;
    let maps = m["numberOfMaps"].as_u64().unwrap_or_default();
    let format = u8::try_from(maps)
        .ok()
        .and_then(MatchFormat::from_maps)
        .ok_or_else(|| parse_error(format!("match {id}: {maps} maps")))?;
    let start = m["startTime"]
        .as_f64()
        .and_then(|ms| Utc.timestamp_millis_opt(ms as i64).single());
    let team_a = bracket_team(&matchup["team1"], teams)?;
    let team_b = bracket_team(&matchup["team2"], teams)?;

    let finished = !matchup["resultId"].is_null();
    let (status, winner, score) = if finished {
        let s = &matchup["score"];
        let (Some(sa), Some(sb), Some(first_won)) = (
            s["team1Score"].as_u64(),
            s["team2Score"].as_u64(),
            s["team1Winner"].as_bool(),
        ) else {
            return Err(parse_error(format!("finished match {id} has no score")));
        };
        let winner = if first_won { team_a } else { team_b };
        if winner.is_none() {
            return Err(parse_error(format!("finished match {id} has no known winner")));
        }
        let score = MatchScore { team_a: sa as u8, team_b: sb as u8 };
        (MatchStatus::Finished, winner, Some(score))
    } else {
        let status = match m["status"].as_str() {
            Some("Normal") => MatchStatus::Upcoming,
            Some("Live") => MatchStatus::Live,
            other => {
                return Err(parse_error(format!("match {id}: unknown status {other:?}")));
            }
        };
        (status, None, None)
    };

    Ok(Some(Match {
        id,
        stage: StageOrdinal(0),
        placement,
        team_a,
        team_b,
        format,
        scheduled_start: start,
        actual_start: None,
        status,
        winner,
        score,
    }))
}

/// `{"type":"known","id":{"teamId":..},"name":..,"logo":{..}}`, or a
/// placeholder (`text`, `oneOf`, ...) for a team still to be decided.
fn bracket_team(value: &Value, teams: &mut Vec<Team>) -> Result<Option<TeamId>, SourceError> {
    if value["type"].as_str() != Some("known") {
        return Ok(None);
    }
    let id = value["id"]["teamId"]
        .as_i64()
        .map(TeamId)
        .ok_or_else(|| parse_error("bracket team without an id".into()))?;
    let name = value["name"]
        .as_str()
        .filter(|n| !n.is_empty())
        .ok_or_else(|| parse_error(format!("team {id} has no name")))?;
    let logo = ["nightLogoURL", "dayLogoURL"]
        .iter()
        .find_map(|k| value["logo"][*k].as_str())
        .map(absolute_url);
    add_team(teams, Team { id, name: name.to_string(), logo_url: logo });
    Ok(Some(id))
}

fn add_team(teams: &mut Vec<Team>, team: Team) {
    if !teams.iter().any(|t| t.id == team.id) {
        teams.push(team);
    }
}

fn absolute_url(src: &str) -> String {
    if src.starts_with('/') {
        format!("https://www.hltv.org{src}")
    } else {
        src.to_string()
    }
}

/// `/team/12394/betboom` -> 12394.
fn team_href(href: &str) -> Result<TeamId, SourceError> {
    href.strip_prefix("/team/")
        .and_then(|rest| rest.split('/').next()?.parse().ok())
        .map(TeamId)
        .ok_or_else(|| parse_error(format!("bad team link {href:?}")))
}

/// `/matches/2394846/betboom-vs-liquid-...` -> 2394846.
fn match_href(href: &str) -> Result<MatchId, SourceError> {
    href.strip_prefix("https://www.hltv.org")
        .unwrap_or(href)
        .strip_prefix("/matches/")
        .and_then(|rest| rest.split('/').next()?.parse().ok())
        .map(MatchId)
        .ok_or_else(|| parse_error(format!("bad match link {href:?}")))
}

fn meta(doc: &Html, property: &str) -> Option<String> {
    doc.select(&sel(&format!("meta[property=\"{property}\"]")))
        .find_map(|m| m.value().attr("content"))
        .map(|c| c.trim().to_string())
        .filter(|c| !c.is_empty())
}

/// Cloudflare's interstitial ("Just a moment...") instead of an HLTV page.
pub fn is_challenge_page(html: &str) -> bool {
    is_challenge(&Html::parse_document(html))
}

fn is_challenge(doc: &Html) -> bool {
    doc.select(&sel("title"))
        .next()
        .is_some_and(|t| text_of(t).starts_with("Just a moment"))
}

fn text_of(el: ElementRef<'_>) -> String {
    el.text().collect::<String>().trim().to_string()
}

/// Current and upcoming top Events from `https://www.hltv.org/events`.
pub fn event_list(html: &str) -> Result<Vec<Event>, SourceError> {
    let doc = Html::parse_document(html);
    let mut events: Vec<(Event, String)> = Vec::new();
    for a in doc.select(&sel("a.ongoing-event, a.big-event")) {
        let href = a.value().attr("href").unwrap_or_default();
        let (id, slug) = event_href(href)
            .ok_or_else(|| parse_error(format!("event link without an id: {href:?}")))?;
        if events.iter().any(|(e, _)| e.id == id) {
            continue;
        }
        let name = first_text(a, ".event-name-small .text-ellipsis, .big-event-name")
            .ok_or_else(|| parse_error(format!("event {id} has no name")))?;
        let dates: Vec<DateTime<Utc>> = a
            .select(&sel("[data-unix]"))
            .map(|d| unix_ms(d.value().attr("data-unix").unwrap_or_default()))
            .collect::<Result<_, _>>()?;
        let event = Event {
            id,
            name,
            starts_at: dates.first().copied(),
            ends_at: dates.get(1).copied(),
        };
        events.push((event, slug));
    }
    if events.is_empty() {
        return Err(parse_error("the event list has no events".into()));
    }
    // A Stage of a Major is its own HLTV event; list only the Major itself.
    let slugs: Vec<String> = events.iter().map(|(_, s)| s.clone()).collect();
    Ok(events
        .into_iter()
        .filter(|(_, slug)| !slugs.iter().any(|parent| is_stage_slug_of(slug, parent)))
        .map(|(e, _)| e)
        .collect())
}

/// `pgl-major-singapore-2026-stage-1` is a Stage of `pgl-major-singapore-2026`.
fn is_stage_slug_of(slug: &str, parent: &str) -> bool {
    slug.strip_prefix(parent)
        .is_some_and(|rest| rest.starts_with("-stage-"))
}

/// The Matches being played right now, from `https://www.hltv.org/matches`.
/// Event pages do not say reliably whether a Match is live; this list does.
pub fn live_matches(html: &str) -> Result<Vec<MatchId>, SourceError> {
    let doc = Html::parse_document(html);
    if is_challenge(&doc) {
        return Err(SourceError::Unavailable(
            "hltv: Cloudflare challenge instead of the page".into(),
        ));
    }
    let wrappers: Vec<ElementRef> = doc.select(&sel("div.match-wrapper[data-match-id]")).collect();
    if wrappers.is_empty() {
        return Err(parse_error("not the HLTV match list (no matches)".into()));
    }
    let mut live = Vec::new();
    for w in wrappers {
        if w.value().attr("live") != Some("true") {
            continue;
        }
        let raw = w.value().attr("data-match-id").unwrap_or_default();
        let id = raw
            .parse()
            .map(MatchId)
            .map_err(|_| parse_error(format!("bad match id {raw:?}")))?;
        if !live.contains(&id) {
            live.push(id);
        }
    }
    Ok(live)
}

pub(crate) fn parse_error(message: String) -> SourceError {
    SourceError::Parse(format!("hltv: {message}"))
}

fn sel(css: &str) -> Selector {
    Selector::parse(css).expect("static selector")
}

fn first_text(el: ElementRef<'_>, css: &str) -> Option<String> {
    el.select(&sel(css))
        .map(|e| e.text().collect::<String>().trim().to_string())
        .find(|t| !t.is_empty())
}

/// `/events/8302/pgl-major-singapore-2026` -> (8302, "pgl-major-singapore-2026").
fn event_href(href: &str) -> Option<(EventId, String)> {
    let rest = href
        .strip_prefix("https://www.hltv.org")
        .unwrap_or(href)
        .strip_prefix("/events/")?;
    let (id, slug) = rest.split_once('/')?;
    let slug = slug.split(['#', '?', '/']).next().unwrap_or_default();
    Some((EventId(id.parse().ok()?), slug.to_string()))
}

fn unix_ms(value: &str) -> Result<DateTime<Utc>, SourceError> {
    value
        .parse::<i64>()
        .ok()
        .and_then(|ms| Utc.timestamp_millis_opt(ms).single())
        .ok_or_else(|| parse_error(format!("bad data-unix timestamp {value:?}")))
}
