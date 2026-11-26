//! `kallbet-ai hltv-collect`: fetch historical HLTV `/matches/*` map, veto and
//! player statistics into the research archive format with an on-disk cache,
//! rate limit (8-10 s), daily UTC budget (default 400), and hard stop rule.
//!
//! Single-match mode:
//!   kallbet-ai hltv-collect --match-id ID --url URL --out DIR
//!       [--min-interval-secs 9] [--daily-budget 400] [--ios-version 17.2.1]
//!
//! Resumable archive batch mode:
//!   kallbet-ai hltv-collect --archive DIR [--out DIR/hltv]
//!       [--base-url https://www.hltv.org] [--min-interval-secs 9]
//!       [--daily-budget 400] [--max-requests N] [--ios-version 17.2.1]

use super::collector::{
    CollectorConfig, DEFAULT_DAILY_BUDGET, DEFAULT_MIN_INTERVAL_SECS, HltvCollector,
    REMOTE_MIN_INTERVAL_SECS, ReqwestHttp, ResearchHttp, TokioPause, UaMode,
    discover_archive_targets,
};
use crate::{Result, adapters::baseline::SystemClock};
use std::{path::PathBuf, sync::Arc, time::Duration};

pub const USAGE: &str = "usage: kallbet-ai hltv-collect (--match-id ID --url URL --out DIR | --archive DIR [--out DIR]) \
[--base-url URL] [--min-interval-secs N] [--daily-budget N] [--max-requests N] [--ios-version MAJOR.MINOR.PATCH]";

#[derive(Debug, PartialEq, Eq)]
pub enum Mode {
    Single { match_id: i64, url: String },
    Archive { archive_dir: PathBuf, base_url: String },
}

#[derive(Debug, PartialEq, Eq)]
pub struct Options {
    pub mode: Mode,
    pub out: PathBuf,
    pub min_interval_secs: u64,
    pub daily_budget: u32,
    pub max_requests: Option<u32>,
    pub ios_major: u8,
    pub ios_minor: u8,
    pub ios_patch: u8,
}

fn parse_ios_version(raw: &str) -> Result<(u8, u8, u8)> {
    let parts: Vec<&str> = raw.split('.').collect();
    if parts.len() != 3 {
        return Err(format!("invalid --ios-version {raw:?}; expected MAJOR.MINOR.PATCH").into());
    }
    Ok((parts[0].parse()?, parts[1].parse()?, parts[2].parse()?))
}

/// Picks one iOS version per run from the allowed pattern (major 14..=17,
/// minor 0..=9, patch 0..=9) using the current clock nanos when `--ios-version`
/// is omitted.
fn random_ios_version_for_run() -> (u8, u8, u8) {
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(17_2_1);
    let major = 14 + (seed % 4) as u8;
    let minor = ((seed / 7) % 10) as u8;
    let patch = ((seed / 71) % 10) as u8;
    (major, minor, patch)
}

pub fn parse(args: &[String]) -> Result<Options> {
    let mut match_id: Option<i64> = None;
    let mut url: Option<String> = None;
    let mut archive_dir: Option<PathBuf> = None;
    let mut base_url = "https://www.hltv.org".to_string();
    let mut out: Option<PathBuf> = None;
    let mut min_interval_secs: u64 = DEFAULT_MIN_INTERVAL_SECS;
    let mut daily_budget: u32 = DEFAULT_DAILY_BUDGET;
    let mut max_requests: Option<u32> = None;
    let mut ios_version: Option<(u8, u8, u8)> = None;

    let mut it = args.iter();
    while let Some(arg) = it.next() {
        let mut value = || it.next().cloned().ok_or(format!("{arg} needs a value"));
        match arg.as_str() {
            "--match-id" => match_id = Some(value()?.parse()?),
            "--url" => url = Some(value()?),
            "--archive" => archive_dir = Some(value()?.into()),
            "--base-url" => base_url = value()?,
            "--out" => out = Some(value()?.into()),
            "--min-interval-secs" => min_interval_secs = value()?.parse()?,
            "--daily-budget" => daily_budget = value()?.parse()?,
            "--max-requests" => max_requests = Some(value()?.parse()?),
            "--ios-version" => ios_version = Some(parse_ios_version(&value()?)?),
            other => return Err(format!("unknown hltv-collect option {other:?}\n{USAGE}").into()),
        }
    }

    let (mode, default_out, target_url_for_guard) = match (archive_dir, match_id, url) {
        (Some(dir), None, None) => {
            let def = dir.join("hltv");
            let u = base_url.clone();
            (Mode::Archive { archive_dir: dir, base_url }, Some(def), u)
        }
        (None, Some(id), Some(u)) => (Mode::Single { match_id: id, url: u.clone() }, None, u),
        _ => {
            return Err(format!(
                "specify either (--match-id ID --url URL --out DIR) or (--archive DIR)\n{USAGE}"
            )
            .into());
        }
    };

    let out = out
        .or(default_out)
        .ok_or_else(|| format!("missing --out\n{USAGE}"))?;

    let is_local = is_loopback_url(&target_url_for_guard);
    if is_local {
        if min_interval_secs < 2 {
            return Err("--min-interval-secs must be at least 2".into());
        }
    } else if min_interval_secs < REMOTE_MIN_INTERVAL_SECS {
        return Err(format!(
            "--min-interval-secs must be at least {REMOTE_MIN_INTERVAL_SECS} for remote HLTV requests"
        )
        .into());
    }

    let (ios_major, ios_minor, ios_patch) =
        ios_version.unwrap_or_else(random_ios_version_for_run);

    Ok(Options {
        mode,
        out,
        min_interval_secs,
        daily_budget,
        max_requests,
        ios_major,
        ios_minor,
        ios_patch,
    })
}

fn is_loopback_url(url: &str) -> bool {
    let Some(rest) = url.strip_prefix("http://") else {
        return false;
    };
    let host = rest.split(['/', ':']).next().unwrap_or("");
    host == "127.0.0.1" || host == "localhost" || host == "[::1]"
}

fn build_transport(url: &str, ios_major: u8) -> Result<Arc<dyn ResearchHttp>> {
    if is_loopback_url(url) {
        return Ok(Arc::new(ReqwestHttp::new()));
    }
    #[cfg(feature = "hltv-impersonate")]
    {
        return Ok(Arc::new(super::impersonate::WreqIosHttp::for_ios_major(
            ios_major,
        )?));
    }
    #[cfg(not(feature = "hltv-impersonate"))]
    {
        let _ = ios_major;
        Err("remote HLTV collection requires building kallbet-ai with `--features hltv-impersonate` \
             (kept out of the default build so the production image stays unchanged)"
            .into())
    }
}

pub async fn run(args: &[String]) -> Result<String> {
    let o = parse(args)?;
    let probe_url = match &o.mode {
        Mode::Single { url, .. } => url.as_str(),
        Mode::Archive { base_url, .. } => base_url.as_str(),
    };
    let http = build_transport(probe_url, o.ios_major)?;
    let mut collector = HltvCollector::new(
        CollectorConfig {
            out_dir: o.out.clone(),
            min_interval: Duration::from_secs(o.min_interval_secs),
            ua_mode: UaMode::FixedPerRun {
                major: o.ios_major,
                minor: o.ios_minor,
                patch: o.ios_patch,
            },
            daily_budget: o.daily_budget,
        },
        http,
        Arc::new(SystemClock),
        Arc::new(TokioPause),
    );

    match o.mode {
        Mode::Single { match_id, url } => {
            let outcome = collector.collect_match(match_id, &url).await?;
            Ok(format!(
                "hltv-collect: match {} ({} vs {}, {} maps, {} veto steps, cached={}, requests={}) -> {}",
                outcome.record.match_id,
                outcome.record.team_a.name,
                outcome.record.team_b.name,
                outcome.record.maps.len(),
                outcome.record.veto.len(),
                outcome.from_cache,
                outcome.requests_used,
                outcome.json_path.display()
            ))
        }
        Mode::Archive {
            archive_dir,
            base_url,
        } => {
            let targets = discover_archive_targets(&archive_dir, &o.out, &base_url)?;
            let report = collector.collect_batch(&targets, o.max_requests).await?;
            let stop_note = match &report.stopped_by_challenge {
                Some(s) => format!(" STOPPED(http={} {} saved={})", s.status, s.reason, s.saved_file),
                None if report.stopped_by_budget => " STOPPED(budget)".to_string(),
                None => String::new(),
            };
            Ok(format!(
                "hltv-collect: targets={} fetched={} cached={} parse_errors={} requests={} spent_today={}/{}{} out={}",
                report.targets_total,
                report.fetched_ok,
                report.cached_hits,
                report.parse_errors,
                report.requests_made,
                report.spent_today,
                report.daily_budget,
                stop_note,
                o.out.display()
            ))
        }
    }
}
