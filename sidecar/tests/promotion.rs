//! Ticket #47 (spec #33): a promoted Candidate Signal, or the frozen
//! combination, selected per bot in the live sidecar. The lab side runs the
//! Signal Lab over the saved replay archive; the live side is Seam 1 (the real
//! scheduler against the in-process server on the fake clock).
mod support;
use async_trait::async_trait;
use chrono::Duration;
use kallbet::{
    domain::StageKind,
    poller,
    testing::{EventSnapshotBuilder, TEST_BOT_SECRET},
};
use kallbet_ai::{
    adapters::{
        backtest::ReplayArchive,
        lab::{Holdout, Lab, LabOutcome, MatchView, signals, stats::Verdict},
    },
    promotion::{self, Promotion},
    *,
};
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use support::Harness;

fn archive() -> ReplayArchive {
    ReplayArchive::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/backtest"))
        .unwrap()
}

fn event_start(archive: &ReplayArchive, needle: &str) -> Instant {
    archive
        .stages
        .iter()
        .filter(|s| s.event.contains(needle) && s.event.contains("2026"))
        .map(|s| s.cutoff)
        .min()
        .unwrap()
}

/// A lab of the Candidate Signals that can run live, whose Holdout begins with
/// `needle`'s 2026 Major. From Cologne, the saved archive's training side is
/// four Majors and `rating_calibration` is promoted (ticket #37's notes).
fn lab_from(needle: &str) -> Lab {
    let mut lab = Lab::new(archive()).unwrap();
    lab.signals = promotion::LIVE
        .iter()
        .map(|name| signals::by_name(name).unwrap())
        .collect();
    lab.holdout = Holdout {
        from: event_start(&lab.archive, needle),
        training_stages: vec![],
    };
    lab
}

/// The lab run whose report the live bots read, written as the `lab` CLI writes it.
async fn promoted_report(dir: &Path) -> (LabOutcome, PathBuf) {
    let outcome = lab_from("Cologne").run().await.unwrap();
    let c = outcome
        .report
        .candidates
        .iter()
        .find(|c| c.name == "rating_calibration")
        .unwrap();
    assert_eq!(c.verdict, Verdict::Promote, "{}", c.reason);
    assert!(
        outcome
            .report
            .selection
            .frozen
            .contains(&"rating_calibration".to_string())
    );
    let path = dir.join("report.json");
    std::fs::write(&path, serde_json::to_vec_pretty(&outcome.report).unwrap()).unwrap();
    (outcome, path)
}

/// A Match as the lab's replay ports give it at its cutoff, without the
/// archive history (the live sidecar has none).
async fn live_view(lab: &Lab, id: i64) -> MatchView {
    let m = lab
        .archive
        .stages
        .iter()
        .flat_map(|s| &s.matches)
        .find(|m| m.game.id == id)
        .unwrap();
    let cutoff = m.cutoff();
    MatchView::live(
        m.game.clone(),
        cutoff,
        lab.feeds.signal.signal(&m.game, cutoff).await.unwrap(),
        lab.feeds.odds.quote(&m.game, cutoff).await.unwrap(),
    )
}

#[tokio::test]
async fn the_live_value_equals_the_value_the_lab_computed_for_the_same_match_and_inputs() {
    let dir = tempfile::tempdir().unwrap();
    let (outcome, report) = promoted_report(dir.path()).await;
    let lab = lab_from("Cologne");

    // The frozen combination: the lab's Holdout step fits it on the same
    // training-side Events as the deployment, so every Holdout Match compares.
    let combination = Promotion::load("combination", &report).unwrap();
    assert!(!outcome.holdout.is_empty());
    let mut compared = 0;
    for row in &outcome.holdout {
        let live = combination
            .live(&live_view(&lab, row.match_id).await)
            .unwrap();
        match row.variant {
            Some(v) => {
                assert!(live.available, "Match {}", row.match_id);
                assert_eq!(live.probability, Some(v), "Match {}", row.match_id);
                compared += 1;
            }
            None => assert_eq!(live.probability, row.baseline, "Match {}", row.match_id),
        }
    }
    assert!(compared > 100, "{compared}");

    // The signal alone: a lab whose Holdout begins with Singapore fits
    // `rating_calibration` for Cologne on the same four Majors the deployment
    // was fitted on.
    let single = Promotion::load("rating_calibration", &report).unwrap();
    let later = lab_from("Singapore").run().await.unwrap();
    let cologne = later
        .report
        .events
        .iter()
        .find(|e| e.title.contains("Cologne") && e.title.contains("2026"))
        .unwrap()
        .id;
    let mut compared = 0;
    for row in later
        .predictions
        .iter()
        .filter(|p| p.signal == "rating_calibration" && p.event_id == cologne)
    {
        let live = single.live(&live_view(&lab, row.match_id).await).unwrap();
        match row.variant {
            Some(v) => {
                assert_eq!(live.probability, Some(v), "Match {}", row.match_id);
                compared += 1;
            }
            None => assert!(!live.available),
        }
    }
    assert!(compared > 50, "{compared}");
}

fn config(dir: &Path, base: &str, bots: Value) -> PathBuf {
    let path = dir.join("config.json");
    let vrs = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/vrs/standings_global_2026_11_02.md"
    );
    std::fs::write(
        &path,
        json!({"platform_url": base, "audit_file": dir.join("audit.db"), "vrs_files": [vrs], "bots": bots})
            .to_string(),
    )
    .unwrap();
    path
}

fn bot(key: &str, policy: Value, invite: &str) -> Value {
    json!({"bot_key": key, "nickname": key, "invite_links": [invite], "policy": policy})
}

async fn load_error(policy: Value, report: &Path) -> String {
    let h = Harness::new().await;
    let dir = tempfile::tempdir().unwrap();
    let mut policy = policy;
    policy["Promoted"]["report"] = json!(report);
    let path = config(dir.path(), &h.base, json!([bot("lab", policy, "")]));
    match Sidecar::from_config(
        Config::load(path).unwrap(),
        TEST_BOT_SECRET.into(),
        h.clock(),
    )
    .await
    {
        Ok(_) => panic!("configuration accepted"),
        Err(e) => e.to_string(),
    }
}

#[tokio::test]
async fn selecting_a_name_the_lab_did_not_promote_is_a_configuration_error_with_the_reason() {
    let dir = tempfile::tempdir().unwrap();
    let report = dir.path().join("report.json");
    std::fs::write(
        &report,
        json!({
            "protocol_version": kallbet_ai::adapters::lab::PROTOCOL_VERSION,
            "candidates": [
                {"name": "rating_calibration", "verdict": "no evidence", "reason": "interval crosses zero"},
                {"name": "own_rating", "verdict": "promote", "reason": "interval below zero"},
            ],
            "selection": {"frozen": []},
            "deployment": {},
        })
        .to_string(),
    )
    .unwrap();
    let promoted = |name: &str| json!({"Promoted": {"name": name}});

    let e = load_error(promoted("rating_calibration"), &report).await;
    assert!(
        e.contains("rating_calibration was not promoted: no evidence (interval crosses zero)"),
        "{e}"
    );
    let e = load_error(promoted("recent_form"), &report).await;
    assert!(
        e.contains("recent_form was not evaluated by this lab run"),
        "{e}"
    );
    let e = load_error(promoted("combination"), &report).await;
    assert!(e.contains("froze no Candidate Signal"), "{e}");

    // Finding 17 (review of 6eea194): verdicts of a run under another protocol
    // than the prescribed one (here one bootstrap resample) cannot be promoted.
    let mut degenerate = serde_json::from_slice::<Value>(&std::fs::read(&report).unwrap()).unwrap();
    degenerate["resamples"] = json!(1);
    degenerate["level"] = json!(0.9);
    degenerate["min_matches"] = json!(100);
    std::fs::write(&report, degenerate.to_string()).unwrap();
    let e = load_error(promoted("own_rating"), &report).await;
    assert!(e.contains("1 bootstrap resamples"), "{e}");
    degenerate["resamples"] = json!(2000);
    std::fs::write(&report, degenerate.to_string()).unwrap();

    let mut old = serde_json::from_slice::<Value>(&std::fs::read(&report).unwrap()).unwrap();
    old["protocol_version"] = json!("signal-lab/1");
    std::fs::write(&report, old.to_string()).unwrap();
    let e = load_error(promoted("rating_calibration"), &report).await;
    assert!(e.contains("protocol is \"signal-lab/1\""), "{e}");

    let missing = dir.path().join("missing.json");
    let e = load_error(promoted("rating_calibration"), &missing).await;
    assert!(e.contains("promoted criterion rating_calibration"), "{e}");
}

/// What the Signal Estimator gave, for which Match and when.
type Seen = Vec<(Match, Instant, Option<AsOf<Signal>>)>;

/// The sidecar's Signal Estimator, recording what it gave the scheduler.
struct Spy {
    inner: Arc<dyn SignalEstimator>,
    seen: Mutex<Seen>,
}
#[async_trait]
impl SignalEstimator for Spy {
    async fn signal(&self, game: &Match, at: Instant) -> Result<Option<AsOf<Signal>>> {
        let s = self.inner.signal(game, at).await?;
        self.seen
            .lock()
            .unwrap()
            .push((game.clone(), at, s.clone()));
        Ok(s)
    }
}

struct Live {
    h: Harness,
    _dir: tempfile::TempDir,
    report: PathBuf,
    sidecar: Sidecar,
    spy: Arc<Spy>,
    human: kallbet::testing::TestClient,
    path: String,
    start: Instant,
}

/// One League with Match 42 (`team_a` vs `team_b`), a VrsOnly bot as before
/// and a bot with `rating_calibration` promoted, built by `Sidecar::from_config`.
async fn live(team_a: &str, team_b: &str) -> Live {
    live_with(team_a, team_b, None).await
}

/// As `live`, the bot selecting `name` from `report` when given.
async fn live_with(team_a: &str, team_b: &str, selected: Option<(&str, PathBuf)>) -> Live {
    let h = Harness::new().await;
    let dir = tempfile::tempdir().unwrap();
    let (name, report) = match selected {
        Some((name, report)) => (name.to_string(), report),
        None => ("rating_calibration".to_string(), promoted_report(dir.path()).await.1),
    };
    let now: Instant = "2026-11-24T00:00:00Z".parse().unwrap();
    h.app.clock.set(now);
    let start = now + Duration::hours(4);
    h.app.source.put_event(
        EventSnapshotBuilder::new(99, "Major")
            .stage(StageKind::Swiss, "Stage 1")
            .swiss_match(42, (1, 0, 0), team_a, team_b, start)
            .build(now),
    );
    let human = h.app.user("Alice").await;
    let league = human
        .post("/api/leagues", &json!({"name": "Friends", "event_id": 99}))
        .await
        .json();
    let id = league["id"].as_i64().unwrap();
    let invite = format!(
        "{}/join/{}",
        h.base,
        league["invite_token"].as_str().unwrap()
    );
    poller::tick(&h.app.state).await;
    let path = config(
        dir.path(),
        &h.base,
        json!([
            bot("vrs", json!("VrsOnly"), &invite),
            bot(
                "lab",
                json!({"Promoted": {"name": name, "report": report}}),
                &invite
            ),
        ]),
    );
    let mut sidecar = Sidecar::from_config(
        Config::load(path).unwrap(),
        TEST_BOT_SECRET.into(),
        h.clock(),
    )
    .await
    .unwrap();
    let spy = Arc::new(Spy {
        inner: sidecar.ports.signal.clone(),
        seen: Mutex::default(),
    });
    sidecar.ports.signal = spy.clone();
    sidecar.ports.odds = h.scripts.clone();
    sidecar.ports.context = h.scripts.clone();
    sidecar.ports.llm = h.scripts.clone();
    Live {
        h,
        _dir: dir,
        report,
        sidecar,
        spy,
        human,
        path: format!("/api/leagues/{id}/matches"),
        start,
    }
}

impl Live {
    /// The bots' revealed picks on the Match card, after a human's early Reveal.
    async fn picks(&self) -> Vec<Value> {
        let view = self.human.get(&self.path).await.json();
        view["stages"][0]["groups"][0]["matches"][0]["predictions"]["revealed"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|p| p["is_bot"] == true)
            .cloned()
            .collect()
    }
    fn by_summary<'a>(picks: &'a [Value], needle: &str) -> &'a Value {
        picks
            .iter()
            .find(|p| {
                p["rationale"]["summary"]["en"]
                    .as_str()
                    .is_some_and(|s| s.contains(needle))
            })
            .unwrap_or_else(|| panic!("a pick whose summary says {needle}: {picks:?}"))
    }
    async fn reveal(&self) {
        let put = self
            .human
            .put(
                &format!("{}/42/prediction", self.path),
                &json!({"team_id": 2}),
            )
            .await;
        assert!(put.status.is_success());
        self.h.app.clock.advance(Duration::minutes(5));
    }
}

#[tokio::test]
async fn a_promoted_bot_submits_a_prediction_from_its_criterion_named_in_three_languages() {
    let mut l = live("MOUZ", "Vitality").await;
    let at = l.start - Duration::minutes(22);
    assert_eq!(l.h.tick(&mut l.sidecar, at).await.unwrap(), 2);
    assert!(l.h.scripts.llm_calls.lock().unwrap().is_empty());
    l.reveal().await;
    let picks = l.picks().await;
    assert_eq!(picks.len(), 2);

    let lab = Live::by_summary(&picks, "Signal Lab criterion: rating_calibration.");
    let summary = &lab["rationale"]["summary"];
    assert_eq!(summary["en"], "Signal Lab criterion: rating_calibration.");
    assert_eq!(
        summary["ru"],
        "Критерий лаборатории сигналов: rating_calibration."
    );
    assert_eq!(
        summary["uk"],
        "Критерій лабораторії сигналів: rating_calibration."
    );
    assert_eq!(lab["rationale"]["preliminary"], false);

    // The value is the lab's, for the inputs the live ports gave the scheduler.
    let (game, now, signal) = l
        .spy
        .seen
        .lock()
        .unwrap()
        .iter()
        .rfind(|(g, _, _)| g.id == 42)
        .cloned()
        .unwrap();
    assert!(signal.is_some());
    let view = MatchView::live(game.clone(), now, signal, None);
    let promotion = Promotion::load("rating_calibration", &l.report).unwrap();
    let expected = promotion.live(&view).unwrap();
    assert!(expected.available);
    let p = expected.probability.unwrap();
    let (team, p_win) = if p >= 0.5 {
        (game.team_a.id, p)
    } else {
        (game.team_b.id, 1.0 - p)
    };
    assert_eq!(lab["team_id"], team);
    assert!((lab["rationale"]["p_win"].as_f64().unwrap() - p_win).abs() < 1e-12);

    // It reflects the criterion: the VrsOnly bot beside it gives the anchor.
    let anchor = view.anchor().unwrap().unwrap();
    let vrs = picks.iter().find(|p| p != &lab).unwrap();
    let anchor_win = if anchor >= 0.5 { anchor } else { 1.0 - anchor };
    assert!((vrs["rationale"]["p_win"].as_f64().unwrap() - anchor_win).abs() < 1e-12);
    assert!((p_win - anchor_win).abs() > 1e-3, "{p_win} vs {anchor_win}");

    // The audit records it, and a later tick on the same inputs decides nothing new.
    let submissions = l.sidecar.audit.submissions().await.unwrap();
    assert_eq!(submissions.len(), 2);
    assert!(l.sidecar.audit.failures().await.unwrap().is_empty());
    assert_eq!(
        l.h.tick(&mut l.sidecar, at + Duration::minutes(1))
            .await
            .unwrap(),
        0
    );
}

#[tokio::test]
async fn a_signal_without_live_data_degrades_to_the_anchor_and_says_so_without_failing_the_tick() {
    // "Nobody Esports" has no Valve rating: the calibration has nothing to
    // calibrate, so the anchor (the market line alone) decides.
    let mut l = live("Nobody Esports", "Vitality").await;
    let at = l.start - Duration::minutes(22);
    *l.h.scripts.market.lock().unwrap() =
        Some(AsOf::new(0.37, at - Duration::minutes(5), at).unwrap());
    // The VrsOnly bot has no rating to decide on; the promoted one still submits.
    assert_eq!(l.h.tick(&mut l.sidecar, at).await.unwrap(), 1);
    assert!(l.sidecar.audit.failures().await.unwrap().is_empty());
    l.reveal().await;
    let picks = l.picks().await;
    let lab = Live::by_summary(
        &picks,
        "No live data for the Signal Lab criterion rating_calibration",
    );
    let summary = &lab["rationale"]["summary"];
    assert!(
        summary["ru"]
            .as_str()
            .unwrap()
            .contains("rating_calibration")
    );
    assert!(
        summary["uk"]
            .as_str()
            .unwrap()
            .contains("rating_calibration")
    );
    assert!((lab["rationale"]["p_win"].as_f64().unwrap() - 0.63).abs() < 1e-12);
    let audited = l.sidecar.audit.submissions().await.unwrap();
    assert!(audited.iter().any(|p| {
        p.rationale["summary"]["en"]
            .as_str()
            .is_some_and(|s| s.contains("No live data") && s.contains("the anchor decided"))
    }));
}

#[test]
fn the_sample_configurations_select_no_promoted_criterion() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../deploy/sidecar");
    let mut seen = 0;
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|e| e == "json") {
            let text = std::fs::read_to_string(&path).unwrap();
            assert!(!text.contains("Promoted"), "{}", path.display());
            seen += 1;
        }
    }
    assert!(seen >= 3);
}

fn research_report() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../docs/lab/research/report.json")
}

// Finding 7 (review of 6eea194): every promoted Candidate Signal, and the
// frozen combination, can be selected. One whose inputs the live sidecar has
// no port for (the results history, played maps, the venue) decides by the
// anchor for each Match and says why; nothing fails at startup.
#[tokio::test]
async fn the_research_reports_promoted_criteria_load_and_fall_back_per_match_with_the_reason() {
    let report = research_report();
    let lab = lab_from("Cologne");
    let id = lab.archive.stages.iter().find(|s| s.id == 20708).unwrap().matches[0].game.id;
    let view = live_view(&lab, id).await;
    for (name, needs) in [
        ("own_rating", "results history"),
        ("per_map_rating", "played maps"),
        ("lan_online", "no port"),
        ("combination", "results history"),
    ] {
        let p = Promotion::load(name, &report).unwrap_or_else(|e| panic!("{name}: {e}"));
        let live = p.live(&view).unwrap();
        assert!(!live.available, "{name}");
        assert_eq!(live.probability, view.anchor().unwrap(), "{name}");
        let reason = live.reason.clone().unwrap_or_default();
        assert!(reason.contains(needs) && reason.contains("no port"), "{name}: {reason}");
    }
    let calibration = Promotion::load("rating_calibration", &report).unwrap();
    let live = calibration.live(&view).unwrap();
    assert!(live.available && live.reason.is_none());
}

#[tokio::test]
async fn the_frozen_combination_of_the_research_report_runs_live_on_the_anchor() {
    let mut l = live_with("Vitality", "Spirit", Some(("combination", research_report()))).await;
    let at = l.start - Duration::minutes(22);
    *l.h.scripts.market.lock().unwrap() =
        Some(AsOf::new(0.62, at - Duration::minutes(5), at).unwrap());
    assert!(l.h.tick(&mut l.sidecar, at).await.unwrap() >= 1);
    assert!(l.sidecar.audit.failures().await.unwrap().is_empty());
    l.reveal().await;
    let picks = l.picks().await;
    let lab = Live::by_summary(&picks, "No live data for the Signal Lab criterion own_rating");
    let en = lab["rationale"]["summary"]["en"].as_str().unwrap();
    assert!(en.contains("results history"), "{en}");
}
