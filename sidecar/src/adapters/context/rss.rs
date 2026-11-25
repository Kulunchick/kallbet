//! Curated CS2 news RSS 2.0 parser and team-relevance filter (Seam 2).

use super::names::{contains_word_phrase, team_search_terms};
use crate::*;
use chrono::{DateTime, Duration, Utc};
use std::collections::BTreeSet;

const MAX_DESCRIPTION_CHARS: usize = 500;

/// Parses an RSS 2.0 XML document and returns all dated items strictly predating `cutoff`.
///
/// Rejects non-RSS payloads (such as Cloudflare HTML challenge pages) with an error.
pub fn parse_rss_items(
    xml: &str,
    feed_url: &str,
    cutoff: Instant,
) -> Result<Vec<AsOf<NewsItem>>> {
    let xml = xml.trim_start_matches('\u{feff}');
    let channel = extract_tag_inner(xml, "channel").ok_or("missing RSS <channel> element")?;

    // Channel header sits before the first <item>.
    let header = match find_open_tag(channel, "item") {
        Some((start, _)) => &channel[..start],
        None => channel,
    };
    let source = extract_tag_inner(header, "title")
        .map(decode_text)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| feed_url.to_string());

    let mut out = Vec::new();
    let mut cursor = 0;
    while let Some((item_body, next_pos)) = next_tag_inner(channel, cursor, "item") {
        cursor = next_pos;
        let Some(title) = extract_tag_inner(item_body, "title")
            .map(decode_text)
            .filter(|s| !s.is_empty())
        else {
            continue;
        };
        let Some(url) = extract_tag_inner(item_body, "link")
            .map(decode_text)
            .filter(|u| u.starts_with("http://") || u.starts_with("https://"))
        else {
            continue;
        };
        let Some(pub_raw) = extract_tag_inner(item_body, "pubDate").map(decode_text) else {
            continue;
        };
        let Ok(published_at) = parse_rss_date(&pub_raw) else {
            continue;
        };
        // Strict point-in-time barrier: exclude any item published at or after cutoff.
        if published_at >= cutoff {
            continue;
        }
        let description = extract_tag_inner(item_body, "description")
            .map(decode_text)
            .map(|d| clean_description(&d))
            .unwrap_or_default();

        let item = NewsItem {
            source: source.clone(),
            feed_url: feed_url.to_string(),
            title,
            description,
            url,
            published_at,
        };
        out.push(AsOf::new(item, published_at, cutoff)?);
    }
    Ok(out)
}

/// Filters parsed `AsOf<NewsItem>` entries to recent headlines strictly before `cutoff`
/// that mention `team_a` or `team_b` (or any of their aliases) on word boundaries.
#[allow(clippy::too_many_arguments)]
pub fn filter_match_news(
    items: &[AsOf<NewsItem>],
    team_a: &str,
    aliases_a: &[String],
    team_b: &str,
    aliases_b: &[String],
    cutoff: Instant,
    lookback: Duration,
    max_items: usize,
) -> Result<Vec<AsOf<NewsItemFact>>> {
    let terms_a = team_search_terms(team_a, aliases_a);
    let terms_b = team_search_terms(team_b, aliases_b);
    let oldest = cutoff - lookback;

    let mut matched: Vec<NewsItemFact> = Vec::new();
    for entry in items {
        if entry.observed_at() >= cutoff || entry.observed_at() < oldest {
            continue;
        }
        let item = entry.at(cutoff)?;
        let haystack = format!("{} {}", item.title, item.description);
        let mut matched_teams = Vec::new();
        if terms_a.iter().any(|t| contains_word_phrase(&haystack, t)) {
            matched_teams.push(team_a.to_string());
        }
        if terms_b.iter().any(|t| contains_word_phrase(&haystack, t)) {
            matched_teams.push(team_b.to_string());
        }
        if matched_teams.is_empty() {
            continue;
        }
        matched.push(NewsItemFact {
            source: item.source.clone(),
            feed_url: item.feed_url.clone(),
            title: item.title.clone(),
            description: item.description.clone(),
            url: item.url.clone(),
            published_at: item.published_at,
            matched_teams,
        });
    }

    // Deterministic order: newest first, then URL, deduplicated by URL.
    matched.sort_by(|a, b| {
        b.published_at
            .cmp(&a.published_at)
            .then_with(|| a.url.cmp(&b.url))
    });
    let mut seen_urls = BTreeSet::new();
    matched.retain(|item| seen_urls.insert(item.url.clone()));
    matched.truncate(max_items);

    matched
        .into_iter()
        .map(|fact| {
            let ts = fact.published_at;
            AsOf::new(fact, ts, cutoff)
        })
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(Into::into)
}

fn parse_rss_date(raw: &str) -> Result<Instant> {
    let trimmed = raw.trim();
    if let Ok(dt) = DateTime::parse_from_rfc2822(trimmed) {
        return Ok(dt.with_timezone(&Utc));
    }
    if let Ok(dt) = DateTime::parse_from_rfc3339(trimmed) {
        return Ok(dt.with_timezone(&Utc));
    }
    Err(format!("unsupported RSS date format: {trimmed}").into())
}

fn extract_tag_inner<'a>(xml: &'a str, tag: &str) -> Option<&'a str> {
    next_tag_inner(xml, 0, tag).map(|(body, _)| body)
}

fn next_tag_inner<'a>(xml: &'a str, from: usize, tag: &str) -> Option<(&'a str, usize)> {
    let slice = &xml[from..];
    let (_, open_end_rel) = find_open_tag(slice, tag)?;
    let content_start = from + open_end_rel;
    let close_needle = format!("</{tag}>");
    let close_pos = find_closing_tag(xml, content_start, &close_needle)?;
    Some((&xml[content_start..close_pos], close_pos + close_needle.len()))
}

/// Finds `<tag>` or `<tag ...>` skipping XML comments and CDATA blocks.
/// Returns `(start_offset, end_of_open_tag_offset)`. Self-closing tags are skipped.
fn find_open_tag(xml: &str, tag: &str) -> Option<(usize, usize)> {
    let bytes = xml.as_bytes();
    let tag_bytes = tag.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i..].starts_with(b"<![CDATA[") {
            let end = xml[i + 9..].find("]]>")?;
            i += 9 + end + 3;
            continue;
        }
        if bytes[i..].starts_with(b"<!--") {
            let end = xml[i + 4..].find("-->")?;
            i += 4 + end + 3;
            continue;
        }
        if bytes[i] == b'<'
            && bytes
                .get(i + 1..i + 1 + tag_bytes.len())
                .is_some_and(|s| s == tag_bytes)
        {
            let after = bytes.get(i + 1 + tag_bytes.len()).copied();
            if matches!(
                after,
                Some(b'>' | b' ' | b'\t' | b'\r' | b'\n' | b'/')
            ) && let Some(gt_rel) = xml[i + 1..].find('>')
            {
                let gt = i + 1 + gt_rel;
                let self_closing = xml[i + 1..gt].trim_end().ends_with('/');
                if !self_closing {
                    return Some((i, gt + 1));
                }
                i = gt + 1;
                continue;
            }
        }
        i += 1;
    }
    None
}

/// Finds `</tag>` while skipping over `<![CDATA[...]]>` and `<!-- ... -->`.
fn find_closing_tag(xml: &str, from: usize, close_needle: &str) -> Option<usize> {
    let bytes = xml.as_bytes();
    let needle = close_needle.as_bytes();
    let mut i = from;
    while i < bytes.len() {
        if bytes[i..].starts_with(b"<![CDATA[") {
            let end = xml[i + 9..].find("]]>")?;
            i += 9 + end + 3;
            continue;
        }
        if bytes[i..].starts_with(b"<!--") {
            let end = xml[i + 4..].find("-->")?;
            i += 4 + end + 3;
            continue;
        }
        if bytes[i..].starts_with(needle) {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn decode_text(raw: &str) -> String {
    let trimmed = raw.trim();
    if let Some(rest) = trimmed.strip_prefix("<![CDATA[")
        && let Some(inner) = rest.strip_suffix("]]>")
    {
        return inner.trim().to_string();
    }
    unescape_xml(trimmed).trim().to_string()
}

fn unescape_xml(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut rest = input;
    while let Some(amp) = rest.find('&') {
        out.push_str(&rest[..amp]);
        let after = &rest[amp + 1..];
        if let Some(semi) = after.find(';') {
            let entity = &after[..semi];
            if let Some(decoded) = decode_entity(entity) {
                out.push_str(&decoded);
                rest = &after[semi + 1..];
                continue;
            }
        }
        out.push('&');
        rest = after;
    }
    out.push_str(rest);
    out
}

fn decode_entity(entity: &str) -> Option<String> {
    match entity {
        "amp" => Some("&".into()),
        "lt" => Some("<".into()),
        "gt" => Some(">".into()),
        "quot" => Some("\"".into()),
        "apos" => Some("'".into()),
        "nbsp" => Some(" ".into()),
        _ if entity.starts_with("#x") || entity.starts_with("#X") => {
            u32::from_str_radix(&entity[2..], 16)
                .ok()
                .and_then(char::from_u32)
                .map(|c| c.to_string())
        }
        _ if entity.starts_with('#') => entity[1..]
            .parse::<u32>()
            .ok()
            .and_then(char::from_u32)
            .map(|c| c.to_string()),
        _ => None,
    }
}

fn clean_description(raw: &str) -> String {
    let mut plain = String::with_capacity(raw.len());
    let mut in_tag = false;
    for ch in raw.chars() {
        match ch {
            '<' => {
                in_tag = true;
                plain.push(' ');
            }
            '>' => in_tag = false,
            _ if !in_tag => plain.push(ch),
            _ => {}
        }
    }
    let normalized = plain.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.chars().count() <= MAX_DESCRIPTION_CHARS {
        normalized
    } else {
        normalized.chars().take(MAX_DESCRIPTION_CHARS).collect()
    }
}
