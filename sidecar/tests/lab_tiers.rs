//! Seam 1 of ticket #55 (spec #33): the tier-1 slice of the Signal Lab report
//! and the `--tiers` / `--train-tiers` filters, over the saved replay archive
//! (`tests/fixtures/backtest/`, every Event of which is PandaScore tier "s").
use kallbet_ai::{
    adapters::{
        backtest::ReplayArchive,
        lab::{
            CandidateSignal, FittedSignal, Holdout, Lab, Observed, TierFilter, signals,
        },
    },
    *,
};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/backtest")
}

/// Shanghai is tier "a", Austin has no tier, the rest "s"; the Holdout begins
/// with IEM Cologne Major 2026.
fn lab() -> Lab {
    let mut archive = ReplayArchive::load(&fixtures()).unwrap();
    for stage in &mut archive.stages {
        if stage.event.contains("Shanghai") {
            stage.tier = Some("a".into());
        }
        if stage.event.contains("Austin") {
            stage.tier = None;
        }
    }
    let mut lab = Lab::new(archive).unwrap();
    lab.holdout = Holdout {
        from: lab
            .archive
            .stages
            .iter()
            .filter(|s| s.event.contains("Cologne") && s.event.contains("2026"))
            .map(|s| s.cutoff)
            .min()
            .unwrap(),
        training_stages: vec![],
    };
    lab.signals = vec![signals::by_name("rating_calibration").unwrap()];
    lab
}

fn matches_of(lab: &Lab, pick: impl Fn(&str, &Option<String>) -> bool) -> BTreeSet<i64> {
    lab.archive
        .stages
        .iter()
        .filter(|s| pick(&s.event, &s.tier))
        .flat_map(|s| s.matches.iter().map(|m| m.game.id))
        .collect()
}

#[test]
fn the_archive_loader_reads_the_tier_of_each_stage() {
    let archive = ReplayArchive::load(&fixtures()).unwrap();
    assert!(archive.stages.iter().all(|s| s.tier.as_deref() == Some("s")));
}

#[test]
fn tier_lists_parse_lowercase_and_refuse_an_empty_one() {
    let set = TierFilter::parse("S, a").unwrap();
    assert_eq!(set.into_iter().collect::<Vec<_>>(), ["a", "s"]);
    assert!(TierFilter::parse(" , ").is_err());
}

#[tokio::test]
async fn the_report_has_a_tier_s_slice_with_counts_and_names_the_untiered_event() {
    let lab = lab();
    let outcome = lab.run().await.unwrap();
    let view = &outcome.report.tier_view;
    assert_eq!(view.slice, "s");
    assert_eq!(view.evaluated, "all");
    let in_tier = |t: &str| {
        matches_of(&lab, |e, tier| tier.as_deref() == Some(t) && !e.contains("2026")).len()
    };
    assert_eq!(view.matches_by_tier["a"], in_tier("a"));
    assert_eq!(view.matches_by_tier["s"], in_tier("s"));
    assert_eq!(view.untiered.len(), 1, "{:?}", view.untiered);
    assert!(view.untiered[0].contains("Austin"), "{:?}", view.untiered);
    // The slice drops the tier-A and the untiered Events from the evaluation.
    let full = &outcome.report.candidates[0];
    let slice = &view.candidates[0];
    assert!(slice.events < full.events, "{} vs {}", slice.events, full.events);
    assert!(slice.paired < full.paired);
    let md = outcome.report.markdown();
    assert!(md.contains("Tier slice `s` (secondary view)"), "{md}");
    assert!(md.contains("secondary view"), "{md}");
    assert!(md.contains("Austin"), "{md}");
    assert!(md.contains("| Candidate Signal | Verdict (full) |"), "{md}");
    // The main table is still the full archive: tiered or not, every Event.
    assert_eq!(slice.events, 1);
}

#[tokio::test]
async fn tiers_restricts_evaluation_and_leaves_the_event_without_a_tier_reported() {
    let mut lab = lab();
    lab.tiers.evaluate = Some(TierFilter::parse("a").unwrap());
    let outcome = lab.run().await.unwrap();
    let shanghai = matches_of(&lab, |e, _| e.contains("Shanghai"));
    let evaluated: BTreeSet<i64> = outcome.predictions.iter().map(|p| p.match_id).collect();
    assert!(!evaluated.is_empty());
    assert!(evaluated.is_subset(&shanghai));
    assert_eq!(outcome.report.tier_view.evaluated, "a");
    assert_eq!(outcome.report.tier_view.untiered.len(), 1);
}

/// Wraps a Candidate Signal and records the Matches of each fit.
struct Spy {
    inner: Arc<dyn CandidateSignal>,
    fits: Arc<Mutex<Vec<BTreeSet<i64>>>>,
}
impl CandidateSignal for Spy {
    fn name(&self) -> &'static str {
        "spy"
    }
    fn description(&self) -> &'static str {
        "records what it is fitted on"
    }
    fn needs(&self) -> &'static str {
        self.inner.needs()
    }
    fn fit(&self, training: &[Observed]) -> Result<Box<dyn FittedSignal>> {
        self.fits
            .lock()
            .unwrap()
            .push(training.iter().map(|o| o.view.game.id).collect());
        self.inner.fit(training)
    }
}

#[tokio::test]
async fn the_tier_filter_cannot_let_a_later_or_holdout_match_into_fitting() {
    let fits = Arc::new(Mutex::new(Vec::new()));
    let mut lab = lab();
    lab.signals = vec![Arc::new(Spy {
        inner: signals::by_name("rating_calibration").unwrap(),
        fits: fits.clone(),
    })];
    // Evaluate tier "s" only, fit on tier "s" only.
    lab.tiers = TierFilter {
        evaluate: Some(TierFilter::parse("s").unwrap()),
        train: Some(TierFilter::parse("s").unwrap()),
    };
    lab.run().await.unwrap();
    let holdout = matches_of(&lab, |e, _| e.contains("2026"));
    let wrong_tier = matches_of(&lab, |e, _| e.contains("Shanghai") || e.contains("Austin"));
    let latest = matches_of(&lab, |e, _| e.contains("Budapest"));
    assert!(!holdout.is_empty() && !wrong_tier.is_empty() && !latest.is_empty());
    let seen = fits.lock().unwrap().clone();
    let fits = seen;
    assert!(!fits.is_empty());
    for fit in fits.iter() {
        assert!(fit.is_disjoint(&holdout), "a fit saw a Holdout Match");
        assert!(fit.is_disjoint(&wrong_tier), "a fit saw an Event outside --train-tiers");
        // Budapest is the last training-side Event: no one trains on it.
        assert!(fit.is_disjoint(&latest), "a fit saw the last Event's Matches");
    }
    // A fit for a later Event holds the earlier Events and never its own.
    let copenhagen = matches_of(&lab, |e, _| e.contains("Copenhagen"));
    assert!(fits.iter().any(|f| copenhagen.is_subset(f)));

    // Training on every tier admits the others but still not the Holdout.
    let all = Arc::new(Mutex::new(Vec::new()));
    let mut open = lab;
    open.signals = vec![Arc::new(Spy {
        inner: signals::by_name("rating_calibration").unwrap(),
        fits: all.clone(),
    })];
    open.tiers.train = None;
    open.run().await.unwrap();
    let all = all.lock().unwrap().clone();
    assert!(all.iter().any(|f| !f.is_disjoint(&wrong_tier)));
    assert!(all.iter().all(|f| f.is_disjoint(&holdout)));
}
