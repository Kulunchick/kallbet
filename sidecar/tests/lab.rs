//! Seam 1 of ticket #34: the Signal Lab's runner over the saved replay archive
//! (`tests/fixtures/backtest/`, provenance in its README). The protocol of spec
//! #33 is observed through what a caller sees: the report's numbers and
//! verdicts, the per-Match predictions, an abort on leaked data, the run record.
use chrono::Duration;
use kallbet_ai::{
    adapters::{
        backtest::{Feeds, ReplayArchive},
        lab::{
            CandidateSignal, FittedSignal, Lab, LabOutcome, MatchView, Observed, PROTOCOL_VERSION,
            signals,
            stats::{self, Verdict},
        },
    },
    *,
};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    sync::Arc,
};

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/backtest")
}

fn archive_of(prefixes: &[&str]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    for entry in std::fs::read_dir(fixtures()).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        if prefixes.iter().any(|p| name.starts_with(p)) {
            std::fs::copy(&path, dir.path().join(&name)).unwrap();
        }
    }
    dir
}

fn whole() -> ReplayArchive {
    ReplayArchive::load(&fixtures()).unwrap()
}

fn calibration(outcome: &LabOutcome) -> &kallbet_ai::adapters::lab::CandidateReport {
    outcome
        .report
        .candidates
        .iter()
        .find(|c| c.name == "rating_calibration")
        .expect("the calibration Candidate Signal is reported")
}

#[tokio::test]
async fn the_report_compares_the_calibration_with_the_anchor_under_the_protocol() {
    let lab = Lab::new(whole()).unwrap();
    let outcome = lab.run().await.unwrap();
    let report = &outcome.report;
    assert_eq!(report.protocol_version, PROTOCOL_VERSION);
    assert_eq!(report.archive_digest, lab.archive.digest);
    assert_eq!(report.seed, lab.protocol.seed);

    let c = calibration(&outcome);
    // Recompute every number of the row from the per-Match predictions.
    let winners: BTreeMap<i64, (i64, bool)> = lab
        .archive
        .stages
        .iter()
        .flat_map(|s| &s.matches)
        .map(|m| (m.game.id, (m.game.event_id, m.winner == m.game.team_a.id)))
        .collect();
    let rows: Vec<_> = outcome
        .predictions
        .iter()
        .filter(|p| p.signal == "rating_calibration")
        .collect();
    assert_eq!(rows.len(), winners.len(), "every scored Match is listed");
    let computed = rows.iter().filter(|p| p.variant.is_some()).count();
    assert_eq!(c.computed_on, computed);
    let paired: Vec<_> = rows
        .iter()
        .filter_map(|p| Some((p.match_id, p.baseline?, p.variant?)))
        .collect();
    assert_eq!(c.paired, paired.len());
    assert!(c.paired > 100, "{}", c.paired);
    let mean = |f: &dyn Fn(f64, bool) -> f64, pick: usize| {
        paired
            .iter()
            .map(|(id, b, v)| f(if pick == 0 { *b } else { *v }, winners[id].1))
            .sum::<f64>()
            / paired.len() as f64
    };
    let close = |a: f64, b: f64| (a - b).abs() < 1e-12;
    let ll = |p, w| stats::log_loss(p, w);
    let br = |p, w| stats::brier(p, w);
    let acc = |p, w| f64::from(u8::from(stats::correct(p, w)));
    let (baseline, variant, difference) = (
        c.baseline.unwrap(),
        c.variant.unwrap(),
        c.difference.unwrap(),
    );
    assert!(close(baseline.log_loss, mean(&ll, 0)));
    assert!(close(variant.log_loss, mean(&ll, 1)));
    assert!(close(difference.log_loss, mean(&ll, 1) - mean(&ll, 0)));
    assert!(close(difference.brier, mean(&br, 1) - mean(&br, 0)));
    assert!(close(difference.accuracy, mean(&acc, 1) - mean(&acc, 0)));

    // The 90% interval is the Event bootstrap of those differences, with the
    // recorded seed; the detectable effect comes from its standard error.
    let diffs: Vec<(i64, f64)> = paired
        .iter()
        .map(|(id, b, v)| {
            let (event, won) = winners[id];
            (event, ll(*v, won) - ll(*b, won))
        })
        .collect();
    let boot = stats::event_bootstrap(
        &diffs,
        lab.protocol.resamples,
        lab.protocol.level,
        lab.protocol.seed,
    )
    .unwrap();
    let interval = c.interval.unwrap();
    assert_eq!(interval, boot.interval);
    assert_eq!(c.detectable, Some(stats::detectable_effect(boot.std_error)));
    let events: BTreeSet<i64> = diffs.iter().map(|d| d.0).collect();
    assert_eq!(c.events, events.len());

    // The verdict is the written rule applied to those numbers.
    let (verdict, reason) = stats::verdict(
        c.paired,
        interval,
        c.market.as_ref().map(|m| m.log_loss_difference),
        &lab.protocol,
    );
    assert_eq!(c.verdict, verdict);
    assert_eq!(c.reason, reason);

    // The markdown names the run's identity and the row.
    let md = report.markdown();
    for needle in [
        report.archive_digest.as_str(),
        PROTOCOL_VERSION,
        &format!("seed {}", lab.protocol.seed),
        "rating_calibration",
    ] {
        assert!(md.contains(needle), "{needle} missing:\n{md}");
    }
}

/// A test Candidate Signal that predicts 0.5 dated `offset` after each cutoff.
struct Leaky(Duration);
struct LeakyFit(Duration);
impl CandidateSignal for Leaky {
    fn name(&self) -> &'static str {
        "leaky"
    }
    fn description(&self) -> &'static str {
        "reads a datum dated relative to the Match cutoff"
    }
    fn needs(&self) -> &'static str {
        "nothing"
    }
    fn fit(&self, _: &[Observed]) -> Result<Box<dyn FittedSignal>> {
        Ok(Box::new(LeakyFit(self.0)))
    }
}
impl FittedSignal for LeakyFit {
    fn predict(&self, m: &MatchView) -> Result<Option<AsOf<f64>>> {
        let observed = m.cutoff + self.0;
        Ok(Some(AsOf::new(
            0.5,
            observed,
            observed + Duration::days(365),
        )?))
    }
}

fn singapore() -> tempfile::TempDir {
    archive_of(&[
        "panda-22378",
        "standings_global_2026_11_02",
        "oddspapi-fixtures-pgl-singapore",
        "od1740001",
    ])
}

#[tokio::test]
async fn a_candidate_signal_reading_a_datum_at_or_after_the_cutoff_aborts_the_run() {
    let dir = singapore();
    for offset in [Duration::zero(), Duration::seconds(1), Duration::hours(2)] {
        let mut lab = Lab::new(ReplayArchive::load(dir.path()).unwrap()).unwrap();
        lab.signals = vec![Arc::new(Leaky(offset))];
        let err = lab.run().await.err().expect("the run aborts");
        let leak = err
            .downcast_ref::<TemporalLeakError>()
            .unwrap_or_else(|| panic!("not a TemporalLeakError: {err}"));
        assert_eq!(leak.observed_at - leak.cutoff, offset);
    }
    let mut lab = Lab::new(ReplayArchive::load(dir.path()).unwrap()).unwrap();
    lab.signals = vec![Arc::new(Leaky(-Duration::seconds(1)))];
    lab.run()
        .await
        .expect("a datum observed before the cutoff is accepted");
}

/// A Valve rating observed at the cutoff of every Match: the lab reads it
/// through the Signal Estimator port, and the anchor inputs abort as well.
struct LateRating;
#[async_trait::async_trait]
impl SignalEstimator for LateRating {
    async fn signal(&self, _: &Match, cutoff: Instant) -> Result<Option<AsOf<Signal>>> {
        let s = Signal {
            map: 0.6,
            series: 0.6,
        };
        Ok(Some(AsOf::new(s, cutoff, cutoff + Duration::days(1))?))
    }
}

#[tokio::test]
async fn a_port_datum_at_the_cutoff_aborts_the_run_too() {
    let dir = singapore();
    let archive = ReplayArchive::load(dir.path()).unwrap();
    let mut lab = Lab::new(archive).unwrap();
    let replay = Feeds::replay(&lab.archive).unwrap();
    lab.feeds = Feeds {
        signal: Arc::new(LateRating),
        ..replay
    };
    let err = lab.run().await.err().expect("the run aborts");
    assert!(err.is::<TemporalLeakError>(), "{err}");
}

#[tokio::test]
async fn a_result_injected_into_a_later_event_changes_nothing_in_an_earlier_one() {
    let before = Lab::new(whole()).unwrap();
    let base = before.run().await.unwrap();

    // Reverse every result of IEM Cologne Major 2026, the Event before Singapore.
    let mut archive = whole();
    let cologne = archive
        .stages
        .iter()
        .find(|s| s.event.contains("Cologne") && s.event.contains("2026"))
        .unwrap()
        .event_id;
    let cologne_start = archive
        .stages
        .iter()
        .filter(|s| s.event_id == cologne)
        .map(|s| s.cutoff)
        .min()
        .unwrap();
    for stage in archive.stages.iter_mut().filter(|s| s.event_id == cologne) {
        for m in &mut stage.matches {
            m.winner = if m.winner == m.game.team_a.id {
                m.game.team_b.id
            } else {
                m.game.team_a.id
            };
        }
    }
    let starts: BTreeMap<i64, Instant> =
        archive.stages.iter().fold(BTreeMap::new(), |mut acc, s| {
            let e = acc.entry(s.event_id).or_insert(s.cutoff);
            *e = (*e).min(s.cutoff);
            acc
        });
    let after = Lab::new(archive).unwrap().run().await.unwrap();

    type ByMatch = BTreeMap<(String, i64), (i64, Option<f64>, Option<f64>)>;
    let key = |o: &LabOutcome| -> ByMatch {
        o.predictions
            .iter()
            .map(|p| {
                (
                    (p.signal.clone(), p.match_id),
                    (p.event_id, p.baseline, p.variant),
                )
            })
            .collect()
    };
    let (base, after) = (key(&base), key(&after));
    assert_eq!(base.len(), after.len());
    let mut later_changed = 0;
    for (k, (event, b_base, v_base)) in &base {
        let (_, b_after, v_after) = &after[k];
        assert_eq!(b_base, b_after, "the anchor never fits");
        if starts[event] <= cologne_start {
            // Cologne itself and every earlier Event: not one probability moves.
            assert_eq!(v_base, v_after, "{k:?} moved");
        } else if v_base != v_after {
            later_changed += 1;
        }
    }
    assert!(
        later_changed > 0,
        "Singapore is fitted on Cologne, so reversing Cologne must move it"
    );
}

/// A Candidate Signal that never predicts.
struct Silent;
struct SilentFit;
impl CandidateSignal for Silent {
    fn name(&self) -> &'static str {
        "silent"
    }
    fn description(&self) -> &'static str {
        "never has its data"
    }
    fn needs(&self) -> &'static str {
        "a datum no saved response carries"
    }
    fn fit(&self, _: &[Observed]) -> Result<Box<dyn FittedSignal>> {
        Ok(Box::new(SilentFit))
    }
}
impl FittedSignal for SilentFit {
    fn predict(&self, _: &MatchView) -> Result<Option<AsOf<f64>>> {
        Ok(None)
    }
}

#[tokio::test]
async fn a_candidate_signal_without_its_data_is_reported_not_testable_with_the_reason() {
    // No Valve standings saved: the calibration cannot be computed at all.
    let dir = archive_of(&[
        "panda-22378",
        "oddspapi-fixtures-pgl-singapore",
        "od1740001",
    ]);
    let mut lab = Lab::new(ReplayArchive::load(dir.path()).unwrap()).unwrap();
    lab.signals.push(Arc::new(Silent));
    let outcome = lab.run().await.unwrap();
    let names: Vec<_> = outcome
        .report
        .candidates
        .iter()
        .map(|c| c.name.as_str())
        .collect();
    assert_eq!(
        names.len(),
        signals::registry().len() + 1,
        "none is left out"
    );
    assert_eq!(names.last(), Some(&"silent"));
    let c = calibration(&outcome);
    assert_eq!(c.verdict, Verdict::NotTestable);
    assert!(c.reason.contains("Valve standings"), "{}", c.reason);
    let silent = outcome.report.candidates.last().unwrap();
    assert_eq!(silent.verdict, Verdict::NotTestable);
    assert!(
        silent.reason.contains("a datum no saved response carries"),
        "{}",
        silent.reason
    );
    assert_eq!(silent.computed_on, 0);
    let md = outcome.report.markdown();
    assert!(md.contains("not testable"), "{md}");
}

#[test]
fn every_registered_candidate_signal_has_a_distinct_name() {
    let names: Vec<_> = signals::registry().iter().map(|s| s.name()).collect();
    assert!(names.contains(&"rating_calibration"));
    let unique: BTreeSet<_> = names.iter().collect();
    assert_eq!(unique.len(), names.len());
    assert!(signals::by_name("rating_calibration").is_some());
    assert!(signals::by_name("nope").is_none());
}

fn run_lab(archive: &Path, out: &Path, extra: &[&str]) -> std::process::Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_kallbet-ai"))
        .arg("lab")
        .arg("--archive")
        .arg(archive)
        .arg("--out")
        .arg(out)
        .args(extra)
        .env_remove("SIDECAR_CONFIG")
        .env_remove("ODDSPAPI_API_KEY")
        .output()
        .unwrap()
}

#[test]
fn the_lab_subcommand_writes_a_reproducible_report_and_records_each_run() {
    let (one, two) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    for out in [&one, &two] {
        let run = run_lab(&fixtures(), out.path(), &["--seed", "7"]);
        assert!(
            run.status.success(),
            "{}",
            String::from_utf8_lossy(&run.stderr)
        );
    }
    let read = |dir: &tempfile::TempDir, f: &str| std::fs::read(dir.path().join(f)).unwrap();
    assert_eq!(read(&one, "report.md"), read(&two, "report.md"));
    assert_eq!(read(&one, "report.json"), read(&two, "report.json"));

    let json: Value = serde_json::from_slice(&read(&one, "report.json")).unwrap();
    assert_eq!(json["protocol_version"], PROTOCOL_VERSION);
    assert_eq!(json["seed"], 7);
    let digest = json["archive_digest"].as_str().unwrap().to_owned();
    assert_eq!(digest.len(), 64);
    let md = String::from_utf8(read(&one, "report.md")).unwrap();
    assert!(md.contains(&digest) && md.contains("seed 7"), "{md}");
    let c = json["candidates"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "rating_calibration")
        .unwrap();
    for field in [
        "computed_on",
        "paired",
        "difference",
        "interval",
        "detectable",
        "verdict",
    ] {
        assert!(!c[field].is_null(), "{field}: {c}");
    }

    // A second run into the same directory is appended to the same audit store.
    let again = run_lab(&fixtures(), one.path(), &["--seed", "8"]);
    assert!(
        again.status.success(),
        "{}",
        String::from_utf8_lossy(&again.stderr)
    );
    let runs = tokio::runtime::Runtime::new().unwrap().block_on(async {
        let audit = Audit::open(&one.path().join("audit.sqlite")).await.unwrap();
        audit.lab_runs().await.unwrap()
    });
    assert_eq!(runs.len(), 2);
    assert_eq!(runs[0].archive_sha256, digest);
    assert_eq!(runs[0].protocol_version, PROTOCOL_VERSION);
    assert_eq!((runs[0].seed, runs[1].seed), (7, 8));
    assert!(runs[0].started_at <= runs[0].finished_at);
    assert_eq!(
        runs[0].verdicts["rating_calibration"], c["verdict"],
        "{:?}",
        runs[0].verdicts
    );

    let unknown = run_lab(&fixtures(), one.path(), &["--signal", "nope"]);
    assert!(!unknown.status.success());
    assert!(String::from_utf8_lossy(&unknown.stderr).contains("rating_calibration"));
}
