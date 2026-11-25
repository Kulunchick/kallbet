mod support;
use kallbet::testing::TEST_BOT_SECRET;
use kallbet_ai::{Config, Sidecar};
use serde_json::json;
use support::Harness;

#[tokio::test]
async fn starts_with_two_profiles_and_an_independent_audit_file() {
    let h = Harness::new().await;
    let dir = tempfile::tempdir().unwrap();
    let audit = dir.path().join("ai.db");
    let path = dir.path().join("profiles.json");
    let profiles=["baseline","reference"].map(|name|json!({"bot_key":name,"nickname":name,"policy":"VrsOnly","provider_chain":["local"],"invite_links":[]}));
    std::fs::write(&path,json!({"platform_url":h.base,"audit_file":audit,"bots":profiles,"vrs_files":[concat!(env!("CARGO_MANIFEST_DIR"),"/tests/fixtures/vrs/standings_global_2026_11_02.md")]}).to_string()).unwrap();
    let config = Config::load(path).unwrap();
    let mut sidecar = Sidecar::from_config(config, TEST_BOT_SECRET.into())
        .await
        .unwrap();
    assert!(audit.is_file());
    assert_eq!(sidecar.bots.len(), 2);
    assert_eq!(sidecar.bots[1].provider_chain, ["local"]);
    assert_eq!(sidecar.tick().await.unwrap(), 0);
}
