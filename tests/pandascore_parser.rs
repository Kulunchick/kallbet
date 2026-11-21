//! Seam 2: the PandaScore parser, fed saved JSON instead of the REST API.
//! No network. See `tests/fixtures/pandascore/README.md` for which fixtures
//! are documentation examples and which are constructed.

use chrono::{TimeZone, Utc};
use kallbet::domain::{
    MatchFormat, MatchPlacement, MatchScore, MatchStatus, StageKind, StageOrdinal,
};
use kallbet::ports::SourceError;
use kallbet::sources::fallback::{ExternalMatch, ExternalSnapshot, Side};
use kallbet::sources::pandascore::parse::{self, TournamentInput};
use serde_json::{Value, json};

fn fixture(name: &str) -> Value {
    let path = format!("{}/tests/fixtures/pandascore/{name}", env!("CARGO_MANIFEST_DIR"));
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

/// `value[0]` of a documentation example file.
fn doc_example(name: &str) -> Value {
    fixture(name)["value"][0].clone()
}

fn inputs(doc: &Value) -> Vec<TournamentInput> {
    doc["tournaments"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| TournamentInput {
            tournament: t["tournament"].clone(),
            matches: t["matches"].as_array().unwrap().clone(),
        })
        .collect()
}

fn swiss_major() -> ExternalSnapshot {
    let doc = fixture("constructed-swiss-major.json");
    parse::snapshot(&doc["serie"], &inputs(&doc)).unwrap()
}

fn by_teams<'a>(s: &'a ExternalSnapshot, a: &str, b: &str) -> &'a ExternalMatch {
    s.matches
        .iter()
        .find(|m| {
            m.team_a.as_ref().map(|t| t.name.as_str()) == Some(a)
                && m.team_b.as_ref().map(|t| t.name.as_str()) == Some(b)
        })
        .unwrap_or_else(|| panic!("no match {a} vs {b}"))
}

#[test]
fn a_major_becomes_one_event_with_a_swiss_stage_and_the_playoffs() {
    let s = swiss_major();

    assert_eq!(s.event.external_id, "50");
    assert_eq!(s.event.name, "Constructed Major 2026");
    assert_eq!(
        s.event.starts_at,
        Some(Utc.with_ymd_and_hms(2026, 12, 1, 9, 0, 0).unwrap())
    );
    let stages: Vec<_> = s.stages.iter().map(|g| (g.ordinal, g.kind, g.name.as_str())).collect();
    assert_eq!(
        stages,
        vec![
            (StageOrdinal(1), StageKind::Swiss, "Stage 1 (Swiss)"),
            (StageOrdinal(2), StageKind::Playoffs, "Playoffs"),
        ]
    );
    assert_eq!(s.matches.len(), 12 + 7);
}

#[test]
fn swiss_rounds_and_record_buckets_are_replayed_from_the_results() {
    let s = swiss_major();
    let placement = |a, b| by_teams(&s, a, b).placement;

    let swiss = |round, wins, losses| MatchPlacement::Swiss { round, wins, losses };
    assert_eq!(placement("Alpha Team", "Bravo Esports"), swiss(1, 0, 0));
    assert_eq!(placement("Alpha Team", "Charlie Gaming"), swiss(2, 1, 0));
    assert_eq!(placement("Bravo Esports", "Delta"), swiss(2, 0, 1));
    assert_eq!(placement("Alpha Team", "Echo"), swiss(3, 2, 0));
    assert_eq!(placement("Charlie Gaming", "Golf"), swiss(3, 1, 1));
}

#[test]
fn a_swiss_match_without_two_known_teams_or_with_unequal_records_is_unplaced() {
    let s = swiss_major();
    let delta_vs_tbd = s
        .matches
        .iter()
        .find(|m| m.team_a.as_ref().is_some_and(|t| t.name == "Delta") && m.team_b.is_none())
        .unwrap();
    assert_eq!(delta_vs_tbd.placement, MatchPlacement::Unplaced);
    // Bravo is 0-2, Foxtrot's Match is still running: records differ.
    assert_eq!(by_teams(&s, "Bravo Esports", "Foxtrot").placement, MatchPlacement::Unplaced);
}

#[test]
fn a_finished_match_has_winner_score_format_and_start() {
    let s = swiss_major();
    let m = by_teams(&s, "Alpha Team", "Bravo Esports");

    assert_eq!(m.status, MatchStatus::Finished);
    assert_eq!(m.winner, Some(Side::A));
    assert_eq!(m.score, Some(MatchScore { team_a: 1, team_b: 0 }));
    assert_eq!(m.format, MatchFormat::Bo1);
    assert_eq!(m.scheduled_start, Some(Utc.with_ymd_and_hms(2026, 12, 1, 10, 0, 0).unwrap()));
    assert_eq!(m.actual_start, m.scheduled_start);
    assert_eq!(m.stage, StageOrdinal(1));
    assert_eq!(m.external_id, "101");
    let a = m.team_a.as_ref().unwrap();
    assert_eq!((a.external_id.as_str(), a.canonical), ("1", None));
    assert!(m.canonical.is_none());
}

#[test]
fn the_winner_is_the_side_of_winner_id_whichever_side_that_is() {
    let s = swiss_major();
    // Delta (side B) won against Bravo.
    let m = by_teams(&s, "Bravo Esports", "Delta");
    assert_eq!(m.winner, Some(Side::B));
    assert_eq!(m.score, Some(MatchScore { team_a: 0, team_b: 1 }));
}

#[test]
fn a_forfeit_is_a_finished_match_won_by_the_other_side() {
    let s = swiss_major();
    let m = by_teams(&s, "Echo", "Golf");
    assert_eq!(m.status, MatchStatus::Finished);
    assert_eq!(m.winner, Some(Side::A));
}

#[test]
fn a_running_match_is_live_with_its_actual_start() {
    let s = swiss_major();
    let m = by_teams(&s, "Foxtrot", "Hotel");
    assert_eq!(m.status, MatchStatus::Live);
    assert_eq!(m.winner, None);
    assert_eq!(m.actual_start, Some(Utc.with_ymd_and_hms(2026, 12, 2, 13, 7, 0).unwrap()));
}

#[test]
fn a_canceled_match_without_forfeit_is_cancelled_with_no_winner_or_start() {
    let s = swiss_major();
    let m = by_teams(&s, "Bravo Esports", "Foxtrot");
    assert_eq!(m.status, MatchStatus::Cancelled);
    assert_eq!(m.winner, None);
    assert_eq!(m.actual_start, None);
}

#[test]
fn a_postponed_match_is_upcoming_with_no_scheduled_start() {
    // PandaScore leaves `scheduled_at` stale on `postponed`: the time is unknown.
    let s = swiss_major();
    let m = by_teams(&s, "Alpha Team", "Echo");
    assert_eq!(m.status, MatchStatus::Upcoming);
    assert_eq!(m.scheduled_start, None);
    assert_eq!(m.format, MatchFormat::Bo3);
}

#[test]
fn playoffs_slots_are_derived_from_previous_matches() {
    let s = swiss_major();
    let playoffs: Vec<_> = s.matches.iter().filter(|m| m.stage == StageOrdinal(2)).collect();
    let mut placements: Vec<_> = playoffs.iter().map(|m| m.placement).collect();
    placements.sort_by_key(|p| match p {
        MatchPlacement::Playoffs { rounds_before_final, slot } => (*rounds_before_final, *slot),
        _ => (99, 99),
    });
    let p = |r, s| MatchPlacement::Playoffs { rounds_before_final: r, slot: s };
    assert_eq!(
        placements,
        vec![p(0, 0), p(1, 0), p(1, 1), p(2, 0), p(2, 1), p(2, 2), p(2, 3)]
    );
    // Both teams still to be decided, final is a Bo5.
    let fin = playoffs.iter().find(|m| m.placement == p(0, 0)).unwrap();
    assert!(fin.team_a.is_none() && fin.team_b.is_none());
    assert_eq!(fin.format, MatchFormat::Bo5);
}

// The non-Swiss Event: documentation examples (CCT Europe, Playoffs only).

fn cct_playoffs_only() -> ExternalSnapshot {
    let m = doc_example("doc-example-csgo_matches-1-0.json");
    let serie = m["serie"].clone();
    let t = TournamentInput { tournament: m["tournament"].clone(), matches: vec![m] };
    parse::snapshot(&serie, &[t]).unwrap()
}

#[test]
fn an_event_without_a_swiss_stage_has_only_its_playoffs_stage() {
    let s = cct_playoffs_only();

    assert_eq!(s.event.external_id, "9682");
    assert_eq!(s.event.name, "European Contenders #1 season 3 2025");
    assert_eq!(s.stages.len(), 1);
    assert_eq!(s.stages[0].kind, StageKind::Playoffs);
    assert_eq!(s.stages[0].name, "Playoffs");
}

#[test]
fn a_tbd_versus_tbd_upcoming_match_parses_without_teams() {
    let s = cct_playoffs_only();
    let m = &s.matches[0];

    assert_eq!(m.external_id, "1246497");
    assert!(m.team_a.is_none() && m.team_b.is_none());
    assert_eq!(m.status, MatchStatus::Upcoming);
    assert_eq!(m.format, MatchFormat::Bo3);
    assert_eq!(m.scheduled_start, Some(Utc.with_ymd_and_hms(2025, 10, 1, 17, 30, 0).unwrap()));
    assert_eq!(m.score, None);
    // A lone bracket match with no `previous_matches` has no known slot.
    assert_eq!(m.placement, MatchPlacement::Unplaced);
}

#[test]
fn a_stage_without_a_bracket_that_is_not_swiss_is_a_groups_stage() {
    let m = doc_example("doc-example-csgo_matches-1-0.json");
    let serie = m["serie"].clone();
    let mut tournament = m["tournament"].clone();
    tournament["has_bracket"] = json!(false);
    tournament["name"] = json!("Group Stage");
    let s = parse::snapshot(&serie, &[TournamentInput { tournament, matches: vec![m] }]).unwrap();

    assert_eq!(s.stages[0].kind, StageKind::Groups);
    assert_eq!(s.matches[0].placement, MatchPlacement::Unplaced);
}

#[test]
fn a_finished_documentation_match_carries_teams_scores_and_winner() {
    // The documentation's bracket example (shape shared by all games).
    let m = doc_example("doc-example-tournaments_brackets-0.json");
    let (a_id, b_id) = (
        m["opponents"][0]["opponent"]["id"].as_i64().unwrap(),
        m["opponents"][1]["opponent"]["id"].as_i64().unwrap(),
    );
    let parsed = parse::match_record(&m, StageOrdinal(1), MatchPlacement::Unplaced).unwrap();

    assert_eq!(parsed.status, MatchStatus::Finished);
    assert_eq!(parsed.team_a.as_ref().unwrap().external_id, a_id.to_string());
    assert_eq!(parsed.team_b.as_ref().unwrap().external_id, b_id.to_string());
    assert_eq!(parsed.format, MatchFormat::Bo5);
    assert_eq!(parsed.score, Some(MatchScore { team_a: 3, team_b: 2 }));
    assert_eq!(parsed.winner, Some(Side::A));
}

// Anything not understood is an error, never empty data.

fn parse_err(mutate: impl FnOnce(&mut Value)) -> SourceError {
    let mut m = doc_example("doc-example-csgo_matches-1-0.json");
    mutate(&mut m);
    let serie = m["serie"].clone();
    let t = TournamentInput { tournament: m["tournament"].clone(), matches: vec![m] };
    parse::snapshot(&serie, &[t]).unwrap_err()
}

#[test]
fn an_unknown_status_is_a_parse_error() {
    assert!(matches!(parse_err(|m| m["status"] = json!("abandoned")), SourceError::Parse(_)));
}

#[test]
fn a_format_other_than_best_of_one_three_or_five_is_a_parse_error() {
    assert!(matches!(
        parse_err(|m| m["match_type"] = json!("first_to")),
        SourceError::Parse(_)
    ));
    assert!(matches!(parse_err(|m| m["number_of_games"] = json!(2)), SourceError::Parse(_)));
}

#[test]
fn a_match_without_an_id_is_a_parse_error() {
    assert!(matches!(
        parse_err(|m| {
            m.as_object_mut().unwrap().remove("id");
        }),
        SourceError::Parse(_)
    ));
}

#[test]
fn a_response_that_is_not_a_serie_is_a_parse_error() {
    let err = parse::snapshot(&json!({"error": "Not Found"}), &[]).unwrap_err();
    assert!(matches!(err, SourceError::Parse(_)));
}

#[test]
fn an_event_list_is_read_from_series_objects() {
    let m = doc_example("doc-example-csgo_matches-1-0.json");
    let events = parse::events(&json!([m["serie"].clone()])).unwrap();

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].external_id, "9682");
    assert_eq!(events[0].canonical, None);
    assert_eq!(
        events[0].ends_at,
        Some(Utc.with_ymd_and_hms(2025, 10, 1, 20, 30, 0).unwrap())
    );
}
