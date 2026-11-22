//! Seam 2: the PandaScore parser, fed saved JSON instead of the REST API.
//! No network. Most tests run on real captures (`real-*.json`, see
//! `tests/fixtures/pandascore/README.md`); the cases no capture contains
//! (running, postponed, canceled, forfeit, one opponent TBD) run on the
//! labelled `constructed-statuses.json`.

use std::collections::BTreeMap;

use chrono::{TimeZone, Utc};
use kallbet::domain::{
    MatchFormat, MatchPlacement, MatchScore, MatchStatus, StageKind, StageOrdinal,
};
use kallbet::ports::SourceError;
use kallbet::sources::fallback::mapping::choose_event;
use kallbet::sources::fallback::{EventHint, ExternalMatch, ExternalSnapshot, Side};
use kallbet::sources::pandascore::parse::{self, TournamentInput};
use kallbet::domain::EventId;
use serde_json::{Value, json};

fn fixture(name: &str) -> Value {
    let path = format!("{}/tests/fixtures/pandascore/{name}", env!("CARGO_MANIFEST_DIR"));
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

/// `value[0]` of a documentation example file.
fn doc_example(name: &str) -> Value {
    fixture(name)["value"][0].clone()
}

fn by_id(items: &Value, id: i64) -> &Value {
    items.as_array().unwrap().iter().find(|i| i["id"] == id).unwrap()
}

fn array(v: &Value) -> Vec<Value> {
    v.as_array().unwrap().clone()
}

/// IEM Cologne Major 2026 (serie 10488) as the fetch layer reads it: the
/// matches of Stage 1 and of the Playoffs bracket were captured; Stages 2
/// and 3 were not, so they carry no matches here.
fn cologne() -> ExternalSnapshot {
    let serie = fixture("real-major-serie-filter.json")[0].clone();
    let tournaments = fixture("real-major-tournaments.json");
    let inputs: Vec<TournamentInput> = array(&tournaments)
        .into_iter()
        .map(|t| {
            let matches = match t["id"].as_i64().unwrap() {
                20708 => array(&fixture("real-major-stage1-matches.json")),
                20710 => array(&fixture("real-major-playoffs-brackets.json")),
                _ => vec![],
            };
            TournamentInput { tournament: t, matches }
        })
        .collect();
    parse::snapshot(&serie, &inputs).unwrap()
}

/// A serie of a list capture with its embedded tournaments and no matches.
fn stages_of(series_file: &str, serie_id: i64) -> Vec<(StageKind, String)> {
    let series = fixture(series_file);
    let serie = by_id(&series, serie_id);
    let inputs: Vec<TournamentInput> = array(&serie["tournaments"])
        .into_iter()
        .map(|t| TournamentInput { tournament: t, matches: vec![] })
        .collect();
    let s = parse::snapshot(serie, &inputs).unwrap();
    s.stages.into_iter().map(|g| (g.kind, g.name)).collect()
}

fn by_name<'a>(s: &'a ExternalSnapshot, name: &str, matches: &'a [Value]) -> &'a ExternalMatch {
    let id = matches.iter().find(|m| m["name"] == name).unwrap_or_else(|| panic!("no {name}"))["id"]
        .as_i64()
        .unwrap()
        .to_string();
    s.matches.iter().find(|m| m.external_id == id).unwrap()
}

// The Major: one Event, four Stages.

#[test]
fn a_major_becomes_one_event_named_with_its_league() {
    let s = cologne();

    assert_eq!(s.event.external_id, "10488");
    // The serie is "Cologne Major 2026"; the league "IEM" is a separate object.
    assert_eq!(s.event.name, "IEM Cologne Major 2026");
    assert_eq!(s.event.starts_at, Some(Utc.with_ymd_and_hms(2026, 6, 2, 10, 0, 0).unwrap()));
    assert_eq!(s.event.ends_at, Some(Utc.with_ymd_and_hms(2026, 6, 21, 21, 0, 0).unwrap()));
}

#[test]
fn the_majors_three_stages_are_swiss_and_the_playoffs_follow() {
    let s = cologne();

    let stages: Vec<_> = s.stages.iter().map(|g| (g.ordinal, g.kind, g.name.as_str())).collect();
    assert_eq!(
        stages,
        vec![
            (StageOrdinal(1), StageKind::Swiss, "Stage 1"),
            (StageOrdinal(2), StageKind::Swiss, "Stage 2"),
            (StageOrdinal(3), StageKind::Swiss, "Stage 3"),
            (StageOrdinal(4), StageKind::Playoffs, "Playoffs"),
        ]
    );
    assert_eq!(s.matches.len(), 33 + 7);
}

#[test]
fn other_majors_name_their_swiss_stages_differently_and_are_still_recognised() {
    use StageKind::{Groups, Playoffs, Swiss};
    let kinds = |file, id| stages_of(file, id).into_iter().map(|(k, _)| k).collect::<Vec<_>>();

    // Shanghai 2024: Opening Stage, Elimination Stage, Playoffs.
    assert_eq!(kinds("real-series-search.json", 8751), vec![Swiss, Swiss, Playoffs]);
    // Rio 2022: Challengers Stage, Legends Stage, Playoffs, Showmatch.
    assert_eq!(kinds("real-series-search.json", 5185), vec![Swiss, Swiss, Playoffs, Groups]);
    // Berlin 2019: Challengers, Legends, and a bracket called "Champions Stage".
    assert_eq!(kinds("real-series-search.json", 1824), vec![Swiss, Swiss, Playoffs]);
    // Singapore 2026 (not started): Stage 1-3 and Playoffs.
    assert_eq!(kinds("real-series-upcoming.json", 11176), vec![Swiss, Swiss, Swiss, Playoffs]);
}

#[test]
fn a_group_stage_flagged_with_a_bracket_is_groups_not_playoffs() {
    // Elisa Open Suomi: "Group Stage" has_bracket=true but is a round robin.
    assert_eq!(
        stages_of("real-series-running.json", 11156),
        vec![
            (StageKind::Groups, "Group Stage".to_string()),
            (StageKind::Playoffs, "Playoffs".to_string()),
        ]
    );
    // Play-In groups and lettered groups with brackets likewise.
    let kinds: Vec<_> =
        stages_of("real-series-running.json", 11177).into_iter().map(|(k, _)| k).collect();
    assert_eq!(kinds, vec![StageKind::Groups; 4].into_iter().chain([StageKind::Playoffs]).collect::<Vec<_>>());
}

// Swiss rounds and record buckets, replayed from Stage 1's real results.

fn stage1() -> (ExternalSnapshot, Vec<Value>) {
    (cologne(), array(&fixture("real-major-stage1-matches.json")))
}

#[test]
fn stage_1_has_five_swiss_rounds_with_the_record_buckets_of_a_sixteen_team_swiss() {
    let (s, raw) = stage1();
    let mut buckets: BTreeMap<(u8, u8, u8), usize> = BTreeMap::new();
    for m in s.matches.iter().filter(|m| m.stage == StageOrdinal(1)) {
        let MatchPlacement::Swiss { round, wins, losses } = m.placement else {
            panic!("{} is {:?}", m.external_id, m.placement);
        };
        *buckets.entry((round, wins, losses)).or_default() += 1;
    }
    let expected: BTreeMap<_, _> = [
        ((1, 0, 0), 8),
        ((2, 1, 0), 4),
        ((2, 0, 1), 4),
        ((3, 2, 0), 2),
        ((3, 1, 1), 4),
        ((3, 0, 2), 2),
        ((4, 2, 1), 3),
        ((4, 1, 2), 3),
        ((5, 2, 2), 3),
    ]
    .into_iter()
    .collect();
    assert_eq!(buckets, expected);
    assert_eq!(raw.len(), 33);
}

#[test]
fn a_swiss_matchs_round_agrees_with_the_round_in_its_name() {
    let (s, raw) = stage1();
    for m in &raw {
        let name = m["name"].as_str().unwrap();
        let round: u8 = name["Round ".len()..name.find(':').unwrap()].parse().unwrap();
        let parsed = s.matches.iter().find(|p| p.external_id == m["id"].as_i64().unwrap().to_string()).unwrap();
        assert!(
            matches!(parsed.placement, MatchPlacement::Swiss { round: r, .. } if r == round),
            "{name}: {:?}",
            parsed.placement
        );
    }
}

#[test]
fn bo3_is_exactly_the_advance_or_eliminate_match() {
    // At a Major, a team on two wins or two losses plays a Bo3; the rest Bo1.
    let (s, _) = stage1();
    for m in s.matches.iter().filter(|m| m.stage == StageOrdinal(1)) {
        let MatchPlacement::Swiss { wins, losses, .. } = m.placement else { unreachable!() };
        let decisive = wins == 2 || losses == 2;
        assert_eq!(m.format, if decisive { MatchFormat::Bo3 } else { MatchFormat::Bo1 }, "{m:?}");
    }
    assert_eq!(s.matches.iter().filter(|m| m.format == MatchFormat::Bo1).count(), 20);
    assert_eq!(s.matches.iter().filter(|m| m.stage == StageOrdinal(1) && m.format == MatchFormat::Bo3).count(), 13);
}

#[test]
fn a_finished_bo1_has_winner_score_and_both_starts() {
    let (s, raw) = stage1();
    let m = by_name(&s, "Round 1: M80 vs LVG", &raw);

    assert_eq!(m.status, MatchStatus::Finished);
    assert_eq!(m.team_a.as_ref().unwrap().name, "M80");
    assert_eq!(m.team_b.as_ref().unwrap().name, "Lynn Vision");
    assert_eq!(m.winner, Some(Side::A));
    assert_eq!(m.score, Some(MatchScore { team_a: 1, team_b: 0 }));
    assert_eq!(m.format, MatchFormat::Bo1);
    assert_eq!(m.scheduled_start, Some(Utc.with_ymd_and_hms(2026, 6, 2, 10, 30, 0).unwrap()));
    assert_eq!(m.actual_start, Some(Utc.with_ymd_and_hms(2026, 6, 2, 10, 31, 13).unwrap()));
    assert_eq!(m.stage, StageOrdinal(1));
    assert!(m.canonical.is_none() && m.team_a.as_ref().unwrap().canonical.is_none());
}

#[test]
fn the_winner_is_the_side_of_winner_id_whichever_side_that_is() {
    let (s, raw) = stage1();
    // FLY (side B) beat SIN; `results` lists SIN first.
    let m = by_name(&s, "Round 1: SIN vs FLY", &raw);
    assert_eq!(m.winner, Some(Side::B));
    assert_eq!(m.score, Some(MatchScore { team_a: 0, team_b: 1 }));
}

#[test]
fn a_bo3_score_counts_maps_per_team_in_the_opponents_order() {
    let (s, raw) = stage1();
    // "M80 vs B8" in Stage 1 round 3: results list M80 (0) before B8 (2).
    let m = by_name(&s, "Round 3: M80 vs B8", &raw);
    assert_eq!(m.format, MatchFormat::Bo3);
    assert_eq!(m.winner, Some(Side::B));
    assert_eq!(m.score, Some(MatchScore { team_a: 0, team_b: 2 }));
    // A 2-1 win.
    let m = by_name(&s, "Round 4: TYLOO vs SHK", &raw);
    assert_eq!(m.score, Some(MatchScore { team_a: 2, team_b: 1 }));
    assert_eq!(m.winner, Some(Side::A));
}

#[test]
fn a_not_yet_started_major_places_round_one_and_leaves_tbd_rounds_unplaced() {
    // Singapore's Stage 1 and 2 (as embedded in the tournaments list: no
    // opponents or results). Round 1 is always 0-0; later rounds depend on
    // results that do not exist yet.
    let serie = fixture("real-series-upcoming.json")[0].clone();
    let tournaments = fixture("real-upcoming-major-tournaments.json");
    let inputs: Vec<TournamentInput> = array(&tournaments)
        .into_iter()
        .map(|t| TournamentInput { matches: array(&t["matches"]), tournament: t })
        .collect();
    let s = parse::snapshot(&serie, &inputs).unwrap();

    assert_eq!(s.event.name, "PGL Major Singapore 2026");
    let swiss_1 = MatchPlacement::Swiss { round: 1, wins: 0, losses: 0 };
    let stage_2: Vec<_> = s.matches.iter().filter(|m| m.stage == StageOrdinal(2)).collect();
    assert_eq!(stage_2.iter().filter(|m| m.placement == swiss_1).count(), 8);
    assert_eq!(stage_2.iter().filter(|m| m.placement == MatchPlacement::Unplaced).count(), 33 - 8);
    assert!(stage_2.iter().all(|m| m.status == MatchStatus::Upcoming && m.winner.is_none()));
}

// The Playoffs bracket.

#[test]
fn the_playoffs_bracket_has_its_slot_structure_and_formats() {
    let s = cologne();
    let raw = array(&fixture("real-major-playoffs-brackets.json"));
    let at = |name: &str| by_name(&s, name, &raw);
    let p = |r, slot| MatchPlacement::Playoffs { rounds_before_final: r, slot };

    assert_eq!(at("Grand final: FURIA vs FAL").placement, p(0, 0));
    assert_eq!(at("Semifinal 1: TS vs FAL").placement, p(1, 0));
    assert_eq!(at("Semifinal 2: AUR vs FURIA").placement, p(1, 1));
    // Quarterfinals top to bottom: 1 and 2 feed semifinal 1, 3 and 4 semifinal 2.
    assert_eq!(at("Quarterfinal 1: TS vs G2").placement, p(2, 0));
    assert_eq!(at("Quarterfinal 2: VIT vs FAL").placement, p(2, 1));
    assert_eq!(at("Quarterfinal 3: FURIA vs 9z").placement, p(2, 2));
    assert_eq!(at("Quarterfinal 4: AUR vs BB").placement, p(2, 3));

    assert_eq!(at("Grand final: FURIA vs FAL").format, MatchFormat::Bo5);
    assert_eq!(at("Semifinal 1: TS vs FAL").format, MatchFormat::Bo3);
    assert_eq!(at("Quarterfinal 3: FURIA vs 9z").format, MatchFormat::Bo3);
    assert!(s.matches.iter().filter(|m| m.stage == StageOrdinal(4)).all(|m| m.status == MatchStatus::Finished));
}

#[test]
fn bracket_winners_and_teams_come_through() {
    let s = cologne();
    let raw = array(&fixture("real-major-playoffs-brackets.json"));
    let fin = by_name(&s, "Grand final: FURIA vs FAL", &raw);

    assert_eq!(fin.team_a.as_ref().unwrap().name, "FURIA");
    assert_eq!(fin.team_b.as_ref().unwrap().name, "Team Falcons");
    assert_eq!(fin.winner, Some(Side::B));
    // Qualified teams were listed under their own names in the quarterfinals.
    let qf = by_name(&s, "Quarterfinal 1: TS vs G2", &raw);
    assert_eq!(qf.team_a.as_ref().unwrap().name, "Spirit");
    assert_eq!(qf.winner, Some(Side::A));
}

#[test]
fn a_double_elimination_bracket_is_the_playoffs_with_unplaced_matches() {
    // Elisa Open Suomi's Playoffs: upper/lower bracket, 6 matches, none played.
    // (The capture is the embedded form, without `previous_matches`.)
    let serie = fixture("real-nonswiss-serie-filter.json")[0].clone();
    let tournaments = fixture("real-nonswiss-tournaments.json");
    let playoffs = by_id(&tournaments, 22330).clone();
    let t = TournamentInput { matches: array(&playoffs["matches"]), tournament: playoffs };
    let s = parse::snapshot(&serie, &[t]).unwrap();

    assert_eq!(s.event.name, "Elisa Open Suomi Season 14 2026");
    assert_eq!(s.stages[0].kind, StageKind::Playoffs);
    assert_eq!(s.matches.len(), 6);
    assert!(s.matches.iter().all(|m| m.placement == MatchPlacement::Unplaced));
    assert!(s.matches.iter().all(|m| m.status == MatchStatus::Upcoming && m.format == MatchFormat::Bo3));
}

// The Event lists and name matching.

#[test]
fn an_event_list_is_read_from_series_objects_and_names_include_the_league() {
    let events = parse::events(&fixture("real-series-running.json")).unwrap();

    let named: Vec<_> = events.iter().map(|e| (e.external_id.as_str(), e.name.as_str())).collect();
    assert_eq!(
        named,
        vec![
            ("10918", "DACH CS Masters Season 6 2026"),
            ("11070", "ESEA Advanced Europe season 59 2026"),
            ("11163", "NODWIN Clutch Series Season 14 2026"),
            ("11156", "Elisa Open Suomi Season 14 2026"),
            ("11177", "European Pro League Season 10 2026"),
        ]
    );
    assert!(events.iter().all(|e| e.canonical.is_none() && e.starts_at.is_some()));
    assert_eq!(parse::events(&fixture("real-series-upcoming.json")).unwrap().len(), 3);
    assert_eq!(parse::events(&fixture("real-series-range.json")).unwrap().len(), 5);
}

#[test]
fn the_chain_picks_the_major_among_all_majors_by_hltv_name_and_dates() {
    let candidates = parse::events(&fixture("real-series-search.json")).unwrap();
    let hint = EventHint {
        id: EventId(7148),
        name: "IEM Cologne Major 2026".into(),
        starts_at: Some(Utc.with_ymd_and_hms(2026, 6, 2, 0, 0, 0).unwrap()),
        ends_at: Some(Utc.with_ymd_and_hms(2026, 6, 21, 0, 0, 0).unwrap()),
    };

    let (id, _) = choose_event(&hint, &candidates).expect("a match");
    assert_eq!(id, "10488");
}

#[test]
fn the_chain_picks_an_event_whose_serie_name_is_only_a_season_by_its_league() {
    let candidates = parse::events(&fixture("real-series-running.json")).unwrap();
    let hint = EventHint {
        id: EventId(1),
        name: "Elisa Open Suomi Season 14 2026".into(),
        starts_at: Some(Utc.with_ymd_and_hms(2026, 11, 2, 0, 0, 0).unwrap()),
        ends_at: Some(Utc.with_ymd_and_hms(2026, 11, 29, 0, 0, 0).unwrap()),
    };

    // Several running series are "Season N 2026": only the league tells them apart.
    let (id, _) = choose_event(&hint, &candidates).expect("a match");
    assert_eq!(id, "11156");
}

// What no real capture holds: constructed statuses.

fn constructed() -> (ExternalSnapshot, Vec<Value>) {
    let doc = fixture("constructed-statuses.json");
    let matches = array(&doc["matches"]);
    let t = TournamentInput { tournament: doc["tournament"].clone(), matches: matches.clone() };
    (parse::snapshot(&doc["serie"], &[t]).unwrap(), matches)
}

fn constructed_match(id: &str) -> ExternalMatch {
    constructed().0.matches.into_iter().find(|m| m.external_id == id).unwrap()
}

#[test]
fn a_stage_without_a_bracket_that_is_not_swiss_is_a_groups_stage() {
    let (s, _) = constructed();
    assert_eq!(s.stages[0].kind, StageKind::Groups);
    assert!(s.matches.iter().all(|m| m.placement == MatchPlacement::Unplaced));
}

#[test]
fn a_forfeit_is_a_finished_match_won_by_the_other_side() {
    let m = constructed_match("106");
    assert_eq!(m.status, MatchStatus::Finished);
    assert_eq!(m.winner, Some(Side::A));
}

#[test]
fn a_running_match_is_live_with_its_actual_start() {
    let m = constructed_match("108");
    assert_eq!(m.status, MatchStatus::Live);
    assert_eq!(m.winner, None);
    assert_eq!(m.actual_start, Some(Utc.with_ymd_and_hms(2026, 12, 2, 13, 7, 0).unwrap()));
}

#[test]
fn a_canceled_match_without_forfeit_is_cancelled_with_no_winner_or_start() {
    let m = constructed_match("112");
    assert_eq!(m.status, MatchStatus::Cancelled);
    assert_eq!(m.winner, None);
    assert_eq!(m.actual_start, None);
}

#[test]
fn a_postponed_match_is_upcoming_with_no_scheduled_start() {
    // PandaScore leaves `scheduled_at` stale on `postponed`: the time is unknown.
    let m = constructed_match("109");
    assert_eq!(m.status, MatchStatus::Upcoming);
    assert_eq!(m.scheduled_start, None);
}

#[test]
fn a_match_with_one_opponent_still_tbd_has_one_team() {
    let m = constructed_match("111");
    assert!(m.team_a.is_some() && m.team_b.is_none());
    assert_eq!(m.status, MatchStatus::Upcoming);
}

// The documentation example: a TBD-versus-TBD bracket match (real shape).

fn cct_playoffs_only() -> ExternalSnapshot {
    let m = doc_example("doc-example-csgo_matches-1-0.json");
    let serie = m["serie"].clone();
    let t = TournamentInput { tournament: m["tournament"].clone(), matches: vec![m] };
    parse::snapshot(&serie, &[t]).unwrap()
}

#[test]
fn a_tbd_versus_tbd_upcoming_match_parses_without_teams() {
    let s = cct_playoffs_only();
    let m = &s.matches[0];

    assert_eq!(s.stages.len(), 1);
    assert_eq!(s.stages[0].kind, StageKind::Playoffs);
    assert_eq!(m.external_id, "1246497");
    assert!(m.team_a.is_none() && m.team_b.is_none());
    assert_eq!(m.status, MatchStatus::Upcoming);
    assert_eq!(m.format, MatchFormat::Bo3);
    assert_eq!(m.scheduled_start, Some(Utc.with_ymd_and_hms(2025, 10, 1, 17, 30, 0).unwrap()));
    assert_eq!(m.score, None);
    // A lone bracket match with no `previous_matches` has no known slot.
    assert_eq!(m.placement, MatchPlacement::Unplaced);
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
