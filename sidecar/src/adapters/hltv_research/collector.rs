//! Guarded, resumable collector for historical HLTV `/matches/*` pages (ticket #36).
//!
//! Enforces the operator's non-negotiable conditions:
//! - `/matches/*` paths only (explicitly refuses `/stats/*` or any other path).
//! - On-disk raw cache (`matches/hltv-match-<id>.html.gz`) written before parsing
//!   so nothing is ever fetched twice.
//! - Minimum rate-limit interval between network calls (default 9 s; >= 8 s for remote HLTV).
//! - Daily UTC request budget (default 400), persisted in `manifest.json` and charged
//!   per actual HTTP request (including redirect hops).
//! - Hard stop rule: any HTTP 403, HTTP 429, `cf-mitigated` header, or Cloudflare challenge
//!   body saves the raw response for audit, records it in `manifest.json`, and permanently
//!   halts the collector (zero retries, zero evasion).

use super::{
    archive::{HltvArchiveManifest, HltvMatchRecord, ManifestMatchEntry, StopRecord},
    parse::{ParseError, is_cloudflare_challenge, parse_match_page},
};
use crate::{Clock, Instant};
use async_trait::async_trait;
use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use tracing::{info, warn};

/// Default daily UTC HTTP request cap authorized by the operator.
pub const DEFAULT_DAILY_BUDGET: u32 = 400;
/// Default spacing between HLTV requests (inside the operator's 8-10 s window).
pub const DEFAULT_MIN_INTERVAL_SECS: u64 = 9;
/// Minimum allowed spacing for non-loopback HLTV requests.
pub const REMOTE_MIN_INTERVAL_SECS: u64 = 8;

/// Builds the operator's iOS-app User-Agent string:
/// `hltv/2.13.2 (org.hltv; build:1; iOS <major>.<minor>.<patch>) Alamofire/4.9.1`
/// where `major` is in `14..=17` and `minor`/`patch` are each in `0..=9`.
pub fn ios_user_agent(major: u8, minor: u8, patch: u8) -> Result<String, String> {
    if !(14..=17).contains(&major) {
        return Err(format!("iOS major version {major} must be in 14..=17"));
    }
    if minor > 9 || patch > 9 {
        return Err(format!(
            "iOS minor ({minor}) and patch ({patch}) must be in 0..=9"
        ));
    }
    Ok(format!(
        "hltv/2.13.2 (org.hltv; build:1; iOS {major}.{minor}.{patch}) Alamofire/4.9.1"
    ))
}

/// Validates that a URL targets only `/matches/...` and never `/stats/...`.
pub fn validate_matches_url(url: &str) -> Result<(), CollectorError> {
    let after_scheme = url
        .split_once("://")
        .map(|(_, rest)| rest)
        .ok_or_else(|| CollectorError::DisallowedPath(format!("missing scheme in {url:?}")))?;
    let path = match after_scheme.find('/') {
        Some(idx) => &after_scheme[idx..],
        None => "/",
    };
    let path_only = path.split(['?', '#']).next().unwrap_or(path);
    if !path_only.starts_with("/matches/") || path_only.contains("/stats/") {
        return Err(CollectorError::DisallowedPath(format!(
            "only /matches/* paths are permitted; refused {url:?}"
        )));
    }
    Ok(())
}

/// Resolves a redirect `Location` header relative to `origin_url` and enforces
/// that it stays on the same origin and under `/matches/`.
pub fn resolve_same_origin_matches_redirect(
    origin_url: &str,
    location: &str,
) -> Result<String, CollectorError> {
    let (scheme, rest) = origin_url
        .split_once("://")
        .ok_or_else(|| CollectorError::DisallowedPath(format!("bad origin {origin_url:?}")))?;
    let host_port = rest.split('/').next().unwrap_or("");
    let resolved = if location.starts_with("http://") || location.starts_with("https://") {
        let (loc_scheme, loc_rest) = location.split_once("://").unwrap();
        let loc_host = loc_rest.split('/').next().unwrap_or("");
        if !loc_scheme.eq_ignore_ascii_case(scheme) || !loc_host.eq_ignore_ascii_case(host_port) {
            return Err(CollectorError::DisallowedPath(format!(
                "refused cross-origin redirect to {location:?}"
            )));
        }
        location.to_string()
    } else if location.starts_with('/') {
        format!("{scheme}://{host_port}{location}")
    } else {
        return Err(CollectorError::DisallowedPath(format!(
            "refused relative redirect {location:?}"
        )));
    };
    validate_matches_url(&resolved)?;
    Ok(resolved)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UaMode {
    /// Keep one iOS version for the entire collector run (matches a real device).
    FixedPerRun { major: u8, minor: u8, patch: u8 },
}

#[derive(Debug, Clone)]
pub struct CollectorConfig {
    pub out_dir: PathBuf,
    pub min_interval: Duration,
    pub ua_mode: UaMode,
    pub daily_budget: u32,
}

impl CollectorConfig {
    pub fn with_defaults(out_dir: PathBuf) -> Self {
        Self {
            out_dir,
            min_interval: Duration::from_secs(DEFAULT_MIN_INTERVAL_SECS),
            ua_mode: UaMode::FixedPerRun {
                major: 17,
                minor: 2,
                patch: 1,
            },
            daily_budget: DEFAULT_DAILY_BUDGET,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: u16,
    pub cf_mitigated: Option<String>,
    pub location: Option<String>,
    pub body: String,
}

#[derive(Debug, thiserror::Error)]
pub enum CollectorError {
    #[error("collector stopped after HTTP {status}: {reason}")]
    Stopped { status: u16, reason: String },
    #[error("collector already stopped by a previous block or challenge; refusing further requests")]
    AlreadyStopped,
    #[error("daily request budget reached ({spent}/{budget} for {day})")]
    BudgetExhausted {
        day: String,
        spent: u32,
        budget: u32,
    },
    #[error("disallowed URL path: {0}")]
    DisallowedPath(String),
    #[error("invalid user-agent configuration: {0}")]
    InvalidConfig(String),
    #[error("HTTP transport error: {0}")]
    Transport(String),
    #[error("unexpected HTTP status {0}")]
    HttpStatus(u16),
    #[error("parse error: {0}")]
    Parse(#[from] ParseError),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

#[async_trait]
pub trait ResearchHttp: Send + Sync {
    async fn get(&self, url: &str, user_agent: &str) -> Result<HttpResponse, CollectorError>;
}

#[async_trait]
pub trait Pause: Send + Sync {
    async fn sleep(&self, duration: Duration);
}

pub struct TokioPause;

#[async_trait]
impl Pause for TokioPause {
    async fn sleep(&self, duration: Duration) {
        tokio::time::sleep(duration).await;
    }
}

/// Plain `reqwest` transport used ONLY against local stand-in servers in tests.
/// Remote HLTV requests require the optional `hltv-impersonate` feature (`wreq`).
/// Disables automatic redirects so the collector counts every HTTP hop and enforces
/// `/matches/*` on redirect targets.
pub struct ReqwestHttp {
    client: reqwest::Client,
}

impl ReqwestHttp {
    pub fn new() -> Self {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_secs(30))
            .build()
            .expect("build reqwest client");
        Self { client }
    }
}

impl Default for ReqwestHttp {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ResearchHttp for ReqwestHttp {
    async fn get(&self, url: &str, user_agent: &str) -> Result<HttpResponse, CollectorError> {
        let resp = self
            .client
            .get(url)
            .header(reqwest::header::USER_AGENT, user_agent)
            .header(reqwest::header::ACCEPT, "*/*")
            .header(reqwest::header::ACCEPT_LANGUAGE, "en-US;q=1.0")
            .send()
            .await
            .map_err(|e| CollectorError::Transport(e.to_string()))?;
        let status = resp.status().as_u16();
        let cf_mitigated = resp
            .headers()
            .get("cf-mitigated")
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned);
        let location = resp
            .headers()
            .get(reqwest::header::LOCATION)
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned);
        let body = resp
            .text()
            .await
            .map_err(|e| CollectorError::Transport(e.to_string()))?;
        Ok(HttpResponse {
            status,
            cf_mitigated,
            location,
            body,
        })
    }
}

#[derive(Debug, Clone)]
pub struct CollectOutcome {
    pub record: HltvMatchRecord,
    pub from_cache: bool,
    pub requests_used: u32,
    pub final_url: String,
    pub raw_path: PathBuf,
    pub json_path: PathBuf,
}

#[derive(Debug, Clone, Default)]
pub struct BatchReport {
    pub targets_total: usize,
    pub cached_hits: usize,
    pub fetched_ok: usize,
    pub parse_errors: usize,
    pub requests_made: u32,
    pub spent_today: u32,
    pub daily_budget: u32,
    pub stopped_by_budget: bool,
    pub stopped_by_challenge: Option<StopRecord>,
}

pub struct HltvCollector {
    config: CollectorConfig,
    http: Arc<dyn ResearchHttp>,
    clock: Arc<dyn Clock>,
    pause: Arc<dyn Pause>,
    last_request: Option<Instant>,
    stopped: bool,
    manifest: HltvArchiveManifest,
}

impl HltvCollector {
    pub fn new(
        config: CollectorConfig,
        http: Arc<dyn ResearchHttp>,
        clock: Arc<dyn Clock>,
        pause: Arc<dyn Pause>,
    ) -> Self {
        let manifest = load_manifest(&config.out_dir).unwrap_or_default();
        Self {
            config,
            http,
            clock,
            pause,
            last_request: None,
            stopped: false,
            manifest,
        }
    }

    pub fn is_stopped(&self) -> bool {
        self.stopped
    }

    pub fn manifest(&self) -> &HltvArchiveManifest {
        &self.manifest
    }

    pub fn spent_today(&self) -> u32 {
        let day = self.utc_day();
        self.manifest.daily_requests.get(&day).copied().unwrap_or(0)
    }

    fn utc_day(&self) -> String {
        self.clock.now().format("%Y-%m-%d").to_string()
    }

    fn user_agent(&self) -> Result<String, CollectorError> {
        match self.config.ua_mode {
            UaMode::FixedPerRun {
                major,
                minor,
                patch,
            } => ios_user_agent(major, minor, patch).map_err(CollectorError::InvalidConfig),
        }
    }

    fn charge_one_request(&mut self) -> Result<(), CollectorError> {
        let day = self.utc_day();
        let spent = self.manifest.daily_requests.get(&day).copied().unwrap_or(0);
        if spent >= self.config.daily_budget {
            return Err(CollectorError::BudgetExhausted {
                day,
                spent,
                budget: self.config.daily_budget,
            });
        }
        self.manifest.daily_requests.insert(day, spent + 1);
        self.manifest.updated_at = Some(self.clock.now());
        save_manifest(&self.config.out_dir, &self.manifest)?;
        Ok(())
    }

    async fn wait_rate_limit(&mut self) {
        if let Some(prev) = self.last_request {
            let elapsed = (self.clock.now() - prev).to_std().unwrap_or(Duration::ZERO);
            if elapsed < self.config.min_interval {
                self.pause.sleep(self.config.min_interval - elapsed).await;
            }
        }
    }

    /// Sends one HTTP GET (enforcing per-request rate limit, daily budget, and stop rule).
    async fn send_checked(
        &mut self,
        match_id: i64,
        url: &str,
        ua: &str,
    ) -> Result<HttpResponse, CollectorError> {
        validate_matches_url(url)?;
        self.wait_rate_limit().await;
        self.charge_one_request()?;
        self.last_request = Some(self.clock.now());
        let resp = self.http.get(url, ua).await?;

        if is_cloudflare_challenge(resp.status, resp.cf_mitigated.as_deref(), &resp.body) {
            self.stopped = true;
            let blocked_dir = self.config.out_dir.join("blocked");
            std::fs::create_dir_all(&blocked_dir)?;
            let blocked_rel = format!("blocked/hltv-match-{match_id}-blocked.html");
            let blocked_path = self.config.out_dir.join(&blocked_rel);
            std::fs::write(&blocked_path, resp.body.as_bytes())?;
            // Also keep top-level copy path compatibility if caller checks it? No, keep clean in blocked/
            let reason = if let Some(cf) = &resp.cf_mitigated {
                format!("cf-mitigated={cf}")
            } else if resp.status == 429 {
                "rate limited (429)".to_string()
            } else if resp.status == 403 {
                "forbidden (403)".to_string()
            } else {
                "Cloudflare challenge body".to_string()
            };
            let stop = StopRecord {
                at: self.clock.now(),
                match_id,
                url: url.to_string(),
                status: resp.status,
                reason: reason.clone(),
                saved_file: blocked_rel,
            };
            self.manifest.last_stop = Some(stop);
            self.manifest.updated_at = Some(self.clock.now());
            save_manifest(&self.config.out_dir, &self.manifest)?;
            warn!(match_id, status = resp.status, %reason, "HLTV stop rule triggered");
            return Err(CollectorError::Stopped {
                status: resp.status,
                reason,
            });
        }
        Ok(resp)
    }

    pub async fn collect_match(
        &mut self,
        match_id: i64,
        url: &str,
    ) -> Result<CollectOutcome, CollectorError> {
        if self.stopped {
            return Err(CollectorError::AlreadyStopped);
        }
        validate_matches_url(url)?;

        let matches_dir = self.config.out_dir.join("matches");
        std::fs::create_dir_all(&matches_dir)?;
        let raw_rel = format!("matches/hltv-match-{match_id}.html.gz");
        let json_rel = format!("matches/hltv-match-{match_id}.json");
        let raw_path = self.config.out_dir.join(&raw_rel);
        let json_path = self.config.out_dir.join(&json_rel);

        // Cache hit: never fetch the same Match twice.
        if raw_path.exists() {
            let record = if json_path.exists() {
                let bytes = std::fs::read(&json_path)?;
                serde_json::from_slice(&bytes).map_err(|e| {
                    CollectorError::Parse(ParseError::InvalidMarkup(format!(
                        "cached archive JSON unreadable: {e}"
                    )))
                })?
            } else {
                let html = read_gzip_string(&raw_path)?;
                let rec = match parse_match_page(match_id, url, self.clock.now(), &html) {
                    Ok(rec) => rec,
                    Err(e) => {
                        self.manifest
                            .parse_errors
                            .insert(match_id, e.to_string());
                        self.manifest.updated_at = Some(self.clock.now());
                        save_manifest(&self.config.out_dir, &self.manifest)?;
                        return Err(CollectorError::Parse(e));
                    }
                };
                write_json(&json_path, &rec)?;
                self.record_manifest_match(&rec, &raw_rel, &json_rel)?;
                rec
            };
            if !self.manifest.matches.contains_key(&match_id) {
                self.record_manifest_match(&record, &raw_rel, &json_rel)?;
            }
            return Ok(CollectOutcome {
                final_url: record.url.clone(),
                record,
                from_cache: true,
                requests_used: 0,
                raw_path,
                json_path,
            });
        }

        let ua = self.user_agent()?;
        self.manifest.last_ios_user_agent = Some(ua.clone());

        let mut current_url = url.to_string();
        let mut requests_used = 0u32;
        let mut resp = self.send_checked(match_id, &current_url, &ua).await?;
        requests_used += 1;

        // Follow at most 2 same-origin `/matches/*` redirects (e.g. /matches/<id>/match -> canonical slug).
        for _ in 0..2 {
            if matches!(resp.status, 301 | 302 | 303 | 307 | 308) {
                let Some(loc) = resp.location.as_deref() else {
                    return Err(CollectorError::HttpStatus(resp.status));
                };
                current_url = resolve_same_origin_matches_redirect(&current_url, loc)?;
                resp = self.send_checked(match_id, &current_url, &ua).await?;
                requests_used += 1;
            } else {
                break;
            }
        }

        if resp.status != 200 {
            return Err(CollectorError::HttpStatus(resp.status));
        }

        // Save raw HTML + provenance BEFORE parsing so an edge-case page is never fetched twice.
        let fetched_at = self.clock.now();
        write_gzip(&raw_path, resp.body.as_bytes())?;
        let sha256 = sha256_hex(resp.body.as_bytes());
        append_provenance(&self.config.out_dir, &raw_rel, &current_url, fetched_at, &sha256)?;

        let record = match parse_match_page(match_id, &current_url, fetched_at, &resp.body) {
            Ok(rec) => rec,
            Err(e) => {
                self.manifest
                    .parse_errors
                    .insert(match_id, e.to_string());
                self.manifest.updated_at = Some(fetched_at);
                save_manifest(&self.config.out_dir, &self.manifest)?;
                return Err(CollectorError::Parse(e));
            }
        };

        write_json(&json_path, &record)?;
        self.record_manifest_match(&record, &raw_rel, &json_rel)?;
        info!(
            match_id,
            maps = record.maps.len(),
            veto = record.veto.len(),
            requests_used,
            "HLTV match collected into archive"
        );

        Ok(CollectOutcome {
            final_url: current_url,
            record,
            from_cache: false,
            requests_used,
            raw_path,
            json_path,
        })
    }

    /// Collects a list of `(match_id, url)` targets in order, skipping cached pages,
    /// recording per-page parse quirks without refetching, and stopping immediately
    /// on budget exhaustion or any 403/429/Cloudflare challenge.
    pub async fn collect_batch(
        &mut self,
        targets: &[(i64, String)],
        max_requests_this_run: Option<u32>,
    ) -> Result<BatchReport, CollectorError> {
        let mut report = BatchReport {
            targets_total: targets.len(),
            daily_budget: self.config.daily_budget,
            spent_today: self.spent_today(),
            ..Default::default()
        };

        for (match_id, url) in targets {
            let raw_path = self
                .config
                .out_dir
                .join(format!("matches/hltv-match-{match_id}.html.gz"));
            let is_cached = raw_path.exists();

            if !is_cached {
                if let Some(limit) = max_requests_this_run
                    && report.requests_made >= limit
                {
                    report.stopped_by_budget = true;
                    break;
                }
                if self.spent_today() >= self.config.daily_budget {
                    report.stopped_by_budget = true;
                    break;
                }
            }

            match self.collect_match(*match_id, url).await {
                Ok(outcome) => {
                    report.requests_made += outcome.requests_used;
                    if outcome.from_cache {
                        report.cached_hits += 1;
                    } else {
                        report.fetched_ok += 1;
                    }
                }
                Err(CollectorError::Parse(e)) => {
                    // Raw HTML was cached before parse; record and continue with next match.
                    warn!(match_id = *match_id, error = %e, "cached raw HLTV page failed parse; continuing");
                    if !is_cached {
                        report.requests_made += 1;
                    }
                    report.parse_errors += 1;
                }
                Err(CollectorError::BudgetExhausted { .. }) => {
                    report.stopped_by_budget = true;
                    break;
                }
                Err(CollectorError::Stopped { .. }) | Err(CollectorError::AlreadyStopped) => {
                    report.stopped_by_challenge = self.manifest.last_stop.clone();
                    break;
                }
                Err(other) => return Err(other),
            }
        }

        report.spent_today = self.spent_today();
        Ok(report)
    }

    fn record_manifest_match(
        &mut self,
        rec: &HltvMatchRecord,
        raw_rel: &str,
        json_rel: &str,
    ) -> std::io::Result<()> {
        let players: usize = rec.overall_stats.iter().map(|t| t.players.len()).sum();
        self.manifest.parse_errors.remove(&rec.match_id);
        self.manifest.matches.insert(
            rec.match_id,
            ManifestMatchEntry {
                match_id: rec.match_id,
                json_file: json_rel.to_string(),
                raw_file: raw_rel.to_string(),
                url: rec.url.clone(),
                event_id: rec.event_id,
                event_name: rec.event_name.clone(),
                started_at: rec.started_at,
                format: rec.format.clone(),
                team_a: rec.team_a.name.clone(),
                team_b: rec.team_b.name.clone(),
                maps: rec.maps.len(),
                veto_steps: rec.veto.len(),
                players,
            },
        );
        self.manifest.updated_at = Some(self.clock.now());
        save_manifest(&self.config.out_dir, &self.manifest)
    }
}

/// Discovers unique HLTV match IDs from `<out_dir>/targets.tsv` (preserving its
/// priority order when present) plus Liquipedia revisions in `archive_dir`
/// (`lp-*.json(.gz)`, `roster-*.json(.gz)`, `<out_dir>/liquipedia/*.json(.gz)`).
pub fn discover_archive_targets(
    archive_dir: &Path,
    out_dir: &Path,
    base_url: &str,
) -> Result<Vec<(i64, String)>, CollectorError> {
    let base = base_url.trim_end_matches('/');
    let mut ordered: Vec<(i64, String)> = Vec::new();
    let mut seen: BTreeSet<i64> = BTreeSet::new();

    // 1. Explicit targets.tsv (match_id [\t url ...]) — preserves priority order.
    let targets_tsv = out_dir.join("targets.tsv");
    if targets_tsv.is_file() {
        for line in std::fs::read_to_string(&targets_tsv)?.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with("match_id\t") {
                continue;
            }
            let mut cols = line.split('\t');
            if let Some(id) = cols.next().and_then(|s| s.parse::<i64>().ok())
                && id > 0
                && seen.insert(id)
            {
                let url = cols
                    .next()
                    .filter(|s| !s.is_empty())
                    .map(str::to_string)
                    .unwrap_or_else(|| format!("{base}/matches/{id}/match"));
                ordered.push((id, url));
            }
        }
        if !ordered.is_empty() {
            return Ok(ordered);
        }
    }

    // 2. Fallback: scan Liquipedia JSON / JSON.GZ files in archive root and out_dir/liquipedia.
    let mut ids: BTreeSet<i64> = BTreeSet::new();
    let mut scan_dirs = vec![archive_dir.to_path_buf(), out_dir.join("liquipedia")];
    let re = regex::Regex::new(r"\|hltv=(\d+)").expect("valid hltv regex");
    for dir in scan_dirs.drain(..) {
        if !dir.is_dir() {
            continue;
        }
        let mut entries: Vec<PathBuf> = std::fs::read_dir(&dir)?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.is_file())
            .collect();
        entries.sort();
        for path in entries {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let is_lp_dir = dir.file_name().and_then(|n| n.to_str()) == Some("liquipedia");
            let is_lp_file = is_lp_dir
                || name.starts_with("lp-")
                || name.starts_with("roster-");
            if !is_lp_file
                || !(name.ends_with(".json") || name.ends_with(".json.gz"))
            {
                continue;
            }
            let text = if name.ends_with(".gz") {
                match read_gzip_string(&path) {
                    Ok(t) => t,
                    Err(_) => continue,
                }
            } else {
                match std::fs::read_to_string(&path) {
                    Ok(t) => t,
                    Err(_) => continue,
                }
            };
            for cap in re.captures_iter(&text) {
                if let Ok(id) = cap[1].parse::<i64>()
                    && id > 0
                {
                    ids.insert(id);
                }
            }
        }
    }

    Ok(ids
        .into_iter()
        .map(|id| (id, format!("{base}/matches/{id}/match")))
        .collect())
}

fn load_manifest(out_dir: &Path) -> Option<HltvArchiveManifest> {
    let path = out_dir.join("manifest.json");
    let bytes = std::fs::read(path).ok()?;
    serde_json::from_slice(&bytes).ok()
}

fn save_manifest(out_dir: &Path, manifest: &HltvArchiveManifest) -> std::io::Result<()> {
    std::fs::create_dir_all(out_dir)?;
    let path = out_dir.join("manifest.json");
    let tmp = out_dir.join("manifest.json.tmp");
    let body = serde_json::to_string_pretty(manifest)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    std::fs::write(&tmp, body + "\n")?;
    std::fs::rename(tmp, path)
}

fn write_gzip(path: &PathBuf, data: &[u8]) -> std::io::Result<()> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)?;
    let compressed = encoder.finish()?;
    let tmp = path.with_extension("gz.tmp");
    std::fs::write(&tmp, compressed)?;
    std::fs::rename(tmp, path)
}

fn read_gzip_string(path: &Path) -> std::io::Result<String> {
    let raw = std::fs::read(path)?;
    let mut out = String::new();
    GzDecoder::new(&raw[..]).read_to_string(&mut out)?;
    Ok(out)
}

fn write_json(path: &PathBuf, record: &HltvMatchRecord) -> std::io::Result<()> {
    let body = serde_json::to_string_pretty(record)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, body + "\n")?;
    std::fs::rename(tmp, path)
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

fn append_provenance(
    out_dir: &Path,
    rel_file: &str,
    url: &str,
    fetched_at: Instant,
    sha256: &str,
) -> std::io::Result<()> {
    let path = out_dir.join("sources.tsv");
    let header = "file\trequest\tfetched_at\tsha256";
    let prefix = format!("{rel_file}\t");
    let mut lines: Vec<String> = if path.exists() {
        std::fs::read_to_string(&path)?
            .lines()
            .filter(|l| !l.is_empty() && *l != header && !l.starts_with(&prefix))
            .map(str::to_owned)
            .collect()
    } else {
        Vec::new()
    };
    lines.push(format!(
        "{rel_file}\t{url}\t{}\t{sha256}",
        fetched_at.to_rfc3339()
    ));
    lines.sort();
    let mut out = String::with_capacity(lines.len() * 128);
    out.push_str(header);
    out.push('\n');
    for line in lines {
        out.push_str(&line);
        out.push('\n');
    }
    let tmp = out_dir.join("sources.tsv.tmp");
    std::fs::write(&tmp, out)?;
    std::fs::rename(tmp, path)
}
