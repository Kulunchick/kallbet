//! Ticket (spec #33, operator decision): qualifiers and Major RMRs are neither
//! scored nor fitted on (`signal-lab/4`), yet their results stay in the
//! histories, so ratings still update after them.
use kallbet_ai::{
    adapters::{
        backtest::ReplayArchive,
        lab::{CandidateSignal, FittedSignal, Holdout, Lab, MatchView, Observed, is_qualifier},
    },
    *,
};
use std::{
    collections::BTreeSet,
    path::Path,
    sync::{Arc, Mutex},
};

#[test]
fn the_name_rule_classifies_the_five_research_archive_qualifiers() {
    for name in [
        "Perfect World Major Shanghai American RMR",
        "Perfect World Major Shanghai European RMR A 2024",
        "Perfect World Major Shanghai European RMR B 2024",
        "BLAST.tv Major Austin European Regional Qualifier 2025",
        "BLAST Premier Bounty Closed Qualifier 2025",
        "blast bounty closed QUALIFIER",
    ] {
        assert!(is_qualifier(name), "{name}");
    }
    for name in [
        "PGL Major Copenhagen 2024",
        "BLAST.tv Major Austin 2025",
        "IEM Cologne Major 2026",
        "PGL Major Singapore 2026",
        "Perfect World Major Shanghai 2024",
        "BLAST Premier Bounty 2025",
    ] {
        assert!(!is_qualifier(name), "{name}");
    }
}

#[derive(Default)]
struct Seen {
    /// Every fit's training Matches, in call order.
    fits: Vec<BTreeSet<i64>>,
    /// Every scored Match with the results its history held, in call order.
    predicts: Vec<(i64, BTreeSet<i64>)>,
}
struct Spy(Arc<Mutex<Seen>>);
struct SpyFit(Arc<Mutex<Seen>>);
impl CandidateSignal for Spy {
    fn name(&self) -> &'static str {
        "spy"
    }
    fn description(&self) -> &'static str {
        "records what it is fitted on and shown"
    }
    fn needs(&self) -> &'static str {
        "nothing"
    }
    fn fit(&self, training: &[Observed]) -> Result<Box<dyn FittedSignal>> {
        self.0
            .lock()
            .unwrap()
            .fits
            .push(training.iter().map(|o| o.view.game.id).collect());
        Ok(Box::new(SpyFit(self.0.clone())))
    }
}
impl FittedSignal for SpyFit {
    fn predict(&self, m: &MatchView) -> Result<Option<AsOf<f64>>> {
        let mut seen = self.0.lock().unwrap();
        let ids = m.history_before()?.iter().map(|h| h.match_id).collect();
        seen.predicts.push((m.game.id, ids));
        Ok(None)
    }
}

/// What the primary evaluation saw (the first calls: the secondary
/// with-qualifiers view runs after it), the qualifier Matches, the report.
async fn run(include: bool) -> (Seen, BTreeSet<i64>, kallbet_ai::adapters::lab::LabReport) {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/backtest");
    let mut archive = ReplayArchive::load(&dir).unwrap();
    for stage in &mut archive.stages {
        if stage.event.contains("Shanghai") {
            stage.event = format!("{} European RMR A", stage.event);
        }
    }
    let qualifier: BTreeSet<i64> = archive
        .stages
        .iter()
        .filter(|s| is_qualifier(&s.event))
        .flat_map(|s| s.matches.iter().map(|m| m.game.id))
        .collect();
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
    let seen = Arc::new(Mutex::new(Seen::default()));
    lab.signals = vec![Arc::new(Spy(seen.clone()))];
    lab.include_qualifiers = include;
    let outcome = lab.run().await.unwrap();
    let mut seen = std::mem::take(&mut *seen.lock().unwrap());
    if !include {
        // One fit per Event, then one prediction per Match, of the primary run.
        seen.fits.truncate(outcome.report.events.len());
        seen.predicts.truncate(outcome.predictions.len());
    }
    (seen, qualifier, outcome.report)
}

#[tokio::test]
async fn a_qualifier_match_is_neither_scored_nor_fitted_on_but_updates_the_history() {
    let (seen, qualifier, report) = run(false).await;
    assert!(!qualifier.is_empty());
    let scored: BTreeSet<i64> = seen.predicts.iter().map(|p| p.0).collect();
    let fitted: BTreeSet<i64> = seen.fits.iter().flatten().copied().collect();
    let history: BTreeSet<i64> = seen.predicts.iter().flat_map(|p| p.1.clone()).collect();
    assert!(!scored.is_empty() && !fitted.is_empty());
    assert!(scored.is_disjoint(&qualifier), "a qualifier Match was scored");
    assert!(fitted.is_disjoint(&qualifier), "a qualifier Match was fitted on");
    assert!(
        qualifier.is_subset(&history),
        "the qualifier results are in the history later Matches read"
    );
    assert_eq!(report.protocol_version, "signal-lab/4");
    let q = &report.qualifiers;
    assert!(!q.included);
    assert_eq!(q.excluded.len(), 1);
    assert_eq!(q.excluded[0].matches, qualifier.len());
    assert!(report.events.iter().all(|e| !is_qualifier(&e.title)));
    let md = report.markdown();
    assert!(md.contains("## Qualifiers and RMRs"), "{md}");
    assert!(md.contains("RMR` or `Qualifier"), "{md}");
    assert!(md.contains(&q.excluded[0].title), "{md}");
    // The secondary column: the same signal with the qualifiers scored.
    assert!(md.contains("With qualifiers (secondary view)"), "{md}");
    assert_eq!(q.with_qualifiers.len(), 1);
    assert_eq!(q.with_qualifiers[0].name, "spy");
}

#[tokio::test]
async fn include_qualifiers_restores_scoring_and_fitting() {
    let (seen, qualifier, report) = run(true).await;
    let scored: BTreeSet<i64> = seen.predicts.iter().map(|p| p.0).collect();
    let fitted: BTreeSet<i64> = seen.fits.iter().flatten().copied().collect();
    assert!(!scored.is_disjoint(&qualifier));
    assert!(!fitted.is_disjoint(&qualifier));
    assert!(report.qualifiers.included && report.qualifiers.excluded.is_empty());
}
