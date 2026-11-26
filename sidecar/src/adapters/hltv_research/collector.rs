//! Small, guarded collector for historical HLTV `/matches/<id>/<slug>` pages (ticket #36).
//!
//! Enforces the non-negotiable conditions of #36:
//! - On-disk cache (`hltv-match-<id>.html.gz`) so nothing is ever fetched twice.
//! - Minimum rate-limit interval between network calls (via injectable `Clock` + `Pause`).
//! - Hard stop rule: any HTTP 403, HTTP 429, `cf-mitigated` header, or Cloudflare challenge
//!   body saves the raw response for audit and permanently halts the collector (zero retries,
//!   zero escalation).

use super::{
    archive::HltvMatchRecord,
    parse::{ParseError, is_cloudflare_challenge, parse_match_page},
};
use crate::{Clock, Instant};
use async_trait::async_trait;
use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use std::{
    io::{Read, Write},
    path::PathBuf,
    sync::Arc,
    time::Duration,
};
use tracing::{info, warn};

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
}

#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: u16,
    pub cf_mitigated: Option<String>,
    pub body: String,
}

#[derive(Debug, thiserror::Error)]
pub enum CollectorError {
    #[error("collector stopped after HTTP {status}: {reason}")]
    Stopped { status: u16, reason: String },
    #[error("collector already stopped by a previous block or challenge; refusing further requests")]
    AlreadyStopped,
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
pub struct ReqwestHttp {
    client: reqwest::Client,
}

impl ReqwestHttp {
    pub fn new() -> Self {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let client = reqwest::Client::builder()
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
        let body = resp
            .text()
            .await
            .map_err(|e| CollectorError::Transport(e.to_string()))?;
        Ok(HttpResponse {
            status,
            cf_mitigated,
            body,
        })
    }
}

#[derive(Debug, Clone)]
pub struct CollectOutcome {
    pub record: HltvMatchRecord,
    pub from_cache: bool,
    pub raw_path: PathBuf,
    pub json_path: PathBuf,
}

pub struct HltvCollector {
    config: CollectorConfig,
    http: Arc<dyn ResearchHttp>,
    clock: Arc<dyn Clock>,
    pause: Arc<dyn Pause>,
    last_request: Option<Instant>,
    stopped: bool,
}

impl HltvCollector {
    pub fn new(
        config: CollectorConfig,
        http: Arc<dyn ResearchHttp>,
        clock: Arc<dyn Clock>,
        pause: Arc<dyn Pause>,
    ) -> Self {
        Self {
            config,
            http,
            clock,
            pause,
            last_request: None,
            stopped: false,
        }
    }

    pub fn is_stopped(&self) -> bool {
        self.stopped
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

    pub async fn collect_match(
        &mut self,
        match_id: i64,
        url: &str,
    ) -> Result<CollectOutcome, CollectorError> {
        if self.stopped {
            return Err(CollectorError::AlreadyStopped);
        }
        std::fs::create_dir_all(&self.config.out_dir)?;
        let raw_path = self.config.out_dir.join(format!("hltv-match-{match_id}.html.gz"));
        let json_path = self.config.out_dir.join(format!("hltv-match-{match_id}.json"));

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
                let rec = parse_match_page(match_id, url, self.clock.now(), &html)?;
                write_json(&json_path, &rec)?;
                rec
            };
            return Ok(CollectOutcome {
                record,
                from_cache: true,
                raw_path,
                json_path,
            });
        }

        // Enforce minimum interval between network requests.
        if let Some(prev) = self.last_request {
            let elapsed = (self.clock.now() - prev).to_std().unwrap_or(Duration::ZERO);
            if elapsed < self.config.min_interval {
                self.pause.sleep(self.config.min_interval - elapsed).await;
            }
        }

        let ua = self.user_agent()?;
        self.last_request = Some(self.clock.now());
        let resp = self.http.get(url, &ua).await?;

        // Hard stop rule: 403, 429, cf-mitigated, or Cloudflare challenge ends the attempt.
        if is_cloudflare_challenge(resp.status, resp.cf_mitigated.as_deref(), &resp.body) {
            self.stopped = true;
            let blocked_path = self
                .config
                .out_dir
                .join(format!("hltv-match-{match_id}-blocked.html"));
            std::fs::write(&blocked_path, resp.body.as_bytes())?;
            let reason = if let Some(cf) = &resp.cf_mitigated {
                format!("cf-mitigated={cf}")
            } else if resp.status == 429 {
                "rate limited (429)".to_string()
            } else if resp.status == 403 {
                "forbidden (403)".to_string()
            } else {
                "Cloudflare challenge body".to_string()
            };
            warn!(match_id, status = resp.status, %reason, "HLTV stop rule triggered");
            return Err(CollectorError::Stopped {
                status: resp.status,
                reason,
            });
        }

        if resp.status != 200 {
            return Err(CollectorError::HttpStatus(resp.status));
        }

        let fetched_at = self.clock.now();
        let record = parse_match_page(match_id, url, fetched_at, &resp.body)?;
        write_gzip(&raw_path, resp.body.as_bytes())?;
        write_json(&json_path, &record)?;
        append_provenance(&self.config.out_dir, match_id, url, fetched_at)?;
        info!(
            match_id,
            maps = record.maps.len(),
            veto = record.veto.len(),
            "HLTV match collected into archive"
        );

        Ok(CollectOutcome {
            record,
            from_cache: false,
            raw_path,
            json_path,
        })
    }
}

fn write_gzip(path: &PathBuf, data: &[u8]) -> std::io::Result<()> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)?;
    let compressed = encoder.finish()?;
    std::fs::write(path, compressed)
}

fn read_gzip_string(path: &PathBuf) -> std::io::Result<String> {
    let raw = std::fs::read(path)?;
    let mut out = String::new();
    GzDecoder::new(&raw[..]).read_to_string(&mut out)?;
    Ok(out)
}

fn write_json(path: &PathBuf, record: &HltvMatchRecord) -> std::io::Result<()> {
    let body = serde_json::to_string_pretty(record)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    std::fs::write(path, body + "\n")
}

fn append_provenance(
    out_dir: &std::path::Path,
    match_id: i64,
    url: &str,
    fetched_at: Instant,
) -> std::io::Result<()> {
    let path = out_dir.join("sources.tsv");
    let mut lines: Vec<String> = if path.exists() {
        std::fs::read_to_string(&path)?
            .lines()
            .map(str::to_owned)
            .filter(|l| !l.starts_with(&format!("hltv-match-{match_id}.html.gz\t")))
            .collect()
    } else {
        Vec::new()
    };
    lines.push(format!(
        "hltv-match-{match_id}.html.gz\t{url}\t{}",
        fetched_at.to_rfc3339()
    ));
    lines.sort();
    std::fs::write(path, lines.join("\n") + "\n")
}
