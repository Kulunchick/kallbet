//! Ticket #38 (spec #33): the market Candidate Signals (source comparison, line
//! movement, favourite-longshot correction) at the lab's seam, over the saved
//! replay archive, and their pure statistics with fixed data.
use chrono::Duration;
use kallbet_ai::{
    adapters::{
        backtest::{
            ReplayArchive,
            feeds::{MarketSource, PriceHistory, ReplayOdds},
        },
        lab::{
            Lab, LabOutcome,
            signals::{
                favourite_longshot::{FavouriteLongshot, fit_gamma},
                line_movement::{LineMovement, Movement, fit_coefficients, logit, movement},
            },
            stats::Verdict,
        },
        odds::oddspapi,
    },
    *,
};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::Arc,
};

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/backtest")
}
fn whole() -> ReplayArchive {
    ReplayArchive::load(&fixtures()).unwrap()
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
fn row<'a>(o: &'a LabOutcome, name: &str) -> &'a kallbet_ai::adapters::lab::CandidateReport {
    o.report
        .candidates
        .iter()
        .find(|c| c.name == name)
        .unwrap_or_else(|| panic!("{name} is registered and reported"))
}

const MARKET_SIGNALS: [&str; 6] = [
    "market_source_pinnacle",
    "market_source_polymarket",
    "market_source_kalshi",
    "market_source_average",
    "line_movement",
    "favourite_longshot",
];

#[tokio::test]
async fn the_three_signals_are_reported_with_the_protocol_columns() {
    let archive = whole();
    let odds = ReplayOdds::new(&archive);
    // The closing point of every source's history matches its closing quote.
    for m in archive.stages.iter().flat_map(|s| &s.matches) {
        let cutoff = m.cutoff();
        for source in MarketSource::ALL {
            let quote = odds.quote_from(source, &m.game, cutoff).unwrap();
            let pts = odds.points_from(source, &m.game, cutoff).unwrap();
            if let Some(q) = quote {
                let last = pts
                    .last()
                    .unwrap_or_else(|| panic!("{source:?} quote without history on {}", m.game.id));
                let qv = *q.at(cutoff).unwrap();
                let pv = *last.at(cutoff).unwrap();
                assert!(
                    (qv - pv).abs() < 1e-12,
                    "{source:?} match {}: quote {qv} vs last point {pv}",
                    m.game.id
                );
            }
        }
    }

    let outcome = Lab::new(archive).unwrap().run().await.unwrap();
    for name in MARKET_SIGNALS {
        let c = row(&outcome, name);
        assert!(!c.reason.is_empty(), "{name} states its reason");
        assert!(c.computed_on > 0, "{name} computed on saved archive");
        assert!(c.paired > 0, "{name} paired on saved archive");
        assert!(c.difference.is_some(), "{name} reports metric differences");
        assert!(c.interval.is_some(), "{name} reports bootstrap interval");
        assert!(c.detectable.is_some(), "{name} reports detectable effect");
    }
    // The four source rows cover the very same Matches: those every source priced.
    let sources = ["pinnacle", "polymarket", "kalshi", "average"]
        .map(|s| row(&outcome, &format!("market_source_{s}")));
    assert!(sources[0].computed_on > 0, "some Match was priced by all three");
    for s in &sources {
        assert_eq!(s.computed_on, sources[0].computed_on);
        assert_eq!(s.paired, sources[0].paired);
    }
    // Pinnacle is first in the live Odds Source order, so on Matches priced by all
    // three its probability matches the anchor identically.
    assert!(
        sources[0].difference.unwrap().log_loss.abs() < 1e-12,
        "pinnacle matches anchor on common Matches: {:?}",
        sources[0].difference
    );
    let md = outcome.report.markdown();
    for name in MARKET_SIGNALS {
        assert!(md.contains(name), "{name} missing from markdown report");
    }
}

#[tokio::test]
async fn only_points_before_the_cutoff_are_read_and_in_play_points_exist() {
    let archive = whole();
    let odds = ReplayOdds::new(&archive);
    let mut read = [0usize; 3];
    let mut excluded = [0usize; 3];
    for m in archive.stages.iter().flat_map(|s| &s.matches) {
        let cutoff = m.cutoff();
        for (i, source) in MarketSource::ALL.iter().enumerate() {
            let before = odds.points_from(*source, &m.game, cutoff).unwrap();
            assert!(before.iter().all(|p| p.observed_at() < cutoff), "{source:?}");
            // Points are strictly ordered oldest first.
            for w in before.windows(2) {
                assert!(w[0].observed_at() <= w[1].observed_at(), "{source:?}");
            }
            read[i] += before.len();
            // Asked three days later, Polymarket and Kalshi histories show more
            // points (the in-play ones kept out by cutoff). OddsPapi's
            // `SettledFixture::points` also caps at Hard Lock, so check the raw
            // saved OddsPapi history bytes directly to prove in-play ticks exist
            // there while `points_from` never returns them.
            if *source == MarketSource::OddsPapi {
                let capped = odds
                    .points_from(*source, &m.game, cutoff + Duration::days(3))
                    .unwrap();
                assert!(
                    capped.iter().all(|p| p.observed_at() < cutoff + Duration::minutes(15)),
                    "OddsPapi SettledFixture never leaks past Hard Lock"
                );
                for list in &archive.fixture_lists {
                    if let Some(fx) = oddspapi::match_settled_fixture(list, &m.game).unwrap()
                        && let Some(body) = archive.histories.get(fx.id())
                    {
                        let raw_later = oddspapi::parse_history_points(
                            body,
                            fx.id(),
                            cutoff + Duration::days(3),
                        )
                        .unwrap();
                        excluded[i] += raw_later.len().saturating_sub(before.len());
                        break;
                    }
                }
            } else {
                let later = odds
                    .points_from(*source, &m.game, cutoff + Duration::days(3))
                    .unwrap();
                excluded[i] += later.len() - before.len();
            }
        }
    }
    for (i, source) in MarketSource::ALL.iter().enumerate() {
        assert!(read[i] > 0, "{source:?} has pre-cutoff price points");
        assert!(
            excluded[i] > 0,
            "{source:?} saved histories hold in-play points at or after the cutoff"
        );
    }

    // A PriceHistory port returning a datum at or after the cutoff aborts the run.
    struct LeakyHistory(Duration);
    #[async_trait::async_trait]
    impl PriceHistory for LeakyHistory {
        async fn points(
            &self,
            source: MarketSource,
            _game: &Match,
            cutoff: Instant,
        ) -> Result<Vec<AsOf<f64>>> {
            if source != MarketSource::OddsPapi {
                return Ok(vec![]);
            }
            let at = cutoff + self.0;
            Ok(vec![AsOf::new(0.6, at, at + Duration::days(1))?])
        }
    }
    let dir = archive_of(&[
        "panda-22378",
        "standings_global_2026_11_02",
        "oddspapi-fixtures-pgl-singapore",
        "od1740001",
    ]);
    for offset in [Duration::zero(), Duration::seconds(1)] {
        let mut lab = Lab::new(ReplayArchive::load(dir.path()).unwrap()).unwrap();
        lab.price_history = Arc::new(LeakyHistory(offset));
        let err = lab.run().await.err().expect("leaky price point aborts");
        assert!(err.is::<TemporalLeakError>(), "{err}");
    }
}

/// A line the test scripts: the same points for OddsPapi of every Match.
struct Scripted(Vec<(i64, f64)>);
#[async_trait::async_trait]
impl PriceHistory for Scripted {
    async fn points(
        &self,
        source: MarketSource,
        _game: &Match,
        cutoff: Instant,
    ) -> Result<Vec<AsOf<f64>>> {
        if source != MarketSource::OddsPapi {
            return Ok(vec![]);
        }
        self.0
            .iter()
            .map(|(hours_before, p)| {
                Ok(AsOf::new(*p, cutoff - Duration::hours(*hours_before), cutoff)?)
            })
            .collect()
    }
}

#[tokio::test]
async fn a_single_point_or_no_line_gives_no_movement_not_a_zero() {
    for points in [vec![], vec![(2, 0.6)]] {
        let mut lab = Lab::new(whole()).unwrap();
        lab.signals = vec![Arc::new(LineMovement)];
        lab.price_history = Arc::new(Scripted(points));
        let outcome = lab.run().await.unwrap();
        let c = row(&outcome, "line_movement");
        assert_eq!(c.computed_on, 0);
        assert!(outcome
            .predictions
            .iter()
            .filter(|p| p.signal == "line_movement")
            .all(|p| p.variant.is_none()));
    }
    // Two points do give a value on the Events that have a training history.
    let mut lab = Lab::new(whole()).unwrap();
    lab.signals = vec![Arc::new(LineMovement)];
    lab.price_history = Arc::new(Scripted(vec![(30, 0.55), (1, 0.62)]));
    let outcome = lab.run().await.unwrap();
    assert!(row(&outcome, "line_movement").computed_on > 0);
}

#[tokio::test]
async fn walk_forward_fits_favourite_longshot_and_line_movement_only_on_earlier_events() {
    let run_fitted = |archive: ReplayArchive| async move {
        let mut lab = Lab::new(archive).unwrap();
        lab.signals = vec![Arc::new(FavouriteLongshot), Arc::new(LineMovement)];
        lab.run().await.unwrap()
    };
    let base = run_fitted(whole()).await;

    // Cologne 2026 is the first Event with market prices (88 priced Matches); it
    // has no earlier priced Events, so neither fitted market signal predicts on it.
    // Singapore 2026 follows Cologne and is fitted on Cologne's 88 Matches.
    for name in ["favourite_longshot", "line_movement"] {
        let c = row(&base, name);
        let cologne = c
            .per_event
            .iter()
            .find(|e| e.title.contains("Cologne"))
            .unwrap();
        let singapore = c
            .per_event
            .iter()
            .find(|e| e.title.contains("Singapore"))
            .unwrap();
        assert_eq!(cologne.computed, 0, "{name} must not fit on Cologne's own outcomes");
        assert!(singapore.computed > 0, "{name} fits on Cologne for Singapore");
    }

    let predictions_of = |o: &LabOutcome, signal: &str| -> BTreeMap<i64, (i64, Option<f64>)> {
        o.predictions
            .iter()
            .filter(|p| p.signal == signal)
            .map(|p| (p.match_id, (p.event_id, p.variant)))
            .collect()
    };

    // 1. Flipping every outcome of Singapore 2026 changes nothing in Singapore or earlier.
    let mut flip_sg = whole();
    let sg_id = flip_sg
        .stages
        .iter()
        .find(|s| s.event.contains("Singapore"))
        .unwrap()
        .event_id;
    for stage in flip_sg.stages.iter_mut().filter(|s| s.event_id == sg_id) {
        for m in &mut stage.matches {
            m.winner = m.loser();
        }
    }
    let after_sg = run_fitted(flip_sg).await;
    for name in ["favourite_longshot", "line_movement"] {
        assert_eq!(
            predictions_of(&base, name),
            predictions_of(&after_sg, name),
            "reversing Singapore's own outcomes must not move any {name} prediction"
        );
    }

    // 2. Flipping every outcome of Cologne 2026 leaves Cologne & earlier unchanged,
    //    and moves Singapore's predictions because Singapore fits on Cologne.
    let mut flip_col = whole();
    let col_id = flip_col
        .stages
        .iter()
        .find(|s| s.event.contains("Cologne"))
        .unwrap()
        .event_id;
    for stage in flip_col.stages.iter_mut().filter(|s| s.event_id == col_id) {
        for m in &mut stage.matches {
            m.winner = m.loser();
        }
    }
    let after_col = run_fitted(flip_col).await;
    for name in ["favourite_longshot", "line_movement"] {
        let before = predictions_of(&base, name);
        let after = predictions_of(&after_col, name);
        let mut sg_moved = 0;
        for (mid, (eid, v0)) in &before {
            let (_, v1) = &after[mid];
            if *eid != sg_id {
                assert_eq!(v0, v1, "{name} moved on non-later Event {eid} match {mid}");
            } else if v0 != v1 {
                sg_moved += 1;
            }
        }
        assert!(
            sg_moved > 0,
            "{name} should update on Singapore when Cologne's outcomes flip"
        );
    }
}

#[tokio::test]
async fn archive_without_market_histories_reports_market_signals_not_testable() {
    let dir = archive_of(&["panda-22378", "standings_global_2026_11_02"]);
    let outcome = Lab::new(ReplayArchive::load(dir.path()).unwrap())
        .unwrap()
        .run()
        .await
        .unwrap();
    for name in MARKET_SIGNALS {
        let c = row(&outcome, name);
        assert_eq!(c.verdict, Verdict::NotTestable, "{name}");
        assert_eq!(c.computed_on, 0, "{name}");
        assert!(!c.reason.is_empty(), "{name}");
    }
}

#[test]
fn movement_is_the_logit_change_and_needs_two_points() {
    let cutoff: Instant = "2026-11-24T12:00:00Z".parse().unwrap();
    let at = |h: i64, p: f64| (cutoff - Duration::hours(h), p);
    assert_eq!(movement(&[], cutoff), None);
    assert_eq!(movement(&[at(3, 0.6)], cutoff), None);
    let mv = movement(&[at(48, 0.5), at(8, 0.55), at(1, 0.65)], cutoff).unwrap();
    assert!((mv.total - (logit(0.65) - logit(0.5))).abs() < 1e-12);
    // The reference of the late move is the last point 6 hours or more before.
    assert!((mv.late - (logit(0.65) - logit(0.55))).abs() < 1e-12);
    // With every point inside the last hours, the opening one is the reference.
    let mv = movement(&[at(3, 0.5), at(1, 0.6)], cutoff).unwrap();
    assert_eq!(mv.total, mv.late);
}

#[test]
fn the_movement_coefficients_follow_the_data_and_are_symmetric_and_deterministic() {
    // Rows where a rising line goes with winning: a positive coefficient.
    let mut rows = Vec::new();
    for i in 0..60 {
        let up = i % 2 == 0;
        let x = if up { 0.5 } else { -0.5 };
        let won = if i % 6 == 0 { !up } else { up };
        rows.push((Movement { total: x, late: x }, 0.0, won));
    }
    let b = fit_coefficients(&rows, 1.0);
    assert!(b[0] > 0.0 && b[1] > 0.0, "{b:?}");
    assert_eq!(b, fit_coefficients(&rows, 1.0));
    // Swapping team A and team B negates movement and offset and flips `won`,
    // leaving the fitted coefficients unchanged.
    let mirrored: Vec<_> = rows
        .iter()
        .map(|(m, off, won)| (Movement { total: -m.total, late: -m.late }, -*off, !*won))
        .collect();
    let bm = fit_coefficients(&mirrored, 1.0);
    assert!((b[0] - bm[0]).abs() < 1e-10 && (b[1] - bm[1]).abs() < 1e-10);
}

#[test]
fn gamma_is_above_one_when_longshots_win_less_than_priced_and_is_symmetric() {
    // Priced 0.30 / 0.70: the longshot wins 20%, the favourite 80% of the time.
    let mut rows = Vec::new();
    for i in 0..100 {
        rows.push((logit(0.3), i < 20));
        rows.push((logit(0.7), i < 80));
    }
    let g = fit_gamma(&rows).unwrap();
    assert!(g > 1.1, "{g}");
    let mirrored: Vec<_> = rows.iter().map(|(x, w)| (-*x, !*w)).collect();
    assert!((g - fit_gamma(&mirrored).unwrap()).abs() < 1e-9);

    // Priced 0.30 / 0.70 when longshots win 40% (favourites overpriced): gamma < 1.
    let mut fav_over = Vec::new();
    for i in 0..100 {
        fav_over.push((logit(0.3), i < 40));
        fav_over.push((logit(0.7), i < 60));
    }
    assert!(fit_gamma(&fav_over).unwrap() < 0.9);

    // Well calibrated prices leave gamma at one.
    let mut fair = Vec::new();
    for i in 0..100 {
        fair.push((logit(0.3), i < 30));
        fair.push((logit(0.7), i < 70));
    }
    assert!((fit_gamma(&fair).unwrap() - 1.0).abs() < 0.05);
    assert_eq!(fit_gamma(&[]), None);
}

#[test]
fn oddspapi_points_merges_async_ticks_respects_suspension_and_cutoff() {
    let body = serde_json::json!({
        "fixtureId": "fx1",
        "bookmakers": {
            "pinnacle+30": {
                "markets": {
                    "171": {
                        "outcomes": {
                            "171": { "players": { "0": [
                                { "createdAt": "2026-11-24T10:00:00Z", "price": 2.0, "active": true },
                                { "createdAt": "2026-11-24T10:30:00Z", "price": 1.5, "active": true },
                                { "createdAt": "2026-11-24T11:00:00Z", "price": 1.5, "active": false },
                                { "createdAt": "2026-11-24T12:00:00Z", "price": 1.1, "active": true }
                            ]}},
                            "172": { "players": { "0": [
                                { "createdAt": "2026-11-24T10:00:00Z", "price": 2.0, "active": true },
                                { "createdAt": "2026-11-24T11:00:00Z", "price": 2.5, "active": true },
                                { "createdAt": "2026-11-24T12:00:00Z", "price": 8.0, "active": true }
                            ]}}
                        }
                    }
                }
            }
        }
    });
    let cutoff: Instant = "2026-11-24T12:00:00Z".parse().unwrap();
    let pts = oddspapi::parse_history_points(&serde_json::to_vec(&body).unwrap(), "fx1", cutoff)
        .unwrap();
    // 10:00 both active (0.5), 10:30 side 171 updated while 172 still active (1/1.5 / (1/1.5 + 1/2.0) = 4/7),
    // 11:00 side 171 suspended (no point), 12:00 at cutoff (excluded).
    assert_eq!(pts.len(), 2, "{pts:?}");
    assert!((pts[0].1 - 0.5).abs() < 1e-12);
    assert!((pts[1].1 - (4.0 / 7.0)).abs() < 1e-12);
}
