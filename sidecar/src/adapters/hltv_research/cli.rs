//! `kallbet-ai hltv-collect`: fetch historical HLTV `/matches/*` map, veto and
//! player statistics into the research archive format with an on-disk cache,
//! per-connection rate limit, optional concurrency, daily UTC budget, and
//! global hard stop rule.
//!
//! Single-match mode:
//!   kallbet-ai hltv-collect --match-id ID --url URL --out DIR
//!       [--min-interval-secs 9] [--daily-budget 400] [--ios-version 17.2.1]
//!
//! Resumable archive batch mode:
//!   kallbet-ai hltv-collect --archive DIR [--out DIR/hltv]
//!       [--base-url https://www.hltv.org] [--concurrency 1] [--min-interval-secs 9]
//!       [--daily-budget 400] [--max-requests N] [--ios-version 17.2.1]

use super::demo::{DEMO_MIN_INTERVAL_SECS, DemoConfig, ResearchDownload, collect_demos};
use super::ranking::{RANKING_MIN_INTERVAL_SECS, RankingConfig, collect_rankings, mondays};
use super::collector::{
    CollectorConfig, DEFAULT_CONCURRENCY, DEFAULT_DAILY_BUDGET, DEFAULT_MIN_INTERVAL_SECS,
    HltvCollector, REMOTE_MIN_INTERVAL_SECS, ReqwestHttp, ResearchHttp, TokioPause, UaMode,
    discover_archive_targets,
};
use crate::{Clock, Result, adapters::baseline::SystemClock};
use chrono::NaiveDate;
use std::{path::PathBuf, sync::Arc, time::Duration};

pub const USAGE: &str = "usage: kallbet-ai hltv-collect (--match-id ID --url URL --out DIR | --archive DIR [--out DIR]) \
[--base-url URL] [--concurrency N] [--min-interval-secs N] [--daily-budget N] [--max-requests N] [--ios-version MAJOR.MINOR.PATCH] | --ranking-from YYYY-MM-DD --ranking-until YYYY-MM-DD --out DIR (weekly /ranking/teams/ pages into DIR/ranking, one connection, >= 6 s apart) | --demo-ids FILE --out DIR (demo pilot: /download/demo/<id> one at a time into DIR/demos, one connection, >= 6 s apart)";

#[derive(Debug, PartialEq, Eq)]
pub enum Mode {
    Single { match_id: i64, url: String },
    Archive { archive_dir: PathBuf, base_url: String },
    Ranking { from: NaiveDate, until: NaiveDate, base_url: String },
    Demo { ids: Vec<i64>, base_url: String },
}

#[derive(Debug, PartialEq, Eq)]
pub struct Options {
    pub mode: Mode,
    pub out: PathBuf,
    pub concurrency: usize,
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
/// minor 0..=9, patch 0..=9) seeded from the injected [`Clock`] when
/// `--ios-version` is omitted.
fn random_ios_version_for_run(clock: &dyn Clock) -> (u8, u8, u8) {
    let now = clock.now();
    let seed = now
        .timestamp_nanos_opt()
        .map(|n| n as u64)
        .unwrap_or_else(|| {
            (now.timestamp() as u64).wrapping_mul(1_000_000_000)
                ^ u64::from(now.timestamp_subsec_nanos())
        });
    let major = 14 + (seed % 4) as u8;
    let minor = ((seed / 7) % 10) as u8;
    let patch = ((seed / 71) % 10) as u8;
    (major, minor, patch)
}

pub fn parse(args: &[String]) -> Result<Options> {
    parse_with_clock(args, &SystemClock)
}

pub fn parse_with_clock(args: &[String], clock: &dyn Clock) -> Result<Options> {
    let mut match_id: Option<i64> = None;
    let mut url: Option<String> = None;
    let mut archive_dir: Option<PathBuf> = None;
    let mut base_url = "https://www.hltv.org".to_string();
    let mut out: Option<PathBuf> = None;
    let mut concurrency: usize = DEFAULT_CONCURRENCY;
    let mut min_interval_secs: u64 = DEFAULT_MIN_INTERVAL_SECS;
    let mut daily_budget: u32 = DEFAULT_DAILY_BUDGET;
    let mut max_requests: Option<u32> = None;
    let mut ios_version: Option<(u8, u8, u8)> = None;
    let mut ranking_from: Option<NaiveDate> = None;
    let mut ranking_until: Option<NaiveDate> = None;
    let mut demo_ids_file: Option<PathBuf> = None;

    let mut it = args.iter();
    while let Some(arg) = it.next() {
        let mut value = || it.next().cloned().ok_or(format!("{arg} needs a value"));
        match arg.as_str() {
            "--match-id" => match_id = Some(value()?.parse()?),
            "--url" => url = Some(value()?),
            "--archive" => archive_dir = Some(value()?.into()),
            "--base-url" => base_url = value()?,
            "--out" => out = Some(value()?.into()),
            "--concurrency" => concurrency = value()?.parse()?,
            "--min-interval-secs" => min_interval_secs = value()?.parse()?,
            "--daily-budget" => daily_budget = value()?.parse()?,
            "--max-requests" => max_requests = Some(value()?.parse()?),
            "--ranking-from" => ranking_from = Some(value()?.parse()?),
            "--ranking-until" => ranking_until = Some(value()?.parse()?),
            "--demo-ids" => demo_ids_file = Some(value()?.into()),
            "--ios-version" => ios_version = Some(parse_ios_version(&value()?)?),
            other => return Err(format!("unknown hltv-collect option {other:?}\n{USAGE}").into()),
        }
    }

    if concurrency == 0 {
        return Err("--concurrency must be at least 1".into());
    }

    let demo_ids: Option<Vec<i64>> = match demo_ids_file {
        None => None,
        Some(file) => {
            if archive_dir.is_some()
                || match_id.is_some()
                || url.is_some()
                || ranking_from.is_some()
                || ranking_until.is_some()
            {
                return Err(format!("--demo-ids cannot be combined with other modes
{USAGE}").into());
            }
            if concurrency != 1 {
                return Err("demo downloads use one connection: --concurrency must be 1".into());
            }
            if min_interval_secs < DEMO_MIN_INTERVAL_SECS {
                return Err(format!("--min-interval-secs must be at least {DEMO_MIN_INTERVAL_SECS} for demo downloads").into());
            }
            let text = std::fs::read_to_string(&file)?;
            let mut ids: Vec<i64> = Vec::new();
            for line in text.lines() {
                let first = line.split('\t').next().unwrap_or("").trim();
                if let Ok(id) = first.parse::<i64>()
                    && id > 0
                    && !ids.contains(&id)
                {
                    ids.push(id);
                }
            }
            Some(ids)
        }
    };

    let ranking = match (ranking_from, ranking_until) {
        (None, None) => None,
        (Some(from), Some(until)) if archive_dir.is_none() && match_id.is_none() && url.is_none() => {
            if from > until {
                return Err("--ranking-from must not be after --ranking-until".into());
            }
            if concurrency != 1 {
                return Err("ranking collection uses one connection: --concurrency must be 1".into());
            }
            if min_interval_secs < RANKING_MIN_INTERVAL_SECS {
                return Err(format!(
                    "--min-interval-secs must be at least {RANKING_MIN_INTERVAL_SECS} for ranking pages"
                )
                .into());
            }
            Some((from, until))
        }
        _ => {
            return Err(format!(
                "--ranking-from and --ranking-until go together and cannot be combined with --archive or --match-id
{USAGE}"
            )
            .into());
        }
    };
    let (mode, default_out, target_url_for_guard) = match (archive_dir, match_id, url) {
        _ if demo_ids.is_some() => {
            let u = base_url.clone();
            (Mode::Demo { ids: demo_ids.unwrap(), base_url }, None, u)
        }
        _ if ranking.is_some() => {
            let (from, until) = ranking.unwrap();
            let u = base_url.clone();
            (Mode::Ranking { from, until, base_url }, None, u)
        }
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
        ios_version.unwrap_or_else(|| random_ios_version_for_run(clock));

    Ok(Options {
        mode,
        out,
        concurrency,
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

fn build_download_transport(url: &str, ios_major: u8) -> Result<Arc<dyn ResearchDownload>> {
    if is_loopback_url(url) {
        return Err("demo downloads are only implemented for the impersonating transport".into());
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
        Err("demo downloads require building kallbet-ai with `--features hltv-impersonate`".into())
    }
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
    run_with_clock(args, Arc::new(SystemClock)).await
}

pub async fn run_with_clock(args: &[String], clock: Arc<dyn Clock>) -> Result<String> {
    let o = parse_with_clock(args, clock.as_ref())?;
    let probe_url = match &o.mode {
        Mode::Single { url, .. } => url.as_str(),
        Mode::Archive { base_url, .. } | Mode::Ranking { base_url, .. } | Mode::Demo { base_url, .. } => {
            base_url.as_str()
        }
    };
    if let Mode::Demo { ids, base_url } = &o.mode {
        let dl = build_download_transport(base_url, o.ios_major)?;
        let report = collect_demos(
            &DemoConfig {
                out_dir: o.out.clone(),
                min_interval: Duration::from_secs(o.min_interval_secs),
                ios: (o.ios_major, o.ios_minor, o.ios_patch),
                max_requests: o.max_requests,
            },
            base_url,
            ids,
            dl,
            clock,
            Arc::new(TokioPause),
        )
        .await?;
        let stop_note = match &report.stopped {
            Some(s) => format!(" STOPPED(demo={} http={} {})", s.demo_id, s.status, s.reason),
            None if report.stopped_by_max_requests => " STOPPED(max-requests)".to_string(),
            None => String::new(),
        };
        return Ok(format!(
            "hltv-collect demos: targets={} downloaded={} cached={} requests={} bytes={}{} out={}",
            report.targets_total,
            report.downloaded,
            report.cached_hits,
            report.requests_made,
            report.bytes,
            stop_note,
            o.out.join("demos").display()
        ));
    }
    let http = build_transport(probe_url, o.ios_major)?;
    let collector_clock = clock.clone();
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
            concurrency: o.concurrency,
        },
        http.clone(),
        clock,
        Arc::new(TokioPause),
    );

    match o.mode {
        Mode::Ranking { from, until, base_url } => {
            let dates = mondays(from, until);
            let report = collect_rankings(
                &RankingConfig {
                    out_dir: o.out.join("ranking"),
                    min_interval: Duration::from_secs(o.min_interval_secs),
                    ios: (o.ios_major, o.ios_minor, o.ios_patch),
                    max_requests: o.max_requests,
                },
                &base_url,
                &dates,
                http,
                collector_clock,
                Arc::new(TokioPause),
            )
            .await?;
            let stop_note = match &report.stopped {
                Some(s) => format!(" STOPPED(http={} {} saved={})", s.status, s.reason, s.saved_file),
                None if report.stopped_by_max_requests => " STOPPED(max-requests)".to_string(),
                None => String::new(),
            };
            Ok(format!(
                "hltv-collect ranking: targets={} fetched={} cached={} unavailable={} requests={}{} out={}",
                report.targets_total,
                report.fetched_ok,
                report.cached_hits,
                report.unavailable,
                report.requests_made,
                stop_note,
                o.out.join("ranking").display()
            ))
        }
        Mode::Demo { .. } => unreachable!("handled before the collector is built"),
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
                "hltv-collect: targets={} fetched={} cached={} parse_errors={} requests={} spent_today={}/{} concurrency={}{} out={}",
                report.targets_total,
                report.fetched_ok,
                report.cached_hits,
                report.parse_errors,
                report.requests_made,
                report.spent_today,
                report.daily_budget,
                o.concurrency,
                stop_note,
                o.out.display()
            ))
        }
    }
}
