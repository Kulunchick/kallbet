use chrono::Duration;
use kallbet::domain::{TeamId, pickem::SwissPickem};
use kallbet_ai::{
    AsOf, Format, Instant, PairwiseProbability, PickemSolver,
    adapters::pickem::MonteCarloPickemSolver, vrs::Standings,
};
use serde_json::{Value, json};
use std::collections::HashSet;

const VRS_NOV: &str = include_str!("fixtures/vrs/standings_global_2026_11_02.md");

fn cutoff() -> Instant {
    "2026-11-24T04:00:00Z".parse().unwrap()
}

fn swiss_stage_json(teams: &[(i64, &str)], r1_format: &str, r1_pairs: &[(i64, i64)]) -> Value {
    let team_objs: Vec<Value> = teams
        .iter()
        .map(|(id, name)| json!({ "id": id, "name": name }))
        .collect();
    let name_of = |id: i64| {
        teams
            .iter()
            .find(|(tid, _)| *tid == id)
            .map(|(_, n)| *n)
            .unwrap_or("T")
    };
    let matches: Vec<Value> = r1_pairs
        .iter()
        .enumerate()
        .map(|(idx, (a, b))| {
            json!({
                "id": 100 + idx as i64,
                "team_a": { "id": a, "name": name_of(*a) },
                "team_b": { "id": b, "name": name_of(*b) },
                "format": r1_format,
                "scheduled_start": "2026-11-24T04:00:00Z",
                "actual_start": null,
                "status": "upcoming",
                "winner": null
            })
        })
        .collect();
    json!({
        "stage": { "ordinal": 1, "kind": "swiss", "name": "Stage 1" },
        "teams": team_objs,
        "groups": [{
            "kind": "record",
            "wins": 0,
            "losses": 0,
            "matches": matches
        }]
    })
}

#[test]
fn swiss_solver_produces_deterministic_valid_disjoint_pickem_from_real_vrs_and_waits_when_incomplete()
 {
    let cut = cutoff();
    let obs = cut - Duration::hours(1);
    let standings = Standings::parse([VRS_NOV]).unwrap();
    let names = [
        "Ninjas in Pyjamas",
        "PARIVISION",
        "HEROIC",
        "FaZe",
        "Liquid",
        "Eternal Fire",
        "100 Thieves",
        "9z",
        "M80",
        "DENDELE",
        "Imperial",
        "Wildcard",
        "BC.Game",
        "FlyQuest",
        "BIG",
        "GamerLegion",
    ];
    let teams: Vec<(i64, &str)> = names
        .iter()
        .enumerate()
        .map(|(i, n)| (i as i64 + 1, *n))
        .collect();
    let r1_pairs: Vec<(i64, i64)> = (1..=8).map(|i| (i, i + 8)).collect();

    let mut ratings = Vec::new();
    for i in 0..teams.len() {
        for j in (i + 1)..teams.len() {
            let sig = standings
                .estimate(teams[i].1, teams[j].1, Format::Bo1, cut)
                .unwrap()
                .unwrap();
            ratings.push(
                AsOf::new(
                    PairwiseProbability {
                        first: teams[i].0,
                        second: teams[j].0,
                        map: sig.at(cut).unwrap().map,
                    },
                    sig.observed_at(),
                    cut,
                )
                .unwrap(),
            );
        }
    }

    // Incomplete Swiss stage (< 16 teams) waits and returns None.
    let incomplete = AsOf::new(
        swiss_stage_json(&teams[..8], "bo1", &[(1, 2), (3, 4), (5, 6), (7, 8)]),
        obs,
        cut,
    )
    .unwrap();
    let solver = MonteCarloPickemSolver::seeded(42);
    assert_eq!(solver.solve(&incomplete, &ratings, cut).unwrap(), None);

    // Full 16-team Stage 1 (Bo1 opening, Bo3 elimination/advancement) produces a valid Pick'em.
    let stage = AsOf::new(swiss_stage_json(&teams, "bo1", &r1_pairs), obs, cut).unwrap();
    let first_run = solver.solve(&stage, &ratings, cut).unwrap().unwrap();
    let second_run = solver.solve(&stage, &ratings, cut).unwrap().unwrap();
    assert_eq!(first_run, second_run, "fixed seed must be deterministic");

    let parse_ids = |key: &str| -> Vec<TeamId> {
        first_run[key]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| TeamId(v.as_i64().unwrap()))
            .collect()
    };
    let pickem = SwissPickem {
        three_nil: parse_ids("three_nil"),
        zero_three: parse_ids("zero_three"),
        advance: parse_ids("advance"),
    };
    let stage_team_ids: Vec<TeamId> = (1..=16).map(TeamId).collect();
    assert_eq!(pickem.validate(&stage_team_ids), Ok(()));

    let unique: HashSet<TeamId> = pickem
        .three_nil
        .iter()
        .chain(&pickem.zero_three)
        .chain(&pickem.advance)
        .copied()
        .collect();
    assert_eq!(unique.len(), 10);

    // All-Bo3 Stage 3 format also produces a valid 10-team Pick'em.
    let stage3 = AsOf::new(swiss_stage_json(&teams, "bo3", &r1_pairs), obs, cut).unwrap();
    let stage3_out = solver.solve(&stage3, &ratings, cut).unwrap().unwrap();
    let stage3_pickem = SwissPickem {
        three_nil: stage3_out["three_nil"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| TeamId(v.as_i64().unwrap()))
            .collect(),
        zero_three: stage3_out["zero_three"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| TeamId(v.as_i64().unwrap()))
            .collect(),
        advance: stage3_out["advance"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| TeamId(v.as_i64().unwrap()))
            .collect(),
    };
    assert_eq!(stage3_pickem.validate(&stage_team_ids), Ok(()));
}

#[test]
fn swiss_solver_places_heavy_favourite_in_advance_when_three_one_or_two_beats_three_nil() {
    let cut = cutoff();
    let obs = cut - Duration::hours(1);
    let teams: Vec<(i64, String)> = (1..=16).map(|i| (i, format!("T{i}"))).collect();
    let team_refs: Vec<(i64, &str)> = teams.iter().map(|(i, s)| (*i, s.as_str())).collect();
    let r1_pairs: Vec<(i64, i64)> = (1..=8).map(|i| (i, i + 8)).collect();

    // Team 1 is the strongest overall team in the tournament (~90% map win rate vs everyone else,
    // nearly unstoppable in Bo3s), EXCEPT its Round 1 Bo1 opponent (Team 9) is a stylistic counter
    // where Team 1 only has a 35% Bo1 win probability.
    // Teams 2 and 3 are strong contenders (78% vs mid-tier, 95% vs their weak R1 opponents 10 & 11).
    // Teams 15 and 16 are clear 0-3 underdogs (8% vs everyone).
    // Consequently Team 1 has P(3-1 or 3-2) >> P(3-0), while Teams 2 and 3 have high P(3-0), so
    // total expected Pick'em Points are strictly maximised by placing Team 1 in `advance` (not `3-0`)
    // and Teams 2 & 3 in `three_nil`.
    let mut ratings = Vec::new();
    for a in 1..=16_i64 {
        for b in (a + 1)..=16_i64 {
            let map = if a == 1 && b == 9 {
                0.35
            } else if a == 1 {
                0.90
            } else if b == 15 || b == 16 {
                if a >= 15 { 0.50 } else { 0.92 }
            } else if (a == 2 && b == 10) || (a == 3 && b == 11) {
                0.95
            } else if a == 2 || a == 3 {
                0.78
            } else if a <= 8 && b > 8 {
                0.62
            } else {
                0.50
            };
            ratings.push(
                AsOf::new(
                    PairwiseProbability {
                        first: a,
                        second: b,
                        map,
                    },
                    obs,
                    cut,
                )
                .unwrap(),
            );
        }
    }

    let stage = AsOf::new(swiss_stage_json(&team_refs, "bo1", &r1_pairs), obs, cut).unwrap();
    let solver = MonteCarloPickemSolver::seeded(2026);
    let out = solver.solve(&stage, &ratings, cut).unwrap().unwrap();

    let three_nil: Vec<i64> = out["three_nil"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_i64().unwrap())
        .collect();
    let zero_three: Vec<i64> = out["zero_three"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_i64().unwrap())
        .collect();
    let advance: Vec<i64> = out["advance"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_i64().unwrap())
        .collect();

    assert!(
        advance.contains(&1) && !three_nil.contains(&1),
        "heavy favourite Team 1 should be placed in advance rather than three_nil, got three_nil={three_nil:?}, advance={advance:?}"
    );
    assert!(three_nil.contains(&2) && three_nil.contains(&3));
    assert!(zero_three.contains(&15) && zero_three.contains(&16));
}

#[test]
fn playoffs_solver_waits_until_ready_and_maximises_path_consistent_bracket_ev() {
    use kallbet::domain::pickem::{Bracket, BracketPickem, BracketSlot};

    let cut = cutoff();
    let obs = cut - Duration::hours(1);
    let solver = MonteCarloPickemSolver::seeded(7);

    let ratings = vec![
        // Team 1 narrowly beats Team 2 in SF0 (p_map = 0.5133 -> Bo3 ~ 0.52),
        // Team 3 and Team 4 are 50/50 in SF1,
        // Team 1 loses badly to 3 & 4 (p_map = 0.14 -> Bo5 ~ 0.02),
        // while Team 2 dominates 3 & 4 (p_map = 0.86 -> Bo5 ~ 0.98).
        AsOf::new(
            PairwiseProbability {
                first: 1,
                second: 2,
                map: 0.515,
            },
            obs,
            cut,
        )
        .unwrap(),
        AsOf::new(
            PairwiseProbability {
                first: 3,
                second: 4,
                map: 0.50,
            },
            obs,
            cut,
        )
        .unwrap(),
        AsOf::new(
            PairwiseProbability {
                first: 1,
                second: 3,
                map: 0.15,
            },
            obs,
            cut,
        )
        .unwrap(),
        AsOf::new(
            PairwiseProbability {
                first: 1,
                second: 4,
                map: 0.15,
            },
            obs,
            cut,
        )
        .unwrap(),
        AsOf::new(
            PairwiseProbability {
                first: 2,
                second: 3,
                map: 0.85,
            },
            obs,
            cut,
        )
        .unwrap(),
        AsOf::new(
            PairwiseProbability {
                first: 2,
                second: 4,
                map: 0.85,
            },
            obs,
            cut,
        )
        .unwrap(),
    ];

    // Not ready while a leaf slot is missing a team.
    let unready = AsOf::new(
        json!({
            "stage": { "ordinal": 2, "kind": "playoffs", "name": "Playoffs" },
            "teams": [{ "id": 1, "name": "T1" }, { "id": 2, "name": "T2" }, { "id": 3, "name": "T3" }],
            "bracket": {
                "ready": false,
                "slots": [
                    { "rounds_before_final": 1, "slot": 0, "team_a": 1, "team_b": 2, "leaf": true, "winner": null, "format": "bo3" },
                    { "rounds_before_final": 1, "slot": 1, "team_a": 3, "team_b": null, "leaf": true, "winner": null, "format": "bo3" },
                    { "rounds_before_final": 0, "slot": 0, "team_a": null, "team_b": null, "leaf": false, "winner": null, "format": "bo5" }
                ]
            }
        }),
        obs,
        cut,
    )
    .unwrap();
    assert_eq!(solver.solve(&unready, &ratings, cut).unwrap(), None);

    // Ready 4-team bracket: global EV picks Team 2 through SF0 and Final rather than myopic Team 1.
    let ready = AsOf::new(
        json!({
            "stage": { "ordinal": 2, "kind": "playoffs", "name": "Playoffs" },
            "teams": [
                { "id": 1, "name": "T1" },
                { "id": 2, "name": "T2" },
                { "id": 3, "name": "T3" },
                { "id": 4, "name": "T4" }
            ],
            "bracket": {
                "ready": true,
                "slots": [
                    { "rounds_before_final": 1, "slot": 0, "team_a": 1, "team_b": 2, "leaf": true, "winner": null, "format": "bo3" },
                    { "rounds_before_final": 1, "slot": 1, "team_a": 3, "team_b": 4, "leaf": true, "winner": null, "format": "bo3" },
                    { "rounds_before_final": 0, "slot": 0, "team_a": null, "team_b": null, "leaf": false, "winner": null, "format": "bo5" }
                ]
            }
        }),
        obs,
        cut,
    )
    .unwrap();

    let out = solver.solve(&ready, &ratings, cut).unwrap().unwrap();
    let picks: Vec<(u8, u8, TeamId)> = out["picks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| {
            (
                p["rounds_before_final"].as_u64().unwrap() as u8,
                p["slot"].as_u64().unwrap() as u8,
                TeamId(p["team_id"].as_i64().unwrap()),
            )
        })
        .collect();

    let domain_bracket = Bracket {
        slots: vec![
            BracketSlot {
                rounds_before_final: 1,
                slot: 0,
                leaf: true,
                teams: [Some(TeamId(1)), Some(TeamId(2))],
                winner: None,
            },
            BracketSlot {
                rounds_before_final: 1,
                slot: 1,
                leaf: true,
                teams: [Some(TeamId(3)), Some(TeamId(4))],
                winner: None,
            },
            BracketSlot {
                rounds_before_final: 0,
                slot: 0,
                leaf: false,
                teams: [None, None],
                winner: None,
            },
        ],
    };
    let bp = BracketPickem {
        winners: picks.clone(),
    };
    assert_eq!(bp.validate(&domain_bracket), Ok(()));

    let winner_of = |r: u8, s: u8| {
        picks
            .iter()
            .find(|&&(pr, ps, _)| pr == r && ps == s)
            .unwrap()
            .2
    };
    assert_eq!(winner_of(1, 0), TeamId(2));
    assert_eq!(winner_of(0, 0), TeamId(2));
}

#[test]
fn solver_rejects_mid_stage_data_and_post_cutoff_ratings_with_temporal_leak_error() {
    use kallbet_ai::TemporalLeakError;

    let cut = cutoff();
    let obs = cut - Duration::hours(1);
    let solver = MonteCarloPickemSolver::seeded(99);
    let rating = AsOf::new(
        PairwiseProbability {
            first: 1,
            second: 2,
            map: 0.6,
        },
        obs,
        cut,
    )
    .unwrap();

    // 1. Playoffs bracket with a non-leaf team already populated (mid-Stage leakage).
    let leaked_playoffs = AsOf::new(
        json!({
            "stage": { "ordinal": 2, "kind": "playoffs", "name": "Playoffs" },
            "teams": [
                { "id": 1, "name": "T1" },
                { "id": 2, "name": "T2" },
                { "id": 3, "name": "T3" },
                { "id": 4, "name": "T4" }
            ],
            "bracket": {
                "ready": true,
                "slots": [
                    { "rounds_before_final": 1, "slot": 0, "team_a": 1, "team_b": 2, "leaf": true, "winner": 1 },
                    { "rounds_before_final": 1, "slot": 1, "team_a": 3, "team_b": 4, "leaf": true, "winner": null },
                    { "rounds_before_final": 0, "slot": 0, "team_a": 1, "team_b": null, "leaf": false, "winner": null }
                ]
            }
        }),
        obs,
        cut,
    )
    .unwrap();
    assert!(
        solver
            .solve(&leaked_playoffs, std::slice::from_ref(&rating), cut)
            .unwrap_err()
            .is::<TemporalLeakError>()
    );

    // 2. Swiss stage containing Round 2 pairings (wins + losses > 0).
    let teams: Vec<(i64, String)> = (1..=16).map(|i| (i, format!("T{i}"))).collect();
    let team_refs: Vec<(i64, &str)> = teams.iter().map(|(i, s)| (*i, s.as_str())).collect();
    let r1_pairs: Vec<(i64, i64)> = (1..=8).map(|i| (i, i + 8)).collect();
    let mut swiss_json = swiss_stage_json(&team_refs, "bo1", &r1_pairs);
    swiss_json["groups"].as_array_mut().unwrap().push(json!({
        "kind": "record",
        "wins": 1,
        "losses": 0,
        "matches": [{
            "id": 201,
            "team_a": { "id": 1, "name": "T1" },
            "team_b": { "id": 2, "name": "T2" },
            "format": "bo1",
            "scheduled_start": "2026-11-25T04:00:00Z",
            "actual_start": null,
            "status": "upcoming",
            "winner": null
        }]
    }));
    let leaked_swiss = AsOf::new(swiss_json, obs, cut).unwrap();
    assert!(
        solver
            .solve(&leaked_swiss, std::slice::from_ref(&rating), cut)
            .unwrap_err()
            .is::<TemporalLeakError>()
    );

    // 3. Rating observed at an earlier-valid cutoff replayed against a stricter cutoff <= observed_at.
    let earlier_cutoff = obs;
    let valid_earlier_stage = AsOf::new(
        swiss_stage_json(&team_refs, "bo1", &r1_pairs),
        obs - Duration::minutes(5),
        earlier_cutoff,
    )
    .unwrap();
    assert!(
        solver
            .solve(&valid_earlier_stage, &[rating], earlier_cutoff)
            .unwrap_err()
            .is::<TemporalLeakError>()
    );
}

#[test]
fn solves_real_pgl_singapore_2026_stage1_pre_start_liquipedia_fixture() {
    use kallbet::domain::MatchPlacement;
    use kallbet::sources::liquipedia::parse::{self, TeamNames};

    const ROOT_JSON: &str =
        include_str!("fixtures/context/liquipedia/pgl-singapore-2026-rev-before-start.json");
    const STAGE1_JSON: &str =
        include_str!("fixtures/context/liquipedia/pgl-singapore-stage1-before-start.json");

    let mut pages = parse::query_pages(ROOT_JSON.trim_start_matches('\u{feff}')).unwrap();
    pages.extend(parse::query_pages(STAGE1_JSON.trim_start_matches('\u{feff}')).unwrap());

    let mut names = TeamNames::new();
    for (key, display) in [
        ("nip", "Ninjas in Pyjamas"),
        ("pvision", "PARIVISION"),
        ("heroic", "HEROIC"),
        ("faze", "FaZe"),
        ("tl", "Liquid"),
        ("ef", "Eternal Fire"),
        ("100t", "100 Thieves"),
        ("9z", "9z"),
        ("m80", "M80"),
        ("dendele", "DENDELE"),
        ("imp", "Imperial"),
        ("wildcard", "Wildcard"),
        ("gosh", "BC.Game"),
        ("fq", "FlyQuest"),
        ("chinggis", "Chinggis Warriors"),
        ("lvg", "Lynn Vision"),
    ] {
        names.insert(key.into(), display.into());
    }

    let stage_cutoff: Instant = "2026-11-24T04:00:00Z".parse().unwrap();
    let parsed_at: Instant = "2026-11-23T20:00:00Z".parse().unwrap();
    let snap = parse::parse_event("PGL/2026/Singapore", &pages)
        .unwrap()
        .into_snapshot(&names, parsed_at);

    // Extract the 16 teams and 8 Round 1 pairings from the real pre-Stage snapshot.
    let mut team_ids_by_name = std::collections::BTreeMap::new();
    let mut r1_pairs = Vec::new();
    for m in &snap.matches {
        if matches!(m.placement, MatchPlacement::Swiss { round: 1, .. }) {
            let ta = m.team_a.as_ref().unwrap();
            let tb = m.team_b.as_ref().unwrap();
            let next_id = team_ids_by_name.len() as i64 + 1;
            let id_a = *team_ids_by_name.entry(ta.name.clone()).or_insert(next_id);
            let next_id = team_ids_by_name.len() as i64 + 1;
            let id_b = *team_ids_by_name.entry(tb.name.clone()).or_insert(next_id);
            r1_pairs.push((id_a, id_b));
        }
    }
    assert_eq!(team_ids_by_name.len(), 16);
    assert_eq!(r1_pairs.len(), 8);

    let mut ordered_teams: Vec<(i64, String)> = team_ids_by_name
        .into_iter()
        .map(|(n, id)| (id, n))
        .collect();
    ordered_teams.sort_by_key(|(id, _)| *id);
    let team_refs: Vec<(i64, &str)> = ordered_teams
        .iter()
        .map(|(id, name)| (*id, name.as_str()))
        .collect();

    let standings = Standings::parse([VRS_NOV]).unwrap();
    let mut ratings = Vec::new();
    for i in 0..team_refs.len() {
        for j in (i + 1)..team_refs.len() {
            if let Some(sig) = standings
                .estimate(team_refs[i].1, team_refs[j].1, Format::Bo1, stage_cutoff)
                .unwrap()
            {
                ratings.push(
                    AsOf::new(
                        PairwiseProbability {
                            first: team_refs[i].0,
                            second: team_refs[j].0,
                            map: sig.at(stage_cutoff).unwrap().map,
                        },
                        sig.observed_at(),
                        stage_cutoff,
                    )
                    .unwrap(),
                );
            }
        }
    }

    let stage = AsOf::new(
        swiss_stage_json(&team_refs, "bo1", &r1_pairs),
        parsed_at,
        stage_cutoff,
    )
    .unwrap();
    let solver = MonteCarloPickemSolver::seeded(8302);
    let out = solver
        .solve(&stage, &ratings, stage_cutoff)
        .unwrap()
        .unwrap();
    let pickem = SwissPickem {
        three_nil: out["three_nil"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| TeamId(v.as_i64().unwrap()))
            .collect(),
        zero_three: out["zero_three"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| TeamId(v.as_i64().unwrap()))
            .collect(),
        advance: out["advance"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| TeamId(v.as_i64().unwrap()))
            .collect(),
    };
    let all_ids: Vec<TeamId> = ordered_teams.iter().map(|(id, _)| TeamId(*id)).collect();
    assert_eq!(pickem.validate(&all_ids), Ok(()));
}
#[test]
fn swiss_pairing_priorities_determine_the_last_advancer() {
    use kallbet_ai::RandomSource;

    // One tournament, with equal strengths. 0 selects the first side of a pairing.
    struct Outcomes(std::str::Bytes<'static>);
    impl RandomSource for Outcomes {
        fn next_u64(&mut self) -> u64 {
            match self.0.next().unwrap() {
                b'0' => 0,
                _ => u64::MAX,
            }
        }
    }

    let cut = cutoff();
    let before = cut - Duration::minutes(1);
    let teams: Vec<_> = (1..=16).map(|id| (id, "team")).collect();
    let pairs: Vec<_> = (1..=8).map(|id| (id, id + 8)).collect();
    let stage = AsOf::new(swiss_stage_json(&teams, "bo1", &pairs), before, cut).unwrap();
    let rating = AsOf::new(
        PairwiseProbability {
            first: 1,
            second: 2,
            map: 0.5,
        },
        before,
        cut,
    )
    .unwrap();
    let mut outcomes = Outcomes("010011101011110100111000101110001".bytes());
    let pick = MonteCarloPickemSolver::with_simulations(0, 1)
        .solve_with_rng(&stage, &[rating], cut, &mut outcomes)
        .unwrap()
        .unwrap();
    assert_eq!(pick["advance"], json!([6, 9, 12, 13, 14, 15]));
}
