//! Seam 1: the Playoffs Pick'em (a bracket: the winner of every slot).

use axum::http::StatusCode;
use chrono::{DateTime, Duration, Utc};
use kallbet::poller;
use kallbet::domain::{EventId, MatchId, StageKind, TeamId};
use kallbet::ports::Clock;
use kallbet::testing::{EventSnapshotBuilder, TestApp, TestClient, TestResponse};
use serde_json::{Value, json};

const EVENT: i64 = 9001;

struct Setup {
    app: TestApp,
    alice: TestClient,
    bob: TestClient,
    league: i64,
    /// Scheduled Start of the first Playoffs Match.
    start: DateTime<Utc>,
}

/// Teams "T1".."Tn" are ids 1..n in order of appearance.
impl Setup {
    async fn with(app: TestApp, event: EventSnapshotBuilder, start: DateTime<Utc>) -> Setup {
        app.source.put_event(event.build(app.clock.now()));
        let alice = app.user("alice").await;
        let bob = app.user("bob").await;
        let league = alice
            .post("/api/leagues", &json!({ "event_id": EVENT, "name": "Friends" }))
            .await
            .json();
        let token = league["invite_token"].as_str().unwrap();
        bob.post_empty(&format!("/api/invites/{token}/join")).await;
        poller::tick(&app.state).await;
        Setup { league: league["id"].as_i64().unwrap(), app, alice, bob, start }
    }

    fn url(&self, stage: u8) -> String {
        format!("/api/leagues/{}/stages/{stage}/pickem", self.league)
    }

    async fn view(&self, client: &TestClient, stage: u8) -> Value {
        let res = client.get(&self.url(stage)).await;
        assert_eq!(res.status, StatusCode::OK, "{}", res.text());
        res.json()
    }
}

/// An 8-team bracket as the only Stage (Stage 1): quarterfinals T1-T2,
/// T3-T4, T5-T6, T7-T8; semifinals and final still to be decided.
async fn quarterfinals() -> Setup {
    let app = TestApp::new().await;
    let start = app.clock.now() + Duration::hours(2);
    let day = start + Duration::days(1);
    let event = EventSnapshotBuilder::new(EVENT, "Test Open")
        .stage(StageKind::Playoffs, "Playoffs")
        .playoff_match(1, (2, 0), "T1", "T2", start)
        .playoff_match(2, (2, 1), "T3", "T4", start)
        .playoff_match(3, (2, 2), "T5", "T6", start + Duration::hours(3))
        .playoff_match(4, (2, 3), "T7", "T8", start + Duration::hours(6))
        .playoff_slot(5, (1, 0), day)
        .playoff_slot(6, (1, 1), day)
        .playoff_slot(7, (0, 0), day + Duration::days(1));
    Setup::with(app, event, start).await
}

/// A bracket body from `(rounds_before_final, slot, team)` triples.
fn body(picks: &[(u8, u8, i64)]) -> Value {
    json!({ "picks": picks.iter()
        .map(|(r, s, t)| json!({ "rounds_before_final": r, "slot": s, "team_id": t }))
        .collect::<Vec<_>>() })
}

/// T1, T3, T5, T7 win the quarterfinals; T1 and T7 the semifinals; T7 the final.
fn valid() -> Value {
    body(&[(2, 0, 1), (2, 1, 3), (2, 2, 5), (2, 3, 7), (1, 0, 1), (1, 1, 7), (0, 0, 7)])
}

fn assert_rejected(res: &TestResponse, status: StatusCode, code: &str) {
    assert_eq!(res.status, status, "{}", res.text());
    assert_eq!(res.error_code(), code);
}

fn pick_of(view: &Value, round: u64, slot: u64) -> Option<i64> {
    view["my_pickem"]["picks"]
        .as_array()?
        .iter()
        .find(|p| p["rounds_before_final"] == round && p["slot"] == slot)?["team_id"]
        .as_i64()
}

#[tokio::test]
async fn a_member_fills_the_bracket_and_sees_it_back() {
    let s = quarterfinals().await;

    let res = s.alice.put(&s.url(1), &valid()).await;

    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    let view = s.view(&s.alice, 1).await;
    assert_eq!(view["my_pickem"]["shape"], "playoffs");
    assert_eq!(pick_of(&view, 2, 3), Some(7));
    assert_eq!(pick_of(&view, 1, 0), Some(1));
    assert_eq!(pick_of(&view, 0, 0), Some(7));
    // The bracket's shape: 4 first-round slots with known teams, 3 later ones.
    let slots = view["bracket"]["slots"].as_array().unwrap();
    assert_eq!(slots.len(), 7);
    assert_eq!(view["bracket"]["ready"], true);
    let qf = slots
        .iter()
        .find(|x| x["rounds_before_final"] == 2 && x["slot"] == 1)
        .unwrap();
    assert_eq!(qf["team_a"], 3);
    assert_eq!(qf["team_b"], 4);
    assert_eq!(s.view(&s.bob, 1).await["my_pickem"], Value::Null);
}

#[tokio::test]
async fn a_picked_winner_must_come_from_the_picks_below_it() {
    let s = quarterfinals().await;
    let unprocessable = StatusCode::UNPROCESSABLE_ENTITY;

    // A quarterfinal winner who is not in that Match.
    let b = body(&[(2, 0, 3), (2, 1, 3), (2, 2, 5), (2, 3, 7), (1, 0, 3), (1, 1, 7), (0, 0, 7)]);
    assert_rejected(&s.alice.put(&s.url(1), &b).await, unprocessable, "pickem_team_not_in_stage");
    // A semifinal winner who was not picked through the quarterfinals.
    let b = body(&[(2, 0, 1), (2, 1, 3), (2, 2, 5), (2, 3, 7), (1, 0, 5), (1, 1, 7), (0, 0, 7)]);
    assert_rejected(&s.alice.put(&s.url(1), &b).await, unprocessable, "pickem_team_not_in_stage");
    // A final winner who lost his semifinal in the picks.
    let b = body(&[(2, 0, 1), (2, 1, 3), (2, 2, 5), (2, 3, 7), (1, 0, 1), (1, 1, 7), (0, 0, 3)]);
    assert_rejected(&s.alice.put(&s.url(1), &b).await, unprocessable, "pickem_team_not_in_stage");
    // A slot left out, and a slot picked twice.
    let b = body(&[(2, 0, 1), (2, 1, 3), (2, 2, 5), (2, 3, 7), (1, 0, 1), (1, 1, 7)]);
    assert_rejected(&s.alice.put(&s.url(1), &b).await, unprocessable, "pickem_wrong_count");
    let b = body(&[(2, 0, 1), (2, 0, 1), (2, 2, 5), (2, 3, 7), (1, 0, 1), (1, 1, 7), (0, 0, 7)]);
    assert_rejected(&s.alice.put(&s.url(1), &b).await, unprocessable, "pickem_wrong_count");
    // A slot that is not part of the bracket.
    let b = body(&[
        (2, 0, 1), (2, 1, 3), (2, 2, 5), (2, 3, 7), (1, 0, 1), (1, 1, 7), (0, 0, 7), (3, 0, 1),
    ]);
    assert_rejected(&s.alice.put(&s.url(1), &b).await, unprocessable, "pickem_wrong_count");

    assert_eq!(s.view(&s.alice, 1).await["my_pickem"], Value::Null);
}

#[tokio::test]
async fn saving_again_replaces_the_bracket() {
    let s = quarterfinals().await;
    s.alice.put(&s.url(1), &valid()).await;
    let changed = body(&[(2, 0, 2), (2, 1, 4), (2, 2, 6), (2, 3, 8), (1, 0, 2), (1, 1, 8), (0, 0, 2)]);

    assert_eq!(s.alice.put(&s.url(1), &changed).await.status, StatusCode::OK);

    let view = s.view(&s.alice, 1).await;
    assert_eq!(view["my_pickem"]["picks"].as_array().unwrap().len(), 7);
    assert_eq!(pick_of(&view, 0, 0), Some(2));
}

#[tokio::test]
async fn other_members_brackets_are_hidden_until_the_first_match_starts() {
    let s = quarterfinals().await;
    s.alice.put(&s.url(1), &valid()).await;
    s.bob.put(&s.url(1), &valid()).await;

    // Everyone has picked, yet nothing about the others is returned and the
    // Pick'em is not Locked, even after the Matches' own Lock (15 minutes
    // before the start): the Pick'em Locks at the start itself.
    s.app.clock.set(s.start - Duration::minutes(1));
    let before = s.view(&s.bob, 1).await;
    assert_eq!(before["pickems"], Value::Null);
    assert_eq!(before["lock"]["locked"], false);
    assert!(!serde_json::to_string(&before).unwrap().contains("alice"));
    assert_eq!(s.bob.put(&s.url(1), &valid()).await.status, StatusCode::OK);

    s.app.clock.set(s.start);
    let after = s.view(&s.bob, 1).await;
    let all = after["pickems"].as_array().unwrap();
    assert_eq!(all.len(), 2);
    let alice = all.iter().find(|p| p["nickname"] == "alice").unwrap();
    assert_eq!(alice["shape"], "playoffs");
    assert_eq!(alice["picks"].as_array().unwrap().len(), 7);
}

#[tokio::test]
async fn edits_are_rejected_once_the_first_match_has_started_and_the_lock_survives() {
    let s = quarterfinals().await;
    s.alice.put(&s.url(1), &valid()).await;
    s.app.clock.set(s.start);
    poller::tick(&s.app.state).await;

    assert_rejected(&s.alice.put(&s.url(1), &valid()).await, StatusCode::CONFLICT, "pickem_locked");
    assert_rejected(&s.bob.put(&s.url(1), &valid()).await, StatusCode::CONFLICT, "pickem_locked");

    // Postponing the Matches afterwards does not reopen it.
    let event = EventId(EVENT);
    s.app.source.postpone(event, MatchId(1), s.start + Duration::days(3));
    s.app.source.postpone(event, MatchId(2), s.start + Duration::days(3));
    s.app.clock.advance(Duration::minutes(10));
    poller::tick(&s.app.state).await;
    assert_rejected(&s.alice.put(&s.url(1), &valid()).await, StatusCode::CONFLICT, "pickem_locked");
    assert_eq!(s.view(&s.alice, 1).await["lock"]["locked"], true);
}

/// Alice's `valid()` bracket against what happens: T1, T3, T6 and T7 win the
/// quarterfinals (T5, picked by Alice, loses), T1 and T7 the semifinals, T7
/// the final.
fn play_the_bracket(s: &Setup) {
    let event = EventId(EVENT);
    let src = &s.app.source;
    src.finish(event, MatchId(1), "T1", (2, 0));
    src.finish(event, MatchId(2), "T3", (2, 0));
    src.finish(event, MatchId(3), "T6", (2, 0));
    src.finish(event, MatchId(4), "T7", (2, 0));
    let teams = |a: i64, b: i64| move |m: &mut kallbet::domain::Match| {
        (m.team_a, m.team_b) = (Some(TeamId(a)), Some(TeamId(b)));
    };
    src.update_match(event, MatchId(5), teams(1, 3));
    src.update_match(event, MatchId(6), teams(6, 7));
    src.finish(event, MatchId(5), "T1", (2, 1));
    src.finish(event, MatchId(6), "T7", (2, 1));
    src.update_match(event, MatchId(7), teams(1, 7));
    src.finish(event, MatchId(7), "T7", (3, 2));
}

async fn leaderboard(s: &Setup) -> Vec<(i64, String, i64)> {
    let res = s
        .alice
        .get(&format!("/api/leagues/{}/pickem-leaderboard", s.league))
        .await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    res.json()["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| {
            (
                e["place"].as_i64().unwrap(),
                e["nickname"].as_str().unwrap().to_string(),
                e["points"].as_i64().unwrap(),
            )
        })
        .collect()
}

#[tokio::test]
async fn one_point_per_slot_whose_picked_winner_won_and_it_joins_the_leaderboard() {
    let s = quarterfinals().await;
    s.alice.put(&s.url(1), &valid()).await;
    // Bob picks the other team in every quarterfinal and semifinal.
    let bob = body(&[(2, 0, 2), (2, 1, 4), (2, 2, 6), (2, 3, 8), (1, 0, 2), (1, 1, 6), (0, 0, 6)]);
    assert_eq!(s.bob.put(&s.url(1), &bob).await.status, StatusCode::OK);
    s.app.clock.set(s.start);
    play_the_bracket(&s);
    poller::tick(&s.app.state).await;

    // Alice: QF T1, T3, T7 right (T5 wrong), both semifinals, the final = 6.
    // Bob: only QF T6 is right = 1.
    assert_eq!(leaderboard(&s).await, vec![(1, "alice".into(), 6), (2, "bob".into(), 1)]);
    let all = s.view(&s.alice, 1).await["pickems"].clone();
    let points = |name: &str| {
        all.as_array().unwrap().iter().find(|p| p["nickname"] == name).unwrap()["points"].clone()
    };
    assert_eq!(points("alice"), 6);
    assert_eq!(points("bob"), 1);
}

/// A6: a result that arrives before the Stage Locks (a forfeit reported
/// before the planned start) scores nothing until Lock, so Points cannot
/// leak a Member's unrevealed Picks.
#[tokio::test]
async fn a_result_before_the_stage_locks_scores_nothing_until_lock() {
    let s = quarterfinals().await;
    s.alice.put(&s.url(1), &valid()).await;
    // T2 forfeits the first quarterfinal an hour before its planned start.
    s.app.clock.advance(Duration::hours(1));
    let start = s.start;
    s.app.source.update_match(EventId(EVENT), MatchId(1), |m| m.actual_start = Some(start));
    s.app.source.finish(EventId(EVENT), MatchId(1), "T1", (2, 0));
    poller::tick(&s.app.state).await;
    assert_eq!(s.view(&s.bob, 1).await["lock"]["locked"], false, "precondition: open");

    assert_eq!(leaderboard(&s).await[0].2, 0, "a Point before Lock tells bob alice's Pick");

    s.app.clock.set(s.start);
    poller::tick(&s.app.state).await;
    assert_eq!(leaderboard(&s).await, vec![(1, "alice".into(), 1), (2, "bob".into(), 0)]);
}

#[tokio::test]
async fn a_void_slot_scores_nothing() {
    let s = quarterfinals().await;
    s.alice.put(&s.url(1), &valid()).await;
    s.app.clock.set(s.start);
    play_the_bracket(&s);
    s.app.source.cancel(EventId(EVENT), MatchId(7));
    poller::tick(&s.app.state).await;

    assert_eq!(leaderboard(&s).await[0], (1, "alice".into(), 5));
}

#[tokio::test]
async fn an_event_without_a_swiss_stage_offers_only_the_playoffs_pickem() {
    let app = TestApp::new().await;
    let start = app.clock.now() + Duration::hours(2);
    let event = EventSnapshotBuilder::new(EVENT, "Test Open")
        .stage(StageKind::Groups, "Groups")
        .unplaced_match(10, "T1", "T2", start)
        .stage(StageKind::Playoffs, "Playoffs")
        .playoff_match(1, (1, 0), "T1", "T2", start + Duration::days(1))
        .playoff_match(2, (1, 1), "T3", "T4", start + Duration::days(1))
        .playoff_slot(3, (0, 0), start + Duration::days(2));
    let s = Setup::with(app, event, start).await;

    assert_eq!(s.alice.get(&s.url(1)).await.status, StatusCode::NOT_FOUND);
    let b = body(&[(1, 0, 1), (1, 1, 3), (0, 0, 3)]);
    assert_eq!(s.alice.put(&s.url(1), &b).await.status, StatusCode::NOT_FOUND);
    let view = s.view(&s.alice, 2).await;
    assert_eq!(view["stage"]["kind"], "playoffs");
    // The Group Stage's first Match has started, which does not Lock the Playoffs.
    s.app.clock.set(start + Duration::hours(1));
    assert_eq!(s.alice.put(&s.url(2), &b).await.status, StatusCode::OK);
}

#[tokio::test]
async fn first_round_teams_still_tbd_cannot_be_picked_until_known() {
    let app = TestApp::new().await;
    let start = app.clock.now() + Duration::hours(2);
    // T3 and T4 exist (an unplaced Match names them) but their slot is TBD.
    let event = EventSnapshotBuilder::new(EVENT, "Test Open")
        .stage(StageKind::Playoffs, "Playoffs")
        .playoff_match(1, (1, 0), "T1", "T2", start)
        .playoff_slot(2, (1, 1), start)
        .playoff_slot(3, (0, 0), start + Duration::days(1))
        .unplaced_match(9, "T3", "T4", start + Duration::days(9));
    let s = Setup::with(app, event, start).await;

    let view = s.view(&s.alice, 1).await;
    assert_eq!(view["bracket"]["ready"], false);
    assert_eq!(view["bracket"]["slots"].as_array().unwrap().len(), 3);
    let b = body(&[(1, 0, 1), (1, 1, 3), (0, 0, 3)]);
    assert_rejected(
        &s.alice.put(&s.url(1), &b).await,
        StatusCode::UNPROCESSABLE_ENTITY,
        "pickem_bracket_not_ready",
    );

    // Once the teams are known it opens (while the Stage has not started).
    s.app.source.update_match(EventId(EVENT), MatchId(2), |m| {
        (m.team_a, m.team_b) = (Some(TeamId(3)), Some(TeamId(4)));
    });
    s.app.clock.advance(Duration::minutes(6));
    poller::tick(&s.app.state).await;
    assert_eq!(s.view(&s.alice, 1).await["bracket"]["ready"], true);
    assert_eq!(s.alice.put(&s.url(1), &b).await.status, StatusCode::OK);

    // A bracket that never became ready still Locks at the first start.
    let app = TestApp::new().await;
    let start = app.clock.now() + Duration::hours(2);
    let event = EventSnapshotBuilder::new(EVENT, "Test Open")
        .stage(StageKind::Playoffs, "Playoffs")
        .playoff_match(1, (1, 0), "T1", "T2", start)
        .playoff_slot(2, (1, 1), start)
        .playoff_slot(3, (0, 0), start + Duration::days(1));
    let s = Setup::with(app, event, start).await;
    s.app.clock.set(start);
    let view = s.view(&s.alice, 1).await;
    assert_eq!(view["lock"]["locked"], true);
}

#[tokio::test]
async fn four_and_sixteen_team_brackets_have_the_matching_number_of_slots() {
    // 4 teams: semifinals and a final.
    let app = TestApp::new().await;
    let start = app.clock.now() + Duration::hours(2);
    let event = EventSnapshotBuilder::new(EVENT, "Four")
        .stage(StageKind::Playoffs, "Playoffs")
        .playoff_match(1, (1, 0), "T1", "T2", start)
        .playoff_match(2, (1, 1), "T3", "T4", start)
        .playoff_slot(3, (0, 0), start + Duration::days(1));
    let s = Setup::with(app, event, start).await;
    let view = s.view(&s.alice, 1).await;
    assert_eq!(view["bracket"]["slots"].as_array().unwrap().len(), 3);
    let b = body(&[(1, 0, 2), (1, 1, 3), (0, 0, 2)]);
    assert_eq!(s.alice.put(&s.url(1), &b).await.status, StatusCode::OK);

    // 16 teams: 8 + 4 + 2 + 1 slots.
    let app = TestApp::new().await;
    let start = app.clock.now() + Duration::hours(2);
    let mut event =
        EventSnapshotBuilder::new(EVENT, "Sixteen").stage(StageKind::Playoffs, "Playoffs");
    for i in 0..8u8 {
        let (a, b) = (format!("T{}", 2 * i + 1), format!("T{}", 2 * i + 2));
        event = event.playoff_match(1 + i as i64, (3, i), &a, &b, start);
    }
    for r in 0..3u8 {
        for slot in 0..(1u8 << r) {
            let id = 100 + (r * 10 + slot) as i64;
            event = event.playoff_slot(id, (r, slot), start + Duration::days(1));
        }
    }
    let s = Setup::with(app, event, start).await;
    let view = s.view(&s.alice, 1).await;
    assert_eq!(view["bracket"]["slots"].as_array().unwrap().len(), 15);
    assert_eq!(view["teams"].as_array().unwrap().len(), 16);
    // Odd-numbered teams win every Match.
    let mut picks = Vec::new();
    for slot in 0..8u8 {
        picks.push((3, slot, 2 * slot as i64 + 1));
    }
    for slot in 0..4u8 {
        picks.push((2, slot, 4 * slot as i64 + 1));
    }
    for slot in 0..2u8 {
        picks.push((1, slot, 8 * slot as i64 + 1));
    }
    picks.push((0, 0, 1));
    assert_eq!(s.alice.put(&s.url(1), &body(&picks)).await.status, StatusCode::OK);
}
