//! Seam 1: Bot Users and Pick'em (spec #21, the follow-up to ticket #22;
//! ADR-0003's amendment). The humans Lock a Stage's Pick'em early among
//! themselves; a Bot User neither holds that up nor takes part in the human
//! final edit window, and it may still create or replace its own Pick'em
//! until the Stage's Hard Lock (its first Match starting) while it is shown
//! nobody else's.
//!
//! Every rule is checked for both shapes: a Swiss Stage and a Playoffs
//! bracket.

use axum::http::StatusCode;
use chrono::{DateTime, Duration, Utc};
use kallbet::domain::pickem::{Pick, PickKind};
use kallbet::domain::{EventId, LeagueId, MatchId, StageKind, StageOrdinal, TeamId, UserId};
use kallbet::poller;
use kallbet::ports::Clock;
use kallbet::store::pickems;
use kallbet::testing::{EventSnapshotBuilder, TestApp, TestClient, TestResponse};
use serde_json::{Value, json};

const EVENT: i64 = 7148;

/// The two Pick'em shapes. Either way the Event has one Stage (ordinal 1)
/// whose opening Matches all have the same Scheduled Start.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Shape {
    /// 16 teams "T1".."T16" (ids 1..16): round 1 is Matches 101..=108, and
    /// T1's later Matches are 201 and 202.
    Swiss,
    /// 8 teams "T1".."T8" (ids 1..8): quarterfinals are Matches 1..=4, the
    /// semifinals 5 and 6, the final 7.
    Playoffs,
}

const SHAPES: [Shape; 2] = [Shape::Swiss, Shape::Playoffs];

fn swiss(three_nil: [i64; 2], zero_three: [i64; 2], advance: [i64; 6]) -> Value {
    json!({ "three_nil": three_nil, "zero_three": zero_three, "advance": advance })
}

/// A bracket body from `(rounds_before_final, slot, team)` triples, in the
/// order the API lists them (first round to final, top to bottom).
fn bracket(picks: &[(u8, u8, i64)]) -> Value {
    json!({ "picks": picks.iter()
        .map(|(r, s, t)| json!({ "rounds_before_final": r, "slot": s, "team_id": t }))
        .collect::<Vec<_>>() })
}

impl Shape {
    /// Scripts the Event, its opening Matches `start`ing four hours from now.
    fn event(self, app: &TestApp) -> DateTime<Utc> {
        let now = app.clock.now();
        let start = now + Duration::hours(4);
        let later = start + Duration::days(2);
        let event = EventSnapshotBuilder::new(EVENT, "Test Major");
        let event = match self {
            Shape::Swiss => {
                let mut event = event.stage(StageKind::Swiss, "Stage 1");
                for i in 0..8 {
                    event = event.swiss_match(
                        101 + i,
                        (1, 0, 0),
                        &format!("T{}", 2 * i + 1),
                        &format!("T{}", 2 * i + 2),
                        start,
                    );
                }
                event
                    .swiss_match(201, (2, 1, 0), "T1", "T3", later)
                    .swiss_match(202, (3, 2, 0), "T1", "T5", later)
            }
            Shape::Playoffs => event
                .stage(StageKind::Playoffs, "Playoffs")
                .playoff_match(1, (2, 0), "T1", "T2", start)
                .playoff_match(2, (2, 1), "T3", "T4", start)
                .playoff_match(3, (2, 2), "T5", "T6", start)
                .playoff_match(4, (2, 3), "T7", "T8", start)
                .playoff_slot(5, (1, 0), later)
                .playoff_slot(6, (1, 1), later)
                .playoff_slot(7, (0, 0), later + Duration::days(1)),
        };
        app.source.put_event(event.build(now));
        start
    }

    /// The Matches that open the Stage.
    fn opening(self) -> Vec<i64> {
        match self {
            Shape::Swiss => (101..=108).collect(),
            Shape::Playoffs => (1..=4).collect(),
        }
    }

    /// A valid Pick'em. Worth 1 Point after [`Self::early_results`].
    fn first(self) -> Value {
        match self {
            Shape::Swiss => swiss([1, 4], [16, 2], [7, 3, 5, 6, 9, 10]),
            Shape::Playoffs => bracket(&[
                (2, 0, 1),
                (2, 1, 3),
                (2, 2, 5),
                (2, 3, 7),
                (1, 0, 1),
                (1, 1, 7),
                (0, 0, 7),
            ]),
        }
    }

    /// Another valid Pick'em. Worth nothing after [`Self::early_results`].
    fn second(self) -> Value {
        match self {
            Shape::Swiss => swiss([2, 3], [15, 1], [7, 4, 5, 6, 9, 10]),
            Shape::Playoffs => bracket(&[
                (2, 0, 2),
                (2, 1, 4),
                (2, 2, 6),
                (2, 3, 8),
                (1, 0, 2),
                (1, 1, 6),
                (0, 0, 6),
            ]),
        }
    }

    /// Results reported before the Stage starts (forfeits whose Matches keep
    /// their planned start): T1 goes 3-0 in the Swiss Stage, or wins its
    /// quarterfinal. The Stage itself has not started.
    fn early_results(self, app: &TestApp, start: DateTime<Utc>) {
        let games: &[i64] = match self {
            Shape::Swiss => &[101, 201, 202],
            Shape::Playoffs => &[1],
        };
        for &game in games {
            app.source
                .update_match(EventId(EVENT), MatchId(game), |m| {
                    m.actual_start = Some(start)
                });
            app.source
                .finish(EventId(EVENT), MatchId(game), "T1", (2, 0));
        }
    }
}

struct Setup {
    app: TestApp,
    shape: Shape,
    alice: TestClient,
    bob: TestClient,
    /// The Bot User "Oracle", a Member.
    oracle: TestClient,
    league: i64,
    token: String,
    /// Scheduled Start of the Stage's opening Matches: its Hard Lock.
    start: DateTime<Utc>,
}

async fn join(client: &TestClient, token: &str) {
    let res = client
        .post_empty(&format!("/api/invites/{token}/join"))
        .await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
}

fn url(league: i64) -> String {
    format!("/api/leagues/{league}/stages/1/pickem")
}

async fn view_in(client: &TestClient, league: i64) -> Value {
    let res = client.get(&url(league)).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    res.json()
}

async fn submit_in(client: &TestClient, league: i64, body: &Value) {
    let res = client.put(&url(league), body).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
}

async fn user_id(client: &TestClient) -> i64 {
    client.get("/api/me").await.json()["id"].as_i64().unwrap()
}

/// A League of alice (creator) and bob, both human, and the Bot User Oracle,
/// which joined through the Invite Link; polled.
async fn setup(shape: Shape) -> Setup {
    let s = humans_only(shape).await;
    join(&s.oracle, &s.token).await;
    s
}

/// As [`setup`], but Oracle is not a Member: a League as it was before Bot
/// Users existed.
async fn humans_only(shape: Shape) -> Setup {
    let app = TestApp::new().await;
    let start = shape.event(&app);
    let alice = app.user("alice").await;
    let bob = app.user("bob").await;
    let oracle = app.bot("oracle", "Oracle").await;
    let res = alice
        .post(
            "/api/leagues",
            &json!({ "event_id": EVENT, "name": "Friends" }),
        )
        .await;
    assert_eq!(res.status, StatusCode::CREATED, "{}", res.text());
    let league = res.json();
    let token = league["invite_token"].as_str().unwrap().to_string();
    join(&bob, &token).await;
    poller::tick(&app.state).await;
    Setup {
        league: league["id"].as_i64().unwrap(),
        token,
        app,
        shape,
        alice,
        bob,
        oracle,
        start,
    }
}

impl Setup {
    async fn put(&self, client: &TestClient, body: &Value) -> TestResponse {
        client.put(&url(self.league), body).await
    }

    async fn submit(&self, client: &TestClient, body: &Value) {
        submit_in(client, self.league, body).await
    }

    async fn view(&self, client: &TestClient) -> Value {
        view_in(client, self.league).await
    }

    /// The Pick'em Leaderboard as `client` is shown it: `(nickname, points)`.
    async fn leaderboard(&self, client: &TestClient) -> Vec<(String, i64)> {
        let res = client
            .get(&format!("/api/leagues/{}/pickem-leaderboard", self.league))
            .await;
        assert_eq!(res.status, StatusCode::OK, "{}", res.text());
        let mut entries: Vec<(String, i64)> = res.json()["entries"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| {
                (
                    e["nickname"].as_str().unwrap().to_string(),
                    e["points"].as_i64().unwrap(),
                )
            })
            .collect();
        entries.sort();
        entries
    }

    /// alice (`first`) and bob (`second`) submit now and their final edit
    /// window runs out: the humans' early Lock. Returns its instant.
    async fn humans_lock_early(&self) -> DateTime<Utc> {
        self.submit(&self.alice, &self.shape.first()).await;
        self.submit(&self.bob, &self.shape.second()).await;
        let locked_at = self.app.clock.now() + Duration::minutes(5);
        self.app.clock.set(locked_at);
        let view = self.view(&self.alice).await;
        assert_eq!(view["lock"]["locked_at"], json!(locked_at), "{view}");
        locked_at
    }

    /// Moves every opening Match's Scheduled Start.
    fn postpone_opening(&self, to: DateTime<Utc>) {
        for game in self.shape.opening() {
            self.app.source.postpone(EventId(EVENT), MatchId(game), to);
        }
    }
}

/// Whether a Pick'em from the API holds exactly the picks of `body`.
fn holds(pickem: &Value, body: &Value) -> bool {
    body.as_object()
        .unwrap()
        .iter()
        .all(|(key, value)| &pickem[key] == value)
}

fn assert_locked(res: &TestResponse) {
    assert_eq!(res.status, StatusCode::CONFLICT, "{}", res.text());
    assert_eq!(res.error_code(), "pickem_locked");
}

/// The Nicknames whose Pick'ems the view reveals, sorted.
fn revealed(view: &Value) -> Vec<String> {
    let mut names: Vec<String> = view["pickems"]
        .as_array()
        .unwrap_or_else(|| panic!("no Pick'ems revealed: {view}"))
        .iter()
        .map(|p| p["nickname"].as_str().unwrap().to_string())
        .collect();
    names.sort();
    names
}

fn pickem_of<'a>(view: &'a Value, nickname: &str) -> &'a Value {
    view["pickems"]
        .as_array()
        .unwrap_or_else(|| panic!("no Pick'ems revealed: {view}"))
        .iter()
        .find(|p| p["nickname"] == nickname)
        .unwrap_or_else(|| panic!("no Pick'em of {nickname}: {view}"))
}

// --- Rule 1: the human quorum ----------------------------------------------

#[tokio::test]
async fn humans_lock_early_without_waiting_for_a_bot_user() {
    for shape in SHAPES {
        let s = setup(shape).await;
        // Oracle is a Member and submits nothing.
        s.submit(&s.alice, &shape.first()).await;
        s.submit(&s.bob, &shape.second()).await;
        let closes = s.app.clock.now() + Duration::minutes(5);

        s.app.clock.set(closes);

        let view = s.view(&s.alice).await;
        assert_eq!(view["lock"]["locked"], true, "{shape:?}: {view}");
        assert_eq!(view["lock"]["locked_at"], json!(closes), "{shape:?}");
        assert_eq!(revealed(&view), ["alice", "bob"], "{shape:?}");
        assert_locked(&s.put(&s.alice, &shape.second()).await);
    }
}

#[tokio::test]
async fn a_bot_user_that_submitted_does_not_lock_before_every_human_has() {
    for shape in SHAPES {
        let s = setup(shape).await;
        s.submit(&s.oracle, &shape.first()).await;
        s.submit(&s.alice, &shape.first()).await;

        s.app.clock.advance(Duration::minutes(30));
        poller::tick(&s.app.state).await;

        let view = s.view(&s.bob).await;
        assert_eq!(view["lock"]["locked"], false, "{shape:?}: {view}");
        assert_eq!(view["lock"]["edit_window"], false, "{shape:?}");
        assert_eq!(view["lock"]["locks_at"], json!(s.start), "{shape:?}");
        assert_eq!(view["pickems"], Value::Null, "{shape:?}");
        // alice is not shut out by a window either.
        s.submit(&s.alice, &shape.second()).await;
    }
}

#[tokio::test]
async fn kicking_the_last_human_who_has_not_submitted_locks_despite_a_silent_bot_user() {
    for shape in SHAPES {
        let s = setup(shape).await;
        s.submit(&s.alice, &shape.first()).await;
        let bob = user_id(&s.bob).await;

        let res = s
            .alice
            .delete(&format!("/api/leagues/{}/members/{bob}", s.league))
            .await;
        assert_eq!(res.status, StatusCode::NO_CONTENT, "{}", res.text());

        let view = s.view(&s.alice).await;
        assert_eq!(view["lock"]["locked"], true, "{shape:?}: {view}");
        assert_eq!(revealed(&view), ["alice"], "{shape:?}");
    }
}

// --- Rule 2: a League of Bot Users only ------------------------------------

#[tokio::test]
async fn a_league_of_only_bot_users_stays_open_until_the_stage_starts() {
    for shape in SHAPES {
        let app = TestApp::new().await;
        let start = shape.event(&app);
        let oracle = app.bot("oracle", "Oracle").await;
        let market = app.bot("market", "Market Favourite").await;
        let res = oracle
            .post(
                "/api/leagues",
                &json!({ "event_id": EVENT, "name": "Benchmark" }),
            )
            .await;
        assert_eq!(res.status, StatusCode::CREATED, "{}", res.text());
        let league = res.json()["id"].as_i64().unwrap();
        join(&market, res.json()["invite_token"].as_str().unwrap()).await;
        poller::tick(&app.state).await;

        // Every Member has submitted, and no human is there to Lock early.
        submit_in(&oracle, league, &shape.first()).await;
        submit_in(&market, league, &shape.second()).await;
        app.clock.advance(Duration::hours(1));
        poller::tick(&app.state).await;

        for bot in [&oracle, &market] {
            let view = view_in(bot, league).await;
            assert_eq!(view["lock"]["locked"], false, "{shape:?}: {view}");
            assert_eq!(view["lock"]["can_edit"], true, "{shape:?}");
            assert_eq!(view["lock"]["edit_window"], false, "{shape:?}");
            assert_eq!(view["lock"]["locks_at"], json!(start), "{shape:?}");
            assert_eq!(view["pickems"], Value::Null, "{shape:?}");
        }
        submit_in(&market, league, &shape.first()).await;

        // The Stage starts: Hard Lock.
        app.clock.set(start);
        assert_locked(&oracle.put(&url(league), &shape.second()).await);
        let view = view_in(&oracle, league).await;
        assert_eq!(view["lock"]["locked"], true, "{shape:?}: {view}");
        assert_eq!(view["lock"]["locked_at"], json!(start), "{shape:?}");
        assert_eq!(view["lock"]["can_edit"], false, "{shape:?}");
        assert_eq!(revealed(&view), ["Market Favourite", "Oracle"], "{shape:?}");
    }
}

// --- Rule 3: the human final edit window -----------------------------------

#[tokio::test]
async fn a_bot_users_submission_never_opens_the_final_edit_window() {
    for shape in SHAPES {
        let s = setup(shape).await;
        s.submit(&s.alice, &shape.first()).await;
        // Oracle is the last Member but one to submit, and bob never does.
        s.submit(&s.oracle, &shape.first()).await;

        let view = s.view(&s.alice).await;
        assert_eq!(view["lock"]["edit_window"], false, "{shape:?}: {view}");
        assert_eq!(view["lock"]["locks_at"], json!(s.start), "{shape:?}");
        s.app.clock.advance(Duration::minutes(10));
        let view = s.view(&s.alice).await;
        assert_eq!(view["lock"]["locked"], false, "{shape:?}: {view}");
        assert_eq!(view["lock"]["can_edit"], true, "{shape:?}");
        s.submit(&s.alice, &shape.second()).await;
    }
}

#[tokio::test]
async fn the_last_human_gets_the_final_edit_window_and_it_does_not_block_a_bot_user() {
    for shape in SHAPES {
        let s = setup(shape).await;
        s.submit(&s.alice, &shape.first()).await;
        s.submit(&s.bob, &shape.second()).await;
        let closes = s.app.clock.now() + Duration::minutes(5);

        // bob holds the window: alice is shut out, Oracle is not.
        s.app.clock.advance(Duration::minutes(1));
        assert_locked(&s.put(&s.alice, &shape.second()).await);
        s.submit(&s.oracle, &shape.first()).await;
        let as_bot = s.view(&s.oracle).await;
        assert_eq!(as_bot["lock"]["edit_window"], false, "{shape:?}: {as_bot}");
        assert_eq!(as_bot["lock"]["can_edit"], true, "{shape:?}");
        assert_eq!(as_bot["lock"]["locks_at"], json!(s.start), "{shape:?}");

        // Oracle's submission neither took the window nor moved it.
        let as_bob = s.view(&s.bob).await;
        assert_eq!(as_bob["lock"]["edit_window"], true, "{shape:?}: {as_bob}");
        assert_eq!(as_bob["lock"]["can_edit"], true, "{shape:?}");
        assert_eq!(as_bob["lock"]["locks_at"], json!(closes), "{shape:?}");
        assert_eq!(s.view(&s.alice).await["lock"]["can_edit"], false);
        s.submit(&s.bob, &shape.first()).await;

        s.app.clock.set(closes);
        let view = s.view(&s.alice).await;
        assert_eq!(view["lock"]["locked_at"], json!(closes), "{shape:?}: {view}");
        assert_locked(&s.put(&s.bob, &shape.second()).await);
    }
}

// --- Rule 4: a Bot User writes until the Hard Lock -------------------------

#[tokio::test]
async fn after_the_humans_early_lock_only_a_bot_user_can_still_submit_until_the_stage_starts() {
    for shape in SHAPES {
        let s = setup(shape).await;
        s.humans_lock_early().await;

        // Its first Pick'em, after the humans Locked.
        s.submit(&s.oracle, &shape.first()).await;
        let view = s.view(&s.oracle).await;
        assert!(holds(&view["my_pickem"], &shape.first()), "{shape:?}: {view}");
        assert_eq!(view["lock"]["locked"], false, "{shape:?}: {view}");
        assert_eq!(view["lock"]["locked_at"], Value::Null, "{shape:?}");
        assert_eq!(view["lock"]["can_edit"], true, "{shape:?}");
        assert_eq!(view["lock"]["locks_at"], json!(s.start), "{shape:?}");

        // The pre-Stage refresh, a minute before the first Match.
        s.app.clock.set(s.start - Duration::minutes(1));
        poller::tick(&s.app.state).await;
        s.submit(&s.oracle, &shape.second()).await;
        let view = s.view(&s.oracle).await;
        assert!(holds(&view["my_pickem"], &shape.second()), "{shape:?}: {view}");
        // The humans stay Locked.
        assert_locked(&s.put(&s.alice, &shape.second()).await);
        assert_locked(&s.put(&s.bob, &shape.first()).await);

        // The Stage starts.
        s.app.clock.set(s.start);
        assert_locked(&s.put(&s.oracle, &shape.first()).await);
        let view = s.view(&s.oracle).await;
        assert!(holds(&view["my_pickem"], &shape.second()), "{shape:?}: {view}");
        assert_eq!(view["lock"]["locked"], true, "{shape:?}: {view}");
        assert_eq!(view["lock"]["locked_at"], json!(s.start), "{shape:?}");
        assert_eq!(view["lock"]["can_edit"], false, "{shape:?}");
    }
}

#[tokio::test]
async fn a_bot_user_that_joins_after_the_humans_early_lock_can_submit_and_counts() {
    for shape in SHAPES {
        let s = setup(shape).await;
        s.humans_lock_early().await;
        s.app.clock.advance(Duration::minutes(30));
        let seer = s.app.bot("seer", "Seer").await;
        join(&seer, &s.token).await;
        // A human who joins now is too late, as ever.
        let carol = s.app.user("carol").await;
        join(&carol, &s.token).await;

        s.submit(&seer, &shape.first()).await;
        assert_locked(&s.put(&carol, &shape.first()).await);

        let view = s.view(&s.alice).await;
        assert_eq!(revealed(&view), ["Seer", "alice", "bob"], "{shape:?}");
        // alice, bob, Oracle and Seer; not carol.
        assert_eq!(view["eligible"], 4, "{shape:?}: {view}");
        assert_eq!(view["submitted"], 3, "{shape:?}: {view}");
    }
}

#[tokio::test]
async fn a_bot_user_that_joins_after_the_stage_started_cannot_submit() {
    for shape in SHAPES {
        let s = setup(shape).await;
        s.humans_lock_early().await;
        s.app.clock.set(s.start + Duration::minutes(1));
        poller::tick(&s.app.state).await;
        let seer = s.app.bot("seer", "Seer").await;
        join(&seer, &s.token).await;

        assert_locked(&s.put(&seer, &shape.first()).await);

        let view = s.view(&seer).await;
        assert_eq!(view["lock"]["locked"], true, "{shape:?}: {view}");
        assert_eq!(view["my_pickem"], Value::Null, "{shape:?}");
        // alice, bob and Oracle.
        assert_eq!(view["eligible"], 3, "{shape:?}: {view}");
    }
}

#[tokio::test]
async fn an_early_actual_start_is_the_hard_lock() {
    for shape in SHAPES {
        let s = setup(shape).await;
        s.humans_lock_early().await;
        s.submit(&s.oracle, &shape.first()).await;
        // An opening Match goes live hours before its Scheduled Start.
        s.app.clock.advance(Duration::minutes(30));
        let started = s.app.clock.now();
        s.app
            .source
            .start(EventId(EVENT), MatchId(shape.opening()[0]), started);
        poller::tick(&s.app.state).await;

        assert_locked(&s.put(&s.oracle, &shape.second()).await);
        let view = s.view(&s.oracle).await;
        assert_eq!(view["lock"]["locked_at"], json!(started), "{shape:?}: {view}");
        assert!(holds(&view["my_pickem"], &shape.first()), "{shape:?}: {view}");
    }
}

#[tokio::test]
async fn an_archived_league_rejects_a_bot_users_pickem() {
    let app = TestApp::new().await;
    let now = app.clock.now();
    // A Swiss Stage days away and a final that is about to be decided.
    let mut event =
        EventSnapshotBuilder::new(EVENT, "Test Major").stage(StageKind::Swiss, "Stage 1");
    for i in 0..8 {
        event = event.swiss_match(
            101 + i,
            (1, 0, 0),
            &format!("T{}", 2 * i + 1),
            &format!("T{}", 2 * i + 2),
            now + Duration::days(9),
        );
    }
    let event = event
        .stage(StageKind::Playoffs, "Playoffs")
        .playoff_match(301, (0, 0), "T1", "T2", now + Duration::days(1));
    app.source.put_event(event.build(now));
    let alice = app.user("alice").await;
    let oracle = app.bot("oracle", "Oracle").await;
    let res = alice
        .post(
            "/api/leagues",
            &json!({ "event_id": EVENT, "name": "Friends" }),
        )
        .await;
    let league = res.json()["id"].as_i64().unwrap();
    join(&oracle, res.json()["invite_token"].as_str().unwrap()).await;
    poller::tick(&app.state).await;
    submit_in(&oracle, league, &Shape::Swiss.first()).await;
    // alice, the only human, Locks early; Oracle may still write and is
    // shown nothing.
    submit_in(&alice, league, &Shape::Swiss.second()).await;
    app.clock.advance(Duration::minutes(5));
    assert_eq!(revealed(&view_in(&alice, league).await), ["Oracle", "alice"]);
    let view = view_in(&oracle, league).await;
    assert_eq!(view["pickems"], Value::Null, "{view}");
    assert_eq!(view["lock"]["can_edit"], true, "{view}");

    app.source
        .finish(EventId(EVENT), MatchId(301), "T2", (2, 0));
    app.clock.advance(Duration::minutes(6));
    poller::tick(&app.state).await;

    // The Stage is far from its Hard Lock, but the League is read-only.
    let res = oracle.put(&url(league), &Shape::Swiss.second()).await;
    assert_eq!(res.status, StatusCode::CONFLICT, "{}", res.text());
    assert_eq!(res.error_code(), "league_archived");
    let view = view_in(&oracle, league).await;
    assert!(holds(&view["my_pickem"], &Shape::Swiss.first()), "{view}");
    assert_eq!(view["lock"]["can_edit"], false, "{view}");
    // It can no longer change its Pick'em, so it sees what the humans see,
    // and they no longer see its Pick'em as preliminary.
    assert_eq!(revealed(&view), ["Oracle", "alice"]);
    let as_alice = view_in(&alice, league).await;
    assert_eq!(pickem_of(&as_alice, "Oracle")["preliminary"], false);
}

// --- Rule 5: a Bot User is shown nobody else's Pick'em before Hard Lock ----

#[tokio::test]
async fn before_the_stage_starts_a_bot_user_is_shown_no_other_members_pickem() {
    for shape in SHAPES {
        let s = setup(shape).await;
        let market = s.app.bot("market", "Market Favourite").await;
        join(&market, &s.token).await;
        s.submit(&s.oracle, &shape.first()).await;
        s.submit(&market, &shape.second()).await;
        let early = s.humans_lock_early().await;

        // The humans see each other's, and the Bot Users'.
        let as_alice = s.view(&s.alice).await;
        assert_eq!(
            revealed(&as_alice),
            ["Market Favourite", "Oracle", "alice", "bob"],
            "{shape:?}"
        );

        // Oracle is shown its own Pick'em and nobody else's: not the
        // humans', not the other Bot User's.
        let res = s.oracle.get(&url(s.league)).await;
        let text = res.text();
        let view = res.json();
        assert_eq!(view["pickems"], Value::Null, "{shape:?}: {view}");
        for name in ["alice", "bob", "Market"] {
            assert!(!text.contains(name), "{shape:?}: {name} leaked: {text}");
        }
        assert!(holds(&view["my_pickem"], &shape.first()), "{shape:?}: {view}");
        // Nor is it told that the humans Locked.
        assert_eq!(view["lock"]["locked"], false, "{shape:?}: {view}");
        assert_eq!(view["lock"]["locked_at"], Value::Null, "{shape:?}");
        assert!(!text.contains(&json!(early).to_string()), "{shape:?}: {text}");

        // Still nothing after it refreshes, in the answer or the next view.
        s.app.clock.set(s.start - Duration::minutes(1));
        let res = s.put(&s.oracle, &shape.second()).await;
        assert_eq!(res.status, StatusCode::OK, "{}", res.text());
        let text = res.text() + &s.oracle.get(&url(s.league)).await.text();
        for name in ["alice", "bob", "Market"] {
            assert!(!text.contains(name), "{shape:?}: {name} leaked: {text}");
        }

        // From the Hard Lock on it sees what everyone sees.
        s.app.clock.set(s.start);
        let view = s.view(&s.oracle).await;
        assert_eq!(
            revealed(&view),
            ["Market Favourite", "Oracle", "alice", "bob"],
            "{shape:?}"
        );
        assert!(holds(pickem_of(&view, "alice"), &shape.first()), "{shape:?}");
        assert!(holds(pickem_of(&view, "bob"), &shape.second()), "{shape:?}");
    }
}

#[tokio::test]
async fn before_the_stage_starts_the_pickem_leaderboard_tells_a_bot_user_nothing() {
    for shape in SHAPES {
        let s = setup(shape).await;
        s.submit(&s.oracle, &shape.first()).await;
        s.humans_lock_early().await;
        // Results arrive before the Stage starts (forfeits): `first` now
        // scores a Point, `second` none.
        s.app.clock.advance(Duration::minutes(30));
        shape.early_results(&s.app, s.start);
        poller::tick(&s.app.state).await;
        let as_bot = s.view(&s.oracle).await;
        assert_eq!(as_bot["lock"]["locked"], false, "{shape:?}: {as_bot}");

        // The humans are Locked, so their Points show (to them).
        let scored = vec![
            ("Oracle".to_string(), 1),
            ("alice".to_string(), 1),
            ("bob".to_string(), 0),
        ];
        assert_eq!(s.leaderboard(&s.alice).await, scored, "{shape:?}");
        // A Point would tell Oracle that alice picked as it did and bob did
        // not, while it can still change its own.
        let blank = vec![
            ("Oracle".to_string(), 0),
            ("alice".to_string(), 0),
            ("bob".to_string(), 0),
        ];
        assert_eq!(s.leaderboard(&s.oracle).await, blank, "{shape:?}");
        s.submit(&s.oracle, &shape.first()).await;

        // From the Hard Lock on it sees the same Leaderboard.
        s.app.clock.set(s.start);
        poller::tick(&s.app.state).await;
        assert_eq!(s.leaderboard(&s.oracle).await, scored, "{shape:?}");
        assert_eq!(s.leaderboard(&s.alice).await, scored, "{shape:?}");
    }
}

#[tokio::test]
async fn a_revealed_bot_users_pickem_is_marked_and_preliminary_until_the_stage_starts() {
    for shape in SHAPES {
        let s = setup(shape).await;
        s.submit(&s.oracle, &shape.first()).await;
        s.humans_lock_early().await;

        let view = s.view(&s.bob).await;
        let oracle = pickem_of(&view, "Oracle");
        assert_eq!(oracle["is_bot"], true, "{shape:?}: {view}");
        assert_eq!(oracle["preliminary"], true, "{shape:?}");
        let alice = pickem_of(&view, "alice");
        assert_eq!(alice["is_bot"], false, "{shape:?}");
        assert_eq!(alice["preliminary"], false, "{shape:?}");

        // Its refresh is what the humans see next.
        s.submit(&s.oracle, &shape.second()).await;
        let view = s.view(&s.bob).await;
        assert!(holds(pickem_of(&view, "Oracle"), &shape.second()), "{shape:?}");

        s.app.clock.set(s.start);
        let view = s.view(&s.bob).await;
        let oracle = pickem_of(&view, "Oracle");
        assert_eq!(oracle["is_bot"], true, "{shape:?}: {view}");
        assert_eq!(oracle["preliminary"], false, "{shape:?}");
    }
}

// --- Rule 6: a stored Hard Lock is never undone ----------------------------

#[tokio::test]
async fn the_hard_lock_is_never_undone_by_a_postponement() {
    for shape in SHAPES {
        let s = setup(shape).await;
        s.humans_lock_early().await;
        s.submit(&s.oracle, &shape.first()).await;
        // The Scheduled Start arrives; the poller runs, nobody asks.
        s.app.clock.set(s.start);
        poller::tick(&s.app.state).await;
        // Then every opening Match is postponed by days.
        s.postpone_opening(s.start + Duration::days(3));
        s.app.clock.advance(Duration::minutes(10));
        poller::tick(&s.app.state).await;

        assert_locked(&s.put(&s.oracle, &shape.second()).await);
        let view = s.view(&s.oracle).await;
        assert_eq!(view["lock"]["locked"], true, "{shape:?}: {view}");
        assert_eq!(view["lock"]["locked_at"], json!(s.start), "{shape:?}");
        assert_eq!(view["lock"]["locks_at"], json!(s.start), "{shape:?}");
        assert!(holds(&view["my_pickem"], &shape.first()), "{shape:?}: {view}");
        assert_eq!(revealed(&view), ["Oracle", "alice", "bob"], "{shape:?}");
    }
}

#[tokio::test]
async fn a_postponement_before_the_hard_lock_gives_a_bot_user_the_new_deadline() {
    for shape in SHAPES {
        let s = setup(shape).await;
        let early = s.humans_lock_early().await;
        let moved = s.start + Duration::days(1);
        s.app.clock.advance(Duration::minutes(30));
        s.postpone_opening(moved);
        poller::tick(&s.app.state).await;

        // An hour past the old Scheduled Start.
        s.app.clock.set(s.start + Duration::hours(1));
        poller::tick(&s.app.state).await;
        s.submit(&s.oracle, &shape.first()).await;
        let view = s.view(&s.oracle).await;
        assert_eq!(view["lock"]["locked"], false, "{shape:?}: {view}");
        assert_eq!(view["lock"]["locks_at"], json!(moved), "{shape:?}");
        assert_eq!(view["pickems"], Value::Null, "{shape:?}");
        // The humans' early Lock is a Lock all the same: it stays.
        assert_locked(&s.put(&s.alice, &shape.second()).await);
        let as_alice = s.view(&s.alice).await;
        assert_eq!(as_alice["lock"]["locked_at"], json!(early), "{shape:?}");

        s.app.clock.set(moved);
        assert_locked(&s.put(&s.oracle, &shape.second()).await);
    }
}

// --- Locks stored before Bot Users existed (migration 2201, no backfill) ---

/// Makes every stored Pick'em Lock look as it did before migration 2201: the
/// `hard_locked_at` column exists and is empty. Set-up only; the tests assert
/// through the API.
async fn as_stored_before_the_hard_lock_existed(s: &Setup) {
    sqlx::query("UPDATE pickem_locks SET hard_locked_at = NULL")
        .execute(&s.app.state.db)
        .await
        .unwrap();
}

#[tokio::test]
async fn a_stage_the_humans_locked_early_before_bot_users_existed_is_open_to_one_until_it_starts() {
    for shape in SHAPES {
        // No Bot User is a Member: the humans' early Lock is stored exactly
        // as an old row is, with no Hard Lock.
        let s = humans_only(shape).await;
        s.humans_lock_early().await;
        as_stored_before_the_hard_lock_existed(&s).await;
        s.app.clock.advance(Duration::minutes(30));
        poller::tick(&s.app.state).await;

        // Bot Users arrive; one joins and submits.
        join(&s.oracle, &s.token).await;
        s.submit(&s.oracle, &shape.first()).await;
        let view = s.view(&s.oracle).await;
        assert_eq!(view["lock"]["locked"], false, "{shape:?}: {view}");
        assert_eq!(view["lock"]["locks_at"], json!(s.start), "{shape:?}");
        assert_eq!(view["pickems"], Value::Null, "{shape:?}");
        assert_eq!(
            revealed(&s.view(&s.alice).await),
            ["Oracle", "alice", "bob"],
            "{shape:?}"
        );
        // The humans' Lock is untouched.
        assert_locked(&s.put(&s.alice, &shape.second()).await);

        // And it is refused once the Stage starts.
        s.app.clock.set(s.start);
        assert_locked(&s.put(&s.oracle, &shape.second()).await);
        let view = s.view(&s.oracle).await;
        assert!(holds(&view["my_pickem"], &shape.first()), "{shape:?}: {view}");
        assert_eq!(view["lock"]["locked_at"], json!(s.start), "{shape:?}");
    }
}

#[tokio::test]
async fn a_started_stage_locked_before_bot_users_existed_is_closed_to_one() {
    for shape in SHAPES {
        let s = humans_only(shape).await;
        s.submit(&s.alice, &shape.first()).await;
        // The Stage started and its Lock was stored, before the deploy.
        s.app.clock.set(s.start + Duration::minutes(10));
        poller::tick(&s.app.state).await;
        as_stored_before_the_hard_lock_existed(&s).await;

        // The Bot User's own request stamps the Hard Lock before it checks.
        s.app.clock.advance(Duration::hours(1));
        join(&s.oracle, &s.token).await;
        assert_locked(&s.put(&s.oracle, &shape.first()).await);

        let view = s.view(&s.oracle).await;
        assert_eq!(view["lock"]["locked"], true, "{shape:?}: {view}");
        assert_eq!(view["lock"]["locked_at"], json!(s.start), "{shape:?}");
        assert_eq!(view["my_pickem"], Value::Null, "{shape:?}");
        // The Stage has started, so it sees what the humans see.
        assert_eq!(revealed(&view), ["alice"], "{shape:?}");
    }
}

/// The accepted edge of "no backfill": an old Lock stamped because the
/// Scheduled Start arrived, after which the whole Stage was postponed with
/// no Match started. No Bot User existed to see a Pick'em of it, so one may
/// still enter, blind, until the new start. The humans stay Locked.
#[tokio::test]
async fn an_old_lock_of_a_stage_postponed_before_it_started_is_open_to_a_bot_user_until_the_new_start()
 {
    for shape in SHAPES {
        let s = humans_only(shape).await;
        s.submit(&s.alice, &shape.first()).await;
        s.app.clock.set(s.start);
        poller::tick(&s.app.state).await;
        let moved = s.start + Duration::days(1);
        s.postpone_opening(moved);
        s.app.clock.advance(Duration::minutes(10));
        poller::tick(&s.app.state).await;
        as_stored_before_the_hard_lock_existed(&s).await;

        join(&s.oracle, &s.token).await;
        s.submit(&s.oracle, &shape.second()).await;
        let view = s.view(&s.oracle).await;
        assert_eq!(view["lock"]["locked"], false, "{shape:?}: {view}");
        assert_eq!(view["lock"]["locks_at"], json!(moved), "{shape:?}");
        assert_eq!(view["pickems"], Value::Null, "{shape:?}");
        let as_alice = s.view(&s.alice).await;
        assert_eq!(as_alice["lock"]["locked_at"], json!(s.start), "{shape:?}");
        assert_locked(&s.put(&s.alice, &shape.second()).await);

        s.app.clock.set(moved);
        assert_locked(&s.put(&s.oracle, &shape.first()).await);
    }
}

/// The same guarantee as for a human's write (`tests/lock_atomicity.rs`): a
/// Hard Lock stamped between a Bot User's check and its write wins. The
/// store function is shape-generic.
#[tokio::test]
async fn a_bot_users_write_after_the_poller_stamped_the_hard_lock_does_not_land() {
    let s = setup(Shape::Swiss).await;
    let league = LeagueId(s.league);
    let oracle = UserId(user_id(&s.oracle).await);
    let alice = UserId(user_id(&s.alice).await);
    let pick = |team| {
        [Pick {
            kind: PickKind::Advance,
            slot: 0,
            team: TeamId(team),
        }]
    };
    s.humans_lock_early().await;
    let mut conn = s.app.state.db.acquire().await.unwrap();
    let now = s.app.clock.now();
    let saved =
        pickems::save_for_bot(&mut conn, league, StageOrdinal(1), oracle, &pick(1), now).await;
    assert!(saved.unwrap(), "an early Lock is not a Hard Lock");
    // A human's write at that moment is refused, as ever.
    let saved = pickems::save(&mut conn, league, StageOrdinal(1), alice, &pick(2), now).await;
    assert!(!saved.unwrap());
    drop(conn);

    // The Bot User checks a minute before the Stage starts (open) ...
    s.app.clock.set(s.start - Duration::minutes(1));
    let checked_at = s.app.clock.now();
    // ... the poller stamps the Hard Lock ...
    s.app.clock.set(s.start);
    poller::tick(&s.app.state).await;
    // ... then its write runs.
    let mut conn = s.app.state.db.acquire().await.unwrap();
    let saved = pickems::save_for_bot(
        &mut conn,
        league,
        StageOrdinal(1),
        oracle,
        &pick(2),
        checked_at,
    )
    .await;
    assert!(!saved.unwrap(), "refused: the Stage is Hard Locked");
    drop(conn);

    let view = s.view(&s.bob).await;
    assert_eq!(pickem_of(&view, "Oracle")["advance"], json!([1]), "{view}");
    assert!(holds(pickem_of(&view, "alice"), &Shape::Swiss.first()), "{view}");
}
