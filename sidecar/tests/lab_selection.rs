//! Seam 1 of ticket #37 (spec #33): the combiner's learned weights, forward
//! selection and the single-use Holdout, through the Signal Lab's runner over
//! the saved replay archive (`tests/fixtures/backtest/`).
use kallbet_ai::{
    adapters::{
        backtest::ReplayArchive,
        lab::{
            CandidateReport, CandidateSignal, FittedSignal, Holdout, HoldoutStatus, Lab,
            LabOutcome, Observed, signals,
            stats::{self, Verdict},
        },
    },
    *,
};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/backtest")
}

fn whole() -> ReplayArchive {
    ReplayArchive::load(&fixtures()).unwrap()
}

fn candidate<'a>(outcome: &'a LabOutcome, name: &str) -> &'a CandidateReport {
    outcome
        .report
        .candidates
        .iter()
        .find(|c| c.name == name)
        .unwrap_or_else(|| panic!("{name} is reported"))
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

/// A lab whose Holdout begins with IEM Cologne Major 2026, so that the saved
/// archive has one (Cologne and Singapore) and a training side of four Majors.
fn lab_with_holdout(archive: ReplayArchive) -> Lab {
    let mut lab = Lab::new(archive).unwrap();
    lab.holdout = Holdout {
        from: event_start(&lab.archive, "Cologne"),
        training_stages: vec![],
    };
    lab
}

#[tokio::test]
async fn the_learned_weights_are_reported_under_the_protocol_with_the_weights_of_every_event() {
    let lab = Lab::new(whole()).unwrap();
    let outcome = lab.run().await.unwrap();
    let c = candidate(&outcome, "learned_weights");
    assert!(c.paired >= 100, "{}", c.paired);
    let interval = c.interval.expect("an interval");
    let (verdict, reason) = stats::verdict(
        c.paired,
        interval,
        c.market.as_ref().map(|m| m.log_loss_difference),
        &lab.protocol,
    );
    assert_eq!((c.verdict, c.reason.as_str()), (verdict, reason.as_str()));

    // Every evaluated Event has its fitted weights, fitted on exactly the
    // Matches the anchor could predict in the Events that ended before it.
    let events = &outcome.report.events;
    let predicted: BTreeMap<i64, usize> = outcome
        .predictions
        .iter()
        .filter(|p| p.signal == "learned_weights" && p.baseline.is_some())
        .fold(BTreeMap::new(), |mut acc, p| {
            *acc.entry(p.event_id).or_default() += 1;
            acc
        });
    assert_eq!(c.per_event.len(), events.len());
    for (row, info) in c.per_event.iter().zip(events) {
        let p = row.parameters.as_ref().expect("fitted weights per Event");
        let fitted_on: u64 = ["market+rating", "market", "rating"]
            .iter()
            .map(|k| p[k]["matches"].as_u64().unwrap())
            .sum();
        let expected: usize = events
            .iter()
            .filter(|e| e.ends < info.begins)
            .map(|e| predicted.get(&e.id).copied().unwrap_or(0))
            .sum();
        assert_eq!(fitted_on as usize, expected, "{}", row.title);
        for k in ["market+rating", "market", "rating"] {
            assert!(
                p[k].get(if k == "market" { "market" } else { "rating" })
                    .is_some()
            );
        }
    }
    // The first Event has no Event before it: its weights are the anchor's,
    // and so are its probabilities.
    let first = c.per_event[0].parameters.as_ref().unwrap();
    assert_eq!(first["market+rating"]["market"], 0.8);
    assert_eq!(first["market+rating"]["rating"], 0.2);
    for p in outcome
        .predictions
        .iter()
        .filter(|p| p.signal == "learned_weights" && p.event_id == events[0].id)
    {
        if let (Some(b), Some(v)) = (p.baseline, p.variant) {
            assert!((b - v).abs() < 1e-9, "{p:?}");
        }
    }
    let md = outcome.report.markdown();
    assert!(md.contains("## `learned_weights`"), "{md}");
    assert!(
        md.contains("market+rating"),
        "the weights are in the markdown"
    );
}

fn reverse(archive: &mut ReplayArchive, needle: &str) {
    for stage in archive
        .stages
        .iter_mut()
        .filter(|s| s.event.contains(needle) && s.event.contains("2026"))
    {
        for m in &mut stage.matches {
            m.winner = if m.winner == m.game.team_a.id {
                m.game.team_b.id
            } else {
                m.game.team_a.id
            };
        }
    }
}

#[tokio::test]
async fn no_weight_is_fitted_on_a_match_of_the_evaluated_event_or_a_later_one() {
    let base = Lab::new(whole()).unwrap().run().await.unwrap();
    let weights = |o: &LabOutcome| -> Vec<(String, Option<Value>)> {
        candidate(o, "learned_weights")
            .per_event
            .iter()
            .map(|e| (e.title.clone(), e.parameters.clone()))
            .collect()
    };
    // Reversing Singapore, the last Event, changes no fitted weight at all.
    let mut archive = whole();
    reverse(&mut archive, "Singapore");
    let singapore = Lab::new(archive).unwrap().run().await.unwrap();
    assert_eq!(weights(&base), weights(&singapore));
    // Reversing Cologne changes Singapore's weights only.
    let mut archive = whole();
    reverse(&mut archive, "Cologne");
    let cologne = Lab::new(archive).unwrap().run().await.unwrap();
    for (b, c) in weights(&base).iter().zip(weights(&cologne)) {
        if b.0.contains("Singapore") {
            assert_ne!(b.1, c.1, "Singapore is fitted on Cologne");
        } else {
            assert_eq!(*b, c);
        }
    }
}

#[tokio::test]
async fn forward_selection_is_deterministic_and_lists_the_order_and_what_each_added() {
    let one = lab_with_holdout(whole()).run().await.unwrap();
    let two = lab_with_holdout(whole()).run().await.unwrap();
    let s = &one.report.selection;
    assert_eq!(
        s, &two.report.selection,
        "same archive and seed, same selection"
    );
    assert_eq!(one.combination, two.combination);

    let promoted: Vec<String> = one
        .report
        .candidates
        .iter()
        .filter(|c| c.verdict == Verdict::Promote)
        .map(|c| c.name.clone())
        .collect();
    assert!(
        !promoted.is_empty(),
        "the calibration promotes on four Majors"
    );
    assert_eq!(
        s.considered, promoted,
        "only the promoted ones, in registry order"
    );
    assert!(!s.steps.is_empty());
    let entered: Vec<String> = s.steps.iter().map(|x| x.entered.clone()).collect();
    assert_eq!(s.frozen, entered);
    let mut before = s.anchor_log_loss.unwrap();
    for step in &s.steps {
        assert!(
            step.added < 0.0,
            "a step enters only by lowering the log-loss"
        );
        assert!((step.log_loss - before - step.added).abs() < 1e-12);
        assert!(step.interval.is_some());
        let best = step
            .tried
            .iter()
            .map(|t| t.log_loss)
            .fold(f64::INFINITY, f64::min);
        assert_eq!(step.log_loss, best, "the best of those tried enters");
        before = step.log_loss;
    }
    assert_eq!(s.final_log_loss, Some(before));
    // The combination's log-loss is the mean over its walk-forward predictions.
    let winners: BTreeMap<i64, bool> = whole()
        .stages
        .iter()
        .flat_map(|st| &st.matches)
        .map(|m| (m.game.id, m.winner == m.game.team_a.id))
        .collect();
    let scored: Vec<f64> = one
        .combination
        .iter()
        .filter_map(|p| Some(stats::log_loss(p.variant?, winners[&p.match_id])))
        .collect();
    assert_eq!(scored.len(), s.matches);
    let mean = scored.iter().sum::<f64>() / scored.len() as f64;
    assert!((mean - before).abs() < 1e-12, "{mean} vs {before}");
    // The order does not depend on the bootstrap seed.
    let mut other = lab_with_holdout(whole());
    other.protocol.seed = 1;
    let other = other.run().await.unwrap();
    assert_eq!(other.report.selection.frozen, s.frozen);

    let md = one.report.markdown();
    assert!(md.contains("## Forward selection"), "{md}");
    for name in &s.frozen {
        assert!(md.contains(&format!("| `{name}` |")), "{name}:\n{md}");
    }
}

/// Wraps a Candidate Signal and records every Match its fits are given.
struct Spy {
    inner: Arc<dyn CandidateSignal>,
    seen: Arc<Mutex<BTreeSet<i64>>>,
}
impl CandidateSignal for Spy {
    fn name(&self) -> &'static str {
        "spy"
    }
    fn description(&self) -> &'static str {
        "the rating calibration, recording what it is fitted on"
    }
    fn needs(&self) -> &'static str {
        self.inner.needs()
    }
    fn fit(&self, training: &[Observed]) -> Result<Box<dyn FittedSignal>> {
        self.seen
            .lock()
            .unwrap()
            .extend(training.iter().map(|o| o.view.game.id));
        self.inner.fit(training)
    }
}

#[tokio::test]
async fn no_fitting_and_no_selection_step_sees_a_holdout_match() {
    let archive = whole();
    let holdout_ids: BTreeSet<i64> = archive
        .stages
        .iter()
        .filter(|s| s.event.contains("2026"))
        .flat_map(|s| s.matches.iter().map(|m| m.game.id))
        .collect();
    assert!(!holdout_ids.is_empty());
    let seen = Arc::new(Mutex::new(BTreeSet::new()));
    let spied = |archive: ReplayArchive, seen: &Arc<Mutex<BTreeSet<i64>>>| {
        let mut lab = lab_with_holdout(archive);
        lab.signals = vec![
            Arc::new(Spy {
                inner: signals::by_name("rating_calibration").unwrap(),
                seen: seen.clone(),
            }),
            signals::by_name("learned_weights").unwrap(),
        ];
        lab
    };
    let outcome = spied(archive, &seen).run().await.unwrap();
    let h = &outcome.report.holdout;
    assert_eq!(h.status, HoldoutStatus::Evaluated);
    assert!(h.matches > 0);
    assert!(outcome.report.selection.frozen.contains(&"spy".to_string()));
    assert_eq!(h.combination, outcome.report.selection.frozen);
    assert!(h.statement.contains("Evaluated once"), "{}", h.statement);
    assert!(h.statement.contains("after the selection was frozen"));
    assert!(h.played_from.is_some() && h.played_to.is_some());

    // No fit ever got a Holdout Match, at any step, final one included.
    let fitted_on = seen.lock().unwrap().clone();
    assert!(!fitted_on.is_empty());
    assert!(
        fitted_on.is_disjoint(&holdout_ids),
        "a fit saw a Holdout Match"
    );
    // Nothing before the final step reports or predicts a Holdout Match.
    for p in outcome.predictions.iter().chain(&outcome.combination) {
        assert!(!holdout_ids.contains(&p.match_id), "{p:?}");
    }
    let holdout_events: BTreeSet<i64> = outcome.holdout.iter().map(|p| p.event_id).collect();
    for e in &outcome.report.events {
        assert!(
            !e.title.contains("2026"),
            "{} is on the training side",
            e.title
        );
    }
    assert!(!holdout_events.is_empty());

    // Reversing every Holdout result changes nothing but the Holdout's scores:
    // not one verdict, selection step or prediction, Holdout ones included.
    let mut reversed = whole();
    reverse(&mut reversed, "Cologne");
    reverse(&mut reversed, "Singapore");
    let other_seen = Arc::new(Mutex::new(BTreeSet::new()));
    let after = spied(reversed, &other_seen).run().await.unwrap();
    let json = |v: &dyn erased::Ser| v.value();
    assert_eq!(
        json(&outcome.report.candidates),
        json(&after.report.candidates)
    );
    assert_eq!(
        json(&outcome.report.selection),
        json(&after.report.selection)
    );
    assert_eq!(outcome.predictions, after.predictions);
    assert_eq!(outcome.combination, after.combination);
    assert_eq!(
        outcome.holdout, after.holdout,
        "Holdout predictions read no outcome"
    );
    assert_ne!(
        h.difference, after.report.holdout.difference,
        "the Holdout's scores do read its outcomes"
    );

    let md = outcome.report.markdown();
    assert!(md.contains("## Holdout: evaluated"), "{md}");
}

mod erased {
    pub trait Ser {
        fn value(&self) -> serde_json::Value;
    }
    impl<T: serde::Serialize> Ser for T {
        fn value(&self) -> serde_json::Value {
            serde_json::to_value(self).unwrap()
        }
    }
}

#[tokio::test]
async fn with_the_holdout_not_yet_played_the_run_completes_and_says_it_is_pending() {
    // The saved archive ends with Singapore Stage 1, on the training side.
    let lab = Lab::new(whole()).unwrap();
    let outcome = lab.run().await.unwrap();
    let h = &outcome.report.holdout;
    assert_eq!(h.status, HoldoutStatus::Pending);
    assert!(h.stages.is_empty(), "{:?}", h.stages);
    assert_eq!(h.matches, 0);
    assert!(outcome.holdout.is_empty());
    assert!(h.statement.contains("pending"), "{}", h.statement);
    assert!(
        outcome
            .report
            .events
            .iter()
            .any(|e| e.title.contains("Singapore")),
        "Singapore Stage 1 belongs to the training side"
    );
    let md = outcome.report.markdown();
    assert!(md.contains("## Holdout: pending"), "{md}");
    assert!(
        md.contains("2026-11-25 23:46:46 UTC"),
        "the boundary is stated"
    );

    // A Singapore Stage after Stage 1, played: it is the Holdout.
    let mut archive = whole();
    let stage1 = archive
        .stages
        .iter()
        .find(|s| s.id == kallbet_ai::adapters::lab::holdout::SINGAPORE_STAGE_1)
        .expect("Singapore Stage 1 is saved")
        .clone();
    let mut stage2 = stage1.clone();
    stage2.id = 1;
    stage2.name = "Stage 2".into();
    let shift = chrono::Duration::days(3);
    stage2.cutoff += shift;
    for m in &mut stage2.matches {
        m.game.id += 1_000_000;
        m.actual_start += shift;
        m.finished_at += shift;
        m.game.scheduled_start = m.game.scheduled_start.map(|t| t + shift);
    }
    archive.stages.push(stage2);
    let played = Lab::new(archive).unwrap().run().await.unwrap();
    let h = &played.report.holdout;
    assert_eq!(h.status, HoldoutStatus::Evaluated);
    assert_eq!(h.stages.len(), 1, "{:?}", h.stages);
    assert!(h.stages[0].contains("Stage 2"));
    assert_eq!(played.holdout.len(), stage1.matches.len());
}

#[test]
fn the_run_record_says_whether_the_holdout_was_evaluated() {
    let out = tempfile::tempdir().unwrap();
    let run = std::process::Command::new(env!("CARGO_BIN_EXE_kallbet-ai"))
        .arg("lab")
        .arg("--archive")
        .arg(fixtures())
        .arg("--out")
        .arg(out.path())
        .args(["--resamples", "200"])
        .env_remove("SIDECAR_CONFIG")
        .env_remove("ODDSPAPI_API_KEY")
        .output()
        .unwrap();
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    let runs = tokio::runtime::Runtime::new().unwrap().block_on(async {
        let audit = Audit::open(&out.path().join("audit.sqlite")).await.unwrap();
        audit.lab_runs().await.unwrap()
    });
    assert_eq!(runs[0].holdout.as_deref(), Some("pending"));
    let json: Value =
        serde_json::from_slice(&std::fs::read(out.path().join("report.json")).unwrap()).unwrap();
    assert_eq!(json["holdout"]["status"], "pending");
    assert!(json["selection"]["steps"].is_array());
}

/// Records every training Match each fit is given.
struct Recorder {
    inner: Arc<dyn CandidateSignal>,
    fitted_on: Arc<Mutex<BTreeSet<i64>>>,
}
impl CandidateSignal for Recorder {
    fn name(&self) -> &'static str {
        "recorder"
    }
    fn description(&self) -> &'static str {
        "the rating calibration, recording its training Matches"
    }
    fn needs(&self) -> &'static str {
        self.inner.needs()
    }
    fn fit(&self, training: &[Observed]) -> Result<Box<dyn FittedSignal>> {
        self.fitted_on
            .lock()
            .unwrap()
            .extend(training.iter().map(|o| o.view.game.id));
        self.inner.fit(training)
    }
}

fn recorded(lab: &mut Lab) -> Arc<Mutex<BTreeSet<i64>>> {
    let fitted_on = Arc::new(Mutex::new(BTreeSet::new()));
    lab.signals = vec![Arc::new(Recorder {
        inner: signals::by_name("rating_calibration").unwrap(),
        fitted_on: fitted_on.clone(),
    })];
    fitted_on
}

const AUSTIN: i64 = 9309;

fn event_matches(archive: &ReplayArchive, event: i64) -> BTreeSet<i64> {
    archive
        .stages
        .iter()
        .filter(|s| s.event_id == event)
        .flat_map(|s| s.matches.iter().map(|m| m.game.id))
        .collect()
}

// Finding 9 (review of 6eea194): only complete Events train. An Event with a
// Stage still in play, or a result whose finish is unknown, has no known end:
// it trains no other Event, and as its last result may fall after the
// boundary, it is not on the training side either.
#[tokio::test]
async fn an_event_without_a_known_end_trains_nothing() {
    for break_it in [
        (|a: &mut ReplayArchive| {
            a.stages.iter_mut().find(|s| s.id == 16725).unwrap().complete = false
        }) as fn(&mut ReplayArchive),
        |a: &mut ReplayArchive| {
            let s = a.stages.iter_mut().find(|s| s.id == 16658).unwrap();
            s.matches[0].reported_finish = None;
        },
    ] {
        let mut archive = whole();
        break_it(&mut archive);
        let austin = event_matches(&archive, AUSTIN);
        let mut lab = Lab::new(archive).unwrap();
        let fitted_on = recorded(&mut lab);
        let outcome = lab.run().await.unwrap();
        assert!(fitted_on.lock().unwrap().is_disjoint(&austin));
        assert!(outcome.report.events.iter().all(|e| e.id != AUSTIN));
        assert!(
            outcome.report.holdout.stages.iter().any(|s| s.contains("Austin")),
            "{:?}",
            outcome.report.holdout.stages
        );
    }
}

/// The saved archive with Singapore Stage 1 complete and a played Stage 2
/// three days later (the Holdout), both with reported finishes.
fn singapore_with_stage_2() -> (ReplayArchive, BTreeSet<i64>) {
    let mut archive = whole();
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
        m.game.id += 1_000_000;
        m.actual_start += shift;
        m.finished_at += shift;
        m.reported_finish = m.reported_finish.map(|t| t + shift);
        m.game.scheduled_start = m.game.scheduled_start.map(|t| t + shift);
    }
    let singapore = stage1.event_id;
    archive.stages.push(stage2);
    let ids = event_matches(&archive, singapore);
    (archive, ids)
}

// Finding 3 (review of 6eea194): the Holdout's later Stages of an Event are
// fitted without any Match of that Event, though Stage 1 is on the training
// side and ended before Stage 2 began.
#[tokio::test]
async fn no_stage_of_the_evaluated_event_trains_it() {
    let (archive, singapore) = singapore_with_stage_2();
    let training_side: usize = archive
        .stages
        .iter()
        .filter(|s| s.id != 1)
        .map(|s| s.matches.len())
        .sum();
    let in_singapore_1 = singapore.len() / 2;
    let mut lab = Lab::new(archive).unwrap();
    let fitted_on = recorded(&mut lab);
    let outcome = lab.run().await.unwrap();
    let h = &outcome.report.holdout;
    assert_eq!(h.status, HoldoutStatus::Evaluated);
    for e in &h.per_event {
        assert_eq!(e.training_matches, training_side - in_singapore_1, "{}", e.title);
    }
    assert!(fitted_on.lock().unwrap().is_disjoint(&singapore));
}

// Finding 2 (review of 6eea194): a Holdout result never reaches a training
// view, also not nested in the results history a signal reads. Cologne
// begins before the boundary and ends after it, so it is all in the Holdout;
// Singapore Stage 1 stays on the training side and is played after Cologne,
// whose results would otherwise be in its history.
#[tokio::test]
async fn holdout_results_reach_no_training_view_through_the_history() {
    let run = |reverse: bool| async move {
        let mut archive = whole();
        let boundary = archive.stages.iter().find(|s| s.id == 20709).unwrap().cutoff;
        if reverse {
            for s in archive.stages.iter_mut().filter(|s| s.event_id == 10488) {
                for m in &mut s.matches {
                    m.winner = m.loser();
                    m.maps_score = m.maps_score.map(|(a, b)| (b, a));
                }
            }
        }
        let mut lab = Lab::new(archive).unwrap();
        lab.holdout = Holdout {
            from: boundary,
            training_stages: vec![kallbet_ai::adapters::lab::holdout::SINGAPORE_STAGE_1],
        };
        lab.signals = ["own_rating", "recent_form", "head_to_head"]
            .into_iter()
            .map(|n| signals::by_name(n).unwrap())
            .collect();
        lab.run().await.unwrap()
    };
    let (plain, reversed) = (run(false).await, run(true).await);
    assert!(plain.report.events.iter().any(|e| e.title.contains("Singapore")));
    assert!(plain.report.events.iter().all(|e| !e.title.contains("Cologne")));
    let singapore: Vec<_> = plain
        .predictions
        .iter()
        .filter(|p| p.variant.is_some() && p.event_id == 11176)
        .collect();
    assert!(!singapore.is_empty());
    assert_eq!(plain.predictions, reversed.predictions);
}

/// The replay's Valve rating, failing for the Matches in `broken`.
struct BrokenFor {
    inner: Arc<dyn SignalEstimator>,
    broken: BTreeSet<i64>,
}
#[async_trait::async_trait]
impl SignalEstimator for BrokenFor {
    async fn signal(&self, game: &Match, cutoff: Instant) -> Result<Option<AsOf<Signal>>> {
        if self.broken.contains(&game.id) {
            return Err("the rating port is down for this Match".into());
        }
        self.inner.signal(game, cutoff).await
    }
}

// Finding 20 (review of 6eea194): the final step is the first to read the
// Holdout. A port failing only for a Holdout Match does not stop the
// training side's evaluation and selection, and what a Candidate Signal is
// told the archive holds (`missing_data`) is the training side only.
#[tokio::test]
async fn the_holdout_is_read_only_by_the_final_step() {
    let archive = whole();
    let mut lab = lab_with_holdout(archive);
    let holdout = lab.holdout_matches(&lab.holdout_stages());
    assert!(!holdout.is_empty());
    let replay = kallbet_ai::adapters::backtest::Feeds::replay(&lab.archive).unwrap();
    lab.feeds = kallbet_ai::adapters::backtest::Feeds {
        signal: Arc::new(BrokenFor {
            inner: replay.signal.clone(),
            broken: holdout,
        }),
        ..replay
    };
    let fitted_on = recorded(&mut lab);
    let err = lab.run().await.err().expect("the final step fails");
    assert!(err.to_string().contains("port is down"), "{err}");
    assert!(!fitted_on.lock().unwrap().is_empty(), "the training side ran first");

    // One training Match: head-to-head has no history to read, whatever the
    // Holdout holds.
    let mut archive = whole();
    let first = archive.stages.iter_mut().find(|s| s.id == 13056).unwrap();
    first.matches.truncate(1);
    let mut lab = Lab::new(archive).unwrap();
    lab.holdout = Holdout {
        from: "2000-01-01T00:00:00Z".parse().unwrap(),
        training_stages: vec![13056],
    };
    lab.signals = vec![signals::by_name("head_to_head").unwrap()];
    let outcome = lab.run().await.unwrap();
    let c = candidate(&outcome, "head_to_head");
    assert_eq!(c.verdict, Verdict::NotTestable);
    assert!(c.reason.contains("fewer than two"), "{}", c.reason);
}

// Finding 10 (review of 6eea194): once a Holdout evaluation is recorded, a
// later run reads the Holdout again only to reproduce it: the same archive
// and the same frozen combination. Any other run is refused before it reads
// a Holdout Match.
#[tokio::test]
async fn a_recorded_holdout_evaluation_is_only_reproduced() {
    let first = lab_with_holdout(whole()).run().await.unwrap();
    assert_eq!(first.report.holdout.status, HoldoutStatus::Evaluated);
    let record = kallbet_ai::adapters::lab::runs::holdout_record(&first.report);
    let evaluated_at: Instant = "2026-12-03T10:00:00Z".parse().unwrap();
    let freeze = |digest: &str, combination: &str| kallbet_ai::adapters::lab::Freeze {
        archive_sha256: digest.into(),
        holdout: combination.into(),
        evaluated_at,
    };

    let mut again = lab_with_holdout(whole());
    again.freeze = Some(freeze(&first.report.archive_digest, &record));
    let reproduced = again.run().await.unwrap();
    assert_eq!(reproduced.holdout, first.holdout);

    for (digest, combination) in [
        ("another archive", record.as_str()),
        (first.report.archive_digest.as_str(), "evaluated: something_else"),
    ] {
        let mut other = lab_with_holdout(whole());
        other.freeze = Some(freeze(digest, combination));
        let err = other.run().await.err().expect("refused");
        assert!(err.to_string().contains("already evaluated"), "{err}");
    }
    // A selection that changes refuses too: here, no promoted signal.
    let mut fewer = lab_with_holdout(whole());
    fewer.signals = vec![signals::by_name("head_to_head").unwrap()];
    fewer.freeze = Some(freeze(&first.report.archive_digest, &record));
    assert!(fewer.run().await.is_err());
}
