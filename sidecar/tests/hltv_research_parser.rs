//! Seam 2 parser tests on real saved HLTV and Liquipedia research fixtures (ticket #36).
//! No network. Every fixture's provenance is listed in `tests/fixtures/research/README.md`.

use chrono::{TimeZone, Utc};
use flate2::read::GzDecoder;
use kallbet_ai::adapters::hltv_research::{
    archive::{Side, VetoAction},
    liquipedia_maps::parse_stage_maps,
    parse::{ParseError, is_cloudflare_challenge, parse_match_page},
};
use std::{io::Read, path::Path};

fn fixture_dir() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/research"))
}

fn read_fixture(name: &str) -> String {
    let path = fixture_dir().join(name);
    let raw = std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    if name.ends_with(".gz") {
        let mut out = String::new();
        GzDecoder::new(&raw[..]).read_to_string(&mut out).unwrap();
        out
    } else {
        String::from_utf8(raw).unwrap()
    }
}

#[test]
fn parses_real_hltv_bo3_match_maps_veto_and_player_stats() {
    let html = read_fixture("hltv-match-2394898-vitality-vs-fut.html.gz");
    let fetched_at = Utc.with_ymd_and_hms(2026, 11, 26, 1, 15, 0).unwrap();
    let url = "https://www.hltv.org/matches/2394898/vitality-vs-fut-iem-cologne-major-2026";

    let rec = parse_match_page(2394898, url, fetched_at, &html).expect("parse real Bo3 page");

    assert_eq!(rec.match_id, 2394898);
    assert_eq!(rec.team_a.id, 9565);
    assert_eq!(rec.team_a.name, "Vitality");
    assert_eq!(rec.team_b.id, 13286);
    assert_eq!(rec.team_b.name, "FUT");
    assert_eq!(rec.event_id, Some(8301));
    assert_eq!(rec.event_name.as_deref(), Some("IEM Cologne Major 2026"));
    assert_eq!(
        rec.started_at,
        Utc.with_ymd_and_hms(2026, 6, 11, 12, 10, 0).unwrap()
    );
    assert_eq!(rec.format, "bo3");
    assert_eq!(rec.is_lan, Some(true));

    // 7-step Bo3 veto order
    assert_eq!(rec.veto.len(), 7);
    assert_eq!(rec.veto[0].step, 1);
    assert_eq!(rec.veto[0].team.as_deref(), Some("FUT"));
    assert_eq!(rec.veto[0].action, VetoAction::Removed);
    assert_eq!(rec.veto[0].map_name, "Inferno");
    assert_eq!(rec.veto[2].step, 3);
    assert_eq!(rec.veto[2].team.as_deref(), Some("FUT"));
    assert_eq!(rec.veto[2].action, VetoAction::Picked);
    assert_eq!(rec.veto[2].map_name, "Anubis");
    assert_eq!(rec.veto[3].step, 4);
    assert_eq!(rec.veto[3].team.as_deref(), Some("Vitality"));
    assert_eq!(rec.veto[3].action, VetoAction::Picked);
    assert_eq!(rec.veto[3].map_name, "Overpass");
    assert_eq!(rec.veto[6].step, 7);
    assert_eq!(rec.veto[6].team, None);
    assert_eq!(rec.veto[6].action, VetoAction::LeftOver);
    assert_eq!(rec.veto[6].map_name, "Nuke");

    // 3 played maps with scores and mapstatsids
    assert_eq!(rec.maps.len(), 3);
    assert_eq!(rec.maps[0].map_name, "Anubis");
    assert_eq!(rec.maps[0].team_a_score, Some(5));
    assert_eq!(rec.maps[0].team_b_score, Some(13));
    assert_eq!(rec.maps[0].winner, Some(Side::B));
    assert_eq!(rec.maps[0].mapstats_id, Some(231262));
    assert!(rec.maps[0].played);
    assert_eq!(rec.maps[1].map_name, "Overpass");
    assert_eq!(rec.maps[1].team_a_score, Some(13));
    assert_eq!(rec.maps[1].team_b_score, Some(2));
    assert_eq!(rec.maps[1].winner, Some(Side::A));
    assert_eq!(rec.maps[1].mapstats_id, Some(231264));
    assert_eq!(rec.maps[2].map_name, "Nuke");
    assert_eq!(rec.maps[2].team_a_score, Some(13));
    assert_eq!(rec.maps[2].team_b_score, Some(8));
    assert_eq!(rec.maps[2].winner, Some(Side::A));
    assert_eq!(rec.maps[2].mapstats_id, Some(231268));

    // Overall player statistics (2 teams x 5 players)
    assert_eq!(rec.overall_stats.len(), 2);
    assert_eq!(rec.overall_stats[0].team_id, Some(9565));
    assert_eq!(rec.overall_stats[0].players.len(), 5);
    assert_eq!(rec.overall_stats[1].team_id, Some(13286));
    assert_eq!(rec.overall_stats[1].players.len(), 5);

    let zywoo = &rec.overall_stats[0].players[0];
    assert_eq!(zywoo.player_id, 11893);
    assert_eq!(zywoo.nickname, "ZywOo");
    assert_eq!(zywoo.kills, 53);
    assert_eq!(zywoo.deaths, 30);
    assert_eq!(zywoo.ek_kills, Some(52));
    assert_eq!(zywoo.ek_deaths, Some(33));
    assert!((zywoo.swing_pct.unwrap() - 6.08).abs() < 1e-6);
    assert!((zywoo.adr.unwrap() - 101.1).abs() < 1e-6);
    assert!((zywoo.eadr.unwrap() - 98.3).abs() < 1e-6);
    assert!((zywoo.kast_pct.unwrap() - 75.9).abs() < 1e-6);
    assert!((zywoo.ekast_pct.unwrap() - 79.5).abs() < 1e-6);
    assert!((zywoo.rating - 1.71).abs() < 1e-6);
    assert_eq!(zywoo.rating_version, "3.0");

    // Per-map player statistics (3 maps)
    assert_eq!(rec.per_map_stats.len(), 3);
    let anubis = &rec.per_map_stats[0];
    assert_eq!(anubis.mapstats_id, 231262);
    assert_eq!(anubis.map_name.as_deref(), Some("Anubis"));
    assert_eq!(anubis.teams.len(), 2);
    assert_eq!(anubis.teams[1].players[0].nickname, "Krabeni");
    assert!((anubis.teams[1].players[0].rating - 1.85).abs() < 1e-6);
}

#[test]
fn parses_real_hltv_bo1_match_maps_veto_and_player_stats() {
    let html = read_fixture("hltv-match-2394774-heroic-vs-sharks.html.gz");
    let fetched_at = Utc.with_ymd_and_hms(2026, 11, 26, 1, 15, 0).unwrap();
    let url = "https://www.hltv.org/matches/2394774/heroic-vs-sharks-iem-cologne-major-2026-stage-1";

    let rec = parse_match_page(2394774, url, fetched_at, &html).expect("parse real Bo1 page");

    assert_eq!(rec.match_id, 2394774);
    assert_eq!(rec.team_a.id, 7175);
    assert_eq!(rec.team_a.name, "HEROIC");
    assert_eq!(rec.team_b.id, 8113);
    assert_eq!(rec.team_b.name, "Sharks");
    assert_eq!(rec.event_id, Some(9028));
    assert_eq!(
        rec.started_at,
        Utc.with_ymd_and_hms(2026, 6, 2, 12, 50, 0).unwrap()
    );
    assert_eq!(rec.format, "bo1");
    assert_eq!(rec.is_lan, Some(true));

    assert_eq!(rec.veto.len(), 7);
    assert!(
        rec.veto[..6]
            .iter()
            .all(|v| v.action == VetoAction::Removed)
    );
    assert_eq!(rec.veto[6].action, VetoAction::LeftOver);
    assert_eq!(rec.veto[6].map_name, "Nuke");

    assert_eq!(rec.maps.len(), 1);
    assert_eq!(rec.maps[0].map_name, "Nuke");
    assert_eq!(rec.maps[0].team_a_score, Some(10));
    assert_eq!(rec.maps[0].team_b_score, Some(13));
    assert_eq!(rec.maps[0].winner, Some(Side::B));
    assert_eq!(rec.maps[0].mapstats_id, Some(230770));

    assert_eq!(rec.overall_stats.len(), 2);
    assert_eq!(rec.overall_stats[0].players[0].nickname, "xfl0ud");
    assert!((rec.overall_stats[0].players[0].rating - 1.55).abs() < 1e-6);
    assert_eq!(rec.overall_stats[1].players[0].nickname, "maxxkor");
    assert!((rec.overall_stats[1].players[0].rating - 1.57).abs() < 1e-6);
    assert_eq!(rec.per_map_stats.len(), 1);
    assert_eq!(rec.per_map_stats[0].mapstats_id, 230770);
}

#[test]
fn rejects_real_cloudflare_challenge_and_broken_markup() {
    let challenge = read_fixture("hltv-mapstats-230770-403-challenge.html");
    assert!(is_cloudflare_challenge(403, Some("challenge"), &challenge));
    assert!(is_cloudflare_challenge(200, None, &challenge));

    let fetched_at = Utc.with_ymd_and_hms(2026, 11, 26, 1, 15, 0).unwrap();
    let err = parse_match_page(230770, "https://www.hltv.org/stats/matches/mapstatsid/230770/heroic-vs-sharks", fetched_at, &challenge)
        .expect_err("challenge HTML must fail parse");
    assert!(matches!(err, ParseError::CloudflareChallenge), "{err:?}");

    // Derived broken variant of a real page (renamed classes) must fail, never return empty data.
    let bo1 = read_fixture("hltv-match-2394774-heroic-vs-sharks.html.gz");
    let broken = bo1.replace("teamsBox", "renamedTeamsBox");
    let err = parse_match_page(2394774, "https://www.hltv.org/matches/2394774/x", fetched_at, &broken)
        .expect_err("broken markup must fail parse");
    assert!(matches!(err, ParseError::InvalidMarkup(_)), "{err:?}");
}

#[test]
fn parses_real_liquipedia_postmatch_stage_maps() {
    let raw = read_fixture("liquipedia-cologne-playoffs-postmatch.json");
    let parsed = parse_stage_maps(&raw).expect("parse Liquipedia Cologne Playoffs post-match");

    assert_eq!(parsed.title, "Intel Extreme Masters/2026/Cologne/Playoffs");
    assert_eq!(parsed.revid, 3390154);
    assert_eq!(parsed.matches.len(), 7);

    // Quarterfinal 1: Spirit vs G2 (includes 4x OT on Map 3 Mirage)
    let qf1 = parsed
        .matches
        .iter()
        .find(|m| m.hltv_match_id == Some(2394998))
        .expect("Spirit vs G2");
    assert_eq!(qf1.team_a_key, "spirit");
    assert_eq!(qf1.team_b_key, "g2");
    assert_eq!(qf1.maps.len(), 3);
    assert_eq!(qf1.maps[0].map_name, "Overpass");
    assert_eq!(qf1.maps[0].team_a_rounds, Some(9));
    assert_eq!(qf1.maps[0].team_b_rounds, Some(13));
    assert_eq!(qf1.maps[0].winner, Some(Side::B));
    assert_eq!(qf1.maps[0].mapstats_id, Some(231751));
    assert_eq!(qf1.maps[2].map_name, "Mirage");
    assert_eq!(qf1.maps[2].team_a_rounds, Some(25));
    assert_eq!(qf1.maps[2].team_b_rounds, Some(22));
    assert_eq!(qf1.maps[2].winner, Some(Side::A));

    // Grand Final: Falcons vs FURIA (Bo5 with 2 skipped maps)
    let gf = parsed
        .matches
        .iter()
        .find(|m| m.hltv_match_id == Some(2395002))
        .expect("Falcons vs FURIA");
    assert_eq!(gf.maps.len(), 5);
    assert!(gf.maps[0].played);
    assert!(gf.maps[2].played);
    assert!(!gf.maps[3].played);
    assert_eq!(gf.maps[3].map_name, "Dust II");
    assert_eq!(gf.maps[3].winner, None);
    assert!(!gf.maps[4].played);
    assert_eq!(gf.maps[4].map_name, "Nuke");
}
