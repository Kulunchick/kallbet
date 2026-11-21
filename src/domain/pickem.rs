//! Pick'em validation and scoring (pure).
//!
//! A Pick'em is a flat list of [`Pick`]s, each a `(kind, slot, team)`. The
//! Swiss shape (#14) uses the kinds `ThreeNil`, `ZeroThree` and `Advance`
//! (slot = position in the list); the Playoffs bracket (#15) adds a second
//! shape with kind `Bracket` (slot = bracket slot) next to it: its own
//! validation and scoring functions, but the same storage, [`lock`] and
//! shared-place [`rank`].
//!
//! Everything reads the effective Match view ([`EventView`]), never raw data.
//!
//! ## A Stage's team list
//!
//! Before all of a Stage's Matches are paired the team list is still known:
//! a Swiss Stage's first round contains all its teams. [`swiss_teams`] is
//! the teams of the Stage's round 1 Matches (or, when a Data Source gives no
//! round numbers, of every Match in the Stage). Teams of later rounds are
//! always among them.

use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};

use super::effective_match::{EventView, SourcedMatch};
use super::{MatchPlacement, MatchStatus, StageOrdinal, TeamId};

pub const THREE_NIL_COUNT: usize = 2;
pub const ZERO_THREE_COUNT: usize = 2;
pub const ADVANCE_COUNT: usize = 6;

/// Wins that advance a team in a Swiss Stage, and losses that eliminate it.
pub const SWISS_WINS: u8 = 3;
pub const SWISS_LOSSES: u8 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PickKind {
    ThreeNil,
    ZeroThree,
    Advance,
    /// Playoffs bracket slot (#15).
    Bracket,
}

impl PickKind {
    pub fn code(self) -> &'static str {
        match self {
            PickKind::ThreeNil => "three_nil",
            PickKind::ZeroThree => "zero_three",
            PickKind::Advance => "advance",
            PickKind::Bracket => "bracket",
        }
    }

    pub fn from_code(code: &str) -> Option<PickKind> {
        [
            PickKind::ThreeNil,
            PickKind::ZeroThree,
            PickKind::Advance,
            PickKind::Bracket,
        ]
        .into_iter()
        .find(|k| k.code() == code)
    }
}

/// A single forecast within a Pick'em.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pick {
    pub kind: PickKind,
    pub slot: u8,
    pub team: TeamId,
}

/// Why a Pick'em was refused. `code()` is the API error code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickemError {
    WrongCount,
    DuplicateTeam,
    TeamNotInStage,
    /// A Playoffs bracket whose first-round teams are not all known yet.
    BracketNotReady,
}

impl PickemError {
    pub fn code(self) -> &'static str {
        match self {
            PickemError::WrongCount => "pickem_wrong_count",
            PickemError::DuplicateTeam => "pickem_duplicate_team",
            PickemError::TeamNotInStage => "pickem_team_not_in_stage",
            PickemError::BracketNotReady => "pickem_bracket_not_ready",
        }
    }
}

/// A Swiss Pick'em: two teams to go 3-0, two to go 0-3, six to advance.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SwissPickem {
    pub three_nil: Vec<TeamId>,
    pub zero_three: Vec<TeamId>,
    pub advance: Vec<TeamId>,
}

impl SwissPickem {
    pub fn to_picks(&self) -> Vec<Pick> {
        let of = |kind, teams: &[TeamId]| {
            teams
                .iter()
                .enumerate()
                .map(|(i, &team)| Pick {
                    kind,
                    slot: i as u8,
                    team,
                })
                .collect::<Vec<_>>()
        };
        let mut picks = of(PickKind::ThreeNil, &self.three_nil);
        picks.extend(of(PickKind::ZeroThree, &self.zero_three));
        picks.extend(of(PickKind::Advance, &self.advance));
        picks
    }

    pub fn from_picks(picks: &[Pick]) -> SwissPickem {
        let of = |kind| {
            let mut of_kind: Vec<&Pick> = picks.iter().filter(|p| p.kind == kind).collect();
            of_kind.sort_by_key(|p| p.slot);
            of_kind.into_iter().map(|p| p.team).collect::<Vec<_>>()
        };
        SwissPickem {
            three_nil: of(PickKind::ThreeNil),
            zero_three: of(PickKind::ZeroThree),
            advance: of(PickKind::Advance),
        }
    }

    /// Correct counts, no team picked twice, every team from the Stage.
    pub fn validate(&self, stage_teams: &[TeamId]) -> Result<(), PickemError> {
        if self.three_nil.len() != THREE_NIL_COUNT
            || self.zero_three.len() != ZERO_THREE_COUNT
            || self.advance.len() != ADVANCE_COUNT
        {
            return Err(PickemError::WrongCount);
        }
        let all = self
            .three_nil
            .iter()
            .chain(&self.zero_three)
            .chain(&self.advance);
        if all.clone().any(|team| !stage_teams.contains(team)) {
            return Err(PickemError::TeamNotInStage);
        }
        let mut seen = HashSet::new();
        if all.into_iter().any(|team| !seen.insert(*team)) {
            return Err(PickemError::DuplicateTeam);
        }
        Ok(())
    }
}

/// The teams of a Swiss Stage, in id order. See the module docs.
pub fn swiss_teams(view: &EventView, stage: StageOrdinal) -> Vec<TeamId> {
    let in_stage = || view.matches.iter().filter(|m| m.data.stage == stage);
    let round_one = |m: &&SourcedMatch| {
        matches!(m.data.placement, MatchPlacement::Swiss { round: 1, .. })
    };
    let first_round: Vec<&SourcedMatch> = in_stage().filter(round_one).collect();
    let source: Vec<&SourcedMatch> = if first_round.is_empty() {
        in_stage().collect()
    } else {
        first_round
    };
    let mut teams: Vec<TeamId> = source
        .iter()
        .flat_map(|m| m.data.teams())
        .flatten()
        .collect();
    teams.sort();
    teams.dedup();
    teams
}

/// How a team's Swiss Stage ended, once it is settled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwissOutcome {
    /// 3-0.
    ThreeNil,
    /// 3 wins and at least one loss.
    Advanced,
    /// 0-3.
    ZeroThree,
    /// 3 losses and at least one win.
    Eliminated,
}

/// The settled teams' outcomes, from the Stage's finished Matches (Void
/// Matches count for nothing). A team still in contention has no entry.
pub fn swiss_outcomes(view: &EventView, stage: StageOrdinal) -> HashMap<TeamId, SwissOutcome> {
    let mut record: HashMap<TeamId, (u8, u8)> = HashMap::new();
    for m in view.matches.iter().filter(|m| m.data.stage == stage) {
        let m = &m.data;
        if m.status != MatchStatus::Finished || m.is_void() {
            continue;
        }
        let (Some(winner), [Some(a), Some(b)]) = (m.winner, m.teams()) else {
            continue;
        };
        let loser = if winner == a { b } else { a };
        record.entry(winner).or_default().0 += 1;
        record.entry(loser).or_default().1 += 1;
    }
    record
        .into_iter()
        .filter_map(|(team, (wins, losses))| {
            let outcome = if wins >= SWISS_WINS {
                if losses == 0 {
                    SwissOutcome::ThreeNil
                } else {
                    SwissOutcome::Advanced
                }
            } else if losses >= SWISS_LOSSES {
                if wins == 0 {
                    SwissOutcome::ZeroThree
                } else {
                    SwissOutcome::Eliminated
                }
            } else {
                return None;
            };
            Some((team, outcome))
        })
        .collect()
}

/// Pick'em Points of a Swiss Pick'em: 1 per correct Pick among the settled
/// teams. A "3-0" Pick needs exactly 3-0, "0-3" exactly 0-3, and an
/// "advance" Pick a team that advanced with at least one loss.
pub fn score_swiss(pickem: &SwissPickem, outcomes: &HashMap<TeamId, SwissOutcome>) -> u32 {
    let correct = |teams: &[TeamId], wanted: SwissOutcome| {
        teams
            .iter()
            .filter(|t| outcomes.get(t) == Some(&wanted))
            .count() as u32
    };
    correct(&pickem.three_nil, SwissOutcome::ThreeNil)
        + correct(&pickem.zero_three, SwissOutcome::ZeroThree)
        + correct(&pickem.advance, SwissOutcome::Advanced)
}

/// When a Stage's Pick'em Locks: the moment its first Match starts (shared by
/// every Pick'em shape). `stored` is the Lock already on record, which never
/// changes. Otherwise it is the earliest start among the Stage's Matches
/// (actual start; "now" for one seen live without a time; else the Scheduled
/// Start), provided that is not after `now`.
pub fn lock(
    view: &EventView,
    stage: StageOrdinal,
    stored: Option<DateTime<Utc>>,
    now: DateTime<Utc>,
) -> Option<DateTime<Utc>> {
    if stored.is_some() {
        return stored;
    }
    view.matches
        .iter()
        .filter(|m| m.data.stage == stage && m.data.status != MatchStatus::Cancelled)
        .filter_map(|m| {
            m.data.actual_start.or(if m.data.has_started() {
                Some(now)
            } else {
                m.data.scheduled_start
            })
        })
        .min()
        .filter(|start| *start <= now)
}

/// The Scheduled Start of the Stage's earliest Match: when the Lock is
/// expected, for a countdown.
pub fn expected_lock(view: &EventView, stage: StageOrdinal) -> Option<DateTime<Utc>> {
    view.matches
        .iter()
        .filter(|m| m.data.stage == stage && m.data.status != MatchStatus::Cancelled)
        .filter_map(|m| m.data.actual_start.or(m.data.scheduled_start))
        .min()
}

/// Ranks by Points, highest first, sharing a place on equal Points (1, 1, 3).
pub fn rank<T>(mut entries: Vec<(T, u32)>) -> Vec<(u32, T, u32)> {
    entries.sort_by_key(|e| std::cmp::Reverse(e.1));
    let mut ranked = Vec::with_capacity(entries.len());
    let mut place = 0;
    let mut previous = None;
    for (i, (item, points)) in entries.into_iter().enumerate() {
        if previous != Some(points) {
            place = i as u32 + 1;
            previous = Some(points);
        }
        ranked.push((place, item, points));
    }
    ranked
}

// ---------------------------------------------------------------------------
// The Playoffs bracket (#15)
// ---------------------------------------------------------------------------

/// The deepest bracket supported: 2^7 = 128 first-round slots. A slot's Pick
/// `slot` number is its heap index `2^rounds_before_final + slot`, so the
/// final is 1, the semifinals 2 and 3, the quarterfinals 4..=7, and it fits a
/// `u8` up to this depth. The winners of slots `i * 2` and `i * 2 + 1` meet
/// in slot `i`.
pub const MAX_BRACKET_ROUNDS: u8 = 7;

/// A slot's Pick number (see [`MAX_BRACKET_ROUNDS`]).
pub fn slot_id(rounds_before_final: u8, slot: u8) -> u8 {
    ((1u16 << rounds_before_final) + slot as u16) as u8
}

fn slot_in_bracket(rounds_before_final: u8, slot: u8) -> bool {
    rounds_before_final <= MAX_BRACKET_ROUNDS && (slot as u16) < (1u16 << rounds_before_final)
}

/// A slot of the bracket as the effective view shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BracketSlot {
    pub rounds_before_final: u8,
    pub slot: u8,
    /// A first slot of the bracket: its teams come from the data, not from
    /// earlier picks. Every slot of the deepest round is one, and so is any
    /// shallower slot whose feeder slots have no Match (a bye).
    pub leaf: bool,
    /// The teams of the slot's Match, `None` while still to be decided.
    pub teams: [Option<TeamId>; 2],
    /// Who actually won the slot's Match (finished, not Void).
    pub winner: Option<TeamId>,
}

impl BracketSlot {
    pub fn id(&self) -> u8 {
        slot_id(self.rounds_before_final, self.slot)
    }
}

/// The shape of a Playoffs Stage, from its Matches' placements. The deepest
/// round with a placed Match, `R` rounds before the final, has `2^R` slots;
/// the bracket is the tree from the final down to there (so 4 teams are
/// semifinals and a final, 16 teams start at the round of 16). A slot with no
/// Match in the data is still a slot, with unknown teams.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Bracket {
    /// Ordered from the first round to the final, each round top to bottom.
    pub slots: Vec<BracketSlot>,
}

impl Bracket {
    pub fn of(view: &EventView, stage: StageOrdinal) -> Bracket {
        let placed: HashMap<(u8, u8), &crate::domain::Match> = view
            .matches
            .iter()
            .filter(|m| m.data.stage == stage)
            .filter_map(|m| match m.data.placement {
                MatchPlacement::Playoffs {
                    rounds_before_final: r,
                    slot: s,
                } if slot_in_bracket(r, s) => Some(((r, s), &m.data)),
                _ => None,
            })
            .collect();
        let Some(deepest) = placed.keys().map(|k| k.0).max() else {
            return Bracket::default();
        };
        let mut slots = Vec::new();
        let mut pending = vec![(0u8, 0u8)];
        while let Some((r, s)) = pending.pop() {
            let feeders = [(r + 1, 2 * s), (r + 1, 2 * s + 1)];
            let leaf = r == deepest || !feeders.iter().any(|f| placed.contains_key(f));
            let game = placed.get(&(r, s)).copied();
            slots.push(BracketSlot {
                rounds_before_final: r,
                slot: s,
                leaf,
                teams: game.map(|g| g.teams()).unwrap_or([None, None]),
                winner: game
                    .filter(|g| g.status == MatchStatus::Finished && !g.is_void())
                    .and_then(|g| g.winner),
            });
            if !leaf {
                pending.extend(feeders);
            }
        }
        slots.sort_by_key(|s| (std::cmp::Reverse(s.rounds_before_final), s.slot));
        Bracket { slots }
    }

    /// The Pick'em can be filled once every first slot's teams are known.
    pub fn ready(&self) -> bool {
        !self.slots.is_empty()
            && self
                .slots
                .iter()
                .filter(|s| s.leaf)
                .all(|s| s.teams.iter().all(Option::is_some))
    }

    /// The first slots' known teams, in id order: the teams a Member picks among.
    pub fn teams(&self) -> Vec<TeamId> {
        let mut teams: Vec<TeamId> = self
            .slots
            .iter()
            .filter(|s| s.leaf)
            .flat_map(|s| s.teams)
            .flatten()
            .collect();
        teams.sort();
        teams.dedup();
        teams
    }
}

/// A Playoffs Pick'em: the picked winner of every slot.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BracketPickem {
    /// `(rounds_before_final, slot, team)`.
    pub winners: Vec<(u8, u8, TeamId)>,
}

impl BracketPickem {
    pub fn to_picks(&self) -> Vec<Pick> {
        self.winners
            .iter()
            .map(|&(r, s, team)| Pick {
                kind: PickKind::Bracket,
                slot: slot_id(r, s),
                team,
            })
            .collect()
    }

    pub fn from_picks(picks: &[Pick], bracket: &Bracket) -> BracketPickem {
        BracketPickem {
            winners: bracket
                .slots
                .iter()
                .filter_map(|s| {
                    let p = picks
                        .iter()
                        .find(|p| p.kind == PickKind::Bracket && p.slot == s.id())?;
                    Some((s.rounds_before_final, s.slot, p.team))
                })
                .collect(),
        }
    }

    fn winner_of(&self, id: u8) -> Option<TeamId> {
        self.winners
            .iter()
            .find(|&&(r, s, _)| slot_id(r, s) == id)
            .map(|w| w.2)
    }

    /// Every slot picked exactly once, and each winner one of the slot's
    /// participants: a first slot's two teams, otherwise the winners picked
    /// in the two slots feeding it. A team may win several slots.
    pub fn validate(&self, bracket: &Bracket) -> Result<(), PickemError> {
        if !bracket.ready() {
            return Err(PickemError::BracketNotReady);
        }
        let mut wanted: Vec<u8> = bracket.slots.iter().map(BracketSlot::id).collect();
        let mut given: Vec<u8> = Vec::new();
        for &(r, s, _) in &self.winners {
            if !slot_in_bracket(r, s) {
                return Err(PickemError::WrongCount);
            }
            given.push(slot_id(r, s));
        }
        wanted.sort();
        given.sort();
        if wanted != given {
            return Err(PickemError::WrongCount);
        }
        for slot in &bracket.slots {
            let participants = if slot.leaf {
                slot.teams
            } else {
                [
                    self.winner_of(slot.id() * 2),
                    self.winner_of(slot.id() * 2 + 1),
                ]
            };
            let picked = self.winner_of(slot.id());
            if picked.is_none() || !participants.contains(&picked) {
                return Err(PickemError::TeamNotInStage);
            }
        }
        Ok(())
    }
}

/// Pick'em Points of a Playoffs Pick'em: 1 per slot whose picked winner
/// actually won that slot's Match.
pub fn score_bracket(pickem: &BracketPickem, bracket: &Bracket) -> u32 {
    bracket
        .slots
        .iter()
        .filter(|s| s.winner.is_some() && pickem.winner_of(s.id()) == s.winner)
        .count() as u32
}
