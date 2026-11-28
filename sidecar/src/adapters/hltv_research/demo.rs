//! Demo pilot downloader (operator-approved): `/download/demo/<id>` of HLTV match pages, one file at a time.
//!
//! Same conditions as the page collectors: ONE connection, at least [`DEMO_MIN_INTERVAL_SECS`] between
//! requests (every redirect hop counts), a provenance row per file (`demos.tsv`), and the stop rule:
//! the first HTTP 403, HTTP 429, `cf-mitigated` header or Cloudflare challenge body halts the run and is
//! recorded in `stop.json`; a later run refuses to start while `stop.json` exists. No retries, no lower
//! interval, no evasion. A redirect may only go to the same origin or to a host under `hltv.org`;
//! anything else is recorded as a stop (reported, not followed).
//! Files are streamed to disk (never held in memory) and never listed as research sources: the caller
//! parses them and deletes them.

use super::collector::{CollectorError, Pause, ios_user_agent};
use super::parse::is_cloudflare_challenge;
use crate::{Clock, Instant};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{
    io::Write,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

/// Minimum spacing between demo requests.
pub const DEMO_MIN_INTERVAL_SECS: u64 = 6;

/// What one GET produced. A 2xx body that is not HTML was streamed to `dest`; anything else is returned
/// in `body` (capped) for the challenge check.
#[derive(Debug, Clone)]
pub struct DownloadResponse {
    pub status: u16,
    pub cf_mitigated: Option<String>,
    pub location: Option<String>,
    pub content_type: Option<String>,
    pub content_disposition: Option<String>,
    pub bytes_written: u64,
    pub body: String,
}

#[async_trait]
pub trait ResearchDownload: Send + Sync {
    /// GET `url`; stream a non-HTML 200 body into `dest`.
    async fn download(&self, url: &str, user_agent: &str, dest: &Path) -> Result<DownloadResponse, CollectorError>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DemoStop {
    pub at: Instant,
    pub demo_id: i64,
    pub url: String,
    pub status: u16,
    pub reason: String,
}

#[derive(Debug, Clone, Default)]
pub struct DemoReport {
    pub targets_total: usize,
    pub downloaded: usize,
    pub cached_hits: usize,
    pub requests_made: u32,
    pub bytes: u64,
    pub stopped: Option<DemoStop>,
    pub stopped_by_max_requests: bool,
}

pub struct DemoConfig {
    /// Directory that receives `demos/<id>.bin`, `demos.tsv` and `stop.json`.
    pub out_dir: PathBuf,
    pub min_interval: Duration,
    pub ios: (u8, u8, u8),
    pub max_requests: Option<u32>,
}

/// Only `/download/demo/<digits>` on the given base may be requested.
pub fn demo_url(base: &str, id: i64) -> String {
    format!("{}/download/demo/{id}", base.trim_end_matches('/'))
}

fn host_of(url: &str) -> Option<&str> {
    let rest = url.split_once("://")?.1;
    Some(rest.split(['/', ':', '?']).next().unwrap_or(""))
}

/// Resolves a redirect target: relative paths stay on the origin; absolute URLs must be https and on
/// the origin host or under `hltv.org`.
pub fn redirect_target(origin: &str, location: &str) -> Result<String, CollectorError> {
    let origin_host = host_of(origin).unwrap_or("");
    if location.starts_with('/') {
        let scheme = origin.split_once("://").map_or("https", |(s, _)| s);
        return Ok(format!("{scheme}://{origin_host}{location}"));
    }
    let Some((scheme, _)) = location.split_once("://") else {
        return Err(CollectorError::DisallowedPath(format!("refused relative redirect {location:?}")));
    };
    let host = host_of(location).unwrap_or("").to_ascii_lowercase();
    let ok = scheme.eq_ignore_ascii_case("https")
        && (host == origin_host.to_ascii_lowercase() || host == "hltv.org" || host.ends_with(".hltv.org"));
    if ok {
        Ok(location.to_string())
    } else {
        Err(CollectorError::DisallowedPath(format!("refused redirect outside hltv.org: {location:?}")))
    }
}

fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, bytes)?;
    std::fs::rename(tmp, path)
}

fn append_line(path: &Path, header: &str, line: &str) -> std::io::Result<()> {
    let new = !path.exists();
    let mut f = std::fs::OpenOptions::new().create(true).append(true).open(path)?;
    if new {
        writeln!(f, "{header}")?;
    }
    writeln!(f, "{line}")
}

const TSV_HEADER: &str = "demo_id\trequest\tfinal_url\tfetched_at\tseconds\tbytes\tstatus\tcontent_type\tcontent_disposition";

/// Downloads each demo id in order. Cached files (`demos/<id>.bin`) cost nothing.
pub async fn collect_demos(
    config: &DemoConfig,
    base_url: &str,
    ids: &[i64],
    http: Arc<dyn ResearchDownload>,
    clock: Arc<dyn Clock>,
    pause: Arc<dyn Pause>,
) -> Result<DemoReport, CollectorError> {
    let demos_dir = config.out_dir.join("demos");
    std::fs::create_dir_all(&demos_dir)?;
    if config.out_dir.join("stop.json").exists() {
        return Err(CollectorError::AlreadyStopped);
    }
    let (major, minor, patch) = config.ios;
    let ua = ios_user_agent(major, minor, patch).map_err(CollectorError::InvalidConfig)?;
    let mut report = DemoReport { targets_total: ids.len(), ..Default::default() };
    let mut last_request: Option<Instant> = None;
    'ids: for &id in ids {
        let dest = demos_dir.join(format!("{id}.bin"));
        if dest.is_file() {
            report.cached_hits += 1;
            continue;
        }
        let first_url = demo_url(base_url, id);
        let mut url = first_url.clone();
        let started = std::time::Instant::now();
        let mut hops = 0;
        loop {
            if config.max_requests.is_some_and(|m| report.requests_made >= m) {
                report.stopped_by_max_requests = true;
                break 'ids;
            }
            if let Some(prev) = last_request {
                let elapsed = (clock.now() - prev).to_std().unwrap_or(Duration::ZERO);
                if elapsed < config.min_interval {
                    pause.sleep(config.min_interval - elapsed).await;
                }
            }
            last_request = Some(clock.now());
            report.requests_made += 1;
            let part = demos_dir.join(format!("{id}.part"));
            let resp = http.download(&url, &ua, &part).await?;
            if is_cloudflare_challenge(resp.status, resp.cf_mitigated.as_deref(), &resp.body) {
                let _ = std::fs::remove_file(&part);
                let reason = if let Some(cf) = &resp.cf_mitigated {
                    format!("cf-mitigated={cf}")
                } else if resp.status == 429 {
                    "rate limited (429)".to_string()
                } else if resp.status == 403 {
                    "forbidden (403)".to_string()
                } else {
                    "Cloudflare challenge body".to_string()
                };
                let stop = DemoStop { at: clock.now(), demo_id: id, url: url.clone(), status: resp.status, reason };
                let json = serde_json::to_string_pretty(&stop)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
                write_atomic(&config.out_dir.join("stop.json"), (json + "\n").as_bytes())?;
                report.stopped = Some(stop);
                break 'ids;
            }
            match resp.status {
                200 if resp.bytes_written > 0 => {
                    std::fs::rename(&part, &dest)?;
                    report.downloaded += 1;
                    report.bytes += resp.bytes_written;
                    append_line(
                        &config.out_dir.join("demos.tsv"),
                        TSV_HEADER,
                        &format!(
                            "{id}\t{first_url}\t{url}\t{}\t{:.1}\t{}\t200\t{}\t{}",
                            clock.now().to_rfc3339(),
                            started.elapsed().as_secs_f64(),
                            resp.bytes_written,
                            resp.content_type.as_deref().unwrap_or(""),
                            resp.content_disposition.as_deref().unwrap_or("").replace(['\t', '\n'], " "),
                        ),
                    )?;
                    break;
                }
                301 | 302 | 303 | 307 | 308 if hops < 3 && resp.location.is_some() => {
                    match redirect_target(&url, resp.location.as_deref().unwrap_or("")) {
                        Ok(next) => {
                            url = next;
                            hops += 1;
                        }
                        Err(e) => {
                            // Reported as a stop: not followed, not retried.
                            let stop = DemoStop {
                                at: clock.now(),
                                demo_id: id,
                                url: url.clone(),
                                status: resp.status,
                                reason: e.to_string(),
                            };
                            let json = serde_json::to_string_pretty(&stop)
                                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
                            write_atomic(&config.out_dir.join("stop.json"), (json + "\n").as_bytes())?;
                            report.stopped = Some(stop);
                            break 'ids;
                        }
                    }
                }
                404 | 410 => {
                    append_line(
                        &config.out_dir.join("demos.tsv"),
                        TSV_HEADER,
                        &format!("{id}\t{first_url}\t{url}\t{}\t0.0\t0\t{}\t\t", clock.now().to_rfc3339(), resp.status),
                    )?;
                    break;
                }
                other => return Err(CollectorError::HttpStatus(other)),
            }
        }
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redirects_stay_on_hltv() {
        let o = "https://www.hltv.org/download/demo/1";
        assert_eq!(redirect_target(o, "/x").unwrap(), "https://www.hltv.org/x");
        assert!(redirect_target(o, "https://r2-demos.hltv.org/a.rar").is_ok());
        assert!(redirect_target(o, "https://evil.example/a.rar").is_err());
        assert!(redirect_target(o, "http://demos.hltv.org/a.rar").is_err());
        assert!(redirect_target(o, "https://nothltv.org/a").is_err());
    }
}
