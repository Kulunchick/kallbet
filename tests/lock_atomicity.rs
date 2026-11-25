//! ADR-0003 under interleavings: once a Lock or Reveal instant has passed
//! with the data we had, no write lands after it, and no later data
//! un-happens it.
//!
//! A request checks Lock, then writes. Whatever another request or the
//! poller stores between the two must still be respected. The interleavings
//! are driven deterministically: the "check" already happened (it saw the
//! Match open), then the other party acts, then the store write runs.

use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use axum::http::StatusCode;
use chrono::{DateTime, Duration, Utc};
use kallbet::domain::pickem::{Pick, PickKind};
use kallbet::domain::{
    Event, EventId, EventSnapshot, LeagueId, MatchId, StageKind, StageOrdinal, TeamId, UserId,
};
use kallbet::poller;
use kallbet::ports::{Clock, MatchSource, SourceError};
use kallbet::store::{pickems, predictions};
use kallbet::testing::{EventSnapshotBuilder, FakeClock, ScriptedSource, TestApp, TestClient};
use serde_json::{Value, json};

const EVENT: i64 = 7148;
const NAVI_FAZE: i64 = 101;
const NAVI: i64 = 1;
const FAZE: i64 = 2;

struct Setup {
    app: TestApp,
    alice: TestClient,
    bob: TestClient,
    league: i64,
    start: DateTime<Utc>,
}

/// alice (creator) and bob in a League; NAVI vs FaZe in two hours, the only
/// Match of a Swiss Stage of two teams; polled.
async fn setup_with(app: TestApp, source: &ScriptedSource) -> Setup {
    let now = app.clock.now();
    let start = now + Duration::hours(2);
    source.put_event(
        EventSnapshotBuilder::new(EVENT, "Test Major")
            .stage(StageKind::Swiss, "Stage 1")
            .swiss_match(NAVI_FAZE, (1, 0, 0), "NAVI", "FaZe", start)
            .build(now),
    );
    let alice = app.user("alice").await;
    let bob = app.user("bob").await;
    let res = alice
        .post(
            "/api/leagues",
            &json!({ "event_id": EVENT, "name": "Friends" }),
        )
        .await;
    assert_eq!(res.status, StatusCode::CREATED, "{}", res.text());
    let league = res.json();
    let token = league["invite_token"].as_str().unwrap().to_string();
    bob.post_empty(&format!("/api/invites/{token}/join")).await;
    poller::tick(&app.state).await;
    Setup {
        league: league["id"].as_i64().unwrap(),
        app,
        alice,
        bob,
        start,
    }
}

async fn setup() -> Setup {
    let app = TestApp::new().await;
    let source = app.source.clone();
    setup_with(app, &source).await
}

impl Setup {
    async fn predict(&self, client: &TestClient, team: i64) {
        let res = client
            .put(
                &format!(
                    "/api/leagues/{}/matches/{NAVI_FAZE}/prediction",
                    self.league
                ),
                &json!({ "team_id": team }),
            )
            .await;
        assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    }

    async fn card(&self, client: &TestClient) -> Value {
        let res = client
            .get(&format!("/api/leagues/{}/matches", self.league))
            .await;
        assert_eq!(res.status, StatusCode::OK, "{}", res.text());
        res.json()["stages"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|s| s["groups"].as_array().unwrap().iter())
            .flat_map(|g| g["matches"].as_array().unwrap().iter())
            .find(|m| m["id"] == NAVI_FAZE)
            .cloned()
            .expect("the Match is in the view")
    }

    async fn user_id(&self, client: &TestClient) -> UserId {
        UserId(client.get("/api/me").await.json()["id"].as_i64().unwrap())
    }

    /// The team `user` is shown with in the revealed Predictions.
    fn revealed_team(card: &Value, user: UserId) -> Option<i64> {
        card["predictions"]["revealed"]
            .as_array()?
            .iter()
            .find(|p| p["user_id"] == user.0)
            .and_then(|p| p["team_id"].as_i64())
    }
}

#[tokio::test]
async fn a_prediction_write_after_another_members_reveal_does_not_land() {
    let s = setup().await;
    let alice = s.user_id(&s.alice).await;
    s.predict(&s.alice, NAVI).await;
    let checked_at = s.app.clock.now();
    // alice's request to change to FaZe has checked Lock: the Match is open.
    // Before it writes, bob's last Prediction Reveals (and Locks) the Match.
    s.predict(&s.bob, FAZE).await;
    s.app.clock.advance(Duration::minutes(5));
    assert!(s.card(&s.bob).await["lock"]["locked"].as_bool().unwrap());

    // alice's write runs now, with the instant of its check.
    let written = predictions::upsert(
        &s.app.state.db,
        LeagueId(s.league),
        MatchId(NAVI_FAZE),
        alice,
        TeamId(FAZE),
        None,
        checked_at,
    )
    .await;
    assert_eq!(written.unwrap(), None, "refused: the Match is Locked");

    let card = s.card(&s.bob).await;
    assert_eq!(Setup::revealed_team(&card, alice), Some(NAVI));
}

#[tokio::test]
async fn a_prediction_write_after_the_poller_stamped_lock_does_not_land() {
    let s = setup().await;
    let alice = s.user_id(&s.alice).await;
    // alice checks 16 minutes before the start (open) ...
    s.app.clock.set(s.start - Duration::minutes(16));
    let checked_at = s.app.clock.now();
    // ... the poller stamps the Lock a minute later ...
    s.app.clock.advance(Duration::minutes(1));
    poller::tick(&s.app.state).await;
    // ... then her write runs.
    let written = predictions::upsert(
        &s.app.state.db,
        LeagueId(s.league),
        MatchId(NAVI_FAZE),
        alice,
        TeamId(NAVI),
        None,
        checked_at,
    )
    .await;
    assert_eq!(written.unwrap(), None, "refused: the Match is Locked");

    let card = s.card(&s.bob).await;
    assert!(card["lock"]["locked"].as_bool().unwrap());
    assert_eq!(card["predictions"]["count"], 0, "{card}");
    assert_eq!(Setup::revealed_team(&card, alice), None);
}

#[tokio::test]
async fn a_pickem_write_after_the_stage_locked_does_not_land() {
    let s = setup().await;
    let alice = s.user_id(&s.alice).await;
    let path = format!("/api/leagues/{}/stages/1/pickem", s.league);
    // alice checks a minute before the Stage's first Match starts (open) ...
    s.app.clock.set(s.start - Duration::minutes(1));
    let checked_at = s.app.clock.now();
    // ... the Match starts and the poller stamps the Stage's Lock ...
    s.app.clock.set(s.start);
    s.app
        .source
        .start(EventId(EVENT), MatchId(NAVI_FAZE), s.start);
    s.app.clock.advance(Duration::minutes(5));
    poller::tick(&s.app.state).await;
    assert!(
        s.bob.get(&path).await.json()["lock"]["locked"]
            .as_bool()
            .unwrap()
    );
    // ... then her save runs.
    let picks = [Pick {
        kind: PickKind::Advance,
        slot: 0,
        team: TeamId(NAVI),
    }];
    let saved = pickems::save(
        &mut s.app.state.db.acquire().await.unwrap(),
        LeagueId(s.league),
        StageOrdinal(1),
        alice,
        &picks,
        checked_at,
    )
    .await;
    assert!(!saved.unwrap(), "refused: the Stage is Locked");

    let res = s.bob.get(&path).await.json();
    let pickems = res["pickems"].as_array().expect("everyone's after Lock");
    assert!(
        pickems.iter().all(|p| p["user_id"] != alice.0),
        "a Pick'em saved after Lock: {res}"
    );
}

/// A source whose fetch takes `delay` of (fake) time, as an HLTV page load
/// does.
struct SlowSource {
    inner: Arc<ScriptedSource>,
    clock: OnceLock<Arc<FakeClock>>,
    delay: Duration,
}

#[async_trait]
impl MatchSource for SlowSource {
    async fn list_events(&self) -> Result<Vec<Event>, SourceError> {
        self.inner.list_events().await
    }

    async fn fetch_event(&self, id: EventId) -> Result<EventSnapshot, SourceError> {
        if let Some(clock) = self.clock.get() {
            clock.advance(self.delay);
        }
        self.inner.fetch_event(id).await
    }
}

#[tokio::test]
async fn a_lock_that_falls_due_during_a_slow_fetch_survives_the_postponement_it_brings() {
    let scripted = Arc::new(ScriptedSource::new());
    let slow = Arc::new(SlowSource {
        inner: scripted.clone(),
        clock: OnceLock::new(),
        delay: Duration::minutes(1),
    });
    let app = TestApp::builder().source(slow.clone()).build().await;
    slow.clock.set(app.clock.clone()).ok();
    let s = setup_with(app, &scripted).await;
    // Lock is due at start - 15 min. The tick begins 30 s before that, and
    // its fetch (one minute) brings a postponement by three hours.
    s.app
        .clock
        .set(s.start - Duration::minutes(15) - Duration::seconds(30));
    scripted.postpone(
        EventId(EVENT),
        MatchId(NAVI_FAZE),
        s.start + Duration::hours(3),
    );
    poller::tick(&s.app.state).await;

    let card = s.card(&s.alice).await;
    assert!(
        card["lock"]["locked"].as_bool().unwrap(),
        "the Lock passed during the fetch, with the old schedule: {card}"
    );
}

#[tokio::test]
async fn a_scheduled_start_override_does_not_erase_a_lock_that_already_passed() {
    let s = setup().await;
    // The Lock passed (start - 15 min) but nothing has evaluated the League
    // since: no poller tick, no request.
    s.app.clock.set(s.start - Duration::minutes(10));
    let res = s
        .alice
        .post(
            &format!("/api/leagues/{}/matches/{NAVI_FAZE}/overrides", s.league),
            &json!({ "field": "scheduled_start",
                     "scheduled_start": s.start + Duration::hours(5) }),
        )
        .await;
    assert_eq!(res.status, StatusCode::CREATED, "{}", res.text());

    let card = s.card(&s.alice).await;
    assert!(
        card["lock"]["locked"].as_bool().unwrap(),
        "the Lock had passed before the Override: {card}"
    );
}

/// Spec #21, ticket #22: a Bot User's write is held back by the Hard Lock,
/// with the same guarantee: one stamped between its check and its write wins.
#[tokio::test]
async fn a_bot_users_write_after_the_poller_stamped_hard_lock_does_not_land() {
    let s = setup().await;
    let oracle = s.app.bot("oracle", "Oracle").await;
    let res = s.alice.get(&format!("/api/leagues/{}", s.league)).await;
    let token = res.json()["invite_token"].as_str().unwrap().to_string();
    oracle.post_empty(&format!("/api/invites/{token}/join")).await;
    let oracle_id = s.user_id(&oracle).await;
    s.predict(&oracle, NAVI).await;
    // The humans Reveal early; the Bot User is still free to change its pick.
    s.predict(&s.alice, NAVI).await;
    s.predict(&s.bob, FAZE).await;
    s.app.clock.advance(Duration::minutes(5));
    assert!(s.card(&s.bob).await["lock"]["locked"].as_bool().unwrap());
    let written = predictions::upsert_for_bot(
        &s.app.state.db,
        LeagueId(s.league),
        MatchId(NAVI_FAZE),
        oracle_id,
        TeamId(FAZE),
        None,
        s.app.clock.now(),
    )
    .await;
    assert!(written.unwrap().is_some(), "an early Reveal is not a Hard Lock");
    // A human's write at that moment is refused, as ever.
    let alice = s.user_id(&s.alice).await;
    let refused = predictions::upsert(
        &s.app.state.db,
        LeagueId(s.league),
        MatchId(NAVI_FAZE),
        alice,
        TeamId(FAZE),
        None,
        s.app.clock.now(),
    )
    .await;
    assert_eq!(refused.unwrap(), None);

    // The Bot User checks 16 minutes before the start (open) ...
    s.app.clock.set(s.start - Duration::minutes(16));
    let checked_at = s.app.clock.now();
    // ... the poller stamps the Hard Lock a minute later ...
    s.app.clock.advance(Duration::minutes(1));
    poller::tick(&s.app.state).await;
    // ... then its write runs.
    let written = predictions::upsert_for_bot(
        &s.app.state.db,
        LeagueId(s.league),
        MatchId(NAVI_FAZE),
        oracle_id,
        TeamId(NAVI),
        None,
        checked_at,
    )
    .await;
    assert_eq!(written.unwrap(), None, "refused: the Match is Hard Locked");

    let card = s.card(&s.bob).await;
    assert_eq!(Setup::revealed_team(&card, oracle_id), Some(FAZE));
    assert_eq!(Setup::revealed_team(&card, alice), Some(NAVI));
}
