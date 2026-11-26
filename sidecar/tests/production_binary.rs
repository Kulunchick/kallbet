//! The real `kallbet-ai` executable, run as a child process against the
//! in-process KallBet server of the shared harness.
mod support;
use chrono::{Duration, Utc};
use kallbet::testing::TEST_BOT_SECRET;
use kallbet_ai::adapters::production::{
    Health, Heartbeat,
    cli::{Command, insecure_platform, parse},
    settings::{LogFormat, Settings},
};
use serde_json::json;
use std::process::Stdio;
use support::Harness;
use tokio::process::Command as Process;

const VRS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/vrs");
const INVITE_CODE: &str = "INVITEcode_0123456789";

struct Setup {
    dir: tempfile::TempDir,
    config: std::path::PathBuf,
}
fn setup(platform: &str, invites: &[&str]) -> Setup {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("sidecar.json");
    std::fs::write(
        &config,
        json!({
            "platform_url": platform,
            "audit_file": dir.path().join("sidecar.db"),
            "vrs_files": [VRS],
            "bots": [{
                "bot_key": "vrs", "nickname": "VRS Baseline", "policy": "VrsOnly",
                "provider_chain": [],
                "invite_links": invites,
            }],
        })
        .to_string(),
    )
    .unwrap();
    Setup { dir, config }
}
fn binary(setup: &Setup, args: &[&str], secret: Option<&str>) -> Process {
    let mut command = Process::new(env!("CARGO_BIN_EXE_kallbet-ai"));
    command
        .args(args)
        .current_dir(setup.dir.path())
        .env("SIDECAR_CONFIG", &setup.config)
        .stdin(Stdio::null())
        .kill_on_drop(true);
    match secret {
        Some(secret) => command.env("KALLBET_BOT_SECRET", secret),
        None => command.env_remove("KALLBET_BOT_SECRET"),
    };
    command
}

#[tokio::test]
async fn one_tick_against_a_real_server_succeeds() {
    let h = Harness::new().await;
    let s = setup(&h.base, &[]);
    let out = binary(&s, &["run", "--once"], Some(TEST_BOT_SECRET))
        .output()
        .await
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "{stderr}");
    assert!(stderr.contains("kallbet-ai starting"), "{stderr}");
    assert!(stderr.contains("single tick ok"), "{stderr}");
    assert!(!stderr.contains(TEST_BOT_SECRET), "{stderr}");
}

#[tokio::test]
async fn a_dead_invite_link_is_skipped_without_the_invite_code_or_the_secret() {
    let h = Harness::new().await;
    // No League has this Invite Link (reissued, say): the join fails, the rest
    // of the tick goes on, and nothing in the output carries the code.
    let invite = format!("https://kallbet.example.com/join/{INVITE_CODE}");
    let s = setup(&h.base, &[&invite]);
    let out = binary(&s, &["run", "--once"], Some(TEST_BOT_SECRET))
        .output()
        .await
        .unwrap();
    let all = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(out.status.success(), "{all}");
    assert!(all.contains("single tick ok"), "{all}");
    assert_eq!(
        all.matches("Invite Link no longer works").count(),
        1,
        "{all}"
    );
    for secret in [TEST_BOT_SECRET, INVITE_CODE] {
        assert!(!all.contains(secret), "{secret} leaked: {all}");
    }
}

#[tokio::test]
async fn a_wrong_bot_secret_fails_the_tick_and_is_not_echoed() {
    let h = Harness::new().await;
    let s = setup(&h.base, &[]);
    let wrong = "wrong-secret-that-must-not-leak-0123";
    let out = binary(&s, &["run", "--once"], Some(wrong))
        .output()
        .await
        .unwrap();
    let all = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(!out.status.success());
    assert!(all.contains("401") || all.contains("Unauthorized"), "{all}");
    assert!(!all.contains(wrong), "{all}");
}

#[tokio::test]
async fn a_missing_secret_or_config_is_refused_with_a_plain_message() {
    let s = setup("http://127.0.0.1:1", &[]);
    let out = binary(&s, &["run", "--once"], None).output().await.unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("KALLBET_BOT_SECRET is not set"));

    let out = binary(&s, &["run", "--once"], Some("x"))
        .env_remove("SIDECAR_CONFIG")
        .output()
        .await
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("SIDECAR_CONFIG is not set"));
}

#[tokio::test]
async fn the_health_command_reads_the_heartbeat_beside_the_audit_file() {
    let s = setup("http://127.0.0.1:1", &[]);
    let out = binary(&s, &["health"], None).output().await.unwrap();
    assert!(!out.status.success(), "no heartbeat yet means unhealthy");
    assert!(String::from_utf8_lossy(&out.stdout).starts_with("unhealthy"));

    let health = Health::beside(&s.dir.path().join("sidecar.db"));
    let beat = |ago: Duration, failures| Heartbeat {
        started_at: Utc::now() - ago - Duration::seconds(2),
        finished_at: Some(Utc::now() - ago),
        consecutive_failures: failures,
        submitted: 0,
    };
    health.write(&beat(Duration::seconds(30), 0)).unwrap();
    let out = binary(&s, &["health"], None).output().await.unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stdout));

    health.write(&beat(Duration::hours(2), 0)).unwrap();
    let stale = binary(&s, &["health"], None).output().await.unwrap();
    assert!(!stale.status.success());

    health.write(&beat(Duration::seconds(30), 10)).unwrap();
    let failing = binary(&s, &["health"], None).output().await.unwrap();
    assert!(!failing.status.success());
}

#[tokio::test]
async fn an_unknown_command_shows_the_usage() {
    let s = setup("http://127.0.0.1:1", &[]);
    let out = binary(&s, &["frobnicate"], None).output().await.unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("usage: kallbet-ai"));
}

#[test]
fn commands_parse() {
    let args = |a: &[&str]| a.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    assert_eq!(parse(&args(&[])).unwrap(), Command::Run { once: false });
    assert_eq!(
        parse(&args(&["run", "--once"])).unwrap(),
        Command::Run { once: true }
    );
    assert_eq!(parse(&args(&["health"])).unwrap(), Command::Health);
    assert_eq!(
        parse(&args(&["backtest", "--event", "8302"])).unwrap(),
        Command::Backtest(args(&["--event", "8302"]))
    );
    assert_eq!(
        parse(&args(&["lab", "--seed", "7"])).unwrap(),
        Command::Lab(args(&["--seed", "7"]))
    );
    assert!(parse(&args(&["run", "--twice"])).is_err());
    assert!(parse(&args(&["nope"])).is_err());
}

fn settings(pairs: &[(&str, &str)]) -> kallbet_ai::Result<Settings> {
    Settings::from_lookup(&|name| {
        pairs
            .iter()
            .find(|(k, _)| *k == name)
            .map(|(_, v)| v.to_string())
    })
}

#[test]
fn settings_have_documented_defaults_and_reject_nonsense() {
    let s = settings(&[("SIDECAR_CONFIG", "c.json")]).unwrap();
    assert_eq!(s.tick_interval.as_secs(), 60);
    assert_eq!(s.tick_timeout.as_secs(), 900);
    assert_eq!(s.log_filter, "info");
    assert_eq!(s.log_format, LogFormat::Text);
    assert!(s.require_bot_secret().is_err());

    let s = settings(&[
        ("SIDECAR_CONFIG", "c.json"),
        ("KALLBET_BOT_SECRET", "abc"),
        ("SIDECAR_TICK_SECONDS", "30"),
        ("SIDECAR_LOG_FORMAT", "json"),
    ])
    .unwrap();
    assert_eq!(s.tick_interval.as_secs(), 30);
    assert_eq!(s.log_format, LogFormat::Json);
    assert_eq!(s.require_bot_secret().unwrap(), "abc");

    for bad in [
        ("SIDECAR_TICK_SECONDS", "1"),
        ("SIDECAR_TICK_SECONDS", "soon"),
        ("SIDECAR_LOG_FORMAT", "xml"),
    ] {
        assert!(settings(&[("SIDECAR_CONFIG", "c.json"), bad]).is_err(), "{bad:?}");
    }
    assert!(settings(&[]).is_err());
}

#[test]
fn plain_http_is_flagged_unless_it_stays_on_the_machine_or_the_compose_network() {
    for fine in [
        "https://kallbet.kulunchick.tech",
        "http://localhost:8095",
        "http://127.0.0.1:8095",
        "http://kallbet:8095",
        "http://10.0.0.5:8095",
    ] {
        assert!(insecure_platform(fine).is_none(), "{fine}");
    }
    assert!(insecure_platform("http://kallbet.kulunchick.tech").is_some());
    assert!(insecure_platform("http://203.0.113.9:8095").is_some());
}

/// SIGTERM (Docker's stop signal) and SIGINT end the loop cleanly, exit code 0.
#[cfg(unix)]
#[tokio::test]
async fn sigterm_and_sigint_stop_the_loop_with_exit_code_zero() {
    use tokio::io::{AsyncBufReadExt, BufReader};
    for signal in ["TERM", "INT"] {
        let h = Harness::new().await;
        let s = setup(&h.base, &[]);
        let mut child = binary(&s, &[], Some(TEST_BOT_SECRET))
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let mut lines = BufReader::new(child.stderr.take().unwrap()).lines();
        let mut seen = String::new();
        while let Some(line) = lines.next_line().await.unwrap() {
            seen += &line;
            seen += "\n";
            if line.contains("tick ok") {
                break;
            }
        }
        assert!(seen.contains("tick ok"), "{seen}");
        let pid = child.id().unwrap().to_string();
        let sent = std::process::Command::new("kill")
            .args([&format!("-{signal}"), &pid])
            .status()
            .unwrap();
        assert!(sent.success());
        let mut rest = String::new();
        while let Some(line) = lines.next_line().await.unwrap() {
            rest += &line;
        }
        let status = child.wait().await.unwrap();
        assert!(status.success(), "{status:?} {rest}");
        assert!(rest.contains("shutting down"), "{rest}");
    }
}
