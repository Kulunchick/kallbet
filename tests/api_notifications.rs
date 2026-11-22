//! Seam 1: Telegram notifications (spec stories 67-70): the reminder 30
//! minutes before Lock, the Reveal announcement and the daily digest, with
//! the fake clock and the recording notifier.

use axum::http::StatusCode;
use chrono::{DateTime, Duration, Utc};
use kallbet::domain::{StageKind, UserId};
use kallbet::poller;
use kallbet::ports::{Clock, Notification, NotificationKind};
use kallbet::testing::{EventSnapshotBuilder, TestApp, TestClient, TestResponse};
use serde_json::json;

const EVENT: i64 = 7148;
const NAVI_FAZE: i64 = 101;
const VITALITY_G2: i64 = 102;
// Teams are numbered in order of appearance.
const NAVI: i64 = 1;
const FAZE: i64 = 2;
const VITALITY: i64 = 3;
const G2: i64 = 4;

struct Setup {
    app: TestApp,
    alice: TestClient,
    bob: TestClient,
    carol: TestClient,
    league: i64,
    /// Scheduled Start of NAVI vs FaZe (Lock is 15 minutes earlier).
    start: DateTime<Utc>,
}

/// A League "Friends" of alice (creator), bob and carol; NAVI vs FaZe two
/// hours from now, Vitality vs G2 an hour later; polled once.
async fn setup() -> Setup {
    let app = TestApp::new().await;
    let now = app.clock.now();
    let start = now + Duration::hours(2);
    app.source.put_event(
        EventSnapshotBuilder::new(EVENT, "Test Major")
            .stage(StageKind::Swiss, "Stage 1")
            .swiss_match(NAVI_FAZE, (1, 0, 0), "NAVI", "FaZe", start)
            .swiss_match(
                VITALITY_G2,
                (1, 0, 0),
                "Vitality",
                "G2",
                start + Duration::hours(1),
            )
            .build(now),
    );
    let alice = app.user("alice").await;
    let bob = app.user("bob").await;
    let carol = app.user("carol").await;
    let league = create_league(&alice, "Friends").await;
    join(&alice, league, &[&bob, &carol]).await;
    poller::tick(&app.state).await;
    Setup {
        app,
        alice,
        bob,
        carol,
        league,
        start,
    }
}

async fn create_league(owner: &TestClient, name: &str) -> i64 {
    let res = owner
        .post("/api/leagues", &json!({ "event_id": EVENT, "name": name }))
        .await;
    assert_eq!(res.status, StatusCode::CREATED, "{}", res.text());
    res.json()["id"].as_i64().unwrap()
}

/// Makes `others` join `league` through its Invite Link (read by `owner`).
async fn join(owner: &TestClient, league: i64, others: &[&TestClient]) {
    let token = owner.get(&format!("/api/leagues/{league}")).await.json()["invite_token"]
        .as_str()
        .unwrap()
        .to_string();
    for client in others {
        let res = client
            .post_empty(&format!("/api/invites/{token}/join"))
            .await;
        assert!(res.status.is_success(), "{}", res.text());
    }
}

impl Setup {
    async fn predict(&self, client: &TestClient, game: i64, team: i64) -> TestResponse {
        let res = client
            .put(
                &format!("/api/leagues/{}/matches/{game}/prediction", self.league),
                &json!({ "team_id": team }),
            )
            .await;
        assert_eq!(res.status, StatusCode::OK, "{}", res.text());
        res
    }

    /// Moves the clock to `at` and runs one poller pass.
    async fn tick_at(&self, at: DateTime<Utc>) {
        self.app.clock.set(at);
        poller::tick(&self.app.state).await;
    }

    async fn notes_for(&self, client: &TestClient, kind: NotificationKind) -> Vec<Notification> {
        let id = client.get("/api/me").await.json()["id"].as_i64().unwrap();
        self.app
            .notifier
            .sent_to(UserId(id))
            .into_iter()
            .filter(|n| n.kind == kind)
            .collect()
    }

    /// The Lock of NAVI vs FaZe.
    fn lock(&self) -> DateTime<Utc> {
        self.start - Duration::minutes(15)
    }
}

impl Setup {
    /// Everyone predicts NAVI vs FaZe (alice and bob NAVI, carol FaZe): an
    /// early Reveal at the last Prediction.
    async fn everyone_predicts(&self) {
        self.predict(&self.alice, NAVI_FAZE, NAVI).await;
        self.predict(&self.bob, NAVI_FAZE, NAVI).await;
        self.predict(&self.carol, NAVI_FAZE, FAZE).await;
    }
}

// ------------------------------------------------------------------ reveal

#[tokio::test]
async fn an_early_reveal_is_announced_once_to_every_eligible_member() {
    let s = setup().await;
    s.everyone_predicts().await;
    s.app.clock.advance(Duration::minutes(5));

    poller::tick(&s.app.state).await;
    poller::tick(&s.app.state).await;
    s.tick_at(s.app.clock.now() + Duration::minutes(5)).await;

    for client in [&s.alice, &s.bob, &s.carol] {
        let notes = s.notes_for(client, NotificationKind::Reveal).await;
        assert_eq!(notes.len(), 1);
        assert!(notes[0].text.contains("Friends"), "{}", notes[0].text);
        assert!(
            notes[0].text.contains("NAVI 2 : 1 FaZe"),
            "{}",
            notes[0].text
        );
    }
}

#[tokio::test]
async fn a_reveal_at_lock_goes_to_members_who_did_not_predict_too() {
    let s = setup().await;
    s.predict(&s.alice, NAVI_FAZE, NAVI).await;

    s.tick_at(s.lock()).await;

    for client in [&s.alice, &s.bob, &s.carol] {
        assert_eq!(s.notes_for(client, NotificationKind::Reveal).await.len(), 1);
    }
}

#[tokio::test]
async fn nothing_is_announced_when_nobody_predicted() {
    let s = setup().await;

    s.tick_at(s.lock()).await;

    assert!(
        s.notes_for(&s.alice, NotificationKind::Reveal)
            .await
            .is_empty()
    );
}

#[tokio::test]
async fn a_member_who_joined_after_the_reveal_is_not_told_about_it() {
    let s = setup().await;
    s.everyone_predicts().await;
    s.app.clock.advance(Duration::minutes(6));
    let dave = s.app.user("dave").await;
    join(&s.alice, s.league, &[&dave]).await;

    poller::tick(&s.app.state).await;

    assert!(
        s.notes_for(&dave, NotificationKind::Reveal)
            .await
            .is_empty()
    );
    assert_eq!(s.notes_for(&s.bob, NotificationKind::Reveal).await.len(), 1);
}

#[tokio::test]
async fn turning_reveal_announcements_off_suppresses_them() {
    let s = setup().await;
    s.bob
        .patch("/api/me", &json!({ "notifications": { "reveal": false } }))
        .await;
    s.everyone_predicts().await;
    s.app.clock.advance(Duration::minutes(5));

    poller::tick(&s.app.state).await;

    assert!(
        s.notes_for(&s.bob, NotificationKind::Reveal)
            .await
            .is_empty()
    );
    assert_eq!(
        s.notes_for(&s.carol, NotificationKind::Reveal).await.len(),
        1
    );
}

#[tokio::test]
async fn a_reveal_that_happened_long_ago_is_not_announced() {
    let s = setup().await;
    s.predict(&s.alice, NAVI_FAZE, NAVI).await;
    // The Reveal at Lock is stored by whoever looks first (here a page load),
    // and the poller only gets to it hours later.
    s.app.clock.set(s.lock());
    s.alice
        .get(&format!("/api/leagues/{}/matches", s.league))
        .await;

    s.tick_at(s.lock() + Duration::hours(4)).await;

    assert!(
        s.app
            .notifier
            .sent()
            .iter()
            .all(|n| n.kind != NotificationKind::Reveal)
    );
}

#[tokio::test]
async fn a_cancelled_match_is_not_announced() {
    let s = setup().await;
    s.predict(&s.alice, NAVI_FAZE, NAVI).await;
    s.app.source.cancel(
        kallbet::domain::EventId(EVENT),
        kallbet::domain::MatchId(NAVI_FAZE),
    );

    s.tick_at(s.app.clock.now() + Duration::minutes(6)).await;

    assert!(
        s.app
            .notifier
            .sent()
            .iter()
            .all(|n| n.kind != NotificationKind::Reveal)
    );
}

// ------------------------------------------------------------------ digest

impl Setup {
    /// alice picks NAVI and Vitality, bob NAVI and G2, carol FaZe and G2.
    async fn predict_both_matches(&self) {
        for (client, first, second) in [
            (&self.alice, NAVI, VITALITY),
            (&self.bob, NAVI, G2),
            (&self.carol, FAZE, G2),
        ] {
            self.predict(client, NAVI_FAZE, first).await;
            self.predict(client, VITALITY_G2, second).await;
        }
    }

    /// NAVI beat FaZe; Vitality beat G2. alice 2 Points, bob 1, carol 0.
    fn finish_both(&self) {
        let event = kallbet::domain::EventId(EVENT);
        self.app
            .source
            .finish(event, kallbet::domain::MatchId(NAVI_FAZE), "NAVI", (2, 0));
        self.app.source.finish(
            event,
            kallbet::domain::MatchId(VITALITY_G2),
            "Vitality",
            (2, 1),
        );
    }

    /// The time at which the poller can see both results.
    fn evening(&self) -> DateTime<Utc> {
        self.start + Duration::hours(4)
    }
}

#[tokio::test]
async fn the_digest_comes_after_the_last_match_of_the_day_with_points_gained_and_place() {
    let s = setup().await;
    s.predict_both_matches().await;
    s.app.source.finish(
        kallbet::domain::EventId(EVENT),
        kallbet::domain::MatchId(NAVI_FAZE),
        "NAVI",
        (2, 0),
    );
    s.tick_at(s.start + Duration::hours(2)).await;
    assert!(
        s.notes_for(&s.alice, NotificationKind::DailyDigest)
            .await
            .is_empty(),
        "one Match of the day is still to be played"
    );

    s.finish_both();
    s.tick_at(s.evening()).await;

    let alice = s.notes_for(&s.alice, NotificationKind::DailyDigest).await;
    let bob = s.notes_for(&s.bob, NotificationKind::DailyDigest).await;
    let carol = s.notes_for(&s.carol, NotificationKind::DailyDigest).await;
    assert_eq!((alice.len(), bob.len(), carol.len()), (1, 1, 1));
    assert!(alice[0].text.contains("Friends"), "{}", alice[0].text);
    assert!(
        alice[0]
            .text
            .contains("Points today: 2. Place: 1, 2 in total"),
        "{}",
        alice[0].text
    );
    assert!(
        bob[0]
            .text
            .contains("Points today: 1. Place: 2, 1 in total"),
        "{}",
        bob[0].text
    );
    assert!(
        carol[0]
            .text
            .contains("Points today: 0. Place: 3, 0 in total"),
        "{}",
        carol[0].text
    );
}

#[tokio::test]
async fn the_digest_is_sent_once() {
    let s = setup().await;
    s.predict_both_matches().await;
    s.finish_both();

    for minutes in [0, 1, 10, 60] {
        s.tick_at(s.evening() + Duration::minutes(minutes)).await;
    }

    assert_eq!(
        s.app
            .notifier
            .sent()
            .iter()
            .filter(|n| n.kind == NotificationKind::DailyDigest)
            .count(),
        3
    );
}

#[tokio::test]
async fn turning_the_digest_off_suppresses_it() {
    let s = setup().await;
    s.predict_both_matches().await;
    s.bob
        .patch("/api/me", &json!({ "notifications": { "digest": false } }))
        .await;
    s.finish_both();

    s.tick_at(s.evening()).await;

    assert!(
        s.notes_for(&s.bob, NotificationKind::DailyDigest)
            .await
            .is_empty()
    );
    assert_eq!(
        s.notes_for(&s.alice, NotificationKind::DailyDigest)
            .await
            .len(),
        1
    );
}

#[tokio::test]
async fn the_digest_is_in_the_language_of_the_user() {
    let s = setup().await;
    s.predict_both_matches().await;
    s.carol.patch("/api/me", &json!({ "language": "uk" })).await;
    s.finish_both();

    s.tick_at(s.evening()).await;

    let carol = s.notes_for(&s.carol, NotificationKind::DailyDigest).await;
    assert!(
        carol[0].text.contains("Очки за день: 0"),
        "{}",
        carol[0].text
    );
}

#[tokio::test]
async fn a_match_postponed_to_another_day_does_not_hold_back_the_digest() {
    let s = setup().await;
    s.predict_both_matches().await;
    let event = kallbet::domain::EventId(EVENT);
    s.app
        .source
        .finish(event, kallbet::domain::MatchId(NAVI_FAZE), "NAVI", (2, 0));
    s.app.source.postpone(
        event,
        kallbet::domain::MatchId(VITALITY_G2),
        s.start + Duration::days(1),
    );

    s.tick_at(s.evening()).await;

    let alice = s.notes_for(&s.alice, NotificationKind::DailyDigest).await;
    assert_eq!(alice.len(), 1);
    assert!(
        alice[0].text.contains("Points today: 1"),
        "{}",
        alice[0].text
    );
}

#[tokio::test]
async fn a_day_with_only_void_matches_has_no_digest() {
    let s = setup().await;
    let event = kallbet::domain::EventId(EVENT);
    s.app
        .source
        .cancel(event, kallbet::domain::MatchId(NAVI_FAZE));
    s.app
        .source
        .cancel(event, kallbet::domain::MatchId(VITALITY_G2));

    s.tick_at(s.evening()).await;

    assert!(
        s.app
            .notifier
            .sent()
            .iter()
            .all(|n| n.kind != NotificationKind::DailyDigest)
    );
}

#[tokio::test]
async fn a_match_day_that_ended_long_ago_gets_no_digest() {
    let s = setup().await;
    s.predict_both_matches().await;
    s.finish_both();

    // The poller sees the results two days late (before the feature, or
    // after downtime): that day is history, not news.
    s.tick_at(s.start + Duration::days(2)).await;

    assert!(
        s.app
            .notifier
            .sent()
            .iter()
            .all(|n| n.kind != NotificationKind::DailyDigest)
    );
}

#[tokio::test]
async fn a_member_of_two_leagues_gets_a_digest_for_each_with_its_own_place() {
    let s = setup().await;
    let other = create_league(&s.alice, "Office").await;
    join(&s.alice, other, &[&s.carol]).await;
    s.predict_both_matches().await;
    // In "Office" carol predicts both right and alice not at all.
    for game in [NAVI_FAZE, VITALITY_G2] {
        let res = s
            .carol
            .put(
                &format!("/api/leagues/{other}/matches/{game}/prediction"),
                &json!({ "team_id": if game == NAVI_FAZE { NAVI } else { VITALITY } }),
            )
            .await;
        assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    }
    s.finish_both();

    s.tick_at(s.evening()).await;

    let carol = s.notes_for(&s.carol, NotificationKind::DailyDigest).await;
    assert_eq!(carol.len(), 2);
    let friends = carol.iter().find(|n| n.text.contains("Friends")).unwrap();
    let office = carol.iter().find(|n| n.text.contains("Office")).unwrap();
    assert!(friends.text.contains("Place: 3"), "{}", friends.text);
    assert!(
        office.text.contains("Points today: 2. Place: 1"),
        "{}",
        office.text
    );
}

// ---------------------------------------------------------------- reminder

#[tokio::test]
async fn reminder_goes_out_thirty_minutes_before_lock_to_members_without_a_prediction() {
    let s = setup().await;
    s.predict(&s.alice, NAVI_FAZE, NAVI).await;

    s.tick_at(s.lock() - Duration::minutes(31)).await;
    assert!(s.app.notifier.sent().is_empty(), "too early");

    s.tick_at(s.lock() - Duration::minutes(30)).await;

    let bob = s.notes_for(&s.bob, NotificationKind::Reminder).await;
    let carol = s.notes_for(&s.carol, NotificationKind::Reminder).await;
    assert_eq!(bob.len(), 1);
    assert_eq!(carol.len(), 1);
    assert!(
        bob[0].text.contains("Friends"),
        "names the League: {}",
        bob[0].text
    );
    assert!(bob[0].text.contains("NAVI") && bob[0].text.contains("FaZe"));
    assert!(
        s.notes_for(&s.alice, NotificationKind::Reminder)
            .await
            .is_empty(),
        "alice already predicted"
    );
}

#[tokio::test]
async fn all_notification_types_are_on_by_default_and_can_be_changed_one_at_a_time() {
    let app = TestApp::new().await;
    let alice = app.user("alice").await;
    assert_eq!(
        alice.get("/api/me").await.json()["notifications"],
        json!({ "reminder": true, "reveal": true, "digest": true })
    );

    let res = alice
        .patch("/api/me", &json!({ "notifications": { "digest": false } }))
        .await;

    assert_eq!(
        res.json()["notifications"],
        json!({ "reminder": true, "reveal": true, "digest": false })
    );
    assert_eq!(
        alice.get("/api/me").await.json()["notifications"],
        json!({ "reminder": true, "reveal": true, "digest": false })
    );
}

#[tokio::test]
async fn deleting_a_league_with_logged_notifications_still_works() {
    let s = setup().await;
    s.tick_at(s.lock() - Duration::minutes(30)).await;
    assert!(!s.app.notifier.sent().is_empty());

    let res = s
        .alice
        .post(
            &format!("/api/leagues/{}/delete", s.league),
            &json!({ "confirm_name": "Friends" }),
        )
        .await;

    assert!(res.status.is_success(), "{}", res.text());
}

#[tokio::test]
async fn re_running_the_poller_sends_no_duplicate_reminders() {
    let s = setup().await;
    s.tick_at(s.lock() - Duration::minutes(30)).await;
    assert_eq!(s.app.notifier.sent().len(), 3);

    for minutes in [29, 20, 10, 1] {
        s.tick_at(s.lock() - Duration::minutes(minutes)).await;
        s.tick_at(s.lock() - Duration::minutes(minutes) + Duration::seconds(20))
            .await;
    }

    assert_eq!(s.app.notifier.sent().len(), 3);
}

#[tokio::test]
async fn no_reminder_once_the_match_is_locked() {
    let s = setup().await;

    s.tick_at(s.lock()).await;

    assert!(s.app.notifier.sent().is_empty());
}

#[tokio::test]
async fn a_match_first_seen_inside_the_window_still_gets_a_late_reminder() {
    let s = setup().await;
    // The poller was "down" until ten minutes before Lock.

    s.tick_at(s.lock() - Duration::minutes(10)).await;

    let bob = s.notes_for(&s.bob, NotificationKind::Reminder).await;
    assert_eq!(bob.len(), 1);
    assert!(bob[0].text.contains("10 min"), "{}", bob[0].text);
}

#[tokio::test]
async fn turning_reminders_off_in_the_profile_suppresses_them() {
    let s = setup().await;
    let res = s
        .bob
        .patch(
            "/api/me",
            &json!({ "notifications": { "reminder": false } }),
        )
        .await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    assert_eq!(
        res.json()["notifications"],
        json!({ "reminder": false, "reveal": true, "digest": true })
    );

    s.tick_at(s.lock() - Duration::minutes(30)).await;

    assert!(
        s.notes_for(&s.bob, NotificationKind::Reminder)
            .await
            .is_empty()
    );
    assert_eq!(
        s.notes_for(&s.carol, NotificationKind::Reminder)
            .await
            .len(),
        1
    );
}

#[tokio::test]
async fn messages_use_the_language_of_the_user() {
    let s = setup().await;
    s.bob.patch("/api/me", &json!({ "language": "uk" })).await;
    s.carol.patch("/api/me", &json!({ "language": "ru" })).await;

    s.tick_at(s.lock() - Duration::minutes(30)).await;

    let bob = s.notes_for(&s.bob, NotificationKind::Reminder).await;
    let carol = s.notes_for(&s.carol, NotificationKind::Reminder).await;
    assert_eq!(bob[0].language, kallbet::domain::Language::Uk);
    assert!(bob[0].text.contains("закриється"), "{}", bob[0].text);
    assert_eq!(carol[0].language, kallbet::domain::Language::Ru);
    assert!(carol[0].text.contains("закроется"), "{}", carol[0].text);
}

#[tokio::test]
async fn a_member_of_two_leagues_on_the_event_is_reminded_in_each_by_name() {
    let s = setup().await;
    let other = create_league(&s.alice, "Office").await;
    join(&s.alice, other, &[&s.bob]).await;

    s.tick_at(s.lock() - Duration::minutes(30)).await;

    let bob = s.notes_for(&s.bob, NotificationKind::Reminder).await;
    assert_eq!(bob.len(), 2);
    assert!(bob.iter().any(|n| n.text.contains("Friends")));
    assert!(bob.iter().any(|n| n.text.contains("Office")));
    let carol = s.notes_for(&s.carol, NotificationKind::Reminder).await;
    assert_eq!(carol.len(), 1, "carol is only in Friends");
}

#[tokio::test]
async fn a_user_who_has_not_started_the_bot_is_not_retried_and_does_not_block_others() {
    let s = setup().await;
    let bob_id = s.bob.get("/api/me").await.json()["id"].as_i64().unwrap();
    s.app.notifier.make_unreachable(UserId(bob_id));

    s.tick_at(s.lock() - Duration::minutes(30)).await;
    s.tick_at(s.lock() - Duration::minutes(25)).await;

    assert_eq!(
        s.app.notifier.undeliverable().len(),
        1,
        "tried once, then given up"
    );
    assert_eq!(
        s.notes_for(&s.carol, NotificationKind::Reminder)
            .await
            .len(),
        1
    );
}

#[tokio::test]
async fn a_passing_send_failure_is_retried_on_a_later_pass() {
    let s = setup().await;
    s.app
        .notifier
        .fail_with(kallbet::ports::NotifyError("telegram is down".into()));
    s.tick_at(s.lock() - Duration::minutes(30)).await;
    assert!(s.app.notifier.sent().is_empty());

    s.app.notifier.recover();
    s.tick_at(s.lock() - Duration::minutes(29)).await;
    s.tick_at(s.lock() - Duration::minutes(28)).await;

    assert_eq!(
        s.notes_for(&s.bob, NotificationKind::Reminder).await.len(),
        1
    );
    assert_eq!(
        s.notes_for(&s.carol, NotificationKind::Reminder)
            .await
            .len(),
        1
    );
}

#[tokio::test]
async fn a_postponed_match_does_not_remind_the_same_member_twice() {
    let s = setup().await;
    s.tick_at(s.lock() - Duration::minutes(30)).await;
    let before = s.app.notifier.sent().len();

    s.app.source.postpone(
        kallbet::domain::EventId(EVENT),
        kallbet::domain::MatchId(NAVI_FAZE),
        s.start + Duration::days(1),
    );
    s.tick_at(s.lock() - Duration::minutes(20)).await;
    s.tick_at(s.lock() + Duration::days(1) - Duration::minutes(20))
        .await;

    assert_eq!(s.app.notifier.sent().len(), before);
}
