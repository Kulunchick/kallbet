use chrono::Duration;
use kallbet_ai::{Format, Instant, Match, Team, TemporalLeakError, adapters::odds::oddspapi};

const ODDS_PAPI: &[u8] =
    include_bytes!("fixtures/odds/oddspapi-odds-by-tournaments-pgl-singapore-pinnacle.json");

fn game(a: &str, b: &str, start: &str) -> Match {
    Match {
        event_id: 99,
        id: 42,
        team_a: Team {
            id: 1,
            name: a.into(),
        },
        team_b: Team {
            id: 2,
            name: b.into(),
        },
        format: Format::Bo3,
        scheduled_start: Some(start.parse().unwrap()),
    }
}

#[test]
fn pinnacle_margin_is_removed_and_outcome_ids_follow_participants() {
    let observed: Instant = "2026-11-25T17:20:00Z".parse().unwrap();
    let cutoff = observed + Duration::minutes(1);
    let mut game = game("Eternal Fire", "M80", "2026-11-26T06:12:00Z");
    let quote = oddspapi::parse(ODDS_PAPI, &game, cutoff).unwrap().unwrap();
    let p_a = *quote.at(cutoff).unwrap();
    assert!((p_a - 0.513352346).abs() < 1e-9);
    std::mem::swap(&mut game.team_a, &mut game.team_b);
    let quote = oddspapi::parse(ODDS_PAPI, &game, cutoff).unwrap().unwrap();
    assert!((p_a + quote.at(cutoff).unwrap() - 1.0).abs() < 1e-12);
    game.scheduled_start = Some("2026-11-27T06:00:00Z".parse().unwrap());
    assert!(oddspapi::parse(ODDS_PAPI, &game, cutoff).unwrap().is_none());
    game.scheduled_start = Some("2026-11-26T06:00:00Z".parse().unwrap());
    let changed_at = "2026-11-25T16:54:26.487Z".parse().unwrap();
    assert!(
        oddspapi::parse(ODDS_PAPI, &game, changed_at)
            .unwrap_err()
            .is::<TemporalLeakError>()
    );
}

#[test]
fn polymarket_uses_the_series_moneyline_and_source_timestamp() {
    use kallbet_ai::adapters::odds::polymarket;
    let body = include_bytes!("fixtures/odds/polymarket-event-cs2-hero-faze-2026-11-26-open.json");
    let mut game = game("FaZe Clan", "HEROIC", "2026-11-26T06:00:00Z");
    let cutoff = "2026-11-25T17:00:00Z".parse().unwrap();
    let price_time = "2026-11-25T16:51:28.410458Z".parse().unwrap();
    let quote = polymarket::parse(body, &game, cutoff).unwrap().unwrap();
    assert!((quote.at(cutoff).unwrap() - 0.455).abs() < 1e-12);
    assert_eq!(quote.observed_at(), price_time);
    assert!(
        polymarket::parse(body, &game, price_time)
            .unwrap_err()
            .is::<TemporalLeakError>()
    );
    game.scheduled_start = Some("2026-11-26T12:00:01Z".parse().unwrap());
    assert!(polymarket::parse(body, &game, cutoff).unwrap().is_none());
}

#[test]
fn kalshi_matches_the_start_in_the_rules_and_normalizes_the_midpoints() {
    use kallbet_ai::adapters::odds::kalshi;
    let body = include_bytes!("fixtures/odds/kalshi-events-KXCS2GAME-open.json");
    let mut game = game("HEROIC", "FaZe Clan", "2026-11-26T06:00:00Z");
    let observed: Instant = "2026-11-25T16:45:00Z".parse().unwrap();
    let cutoff = observed + Duration::seconds(1);
    let quote = kalshi::parse(body, &game, observed, cutoff)
        .unwrap()
        .unwrap();
    assert!((quote.at(cutoff).unwrap() - 0.542288557).abs() < 1e-9);
    assert_eq!(quote.observed_at(), observed);
    assert!(
        kalshi::parse(body, &game, observed, observed)
            .unwrap_err()
            .is::<TemporalLeakError>()
    );
    game.scheduled_start = Some("2026-11-26T13:00:00Z".parse().unwrap());
    assert!(
        kalshi::parse(body, &game, observed, cutoff)
            .unwrap()
            .is_none()
    );
}

#[test]
fn historical_pinnacle_ignores_ticks_at_or_after_the_cutoff() {
    let body =
        include_bytes!("fixtures/odds/oddspapi-historical-odds-od174000143389068-pinnacle.json");
    let cutoff = "2026-11-24T04:45:00Z".parse().unwrap();
    let quote = oddspapi::parse_history(body, "od174000143389068", cutoff)
        .unwrap()
        .unwrap();
    assert!((quote.at(cutoff).unwrap() - 0.560963115).abs() < 1e-9);
    let tick = "2026-11-24T04:05:36.202Z".parse().unwrap();
    assert_eq!(quote.observed_at(), tick);
    let earlier = oddspapi::parse_history(body, "od174000143389068", tick)
        .unwrap()
        .unwrap();
    assert!(earlier.observed_at() < tick);
    assert!(oddspapi::parse_history(body, "different-fixture", cutoff).is_err());
}

#[test]
fn replay_links_settled_fixtures_with_aliases_and_preserves_canonical_team_order() {
    let fixtures = include_bytes!("fixtures/odds/oddspapi-fixtures-pgl-singapore-settled.json");
    let raw_odds =
        include_bytes!("fixtures/odds/oddspapi-historical-odds-od174000143389068-pinnacle.json");
    let mut game = game("M80", "NIP", "2026-11-24T05:00:00Z");
    let fixture = oddspapi::match_settled_fixture(fixtures, &game)
        .unwrap()
        .unwrap();
    assert_eq!(fixture.id(), "od174000143389068");
    let late = "2026-11-24T08:00:00Z".parse().unwrap();
    let quote = fixture.quote(raw_odds, late).unwrap().unwrap();
    assert!((quote.at(late).unwrap() - 0.439036885).abs() < 1e-9);
    assert_eq!(
        quote.observed_at(),
        "2026-11-24T04:05:36.202Z".parse::<Instant>().unwrap()
    );
    game.scheduled_start = Some("2026-11-23T12:00:00Z".parse().unwrap());
    assert!(
        oddspapi::match_settled_fixture(fixtures, &game)
            .unwrap()
            .is_none()
    );
}
