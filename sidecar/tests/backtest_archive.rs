//! Seam 2 of ticket #31: the replay archive read from saved real responses
//! (`tests/fixtures/backtest/`, provenance in its README).
use kallbet_ai::{
    adapters::backtest::{ReplayArchive, StageKind, archive::team_key, feeds::kalshi_start},
    *,
};
use std::path::Path;

fn archive_of(prefixes: &[&str]) -> ReplayArchive {
    let dir = tempfile::tempdir().unwrap();
    let saved = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/backtest");
    for entry in std::fs::read_dir(saved).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        if prefixes.iter().any(|p| name.starts_with(p)) {
            std::fs::copy(&path, dir.path().join(&name)).unwrap();
        }
    }
    ReplayArchive::load(dir.path()).unwrap()
}

#[test]
fn research_daily_roster_is_linked_without_a_pickem_page() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("panda-7-2.json"), "[]").unwrap();
    std::fs::write(
        dir.path().join("roster-99-2026-10-01.json"),
        r#"{"query":{"pages":[{"title":"Example","revisions":[]}]}}"#,
    )
    .unwrap();
    std::fs::write(dir.path().join("roster-99-2026-10-02.json"),
        r#"{"query":{"pages":[{"title":"Example","revisions":[{"timestamp":"2026-10-01T12:00:00Z"}]}]}}"#).unwrap();
    let archive = ReplayArchive::load(dir.path()).unwrap();
    assert_eq!(archive.event_titles[&99], "Example");
    assert_eq!(archive.event_pages["Example"].len(), 1);
}

fn name_of(stage: &kallbet_ai::adapters::backtest::ReplayStage, id: i64) -> String {
    stage
        .teams
        .iter()
        .find(|t| t.id == id)
        .unwrap()
        .name
        .clone()
}

#[test]
fn a_swiss_stage_opens_with_its_teams_round_1_pairs_and_seeds_only() {
    let archive = archive_of(&["panda-22378", "lp-22378"]);
    let stage = &archive.stages[0];
    assert_eq!(
        (stage.kind, stage.name.as_str()),
        (StageKind::Swiss, "Stage 1")
    );
    assert_eq!(stage.event, "PGL Major Singapore 2026");
    // The first Match (Heroic vs Imperial) was scheduled for 04:00 and began 04:08.
    assert_eq!(
        stage.cutoff,
        "2026-11-24T04:00:00Z".parse::<Instant>().unwrap()
    );
    let opening = stage.opening.as_ref().unwrap();
    assert_eq!(
        opening.observed_at,
        "2026-11-16T14:38:40Z".parse::<Instant>().unwrap()
    );
    let view = &opening.value;
    assert_eq!(view["teams"].as_array().unwrap().len(), 16);
    assert_eq!(view["groups"].as_array().unwrap().len(), 1, "Round 1 only");
    let first = &view["groups"][0]["matches"][0];
    assert_eq!(first["team_a"]["name"], "Heroic");
    assert_eq!(first["team_b"]["name"], "Imperial");
    assert_eq!(first["format"], "bo1");
    assert!(first.get("winner").is_none());
    // Seeded from Valve's standings: NIP first, Lynn Vision last.
    let seeds = view["seeds"].as_array().unwrap();
    assert_eq!(name_of(stage, seeds[0].as_i64().unwrap()), "NIP");
    assert_eq!(name_of(stage, seeds[15].as_i64().unwrap()), "Lynn Vision");
    assert_eq!(stage.opening_matches.len(), 8);
}

#[test]
fn a_2024_stage_page_gives_its_seeds_from_the_old_standings_table() {
    let archive = archive_of(&["panda-13056", "lp-13056"]);
    let stage = &archive.stages[0];
    let view = &stage.opening.as_ref().unwrap().value;
    assert_eq!(view["seeds"].as_array().unwrap().len(), 16);
    assert!(stage.opening.as_ref().unwrap().observed_at < stage.cutoff);
}

#[test]
fn a_playoffs_bracket_opens_with_its_quarterfinals_and_a_bo5_final() {
    let archive = archive_of(&["panda-20710", "lp-20710"]);
    let stage = &archive.stages[0];
    assert_eq!(stage.kind, StageKind::Playoffs);
    let slots = stage.opening.as_ref().unwrap().value["bracket"]["slots"]
        .as_array()
        .unwrap()
        .clone();
    assert_eq!(slots.len(), 7);
    assert_eq!(slots.iter().filter(|s| s["leaf"] == true).count(), 4);
    assert!(
        slots
            .iter()
            .filter(|s| s["leaf"] == false)
            .all(|s| s["team_a"].is_null())
    );
    let last = slots.last().unwrap();
    assert_eq!(
        (
            last["rounds_before_final"].as_u64(),
            last["format"].as_str()
        ),
        (Some(0), Some("bo5"))
    );
    // Every finished series is a result, for Pick'em Points.
    assert_eq!(stage.results.len(), 7);
}

#[test]
fn names_from_every_source_meet_on_one_key() {
    for (a, b) in [
        ("NIP", "Ninjas in Pyjamas"),
        ("PVISION", "PARIVISION"),
        ("GOSH", "BC.Game"),
        ("tl", "Team Liquid"),
        ("CW", "Chinggis Warriors"),
        ("Natus Vincere", "navi"),
    ] {
        assert_eq!(team_key(a), team_key(b), "{a} / {b}");
    }
}

#[test]
fn a_kalshi_ticker_names_its_start_in_us_eastern_time() {
    let utc = |s: &str| s.parse::<Instant>().unwrap();
    // "originally scheduled for Nov 25, 2026 at 5:30 AM EST" (rules_primary).
    assert_eq!(
        kalshi_start("KXCS2GAME-26NOV250530CWGOSH"),
        Some(utc("2026-11-25T10:30:00Z"))
    );
    assert_eq!(
        kalshi_start("KXCS2GAME-26JUN020630FLYSIN"),
        Some(utc("2026-06-02T10:30:00Z"))
    );
    assert_eq!(kalshi_start("KXCS2GAME-nonsense"), None);
}
