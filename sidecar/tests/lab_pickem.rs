//! Ticket #46 (spec #33): Pick'em variants compared in Pick'em Points over every
//! completed Stage of the saved archive, with the Event-level resampling.
//! Seam: `adapters::lab::pickem::run` over the saved archive; statistics and
//! the validity check are pure functions with hand-counted cases.
use kallbet::domain::{
    TeamId,
    pickem::{Bracket, BracketPickem, BracketSlot, PickemError, SwissPickem},
};
use kallbet_ai::{
    adapters::{
        backtest::{ReplayArchive, StageKind},
        lab::{
            Lab,
            pickem::{self, PickemOptions, PickemOutcome, validate_payload},
            stats::Verdict,
        },
    },
    *,
};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/backtest")
}

fn lab() -> Lab {
    Lab::new(ReplayArchive::load(&fixtures()).unwrap()).unwrap()
}

/// Few simulations: the tests look at structure, not at the solver's accuracy.
fn options() -> PickemOptions {
    PickemOptions {
        simulations: 200,
        members: 4,
        ..PickemOptions::default()
    }
}

async fn outcome() -> PickemOutcome {
    pickem::run(&lab(), &options()).await.unwrap()
}

#[tokio::test]
async fn the_report_gives_total_and_per_stage_points_for_each_variant_with_an_interval() {
    let lab = lab();
    let out = pickem::run(&lab, &options()).await.unwrap();
    let report = &out.report;
    let names: Vec<_> = report.variants.iter().map(|v| v.name.as_str()).collect();
    assert_eq!(
        names,
        ["baseline", "calibrated", "learned", "variance_seeking"]
    );
    assert!(report.stages.len() >= 10, "{} Stages", report.stages.len());
    // Every Stage of the archive is scored or listed with the reason it is not.
    assert_eq!(report.stages.len() + report.excluded.len(), lab.archive.stages.len());
    for row in &report.stages {
        assert_eq!(row.points.len(), 4);
        assert!(row.points.iter().flatten().all(|p| *p <= row.picks));
    }
    // The totals are the sum of the per-Stage points, and the points are the
    // backtest's: recounted from the submitted Pick'em by KallBet's rules.
    for (i, variant) in report.variants.iter().enumerate() {
        let total: usize = report.stages.iter().filter_map(|r| r.points[i]).sum();
        assert_eq!(variant.total_points, Some(total), "{}", variant.name);
    }
    for sub in &out.submitted {
        let stage = lab.archive.stages.iter().find(|s| s.id == sub.stage_id).unwrap();
        let (points, _, _) =
            kallbet_ai::adapters::backtest::report::pickem_points(stage, &sub.payload);
        let row = report.stages.iter().find(|r| r.stage_id == sub.stage_id).unwrap();
        let column = names.iter().position(|n| *n == sub.variant).unwrap();
        assert_eq!(row.points[column], Some(points), "{} {}", sub.variant, row.stage);
    }
    let baseline = &report.variants[0];
    assert!(baseline.interval.is_none(), "the baseline is the reference");
    for v in &report.variants[1..] {
        match v.verdict {
            Verdict::NotTestable => assert!(!v.reason.is_empty()),
            _ => {
                let i = v.interval.expect("an interval of the difference");
                assert!(i.low <= v.difference.unwrap() && v.difference.unwrap() <= i.high);
                assert!(v.stages_compared > 0 && v.detectable.unwrap() >= 0.0);
            }
        }
    }
    let md = report.markdown();
    for needle in ["baseline", "calibrated", "learned", "variance_seeking", "Pick'em Points"] {
        assert!(md.contains(needle), "{needle}");
    }
}

#[tokio::test]
async fn every_submitted_pickem_is_valid_by_the_servers_rules() {
    let lab = lab();
    let out = pickem::run(&lab, &options()).await.unwrap();
    assert!(!out.submitted.is_empty());
    let (mut swiss, mut playoffs) = (0, 0);
    for sub in &out.submitted {
        let stage = lab.archive.stages.iter().find(|s| s.id == sub.stage_id).unwrap();
        let view = &stage.opening.as_ref().unwrap().value;
        // The lab's own check and the server's domain validators agree.
        validate_payload(view, &sub.payload).unwrap_or_else(|e| panic!("{}: {e}", sub.variant));
        let ids = |key: &str| -> Vec<TeamId> {
            sub.payload[key]
                .as_array()
                .into_iter()
                .flatten()
                .map(|v| TeamId(v.as_i64().unwrap()))
                .collect()
        };
        match stage.kind {
            StageKind::Swiss => {
                swiss += 1;
                let teams: Vec<TeamId> = view["teams"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|t| TeamId(t["id"].as_i64().unwrap()))
                    .collect();
                let p = SwissPickem {
                    three_nil: ids("three_nil"),
                    zero_three: ids("zero_three"),
                    advance: ids("advance"),
                };
                assert_eq!(p.validate(&teams), Ok(()), "{} {}", sub.variant, stage.title());
            }
            StageKind::Playoffs => {
                playoffs += 1;
                let slots: Vec<BracketSlot> = view["bracket"]["slots"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|s| BracketSlot {
                        rounds_before_final: s["rounds_before_final"].as_u64().unwrap() as u8,
                        slot: s["slot"].as_u64().unwrap() as u8,
                        leaf: s["leaf"].as_bool().unwrap(),
                        teams: [s["team_a"].as_i64(), s["team_b"].as_i64()].map(|t| t.map(TeamId)),
                        winner: None,
                    })
                    .collect();
                let p = BracketPickem {
                    winners: sub.payload["picks"]
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
                        .collect(),
                };
                assert_eq!(p.validate(&Bracket { slots }), Ok(()), "{}", sub.variant);
            }
            _ => panic!("a Stage that is neither Swiss nor Playoffs was scored"),
        }
    }
    assert!(swiss > 0 && playoffs > 0, "{swiss} Swiss, {playoffs} Playoffs");
}

#[test]
fn the_validity_check_refuses_what_the_server_refuses() {
    let teams: Vec<Value> = (1..=16).map(|i| json!({"id": i})).collect();
    let view = json!({"stage": {"kind": "swiss"}, "teams": teams});
    let ok = json!({"three_nil": [1, 2], "zero_three": [3, 4], "advance": [5, 6, 7, 8, 9, 10]});
    assert!(validate_payload(&view, &ok).is_ok());
    for bad in [
        json!({"three_nil": [1], "zero_three": [3, 4], "advance": [5, 6, 7, 8, 9, 10]}),
        json!({"three_nil": [1, 2], "zero_three": [3, 4], "advance": [5, 6, 7, 8, 9, 1]}),
        json!({"three_nil": [1, 2], "zero_three": [3, 4], "advance": [5, 6, 7, 8, 9, 99]}),
    ] {
        assert!(validate_payload(&view, &bad).is_err(), "{bad}");
    }
    // The server agrees on the duplicate.
    let dup = SwissPickem {
        three_nil: vec![TeamId(1), TeamId(2)],
        zero_three: vec![TeamId(3), TeamId(4)],
        advance: [5, 6, 7, 8, 9, 1].map(TeamId).to_vec(),
    };
    let ids: Vec<TeamId> = (1..=16).map(TeamId).collect();
    assert_eq!(dup.validate(&ids), Err(PickemError::DuplicateTeam));

    // A bracket pick must be a participant of its slot: a team the pick of the
    // feeder slots did not send there is path-inconsistent.
    let slots = json!([
        {"rounds_before_final": 1, "slot": 0, "leaf": true, "team_a": 1, "team_b": 2},
        {"rounds_before_final": 1, "slot": 1, "leaf": true, "team_a": 3, "team_b": 4},
        {"rounds_before_final": 0, "slot": 0, "leaf": false, "team_a": null, "team_b": null},
    ]);
    let bracket = json!({"stage": {"kind": "playoffs"}, "bracket": {"ready": true, "slots": slots}});
    let pick = |a, b, f| {
        json!({"picks": [
            {"rounds_before_final": 1, "slot": 0, "team_id": a},
            {"rounds_before_final": 1, "slot": 1, "team_id": b},
            {"rounds_before_final": 0, "slot": 0, "team_id": f},
        ]})
    };
    assert!(validate_payload(&bracket, &pick(1, 3, 3)).is_ok());
    assert!(validate_payload(&bracket, &pick(1, 3, 2)).is_err(), "2 lost in the semifinal");
    assert!(validate_payload(&bracket, &pick(1, 4, 3)).is_err(), "3 never reaches the final");
    assert!(validate_payload(&bracket, &pick(1, 9, 9)).is_err(), "9 is not in the slot");
}

#[tokio::test]
async fn a_view_holding_a_result_or_a_later_round_pairing_aborts_the_run() {
    let archive = ReplayArchive::load(&fixtures()).unwrap();
    let swiss = archive
        .stages
        .iter()
        .position(|s| s.kind == StageKind::Swiss && s.opening.is_some() && s.complete)
        .unwrap();
    let playoffs = archive
        .stages
        .iter()
        .position(|s| s.kind == StageKind::Playoffs && s.opening.is_some() && s.complete)
        .unwrap();
    let leaks: [(usize, Box<dyn Fn(&mut Value)>); 3] = [
        // A Match of the opening round already has its winner.
        (swiss, Box::new(|v| v["groups"][0]["matches"][0]["winner"] = json!(1))),
        // A second-round pairing: a team already in a 1-0 group.
        (
            swiss,
            Box::new(|v| {
                let m = v["groups"][0]["matches"][0].clone();
                v["groups"][0]["wins"] = json!(1);
                v["groups"][0]["matches"] = json!([m]);
            }),
        ),
        // A semifinal that already knows its teams.
        (
            playoffs,
            Box::new(|v| {
                let slots = v["bracket"]["slots"].as_array_mut().unwrap();
                let s = slots.iter_mut().find(|s| s["leaf"] == json!(false)).unwrap();
                s["team_a"] = json!(1);
            }),
        ),
    ];
    for (i, (stage, mutate)) in leaks.iter().enumerate() {
        let mut archive = ReplayArchive::load(&fixtures()).unwrap();
        mutate(&mut archive.stages[*stage].opening.as_mut().unwrap().value);
        let lab = Lab::new(archive).unwrap();
        let err = pickem::run(&lab, &options()).await.err().expect("the run aborts");
        assert!(err.is::<TemporalLeakError>(), "leak {i}: {err}");
    }
    // A view revised at or after the Stage's first Match aborts too.
    let mut archive = ReplayArchive::load(&fixtures()).unwrap();
    let cutoff = archive.stages[swiss].cutoff;
    archive.stages[swiss].opening.as_mut().unwrap().observed_at = cutoff;
    let lab = Lab::new(archive).unwrap();
    let err = pickem::run(&lab, &options()).await.err().expect("the run aborts");
    assert!(err.is::<TemporalLeakError>(), "{err}");
}

#[tokio::test]
async fn a_later_result_changes_no_pickem_made_before_it() {
    let base = pickem::run(&lab(), &options()).await.unwrap();
    // Flip every result of the last Stage of the archive: nothing submitted for
    // an earlier Stage of another Event can move.
    let mut archive = ReplayArchive::load(&fixtures()).unwrap();
    let last = archive.stages.iter().map(|s| s.cutoff).max().unwrap();
    let flipped: Vec<i64> = archive
        .stages
        .iter()
        .filter(|s| s.cutoff == last)
        .map(|s| s.id)
        .collect();
    for s in archive.stages.iter_mut().filter(|s| flipped.contains(&s.id)) {
        for r in &mut s.results {
            *r = (r.1, r.0);
        }
        for m in &mut s.matches {
            m.winner = if m.winner == m.game.team_a.id {
                m.game.team_b.id
            } else {
                m.game.team_a.id
            };
        }
    }
    let changed = pickem::run(&Lab::new(archive).unwrap(), &options()).await.unwrap();
    for sub in base.submitted.iter().filter(|s| !flipped.contains(&s.stage_id)) {
        let same = changed
            .submitted
            .iter()
            .find(|c| c.stage_id == sub.stage_id && c.variant == sub.variant)
            .unwrap();
        assert_eq!(sub.payload, same.payload, "{} {}", sub.variant, sub.stage_id);
    }
}

#[tokio::test]
async fn the_simulation_is_seeded_so_the_same_seed_gives_the_same_pickems() {
    let first = outcome().await;
    let second = outcome().await;
    let payloads = |o: &PickemOutcome| {
        o.submitted
            .iter()
            .map(|s| (s.variant.clone(), s.stage_id, s.payload.clone()))
            .collect::<Vec<_>>()
    };
    assert_eq!(payloads(&first), payloads(&second));
    assert_eq!(
        serde_json::to_string(&first.report).unwrap(),
        serde_json::to_string(&second.report).unwrap()
    );
    let mut other = lab();
    other.protocol.seed += 1;
    let third = pickem::run(&other, &options()).await.unwrap();
    assert_ne!(
        serde_json::to_string(&first.report).unwrap(),
        serde_json::to_string(&third.report).unwrap(),
        "the seed is in the report and decides the simulation"
    );
}

#[tokio::test]
async fn the_variance_seeking_variant_changes_a_pickem_only_where_the_bot_trails() {
    let out = outcome().await;
    let baseline = |stage: i64| {
        out.submitted
            .iter()
            .find(|s| s.variant == "baseline" && s.stage_id == stage)
            .unwrap()
    };
    for row in &out.report.stages {
        let variant = out
            .submitted
            .iter()
            .find(|s| s.variant == "variance_seeking" && s.stage_id == row.stage_id)
            .unwrap();
        if !row.trailing {
            assert_eq!(variant.payload, baseline(row.stage_id).payload, "{}", row.stage);
        }
    }
    // The first Stage of every Event has no earlier Leaderboard, so it cannot trail.
    let lab = lab();
    for event in lab.archive.stages.iter().map(|s| s.event_id).collect::<std::collections::BTreeSet<_>>() {
        let first = lab
            .archive
            .stages
            .iter()
            .filter(|s| s.event_id == event)
            .min_by_key(|s| s.cutoff)
            .unwrap();
        if let Some(row) = out.report.stages.iter().find(|r| r.stage_id == first.id) {
            assert!(!row.trailing, "{} has no Leaderboard yet", row.stage);
        }
    }
    // Without simulated Members there is no one to trail: the variant is the baseline.
    let alone = pickem::run(&lab, &PickemOptions { members: 0, ..options() })
        .await
        .unwrap();
    assert!(alone.report.stages.iter().all(|r| !r.trailing));
    assert_eq!(alone.report.variants[3].difference, Some(0.0));
}

#[test]
fn the_verdict_follows_the_written_rule() {
    use kallbet_ai::adapters::lab::{pickem::verdict, stats::Interval};
    let iv = |low, high| Interval { low, high };
    // Under the floor of Stages: no evidence, whatever the numbers.
    assert_eq!(verdict(5, iv(1.0, 2.0), 10).0, Verdict::NoEvidence);
    assert_eq!(verdict(10, iv(0.2, 2.0), 10).0, Verdict::Promote);
    assert_eq!(verdict(10, iv(-2.0, -0.2), 10).0, Verdict::Harmful);
    assert_eq!(verdict(10, iv(-0.2, 2.0), 10).0, Verdict::NoEvidence);
    assert_eq!(verdict(10, iv(0.0, 2.0), 10).0, Verdict::NoEvidence);
}

// Finding 4 (review of 6eea194): a Holdout Stage of an Event whose Stage 1 is
// on the training side is not scored: Singapore Stage 2, played and with an
// opening view, is listed as in the Holdout, never compared.
#[tokio::test]
async fn a_holdout_stage_of_a_training_side_event_is_not_scored() {
    let mut archive = ReplayArchive::load(&fixtures()).unwrap();
    archive.stages.retain(|s| s.event.contains("2026"));
    let stage1 = archive
        .stages
        .iter_mut()
        .find(|s| s.id == kallbet_ai::adapters::lab::holdout::SINGAPORE_STAGE_1)
        .unwrap();
    stage1.complete = true;
    let mut stage2 = stage1.clone();
    stage2.id = 1;
    stage2.name = "Stage 2".into();
    let shift = chrono::Duration::days(3);
    stage2.cutoff += shift;
    for m in &mut stage2.matches {
        m.actual_start += shift;
        m.finished_at += shift;
        m.reported_finish = m.reported_finish.map(|t| t + shift);
        m.game.scheduled_start = m.game.scheduled_start.map(|t| t + shift);
    }
    if let Some(opening) = &mut stage2.opening {
        opening.observed_at += shift;
    }
    archive.stages.push(stage2);
    let lab = Lab::new(archive).unwrap();
    assert!(lab.holdout_stages().contains(&1));
    let out = pickem::run(&lab, &options()).await.unwrap();
    assert!(out.report.stages.iter().all(|r| r.stage_id != 1));
    assert!(out.submitted.iter().all(|s| s.stage_id != 1));
    assert!(
        out.report
            .excluded
            .iter()
            .any(|e| e.contains("Stage 2") && e.contains("Holdout")),
        "{:?}",
        out.report.excluded
    );
}
