use std::collections::{BTreeMap, HashMap};
use std::sync::Mutex;

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::domain::{
    DataSourceKind, Event, EventId, EventSnapshot, Match, MatchId, MatchScore, MatchStatus,
};
use crate::ports::{MatchSource, SourceError};

/// A Data Source that says what the test scripted.
///
/// Script whole Events with [`put_event`](Self::put_event) (build them with
/// `EventSnapshotBuilder`), then change them between polls with
/// [`postpone`](Self::postpone), [`start`](Self::start),
/// [`finish`](Self::finish), [`cancel`](Self::cancel) or the general
/// [`update_event`](Self::update_event) / [`update_match`](Self::update_match).
/// [`fail_with`](Self::fail_with) makes it fail until
/// [`recover`](Self::recover). Compose several (one per
/// [`DataSourceKind`]) to test the fallback chain.
pub struct ScriptedSource {
    kind: DataSourceKind,
    inner: Mutex<Inner>,
}

#[derive(Default)]
struct Inner {
    events: BTreeMap<EventId, EventSnapshot>,
    failure: Option<SourceError>,
    list_events_calls: usize,
    fetch_event_calls: HashMap<EventId, usize>,
}

impl ScriptedSource {
    /// A scripted HLTV.
    pub fn new() -> ScriptedSource {
        ScriptedSource::of(DataSourceKind::Hltv)
    }

    /// A scripted source that reports its snapshots as coming from `kind`.
    pub fn of(kind: DataSourceKind) -> ScriptedSource {
        ScriptedSource {
            kind,
            inner: Mutex::new(Inner::default()),
        }
    }

    pub fn kind(&self) -> DataSourceKind {
        self.kind
    }

    /// Adds or replaces an Event. Its `source` becomes this source's kind.
    pub fn put_event(&self, mut snapshot: EventSnapshot) {
        snapshot.source = self.kind;
        self.inner
            .lock()
            .unwrap()
            .events
            .insert(snapshot.event.id, snapshot);
    }

    pub fn remove_event(&self, event: EventId) {
        self.inner.lock().unwrap().events.remove(&event);
    }

    /// Changes a scripted Event in place (e.g. push a newly paired Swiss
    /// round onto `matches`). Panics if the Event was never scripted.
    pub fn update_event(&self, event: EventId, change: impl FnOnce(&mut EventSnapshot)) {
        let mut inner = self.inner.lock().unwrap();
        let snapshot = inner
            .events
            .get_mut(&event)
            .unwrap_or_else(|| panic!("ScriptedSource: Event {event} was not scripted"));
        change(snapshot);
    }

    /// Changes one scripted Match in place. Panics if it was never scripted.
    pub fn update_match(&self, event: EventId, id: MatchId, change: impl FnOnce(&mut Match)) {
        self.update_event(event, |snapshot| {
            let m = snapshot
                .matches
                .iter_mut()
                .find(|m| m.id == id)
                .unwrap_or_else(|| panic!("ScriptedSource: Match {id} was not scripted"));
            change(m);
        });
    }

    /// Moves the Match's Scheduled Start.
    pub fn postpone(&self, event: EventId, id: MatchId, new_start: DateTime<Utc>) {
        self.update_match(event, id, |m| m.scheduled_start = Some(new_start));
    }

    /// The Match goes live at `at` (use a time before its Scheduled Start for
    /// an early start).
    pub fn start(&self, event: EventId, id: MatchId, at: DateTime<Utc>) {
        self.update_match(event, id, |m| {
            m.status = MatchStatus::Live;
            m.actual_start = Some(at);
        });
    }

    /// The Match ends with `winner` (a team name) and `score` as
    /// `(team_a maps, team_b maps)`.
    pub fn finish(&self, event: EventId, id: MatchId, winner: &str, score: (u8, u8)) {
        self.update_event(event, |snapshot| {
            let winner_id = snapshot
                .team_named(winner)
                .unwrap_or_else(|| panic!("ScriptedSource: no team named {winner}"))
                .id;
            let m = snapshot
                .matches
                .iter_mut()
                .find(|m| m.id == id)
                .unwrap_or_else(|| panic!("ScriptedSource: Match {id} was not scripted"));
            assert!(
                m.teams().contains(&Some(winner_id)),
                "ScriptedSource: {winner} does not play in Match {id}"
            );
            m.status = MatchStatus::Finished;
            m.winner = Some(winner_id);
            m.score = Some(MatchScore {
                team_a: score.0,
                team_b: score.1,
            });
        });
    }

    /// The Match ends without a winner: a Void Match.
    pub fn finish_without_winner(&self, event: EventId, id: MatchId) {
        self.update_match(event, id, |m| {
            m.status = MatchStatus::Finished;
            m.winner = None;
        });
    }

    /// The Match is cancelled: a Void Match.
    pub fn cancel(&self, event: EventId, id: MatchId) {
        self.update_match(event, id, |m| {
            m.status = MatchStatus::Cancelled;
            m.winner = None;
        });
    }

    /// Every call fails with `error` until [`Self::recover`].
    pub fn fail_with(&self, error: SourceError) {
        self.inner.lock().unwrap().failure = Some(error);
    }

    pub fn recover(&self) {
        self.inner.lock().unwrap().failure = None;
    }

    /// How many times `list_events` was called (failed calls included).
    pub fn list_events_calls(&self) -> usize {
        self.inner.lock().unwrap().list_events_calls
    }

    /// How many times `fetch_event(event)` was called (failed calls included).
    pub fn fetch_event_calls(&self, event: EventId) -> usize {
        let inner = self.inner.lock().unwrap();
        inner.fetch_event_calls.get(&event).copied().unwrap_or(0)
    }

    /// How many times `fetch_event` was called for any Event.
    pub fn total_fetch_event_calls(&self) -> usize {
        self.inner.lock().unwrap().fetch_event_calls.values().sum()
    }
}

impl Default for ScriptedSource {
    fn default() -> ScriptedSource {
        ScriptedSource::new()
    }
}

#[async_trait]
impl MatchSource for ScriptedSource {
    async fn list_events(&self) -> Result<Vec<Event>, SourceError> {
        let mut inner = self.inner.lock().unwrap();
        inner.list_events_calls += 1;
        if let Some(error) = &inner.failure {
            return Err(error.clone());
        }
        Ok(inner.events.values().map(|s| s.event.clone()).collect())
    }

    async fn fetch_event(&self, id: EventId) -> Result<EventSnapshot, SourceError> {
        let mut inner = self.inner.lock().unwrap();
        *inner.fetch_event_calls.entry(id).or_default() += 1;
        if let Some(error) = &inner.failure {
            return Err(error.clone());
        }
        inner.events.get(&id).cloned().ok_or(SourceError::NotFound)
    }
}
