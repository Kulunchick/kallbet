//! The VRS estimator re-scans its configured directory while running: a Valve
//! snapshot dropped in (by `fetch-vrs.sh` on a schedule) is used without a
//! restart, at most `RESCAN_EVERY` later.
use chrono::Duration;
use kallbet_ai::{
    adapters::vrs_files::{RESCAN_EVERY, WatchedStandings},
    *,
};
use std::sync::{Arc, Mutex};

const OCTOBER: &str = include_str!("fixtures/vrs/standings_global_2026_10_05.md");
const NOVEMBER: &str = include_str!("fixtures/vrs/standings_global_2026_11_02.md");

struct ManualClock(Mutex<Instant>);
impl Clock for ManualClock {
    fn now(&self) -> Instant {
        *self.0.lock().unwrap()
    }
}

#[tokio::test]
async fn a_snapshot_dropped_into_the_directory_is_used_after_the_next_scan() {
    let dir = tempfile::tempdir().unwrap();
    let vrs = dir.path().join("vrs");
    std::fs::create_dir(&vrs).unwrap();
    std::fs::write(vrs.join("standings_global_2026_10_05.md"), OCTOBER).unwrap();
    let audit = Audit::open(&dir.path().join("audit.db")).await.unwrap();
    let now: Instant = "2026-11-20T12:00:00Z".parse().unwrap();
    let clock = Arc::new(ManualClock(Mutex::new(now)));
    let signal = WatchedStandings::load(&[vrs.clone()], &audit, clock.clone())
        .await
        .unwrap();
    let game = Match {
        event_id: 1,
        id: 1,
        team_a: Team {
            id: 1,
            name: "MOUZ".into(),
        },
        team_b: Team {
            id: 2,
            name: "Vitality".into(),
        },
        format: Format::Bo3,
        scheduled_start: None,
    };
    let observed = |s: Option<AsOf<Signal>>| s.unwrap().observed_at();
    let october = observed(signal.signal(&game, now).await.unwrap());
    assert_eq!(october.date_naive().to_string(), "2026-10-05");

    // fetch-vrs.sh writes a .partial file, then renames it.
    std::fs::write(vrs.join("standings_global_2026_11_02.md.partial"), NOVEMBER).unwrap();
    std::fs::rename(
        vrs.join("standings_global_2026_11_02.md.partial"),
        vrs.join("standings_global_2026_11_02.md"),
    )
    .unwrap();
    let soon = now + RESCAN_EVERY - Duration::seconds(1);
    *clock.0.lock().unwrap() = soon;
    assert_eq!(
        observed(signal.signal(&game, soon).await.unwrap()),
        october,
        "not every Match lists the directory"
    );

    let later = now + RESCAN_EVERY;
    *clock.0.lock().unwrap() = later;
    let november = observed(signal.signal(&game, later).await.unwrap());
    assert_eq!(november.date_naive().to_string(), "2026-11-02");

    // A broken new file is skipped; the estimator goes on.
    std::fs::write(vrs.join("standings_global_2026_12_07.md"), "not a snapshot").unwrap();
    let then = later + RESCAN_EVERY;
    *clock.0.lock().unwrap() = then;
    assert_eq!(
        observed(signal.signal(&game, then).await.unwrap()),
        november
    );
}
