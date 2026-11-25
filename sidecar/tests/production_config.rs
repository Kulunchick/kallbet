//! The shipped example configuration and env files stay usable: they load, they
//! compose into a full sidecar, and the env example lists every variable the
//! code reads.
use kallbet_ai::{Config, Policy, Sidecar};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

const EXAMPLES: [&str; 3] = [
    "sidecar.compose.example.json",
    "sidecar.local.example.json",
    "sidecar.proxy.example.json",
];
fn deploy_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../deploy/sidecar")
}
fn example(name: &str) -> Value {
    serde_json::from_slice(&std::fs::read(deploy_dir().join(name)).unwrap()).unwrap()
}
fn load(name: &str) -> Config {
    Config::load(deploy_dir().join(name)).unwrap()
}

#[test]
fn every_example_has_the_oracle_and_both_baselines() {
    for name in EXAMPLES {
        let config = load(name);
        let bot = |key: &str| config.bots.iter().find(|b| b.bot_key == key).unwrap();
        assert!(matches!(bot("oracle").policy, Policy::HybridAnchored), "{name}");
        assert!(matches!(bot("market").policy, Policy::MarketOnly), "{name}");
        assert!(matches!(bot("vrs").policy, Policy::VrsOnly), "{name}");
        assert!(!bot("oracle").provider_chain.is_empty(), "{name}");
        assert!(bot("market").provider_chain.is_empty(), "{name}");
        assert!(bot("vrs").provider_chain.is_empty(), "{name}");
        for bot in &config.bots {
            assert!(
                bot.invite_links.iter().all(|l| l.contains("/join/REPLACE_WITH_")),
                "{name}: an example must carry a placeholder Invite Link, not a real one"
            );
        }
    }
}

#[test]
fn the_debate_example_seats_claude_codex_and_antigravity() {
    let value = example("sidecar.compose.example.json");
    let oracle = &value["debate"]["bots"]["oracle"];
    assert_eq!(oracle["advocate"][0], "claude");
    assert_eq!(oracle["devil"][0], "codex");
    assert_eq!(oracle["arbiter"][0], "antigravity");
    let providers = value["llm"]["providers"].as_object().unwrap();
    for name in ["claude", "codex", "antigravity"] {
        assert!(providers.contains_key(name), "{name}");
    }
}

/// The examples are composed by `Sidecar::from_config`, which also validates the
/// debate section (seats, `lopsided`). Only the paths and the URL are replaced.
#[tokio::test]
async fn every_example_composes_into_a_sidecar() {
    for name in EXAMPLES {
        let dir = tempfile::tempdir().unwrap();
        let mut value = example(name);
        value["platform_url"] = json!("http://127.0.0.1:1");
        value["audit_file"] = json!(dir.path().join("sidecar.db"));
        value["vrs_files"] = json!([concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/vrs")]);
        let path = dir.path().join("sidecar.json");
        std::fs::write(&path, value.to_string()).unwrap();
        let sidecar = Sidecar::from_config(Config::load(&path).unwrap(), "secret".into())
            .await
            .unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(sidecar.bots.len(), 3, "{name}");
    }
}

#[test]
fn the_compose_example_uses_the_container_paths_compose_mounts() {
    let config = load("sidecar.compose.example.json");
    assert_eq!(config.audit_file, Path::new("/data/sidecar.db"));
    assert_eq!(config.vrs_files, [Path::new("/data/vrs")]);
    assert_eq!(config.platform_url, "http://kallbet:8095");
    let compose = std::fs::read_to_string(deploy_dir().join("../../docker-compose.yml")).unwrap();
    assert!(compose.contains("./vrs:/data/vrs:ro"));
    assert!(compose.contains("sidecar-data:/data"));
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// Every `SIDECAR_*`, `ODDSPAPI_*` or `KALLBET_*` variable the sidecar's source
/// names is documented in the env example. A new variable fails this test until
/// it is added there.
#[test]
fn the_env_example_lists_every_variable_the_sidecar_reads() {
    let example = std::fs::read_to_string(deploy_dir().join("sidecar.env.example")).unwrap();
    let documented = |name: &str| {
        example
            .lines()
            .any(|l| l.trim_start_matches('#').trim_start().starts_with(&format!("{name}=")))
    };
    let mut files = Vec::new();
    rust_files(&Path::new(env!("CARGO_MANIFEST_DIR")).join("src"), &mut files);
    let literal = is_env_name();
    let mut found = Vec::new();
    for file in files {
        let source = std::fs::read_to_string(&file).unwrap();
        for word in source.split('"').skip(1).step_by(2) {
            if literal(word) {
                found.push((word.to_string(), file.clone()));
            }
        }
    }
    assert!(found.len() >= 10, "the scan found too little: {found:?}");
    for (name, file) in found {
        assert!(documented(&name), "{name} (used in {}) is missing from sidecar.env.example", file.display());
    }
    // And the variables of the server side that the sidecar needs.
    assert!(documented("KALLBET_BOT_SECRET"));
}
fn is_env_name() -> impl Fn(&str) -> bool {
    |s: &str| {
        ["SIDECAR_", "ODDSPAPI_", "KALLBET_"].iter().any(|p| s.starts_with(p))
            && s.len() > 9
            && s.bytes().all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_')
    }
}
