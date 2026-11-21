use chrono::{DateTime, Utc};

use crate::domain::{
    DataSourceKind, Event, EventId, EventSnapshot, Match, MatchFormat, MatchId, MatchPlacement,
    MatchStatus, Stage, StageKind, StageOrdinal, Team, TeamId,
};

/// Builds an [`EventSnapshot`] for the `ScriptedSource` in a few lines.
///
/// Teams are named, not numbered: the first mention of a name creates the
/// team (ids 1, 2, 3, ... in order of appearance), later mentions reuse it.
/// Matches go into the Stage declared last and start out `Upcoming`.
///
/// ```
/// use chrono::{Duration, Utc};
/// use kallbet::domain::StageKind;
/// use kallbet::testing::EventSnapshotBuilder;
///
/// let now = Utc::now();
/// let snapshot = EventSnapshotBuilder::new(7148, "Test Major")
///     .stage(StageKind::Swiss, "Stage 1")
///     .swiss_match(101, (1, 0, 0), "NAVI", "FaZe", now + Duration::hours(2))
///     .stage(StageKind::Playoffs, "Playoffs")
///     .playoff_match(201, (0, 0), "NAVI", "FaZe", now + Duration::days(3))
///     .build(now);
/// assert_eq!(snapshot.matches.len(), 2);
/// ```
pub struct EventSnapshotBuilder {
    event: Event,
    stages: Vec<Stage>,
    teams: Vec<Team>,
    matches: Vec<Match>,
    format: MatchFormat,
}

impl EventSnapshotBuilder {
    pub fn new(event_id: i64, name: &str) -> EventSnapshotBuilder {
        EventSnapshotBuilder {
            event: Event {
                id: EventId(event_id),
                name: name.to_string(),
                starts_at: None,
                ends_at: None,
            },
            stages: Vec::new(),
            teams: Vec::new(),
            matches: Vec::new(),
            format: MatchFormat::Bo3,
        }
    }

    pub fn dates(mut self, starts_at: DateTime<Utc>, ends_at: DateTime<Utc>) -> Self {
        self.event.starts_at = Some(starts_at);
        self.event.ends_at = Some(ends_at);
        self
    }

    /// Opens the next Stage (ordinals 1, 2, ...). Following Matches belong to it.
    pub fn stage(mut self, kind: StageKind, name: &str) -> Self {
        self.stages.push(Stage {
            ordinal: StageOrdinal(self.stages.len() as u8 + 1),
            kind,
            name: name.to_string(),
        });
        self
    }

    /// The format of the Matches added after this call. Starts as Bo3.
    pub fn format(mut self, format: MatchFormat) -> Self {
        self.format = format;
        self
    }

    /// A Swiss Match; `bucket` is `(round, wins, losses)`.
    pub fn swiss_match(
        self,
        id: i64,
        bucket: (u8, u8, u8),
        team_a: &str,
        team_b: &str,
        scheduled_start: DateTime<Utc>,
    ) -> Self {
        let placement = MatchPlacement::Swiss {
            round: bucket.0,
            wins: bucket.1,
            losses: bucket.2,
        };
        self.add_match(id, placement, Some(team_a), Some(team_b), scheduled_start)
    }

    /// A Playoffs Match; `slot` is `(rounds_before_final, slot)`, so `(0, 0)`
    /// is the final and `(2, 0)`..`(2, 3)` are the quarterfinals.
    pub fn playoff_match(
        self,
        id: i64,
        slot: (u8, u8),
        team_a: &str,
        team_b: &str,
        scheduled_start: DateTime<Utc>,
    ) -> Self {
        let placement = MatchPlacement::Playoffs {
            rounds_before_final: slot.0,
            slot: slot.1,
        };
        self.add_match(id, placement, Some(team_a), Some(team_b), scheduled_start)
    }

    /// A Playoffs slot whose teams are not decided yet.
    pub fn playoff_slot(self, id: i64, slot: (u8, u8), scheduled_start: DateTime<Utc>) -> Self {
        let placement = MatchPlacement::Playoffs {
            rounds_before_final: slot.0,
            slot: slot.1,
        };
        self.add_match(id, placement, None, None, scheduled_start)
    }

    /// A Match with no placement (e.g. in a `Groups` Stage).
    pub fn unplaced_match(
        self,
        id: i64,
        team_a: &str,
        team_b: &str,
        scheduled_start: DateTime<Utc>,
    ) -> Self {
        self.add_match(
            id,
            MatchPlacement::Unplaced,
            Some(team_a),
            Some(team_b),
            scheduled_start,
        )
    }

    pub fn build(self, fetched_at: DateTime<Utc>) -> EventSnapshot {
        EventSnapshot {
            event: self.event,
            stages: self.stages,
            teams: self.teams,
            matches: self.matches,
            source: DataSourceKind::Hltv,
            fetched_at,
        }
    }

    fn add_match(
        mut self,
        id: i64,
        placement: MatchPlacement,
        team_a: Option<&str>,
        team_b: Option<&str>,
        scheduled_start: DateTime<Utc>,
    ) -> Self {
        let stage = self
            .stages
            .last()
            .expect("EventSnapshotBuilder: call .stage(..) before adding Matches")
            .ordinal;
        let team_a = team_a.map(|name| self.team(name));
        let team_b = team_b.map(|name| self.team(name));
        self.matches.push(Match {
            id: MatchId(id),
            stage,
            placement,
            team_a,
            team_b,
            format: self.format,
            scheduled_start: Some(scheduled_start),
            actual_start: None,
            status: MatchStatus::Upcoming,
            winner: None,
            score: None,
        });
        self
    }

    fn team(&mut self, name: &str) -> TeamId {
        if let Some(team) = self.teams.iter().find(|t| t.name == name) {
            return team.id;
        }
        let id = TeamId(self.teams.len() as i64 + 1);
        self.teams.push(Team {
            id,
            name: name.to_string(),
            logo_url: None,
        });
        id
    }
}
