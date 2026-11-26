//! Signal Lab replay over saved PandaScore and HLTV fixtures.
use chrono::Duration;
use kallbet_ai::{
    Format,
    adapters::{
        backtest::{ReplayArchive, archive::read_file},
        hltv_research::{archive::Side, parse::parse_match_page},
        lab::{Lab, LabOutcome, signals, stats::Verdict},
    },
    vrs::series_probability,
};
use std::path::Path;

const PRIOR: i64 = 1513120; // Vitality vs FUT, HLTV 2394898.
const LATER: i64 = 1513135; // Vitality vs 9z.

async fn run(scores: Option<(u32, u32)>, offset: Duration, hide_prior: bool) -> LabOutcome {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let dir = tempfile::tempdir().unwrap();
    for name in ["panda-21115.json.gz", "standings_global_2026_06_01.md.gz"] {
        std::fs::copy(root.join("backtest").join(name), dir.path().join(name)).unwrap();
    }
    if let Some((winner, loser)) = scores {
        let html =
            read_file(&root.join("research/hltv-match-2394898-vitality-vs-fut.html.gz")).unwrap();
        let mut record = parse_match_page(
            2394898,
            "https://www.hltv.org/matches/2394898/vitality-vs-fut-iem-cologne-major-2026",
            "2026-11-26T01:15:00Z".parse().unwrap(),
            std::str::from_utf8(&html).unwrap(),
        )
        .unwrap();
        record.veto.clear(); // Map scores are usable independently of veto availability.
        for map in record.maps.iter_mut().filter(|map| map.played) {
            let (a, b) = match map.winner.unwrap() {
                Side::A => (winner, loser),
                Side::B => (loser, winner),
            };
            map.team_a_score = Some(a);
            map.team_b_score = Some(b);
        }
        std::fs::write(
            dir.path().join("hltv-match-2394898.json"),
            serde_json::to_vec(&record).unwrap(),
        )
        .unwrap();
    }
    let mut archive = ReplayArchive::load(dir.path()).unwrap();
    let stage = &mut archive.stages[0];
    let finish = stage
        .matches
        .iter()
        .find(|m| m.game.id == PRIOR)
        .unwrap()
        .reported_finish
        .unwrap();
    let later_index = stage
        .matches
        .iter()
        .position(|m| m.game.id == LATER)
        .unwrap();
    let later = &mut stage.matches[later_index];
    later.actual_start = finish + Duration::minutes(15) + offset;
    later.game.scheduled_start = Some(later.actual_start);
    assert_eq!(later.cutoff(), finish + offset);
    if hide_prior {
        let mut training = stage.clone();
        training.id = 88061;
        training.matches = vec![stage.matches.remove(later_index)];
        archive.stages.push(training);
    }
    let mut lab = Lab::new(archive).unwrap();
    if hide_prior {
        lab.holdout.from = finish;
        lab.holdout.training_stages = vec![88061];
    }
    lab.signals = vec![signals::by_name("round_diff_rating").expect("registered Candidate Signal")];
    lab.run().await.unwrap()
}

fn prediction(outcome: &LabOutcome, id: i64) -> Option<f64> {
    outcome
        .predictions
        .iter()
        .find(|row| row.match_id == id)
        .and_then(|row| row.variant)
}

#[tokio::test]
async fn reports_round_rating_coverage_metrics_and_post_hoc_status() {
    let missing = run(None, Duration::seconds(1), false).await;
    assert_eq!(missing.report.candidates[0].verdict, Verdict::NotTestable);
    let replay = run(Some((13, 8)), Duration::seconds(1), false).await;
    assert_eq!(replay.report.protocol_version, "signal-lab/3");
    let row = &replay.report.candidates[0];
    assert_eq!(row.name, "round_diff_rating");
    assert!(row.computed_on >= row.paired && row.paired > 0);
    assert!(row.difference.is_some());
    assert!(row.interval.is_some() && row.accuracy_interval.is_some());
    assert_eq!(row.verdict, Verdict::NoEvidence);
    assert!(row.added_after_list_fixed);
    assert!(
        replay
            .report
            .markdown()
            .contains("no evidence (added after list fixed)")
    );
}

#[tokio::test]
async fn round_rating_uses_margins_only_after_finish_and_excludes_holdout() {
    let close = run(Some((13, 11)), Duration::seconds(1), false).await;
    let wide = run(Some((13, 4)), Duration::seconds(1), false).await;
    assert_eq!(prediction(&wide, PRIOR), None);
    let close = prediction(&close, LATER).unwrap();
    let wide = prediction(&wide, LATER).unwrap();
    assert!((wide - 0.5).abs() > (close - 0.5).abs());
    // Vitality won 2–1. The three margin targets average to (13 + 13 + 4) / 51.
    // 9z has no prior rating in this archive, and there is no market price.
    let rating = 32.0 * (30.0 / 51.0 - 0.5);
    let map = 1.0 / (1.0 + 10.0_f64.powf(-rating / 400.0));
    assert!((wide - series_probability(map, Format::Bo3)).abs() < 1e-12);
    let at = run(Some((13, 4)), Duration::zero(), false).await;
    let hidden = run(Some((13, 4)), Duration::seconds(1), true).await;
    assert_eq!(prediction(&at, LATER), None);
    assert_eq!(prediction(&hidden, LATER), None);
    assert_eq!(hidden.report.candidates[0].verdict, Verdict::NotTestable);
}
