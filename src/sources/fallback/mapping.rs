//! The mapping layer (pure): links a fallback source's records to canonical
//! ids and merges them over the stored data into a canonical snapshot.
//!
//! Linking, per record, first hit wins:
//! 1. a manual mapping (a League Admin's fix, #13);
//! 2. a canonical id the record carries (`ExternalMatch::canonical`, ...);
//! 3. an earlier automatic mapping;
//! 4. matching: a team by normalised name ([`super::names::team_key`]); a
//!    Match by its teams and Scheduled Start within ± the window. One known
//!    team plus the time is enough; the other team is then inferred from the
//!    stored Match and mapped too.
//!
//! What cannot be linked: when the Event has HLTV data, the record is left
//! out and flagged unmatched (better no data than a duplicate Match). When it
//! has none (never fetched from HLTV), it becomes a new Match or team with a
//! local id (negative, see `store::external_ids`).
//!
//! Merging over stored data (the "lower priority never spoils higher
//! priority" rule):
//! - a fallback only updates the progress fields of a stored Match (Scheduled
//!   Start, actual start, status, winner, score) and fills teams still to be
//!   decided; Stage, placement, format and team names stay as stored;
//! - it never moves a Match backwards (finished -> live -> upcoming);
//! - it never touches a Match that a higher-priority source has settled
//!   (finished or cancelled); if it reports another winner, a conflict is
//!   recorded and the stored result stands;
//! - a settled result from a lower-priority source is overwritten by a
//!   higher-priority one, with a conflict if the winner differs.
//!   Stored Matches it does not report stay as they are.

use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Duration, Utc};

use super::external::{
    EventHint, ExternalEvent, ExternalMatch, ExternalSnapshot, ExternalTeam, Side,
};
use super::names::{event_similarity, team_key};
use crate::domain::effective_match::{EventView, SourcedMatch};
use crate::domain::{
    DataSourceKind, Event, EventSnapshot, Match, MatchId, MatchPlacement, MatchScore, MatchStatus,
    Stage, StageOrdinal, Team, TeamId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IdKind {
    Event,
    Match,
    Team,
}

impl IdKind {
    pub fn code(self) -> &'static str {
        match self {
            IdKind::Event => "event",
            IdKind::Match => "match",
            IdKind::Team => "team",
        }
    }

    pub fn from_code(code: &str) -> Option<IdKind> {
        match code {
            "event" => Some(IdKind::Event),
            "match" => Some(IdKind::Match),
            "team" => Some(IdKind::Team),
            _ => None,
        }
    }
}

/// How a mapping came about. A manual one is never replaced automatically.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Origin {
    Auto,
    Source,
    Manual,
}

impl Origin {
    pub fn code(self) -> &'static str {
        match self {
            Origin::Auto => "auto",
            Origin::Source => "source",
            Origin::Manual => "manual",
        }
    }

    pub fn from_code(code: &str) -> Origin {
        match code {
            "manual" => Origin::Manual,
            "source" => Origin::Source,
            _ => Origin::Auto,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mapping {
    pub canonical: i64,
    pub origin: Origin,
}

/// One source's mappings: (kind, external id) -> canonical id.
pub type Mappings = HashMap<(IdKind, String), Mapping>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewMapping {
    pub kind: IdKind,
    pub external_id: String,
    pub canonical: i64,
    pub origin: Origin,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unmatched {
    pub kind: IdKind,
    pub external_id: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conflict {
    pub match_id: MatchId,
    pub kept_source: DataSourceKind,
    pub kept_winner: Option<TeamId>,
    pub other_source: DataSourceKind,
    pub other_winner: Option<TeamId>,
}

pub struct Context<'a> {
    pub source: DataSourceKind,
    /// The canonical Event (stored, or built from the source's).
    pub event: &'a Event,
    /// What is stored for the Event, if anything.
    pub stored: Option<&'a EventView>,
    /// Every known team (all Events).
    pub teams: &'a [Team],
    pub mappings: &'a Mappings,
    pub window: Duration,
    pub now: DateTime<Utc>,
}

pub struct Merged {
    /// Only what this source may write: Matches it may not touch are left
    /// out (the store never deletes a Match).
    pub snapshot: EventSnapshot,
    pub mappings: Vec<NewMapping>,
    pub unmatched: Vec<Unmatched>,
    pub conflicts: Vec<Conflict>,
    /// How many local ids of each kind were taken (`(matches, teams)`).
    pub local_ids_taken: (usize, usize),
}

/// Hands out local ids. The chain runs [`merge`] once with
/// [`LocalIds::counting`] to learn how many it needs, reserves exactly that
/// many in the database, then runs it again with [`LocalIds::reserved`].
pub struct LocalIds {
    next_match: i64,
    next_team: i64,
    taken: (usize, usize),
}

impl LocalIds {
    pub fn counting() -> LocalIds {
        LocalIds::reserved(-1, -1)
    }

    /// `first_match` / `first_team` are the first reserved ids; later ones
    /// count down from there.
    pub fn reserved(first_match: i64, first_team: i64) -> LocalIds {
        LocalIds {
            next_match: first_match,
            next_team: first_team,
            taken: (0, 0),
        }
    }

    fn take_match(&mut self) -> MatchId {
        let id = self.next_match;
        self.next_match -= 1;
        self.taken.0 += 1;
        MatchId(id)
    }

    fn take_team(&mut self) -> TeamId {
        let id = self.next_team;
        self.next_team -= 1;
        self.taken.1 += 1;
        TeamId(id)
    }
}

fn status_rank(status: MatchStatus) -> u8 {
    match status {
        MatchStatus::Upcoming => 0,
        MatchStatus::Live => 1,
        MatchStatus::Finished | MatchStatus::Cancelled => 2,
    }
}

pub fn is_settled(m: &Match) -> bool {
    status_rank(m.status) == 2
}

/// A settled Match's result: its winner, `None` for a Void Match.
fn result(m: &Match) -> Option<TeamId> {
    if m.status == MatchStatus::Finished {
        m.winner
    } else {
        None
    }
}

fn within(a: Option<DateTime<Utc>>, b: Option<DateTime<Utc>>, window: Duration) -> Option<i64> {
    match (a, b) {
        (Some(a), Some(b)) => {
            let diff = (a - b).num_minutes().abs();
            (diff <= window.num_minutes()).then_some(diff)
        }
        _ => Some(0),
    }
}

/// An external team slot as resolved so far.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Slot {
    /// No team (to be decided).
    Tbd,
    /// A team the chain could not link (yet).
    Unresolved,
    Known(TeamId),
}

/// Pairs an external Match's two slots with a stored Match's teams. Returns
/// whether the external order is swapped and how many teams agree, or `None`
/// if they cannot be the same Match.
fn orient(
    ext: [Slot; 2],
    stored: [Option<TeamId>; 2],
    named: &HashSet<TeamId>,
) -> Option<(bool, usize)> {
    let pair = |e: Slot, s: Option<TeamId>| -> Option<usize> {
        match (e, s) {
            (Slot::Known(x), Some(y)) => (x == y).then_some(1),
            (Slot::Known(_), None) | (Slot::Unresolved, None) | (Slot::Tbd, _) => Some(0),
            // The stored team may be this unlinked one, unless another
            // record of the same snapshot is that team.
            (Slot::Unresolved, Some(y)) => (!named.contains(&y)).then_some(0),
        }
    };
    let straight = pair(ext[0], stored[0])
        .zip(pair(ext[1], stored[1]))
        .map(|(a, b)| (false, a + b));
    let swapped = pair(ext[0], stored[1])
        .zip(pair(ext[1], stored[0]))
        .map(|(a, b)| (true, a + b));
    match (straight, swapped) {
        (Some(s), Some(w)) => Some(if w.1 > s.1 { w } else { s }),
        (s, w) => s.or(w),
    }
}

fn describe_match(m: &ExternalMatch) -> String {
    let name = |t: &Option<ExternalTeam>| t.as_ref().map_or("TBD".to_string(), |t| t.name.clone());
    let start = m.scheduled_start.map_or("no start time".to_string(), |s| {
        s.format("%Y-%m-%d %H:%M UTC").to_string()
    });
    format!("{} vs {}, {}", name(&m.team_a), name(&m.team_b), start)
}

struct Merger<'a> {
    ctx: &'a Context<'a>,
    ids: &'a mut LocalIds,
    has_hltv_data: bool,
    teams: HashMap<String, Option<TeamId>>,
    named: HashSet<TeamId>,
    new_teams: HashMap<TeamId, Team>,
    stages: Vec<Stage>,
    stage_map: HashMap<StageOrdinal, StageOrdinal>,
    out: Merged,
}

impl<'a> Merger<'a> {
    fn mapped(&self, kind: IdKind, external_id: &str, origin: Option<Origin>) -> Option<i64> {
        self.ctx
            .mappings
            .get(&(kind, external_id.to_string()))
            .filter(|m| origin.is_none_or(|o| m.origin == o))
            .map(|m| m.canonical)
    }

    fn map(&mut self, kind: IdKind, external_id: &str, canonical: i64, origin: Origin) {
        let existing = self.ctx.mappings.get(&(kind, external_id.to_string()));
        if existing.is_some_and(|m| m.origin == Origin::Manual || m.canonical == canonical) {
            return;
        }
        self.out.mappings.push(NewMapping {
            kind,
            external_id: external_id.to_string(),
            canonical,
            origin,
        });
    }

    fn flag(&mut self, kind: IdKind, external_id: &str, description: String) {
        if self
            .out
            .unmatched
            .iter()
            .any(|u| u.kind == kind && u.external_id == external_id)
        {
            return;
        }
        self.out.unmatched.push(Unmatched {
            kind,
            external_id: external_id.to_string(),
            description,
        });
    }

    fn event_teams(&self) -> &[Team] {
        self.ctx.stored.map_or(&[], |v| v.teams.as_slice())
    }

    fn by_name(&self, name: &str) -> Option<TeamId> {
        let key = team_key(name);
        let unique = |teams: &[Team]| {
            let mut hits = teams.iter().filter(|t| team_key(&t.name) == key);
            match (hits.next(), hits.next()) {
                (Some(t), None) => Some(t.id),
                _ => None,
            }
        };
        unique(self.event_teams()).or_else(|| unique(self.ctx.teams))
    }

    fn resolve_team(&mut self, team: &ExternalTeam) {
        if self.teams.contains_key(&team.external_id) {
            return;
        }
        let id =
            if let Some(id) = self.mapped(IdKind::Team, &team.external_id, Some(Origin::Manual)) {
                Some(TeamId(id))
            } else if let Some(id) = team.canonical {
                self.map(IdKind::Team, &team.external_id, id.0, Origin::Source);
                Some(id)
            } else if let Some(id) = self.mapped(IdKind::Team, &team.external_id, None) {
                Some(TeamId(id))
            } else if let Some(id) = self.by_name(&team.name) {
                self.map(IdKind::Team, &team.external_id, id.0, Origin::Auto);
                Some(id)
            } else if !self.has_hltv_data {
                let id = self.ids.take_team();
                self.map(IdKind::Team, &team.external_id, id.0, Origin::Auto);
                self.new_teams.insert(
                    id,
                    Team {
                        id,
                        name: team.name.clone(),
                        logo_url: None,
                    },
                );
                Some(id)
            } else {
                None
            };
        if let Some(id) = id {
            self.named.insert(id);
            // A team first known here (e.g. by a canonical id the source
            // carries) is stored under the source's spelling.
            self.new_teams.entry(id).or_insert_with(|| Team {
                id,
                name: team.name.clone(),
                logo_url: None,
            });
        }
        self.teams.insert(team.external_id.clone(), id);
    }

    fn slot(&self, team: &Option<ExternalTeam>) -> Slot {
        match team {
            None => Slot::Tbd,
            Some(t) => match self.teams.get(&t.external_id).copied().flatten() {
                Some(id) => Slot::Known(id),
                None => Slot::Unresolved,
            },
        }
    }

    fn stored(&self, id: MatchId) -> Option<&'a SourcedMatch> {
        self.ctx.stored?.matches.iter().find(|m| m.data.id == id)
    }

    /// The canonical Stage an external one is, if it exists yet: same kind,
    /// same position among the Stages of that kind.
    fn existing_stage(
        &self,
        ext: &ExternalSnapshot,
        ordinal: StageOrdinal,
    ) -> Option<StageOrdinal> {
        if let Some(mapped) = self.stage_map.get(&ordinal) {
            return Some(*mapped);
        }
        let stage = ext.stages.iter().find(|s| s.ordinal == ordinal)?;
        let position = ext
            .stages
            .iter()
            .filter(|s| s.kind == stage.kind && s.ordinal <= ordinal)
            .count();
        self.stages
            .iter()
            .filter(|s| s.kind == stage.kind)
            .nth(position - 1)
            .map(|s| s.ordinal)
    }

    /// Like [`existing_stage`](Self::existing_stage), adding the Stage when
    /// it is new.
    fn canonical_stage(&mut self, ext: &ExternalSnapshot, ordinal: StageOrdinal) -> StageOrdinal {
        let Some(stage) = ext.stages.iter().find(|s| s.ordinal == ordinal).cloned() else {
            return ordinal;
        };
        let mapped = self.existing_stage(ext, ordinal).unwrap_or_else(|| {
            let next = StageOrdinal(self.stages.iter().map(|s| s.ordinal.0).max().unwrap_or(0) + 1);
            self.stages.push(Stage {
                ordinal: next,
                kind: stage.kind,
                name: stage.name.clone(),
            });
            next
        });
        self.stage_map.insert(ordinal, mapped);
        mapped
    }

    /// Finds the stored Match an external one is, by teams and time.
    fn find(
        &self,
        em: &ExternalMatch,
        claimed: &HashSet<MatchId>,
        taken: &HashSet<i64>,
    ) -> Option<MatchId> {
        let stored = self.ctx.stored?;
        let ext = [self.slot(&em.team_a), self.slot(&em.team_b)];
        stored
            .matches
            .iter()
            .filter(|sm| !claimed.contains(&sm.data.id) && !taken.contains(&sm.data.id.0))
            .filter_map(|sm| {
                let (_, agree) = orient(ext, sm.data.teams(), &self.named)?;
                let same_slot = matches!(em.placement, MatchPlacement::Playoffs { .. })
                    && em.placement == sm.data.placement;
                if agree == 0 && !same_slot {
                    return None;
                }
                let diff = within(em.scheduled_start, sm.data.scheduled_start, self.ctx.window)?;
                Some((agree, -diff, sm.data.id))
            })
            .max_by_key(|(agree, diff, _)| (*agree, *diff))
            .map(|(_, _, id)| id)
    }
}

/// Links `ext` to canonical ids and merges it over `ctx.stored`.
pub fn merge(ctx: &Context<'_>, ext: &ExternalSnapshot, ids: &mut LocalIds) -> Merged {
    let has_hltv_data = ctx
        .stored
        .is_some_and(|v| v.matches.iter().any(|m| m.data.id.0 > 0));
    let stages = match ctx.stored {
        Some(view) if !view.stages.is_empty() => view.stages.clone(),
        _ => ext.stages.clone(),
    };
    let mut m = Merger {
        ctx,
        ids,
        has_hltv_data,
        teams: HashMap::new(),
        named: HashSet::new(),
        new_teams: HashMap::new(),
        stages,
        stage_map: HashMap::new(),
        out: Merged {
            snapshot: EventSnapshot {
                event: ctx.event.clone(),
                stages: Vec::new(),
                teams: Vec::new(),
                matches: Vec::new(),
                source: ctx.source,
                fetched_at: ctx.now,
            },
            mappings: Vec::new(),
            unmatched: Vec::new(),
            conflicts: Vec::new(),
            local_ids_taken: (0, 0),
        },
    };

    for em in &ext.matches {
        for team in [&em.team_a, &em.team_b].into_iter().flatten() {
            m.resolve_team(team);
        }
    }

    // Canonical ids other external Matches are already mapped to.
    let taken: HashSet<i64> = ctx
        .mappings
        .iter()
        .filter(|((kind, _), _)| *kind == IdKind::Match)
        .map(|(_, mapping)| mapping.canonical)
        .collect();
    let mut claimed: HashSet<MatchId> = HashSet::new();

    for em in &ext.matches {
        let id = if let Some(id) = m.mapped(IdKind::Match, &em.external_id, Some(Origin::Manual)) {
            Some(MatchId(id))
        } else if let Some(id) = em.canonical {
            m.map(IdKind::Match, &em.external_id, id.0, Origin::Source);
            Some(id)
        } else if let Some(id) = m.mapped(IdKind::Match, &em.external_id, None) {
            Some(MatchId(id))
        } else if let Some(id) = m.find(em, &claimed, &taken) {
            m.map(IdKind::Match, &em.external_id, id.0, Origin::Auto);
            Some(id)
        } else {
            None
        };
        let slots = [m.slot(&em.team_a), m.slot(&em.team_b)];
        let unresolved = slots.contains(&Slot::Unresolved);

        let Some(id) = id else {
            let no_teams = slots == [Slot::Tbd, Slot::Tbd];
            // Two teams meet once per Stage: the same pair outside the time
            // window is that Match with a bad time, not a new one.
            let stage = m.existing_stage(ext, em.stage);
            let pair_known = m.ctx.stored.is_some_and(|v| {
                v.matches.iter().any(|sm| {
                    Some(sm.data.stage) == stage
                        && matches!(
                            orient(slots, sm.data.teams(), &HashSet::new()),
                            Some((_, 2))
                        )
                })
            });
            if m.has_hltv_data && (unresolved || no_teams || pair_known) {
                flag_match(&mut m, em);
                continue;
            }
            let stage = m.canonical_stage(ext, em.stage);
            let id = m.ids.take_match();
            m.map(IdKind::Match, &em.external_id, id.0, Origin::Auto);
            claimed.insert(id);
            let game = fresh(em, id, stage, slots);
            m.out.snapshot.matches.push(game);
            continue;
        };
        claimed.insert(id);

        let Some(stored) = m.stored(id) else {
            if unresolved {
                flag_match(&mut m, em);
                continue;
            }
            let stage = m.canonical_stage(ext, em.stage);
            let game = fresh(em, id, stage, slots);
            m.out.snapshot.matches.push(game);
            continue;
        };

        let (swapped, _) = orient(slots, stored.data.teams(), &m.named).unwrap_or((false, 0));
        let (ext_for_a, ext_for_b) = if swapped {
            (&em.team_b, &em.team_a)
        } else {
            (&em.team_a, &em.team_b)
        };
        // Teams: the stored ones; a still-unlinked external team is the
        // stored team on its side (mapped now), a TBD stored side is filled.
        let mut team_for = |ext: &Option<ExternalTeam>,
                            stored_team: Option<TeamId>|
         -> Result<Option<TeamId>, ()> {
            let Some(ext) = ext else {
                return Ok(stored_team);
            };
            match (
                m.teams.get(&ext.external_id).copied().flatten(),
                stored_team,
            ) {
                (Some(id), _) => Ok(stored_team.or(Some(id))),
                (None, Some(stored_team)) => {
                    m.map(IdKind::Team, &ext.external_id, stored_team.0, Origin::Auto);
                    m.teams.insert(ext.external_id.clone(), Some(stored_team));
                    Ok(Some(stored_team))
                }
                (None, None) => Err(()),
            }
        };
        let (Ok(team_a), Ok(team_b)) = (
            team_for(ext_for_a, stored.data.team_a),
            team_for(ext_for_b, stored.data.team_b),
        ) else {
            flag_match(&mut m, em);
            continue;
        };
        let winner = em.winner.and_then(|side| {
            let ext = match side {
                Side::A => &em.team_a,
                Side::B => &em.team_b,
            };
            ext.as_ref()
                .and_then(|t| m.teams.get(&t.external_id).copied().flatten())
        });
        let score = em.score.map(|s| {
            if swapped {
                MatchScore {
                    team_a: s.team_b,
                    team_b: s.team_a,
                }
            } else {
                s
            }
        });
        let reported = Match {
            team_a,
            team_b,
            scheduled_start: em.scheduled_start.or(stored.data.scheduled_start),
            actual_start: em.actual_start.or(stored.data.actual_start),
            status: em.status,
            winner,
            score: score.or(stored.data.score),
            ..stored.data.clone()
        };
        if let Some(game) = over_stored(&mut m.out.conflicts, ctx.source, stored, reported) {
            m.out.snapshot.matches.push(game);
        }
    }

    // Teams of the snapshot: stored names win; new local teams by the
    // source's spelling.
    let mut wanted: Vec<TeamId> = m
        .out
        .snapshot
        .matches
        .iter()
        .flat_map(|g| g.teams())
        .flatten()
        .collect();
    wanted.sort();
    wanted.dedup();
    let teams: Vec<Team> = wanted
        .into_iter()
        .filter_map(|id| {
            m.event_teams()
                .iter()
                .chain(ctx.teams)
                .find(|t| t.id == id)
                .cloned()
                .or_else(|| m.new_teams.get(&id).cloned())
        })
        .collect();
    m.out.snapshot.teams = teams;
    m.out.snapshot.stages = m.stages.clone();
    m.out.local_ids_taken = m.ids.taken;
    m.out
}

fn flag_match(m: &mut Merger<'_>, em: &ExternalMatch) {
    for team in [&em.team_a, &em.team_b].into_iter().flatten() {
        if m.teams.get(&team.external_id).copied().flatten().is_none() {
            m.flag(IdKind::Team, &team.external_id, team.name.clone());
        }
    }
    m.flag(IdKind::Match, &em.external_id, describe_match(em));
}

/// A Match first known from this external record.
fn fresh(em: &ExternalMatch, id: MatchId, stage: StageOrdinal, slots: [Slot; 2]) -> Match {
    let team = |slot: Slot| match slot {
        Slot::Known(id) => Some(id),
        Slot::Tbd | Slot::Unresolved => None,
    };
    let (team_a, team_b) = (team(slots[0]), team(slots[1]));
    let winner = em.winner.and_then(|side| match side {
        Side::A => team_a,
        Side::B => team_b,
    });
    Match {
        id,
        stage,
        placement: em.placement,
        team_a,
        team_b,
        format: em.format,
        scheduled_start: em.scheduled_start,
        actual_start: em.actual_start,
        status: em.status,
        winner,
        score: em.score,
    }
}

/// The merging rule for one stored Match: what `source` may write over it,
/// if anything, recording a conflict when two settled results disagree.
fn over_stored(
    conflicts: &mut Vec<Conflict>,
    source: DataSourceKind,
    stored: &SourcedMatch,
    reported: Match,
) -> Option<Match> {
    let mine = source.priority();
    let theirs = stored.source.priority();
    if is_settled(&stored.data)
        && is_settled(&reported)
        && result(&stored.data) != result(&reported)
    {
        if mine > theirs {
            conflicts.push(Conflict {
                match_id: stored.data.id,
                kept_source: stored.source,
                kept_winner: result(&stored.data),
                other_source: source,
                other_winner: result(&reported),
            });
            return None;
        }
        if mine < theirs {
            conflicts.push(Conflict {
                match_id: stored.data.id,
                kept_source: source,
                kept_winner: result(&reported),
                other_source: stored.source,
                other_winner: result(&stored.data),
            });
        }
        return Some(reported);
    }
    if is_settled(&stored.data) && mine > theirs {
        return None;
    }
    if status_rank(reported.status) < status_rank(stored.data.status) {
        return None;
    }
    Some(reported)
}

/// Conflicts between a fresh snapshot of the canonical source and stored
/// results from lower-priority sources (the snapshot's results stand).
pub fn primary_conflicts(stored: &EventView, snapshot: &EventSnapshot) -> Vec<Conflict> {
    let mut conflicts = Vec::new();
    for game in &snapshot.matches {
        let Some(old) = stored.matches.iter().find(|m| m.data.id == game.id) else {
            continue;
        };
        if old.source.priority() > snapshot.source.priority() {
            let _ = over_stored(&mut conflicts, snapshot.source, old, game.clone());
        }
    }
    conflicts
}

/// Local (negative-id) Matches and teams that a canonical snapshot now
/// knows under their HLTV ids: `(local, canonical)` pairs to re-key.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Adoptions {
    pub teams: Vec<(TeamId, TeamId)>,
    pub matches: Vec<(MatchId, MatchId)>,
}

pub fn adoptions(stored: &EventView, snapshot: &EventSnapshot, window: Duration) -> Adoptions {
    let mut out = Adoptions::default();
    let mut team_of: HashMap<TeamId, TeamId> = HashMap::new();
    for team in stored.teams.iter().filter(|t| t.id.0 < 0) {
        let key = team_key(&team.name);
        let mut hits = snapshot.teams.iter().filter(|t| team_key(&t.name) == key);
        if let (Some(hit), None) = (hits.next(), hits.next()) {
            team_of.insert(team.id, hit.id);
            out.teams.push((team.id, hit.id));
        }
    }
    let known: HashSet<MatchId> = stored.matches.iter().map(|m| m.data.id).collect();
    let mut used: HashSet<MatchId> = HashSet::new();
    // Both teams known (by name) first, then one known team plus the time:
    // the other local team is then that snapshot team.
    for needed in [2, 1] {
        for local in stored.matches.iter().filter(|m| m.data.id.0 < 0) {
            if out.matches.iter().any(|(from, _)| *from == local.data.id) {
                continue;
            }
            let canon = |t: Option<TeamId>| t.map(|t| team_of.get(&t).copied().unwrap_or(t));
            let (Some(a), Some(b)) = (canon(local.data.team_a), canon(local.data.team_b)) else {
                continue;
            };
            let adopted: HashSet<TeamId> = team_of.values().copied().collect();
            let hit = snapshot
                .matches
                .iter()
                .filter(|g| !known.contains(&g.id) && !used.contains(&g.id))
                .filter_map(|g| {
                    let teams = g.teams();
                    let agree = [a, b].iter().filter(|t| teams.contains(&Some(**t))).count();
                    if agree < needed {
                        return None;
                    }
                    // A still-local team pairs with the snapshot's other team.
                    let mut pairs = Vec::new();
                    for local_team in [a, b].into_iter().filter(|t| t.0 < 0) {
                        let other = teams
                            .into_iter()
                            .flatten()
                            .find(|t| *t != a && *t != b && !adopted.contains(t))?;
                        pairs.push((local_team, other));
                    }
                    let diff = within(g.scheduled_start, local.data.scheduled_start, window)?;
                    Some((diff, g.id, pairs))
                })
                .min_by_key(|(diff, id, _)| (*diff, *id));
            if let Some((_, id, pairs)) = hit {
                used.insert(id);
                out.matches.push((local.data.id, id));
                for (from, to) in pairs {
                    team_of.insert(from, to);
                    out.teams.push((from, to));
                }
            }
        }
    }
    out
}

/// Picks the external Event that is the canonical one `hint` describes:
/// one carrying its HLTV id, else the best name match (at least
/// [`MIN_EVENT_SIMILARITY`]) whose dates overlap. `None` if nothing fits or
/// two candidates fit equally well.
pub fn choose_event(hint: &EventHint, candidates: &[ExternalEvent]) -> Option<(String, Origin)> {
    if let Some(c) = candidates.iter().find(|c| c.canonical == Some(hint.id)) {
        return Some((c.external_id.clone(), Origin::Source));
    }
    let mut scored: Vec<(f64, &ExternalEvent)> = candidates
        .iter()
        .filter(|c| c.canonical.is_none())
        .filter(|c| dates_overlap(hint.starts_at, hint.ends_at, c.starts_at, c.ends_at))
        .map(|c| (event_similarity(&hint.name, &c.name), c))
        .filter(|(score, _)| *score >= MIN_EVENT_SIMILARITY)
        .collect();
    scored.sort_by(|a, b| b.0.total_cmp(&a.0));
    match scored.as_slice() {
        [] => None,
        [(best, c), rest @ ..] => {
            if rest
                .first()
                .is_some_and(|(second, _)| (best - second).abs() < 1e-9)
            {
                None
            } else {
                Some((c.external_id.clone(), Origin::Auto))
            }
        }
    }
}

/// How alike two Event names must be (shared distinctive words / all).
pub const MIN_EVENT_SIMILARITY: f64 = 0.4;

fn dates_overlap(
    a_start: Option<DateTime<Utc>>,
    a_end: Option<DateTime<Utc>>,
    b_start: Option<DateTime<Utc>>,
    b_end: Option<DateTime<Utc>>,
) -> bool {
    let slack = Duration::days(2);
    match (a_start, b_start) {
        (Some(a), Some(b)) => {
            let a_end = a_end.unwrap_or(a);
            let b_end = b_end.unwrap_or(b);
            a - slack <= b_end && b - slack <= a_end
        }
        _ => true,
    }
}
