//! The protocol's runner: reads every scored Match of the archive through the
//! replay ports at its cutoff, then evaluates each Candidate Signal Event by
//! Event, walk-forward, against the anchor.
use super::holdout::{Freeze, Holdout};
use super::report::{
    CandidateReport, DirectComparison, EventInfo, EventRow, LabReport, MarketSubset, Metrics, SubsetReport,
    TierView,
};
use super::selection::{self, Member};
use super::signals::{self, CandidateSignal, FittedSignal, MatchView, Observed};
use super::signals::roster::RosterHistory;
use super::stats::{self, Verdict};
use super::{PROTOCOL_VERSION, Protocol};
use crate::adapters::backtest::feeds::{MarketSource, PriceHistory, ReplayOdds};
use crate::adapters::backtest::match_context::MatchContext;
use crate::adapters::backtest::{Feeds, ReplayArchive, ReplayMatch, ReplayStage};
use crate::*;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

pub struct Lab {
    pub archive: ReplayArchive,
    /// The replay ports. Tests replace one to inject a datum.
    pub feeds: Feeds,
    /// Price points of each market source (replay: the saved histories).
    pub price_history: Arc<dyn PriceHistory>,
    pub protocol: Protocol,
    /// The Candidate Signals evaluated, in report order (default: the registry).
    pub signals: Vec<Arc<dyn CandidateSignal>>,
    /// Which Stages are the Holdout (default: spec #33's).
    pub holdout: Holdout,
    /// The Holdout evaluation already recorded, if any (the CLI reads it
    /// from `lab_runs`): a run that would evaluate it otherwise is refused.
    pub freeze: Option<Freeze>,
    /// When the Holdout is evaluated, for the report (the CLI: the recorded
    /// evaluation's start, else this run's).
    pub evaluated_at: Option<Instant>,
    /// Which tiers are evaluated and which may train a fit (default: all).
    pub tiers: TierFilter,
    /// Each team's VRS and HLTV rank by date (ticket #57).
    pub ranks: Arc<crate::adapters::backtest::rank_history::RankHistory>,
}

/// The tier the secondary view of the report slices to (ticket #55).
pub const SLICE_TIERS: &str = "s";

/// Which Events, by their archived PandaScore tier, are evaluated and which
/// may train a fit. `None`: any Event, tiered or not. An Event with no tier is
/// in neither set of a given filter; the report lists it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TierFilter {
    pub evaluate: Option<BTreeSet<String>>,
    pub train: Option<BTreeSet<String>>,
}
impl TierFilter {
    /// `"s,a"` into a tier set.
    pub fn parse(list: &str) -> Result<BTreeSet<String>> {
        let set: BTreeSet<String> = list
            .split(',')
            .map(|t| t.trim().to_lowercase())
            .filter(|t| !t.is_empty())
            .collect();
        if set.is_empty() {
            return Err("a tier list needs at least one tier".into());
        }
        Ok(set)
    }
    fn within(set: &Option<BTreeSet<String>>, event: &Event) -> bool {
        set.as_ref()
            .is_none_or(|s| event.tier.as_ref().is_some_and(|t| s.contains(t)))
    }
    pub(crate) fn evaluates(&self, event: &Event) -> bool {
        Self::within(&self.evaluate, event)
    }
    pub(crate) fn trains_on(&self, event: &Event) -> bool {
        Self::within(&self.train, event)
    }
}
fn show(set: &Option<BTreeSet<String>>) -> String {
    set.as_ref()
        .map_or("all".into(), |s| s.iter().cloned().collect::<Vec<_>>().join(","))
}

/// One Match as one Candidate Signal saw it: the anchor's probability and the
/// variant's, for team A (`None`: not predicted).
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct LabPrediction {
    pub signal: String,
    pub match_id: i64,
    pub event_id: i64,
    pub baseline: Option<f64>,
    pub variant: Option<f64>,
}

pub struct LabOutcome {
    pub report: LabReport,
    /// Every Candidate Signal alone, on the training side.
    pub predictions: Vec<LabPrediction>,
    /// The frozen combination, walk-forward on the training side (`signal`: "combination").
    pub combination: Vec<LabPrediction>,
    /// The frozen combination on the Holdout (empty while it is pending).
    pub holdout: Vec<LabPrediction>,
}

/// An Event of the walk-forward: when it began (its first Match's cutoff) and
/// when it ended (its last result), and its scored Matches.
pub(crate) struct Event {
    pub(crate) info: EventInfo,
    pub(crate) matches: Vec<Observed>,
    /// PandaScore tier of the Event (its first Stage that names one).
    pub(crate) tier: Option<String>,
    /// When its last result became known; `None`: not complete (a Stage
    /// still in play, or a result without a reported finish), so it trains
    /// nothing.
    pub(crate) ended: Option<Instant>,
}
impl Event {
    /// Whether this Event's Matches may train a fit for `evaluated`: it is
    /// another Event (never a Stage of the evaluated one, on either side of
    /// the boundary), complete, and it ended before `evaluated` began.
    pub(crate) fn trains(&self, evaluated: &EventInfo) -> bool {
        self.info.id != evaluated.id && self.ended.is_some_and(|end| end < evaluated.begins)
    }
}

/// The training side, and the Holdout's Stages (not read yet).
pub(super) struct Split {
    pub(super) training: Vec<Event>,
    /// The Holdout Stages of the archive, for the report.
    pub(super) holdout_stages: Vec<String>,
}

/// One Candidate Signal evaluated: its report row, its per-Match predictions
/// and its fit for each training-side Event.
type Evaluation = (CandidateReport, Vec<LabPrediction>, Vec<Box<dyn FittedSignal>>);

/// `slice` without the Matches in `hidden` (shared as it is when none is).
fn hide_results(
    slice: HistorySlice,
    hidden: &BTreeSet<i64>,
    cutoff: Instant,
) -> Result<HistorySlice> {
    let entries = slice.as_slice();
    let mut keep = Vec::with_capacity(entries.len());
    for e in entries {
        if !hidden.contains(&e.at(cutoff)?.match_id) {
            keep.push(e.clone());
        }
    }
    Ok(if keep.len() == entries.len() {
        slice
    } else {
        HistorySlice::from_vec(keep)
    })
}

/// `slice` without the appearances in Matches of `hidden`.
fn hide_players(
    slice: PlayerStatsSlice,
    hidden: &BTreeSet<i64>,
    cutoff: Instant,
) -> Result<PlayerStatsSlice> {
    let entries = slice.as_slice();
    let mut keep = Vec::with_capacity(entries.len());
    for e in entries {
        if !hidden.contains(&e.at(cutoff)?.match_id) {
            keep.push(e.clone());
        }
    }
    Ok(if keep.len() == entries.len() {
        slice
    } else {
        PlayerStatsSlice::from_vec(keep)
    })
}

impl Lab {
    pub fn new(archive: ReplayArchive) -> Result<Self> {
        Ok(Self {
            feeds: Feeds::replay(&archive)?,
            price_history: Arc::new(ReplayOdds::new(&archive)),
            ranks: Arc::new(crate::adapters::backtest::rank_history::RankHistory::new(&archive)?),
            archive,
            protocol: Protocol::default(),
            signals: signals::registry(),
            holdout: Holdout::default(),
            freeze: None,
            evaluated_at: None,
            tiers: TierFilter::default(),
        })
    }

    pub(super) async fn view(
        &self,
        stage: &ReplayStage,
        m: &ReplayMatch,
        rosters: Option<&RosterHistory>,
        hidden: &BTreeSet<i64>,
    ) -> Result<MatchView> {
        let context = self.archive.match_context.view(stage, m);
        self.view_of(&m.game, m.cutoff(), context, rosters, hidden).await
    }

    /// The view of `game` at `cutoff`, also for a Match the archive does not
    /// hold (a hypothetical Pick'em pairing), with the context given. The
    /// Matches in `hidden` (the Holdout's, for a training-side view) are left
    /// out of every history it holds.
    pub(super) async fn view_of(
        &self,
        game: &Match,
        cutoff: Instant,
        context: MatchContext,
        rosters: Option<&RosterHistory>,
        hidden: &BTreeSet<i64>,
    ) -> Result<MatchView> {
        let facts = self.feeds.context.facts(game, cutoff).await?;
        let mut team_a_roster = None;
        let mut team_b_roster = None;
        for fact in facts {
            let obs = fact.observed_at();
            let val = fact.at(cutoff)?;
            if let Ok(ContextFact::LiquipediaRoster(roster)) =
                serde_json::from_value::<ContextFact>(val.clone())
            {
                let side = roster.match_side.clone();
                let wrapped = AsOf::new(roster, obs, cutoff)?;
                match side.as_str() {
                    "team_a" => team_a_roster = Some(wrapped),
                    "team_b" => team_b_roster = Some(wrapped),
                    _ => {}
                }
            }
        }
        let mut player_stats = BTreeMap::new();
        for roster in [&team_a_roster, &team_b_roster].into_iter().flatten() {
            let r = roster.at(cutoff)?;
            for nick in r
                .active_players
                .iter()
                .chain(r.coaches.iter().filter(|c| c.emergency_sub && !c.former).map(|c| &c.player))
            {
                let key = player_key(nick);
                if key.is_empty() || player_stats.contains_key(&key) {
                    continue;
                }
                let slice = self.feeds.player_stats.player_matches(nick, cutoff).await?;
                slice.at(cutoff)?;
                player_stats.insert(key, hide_players(slice, hidden, cutoff)?);
            }
        }
        let mut market_history = BTreeMap::new();
        for source in MarketSource::ALL {
            let points = self.price_history.points(source, game, cutoff).await?;
            if !points.is_empty() {
                market_history.insert(source, points);
            }
        }
        let view = MatchView {
            map_history: self.archive.map_history.before(cutoff),
            veto_history: self.archive.veto_history.before(cutoff),
            ranks: self.ranks.at(game, cutoff, hidden)?,
            market_history,
            signal: self.feeds.signal.signal(game, cutoff).await?,
            market: self.feeds.odds.quote(game, cutoff).await?,
            roster: rosters
                .map(|history| history.at(game, cutoff))
                .transpose()?
                .flatten(),
            history_before: hide_results(
                self.feeds.history.matches_before(cutoff).await?,
                hidden,
                cutoff,
            )?,
            team_a_history: hide_results(
                self.feeds
                    .history
                    .team_matches(&game.team_a.name, cutoff)
                    .await?,
                hidden,
                cutoff,
            )?,
            team_b_history: hide_results(
                self.feeds
                    .history
                    .team_matches(&game.team_b.name, cutoff)
                    .await?,
                hidden,
                cutoff,
            )?,
            context: context.sliced(cutoff),
            team_a_roster,
            team_b_roster,
            player_stats,
            game: game.clone(),
            cutoff,
        };
        // A port that answers with a datum at or after the cutoff ends the run.
        view.signal()?;
        view.market()?;
        view.history_before()?;
        view.team_a_history()?;
        view.team_b_history()?;
        for source in MarketSource::ALL {
            view.points(source)?;
        }
        view.team_a_roster()?;
        view.team_b_roster()?;
        Ok(view)
    }

    /// When each Event's last result became known (`None`: not complete).
    fn event_ends(&self) -> BTreeMap<i64, Option<Instant>> {
        let mut ends: BTreeMap<i64, Option<Instant>> = BTreeMap::new();
        for stage in &self.archive.stages {
            let end = stage.known_end();
            ends.entry(stage.event_id)
                .and_modify(|e| *e = e.zip(end).map(|(a, b)| a.max(b)))
                .or_insert(end);
        }
        ends
    }

    /// The PandaScore ids of the archive's Holdout Stages.
    pub fn holdout_stages(&self) -> BTreeSet<i64> {
        let ends = self.event_ends();
        self.archive
            .stages
            .iter()
            .filter(|stage| self.holdout.contains(stage, ends[&stage.event_id]))
            .map(|stage| stage.id)
            .collect()
    }

    /// The PandaScore ids of the Matches of `stages`.
    pub fn holdout_matches(&self, stages: &BTreeSet<i64>) -> BTreeSet<i64> {
        self.archive
            .stages
            .iter()
            .filter(|s| stages.contains(&s.id))
            .flat_map(|s| s.matches.iter().map(|m| m.game.id))
            .collect()
    }

    /// The training side's Events, and the Holdout's Stages by name; the
    /// Holdout's Matches are not read here (`holdout_events`, the final step).
    pub(super) async fn events(&self) -> Result<Split> {
        let holdout_ids = self.holdout_stages();
        let holdout_stages = self
            .archive
            .stages
            .iter()
            .filter(|s| holdout_ids.contains(&s.id))
            .map(|s| format!("{}: {} ({} scored)", s.event, s.name, s.matches.len()))
            .collect();
        Ok(Split {
            training: self.side(false, &holdout_ids).await?,
            holdout_stages,
        })
    }

    /// The Holdout's Events, read once the selection is frozen.
    pub(super) async fn holdout_events(&self) -> Result<Vec<Event>> {
        self.side(true, &self.holdout_stages()).await
    }

    /// The Events of one side of the boundary, each Match read at its cutoff.
    /// A training-side view holds no Holdout Match in any history.
    async fn side(&self, holdout_side: bool, holdout_ids: &BTreeSet<i64>) -> Result<Vec<Event>> {
        let rosters = self
            .signals
            .iter()
            .any(|signal| signal.uses_rosters())
            .then(|| RosterHistory::new(&self.archive))
            .transpose()?;
        let ends = self.event_ends();
        let hidden = if holdout_side {
            BTreeSet::new()
        } else {
            self.holdout_matches(holdout_ids)
        };
        let excluded = if holdout_side { BTreeSet::new() } else { holdout_ids.clone() };
        let map_history = self.archive.map_history.without_stages(&excluded);
        let mut veto_history = self.archive.veto_history.without_stages(&excluded);
        if holdout_side {
            veto_history.exclude_round_scores(holdout_ids);
        }
        let mut by_id: BTreeMap<i64, Event> = BTreeMap::new();
        for stage in &self.archive.stages {
            if holdout_ids.contains(&stage.id) != holdout_side {
                continue;
            }
            let event = by_id.entry(stage.event_id).or_insert_with(|| Event {
                info: EventInfo {
                    id: stage.event_id,
                    title: stage.event.clone(),
                    begins: stage.cutoff,
                    ends: stage.cutoff,
                    matches: 0,
                },
                matches: vec![],
                tier: None,
                // The whole Event's end, both sides of the boundary.
                ended: ends[&stage.event_id],
            });
            event.info.begins = event.info.begins.min(stage.cutoff);
            event.tier = event.tier.take().or_else(|| stage.tier.clone());
            for m in &stage.matches {
                event.info.ends = event.info.ends.max(m.known_finish().unwrap_or(m.finished_at));
                let mut view = self.view(stage, m, rosters.as_ref(), &hidden).await?;
                view.map_history = map_history.before(view.cutoff);
                view.veto_history = veto_history.before(view.cutoff);
                event.matches.push(Observed {
                    view,
                    team_a_won: m.winner == m.game.team_a.id,
                });
            }
        }
        let mut events: Vec<Event> = by_id.into_values().collect();
        for e in &mut events {
            e.info.matches = e.matches.len();
        }
        events.sort_by_key(|e| (e.info.begins, e.info.id));
        Ok(events)
    }

    /// Every Candidate Signal alone against the anchor on the training side,
    /// as the first step of `run`: no selection, and the Holdout not read.
    pub async fn candidates(&self) -> Result<Vec<CandidateReport>> {
        let events = self.events().await?.training;
        let training_archive = self.archive.without_stages(&self.holdout_stages());
        self.signals
            .iter()
            .map(|signal| {
                self.evaluate(signal.as_ref(), &events, &training_archive, &self.tiers)
                    .map(|(report, _, _)| report)
            })
            .collect()
    }

    /// Runs the protocol. Any datum observed at or after its Match's cutoff,
    /// read from a port or returned by a Candidate Signal, aborts the run with
    /// `TemporalLeakError`.
    ///
    /// Order of work (spec #33): each Candidate Signal alone against the
    /// anchor, then forward selection of those marked `promote`, both on the
    /// training side only; then the frozen combination on the Holdout, once.
    pub async fn run(&self) -> Result<LabOutcome> {
        let Split {
            training: events,
            holdout_stages,
        } = self.events().await?;
        // What a Candidate Signal is told the archive holds: the training side.
        let training_archive = self.archive.without_stages(&self.holdout_stages());
        let mut predictions = Vec::new();
        let mut candidates = Vec::new();
        let mut fitted = Vec::new();
        for signal in &self.signals {
            let (report, rows, fits) =
                self.evaluate(signal.as_ref(), &events, &training_archive, &self.tiers)?;
            if report.verdict == Verdict::Promote {
                fitted.push((signal.clone(), fits));
            }
            candidates.push(report);
            predictions.extend(rows);
        }
        let promoted: Vec<Member> = fitted
            .iter()
            .map(|(signal, fits)| Member {
                signal: signal.clone(),
                fits,
            })
            .collect();
        let tier_view = self.tier_view(&events, &training_archive)?;
        let selected = selection::select(&events, &promoted, &self.protocol)?;
        // The selection is frozen here; only now is the Holdout read.
        let frozen: Vec<Member> = selected
            .report
            .frozen
            .iter()
            .filter_map(|name| promoted.iter().find(|m| m.signal.name() == name))
            .map(|m| Member {
                signal: m.signal.clone(),
                fits: m.fits,
            })
            .collect();
        let training: Vec<_> = events
            .iter()
            .flat_map(|e| e.matches.iter().cloned())
            .collect();
        let mut deployment = BTreeMap::new();
        for member in &promoted {
            if let Some(parameters) = member.signal.fit(&training)?.deployment()? {
                deployment.insert(member.signal.name().into(), parameters);
            }
        }
        if !frozen.is_empty() {
            deployment.insert(
                "combination".into(),
                serde_json::to_value(selection::deployment_combiner(&events, &frozen)?)?,
            );
        }
        self.check_freeze(&frozen)?;
        // The selection is frozen: only now are the Holdout's Matches read.
        let holdout = self.holdout_events().await?;
        let evaluated = selection::holdout(
            &self.holdout,
            &events,
            &holdout,
            holdout_stages,
            &frozen,
            &self.protocol,
            self.evaluated_at,
        )?;
        Ok(LabOutcome {
            report: LabReport {
                protocol_version: PROTOCOL_VERSION.into(),
                seed: self.protocol.seed,
                resamples: self.protocol.resamples,
                level: self.protocol.level,
                min_matches: self.protocol.min_matches,
                archive_digest: self.archive.digest.clone(),
                archive_files: self.archive.files,
                baseline: "the anchor: HybridAnchored without LLM, 0.80 market and 0.20 Valve \
                           rating logit (`debate::base`)"
                    .into(),
                events: events.iter().map(|e| e.info.clone()).collect(),
                deployment,
                candidates,
                selection: selected.report,
                holdout: evaluated.report,
                tier_view,
            },
            predictions,
            combination: selected.predictions,
            holdout: evaluated.predictions,
        })
    }

    /// Refuses to evaluate a Holdout that was already evaluated, unless this
    /// run reproduces that evaluation: the same archive, the same frozen
    /// combination. A Holdout with no Match played is not evaluated.
    fn check_freeze(&self, frozen: &[Member]) -> Result<()> {
        let Some(freeze) = &self.freeze else {
            return Ok(());
        };
        let stages = self.holdout_stages();
        let played = self
            .archive
            .stages
            .iter()
            .any(|s| stages.contains(&s.id) && !s.matches.is_empty());
        if !played {
            return Ok(());
        }
        let names: Vec<&str> = frozen.iter().map(|m| m.signal.name()).collect();
        let record = if names.is_empty() {
            "evaluated: anchor".to_string()
        } else {
            format!("evaluated: {}", names.join(" + "))
        };
        if freeze.archive_sha256 == self.archive.digest && freeze.holdout == record {
            return Ok(());
        }
        Err(format!(
            "the Holdout was already evaluated on {} ({}, archive {}); this run ({record}, archive {}) would evaluate it again, so it is refused",
            freeze.evaluated_at.format("%Y-%m-%d %H:%M UTC"),
            freeze.holdout,
            freeze.archive_sha256,
            self.archive.digest
        )
        .into())
    }

    /// The walk-forward boundary: the training Matches of `event` are those of
    /// the Events that ended before it began.
    pub(super) fn training(events: &[Event], event: &Event) -> Vec<Observed> {
        Self::training_in(events, event, &TierFilter::default())
    }

    /// `training`, with only the Events whose tier `tiers.train` allows. The
    /// tier is a further restriction: it never admits an Event the walk-forward
    /// boundary excludes, and `events` holds no Holdout Match.
    pub(crate) fn training_in(events: &[Event], event: &Event, tiers: &TierFilter) -> Vec<Observed> {
        events
            .iter()
            .filter(|e| e.trains(&event.info) && tiers.trains_on(e))
            .flat_map(|e| e.matches.iter().cloned())
            .collect()
    }

    /// The secondary view of ticket #55: every Candidate Signal on the
    /// `SLICE_TIERS` slice, fitted on the Events `self.tiers.train` allows.
    fn tier_view(&self, events: &[Event], archive: &ReplayArchive) -> Result<TierView> {
        let slice = TierFilter {
            evaluate: Some(TierFilter::parse(SLICE_TIERS)?),
            train: self.tiers.train.clone(),
        };
        let mut matches_by_tier: BTreeMap<String, usize> = BTreeMap::new();
        let mut untiered = Vec::new();
        for e in events {
            match &e.tier {
                Some(t) => *matches_by_tier.entry(t.clone()).or_default() += e.matches.len(),
                None => untiered.push(format!("{}: {}", e.info.title, e.matches.len())),
            }
        }
        let mut candidates = Vec::new();
        for signal in &self.signals {
            candidates.push(self.evaluate(signal.as_ref(), events, archive, &slice)?.0);
        }
        Ok(TierView {
            evaluated: show(&self.tiers.evaluate),
            train: show(&self.tiers.train),
            slice: SLICE_TIERS.into(),
            matches_by_tier,
            untiered,
            candidates,
        })
    }

    fn evaluate(
        &self,
        signal: &dyn CandidateSignal,
        events: &[Event],
        archive: &ReplayArchive,
        tiers: &TierFilter,
    ) -> Result<Evaluation> {
        let missing = signal.missing_data(archive);
        let mut rows = Vec::new();
        let mut fits = Vec::new();
        // (event, a won, baseline, variant, has market) of every paired Match.
        let mut paired: Vec<(i64, bool, f64, f64, bool)> = Vec::new();
        // (event, a won, valve series, unblended rating) for direct comparison with Valve.
        // Whether each paired Match is in the Candidate Signal's own subset, in `paired` order.
        let mut in_subset: Vec<bool> = Vec::new();
        let mut vs_valve_paired: Vec<(i64, bool, f64, f64)> = Vec::new();
        let mut vs_valve_per_event = Vec::new();
        let mut per_event = Vec::new();
        for event in events {
            let training = Self::training_in(events, event, tiers);
            let fitted = match missing {
                None => Some(signal.fit(&training)?),
                Some(_) => None,
            };
            if !tiers.evaluates(event) {
                // Fitted all the same (selection reads one fit per Event), but
                // not scored: it is outside the evaluated tiers.
                fits.extend(fitted);
                continue;
            }
            let parameters = fitted.as_ref().and_then(|f| f.parameters());
            let (mut computed, mut both, mut diff) = (0, 0, 0.0);
            let (mut v_computed, mut v_both, mut v_diff) = (0, 0, 0.0);
            for m in &event.matches {
                let baseline = m.view.anchor()?;
                let variant = match &fitted {
                    Some(f) => match f.predict(&m.view)? {
                        Some(p) => Some(*p.at(m.view.cutoff)?),
                        None => None,
                    },
                    None => None,
                };
                if let Some(p) = variant
                    && !(0.0..=1.0).contains(&p)
                {
                    return Err(format!(
                        "Candidate Signal {} gave {p} for Match {}",
                        signal.name(),
                        m.view.game.id
                    )
                    .into());
                }
                if let Some(f) = &fitted
                    && let Some(rp) = f.predict_rating(&m.view)?
                {
                    let rp = *rp.at(m.view.cutoff)?;
                    if !(0.0..=1.0).contains(&rp) {
                        return Err(format!(
                            "Candidate Signal {} gave rating {rp} for Match {}",
                            signal.name(),
                            m.view.game.id
                        )
                        .into());
                    }
                    v_computed += 1;
                    if let Some(valve) = m.view.signal()? {
                        v_both += 1;
                        v_diff += stats::log_loss(rp, m.team_a_won)
                            - stats::log_loss(valve.series, m.team_a_won);
                        vs_valve_paired.push((event.info.id, m.team_a_won, valve.series, rp));
                    }
                }
                computed += usize::from(variant.is_some());
                if let (Some(b), Some(v)) = (baseline, variant) {
                    both += 1;
                    diff += stats::log_loss(v, m.team_a_won) - stats::log_loss(b, m.team_a_won);
                    paired.push((event.info.id, m.team_a_won, b, v, m.view.market.is_some()));
                    in_subset.push(match &fitted {
                        Some(f) => f.in_subset(&m.view)?,
                        None => false,
                    });
                }
                rows.push(LabPrediction {
                    signal: signal.name().into(),
                    match_id: m.view.game.id,
                    event_id: event.info.id,
                    baseline,
                    variant,
                });
            }
            per_event.push(EventRow {
                event_id: event.info.id,
                title: event.info.title.clone(),
                training_matches: training.len(),
                computed,
                paired: both,
                log_loss_difference: (both > 0).then(|| diff / both as f64),
                parameters,
            });
            vs_valve_per_event.push(EventRow {
                event_id: event.info.id,
                title: event.info.title.clone(),
                training_matches: training.len(),
                computed: v_computed,
                paired: v_both,
                log_loss_difference: (v_both > 0).then(|| v_diff / v_both as f64),
                parameters: None,
            });
            fits.extend(fitted);
        }
        let vs_valve = if vs_valve_paired.is_empty() {
            None
        } else {
            let v_diffs: Vec<(i64, f64)> = vs_valve_paired
                .iter()
                .map(|r| (r.0, stats::log_loss(r.3, r.1) - stats::log_loss(r.2, r.1)))
                .collect();
            stats::event_bootstrap(
                &v_diffs,
                self.protocol.resamples,
                self.protocol.level,
                self.protocol.seed,
            )
            .map(|boot| {
                let v_metrics = |pick: fn(&(i64, bool, f64, f64)) -> f64| {
                    let n = vs_valve_paired.len() as f64;
                    let mean = |f: &dyn Fn(f64, bool) -> f64| {
                        vs_valve_paired.iter().map(|r| f(pick(r), r.1)).sum::<f64>() / n
                    };
                    Metrics {
                        log_loss: mean(&stats::log_loss),
                        brier: mean(&stats::brier),
                        accuracy: mean(&|p, w| f64::from(u8::from(stats::correct(p, w)))),
                    }
                };
                let (b, v) = (v_metrics(|r| r.2), v_metrics(|r| r.3));
                DirectComparison {
                    reference: "Valve Regional Standings (unblended)".into(),
                    paired: vs_valve_paired.len(),
                    events: vs_valve_paired
                        .iter()
                        .map(|r| r.0)
                        .collect::<std::collections::BTreeSet<_>>()
                        .len(),
                    difference: Metrics {
                        log_loss: v.log_loss - b.log_loss,
                        brier: v.brier - b.brier,
                        accuracy: v.accuracy - b.accuracy,
                    },
                    baseline: b,
                    variant: v,
                    interval: boot.interval,
                    std_error: boot.std_error,
                    detectable: stats::detectable_effect(boot.std_error),
                    per_event: vs_valve_per_event,
                }
            })
        };
        let computed_on = rows.iter().filter(|r| r.variant.is_some()).count();
        let mut report = CandidateReport {
            name: signal.name().into(),
            description: signal.description().into(),
            verdict: Verdict::NotTestable,
            added_after_list_fixed: signal.added_after_list_fixed(),
            reason: String::new(),
            computed_on,
            paired: paired.len(),
            events: paired
                .iter()
                .map(|p| p.0)
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            baseline: None,
            variant: None,
            difference: None,
            interval: None,
            accuracy_interval: None,
            std_error: None,
            detectable: None,
            market: None,
            subset: None,
            vs_valve,
            per_event,
        };
        if let Some(reason) = missing {
            report.reason = reason;
            return Ok((report, rows, fits));
        }
        if computed_on == 0 {
            report.reason = format!("computed on no Match: it needs {}", signal.needs());
            return Ok((report, rows, fits));
        }
        let metrics = |pick: fn(&(i64, bool, f64, f64, bool)) -> f64| {
            let n = paired.len() as f64;
            let mean = |f: &dyn Fn(f64, bool) -> f64| {
                paired.iter().map(|r| f(pick(r), r.1)).sum::<f64>() / n
            };
            Metrics {
                log_loss: mean(&stats::log_loss),
                brier: mean(&stats::brier),
                accuracy: mean(&|p, w| f64::from(u8::from(stats::correct(p, w)))),
            }
        };
        let diffs: Vec<(i64, f64)> = paired
            .iter()
            .map(|r| (r.0, stats::log_loss(r.3, r.1) - stats::log_loss(r.2, r.1)))
            .collect();
        let Some(boot) = stats::event_bootstrap(
            &diffs,
            self.protocol.resamples,
            self.protocol.level,
            self.protocol.seed,
        ) else {
            report.verdict = Verdict::NoEvidence;
            report.reason = "no Match was predicted by both the anchor and the variant".into();
            return Ok((report, rows, fits));
        };
        let (b, v) = (metrics(|r| r.2), metrics(|r| r.3));
        report.difference = Some(Metrics {
            log_loss: v.log_loss - b.log_loss,
            brier: v.brier - b.brier,
            accuracy: v.accuracy - b.accuracy,
        });
        report.baseline = Some(b);
        report.variant = Some(v);
        report.interval = Some(boot.interval);
        report.std_error = Some(boot.std_error);
        report.detectable = Some(stats::detectable_effect(boot.std_error));
        // Winners picked: the same paired Matches, resamples and seed.
        let accuracy = stats::accuracy_bootstrap(
            paired.iter().map(|r| (r.0, r.1, r.2, r.3)),
            &self.protocol,
        );
        report.accuracy_interval = accuracy;
        let market: Vec<f64> = paired
            .iter()
            .zip(&diffs)
            .filter(|(r, _)| r.4)
            .map(|(_, d)| d.1)
            .collect();
        report.market = (!market.is_empty()).then(|| MarketSubset {
            matches: market.len(),
            log_loss_difference: market.iter().sum::<f64>() / market.len() as f64,
        });
        report.subset = signal.subset_label().map(|label| {
            let inside: Vec<_> = paired.iter().zip(&in_subset).filter(|(_, s)| **s).collect();
            let acc = |p: fn(&(i64, bool, f64, f64, bool)) -> f64| {
                let hits = inside.iter().filter(|(r, _)| stats::correct(p(r), r.1)).count();
                if inside.is_empty() { 0.0 } else { hits as f64 / inside.len() as f64 }
            };
            SubsetReport {
                label: label.into(),
                matches: inside.len(),
                anchor_accuracy: acc(|r| r.2),
                variant_accuracy: acc(|r| r.3),
            }
        });
        let (verdict, reason) = stats::verdict(
            report.paired,
            boot.interval,
            accuracy,
            report.market.as_ref().map(|m| m.log_loss_difference),
            &self.protocol,
        );
        report.verdict = verdict;
        report.reason = reason;
        Ok((report, rows, fits))
    }
}
