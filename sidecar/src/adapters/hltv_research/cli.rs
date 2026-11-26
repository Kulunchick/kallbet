//! `kallbet-ai hltv-collect`: fetch one historical HLTV Match's map, veto and
//! player statistics into the research archive format with a cache, rate limit,
//! and hard stop rule (ticket #36).

use super::collector::{CollectorConfig, HltvCollector, ReqwestHttp, ResearchHttp, TokioPause, UaMode};
use crate::{Result, adapters::baseline::SystemClock};
use std::{path::PathBuf, sync::Arc, time::Duration};

pub const USAGE: &str = "usage: kallbet-ai hltv-collect --match-id ID --url URL --out DIR \
[--min-interval-secs N] [--ios-version MAJOR.MINOR.PATCH]";

#[derive(Debug, PartialEq, Eq)]
pub struct Options {
    pub match_id: i64,
    pub url: String,
    pub out: PathBuf,
    pub min_interval_secs: u64,
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

pub fn parse(args: &[String]) -> Result<Options> {
    let mut match_id: Option<i64> = None;
    let mut url: Option<String> = None;
    let mut out: Option<PathBuf> = None;
    let mut min_interval_secs: u64 = 5;
    let mut ios_major: u8 = 17;
    let mut ios_minor: u8 = 2;
    let mut ios_patch: u8 = 1;

    let mut it = args.iter();
    while let Some(arg) = it.next() {
        let mut value = || it.next().cloned().ok_or(format!("{arg} needs a value"));
        match arg.as_str() {
            "--match-id" => match_id = Some(value()?.parse()?),
            "--url" => url = Some(value()?),
            "--out" => out = Some(value()?.into()),
            "--min-interval-secs" => min_interval_secs = value()?.parse()?,
            "--ios-version" => {
                let (ma, mi, pa) = parse_ios_version(&value()?)?;
                ios_major = ma;
                ios_minor = mi;
                ios_patch = pa;
            }
            other => return Err(format!("unknown hltv-collect option {other:?}\n{USAGE}").into()),
        }
    }

    if min_interval_secs < 2 {
        return Err("--min-interval-secs must be at least 2".into());
    }

    Ok(Options {
        match_id: match_id.ok_or_else(|| format!("missing --match-id\n{USAGE}"))?,
        url: url.ok_or_else(|| format!("missing --url\n{USAGE}"))?,
        out: out.ok_or_else(|| format!("missing --out\n{USAGE}"))?,
        min_interval_secs,
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
    let http = build_transport(&o.url, o.ios_major)?;
    let mut collector = HltvCollector::new(
        CollectorConfig {
            out_dir: o.out,
            min_interval: Duration::from_secs(o.min_interval_secs),
            ua_mode: UaMode::FixedPerRun {
                major: o.ios_major,
                minor: o.ios_minor,
                patch: o.ios_patch,
            },
        },
        http,
        Arc::new(SystemClock),
        Arc::new(TokioPause),
    );
    let outcome = collector.collect_match(o.match_id, &o.url).await?;
    Ok(format!(
        "hltv-collect: match {} ({} vs {}, {} maps, {} veto steps, cached={}) -> {}",
        outcome.record.match_id,
        outcome.record.team_a.name,
        outcome.record.team_b.name,
        outcome.record.maps.len(),
        outcome.record.veto.len(),
        outcome.from_cache,
        outcome.json_path.display()
    ))
}
