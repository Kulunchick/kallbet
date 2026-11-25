use chrono::Duration;
use kallbet_ai::{AsOf, TemporalLeakError};
#[test]
fn cutoff_is_exclusive_even_for_prevalidated_observations() {
    let cutoff = "2026-11-24T00:00:00Z".parse().unwrap();
    let before = cutoff - Duration::seconds(1);
    let datum = AsOf::new("rating", before, cutoff).unwrap();
    assert_eq!(datum.at(cutoff).unwrap(), &"rating");
    assert!(matches!(datum.at(before), Err(TemporalLeakError { .. })));
    for observed in [cutoff, cutoff + Duration::seconds(1)] {
        assert!(AsOf::new("rating", observed, cutoff).is_err());
    }
}
