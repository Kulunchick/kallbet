//! Seam 2: the HLTV parser on saved pages (`tests/fixtures/hltv/`). No network.

use chrono::{DateTime, TimeZone, Utc};
use kallbet::domain::{
    DataSourceKind, Event, EventId, EventSnapshot, MatchFormat, MatchId, MatchPlacement,
    MatchScore, MatchStatus, StageKind, StageOrdinal, TeamId,
};
use kallbet::ports::SourceError;
use kallbet::sources::hltv::parse;

fn fixture(name: &str) -> String {
    let path = format!("{}/tests/fixtures/hltv/{name}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

fn ms(millis: i64) -> DateTime<Utc> {
    Utc.timestamp_millis_opt(millis).unwrap()
}

fn page(name: &str) -> parse::EventPage {
    parse::event_page(&fixture(name)).unwrap()
}

/// IEM Cologne Major 2026 as HLTV splits it: the parent event (Stage 3 and
/// the Playoffs) plus one event per earlier Stage.
fn cologne_major() -> EventSnapshot {
    let parent = page("event-8301-cologne-major-parent.html");
    let stages = vec![
        page("event-9028-cologne-stage1.html"),
        page("event-9029-cologne-stage2.html"),
    ];
    parse::snapshot(&parent, &stages, &[], ms(1795000000000)).unwrap()
}

fn team_name(snapshot: &EventSnapshot, id: Option<TeamId>) -> &str {
    &snapshot.team(id.unwrap()).unwrap().name
}

#[test]
fn a_major_is_one_event_whose_stages_are_hltv_stage_events() {
    let major = cologne_major();

    assert_eq!(major.event.id, EventId(8301));
    assert_eq!(major.event.name, "IEM Cologne Major 2026");
    // From the start of Stage 1 to the end of the Playoffs.
    assert_eq!(major.event.starts_at, Some(ms(1780394400000)));
    assert_eq!(major.event.ends_at, Some(ms(1782036000000)));
    assert_eq!(major.source, DataSourceKind::Hltv);
    assert_eq!(major.fetched_at, ms(1795000000000));

    let stages: Vec<(u8, StageKind, &str)> = major
        .stages
        .iter()
        .map(|s| (s.ordinal.0, s.kind, s.name.as_str()))
        .collect();
    assert_eq!(
        stages,
        vec![
            (1, StageKind::Swiss, "Stage 1"),
            (2, StageKind::Swiss, "Stage 2"),
            (3, StageKind::Swiss, "Stage 3"),
            (4, StageKind::Playoffs, "Playoffs"),
        ]
    );
    // Every Swiss Stage has 33 Matches (5 rounds), the Playoffs 7.
    let counts: Vec<usize> = (1..=4)
        .map(|o| major.matches_in(StageOrdinal(o)).count())
        .collect();
    assert_eq!(counts, vec![33, 33, 33, 7]);
}

#[test]
fn a_finished_swiss_bo1_has_its_record_bucket_winner_and_one_map_each() {
    let major = cologne_major();

    let m = major.match_by_id(MatchId(2394846)).unwrap();
    assert_eq!(m.stage, StageOrdinal(1));
    assert_eq!(
        m.placement,
        MatchPlacement::Swiss { round: 2, wins: 1, losses: 0 }
    );
    assert_eq!(m.format, MatchFormat::Bo1);
    assert_eq!(m.status, MatchStatus::Finished);
    assert_eq!(m.scheduled_start, Some(ms(1780428600000)));
    assert_eq!(team_name(&major, m.winner), "BETBOOM");
    assert_eq!(m.winner, Some(TeamId(12394)));
    let mut teams = [team_name(&major, m.team_a), team_name(&major, m.team_b)];
    teams.sort();
    assert_eq!(teams, ["BETBOOM", "Liquid"]);
    // A Bo1 is scored in maps (1-0), not in rounds (13-9).
    let winner_maps = if m.team_a == m.winner { m.score.unwrap().team_a } else { m.score.unwrap().team_b };
    assert_eq!(winner_maps, 1);
    assert_eq!(m.score.unwrap().team_a + m.score.unwrap().team_b, 1);
}

#[test]
fn a_finished_swiss_bo3_on_the_parent_page_belongs_to_stage_3() {
    let major = cologne_major();

    let m = major.match_by_id(MatchId(2394903)).unwrap();
    assert_eq!(m.stage, StageOrdinal(3));
    assert_eq!(
        m.placement,
        MatchPlacement::Swiss { round: 1, wins: 0, losses: 0 }
    );
    assert_eq!(m.format, MatchFormat::Bo3);
    assert_eq!(m.status, MatchStatus::Finished);
    assert_eq!(team_name(&major, m.team_a), "PARIVISION");
    assert_eq!(team_name(&major, m.team_b), "9z");
    assert_eq!(m.winner, m.team_b);
    assert_eq!(m.score, Some(MatchScore { team_a: 1, team_b: 2 }));
}

#[test]
fn the_playoffs_are_a_bracket_ending_in_a_bo5_grand_final() {
    let major = cologne_major();

    let final_ = major.match_by_id(MatchId(2395002)).unwrap();
    assert_eq!(final_.stage, StageOrdinal(4));
    assert_eq!(
        final_.placement,
        MatchPlacement::Playoffs { rounds_before_final: 0, slot: 0 }
    );
    assert_eq!(final_.format, MatchFormat::Bo5);
    assert_eq!(final_.status, MatchStatus::Finished);
    assert_eq!(final_.scheduled_start, Some(ms(1782054000000)));
    assert_eq!(final_.team_a, Some(TeamId(8297)));
    assert_eq!(team_name(&major, final_.team_a), "FURIA");
    assert_eq!(team_name(&major, final_.team_b), "Falcons");
    assert_eq!(final_.score, Some(MatchScore { team_a: 0, team_b: 3 }));
    assert_eq!(final_.winner, final_.team_b);

    let quarter = major.match_by_id(MatchId(2394999)).unwrap();
    assert_eq!(
        quarter.placement,
        MatchPlacement::Playoffs { rounds_before_final: 2, slot: 3 }
    );
    assert_eq!(quarter.format, MatchFormat::Bo3);
}

#[test]
fn every_team_in_a_match_is_listed_once_with_its_hltv_name_and_logo() {
    let major = cologne_major();

    for m in &major.matches {
        for team in m.teams().into_iter().flatten() {
            assert!(major.team(team).is_some(), "team {team} of match {} missing", m.id);
        }
    }
    let mut ids: Vec<TeamId> = major.teams.iter().map(|t| t.id).collect();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), major.teams.len());

    let mongolz = major.team_named("The MongolZ").unwrap();
    let logo = mongolz.logo_url.as_deref().unwrap();
    assert!(logo.starts_with("https://img-cdn.hltv.org/teamlogo/"), "{logo}");
}

#[test]
fn a_pasted_stage_event_leads_to_its_major() {
    let stage1 = page("event-9303-stage1-swiss.html");
    assert_eq!(stage1.parent().map(|p| p.id), Some(EventId(8302)));

    // A snapshot is always of the Major, never of one of its Stage events.
    let err = parse::snapshot(&stage1, &[], &[], ms(0)).unwrap_err();
    assert!(matches!(err, SourceError::Parse(_)), "{err:?}");

    let major = page("event-8302-major-singapore.html");
    assert_eq!(major.parent(), None);
    let stage_ids: Vec<EventId> = major.stage_events().iter().map(|l| l.id).collect();
    assert_eq!(stage_ids, vec![EventId(9303), EventId(9304)]);
}

#[test]
fn a_major_snapshot_needs_the_pages_of_all_its_stages() {
    let major = page("event-8302-major-singapore.html");
    let stage1 = page("event-9303-stage1-swiss.html");

    let err = parse::snapshot(&major, &[stage1], &[], ms(0)).unwrap_err();
    assert!(matches!(err, SourceError::Parse(_)), "{err:?}");
}

#[test]
fn an_upcoming_swiss_round_has_its_pairings_scheduled_starts_and_bo1_format() {
    let stage1 = page("event-9303-stage1-swiss.html");
    assert!(!stage1.ended);
    let [swiss] = stage1.stages.as_slice() else { panic!("{:?}", stage1.stages) };
    assert_eq!(swiss.kind, StageKind::Swiss);

    // Only round 1 is paired; later rounds have no Matches yet.
    assert_eq!(swiss.matches.len(), 8);
    let m = swiss.matches.iter().find(|m| m.id == MatchId(2400494)).unwrap();
    assert_eq!(
        m.placement,
        MatchPlacement::Swiss { round: 1, wins: 0, losses: 0 }
    );
    assert_eq!(m.status, MatchStatus::Upcoming);
    assert_eq!(m.format, MatchFormat::Bo1);
    assert_eq!(m.scheduled_start, Some(ms(1795492800000)));
    assert_eq!((m.winner, m.score), (None, None));
    let name = |id: Option<TeamId>| {
        stage1.teams.iter().find(|t| Some(t.id) == id).unwrap().name.clone()
    };
    assert_eq!((name(m.team_a), name(m.team_b)), ("HEROIC".into(), "Imperial".into()));
}

#[test]
fn an_event_without_a_swiss_stage_has_its_double_elimination_bracket_as_a_group_stage() {
    let event = page("event-9496-double-elim-nonswiss.html");
    let live = parse::live_matches(&fixture("matches.html")).unwrap();

    let snapshot = parse::snapshot(&event, &[], &live, ms(0)).unwrap();

    assert_eq!(snapshot.event.id, EventId(9496));
    assert_eq!(snapshot.event.name, "POWER Ligaen Season 33 Finals");
    let [stage] = snapshot.stages.as_slice() else { panic!("{:?}", snapshot.stages) };
    assert_eq!(stage.kind, StageKind::Groups);
    assert!(!snapshot.matches.is_empty());
    assert!(snapshot
        .matches
        .iter()
        .all(|m| m.placement == MatchPlacement::Unplaced));
    // The grand final was being played when the pages were saved.
    let final_ = snapshot.match_by_id(MatchId(2400556)).unwrap();
    assert_eq!(final_.status, MatchStatus::Live);
    assert!(snapshot
        .matches
        .iter()
        .any(|m| m.status == MatchStatus::Finished && m.winner.is_some()));
}

#[test]
fn the_live_list_shows_which_matches_are_being_played() {
    let live = parse::live_matches(&fixture("matches.html")).unwrap();

    assert!(live.contains(&MatchId(2400366)));
    assert!(live.contains(&MatchId(2400556)));
    // Upcoming on the same page.
    assert!(!live.contains(&MatchId(2400494)));
}

#[test]
fn a_swiss_event_with_playoffs_has_live_upcoming_and_tbd_bracket_matches() {
    let event = page("event-9469-swiss-plus-playoffs.html");
    let snapshot = parse::snapshot(&event, &[], &[], ms(0)).unwrap();

    let kinds: Vec<(StageKind, &str)> = snapshot
        .stages
        .iter()
        .map(|s| (s.kind, s.name.as_str()))
        .collect();
    assert_eq!(
        kinds,
        vec![(StageKind::Swiss, "Swiss Stage"), (StageKind::Playoffs, "Playoffs")]
    );

    // Round of 16, second slot: live in the bracket.
    let live = snapshot.match_by_id(MatchId(2400366)).unwrap();
    assert_eq!(live.status, MatchStatus::Live);
    assert_eq!(
        live.placement,
        MatchPlacement::Playoffs { rounds_before_final: 3, slot: 1 }
    );
    assert_eq!(team_name(&snapshot, live.team_a), "INFINITE");
    assert_eq!(team_name(&snapshot, live.team_b), "BAKS");

    let next = snapshot.match_by_id(MatchId(2400367)).unwrap();
    assert_eq!(next.status, MatchStatus::Upcoming);
    assert_eq!(next.scheduled_start, Some(ms(1795285800000)));

    // The grand final exists on HLTV before its teams are known.
    let final_ = snapshot.match_by_id(MatchId(2400374)).unwrap();
    assert_eq!((final_.team_a, final_.team_b), (None, None));
    assert_eq!(final_.status, MatchStatus::Upcoming);
    assert_eq!(final_.format, MatchFormat::Bo3);

    // A renamed team keeps its current HLTV name (rows still show the old one).
    assert!(snapshot.team_named("Hype Tech").is_some());
    assert!(snapshot.team_named("ex-Zero Tenacity").is_none());
}

#[test]
fn a_match_the_live_list_shows_is_live_unless_its_result_is_in() {
    let event = page("event-9469-swiss-plus-playoffs.html");
    // 2400374 is upcoming on the event page, 2400365 finished.
    let live = [MatchId(2400374), MatchId(2400365)];

    let snapshot = parse::snapshot(&event, &[], &live, ms(0)).unwrap();

    assert_eq!(snapshot.match_by_id(MatchId(2400374)).unwrap().status, MatchStatus::Live);
    assert_eq!(snapshot.match_by_id(MatchId(2400365)).unwrap().status, MatchStatus::Finished);
}

#[test]
fn a_cloudflare_challenge_is_an_unavailable_source_not_an_empty_event() {
    let html = fixture("cloudflare-challenge.html");

    assert!(parse::is_challenge_page(&html));
    assert!(matches!(parse::event_page(&html), Err(SourceError::Unavailable(_))));
    assert!(parse::event_list(&html).is_err());
    assert!(parse::live_matches(&html).is_err());
}

#[test]
fn an_unknown_event_id_is_not_found() {
    let html = fixture("event-not-found.html");

    assert_eq!(parse::event_page(&html), Err(SourceError::NotFound));
}

#[test]
fn a_page_of_the_wrong_kind_is_a_parse_error() {
    let list = fixture("events-list.html");
    let event = fixture("event-9028-cologne-stage1.html");

    assert!(matches!(parse::event_page(&list), Err(SourceError::Parse(_))));
    assert!(matches!(parse::event_list(&event), Err(SourceError::Parse(_))));
    assert!(matches!(parse::live_matches(&event), Err(SourceError::Parse(_))));
}

#[test]
fn changed_markup_is_a_parse_error_not_missing_matches() {
    let real = fixture("event-9028-cologne-stage1.html");

    // HLTV renames the class of the standings rows.
    let renamed = real.replace("class=\"bottom-row\"", "class=\"history-row\"");
    assert!(matches!(parse::event_page(&renamed), Err(SourceError::Parse(_))));

    // A bracket's JSON is no longer JSON.
    let parent = fixture("event-8301-cologne-major-parent.html");
    let broken = parent.replacen("data-slotted-bracket-json=\"{", "data-slotted-bracket-json=\"{{", 1);
    assert!(matches!(parse::event_page(&broken), Err(SourceError::Parse(_))));

    // A Match status this parser has never seen.
    let event = fixture("event-9469-swiss-plus-playoffs.html");
    let odd = event.replace("&quot;status&quot;:&quot;Live&quot;", "&quot;status&quot;:&quot;Delayed&quot;");
    assert_ne!(odd, event);
    assert!(matches!(parse::event_page(&odd), Err(SourceError::Parse(_))));
}

#[test]
fn the_event_list_has_ongoing_and_big_upcoming_events_with_hltv_ids() {
    let events = parse::event_list(&fixture("events-list.html")).unwrap();

    let major = events.iter().find(|e| e.id == EventId(8302)).unwrap();
    assert_eq!(
        major,
        &Event {
            id: EventId(8302),
            name: "PGL Major Singapore 2026".into(),
            starts_at: Some(ms(1796209200000)),
            ends_at: Some(ms(1797159600000)),
        }
    );
    let ongoing = events.iter().find(|e| e.id == EventId(9469)).unwrap();
    assert_eq!(ongoing.name, "NODWIN Clutch Series 14");

    // Each Event once, although HLTV repeats ongoing ones in two tabs.
    assert_eq!(events.iter().filter(|e| e.id == EventId(9469)).count(), 1);
}

#[test]
fn the_event_list_leaves_out_stages_of_a_listed_major() {
    let events = parse::event_list(&fixture("events-list.html")).unwrap();
    let ids: Vec<EventId> = events.iter().map(|e| e.id).collect();

    // PGL Major Singapore 2026 Stage 1 and 2 belong to the Major (8302).
    assert!(!ids.contains(&EventId(9303)));
    assert!(!ids.contains(&EventId(9304)));
    // IEM Kraków 2027 Stage 1 belongs to IEM Kraków 2027 (8318).
    assert!(!ids.contains(&EventId(9489)));
    assert!(ids.contains(&EventId(8318)));
}
