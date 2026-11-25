use kallbet_ai::adapters::context::liquipedia::{find_team_roster, parse_revision_rosters};
use kallbet_ai::adapters::context::vrs_diff::VrsRosterIndex;

const SINGAPORE_REV: &str =
    include_str!("fixtures/context/liquipedia/pgl-singapore-2026-rev-before-start.json");
const VRS_OCT: &str = include_str!("fixtures/vrs/standings_global_2026_10_05.md");
const VRS_NOV: &str = include_str!("fixtures/vrs/standings_global_2026_11_02.md");

#[test]
fn vrs_roster_diff_flags_substitutions_between_vrs_baseline_and_active_event_roster() {
    let cutoff: kallbet_ai::Instant = "2026-11-24T00:00:00Z".parse().unwrap();
    let vrs = VrsRosterIndex::parse([VRS_OCT, VRS_NOV]).unwrap();
    let event = parse_revision_rosters(SINGAPORE_REV, cutoff)
        .unwrap()
        .unwrap();

    // Vitality: VRS 2026-11-02 had jL; Singapore active lineup has apEX standing in.
    let vitality = find_team_roster(event.at(cutoff).unwrap(), "Vitality").unwrap();
    let diff = vrs
        .diff_team(&event, vitality, "team_a", cutoff)
        .unwrap()
        .expect("Vitality exists in VRS 2026-11-02");
    let fact = diff.at(cutoff).unwrap();
    assert!(fact.changed);
    assert_eq!(fact.added_players, ["apEX"]);
    assert_eq!(fact.removed_players, ["jL"]);
    assert_eq!(fact.match_side, "team_a");
    assert_eq!(
        diff.observed_at(),
        "2026-11-23T19:49:30Z".parse::<kallbet_ai::Instant>().unwrap()
    );

    // FUT Esports: cmtry -> SunPayus
    let fut = find_team_roster(event.at(cutoff).unwrap(), "FUT").unwrap();
    let fut_diff = vrs.diff_team(&event, fut, "team_b", cutoff).unwrap().unwrap();
    let fut_fact = fut_diff.at(cutoff).unwrap();
    assert!(fut_fact.changed);
    assert_eq!(fut_fact.added_players, ["SunPayus"]);
    assert_eq!(fut_fact.removed_players, ["cmtry"]);

    // Astralis: jabbi -> Anlelele
    let astralis = find_team_roster(event.at(cutoff).unwrap(), "Astralis").unwrap();
    let ast_diff = vrs
        .diff_team(&event, astralis, "team_a", cutoff)
        .unwrap()
        .unwrap();
    assert!(ast_diff.at(cutoff).unwrap().changed);
    assert_eq!(ast_diff.at(cutoff).unwrap().added_players, ["Anlelele"]);
    assert_eq!(ast_diff.at(cutoff).unwrap().removed_players, ["jabbi"]);

    // 9z Team: meyern -> history
    let ninez = find_team_roster(event.at(cutoff).unwrap(), "9z").unwrap();
    let ninez_diff = vrs
        .diff_team(&event, ninez, "team_a", cutoff)
        .unwrap()
        .unwrap();
    assert!(ninez_diff.at(cutoff).unwrap().changed);
    assert_eq!(ninez_diff.at(cutoff).unwrap().added_players, ["history"]);
    assert_eq!(ninez_diff.at(cutoff).unwrap().removed_players, ["meyern"]);

    // MOUZ: unchanged roster (PR, Spinx, torzsi, xelex, xertioN).
    let mouz = find_team_roster(event.at(cutoff).unwrap(), "MOUZ").unwrap();
    let mouz_diff = vrs
        .diff_team(&event, mouz, "team_a", cutoff)
        .unwrap()
        .unwrap();
    let mouz_fact = mouz_diff.at(cutoff).unwrap();
    assert!(!mouz_fact.changed);
    assert!(mouz_fact.added_players.is_empty());
    assert!(mouz_fact.removed_players.is_empty());

    // Renamed team: GOSH (Liquipedia) matches BC.Game (VRS) via Liquipedia rename note alias.
    let gosh = find_team_roster(event.at(cutoff).unwrap(), "GOSH").unwrap();
    let gosh_diff = vrs
        .diff_team(&event, gosh, "team_b", cutoff)
        .unwrap()
        .unwrap();
    assert!(!gosh_diff.at(cutoff).unwrap().changed);
}

#[test]
fn vrs_roster_index_enforces_strict_cutoff_selection() {
    let vrs = VrsRosterIndex::parse([VRS_OCT, VRS_NOV]).unwrap();
    // 2026-11-02 snapshot is dated at 2026-11-02T23:59:59.999999999Z, so a cutoff on
    // 2026-11-02T12:00:00Z must fall back to the 2026-10-05 snapshot.
    let mid_nov2: kallbet_ai::Instant = "2026-11-02T12:00:00Z".parse().unwrap();
    let roster_oct = vrs
        .roster_for("MOUZ", &[], mid_nov2)
        .unwrap()
        .expect("October snapshot is strictly before Nov 2 noon");
    assert_eq!(
        roster_oct.observed_at(),
        "2026-10-05T23:59:59.999999999Z"
            .parse::<kallbet_ai::Instant>()
            .unwrap()
    );

    // Before October snapshot: strictly excluded -> None.
    let too_early: kallbet_ai::Instant = "2026-10-05T23:59:59.999999999Z".parse().unwrap();
    assert!(vrs.roster_for("MOUZ", &[], too_early).unwrap().is_none());
}
