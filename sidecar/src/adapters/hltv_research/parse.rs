//! Pure HTML parser for HLTV `/matches/<id>/<slug>` pages (ticket #36).
//! No network I/O. Returns an explicit error on Cloudflare challenges or changed markup.

use super::archive::*;
use chrono::{DateTime, Utc};
use scraper::{ElementRef, Html, Selector};

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Cloudflare challenge or bot mitigation page")]
    CloudflareChallenge,
    #[error("unrecognised HLTV match markup: {0}")]
    InvalidMarkup(String),
}

/// Detects Cloudflare challenge / block pages from HTTP status, response headers, or HTML body.
pub fn is_cloudflare_challenge(status: u16, cf_mitigated: Option<&str>, body: &str) -> bool {
    if status == 403 || status == 429 {
        return true;
    }
    if cf_mitigated.is_some_and(|v| !v.trim().is_empty()) {
        return true;
    }
    let head = &body[..body.len().min(8192)];
    head.contains("<title>Just a moment...</title>")
        || head.contains("_cf_chl_opt")
        || head.contains("cf-browser-verification")
        || head.contains("Attention Required! | Cloudflare")
        || head.contains("challenges.cloudflare.com")
}

fn sel(s: &str) -> Selector {
    Selector::parse(s).expect("valid static CSS selector")
}

fn text_trimmed(el: ElementRef<'_>) -> String {
    el.text()
        .collect::<Vec<_>>()
        .join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn id_after_prefix(href: &str, prefix: &str) -> Option<i64> {
    let rest = href.split(prefix).nth(1)?;
    let digits = rest.split('/').next()?;
    digits.parse::<i64>().ok()
}

fn parse_team_box(doc: &Html, selector: &str) -> Result<TeamRef, ParseError> {
    let link = doc
        .select(&sel(selector))
        .next()
        .ok_or_else(|| ParseError::InvalidMarkup(format!("missing team link ({selector})")))?;
    let href = link
        .value()
        .attr("href")
        .ok_or_else(|| ParseError::InvalidMarkup("team link without href".into()))?;
    let id = id_after_prefix(href, "/team/")
        .ok_or_else(|| ParseError::InvalidMarkup(format!("bad team href {href:?}")))?;
    let name = link
        .select(&sel(".teamName"))
        .next()
        .map(text_trimmed)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| ParseError::InvalidMarkup("missing .teamName".into()))?;
    Ok(TeamRef { id, name })
}

fn parse_format_and_lan(doc: &Html) -> (String, Option<bool>) {
    for pre in doc.select(&sel(".veto-box .preformatted-text")) {
        let t = text_trimmed(pre);
        let lower = t.to_ascii_lowercase();
        let format = if lower.contains("best of 5") {
            "bo5"
        } else if lower.contains("best of 3") {
            "bo3"
        } else if lower.contains("best of 1") {
            "bo1"
        } else {
            continue;
        };
        let is_lan = if lower.contains("(lan)") {
            Some(true)
        } else if lower.contains("(online)") {
            Some(false)
        } else {
            None
        };
        return (format.to_string(), is_lan);
    }
    ("unknown".to_string(), None)
}

fn parse_veto_line(line: &str) -> Option<VetoStep> {
    let line = line.trim();
    let (num_str, rest) = line.split_once(". ")?;
    let step: u8 = num_str.trim().parse().ok()?;
    let rest = rest.trim();
    if let Some(map_name) = rest.strip_suffix(" was left over") {
        return Some(VetoStep {
            step,
            team: None,
            action: VetoAction::LeftOver,
            map_name: map_name.trim().to_string(),
        });
    }
    if let Some((team, map_name)) = rest.rsplit_once(" removed ") {
        return Some(VetoStep {
            step,
            team: Some(team.trim().to_string()),
            action: VetoAction::Removed,
            map_name: map_name.trim().to_string(),
        });
    }
    if let Some((team, map_name)) = rest.rsplit_once(" picked ") {
        return Some(VetoStep {
            step,
            team: Some(team.trim().to_string()),
            action: VetoAction::Picked,
            map_name: map_name.trim().to_string(),
        });
    }
    None
}

fn parse_vetos(doc: &Html) -> Vec<VetoStep> {
    let mut steps = Vec::new();
    for vbox in doc.select(&sel(".veto-box")) {
        if vbox.select(&sel(".preformatted-text")).next().is_some() {
            continue;
        }
        for line_el in vbox.select(&sel(".padding > div")) {
            let text = text_trimmed(line_el);
            if let Some(step) = parse_veto_line(&text) {
                steps.push(step);
            }
        }
        if !steps.is_empty() {
            break;
        }
    }
    steps
}

fn parse_maps(doc: &Html) -> Vec<MapResult> {
    let mut maps = Vec::new();
    for (idx, holder) in doc.select(&sel(".mapholder")).enumerate() {
        let Some(name_el) = holder.select(&sel(".mapname")).next() else {
            continue;
        };
        let map_name = text_trimmed(name_el);
        if map_name.is_empty() || map_name.eq_ignore_ascii_case("TBA") {
            continue;
        }
        let played = holder.select(&sel(".played")).next().is_some();
        let left_score = holder
            .select(&sel(".results-left .results-team-score"))
            .next()
            .and_then(|e| text_trimmed(e).parse::<u32>().ok());
        let right_score = holder
            .select(&sel(".results-right .results-team-score"))
            .next()
            .and_then(|e| text_trimmed(e).parse::<u32>().ok());
        let winner = match (left_score, right_score) {
            (Some(a), Some(b)) if a > b => Some(Side::A),
            (Some(a), Some(b)) if b > a => Some(Side::B),
            _ => None,
        };
        let half_scores = holder
            .select(&sel(".results-center-half-score"))
            .next()
            .map(|e| {
                e.text()
                    .collect::<String>()
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .filter(|s| !s.is_empty());
        let mapstats_id = holder
            .select(&sel("a.results-stats"))
            .next()
            .and_then(|a| a.value().attr("href"))
            .and_then(|href| id_after_prefix(href, "mapstatsid/"));
        maps.push(MapResult {
            position: (idx + 1) as u8,
            map_name,
            mapstats_id,
            team_a_score: left_score,
            team_b_score: right_score,
            winner,
            half_scores,
            played,
        });
    }
    maps
}

fn parse_kd(raw: &str) -> Option<(u32, u32)> {
    let (k, d) = raw.trim().split_once('-')?;
    Some((k.trim().parse().ok()?, d.trim().parse().ok()?))
}

fn parse_pct(raw: &str) -> Option<f64> {
    raw.trim()
        .trim_end_matches('%')
        .trim_start_matches('+')
        .parse::<f64>()
        .ok()
}

fn parse_stats_table(table: ElementRef<'_>) -> Result<TeamPlayerStats, ParseError> {
    let header_row = table
        .select(&sel("tr.header-row"))
        .next()
        .ok_or_else(|| ParseError::InvalidMarkup("totalstats table missing header-row".into()))?;
    let team_link = header_row.select(&sel("a.teamName")).next();
    let team_id = team_link
        .and_then(|a| a.value().attr("href"))
        .and_then(|h| id_after_prefix(h, "/team/"));
    let team_name = team_link
        .map(text_trimmed)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "Unknown".to_string());
    let rating_version = header_row
        .select(&sel(".ratingDesc"))
        .next()
        .map(text_trimmed)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "2.1".to_string());

    let mut players = Vec::new();
    for row in table.select(&sel("tr")) {
        if row.value().classes().any(|c| c == "header-row") {
            continue;
        }
        let Some(player_link) = row.select(&sel("td.players a[href^=\"/player/\"]")).next() else {
            continue;
        };
        let href = player_link.value().attr("href").unwrap_or("");
        let Some(player_id) = id_after_prefix(href, "/player/") else {
            continue;
        };
        let nickname = player_link
            .select(&sel(".player-nick"))
            .next()
            .map(text_trimmed)
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| text_trimmed(player_link));

        let kd_text = row
            .select(&sel("td.kd.traditional-data"))
            .next()
            .or_else(|| row.select(&sel("td.kd")).next())
            .map(text_trimmed)
            .unwrap_or_default();
        let (kills, deaths) = parse_kd(&kd_text).ok_or_else(|| {
            ParseError::InvalidMarkup(format!("invalid K-D {kd_text:?} for {nickname}"))
        })?;
        let ekd = row
            .select(&sel("td.kd.eco-adjusted-data"))
            .next()
            .and_then(|e| parse_kd(&text_trimmed(e)));
        let swing_pct = row
            .select(&sel("td.roundSwing"))
            .next()
            .and_then(|e| parse_pct(&text_trimmed(e)));
        let adr = row
            .select(&sel("td.adr.traditional-data"))
            .next()
            .or_else(|| row.select(&sel("td.adr")).next())
            .and_then(|e| text_trimmed(e).parse::<f64>().ok());
        let eadr = row
            .select(&sel("td.adr.eco-adjusted-data"))
            .next()
            .and_then(|e| text_trimmed(e).parse::<f64>().ok());
        let kast_pct = row
            .select(&sel("td.kast.traditional-data"))
            .next()
            .or_else(|| row.select(&sel("td.kast")).next())
            .and_then(|e| parse_pct(&text_trimmed(e)));
        let ekast_pct = row
            .select(&sel("td.kast.eco-adjusted-data"))
            .next()
            .and_then(|e| parse_pct(&text_trimmed(e)));
        let rating = row
            .select(&sel("td.rating"))
            .next()
            .and_then(|e| text_trimmed(e).parse::<f64>().ok())
            .ok_or_else(|| ParseError::InvalidMarkup(format!("missing rating for {nickname}")))?;

        players.push(PlayerMatchStats {
            player_id,
            nickname,
            kills,
            deaths,
            ek_kills: ekd.map(|(k, _)| k),
            ek_deaths: ekd.map(|(_, d)| d),
            swing_pct,
            adr,
            eadr,
            kast_pct,
            ekast_pct,
            rating,
            rating_version: rating_version.clone(),
        });
    }

    if players.is_empty() {
        return Err(ParseError::InvalidMarkup(format!(
            "totalstats table for {team_name} had zero players"
        )));
    }

    Ok(TeamPlayerStats {
        team_id,
        team_name,
        players,
    })
}

/// Parses an HLTV `/matches/<id>/<slug>` HTML document into a structured archive record.
pub fn parse_match_page(
    match_id: i64,
    url: &str,
    fetched_at: DateTime<Utc>,
    html: &str,
) -> Result<HltvMatchRecord, ParseError> {
    if is_cloudflare_challenge(200, None, html) {
        return Err(ParseError::CloudflareChallenge);
    }
    let doc = Html::parse_document(html);

    let team_a = parse_team_box(&doc, ".teamsBox .team1-gradient a[href^=\"/team/\"]")?;
    let team_b = parse_team_box(&doc, ".teamsBox .team2-gradient a[href^=\"/team/\"]")?;

    let unix_ms: i64 = doc
        .select(&sel(".timeAndEvent .time[data-unix]"))
        .next()
        .and_then(|e| e.value().attr("data-unix"))
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| ParseError::InvalidMarkup("missing .timeAndEvent .time[data-unix]".into()))?;
    let started_at = DateTime::from_timestamp_millis(unix_ms).ok_or_else(|| {
        ParseError::InvalidMarkup(format!("invalid unix ms timestamp {unix_ms}"))
    })?;

    let event_link = doc.select(&sel(".timeAndEvent .event a[href^=\"/events/\"]")).next();
    let event_id = event_link
        .and_then(|a| a.value().attr("href"))
        .and_then(|h| id_after_prefix(h, "/events/"));
    let event_name = event_link.map(text_trimmed).filter(|s| !s.is_empty());

    let (format, is_lan) = parse_format_and_lan(&doc);
    let maps = parse_maps(&doc);
    if maps.is_empty() {
        return Err(ParseError::InvalidMarkup("no maps found in .mapholder".into()));
    }
    let veto = parse_vetos(&doc);

    let mut overall_stats = Vec::new();
    let mut per_map_stats = Vec::new();

    for content in doc.select(&sel("div.stats-content[id$=\"-content\"]")) {
        let Some(id_attr) = content.value().attr("id") else {
            continue;
        };
        let mut teams = Vec::new();
        for table in content.select(&sel("table.totalstats")) {
            teams.push(parse_stats_table(table)?);
        }
        if teams.is_empty() {
            continue;
        }
        if id_attr == "all-content" {
            overall_stats = teams;
        } else if let Some(ms_str) = id_attr.strip_suffix("-content")
            && let Ok(mapstats_id) = ms_str.parse::<i64>()
        {
            let map_name = maps
                .iter()
                .find(|m| m.mapstats_id == Some(mapstats_id))
                .map(|m| m.map_name.clone());
            per_map_stats.push(MapPlayerStats {
                mapstats_id,
                map_name,
                teams,
            });
        }
    }

    if overall_stats.is_empty() {
        // A page that has no `.matchstats` widget at all (showmatches, some lower-tier events) is a
        // finished Match without player statistics: keep its teams, maps, scores and veto with
        // empty stats. A stats widget whose tables did not parse is changed markup and stays an error.
        let no_stats_widget = doc.select(&sel(".matchstats")).next().is_none();
        let has_scored_map = maps
            .iter()
            .any(|m| m.team_a_score.is_some() && m.team_b_score.is_some());
        if !(no_stats_widget && has_scored_map) {
            return Err(ParseError::InvalidMarkup(
                "missing overall player stats (#all-content)".into(),
            ));
        }
    }

    Ok(HltvMatchRecord {
        match_id,
        url: url.to_string(),
        fetched_at,
        event_id,
        event_name,
        started_at,
        format,
        is_lan,
        team_a,
        team_b,
        maps,
        veto,
        overall_stats,
        per_map_stats,
    })
}
