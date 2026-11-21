//! The test fakes for the three ports behave as the Seam 1 tests need them to.

use std::sync::Arc;

use chrono::{Duration, TimeZone, Utc};
use kallbet::domain::{
    DataSourceKind, EventId, Language, MatchFormat, MatchId, MatchPlacement, MatchStatus,
    StageKind, TelegramId, UserId,
};
use kallbet::ports::{
    Clock, MatchSource, Notification, NotificationKind, Notifier, SourceError,
};
use kallbet::testing::{EventSnapshotBuilder, FakeClock, RecordingNotifier, ScriptedSource};

#[test]
fn fake_clock_stands_still_until_advanced_or_set() {
    let start = Utc.with_ymd_and_hms(2026, 3, 1, 12, 0, 0).unwrap();
    let clock = FakeClock::at(start);
    assert_eq!(clock.now(), start);
    assert_eq!(clock.now(), start);

    clock.advance(Duration::minutes(15));
    assert_eq!(clock.now(), start + Duration::minutes(15));

    let later = Utc.with_ymd_and_hms(2026, 3, 2, 0, 0, 0).unwrap();
    clock.set(later);
    assert_eq!(clock.now(), later);
}

fn major(clock: &FakeClock) -> kallbet::domain::EventSnapshot {
    EventSnapshotBuilder::new(7148, "Test Major")
        .stage(StageKind::Swiss, "Stage 1")
        .swiss_match(101, (1, 0, 0), "NAVI", "FaZe", clock.now() + Duration::hours(2))
        .swiss_match(102, (1, 0, 0), "Vitality", "Spirit", clock.now() + Duration::hours(3))
        .stage(StageKind::Playoffs, "Playoffs")
        .playoff_match(201, (0, 0), "NAVI", "Vitality", clock.now() + Duration::days(5))
        .build(clock.now())
}

#[tokio::test]
async fn scripted_source_serves_the_scripted_events() {
    let clock = FakeClock::new();
    let source = ScriptedSource::new();
    source.put_event(major(&clock));

    let events = source.list_events().await.unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].id, EventId(7148));
    assert_eq!(events[0].name, "Test Major");

    let snapshot = source.fetch_event(EventId(7148)).await.unwrap();
    assert_eq!(snapshot.source, DataSourceKind::Hltv);
    assert_eq!(snapshot.stages.len(), 2);
    assert_eq!(snapshot.teams.len(), 4, "teams are shared across Stages by name");
    assert_eq!(snapshot.matches.len(), 3);

    let opener = snapshot.match_by_id(MatchId(101)).unwrap();
    assert_eq!(opener.format, MatchFormat::Bo3);
    assert_eq!(opener.status, MatchStatus::Upcoming);
    assert_eq!(
        opener.placement,
        MatchPlacement::Swiss { round: 1, wins: 0, losses: 0 }
    );
    let final_ = snapshot.match_by_id(MatchId(201)).unwrap();
    assert_eq!(
        final_.placement,
        MatchPlacement::Playoffs { rounds_before_final: 0, slot: 0 }
    );
    assert_eq!(snapshot.team_named("NAVI").map(|t| t.id), opener.team_a);

    assert_eq!(
        source.fetch_event(EventId(1)).await,
        Err(SourceError::NotFound)
    );
}

#[tokio::test]
async fn scripted_source_can_script_postponement_early_start_result_and_void() {
    let clock = FakeClock::new();
    let source = ScriptedSource::new();
    source.put_event(major(&clock));
    let event = EventId(7148);

    let postponed_to = clock.now() + Duration::hours(6);
    source.postpone(event, MatchId(101), postponed_to);
    source.start(event, MatchId(102), clock.now());
    let snapshot = source.fetch_event(event).await.unwrap();
    assert_eq!(
        snapshot.match_by_id(MatchId(101)).unwrap().scheduled_start,
        Some(postponed_to)
    );
    let live = snapshot.match_by_id(MatchId(102)).unwrap();
    assert_eq!(live.status, MatchStatus::Live);
    assert_eq!(live.actual_start, Some(clock.now()));

    source.finish(event, MatchId(102), "Spirit", (0, 2));
    source.cancel(event, MatchId(101));
    let snapshot = source.fetch_event(event).await.unwrap();
    let finished = snapshot.match_by_id(MatchId(102)).unwrap();
    assert_eq!(finished.status, MatchStatus::Finished);
    assert_eq!(finished.winner, snapshot.team_named("Spirit").map(|t| t.id));
    assert!(!finished.is_void());
    assert!(snapshot.match_by_id(MatchId(101)).unwrap().is_void());
}

#[tokio::test]
async fn scripted_source_can_fail_and_recover_and_counts_calls() {
    let clock = FakeClock::new();
    let source = ScriptedSource::of(DataSourceKind::PandaScore);
    source.put_event(major(&clock));

    source.fail_with(SourceError::Unavailable("cloudflare".into()));
    assert!(source.list_events().await.is_err());
    assert!(source.fetch_event(EventId(7148)).await.is_err());

    source.recover();
    let snapshot = source.fetch_event(EventId(7148)).await.unwrap();
    assert_eq!(snapshot.source, DataSourceKind::PandaScore);

    assert_eq!(source.list_events_calls(), 1);
    assert_eq!(source.fetch_event_calls(EventId(7148)), 2);
    assert_eq!(source.fetch_event_calls(EventId(1)), 0);
}

#[tokio::test]
async fn recording_notifier_records_what_was_sent() {
    let notifier = Arc::new(RecordingNotifier::new());
    let reminder = Notification {
        user_id: UserId(1),
        telegram_id: TelegramId(555),
        language: Language::Uk,
        kind: NotificationKind::Reminder,
        text: "NAVI vs FaZe".into(),
    };

    let port: Arc<dyn Notifier> = notifier.clone();
    port.send(reminder.clone()).await.unwrap();

    assert_eq!(notifier.sent(), vec![reminder.clone()]);
    assert_eq!(notifier.sent_to(UserId(1)), vec![reminder]);
    assert_eq!(notifier.sent_to(UserId(2)), vec![]);
}
