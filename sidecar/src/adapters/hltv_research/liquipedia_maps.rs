//! Pure parser for post-Match Liquipedia Stage pages (map names, half & overtime
//! round scores, skipped maps, HLTV match IDs, and HLTV mapstatsids).
//! Reuses `adapters::context::wikitext`; no network I/O.

use super::archive::{LiquipediaMapResult, LiquipediaMatchMaps, LiquipediaStageMaps, Side};
use super::parse::ParseError;
use crate::adapters::context::wikitext::{Template, find_templates, read_template, strip_comments};
use chrono::{DateTime, Utc};
use serde_json::Value;

fn opponent_key(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    let start = trimmed.find("{{")?;
    let tpl = read_template(trimmed, start)?;
    let key = tpl.positional().next()?.trim();
    (!key.is_empty()).then(|| key.to_ascii_lowercase())
}

fn parse_u32(tpl: &Template, key: &str) -> Option<u32> {
    tpl.get(key)?.trim().parse::<u32>().ok()
}

fn sum_team_rounds(tpl: &Template, team_num: u8) -> Option<u32> {
    let t_key = format!("t{team_num}t");
    let ct_key = format!("t{team_num}ct");
    let reg_t = parse_u32(tpl, &t_key);
    let reg_ct = parse_u32(tpl, &ct_key);
    let mut total = match (reg_t, reg_ct) {
        (Some(a), Some(b)) => a + b,
        (Some(a), None) => a,
        (None, Some(b)) => b,
        (None, None) => return None,
    };
    for ot in 1..=10 {
        let ot_t = parse_u32(tpl, &format!("o{ot}t{team_num}t")).unwrap_or(0);
        let ot_ct = parse_u32(tpl, &format!("o{ot}t{team_num}ct")).unwrap_or(0);
        total += ot_t + ot_ct;
    }
    Some(total)
}

fn parse_map_entry(position: u8, raw: &str) -> Option<LiquipediaMapResult> {
    let start = raw.find("{{")?;
    let tpl = read_template(raw, start)?;
    if !tpl.is("Map") {
        return None;
    }
    let map_name = tpl.get("map")?.trim().to_string();
    if map_name.is_empty() {
        return None;
    }
    let finished = tpl.get("finished").unwrap_or("").trim();
    let played = finished.eq_ignore_ascii_case("true");
    let team_a_first_side = tpl
        .get("t1firstside")
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_ascii_lowercase);
    let team_a_rounds = if played { sum_team_rounds(&tpl, 1) } else { None };
    let team_b_rounds = if played { sum_team_rounds(&tpl, 2) } else { None };
    let winner = match (team_a_rounds, team_b_rounds) {
        (Some(a), Some(b)) if a > b => Some(Side::A),
        (Some(a), Some(b)) if b > a => Some(Side::B),
        _ => match tpl.get("winner").map(str::trim) {
            Some("1") => Some(Side::A),
            Some("2") => Some(Side::B),
            _ => None,
        },
    };
    let mapstats_id = tpl
        .get("stats")
        .and_then(|s| s.trim().parse::<i64>().ok());

    Some(LiquipediaMapResult {
        position,
        map_name,
        played,
        team_a_first_side,
        team_a_rounds,
        team_b_rounds,
        winner,
        mapstats_id,
    })
}

/// Parses a MediaWiki `action=query&prop=revisions&formatversion=2` JSON response
/// of a post-Match Liquipedia Stage page into per-Match map results.
pub fn parse_stage_maps(json: &str) -> Result<LiquipediaStageMaps, ParseError> {
    let v: Value = serde_json::from_str(json)
        .map_err(|e| ParseError::InvalidMarkup(format!("invalid Liquipedia JSON: {e}")))?;
    let page = v
        .pointer("/query/pages/0")
        .ok_or_else(|| ParseError::InvalidMarkup("missing query.pages[0]".into()))?;
    let title = page
        .get("title")
        .and_then(Value::as_str)
        .ok_or_else(|| ParseError::InvalidMarkup("missing page title".into()))?
        .to_string();
    let rev = page
        .pointer("/revisions/0")
        .ok_or_else(|| ParseError::InvalidMarkup("missing revisions[0]".into()))?;
    let revid = rev
        .get("revid")
        .and_then(Value::as_i64)
        .ok_or_else(|| ParseError::InvalidMarkup("missing revid".into()))?;
    let revision_timestamp: DateTime<Utc> = rev
        .get("timestamp")
        .and_then(Value::as_str)
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| ParseError::InvalidMarkup("missing revision timestamp".into()))?;
    let content = rev
        .pointer("/slots/main/content")
        .and_then(Value::as_str)
        .ok_or_else(|| ParseError::InvalidMarkup("missing wikitext content".into()))?;

    let cleaned = strip_comments(content);
    let mut matches = Vec::new();
    for tpl in find_templates(&cleaned, "Match") {
        let Some(team_a_key) = tpl.get("opponent1").and_then(opponent_key) else {
            continue;
        };
        let Some(team_b_key) = tpl.get("opponent2").and_then(opponent_key) else {
            continue;
        };
        let mut maps = Vec::new();
        for pos in 1..=5u8 {
            if let Some(raw_map) = tpl.get(&format!("map{pos}"))
                && let Some(entry) = parse_map_entry(pos, raw_map)
            {
                maps.push(entry);
            }
        }
        if maps.is_empty() {
            continue;
        }
        let date_raw = tpl
            .get("date")
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string);
        let hltv_match_id = tpl
            .get("hltv")
            .and_then(|s| s.trim().parse::<i64>().ok());
        matches.push(LiquipediaMatchMaps {
            team_a_key,
            team_b_key,
            date_raw,
            hltv_match_id,
            maps,
        });
    }

    if matches.is_empty() {
        return Err(ParseError::InvalidMarkup(format!(
            "no matches with maps found on {title}"
        )));
    }

    Ok(LiquipediaStageMaps {
        title,
        revid,
        revision_timestamp,
        matches,
    })
}
