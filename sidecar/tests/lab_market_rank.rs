//! Tests for ticket #60 (`market_rank_rule`: a pick rule where the market and the rank
//! disagree). Saved fixtures only; no network.
use kallbet_ai::{
    Format,
    adapters::{
        backtest::ReplayArchive,
        lab::{
            Lab,
            signals::{self, market_rank_rule::RuleTable},
            stats::Verdict,
        },
    },
};
use std::path::Path;

fn fixtures() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

/// `n` disagreements of one size and format, of which the rank favourite won `rank_won`.
fn rows(gap: f64, format: Format, n: usize, rank_won: usize) -> Vec<(f64, Format, bool)> {
    (0..n).map(|i| (gap, format, i < rank_won)).collect()
}

#[test]
fn the_table_learns_which_side_wins_by_disagreement_size_and_format() {
    let mut training = rows(0.3, Format::Bo3, 20, 4); // small gap: the market side wins
    training.extend(rows(1.5, Format::Bo3, 20, 16)); // large gap: the rank side wins
    let table = RuleTable::fit(&training);

    let (rank, p) = table.pick(0.3, Format::Bo3).unwrap();
    assert!(!rank, "the market side wins small disagreements");
    assert!((p - 17.0 / 22.0).abs() < 1e-12, "smoothed win rate, got {p}");
    let (rank, p) = table.pick(1.5, Format::Bo3).unwrap();
    assert!(rank);
    assert!((p - 17.0 / 22.0).abs() < 1e-12);
    // A format with no rows falls back to the size bucket pooled over formats.
    assert_eq!(table.pick(1.5, Format::Bo5).map(|x| x.0), Some(true));
    // A size bucket with no rows has no rule: follow the anchor.
    assert!(table.pick(0.7, Format::Bo3).is_none());
    assert!(RuleTable::fit(&[]).pick(0.3, Format::Bo3).is_none());
}

#[test]
fn a_cell_with_too_few_rows_falls_back_to_the_pooled_bucket() {
    let mut training = rows(1.5, Format::Bo1, 2, 0); // too few to trust alone
    training.extend(rows(1.5, Format::Bo3, 20, 15));
    let table = RuleTable::fit(&training);
    assert_eq!(table.pick(1.5, Format::Bo1).map(|x| x.0), Some(true));
}

#[tokio::test]
async fn market_rank_rule_is_registered_marked_and_reports_the_disagreement_subset() {
    let signal = signals::by_name("market_rank_rule").expect("registered");
    assert!(signal.added_after_list_fixed());

    let archive = ReplayArchive::load(&fixtures().join("backtest")).unwrap();
    let mut lab = Lab::new(archive).unwrap();
    lab.signals = vec![signal];
    let outcome = lab.run().await.unwrap();
    let row = &outcome.report.candidates[0];
    assert_eq!(row.name, "market_rank_rule");
    assert!(row.added_after_list_fixed);
    assert!(row.computed_on > 0 && row.paired > 0, "computed {} paired {}", row.computed_on, row.paired);
    assert!(matches!(row.verdict, Verdict::Promote | Verdict::NoEvidence | Verdict::Harmful));
    let subset = row.subset.as_ref().expect("the disagreement subset is reported");
    assert!(subset.matches <= row.paired);
    let md = outcome.report.markdown();
    assert!(md.contains("(added after list fixed)"), "{md}");
    assert!(md.contains("disagreement Matches"), "{md}");
    eprintln!("{subset:?}");
}
