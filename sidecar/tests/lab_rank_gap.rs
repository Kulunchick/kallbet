//! Tests for ticket #57 (`rank_gap`: the VRS and HLTV rank gap). Uses only saved
//! real fixtures in `sidecar/tests/fixtures/{backtest,research}/`; no network.
use chrono::{NaiveDate, TimeZone, Utc};
use kallbet_ai::{
    Instant,
    adapters::{
        backtest::{ReplayArchive, archive::read_file, rank_history::RankHistory},
        hltv_research::rank_badges::{RankBadge, parse_rank_badges},
        lab::{
            Lab,
            signals::{self, CandidateSignal, FittedSignal, MatchView, Observed},
            stats::Verdict,
        },
    },
};
use std::{
    collections::BTreeMap,
    path::Path,
    sync::{Arc, Mutex},
};

fn fixtures() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn page(name: &str) -> String {
    String::from_utf8(read_file(&fixtures().join("research").join(name)).unwrap()).unwrap()
}

fn badge(rank: u32, y: i32, m: u32, d: u32) -> Option<RankBadge> {
    Some(RankBadge { rank, date: NaiveDate::from_ymd_opt(y, m, d).unwrap() })
}

#[test]
fn rank_badges_parse_both_teams_from_the_saved_hltv_pages() {
    let heroic = parse_rank_badges(&page("hltv-match-2394774-heroic-vs-sharks.html.gz"));
    assert_eq!(heroic.len(), 2, "{heroic:?}");
    assert_eq!(heroic[0].team_id, 7175);
    assert_eq!(heroic[0].hltv, badge(27, 2026, 6, 1));
    assert_eq!(heroic[0].vrs, badge(31, 2026, 6, 2));
    assert_eq!(heroic[1].team_id, 8113);
    assert_eq!(heroic[1].hltv, badge(35, 2026, 6, 1));
    assert_eq!(heroic[1].vrs, badge(33, 2026, 6, 2));

    let vitality = parse_rank_badges(&page("hltv-match-2394898-vitality-vs-fut.html.gz"));
    assert_eq!(vitality.len(), 2, "{vitality:?}");
    assert_eq!((vitality[0].team_id, vitality[1].team_id), (9565, 13286));
    assert_eq!(vitality[0].hltv, badge(1, 2026, 6, 8));
    assert_eq!(vitality[0].vrs, badge(1, 2026, 6, 11));
    assert_eq!(vitality[1].hltv, badge(14, 2026, 6, 8));
    assert_eq!(vitality[1].vrs, badge(8, 2026, 6, 11));

    // A page without a match header has no badge.
    assert!(parse_rank_badges(&page("hltv-terms.html.gz")).is_empty());
}

/// Cologne 2026 Stage 3 (Vitality vs FUT is Match 1513120, HLTV 2394898; Vitality
/// vs 9z is 1513135) with the June 1 Valve standings and the raw Vitality-FUT page.
fn stage3_with_page() -> tempfile::TempDir {
    let root = fixtures();
    let dir = tempfile::tempdir().unwrap();
    for name in ["panda-21115.json.gz", "standings_global_2026_06_01.md.gz"] {
        std::fs::copy(root.join("backtest").join(name), dir.path().join(name)).unwrap();
    }
    let matches = dir.path().join("hltv").join("matches");
    std::fs::create_dir_all(&matches).unwrap();
    std::fs::write(dir.path().join("hltv").join("manifest.json"), br#"{"version":1}"#).unwrap();
    std::fs::copy(
        root.join("research/hltv-match-2394898-vitality-vs-fut.html.gz"),
        matches.join("hltv-match-2394898-vitality-vs-fut.html.gz"),
    )
    .unwrap();
    dir
}

fn end_of(y: i32, m: u32, d: u32) -> Instant {
    Utc.with_ymd_and_hms(y, m, d, 23, 59, 59).unwrap() + chrono::Duration::nanoseconds(999_999_999)
}

#[test]
fn hltv_rank_is_dated_by_its_ranking_day_and_vrs_rank_comes_from_dated_valve_snapshots() {
    let dir = stage3_with_page();
    let archive = ReplayArchive::load(dir.path()).unwrap();
    assert_eq!(archive.hltv_ranks.len(), 1, "the page's badges are loaded");
    let history = RankHistory::new(&archive).unwrap();
    let none = Default::default();

    // HLTV's June 8 ranking is known once June 8 (UTC) is over, never before.
    let monday_end = end_of(2026, 6, 8);
    let after = monday_end + chrono::Duration::nanoseconds(1);
    let vitality = history.hltv("Vitality", after, &none).unwrap().unwrap();
    assert_eq!(*vitality.at(after).unwrap(), 1);
    assert!(vitality.observed_at() >= monday_end && vitality.observed_at() < after);
    assert_eq!(*history.hltv("FUT", after, &none).unwrap().unwrap().at(after).unwrap(), 14);
    assert!(history.hltv("Vitality", monday_end, &none).unwrap().is_none());
    assert!(history.hltv("9z", after, &none).unwrap().is_none(), "no page, no rank");

    // VRS: the latest Valve snapshot strictly before the cutoff, never the page's badge
    // (FUT's badge says #8; Valve's June 1 standings, the latest before June 11, say #12).
    let match_day = Utc.with_ymd_and_hms(2026, 6, 11, 12, 10, 0).unwrap();
    let fut = history.vrs("FUT", match_day).unwrap();
    assert_eq!(*fut.at(match_day).unwrap(), 12);
    assert_eq!(fut.observed_at(), end_of(2026, 6, 1));
    assert_eq!(*history.vrs("9z", match_day).unwrap().at(match_day).unwrap(), 15);
    assert!(history.vrs("FUT", end_of(2026, 6, 1)).is_none());
}

/// A spy that records what `rank_gap` would read for every Match it is asked about.
#[derive(Default, Clone)]
struct Spy(Arc<Mutex<BTreeMap<i64, (Option<(u32, u32)>, Option<(u32, u32)>)>>>);
impl CandidateSignal for Spy {
    fn name(&self) -> &'static str {
        "rank_spy"
    }
    fn description(&self) -> &'static str {
        "records the ranks of each view"
    }
    fn needs(&self) -> &'static str {
        "ranks"
    }
    fn fit(&self, training: &[Observed]) -> kallbet_ai::Result<Box<dyn FittedSignal>> {
        for o in training {
            self.record(&o.view)?;
        }
        Ok(Box::new(self.clone()))
    }
}
impl Spy {
    fn record(&self, m: &MatchView) -> kallbet_ai::Result<()> {
        let read = |r: &Option<kallbet_ai::AsOf<(u32, u32)>>| {
            r.as_ref().map(|r| r.at(m.cutoff).map(|v| *v)).transpose()
        };
        let row = (read(&m.ranks.vrs)?, read(&m.ranks.hltv)?);
        self.0.lock().unwrap().insert(m.game.id, row);
        Ok(())
    }
}
impl FittedSignal for Spy {
    fn predict(&self, m: &MatchView) -> kallbet_ai::Result<Option<kallbet_ai::AsOf<f64>>> {
        self.record(m)?;
        Ok(None)
    }
}

#[tokio::test]
async fn the_views_carry_point_in_time_ranks_and_holdout_pages_never_reach_training() {
    let dir = stage3_with_page();
    let run = |holdout_prior: bool| {
        let path = dir.path().to_owned();
        async move {
            let mut archive = ReplayArchive::load(&path).unwrap();
            let stage = archive.stages.iter_mut().find(|s| s.id == 21115).unwrap();
            let finish = stage.matches.iter().find(|m| m.game.id == 1513120).unwrap().reported_finish.unwrap();
            let mut training_stage = None;
            if holdout_prior {
                // Vitality-FUT's Stage in the Holdout, Vitality-9z alone on the training side.
                let mut training = stage.clone();
                training.id = 88057;
                let i = stage.matches.iter().position(|m| m.game.id == 1513135).unwrap();
                training.matches = vec![stage.matches.remove(i)];
                training_stage = Some(training.id);
                archive.stages.push(training);
            }
            let mut lab = Lab::new(archive).unwrap();
            if let Some(id) = training_stage {
                lab.holdout.from = finish;
                lab.holdout.training_stages = vec![id];
            }
            let spy = Spy::default();
            lab.signals = vec![Arc::new(spy.clone())];
            lab.run().await.unwrap();
            let seen = spy.0.lock().unwrap().clone();
            seen
        }
    };
    let seen = run(false).await;
    // Vitality vs FUT (June 11): VRS from Valve's June 1 standings, HLTV from the June 8 ranking.
    assert_eq!(seen[&1513120], (Some((1, 12)), Some((1, 14))));
    // Vitality vs 9z: 9z has no HLTV rank in this archive.
    assert_eq!(seen[&1513135], (Some((1, 15)), None));

    let seen = run(true).await;
    assert!(!seen.contains_key(&1513120), "a Holdout Match is never read on the training side");
    assert_eq!(seen[&1513135], (Some((1, 15)), None));
}

#[tokio::test]
async fn rank_gap_is_registered_marked_and_reported_with_its_verdict() {
    let signal = signals::by_name("rank_gap").expect("rank_gap is registered");
    assert!(signal.added_after_list_fixed());
    assert_eq!(signal.feature_names(), vec!["vrs_rank_gap", "hltv_rank_gap"]);

    // The saved backtest archive: VRS ranks from its dated Valve snapshots, no HLTV page.
    let archive = ReplayArchive::load(&fixtures().join("backtest")).unwrap();
    let mut lab = Lab::new(archive).unwrap();
    lab.signals = vec![signal];
    let outcome = lab.run().await.unwrap();
    let row = &outcome.report.candidates[0];
    assert_eq!(row.name, "rank_gap");
    assert!(row.added_after_list_fixed);
    assert!(row.computed_on > 0 && row.paired > 0, "computed {} paired {}", row.computed_on, row.paired);
    let predicted = outcome
        .predictions
        .iter()
        .filter(|p| p.signal == "rank_gap" && p.variant.is_some())
        .count();
    assert_eq!(predicted, row.computed_on);
    assert!(row.difference.is_some() && row.interval.is_some());
    assert!(matches!(
        row.verdict,
        Verdict::Promote | Verdict::NoEvidence | Verdict::Harmful
    ));
    let md = outcome.report.markdown();
    assert!(md.contains("rank_gap") && md.contains("(added after list fixed)"), "{md}");
}
