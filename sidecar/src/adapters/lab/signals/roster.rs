use super::{CandidateSignal, FittedSignal, MatchView, Observed};
use crate::adapters::{
    backtest::ReplayArchive,
    context::{liquipedia, names::team_key, vrs_diff::VrsRosterIndex},
};
use crate::*;
use chrono::Duration;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

#[derive(Clone, Copy)]
pub enum RosterSignal {
    StandIn,
    RecentChange,
    Stability,
    CoachChange,
}

impl RosterSignal {
    pub const ALL: [Self; 4] = [
        Self::StandIn,
        Self::RecentChange,
        Self::Stability,
        Self::CoachChange,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::StandIn => "roster_stand_in",
            Self::RecentChange => "roster_change_30d",
            Self::Stability => "roster_stability",
            Self::CoachChange => "roster_coach_change",
        }
    }

    fn feature(self, view: &MatchView) -> Result<Option<f64>> {
        let Some(roster) = &view.roster else {
            return Ok(None);
        };
        Ok(roster.at(view.cutoff)?.values(self).map(|[a, b]| {
            if matches!(self, Self::Stability) {
                (a / 30.0).ln_1p() - (b / 30.0).ln_1p()
            } else {
                a - b
            }
        }))
    }
}

pub(super) fn candidates() -> Vec<Arc<dyn CandidateSignal>> {
    RosterSignal::ALL
        .into_iter()
        .map(|signal| Arc::new(signal) as Arc<dyn CandidateSignal>)
        .collect()
}

impl CandidateSignal for RosterSignal {
    fn name(&self) -> &'static str {
        (*self).name()
    }
    fn description(&self) -> &'static str {
        match self {
            Self::StandIn => {
                "A stand-in in the active five, from the pre-cutoff Liquipedia revision."
            }
            Self::RecentChange => {
                "An active-five change bounded to the last 30 days by dated lineup observations."
            }
            Self::Stability => {
                "Days since the current five was first observed after a different five."
            }
            Self::CoachChange => {
                "An emergency coach substitute or a change from the previously observed active coach set."
            }
        }
    }
    fn uses_rosters(&self) -> bool {
        true
    }
    fn needs(&self) -> &'static str {
        match self {
            Self::StandIn => "a parsable active five for both teams",
            Self::RecentChange | Self::Stability => {
                "active fives and dated lineup history for both teams"
            }
            Self::CoachChange => "active fives and dated coach evidence for both teams",
        }
    }
    fn fit(&self, training: &[Observed]) -> Result<Box<dyn FittedSignal>> {
        let mut rows = Vec::new();
        for m in training {
            if let (Some(feature), Some(anchor)) = (self.feature(&m.view)?, m.view.anchor()?) {
                rows.push((feature, logit(anchor), m.team_a_won));
            }
        }
        Ok(Box::new(FittedRoster {
            signal: *self,
            weight: fit_weight(&rows),
        }))
    }
}

// Fit one ridge-regularised coefficient with the anchor logit as a fixed offset.
fn fit_weight(rows: &[(f64, f64, bool)]) -> f64 {
    // The absolute non-penalty gradient is bounded by the sum of |x|.
    let bound: f64 = rows.iter().map(|row| row.0.abs()).sum();
    if bound == 0.0 {
        return 0.0;
    }
    let (mut low, mut high) = (-bound, bound);
    for _ in 0..48 {
        let weight = (low + high) / 2.0;
        let gradient = weight
            + rows
                .iter()
                .map(|&(x, anchor, won)| {
                    x * (sigmoid(anchor + weight * x) - if won { 1.0 } else { 0.0 })
                })
                .sum::<f64>();
        if gradient > 0.0 {
            high = weight;
        } else {
            low = weight;
        }
    }
    (low + high) / 2.0
}

fn logit(p: f64) -> f64 {
    let p = p.clamp(super::super::stats::CLIP, 1.0 - super::super::stats::CLIP);
    (p / (1.0 - p)).ln()
}

fn sigmoid(logit: f64) -> f64 {
    1.0 / (1.0 + (-logit).exp())
}

struct FittedRoster {
    signal: RosterSignal,
    weight: f64,
}

impl FittedSignal for FittedRoster {
    fn predict(&self, view: &MatchView) -> Result<Option<AsOf<f64>>> {
        let Some(x) = self.signal.feature(view)? else {
            return Ok(None);
        };
        // A roster can be computed on a Match without an anchor. Its variant
        // uses even odds as the offset, and the lab leaves it out of paired metrics.
        let anchor = view.anchor()?.unwrap_or(0.5);
        let p = if self.weight * x == 0.0 {
            anchor
        } else {
            sigmoid(logit(anchor) + self.weight * x)
        };
        let observation = view
            .observed_at()
            .max(view.roster.as_ref().map(AsOf::observed_at))
            .unwrap();
        Ok(Some(AsOf::new(p, observation, view.cutoff)?))
    }
}

#[derive(Clone)]
pub struct RosterFeatures([TeamFeatures; 2]);
#[derive(Clone)]
struct TeamFeatures {
    stand_in: bool,
    recent_change: Option<bool>,
    stability_days: Option<f64>,
    coach_change: Option<bool>,
}

impl RosterFeatures {
    /// The raw feature values, in Match side order.
    pub fn values(&self, signal: RosterSignal) -> Option<[f64; 2]> {
        let value = |team: &TeamFeatures| match signal {
            RosterSignal::StandIn => Some(flag(team.stand_in)),
            RosterSignal::RecentChange => team.recent_change.map(flag),
            RosterSignal::Stability => team.stability_days,
            RosterSignal::CoachChange => team.coach_change.map(flag),
        };
        Some([value(&self.0[0])?, value(&self.0[1])?])
    }
}

fn flag(value: bool) -> f64 {
    f64::from(u8::from(value))
}

type Names = BTreeSet<String>;
type CoachHistory = BTreeMap<String, Vec<(Instant, Names)>>;

/// Point-in-time replay of the saved Liquipedia and Valve roster observations.
pub struct RosterHistory {
    pages: BTreeMap<String, Vec<AsOf<EventRosters>>>,
    event_titles: BTreeMap<i64, String>,
    coaches: CoachHistory,
    valve: VrsRosterIndex,
}

impl RosterHistory {
    pub fn new(archive: &ReplayArchive) -> Result<Self> {
        let mut pages = BTreeMap::new();
        let mut coaches = CoachHistory::new();
        for (title, revisions) in &archive.event_pages {
            let mut parsed = Vec::new();
            let mut seen = BTreeSet::new();
            for rev in revisions {
                // The archive stores each response with its revision timestamp.
                let limit = rev.timestamp + Duration::nanoseconds(1);
                let Some(event) = liquipedia::parse_revision_rosters(&rev.body, limit)? else {
                    continue;
                };
                let data = event.at(limit)?;
                if !seen.insert(data.revid) {
                    continue;
                }
                for team in &data.teams {
                    let names = coach_names(team);
                    if !names.is_empty() {
                        for name in std::iter::once(&team.team).chain(&team.aliases) {
                            coaches
                                .entry(team_key(name))
                                .or_default()
                                .push((event.observed_at(), names.clone()));
                        }
                    }
                }
                parsed.push(event);
            }
            parsed.sort_by_key(AsOf::observed_at);
            pages.insert(title.clone(), parsed);
        }
        for timeline in coaches.values_mut() {
            timeline.sort_by_key(|(at, _)| *at);
        }
        let valve = VrsRosterIndex::parse(archive.standings.iter().map(String::as_str))?;
        Ok(Self {
            pages,
            event_titles: archive.event_titles.clone(),
            coaches,
            valve,
        })
    }

    pub fn at(&self, game: &Match, cutoff: Instant) -> Result<Option<AsOf<RosterFeatures>>> {
        let Some(revisions) = self
            .event_titles
            .get(&game.event_id)
            .and_then(|title| self.pages.get(title))
        else {
            return Ok(None);
        };
        let end = revisions.partition_point(|rev| rev.observed_at() < cutoff);
        let Some((current, previous)) = revisions[..end].split_last() else {
            return Ok(None);
        };
        let event = current.at(cutoff)?;
        let (Some(a), Some(b)) = (
            liquipedia::find_team_roster(event, &game.team_a.name),
            liquipedia::find_team_roster(event, &game.team_b.name),
        ) else {
            return Ok(None);
        };
        let mut observed = current.observed_at();
        let (Some(a), Some(b)) = (
            self.team_features(a, previous, current.observed_at(), cutoff, &mut observed)?,
            self.team_features(b, previous, current.observed_at(), cutoff, &mut observed)?,
        ) else {
            return Ok(None);
        };
        Ok(Some(AsOf::new(RosterFeatures([a, b]), observed, cutoff)?))
    }

    fn team_features(
        &self,
        team: &TeamRoster,
        previous: &[AsOf<EventRosters>],
        revision: Instant,
        cutoff: Instant,
        observed: &mut Instant,
    ) -> Result<Option<TeamFeatures>> {
        let active = names(&team.active_players);
        if active.len() != 5 {
            return Ok(None);
        }
        let stand_in = team
            .stand_ins
            .iter()
            .any(|sub| active.contains(&sub.player.to_lowercase()));
        let mut timeline = BTreeMap::new();
        for roster in self
            .valve
            .roster_history(&team.team, &team.aliases, cutoff)?
        {
            let lineup = names(roster.at(cutoff)?);
            if lineup.len() == 5 {
                timeline.insert(roster.observed_at(), lineup);
            }
        }
        for rev in previous {
            if let Some(prior) = liquipedia::find_team_roster(rev.at(cutoff)?, &team.team) {
                let lineup = names(&prior.active_players);
                if lineup.len() == 5 {
                    timeline.insert(rev.observed_at(), lineup);
                }
            }
        }
        // A newer Valve roster that contradicts the last wiki observation
        // leaves the timing of the Event lineup change unresolved.
        let later_conflict = timeline
            .last_key_value()
            .is_some_and(|(&at, lineup)| at >= revision && lineup != &active);
        if let Some((&at, _)) = timeline.last_key_value() {
            *observed = (*observed).max(at);
        }
        let (recent_change, stability_days) = if later_conflict {
            (None, None)
        } else {
            lineup_features(&active, revision, cutoff, timeline)
        };

        let coaches = coach_names(team);
        let coach_change = if team
            .coaches
            .iter()
            .any(|coach| !coach.former && coach.emergency_sub)
        {
            Some(true)
        } else if coaches.is_empty() {
            None
        } else {
            std::iter::once(&team.team)
                .chain(&team.aliases)
                .filter_map(|name| self.coaches.get(&team_key(name)))
                .filter_map(|history| {
                    let end = history.partition_point(|(at, _)| *at < revision);
                    history[..end].last()
                })
                .max_by_key(|(at, _)| *at)
                .map(|(_, prior)| prior != &coaches)
        };

        Ok(Some(TeamFeatures {
            stand_in,
            recent_change,
            stability_days,
            coach_change,
        }))
    }
}

fn lineup_features(
    active: &Names,
    revision: Instant,
    cutoff: Instant,
    mut timeline: BTreeMap<Instant, Names>,
) -> (Option<bool>, Option<f64>) {
    timeline.insert(revision, active.clone());
    let mut first_same = revision;
    let mut last_different = None;
    for (at, lineup) in timeline.into_iter().rev() {
        if &lineup != active {
            last_different = Some(at);
            break;
        }
        first_same = at;
    }
    let recent_change = if cutoff - first_same >= Duration::days(30) {
        Some(false)
    } else {
        last_different
            .filter(|&at| cutoff - at <= Duration::days(30))
            .map(|_| true)
    };
    let stability_days =
        last_different.map(|_| (cutoff - first_same).num_seconds() as f64 / 86_400.0);
    (recent_change, stability_days)
}

fn names(players: &[String]) -> Names {
    players
        .iter()
        .map(|p| p.trim().to_lowercase())
        .filter(|p| !p.is_empty())
        .collect()
}

fn coach_names(team: &TeamRoster) -> Names {
    team.coaches
        .iter()
        .filter(|coach| !coach.former)
        .map(|coach| coach.player.to_lowercase())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_change_needs_dated_evidence_and_the_age_uses_its_first_observation() {
        let cutoff: Instant = "2026-06-30T00:00:00Z".parse().unwrap();
        let old = Names::from(["old".into()]);
        let current = Names::from(["current".into()]);
        let at = |days| cutoff - Duration::days(days);
        assert_eq!(
            lineup_features(&current, at(1), cutoff, BTreeMap::new()),
            (None, None)
        );
        assert_eq!(
            lineup_features(
                &current,
                at(1),
                cutoff,
                BTreeMap::from([(at(40), old.clone())])
            ),
            (None, Some(1.0)),
        );
        assert_eq!(
            lineup_features(
                &current,
                at(1),
                cutoff,
                BTreeMap::from([(at(30), old), (at(10), current.clone()),])
            ),
            (Some(true), Some(10.0)),
        );
        assert_eq!(
            lineup_features(
                &current,
                at(1),
                cutoff,
                BTreeMap::from([(at(30), current.clone())])
            ),
            (Some(false), None),
        );
    }

    #[test]
    fn offset_fit_recovers_the_direction_and_uses_zero_without_feature_variation() {
        assert_eq!(fit_weight(&[(0.0, 0.0, true)]), 0.0);
        let weight = fit_weight(&[(1.0, 0.0, true), (-1.0, 0.0, false)]);
        assert!(weight > 0.0);
        assert!((weight - 2.0 * (1.0 - sigmoid(weight))).abs() < 1e-10);
    }
}
