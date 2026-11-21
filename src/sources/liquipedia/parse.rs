//! The pure parser: Liquipedia MediaWiki API responses (page wikitext, page
//! lists, expanded team templates) -> the fallback adapter model. No network.
//!
//! What Liquipedia stores and what it does not (verified on saved pages):
//! - Teams are template keys (`m80`), not names; [`team_names`] reads the
//!   names the fetch layer expanded.
//! - A Match has no winner field. The winner comes from the maps' round
//!   totals, or from a forfeit marker (`score=W` / `score=FF`).
//! - A Swiss Match has no record. Each round's Matchlists are split by record
//!   bucket in a fixed order (High, Mid, Low = most wins first), which gives
//!   it.
//! - `hltv=` carries the HLTV Match id.

use std::collections::{BTreeSet, HashMap};

use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, TimeZone, Utc};
use serde_json::Value;

use super::wikitext::{self, Template, find_templates, headings, read_template, strip_comments};
use crate::domain::{
    MatchFormat, MatchId, MatchPlacement, MatchScore, MatchStatus, Stage, StageKind, StageOrdinal,
};
use crate::ports::SourceError;
use crate::sources::fallback::{ExternalEvent, ExternalMatch, ExternalSnapshot, ExternalTeam, Side};

/// A Match that is unfinished and started less than this ago is Live.
const LIVE_FOR: Duration = Duration::hours(12);

fn parse_err(what: impl Into<String>) -> SourceError {
    SourceError::Parse(what.into())
}

/// One wiki page's wikitext.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Page {
    pub title: String,
    pub wikitext: String,
}

/// Team template key -> team name.
pub type TeamNames = HashMap<String, String>;

// ---------------------------------------------------------------- API JSON

fn json(text: &str) -> Result<Value, SourceError> {
    let value: Value =
        serde_json::from_str(text).map_err(|e| parse_err(format!("not JSON: {e}")))?;
    if let Some(error) = value.get("error") {
        let code = error.get("code").and_then(Value::as_str).unwrap_or("?");
        return Err(match code {
            "ratelimited" | "maxlag" | "readonly" => {
                SourceError::Unavailable(format!("the wiki answered {code}"))
            }
            _ => parse_err(format!("the wiki answered error {code}")),
        });
    }
    Ok(value)
}

/// Pages of an `action=query&prop=revisions&rvslots=main&formatversion=2`
/// answer. Missing pages are left out.
pub fn query_pages(text: &str) -> Result<Vec<Page>, SourceError> {
    let value = json(text)?;
    let pages = value
        .pointer("/query/pages")
        .and_then(Value::as_array)
        .ok_or_else(|| parse_err("no query.pages in the answer"))?;
    let mut out = Vec::new();
    for page in pages {
        if page.get("missing").is_some() {
            continue;
        }
        let title = page
            .get("title")
            .and_then(Value::as_str)
            .ok_or_else(|| parse_err("a page without a title"))?;
        let wikitext = page
            .pointer("/revisions/0/slots/main/content")
            .and_then(Value::as_str)
            .ok_or_else(|| parse_err(format!("page {title} has no wikitext")))?;
        out.push(Page {
            title: title.to_string(),
            wikitext: wikitext.to_string(),
        });
    }
    Ok(out)
}

/// Titles of an `action=query&list=allpages` or `list=search` answer.
pub fn listed_titles(text: &str) -> Result<Vec<String>, SourceError> {
    let value = json(text)?;
    let list = value
        .pointer("/query/allpages")
        .or_else(|| value.pointer("/query/search"))
        .and_then(Value::as_array)
        .ok_or_else(|| parse_err("no page list in the answer"))?;
    list.iter()
        .map(|p| {
            p.get("title")
                .and_then(Value::as_str)
                .map(str::to_string)
                .ok_or_else(|| parse_err("a listed page without a title"))
        })
        .collect()
}

/// The subpages of `root` that can hold a Stage (`Stage 1`, `Playoffs`, ...),
/// in play order: everything before the Playoffs, numbered Stages in number
/// order.
pub fn stage_subpages(titles: &[String], root: &str) -> Vec<String> {
    const SKIP: [&str; 6] = [
        "qualif", "invite", "showmatch", "statistic", "prize", "participants",
    ];
    let mut subs: Vec<(bool, String, u32, &String)> = titles
        .iter()
        .filter_map(|title| {
            let rest = title.strip_prefix(root)?.strip_prefix('/')?;
            let lower = rest.to_lowercase();
            if rest.contains('/') || SKIP.iter().any(|s| lower.contains(s)) {
                return None;
            }
            let digits_at = rest.find(|c: char| c.is_ascii_digit());
            let (prefix, number) = match digits_at {
                Some(i) => (&rest[..i], rest[i..].parse().unwrap_or(0)),
                None => (rest, 0),
            };
            Some((
                lower.contains("playoff"),
                prefix.to_lowercase(),
                number,
                title,
            ))
        })
        .collect();
    subs.sort();
    subs.into_iter().map(|(_, _, _, t)| t.clone()).collect()
}

/// The text the fetch layer sends to `action=expandtemplates` to learn the
/// names of `keys`: one line `key@@{{Team|key}}` each. Keys that could
/// not be put in a template call safely are left out (they keep their key as
/// name).
pub fn team_name_query(keys: &BTreeSet<String>) -> String {
    keys.iter()
        .filter(|k| !k.contains(['|', '{', '}', '[', ']', '\n', '@', '=']))
        .map(|k| format!("{k}@@{{{{Team|{k}}}}}\n"))
        .collect()
}

/// Reads the answer to [`team_name_query`]. A key the wiki could not resolve
/// (it answers with an error element) is left out.
pub fn team_names(text: &str) -> Result<TeamNames, SourceError> {
    let value = json(text)?;
    let expanded = value
        .pointer("/expandtemplates/wikitext")
        .and_then(Value::as_str)
        .ok_or_else(|| parse_err("no expandtemplates.wikitext in the answer"))?;
    // `{{Team|key}}` renders a span holding the team's link:
    // `<span class="team-template-text">[[Natus Vincere|Natus Vincere]]</span>`.
    // (`{{TeamName|key}}` is no use: it answers "100x50px" for teams with
    // a logo.)
    const MARK: &str = "team-template-text\">[[";
    Ok(expanded
        .lines()
        .filter_map(|line| {
            let (key, rendered) = line.split_once("@@")?;
            let link = rendered.split_once(MARK)?.1.split_once("]]")?.0;
            let name = link.rsplit('|').next()?.trim();
            (!name.is_empty()).then(|| (key.to_string(), name.to_string()))
        })
        .collect())
}

// ------------------------------------------------------------------- Event

fn date(value: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(value.trim(), "%Y-%m-%d").ok()
}

/// The Event an `Infobox league` describes; `None` when the text has none
/// (a player page, a Stage subpage, ...).
pub fn event_header(title: &str, wikitext: &str) -> Option<ExternalEvent> {
    let text = strip_comments(wikitext);
    let infobox = find_templates(&text, "Infobox league").into_iter().next()?;
    let name = ["tickername", "name"]
        .iter()
        .filter_map(|k| infobox.get(k))
        .find(|v| !v.is_empty())?;
    Some(ExternalEvent {
        external_id: title.to_string(),
        canonical: None,
        name: name.to_string(),
        starts_at: infobox
            .get("sdate")
            .and_then(date)
            .map(|d| Utc.from_utc_datetime(&d.and_hms_opt(0, 0, 0).unwrap())),
        ends_at: infobox
            .get("edate")
            .and_then(date)
            .map(|d| Utc.from_utc_datetime(&d.and_hms_opt(23, 59, 59).unwrap())),
    })
}

// ------------------------------------------------------------------- dates

/// Offsets of the zone abbreviations Liquipedia prints (`{{Abbr/CEST}}`).
/// Ambiguous ones (`CST`, `IST`) are left out on purpose: they fail the parse
/// instead of guessing a Scheduled Start.
fn zone_offset_minutes(abbr: &str) -> Option<i64> {
    Some(match abbr.to_ascii_uppercase().as_str() {
        "UTC" | "GMT" | "WET" => 0,
        "BST" | "WEST" | "CET" => 60,
        "CEST" | "EET" => 120,
        "EEST" | "MSK" | "TRT" => 180,
        "GST" => 240,
        "PKT" => 300,
        "ICT" | "WIB" => 420,
        "SGT" | "HKT" | "AWST" | "PHT" => 480,
        "KST" | "JST" => 540,
        "AEST" => 600,
        "AEDT" => 660,
        "NZST" => 720,
        "NZDT" => 780,
        "ART" | "BRT" => -180,
        "AST" => -240,
        "EST" | "COT" => -300,
        "EDT" => -240,
        "CDT" => -300,
        "MST" => -420,
        "MDT" => -360,
        "PST" => -480,
        "PDT" => -420,
        _ => return None,
    })
}

const MONTHS: [&str; 12] = [
    "January", "February", "March", "April", "May", "June", "July", "August", "September",
    "October", "November", "December",
];

/// `June 2, 2026 - 12:30 {{Abbr/CEST}}` -> UTC. `Ok(None)` for no date, or a
/// date without a time (a day is no Scheduled Start).
fn start_time(value: &str) -> Result<Option<DateTime<Utc>>, SourceError> {
    let value = value.trim();
    let Some((day, time)) = value.split_once(" - ") else {
        return Ok(None);
    };
    let bad = || parse_err(format!("unreadable match date {value:?}"));
    let mut words = day.split_whitespace();
    let month = words
        .next()
        .and_then(|m| MONTHS.iter().position(|n| n.eq_ignore_ascii_case(m)))
        .ok_or_else(bad)?;
    let dom: u32 = words
        .next()
        .and_then(|d| d.trim_end_matches(',').parse().ok())
        .ok_or_else(bad)?;
    let year: i32 = words.next().and_then(|y| y.parse().ok()).ok_or_else(bad)?;

    let (clock, zone) = match time.split_once("{{Abbr/") {
        Some((clock, zone)) => (clock, zone.trim_end_matches("}}").to_string()),
        None => {
            let mut it = time.split_whitespace();
            (
                it.next().unwrap_or(""),
                it.next().unwrap_or("").to_string(),
            )
        }
    };
    let (h, m) = clock.trim().split_once(':').ok_or_else(bad)?;
    let offset = zone_offset_minutes(zone.trim())
        .ok_or_else(|| parse_err(format!("unknown time zone {zone:?} in {value:?}")))?;
    let local: NaiveDateTime = NaiveDate::from_ymd_opt(year, month as u32 + 1, dom)
        .and_then(|d| d.and_hms_opt(h.trim().parse().ok()?, m.trim().parse().ok()?, 0))
        .ok_or_else(bad)?;
    Ok(Some(
        Utc.from_utc_datetime(&local) - Duration::minutes(offset),
    ))
}

// ------------------------------------------------------------------- Match

#[derive(Debug, Clone)]
struct RawMatch {
    /// Stable position in its page (`R1M3`, `R2.0.4`), never reordered by
    /// the wiki's own ids.
    key: String,
    canonical: Option<MatchId>,
    placement: MatchPlacement,
    team_a: Option<String>,
    team_b: Option<String>,
    format: MatchFormat,
    start: Option<DateTime<Utc>>,
    finished: bool,
    /// Some map has been played to the end.
    in_play: bool,
    winner: Option<Side>,
    score: Option<MatchScore>,
}

fn opponent(value: Option<&str>) -> Result<(Option<String>, Option<String>), SourceError> {
    let Some(value) = value else {
        return Ok((None, None));
    };
    let Some(t) = read_template(value, 0).filter(|_| value.starts_with("{{")) else {
        return Err(parse_err(format!("unreadable opponent {value:?}")));
    };
    if !t.is("TeamOpponent") {
        return Err(parse_err(format!("opponent is not a team: {}", t.name)));
    }
    let key = t.positional().next().unwrap_or("").trim().to_string();
    let team = (!key.is_empty()
        && !key.eq_ignore_ascii_case("tbd")
        && !key.eq_ignore_ascii_case("bye"))
    .then_some(key);
    Ok((team, t.get("score").map(|s| s.trim().to_uppercase())))
}

/// Rounds each side scored on one `{{Map|..}}`, overtime included: the sum of
/// `t1t`, `t1ct`, `o<N>t1t`, `o<N>t1ct` for side 1 and the `t2..` ones for
/// side 2.
fn map_rounds(map: &Template) -> (u32, u32) {
    let (mut a, mut b) = (0, 0);
    for arg in &map.args {
        let Some(key) = arg.key.as_deref() else {
            continue;
        };
        let key = key.to_ascii_lowercase();
        let rest = match key.strip_prefix('o') {
            Some(r) => r.trim_start_matches(|c: char| c.is_ascii_digit()),
            None => key.as_str(),
        };
        let rounds: u32 = arg.value.parse().unwrap_or(0);
        match rest {
            "t1t" | "t1ct" => a += rounds,
            "t2t" | "t2ct" => b += rounds,
            _ => {}
        }
    }
    (a, b)
}

fn read_match(
    t: &Template,
    key: String,
    placement: MatchPlacement,
) -> Result<RawMatch, SourceError> {
    let (team_a, score_a) = opponent(t.get("opponent1"))?;
    let (team_b, score_b) = opponent(t.get("opponent2"))?;
    let start = start_time(t.get("date").unwrap_or(""))?;
    let finished = t.get("finished").is_some_and(|f| f.eq_ignore_ascii_case("true"));
    let canonical = t
        .get("hltv")
        .and_then(|v| v.trim().parse::<i64>().ok())
        .map(MatchId);

    let mut map_count = 0u8;
    let (mut won_a, mut won_b, mut played) = (0u8, 0u8, 0u8);
    for arg in &t.args {
        let Some(k) = arg.key.as_deref() else {
            continue;
        };
        let is_map = k.len() > 3
            && k[..3].eq_ignore_ascii_case("map")
            && k[3..].chars().all(|c| c.is_ascii_digit());
        if !is_map {
            continue;
        }
        map_count += 1;
        let Some(map) = read_template(&arg.value, 0).filter(|_| arg.value.starts_with("{{"))
        else {
            continue;
        };
        if !map.get("finished").is_some_and(|f| f.eq_ignore_ascii_case("true")) {
            continue;
        }
        played += 1;
        let (a, b) = map_rounds(&map);
        match a.cmp(&b) {
            std::cmp::Ordering::Greater => won_a += 1,
            std::cmp::Ordering::Less => won_b += 1,
            std::cmp::Ordering::Equal => {}
        }
    }

    let numeric = |s: &Option<String>| s.as_deref().and_then(|s| s.parse::<u8>().ok());
    let is_win = |s: &Option<String>| s.as_deref() == Some("W");
    let (winner, score) = if is_win(&score_a) || is_win(&score_b) {
        // A forfeit: no maps to count.
        (Some(if is_win(&score_a) { Side::A } else { Side::B }), None)
    } else if let (Some(a), Some(b)) = (numeric(&score_a), numeric(&score_b)) {
        (side_of(a, b), Some(MatchScore { team_a: a, team_b: b }))
    } else if played > 0 {
        (
            side_of(won_a, won_b),
            Some(MatchScore { team_a: won_a, team_b: won_b }),
        )
    } else {
        (None, None)
    };

    let format = t
        .get("bestof")
        .and_then(|b| b.trim().parse().ok())
        .or(Some(map_count))
        .and_then(|n: u8| match n {
            0 => None,
            1 => MatchFormat::from_maps(1),
            2 | 3 => Some(MatchFormat::Bo3),
            _ => Some(MatchFormat::Bo5),
        })
        .unwrap_or(MatchFormat::Bo3);

    Ok(RawMatch {
        key,
        canonical,
        placement,
        team_a,
        team_b,
        format,
        start,
        finished,
        in_play: played > 0,
        winner: if finished { winner } else { None },
        score,
    })
}

fn side_of(a: u8, b: u8) -> Option<Side> {
    match a.cmp(&b) {
        std::cmp::Ordering::Greater => Some(Side::A),
        std::cmp::Ordering::Less => Some(Side::B),
        std::cmp::Ordering::Equal => None,
    }
}

/// An empty slot of a round not yet paired: nothing to predict, and nothing
/// to link either.
fn is_placeholder(m: &RawMatch) -> bool {
    m.team_a.is_none() && m.team_b.is_none() && m.canonical.is_none()
}

// ------------------------------------------------------------------ Stages

#[derive(Debug, Clone)]
struct RawStage {
    /// Prefix of its Matches' external ids: page title and the stage's
    /// position on the page.
    id: String,
    kind: StageKind,
    name: String,
    matches: Vec<RawMatch>,
}

/// The structure of one Event as read from its pages, before team names are
/// known.
#[derive(Debug, Clone)]
pub struct Parsed {
    event: ExternalEvent,
    stages: Vec<RawStage>,
}

/// Parses an Event from its root page and the pages of its Stage subpages
/// (see [`stage_subpages`]), in play order. `pages[0]` is the root page.
pub fn parse_event(root: &str, pages: &[Page]) -> Result<Parsed, SourceError> {
    let root_page = pages
        .iter()
        .find(|p| p.title == root)
        .ok_or_else(|| parse_err(format!("root page {root} missing")))?;
    let event = event_header(root, &root_page.wikitext)
        .ok_or_else(|| parse_err(format!("{root} has no league infobox")))?;

    let mut stages = Vec::new();
    for page in pages {
        let subpage = page
            .title
            .strip_prefix(root)
            .and_then(|r| r.strip_prefix('/'))
            .map(str::to_string);
        stages.extend(page_stages(page, subpage.as_deref())?);
    }
    if stages.is_empty() {
        return Err(parse_err(format!("no Swiss or bracket stage found for {root}")));
    }
    Ok(Parsed { event, stages })
}

enum Section {
    Swiss(Template),
    Bracket(Template),
}

fn section_name(
    heads: &[wikitext::Heading],
    pos: usize,
    subpage: Option<&str>,
    alone: bool,
    fallback: &str,
) -> String {
    if alone && let Some(s) = subpage {
        return s.to_string();
    }
    // Headings under the page's `Results` heading only (`Group Stage`,
    // `Playoffs`); sections before it (`Participants`, ...) name nothing.
    let before: Vec<&wikitext::Heading> = heads.iter().filter(|h| h.pos < pos).collect();
    let results = before
        .iter()
        .rposition(|h| h.title.eq_ignore_ascii_case("results"));
    const GENERIC: [&str; 3] = ["overview", "detailed results", "bracket"];
    results
        .into_iter()
        .flat_map(|i| before[i + 1..].iter().rev())
        .map(|h| h.title.as_str())
        .find(|t| {
            !GENERIC.contains(&t.to_lowercase().as_str()) && !t.to_lowercase().starts_with("round")
        })
        .map(str::to_string)
        .or_else(|| subpage.map(str::to_string))
        .unwrap_or_else(|| fallback.to_string())
}

fn page_stages(page: &Page, subpage: Option<&str>) -> Result<Vec<RawStage>, SourceError> {
    let text = strip_comments(&page.wikitext);
    let mut sections: Vec<(usize, Section)> = Vec::new();
    for t in find_templates(&text, "SwissStandings") {
        sections.push((t.start, Section::Swiss(t)));
    }
    for t in find_templates(&text, "Bracket") {
        sections.push((t.start, Section::Bracket(t)));
    }
    sections.sort_by_key(|(pos, _)| *pos);
    if sections.is_empty() {
        return Ok(Vec::new());
    }

    let heads = headings(&text);
    let alone = sections.len() == 1;
    let mut out = Vec::new();
    for (n, (pos, section)) in sections.into_iter().enumerate() {
        let id = format!("{}#{n}", page.title);
        let at = |what: &str| format!("{} {}", page.title, what);
        match section {
            Section::Swiss(t) => {
                if let Some(format) = t.get("format")
                    && !format.eq_ignore_ascii_case("swiss3")
                {
                    return Err(parse_err(format!("{}: unsupported Swiss format {format}", page.title)));
                }
                out.push(RawStage {
                    id,
                    kind: StageKind::Swiss,
                    name: section_name(&heads, pos, subpage, alone, "Swiss"),
                    matches: swiss_matches(&text, &heads, &page.title).map_err(|e| match e {
                        SourceError::Parse(m) => parse_err(at(&m)),
                        e => e,
                    })?,
                });
            }
            Section::Bracket(t) => {
                let single_elimination = t
                    .positional()
                    .next()
                    .and_then(|n| n.strip_prefix("Bracket/"))
                    .and_then(|n| n.trim().parse::<u32>().ok())
                    .filter(|n| *n >= 2 && n.is_power_of_two());
                let kind = if single_elimination.is_some() {
                    StageKind::Playoffs
                } else {
                    StageKind::Groups
                };
                let fallback = if kind == StageKind::Playoffs { "Playoffs" } else { "Bracket" };
                out.push(RawStage {
                    id,
                    kind,
                    name: section_name(&heads, pos, subpage, alone, fallback),
                    matches: bracket_matches(&t, single_elimination)?,
                });
            }
        }
    }
    Ok(out)
}

fn match_templates(list: &Template) -> impl Iterator<Item = (usize, Template)> + '_ {
    list.args
        .iter()
        .filter(|a| a.key.is_none())
        .filter_map(|a| read_template(&a.value, 0).filter(|_| a.value.starts_with("{{")))
        .filter(|t| t.is("Match"))
        .enumerate()
}

fn swiss_matches(
    text: &str,
    heads: &[wikitext::Heading],
    title: &str,
) -> Result<Vec<RawMatch>, SourceError> {
    // Matchlists per round, in page order.
    let mut rounds: Vec<(u8, Vec<Template>)> = Vec::new();
    for list in find_templates(text, "Matchlist") {
        let round = heads
            .iter()
            .rev()
            .filter(|h| h.pos < list.start)
            .find(|h| h.title.starts_with("Round "))
            .map(|h| h.title.as_str())
            .and_then(|t| t.strip_prefix("Round "))
            .and_then(|n| n.trim().parse::<u8>().ok());
        let Some(round) = round else {
            continue; // a list outside the Swiss rounds
        };
        match rounds.last_mut() {
            Some((r, lists)) if *r == round => lists.push(list),
            _ => rounds.push((round, vec![list])),
        }
    }

    let mut out = Vec::new();
    for (round, lists) in rounds {
        // Record buckets of this round, most wins first; a team with 3 wins
        // or 3 losses has left the Stage.
        let buckets: Vec<(u8, u8)> = (0..round)
            .rev()
            .map(|w| (w, round - 1 - w))
            .filter(|(w, l)| *w < 3 && *l < 3)
            .collect();
        if buckets.len() != lists.len() {
            return Err(parse_err(format!(
                "{title}: Round {round} has {} match lists, expected {}",
                lists.len(),
                buckets.len()
            )));
        }
        for (b, list) in lists.iter().enumerate() {
            let (wins, losses) = buckets[b];
            for (i, t) in match_templates(list) {
                let m = read_match(
                    &t,
                    format!("R{round}.{b}.{i}"),
                    MatchPlacement::Swiss { round, wins, losses },
                )?;
                if !is_placeholder(&m) {
                    out.push(m);
                }
            }
        }
    }
    Ok(out)
}

fn bracket_matches(
    bracket: &Template,
    single_elimination: Option<u32>,
) -> Result<Vec<RawMatch>, SourceError> {
    let total_rounds = single_elimination.map(|n| n.trailing_zeros() as u8);
    let mut out = Vec::new();
    for arg in &bracket.args {
        let Some(key) = arg.key.as_deref() else {
            continue;
        };
        if !arg.value.starts_with("{{") {
            continue;
        }
        let Some(t) = read_template(&arg.value, 0).filter(|t| t.is("Match")) else {
            continue;
        };
        // R<round>M<slot>, both counting from 1.
        let position = key
            .strip_prefix('R')
            .and_then(|k| k.split_once('M'))
            .and_then(|(r, m)| Some((r.parse::<u8>().ok()?, m.parse::<u8>().ok()?)));
        let placement = match (position, total_rounds) {
            (Some((r, m)), Some(total)) if r >= 1 && r <= total && m >= 1 => {
                let before_final = total - r;
                if u32::from(m - 1) < 1u32 << before_final {
                    MatchPlacement::Playoffs {
                        rounds_before_final: before_final,
                        slot: m - 1,
                    }
                } else {
                    MatchPlacement::Unplaced // a third-place match
                }
            }
            _ => MatchPlacement::Unplaced,
        };
        let m = read_match(&t, key.to_string(), placement)?;
        if !is_placeholder(&m) {
            out.push(m);
        }
    }
    Ok(out)
}

// ---------------------------------------------------------------- Snapshot

impl Parsed {
    pub fn event(&self) -> &ExternalEvent {
        &self.event
    }

    /// Team template keys the Event uses.
    pub fn team_keys(&self) -> BTreeSet<String> {
        self.stages
            .iter()
            .flat_map(|s| &s.matches)
            .flat_map(|m| [&m.team_a, &m.team_b])
            .flatten()
            .cloned()
            .collect()
    }

    /// The snapshot, with team names from `names` (a team without one is
    /// named by its key) and Match status judged at `now`.
    pub fn into_snapshot(self, names: &TeamNames, now: DateTime<Utc>) -> ExternalSnapshot {
        let team = |key: Option<String>| {
            key.map(|key| ExternalTeam {
                name: names.get(&key).cloned().unwrap_or_else(|| key.clone()),
                external_id: key,
                canonical: None,
            })
        };
        let mut stages = Vec::new();
        let mut matches = Vec::new();
        for (i, stage) in self.stages.into_iter().enumerate() {
            let ordinal = StageOrdinal(i as u8 + 1);
            stages.push(Stage {
                ordinal,
                kind: stage.kind,
                name: stage.name.clone(),
            });
            for m in stage.matches {
                let status = if m.finished {
                    MatchStatus::Finished
                } else if m.in_play
                    || m.start.is_some_and(|s| s <= now && now - s < LIVE_FOR)
                {
                    MatchStatus::Live
                } else {
                    MatchStatus::Upcoming
                };
                matches.push(ExternalMatch {
                    external_id: format!("{}.{}", stage.id, m.key),
                    canonical: m.canonical,
                    stage: ordinal,
                    placement: m.placement,
                    team_a: team(m.team_a),
                    team_b: team(m.team_b),
                    format: m.format,
                    scheduled_start: m.start,
                    actual_start: None,
                    status,
                    winner: m.winner,
                    score: m.score,
                });
            }
        }
        ExternalSnapshot {
            event: self.event,
            stages,
            matches,
        }
    }
}
