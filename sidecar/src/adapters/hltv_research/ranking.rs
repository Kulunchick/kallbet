//! Guarded, resumable collector for HLTV's weekly world team ranking pages
//! (`/ranking/teams/<year>/<month>/<day>`), a sibling of the `/matches/*` collector.
//!
//! Same conditions as the match collector: one connection, a minimum spacing between
//! requests, a raw on-disk cache written before anything else (so nothing is fetched
//! twice), its own provenance file, and a stop rule: the first HTTP 403, HTTP 429,
//! `cf-mitigated` header or Cloudflare challenge body saves the response, records the stop
//! in `stop.json` and halts the run; a later run refuses to start while `stop.json` exists.
//! No retries, no lower interval, no evasion.

use super::collector::{
    CollectorError, HttpResponse, Pause, ResearchHttp, ios_user_agent,
};
use super::parse::is_cloudflare_challenge;
use crate::{Clock, Instant};
use chrono::{Datelike, Duration as Days, NaiveDate, Weekday};
use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

/// Minimum spacing between ranking requests.
pub const RANKING_MIN_INTERVAL_SECS: u64 = 6;

const MONTHS: [&str; 12] = [
    "january", "february", "march", "april", "may", "june", "july", "august", "september",
    "october", "november", "december",
];

/// `<base>/ranking/teams/<year>/<month name>/<day>`, the form HLTV links to.
pub fn ranking_url(base: &str, day: NaiveDate) -> String {
    format!(
        "{}/ranking/teams/{}/{}/{}",
        base.trim_end_matches('/'),
        day.year(),
        MONTHS[day.month0() as usize],
        day.day()
    )
}

/// Every Monday (HLTV's publication day) from `from` through `until`.
pub fn mondays(from: NaiveDate, until: NaiveDate) -> Vec<NaiveDate> {
    let mut day = from;
    while day.weekday() != Weekday::Mon {
        day += Days::days(1);
    }
    let mut out = Vec::new();
    while day <= until {
        out.push(day);
        day += Days::days(7);
    }
    out
}

/// Only `/ranking/teams/` paths are permitted.
pub fn validate_ranking_url(url: &str) -> Result<(), CollectorError> {
    let after_scheme = url
        .split_once("://")
        .map(|(_, rest)| rest)
        .ok_or_else(|| CollectorError::DisallowedPath(format!("missing scheme in {url:?}")))?;
    let path = after_scheme.find('/').map_or("/", |i| &after_scheme[i..]);
    let path_only = path.split(['?', '#']).next().unwrap_or(path);
    if !path_only.starts_with("/ranking/teams/") || path_only.contains("/stats/") {
        return Err(CollectorError::DisallowedPath(format!(
            "only /ranking/teams/* paths are permitted; refused {url:?}"
        )));
    }
    Ok(())
}

fn redirect_target(origin_url: &str, location: &str) -> Result<String, CollectorError> {
    let (scheme, rest) = origin_url
        .split_once("://")
        .ok_or_else(|| CollectorError::DisallowedPath(format!("bad origin {origin_url:?}")))?;
    let host = rest.split('/').next().unwrap_or("");
    let resolved = if let Some((loc_scheme, loc_rest)) = location.split_once("://") {
        let loc_host = loc_rest.split('/').next().unwrap_or("");
        if !loc_scheme.eq_ignore_ascii_case(scheme) || !loc_host.eq_ignore_ascii_case(host) {
            return Err(CollectorError::DisallowedPath(format!(
                "refused cross-origin redirect to {location:?}"
            )));
        }
        location.to_string()
    } else if location.starts_with('/') {
        format!("{scheme}://{host}{location}")
    } else {
        return Err(CollectorError::DisallowedPath(format!(
            "refused relative redirect {location:?}"
        )));
    };
    validate_ranking_url(&resolved)?;
    Ok(resolved)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RankingStop {
    pub at: Instant,
    pub date: NaiveDate,
    pub url: String,
    pub status: u16,
    pub reason: String,
    pub saved_file: String,
}

#[derive(Debug, Clone, Default)]
pub struct RankingReport {
    pub targets_total: usize,
    pub cached_hits: usize,
    pub fetched_ok: usize,
    pub unavailable: usize,
    pub requests_made: u32,
    pub stopped: Option<RankingStop>,
    pub stopped_by_max_requests: bool,
}

pub struct RankingConfig {
    /// The `ranking/` directory.
    pub out_dir: PathBuf,
    pub min_interval: Duration,
    pub ios: (u8, u8, u8),
    pub max_requests: Option<u32>,
}

pub fn raw_path(out_dir: &Path, day: NaiveDate) -> PathBuf {
    out_dir.join(format!("hltv-ranking-{day}.html.gz"))
}

pub fn read_raw(path: &Path) -> std::io::Result<String> {
    let raw = std::fs::read(path)?;
    let mut out = String::new();
    GzDecoder::new(&raw[..]).read_to_string(&mut out)?;
    Ok(out)
}

fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, bytes)?;
    std::fs::rename(tmp, path)
}

fn gzip(data: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)?;
    encoder.finish()
}

fn append_tsv(path: &Path, header: &str, line: String) -> std::io::Result<()> {
    let mut lines: Vec<String> = if path.exists() {
        std::fs::read_to_string(path)?
            .lines()
            .filter(|l| !l.is_empty() && *l != header)
            .map(str::to_owned)
            .collect()
    } else {
        Vec::new()
    };
    let key = line.split('\t').next().unwrap_or("").to_owned() + "\t";
    lines.retain(|l| !l.starts_with(&key));
    lines.push(line);
    lines.sort();
    let mut out = format!("{header}\n");
    for l in lines {
        out.push_str(&l);
        out.push('\n');
    }
    write_atomic(path, out.as_bytes())
}

const SOURCES_HEADER: &str = "file\trequest\tfetched_at\tsha256";
const UNAVAILABLE_HEADER: &str = "file\trequest\tchecked_at\tstatus";

/// Fetches the ranking page of each date in order. Errors other than the stop rule are
/// returned; the stop rule is reported in [`RankingReport::stopped`].
pub async fn collect_rankings(
    config: &RankingConfig,
    base_url: &str,
    dates: &[NaiveDate],
    http: Arc<dyn ResearchHttp>,
    clock: Arc<dyn Clock>,
    pause: Arc<dyn Pause>,
) -> Result<RankingReport, CollectorError> {
    std::fs::create_dir_all(&config.out_dir)?;
    if config.out_dir.join("stop.json").exists() {
        return Err(CollectorError::AlreadyStopped);
    }
    let (major, minor, patch) = config.ios;
    let ua = ios_user_agent(major, minor, patch).map_err(CollectorError::InvalidConfig)?;
    let mut report = RankingReport {
        targets_total: dates.len(),
        ..Default::default()
    };
    let unavailable: std::collections::BTreeSet<String> =
        std::fs::read_to_string(config.out_dir.join("unavailable.tsv"))
            .unwrap_or_default()
            .lines()
            .filter_map(|l| l.split('\u{9}').next().map(str::to_owned))
            .collect();
    let mut last_request: Option<Instant> = None;
    'dates: for &day in dates {
        let path = raw_path(&config.out_dir, day);
        if path.is_file()
            || unavailable.contains(&path.file_name().unwrap().to_string_lossy().into_owned())
        {
            report.cached_hits += 1;
            continue;
        }
        let mut url = ranking_url(base_url, day);
        validate_ranking_url(&url)?;
        let mut hops = 0;
        loop {
            if config.max_requests.is_some_and(|m| report.requests_made >= m) {
                report.stopped_by_max_requests = true;
                break 'dates;
            }
            if let Some(prev) = last_request {
                let elapsed = (clock.now() - prev).to_std().unwrap_or(Duration::ZERO);
                if elapsed < config.min_interval {
                    pause.sleep(config.min_interval - elapsed).await;
                }
            }
            last_request = Some(clock.now());
            report.requests_made += 1;
            let resp: HttpResponse = http.get(&url, &ua).await?;
            if is_cloudflare_challenge(resp.status, resp.cf_mitigated.as_deref(), &resp.body) {
                let blocked = config.out_dir.join("blocked");
                std::fs::create_dir_all(&blocked)?;
                let saved = format!("blocked/hltv-ranking-{day}-blocked.html");
                std::fs::write(config.out_dir.join(&saved), resp.body.as_bytes())?;
                let reason = if let Some(cf) = &resp.cf_mitigated {
                    format!("cf-mitigated={cf}")
                } else if resp.status == 429 {
                    "rate limited (429)".to_string()
                } else if resp.status == 403 {
                    "forbidden (403)".to_string()
                } else {
                    "Cloudflare challenge body".to_string()
                };
                let stop = RankingStop {
                    at: clock.now(),
                    date: day,
                    url: url.clone(),
                    status: resp.status,
                    reason,
                    saved_file: saved,
                };
                let json = serde_json::to_string_pretty(&stop)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
                write_atomic(&config.out_dir.join("stop.json"), (json + "\n").as_bytes())?;
                report.stopped = Some(stop);
                break 'dates;
            }
            match resp.status {
                200 => {
                    write_atomic(&path, &gzip(resp.body.as_bytes())?)?;
                    let digest = hex::encode(Sha256::digest(resp.body.as_bytes()));
                    let file = path.file_name().unwrap().to_string_lossy().into_owned();
                    append_tsv(
                        &config.out_dir.join("sources.tsv"),
                        SOURCES_HEADER,
                        format!("{file}\t{url}\t{}\t{digest}", clock.now().to_rfc3339()),
                    )?;
                    report.fetched_ok += 1;
                    break;
                }
                301 | 302 | 303 | 307 | 308 if hops < 2 && resp.location.is_some() => {
                    url = redirect_target(&url, resp.location.as_deref().unwrap())?;
                    hops += 1;
                }
                404 => {
                    let file = path.file_name().unwrap().to_string_lossy().into_owned();
                    append_tsv(
                        &config.out_dir.join("unavailable.tsv"),
                        UNAVAILABLE_HEADER,
                        format!("{file}\t{url}\t{}\t404", clock.now().to_rfc3339()),
                    )?;
                    report.unavailable += 1;
                    break;
                }
                other => return Err(CollectorError::HttpStatus(other)),
            }
        }
    }
    Ok(report)
}

/// One team's line of a weekly ranking.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RankedTeam {
    pub rank: u32,
    pub team_id: i64,
    pub name: String,
    pub points: u32,
}

/// A parsed weekly ranking: the publication date (from the page's canonical link) and every ranked team.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RankingPage {
    pub date: NaiveDate,
    pub teams: Vec<RankedTeam>,
}

pub fn parse_ranking(html: &str) -> Result<RankingPage, String> {
    use scraper::{Html, Selector};
    if is_cloudflare_challenge(200, None, html) {
        return Err("Cloudflare challenge page".into());
    }
    let sel = |s: &str| Selector::parse(s).expect("valid static CSS selector");
    let doc = Html::parse_document(html);
    let canonical = doc
        .select(&sel("link[rel=canonical]"))
        .next()
        .and_then(|e| e.value().attr("href"))
        .ok_or("missing canonical link")?;
    let parts: Vec<&str> = canonical.split("/ranking/teams/").nth(1).ok_or("canonical is not a ranking page")?.split('/').collect();
    let month = parts
        .get(1)
        .and_then(|m| MONTHS.iter().position(|n| n == m))
        .ok_or("canonical has no month")?;
    let date = NaiveDate::from_ymd_opt(
        parts[0].parse().map_err(|_| "bad year")?,
        month as u32 + 1,
        parts.get(2).and_then(|d| d.parse().ok()).ok_or("bad day")?,
    )
    .ok_or("invalid canonical date")?;
    let (position, name, points, link) = (
        sel(".ranking-header .position"),
        sel(".teamLine .name"),
        sel(".teamLine .points"),
        sel(".more a.moreLink[href^=\"/team/\"]"),
    );
    let mut teams = Vec::new();
    for team in doc.select(&sel(".ranked-team")) {
        let text = |s: &Selector| team.select(s).next().map(|e| e.text().collect::<String>());
        let rank = text(&position)
            .and_then(|t| t.trim().trim_start_matches('#').parse().ok())
            .ok_or("missing rank")?;
        let team_id = team
            .select(&link)
            .next()
            .and_then(|a| a.value().attr("href"))
            .and_then(|h| h.trim_start_matches("/team/").split('/').next()?.parse().ok())
            .ok_or("missing team id")?;
        let name = text(&name).map(|t| t.trim().to_owned()).filter(|t| !t.is_empty()).ok_or("missing team name")?;
        let points = text(&points)
            .and_then(|t| t.trim_start_matches('(').split_whitespace().next()?.replace(',', "").parse().ok())
            .ok_or("missing points")?;
        teams.push(RankedTeam { rank, team_id, name, points });
    }
    if teams.is_empty() {
        return Err("no ranked teams".into());
    }
    Ok(RankingPage { date, teams })
}
