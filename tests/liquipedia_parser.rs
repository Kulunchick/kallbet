//! Seam 2: the Liquipedia parser on saved MediaWiki API responses and page
//! wikitext (`tests/fixtures/liquipedia/`). No network.

use chrono::{DateTime, TimeZone, Utc};
use kallbet::domain::{
    MatchFormat, MatchId, MatchPlacement, MatchScore, MatchStatus, StageKind, StageOrdinal,
};
use kallbet::ports::SourceError;
use kallbet::sources::fallback::{ExternalMatch, ExternalSnapshot, Side};
use kallbet::sources::liquipedia::parse::{self, Page, TeamNames};

fn fixture(name: &str) -> String {
    let path = format!("{}/tests/fixtures/liquipedia/{name}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// A page of the real `action=query` answer for the Cologne Major (root
/// page, Stage 1 and Playoffs).
fn cologne_page(title: &str) -> Page {
    parse::query_pages(&fixture("api-query-cologne-pages.json"))
        .unwrap()
        .into_iter()
        .find(|p| p.title == title)
        .unwrap_or_else(|| panic!("no page {title}"))
}

fn page(title: &str, file: &str) -> Page {
    Page {
        title: title.to_string(),
        wikitext: fixture(file),
    }
}

fn at(y: i32, mo: u32, d: u32, h: u32, mi: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(y, mo, d, h, mi, 0).unwrap()
}

/// After every fixture match (the pages were saved in November 2026).
fn later() -> DateTime<Utc> {
    at(2026, 12, 1, 0, 0)
}

const COLOGNE: &str = "Intel Extreme Masters/2026/Cologne";

/// The Cologne Major as the fetch layer reads it: root page, Stage 1 (the
/// other Stages are left out), Playoffs.
fn cologne() -> ExternalSnapshot {
    let pages = [
        cologne_page(COLOGNE),
        cologne_page(&format!("{COLOGNE}/Stage 1")),
        cologne_page(&format!("{COLOGNE}/Playoffs")),
    ];
    parse::parse_event(COLOGNE, &pages)
        .unwrap()
        .into_snapshot(&names(), later())
}

fn names() -> TeamNames {
    parse::team_names(&fixture("api-teamnames-cologne.json")).unwrap()
}

fn by_hltv(snapshot: &ExternalSnapshot, id: i64) -> &ExternalMatch {
    snapshot
        .matches
        .iter()
        .find(|m| m.canonical == Some(MatchId(id)))
        .unwrap_or_else(|| panic!("no match with hltv id {id}"))
}

fn team(m: &ExternalMatch, side: Side) -> &str {
    let t = match side {
        Side::A => &m.team_a,
        Side::B => &m.team_b,
    };
    &t.as_ref().unwrap().name
}

#[test]
fn a_major_has_the_event_of_its_infobox_and_a_swiss_stage_then_playoffs() {
    let major = cologne();

    assert_eq!(major.event.external_id, COLOGNE);
    assert_eq!(major.event.name, "IEM Cologne Major 2026");
    assert_eq!(major.event.starts_at, Some(at(2026, 6, 2, 0, 0)));
    assert_eq!(major.event.ends_at, Some(Utc.with_ymd_and_hms(2026, 6, 21, 23, 59, 59).unwrap()));
    assert_eq!(major.event.canonical, None);

    let stages: Vec<_> = major
        .stages
        .iter()
        .map(|s| (s.ordinal.0, s.kind, s.name.as_str()))
        .collect();
    assert_eq!(
        stages,
        vec![(1, StageKind::Swiss, "Stage 1"), (2, StageKind::Playoffs, "Playoffs")]
    );
    // 8 + 8 + 8 + 6 + 3 Swiss Matches, 4 + 2 + 1 bracket Matches. The
    // showmatch on the main page is not part of any Stage.
    let counts: Vec<usize> = [1, 2]
        .into_iter()
        .map(|o| major.matches.iter().filter(|m| m.stage == StageOrdinal(o)).count())
        .collect();
    assert_eq!(counts, vec![33, 7]);
}

#[test]
fn a_swiss_bo1_carries_its_hltv_id_start_in_utc_names_and_winner() {
    let major = cologne();

    // M80 - Lynn Vision Gaming, round 1, 12:30 CEST. One map, 5+8 against 4+4.
    let m = by_hltv(&major, 2394777);
    assert_eq!(team(m, Side::A), "M80");
    assert_eq!(team(m, Side::B), "Lynn Vision Gaming");
    assert_eq!(m.team_a.as_ref().unwrap().external_id, "m80");
    assert_eq!(m.placement, MatchPlacement::Swiss { round: 1, wins: 0, losses: 0 });
    assert_eq!(m.format, MatchFormat::Bo1);
    assert_eq!(m.scheduled_start, Some(at(2026, 6, 2, 10, 30)));
    assert_eq!(m.status, MatchStatus::Finished);
    assert_eq!(m.winner, Some(Side::A));
    assert_eq!(m.score, Some(MatchScore { team_a: 1, team_b: 0 }));
}

#[test]
fn swiss_record_buckets_follow_the_high_mid_low_lists_of_each_round() {
    let major = cologne();
    let bucket = |id| match by_hltv(&major, id).placement {
        MatchPlacement::Swiss { round, wins, losses } => (round, wins, losses),
        other => panic!("{other:?}"),
    };

    // Round 2: B8 and Thunderdown both won round 1 (the High list).
    assert_eq!(bucket(2394840), (2, 1, 0));
    let rounds: Vec<(u8, u8, u8)> = {
        let mut all: Vec<_> = major
            .matches
            .iter()
            .filter_map(|m| match m.placement {
                MatchPlacement::Swiss { round, wins, losses } => Some((round, wins, losses)),
                _ => None,
            })
            .collect();
        all.sort();
        all.dedup();
        all
    };
    assert_eq!(
        rounds,
        vec![
            (1, 0, 0),
            (2, 0, 1),
            (2, 1, 0),
            (3, 0, 2),
            (3, 1, 1),
            (3, 2, 0),
            (4, 1, 2),
            (4, 2, 1),
            (5, 2, 2),
        ]
    );
}

#[test]
fn a_bo3_winner_comes_from_the_maps_with_overtime_counted() {
    let major = cologne();

    // Quarterfinal Spirit - G2: map 1 9-13 (G2); map 2 12-12 and overtime
    // 4-2 (Spirit, 16-14); map 3 12-12 and four overtimes 3-3, 3-3, 3-3,
    // 4-1 (Spirit, 25-22).
    let m = by_hltv(&major, 2394998);
    assert_eq!(m.format, MatchFormat::Bo3);
    assert_eq!(m.placement, MatchPlacement::Playoffs { rounds_before_final: 2, slot: 0 });
    assert_eq!(m.scheduled_start, Some(at(2026, 6, 19, 13, 45)));
    assert_eq!(m.winner, Some(Side::A));
    assert_eq!(m.score, Some(MatchScore { team_a: 2, team_b: 1 }));
}

#[test]
fn a_skipped_third_map_leaves_a_two_nil_and_the_grand_final_is_a_bo5() {
    let major = cologne();

    // Aurora - BetBoom 2-0 (third map `finished=skip`).
    let qf = by_hltv(&major, 2394996);
    assert_eq!(qf.format, MatchFormat::Bo3);
    assert_eq!(qf.score, Some(MatchScore { team_a: 2, team_b: 0 }));

    // Falcons - FURIA, five map slots, three played.
    let final_ = by_hltv(&major, 2395002);
    assert_eq!(final_.format, MatchFormat::Bo5);
    assert_eq!(
        final_.placement,
        MatchPlacement::Playoffs { rounds_before_final: 0, slot: 0 }
    );
    assert_eq!(final_.score, Some(MatchScore { team_a: 3, team_b: 0 }));
    assert_eq!(final_.winner, Some(Side::A));
}

#[test]
fn an_unplayed_match_is_upcoming_without_a_winner_until_its_start_has_passed() {
    // The Singapore Stage 1 page under a root page that only supplies the
    // infobox.
    let pages = [
        cologne_page(COLOGNE),
        page(&format!("{COLOGNE}/Stage 1"), "singapore-stage1-upcoming.wikitext"),
    ];
    let parsed = parse::parse_event(COLOGNE, &pages).unwrap();
    let before = at(2026, 11, 24, 3, 0);
    let snapshot = parsed.clone().into_snapshot(&TeamNames::new(), before);

    // Round 1 is paired (8 Matches); rounds 2 to 5 are empty slots and are
    // left out.
    assert_eq!(snapshot.matches.len(), 8);
    let m = by_hltv(&snapshot, 2400494);
    assert_eq!(team(m, Side::A), "heroic"); // no name known: the key
    assert_eq!(m.scheduled_start, Some(at(2026, 11, 24, 4, 0))); // 12:00 SGT
    assert_eq!(m.status, MatchStatus::Upcoming);
    assert_eq!(m.winner, None);
    assert_eq!(m.score, None);

    // Past its start and not finished: Live.
    let live = parsed.into_snapshot(&TeamNames::new(), at(2026, 11, 24, 4, 30));
    assert_eq!(by_hltv(&live, 2400494).status, MatchStatus::Live);
}

#[test]
fn a_non_swiss_event_is_one_bracket_stage_with_an_unplayed_final() {
    let root = "Dust2.dk/Ligaen/Season 33";
    let pages = [page(root, "ligaen-season33.wikitext")];
    let snapshot = parse::parse_event(root, &pages)
        .unwrap()
        .into_snapshot(&TeamNames::new(), at(2026, 11, 21, 12, 0));

    assert_eq!(snapshot.event.name, "Dust2.dk Ligaen Season 33");
    // A double elimination has no single-elimination tree: a Groups Stage.
    assert_eq!(snapshot.stages.len(), 1);
    assert_eq!(snapshot.stages[0].kind, StageKind::Groups);
    assert_eq!(snapshot.stages[0].name, "Bracket");
    assert_eq!(snapshot.matches.len(), 6);
    assert!(snapshot.matches.iter().all(|m| m.placement == MatchPlacement::Unplaced));

    // 18:00 CET = 17:00 UTC; WOPA won 2-1 on maps.
    let sf = by_hltv(&snapshot, 2400551);
    assert_eq!(sf.scheduled_start, Some(at(2026, 11, 20, 17, 0)));
    assert_eq!(sf.winner, Some(Side::A));
    assert_eq!(sf.score, Some(MatchScore { team_a: 2, team_b: 1 }));

    // The Grand Final: not finished, start 19:00 CET, still ahead at noon.
    let gf = by_hltv(&snapshot, 2400556);
    assert_eq!(gf.status, MatchStatus::Upcoming);
    assert_eq!(gf.winner, None);
    assert_eq!(gf.format, MatchFormat::Bo3);
}

#[test]
fn one_page_can_hold_a_swiss_stage_and_a_single_elimination_bracket() {
    let root = "CCT/2026/Europe/Series 14";
    let pages = [page(root, "cct-europe-series14.wikitext")];
    let snapshot = parse::parse_event(root, &pages)
        .unwrap()
        .into_snapshot(&TeamNames::new(), later());

    let stages: Vec<_> = snapshot.stages.iter().map(|s| (s.kind, s.name.as_str())).collect();
    assert_eq!(
        stages,
        vec![(StageKind::Swiss, "Group Stage"), (StageKind::Playoffs, "Playoffs")]
    );
    // Bracket/16 has 15 slots. Semifinal 2 has neither teams nor an HLTV id
    // yet, so it is left out; the final (teams unknown, HLTV id 2400280) is
    // kept, as the single Match of round 4.
    let playoffs: Vec<_> = snapshot.matches.iter().filter(|m| m.stage == StageOrdinal(2)).collect();
    assert_eq!(playoffs.len(), 14);
    let final_ = by_hltv(&snapshot, 2400280);
    assert_eq!(final_.placement, MatchPlacement::Playoffs { rounds_before_final: 0, slot: 0 });
    assert_eq!(final_.team_a, None);
    assert_eq!(final_.scheduled_start, Some(at(2026, 11, 22, 17, 0)));
}

#[test]
fn a_forfeit_is_a_win_for_the_side_marked_w_without_a_score() {
    let root = "CCT/2026/Europe/Series 14";
    let pages = [page(root, "cct-europe-series14.wikitext")];
    let snapshot = parse::parse_event(root, &pages)
        .unwrap()
        .into_snapshot(&TeamNames::new(), later());

    // `leo team` (W) against `mint` (FF).
    let m = by_hltv(&snapshot, 2400236);
    assert_eq!(m.status, MatchStatus::Finished);
    assert_eq!(m.winner, Some(Side::A));
    assert_eq!(m.score, None);
    assert_eq!(team(m, Side::B), "mint");
}

#[test]
fn team_names_come_from_the_expanded_templates() {
    let names = names();

    assert_eq!(names["m80"], "M80");
    assert_eq!(names["lvg"], "Lynn Vision Gaming");
    assert_eq!(names["navi"], "Natus Vincere");
}

#[test]
fn an_unreadable_date_zone_is_a_parse_error_not_a_guess() {
    let mut p = cologne_page(COLOGNE);
    let stage = cologne_page(&format!("{COLOGNE}/Playoffs"));
    let broken = Page {
        wikitext: stage.wikitext.replace("{{Abbr/CEST}}", "{{Abbr/CST}}"),
        ..stage
    };
    p.title = COLOGNE.to_string();

    let result = parse::parse_event(COLOGNE, &[p, broken]);

    assert!(matches!(result, Err(SourceError::Parse(m)) if m.contains("CST")));
}

#[test]
fn a_swiss_round_with_an_unexpected_number_of_lists_is_a_parse_error() {
    let stage = cologne_page(&format!("{COLOGNE}/Stage 1"));
    // Round 3's three lists lose their headings, so round 2's absorb them.
    let broken = Page {
        wikitext: stage.wikitext.replace("====Round 3====", "====Interlude===="),
        ..stage
    };
    let root = cologne_page(COLOGNE);

    let result = parse::parse_event(COLOGNE, &[root, broken]);

    assert!(matches!(result, Err(SourceError::Parse(_))));
}

#[test]
fn a_page_without_any_stage_is_a_parse_error_never_empty_data() {
    let root = cologne_page(COLOGNE);

    let result = parse::parse_event(COLOGNE, &[root]);

    assert!(matches!(result, Err(SourceError::Parse(_))));
}

#[test]
fn stage_subpages_are_listed_in_play_order_without_other_subpages() {
    let listing = fixture("api-allpages-cologne.json");
    let mut titles = parse::listed_titles(&listing).unwrap();
    titles.push(format!("{COLOGNE}/Closed Qualifier"));
    titles.push(format!("{COLOGNE}/Stage 1/Statistics"));

    let subpages = parse::stage_subpages(&titles, COLOGNE);

    assert_eq!(
        subpages,
        vec![
            format!("{COLOGNE}/Stage 1"),
            format!("{COLOGNE}/Stage 2"),
            format!("{COLOGNE}/Stage 3"),
            format!("{COLOGNE}/Playoffs"),
        ]
    );
}

#[test]
fn search_candidates_keep_only_pages_with_a_league_infobox() {
    let titles = parse::listed_titles(&fixture("api-search-iem-cologne.json")).unwrap();
    let pages = parse::query_pages(&fixture("api-section0-search-candidates.json")).unwrap();

    let events: Vec<_> = pages
        .iter()
        .filter_map(|p| parse::event_header(&p.title, &p.wikitext))
        .collect();

    assert!(titles.contains(&COLOGNE.to_string()));
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].external_id, COLOGNE);
    assert_eq!(events[0].name, "IEM Cologne Major 2026");
}

#[test]
fn a_missing_page_is_left_out_and_an_api_error_is_not_data() {
    assert!(parse::query_pages(&fixture("api-missing-page.json")).unwrap().is_empty());

    let limited = r#"{"error":{"code":"ratelimited","info":"slow down"}}"#;
    assert!(matches!(parse::query_pages(limited), Err(SourceError::Unavailable(_))));
    assert!(matches!(parse::query_pages("<html>"), Err(SourceError::Parse(_))));
}
