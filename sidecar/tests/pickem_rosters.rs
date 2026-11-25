//! Seam 1: the pre-Stage Pick'em refresh (story 42) is due when the odds *or
//! the rosters* changed since the submission; without roster facts (the
//! Context Enricher failing) it goes by the odds alone.
mod support;
use async_trait::async_trait;
use chrono::Duration;
use kallbet::{
    domain::{MatchFormat, StageKind},
    poller,
    testing::EventSnapshotBuilder,
};
use kallbet_ai::{
    adapters::pickem::{self, MonteCarloPickemSolver, submitted_pickems},
    *,
};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};
use support::Harness;

const TEAMS: [&str; 16] = [
    "MOUZ", "Vitality", "FURIA", "Aurora", "Spirit", "BETBOOM", "Falcons", "FUT", "Legacy",
    "Nemiga", "G2", "1win", "Natus Vincere", "Astralis", "Ninjas in Pyjamas", "PARIVISION",
];

/// The harness's scripted facts, or an error while `failing`.
struct Rosters {
    facts: Mutex<Vec<AsOf<Value>>>,
    failing: Mutex<bool>,
}
#[async_trait]
impl ContextEnricher for Rosters {
    async fn facts(&self, _: &Match, _: Instant) -> Result<Vec<AsOf<Value>>> {
        if *self.failing.lock().unwrap() {
            return Err("Liquipedia HTTP 503".into());
        }
        Ok(self.facts.lock().unwrap().clone())
    }
}
fn roster(players: &[&str], observed: Instant, cutoff: Instant) -> AsOf<Value> {
    AsOf::new(
        json!({"kind": "liquipedia_roster", "team": "MOUZ", "match_side": "team_a",
               "active_players": players, "stand_ins": [], "has_stand_in": false}),
        observed,
        cutoff,
    )
    .unwrap()
}

#[tokio::test]
async fn a_stand_in_before_the_stage_triggers_the_refresh_and_a_failed_enricher_does_not() {
    let h = Harness::new().await;
    let t0: Instant = "2026-11-24T00:00:00Z".parse().unwrap();
    h.app.clock.set(t0);
    let start = t0 + Duration::hours(4);
    let mut builder = EventSnapshotBuilder::new(8302, "PGL Major Singapore 2026")
        .stage(StageKind::Swiss, "Stage 1")
        .format(MatchFormat::Bo1);
    for i in 0..8 {
        builder = builder.swiss_match(101 + i as i64, (1, 0, 0), TEAMS[i], TEAMS[i + 8], start);
    }
    h.app.source.put_event(builder.build(t0));
    let alice = h.app.user("Alice").await;
    let league = alice
        .post("/api/leagues", &json!({"name": "Pick'em", "event_id": 8302}))
        .await
        .json();
    let invite = format!(
        "{}/join/{}",
        h.base,
        league["invite_token"].as_str().unwrap()
    );
    poller::tick(&h.app.state).await;

    let context = Arc::new(Rosters {
        facts: Mutex::new(vec![roster(
            &["torzsi", "xertioN", "Jimpphat", "Brollan", "Spinx"],
            t0 - Duration::days(1),
            t0,
        )]),
        failing: Mutex::new(false),
    });
    let mut sidecar = h.sidecar(&invite).await;
    sidecar.ports.pickem = pickem::with_solver(
        sidecar.ports.clock.clone(),
        sidecar.ports.signal.clone(),
        sidecar.ports.odds.clone(),
        context.clone(),
        Arc::new(MonteCarloPickemSolver::with_simulations(2026, 500)),
    );
    h.tick(&mut sidecar, t0).await.unwrap();
    assert_eq!(submitted_pickems(&sidecar.audit).await.unwrap().len(), 1, "the initial Pick'em");

    // In the refresh window, odds and ratings unchanged and the enricher down:
    // nothing to refresh on.
    *context.failing.lock().unwrap() = true;
    h.tick(&mut sidecar, start - Duration::minutes(28)).await.unwrap();
    assert_eq!(submitted_pickems(&sidecar.audit).await.unwrap().len(), 1);

    // Back up, with a stand-in for MOUZ: the inputs changed, one refresh.
    *context.failing.lock().unwrap() = false;
    let at = start - Duration::minutes(25);
    *context.facts.lock().unwrap() = vec![roster(
        &["torzsi", "xertioN", "xelex", "Brollan", "Spinx"],
        at - Duration::minutes(10),
        at,
    )];
    h.tick(&mut sidecar, at).await.unwrap();
    assert_eq!(submitted_pickems(&sidecar.audit).await.unwrap().len(), 2, "refreshed on the roster change");
    h.tick(&mut sidecar, at + Duration::minutes(2)).await.unwrap();
    assert_eq!(submitted_pickems(&sidecar.audit).await.unwrap().len(), 2, "once");
}
