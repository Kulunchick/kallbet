use kallbet_ai::{Format, vrs::Standings};
const OCT: &str = include_str!("fixtures/vrs/standings_global_2026_10_05.md");
const NOV: &str = include_str!("fixtures/vrs/standings_global_2026_11_02.md");

#[test]
fn latest_strictly_prior_standings_drive_each_series_format() {
    let standings = Standings::parse([NOV, OCT]).unwrap();
    let cutoff = "2026-11-24T00:00:00Z".parse().unwrap();
    for (format, expected) in [
        (Format::Bo1, 0.55300022),
        (Format::Bo3, 0.57920257),
        (Format::Bo5, 0.59863353),
    ] {
        let signal = standings
            .estimate("mouz", "Vitality", format, cutoff)
            .unwrap()
            .unwrap();
        let signal = signal.at(cutoff).unwrap();
        assert!((signal.series - expected).abs() < 1e-8);
        assert!((signal.map - 0.55300022).abs() < 1e-8);
    }
    let earlier = "2026-11-02T12:00:00Z".parse().unwrap();
    assert!(
        standings
            .estimate("MOUZ", "Vitality", Format::Bo1, earlier)
            .unwrap()
            .unwrap()
            .at(earlier)
            .unwrap()
            .map
            < 0.5
    );
    let before = "2026-10-01T12:00:00Z".parse().unwrap();
    assert!(
        standings
            .estimate("MOUZ", "Vitality", Format::Bo1, before)
            .unwrap()
            .is_none()
    );
    assert!(
        standings
            .estimate("unknown", "Vitality", Format::Bo1, cutoff)
            .unwrap()
            .is_none()
    );
}
