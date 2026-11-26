//! Ticket #46 (spec #33): Pick'em variants compared in Pick'em Points.
//!
//! Every completed Stage of the archive (outside the Holdout) gets a Pick'em
//! from each variant, built only from what was known before the Stage's first
//! Match: the Liquipedia opening view saved from before it (teams, Round 1
//! pairings or the quarterfinals, seeds) and the pairwise ratings read through
//! the replay ports 1 s before it. The solver aborts with `TemporalLeakError`
//! on a view that holds a result or a later-round pairing, as in the backtest.
//!
//! Variants (all fed to the same seeded Monte-Carlo solver):
//! - `baseline`: the solver as it runs live: the market's line (inverted to a
//!   map probability) for the opening pairs, Valve's rating for every other pair.
//! - `calibrated`: the same pairs rated by the `rating_calibration` Candidate
//!   Signal, fitted walk-forward (Events that ended before the Stage's Event
//!   began). A pair it cannot rate keeps the baseline's rating.
//! - `learned`: likewise with `learned_weights`.
//! - `variance_seeking`: the baseline's ratings, but when the bot trails on
//!   the Pick'em Leaderboard (fewer public Pick'em Points than the best
//!   simulated Member, from the earlier Stages of the same Event) the picks the
//!   field is assumed to share (the baseline's own) are worth less
//!   (`Avoid`): it submits the likeliest Pick'em that differs from the field.
//!
//! The field is `members` simulated Members who submit the baseline's solver
//! on ratings disturbed by a normal noise on the logit scale (a seeded,
//! stated model, like #42's opponents). Pick'em Points are KallBet's
//! (`backtest::report::pickem_points`). The difference from the baseline is
//! the mean over Stages, with the Event bootstrap of the protocol.
use super::runner::Lab;
use super::signals::{FittedSignal, MatchView};
use super::stats::{self, Interval, Verdict};
use super::PROTOCOL_VERSION;
use crate::adapters::backtest::{ReplayStage, StageKind, report::pickem_points};
use crate::adapters::pickem::{Avoid, MonteCarloPickemSolver, invert_series_prob};
use crate::ports::{RandomSource, SeededRng};
use crate::*;
use chrono::Duration;
use serde::Serialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;

/// Fewer compared Stages than this: `no evidence` whatever the numbers.
pub const MIN_STAGES: usize = 10;

pub const VARIANTS: [&str; 4] = ["baseline", "calibrated", "learned", "variance_seeking"];

#[derive(Clone, Debug)]
pub struct PickemOptions {
    /// Swiss Monte-Carlo runs per Pick'em.
    pub simulations: usize,
    /// Simulated Members of the field.
    pub members: usize,
    /// Standard deviation of a Member's noise on the logit of a map probability.
    pub member_noise: f64,
    /// Share of a pick's expected points a shared pick forfeits when trailing.
    pub discount: f64,
}
impl Default for PickemOptions {
    fn default() -> Self {
        Self {
            simulations: 5_000,
            members: 6,
            member_noise: 0.5,
            discount: 0.5,
        }
    }
}

/// One Pick'em a variant submitted.
#[derive(Clone, Debug)]
pub struct SubmittedStagePickem {
    pub variant: String,
    pub stage_id: i64,
    pub payload: Value,
}

pub struct PickemOutcome {
    pub report: PickemReport,
    /// Every variant's Pick'em of every scored Stage (the Members' are not kept).
    pub submitted: Vec<SubmittedStagePickem>,
}

#[derive(Clone, Debug, Serialize)]
pub struct StageRow {
    pub event_id: i64,
    pub stage_id: i64,
    pub event: String,
    pub stage: String,
    pub kind: String,
    /// Picks the Stage has (the most Pick'em Points it can give).
    pub picks: usize,
    /// Pick'em Points by variant, in `VARIANTS` order (`None`: no Pick'em).
    pub points: Vec<Option<usize>>,
    /// The bot trailed on the Pick'em Leaderboard before this Stage (the
    /// variance-seeking variant's own standing against the best Member).
    pub trailing: bool,
    /// The best simulated Member's Pick'em Points on this Stage.
    pub best_member: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct VariantReport {
    pub name: String,
    pub description: String,
    pub verdict: Verdict,
    pub reason: String,
    /// Pick'em Points over the Stages it submitted.
    pub total_points: Option<usize>,
    /// Stages the variance of which was compared with the baseline's.
    pub stages_compared: usize,
    pub events: usize,
    /// Mean Pick'em Points per Stage minus the baseline's (positive: better).
    pub difference: Option<f64>,
    pub interval: Option<Interval>,
    pub std_error: Option<f64>,
    pub detectable: Option<f64>,
    /// Pairs the variant rated itself / pairs it left to the baseline's rating.
    pub rated_pairs: usize,
    pub fallback_pairs: usize,
    /// Mean place on the Pick'em Leaderboard at the end of each Event, among
    /// the simulated Members (ties share the better place).
    pub mean_place: Option<f64>,
    pub events_first: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct PickemReport {
    pub protocol_version: String,
    pub seed: u64,
    pub resamples: usize,
    pub level: f64,
    pub min_stages: usize,
    pub simulations: usize,
    pub members: usize,
    pub member_noise: f64,
    pub discount: f64,
    pub archive_digest: String,
    pub stages: Vec<StageRow>,
    /// Stages not scored, with the reason.
    pub excluded: Vec<String>,
    pub variants: Vec<VariantReport>,
}

/// The verdict rule of Pick'em Points: `interval` is that of the mean
/// difference in Points per Stage (variant − baseline).
pub fn verdict(stages: usize, interval: Interval, min_stages: usize) -> (Verdict, String) {
    if stages < min_stages {
        return (
            Verdict::NoEvidence,
            format!("compared on {stages} Stages, under the floor of {min_stages}"),
        );
    }
    if interval.low > 0.0 {
        return (
            Verdict::Promote,
            "the interval of the Pick'em Points gained excludes zero".into(),
        );
    }
    if interval.high < 0.0 {
        return (
            Verdict::Harmful,
            "the interval of the Pick'em Points difference is below zero: fewer than the baseline"
                .into(),
        );
    }
    (
        Verdict::NoEvidence,
        "the interval of the Pick'em Points difference includes zero".into(),
    )
}

/// Whether `payload` is a Pick'em the server would take for the Stage whose
/// opening view is `view`: Swiss two 3-0, two 0-3 and six to advance, all
/// different and from the Stage; Playoffs one pick for every slot of the
/// bracket, each a participant of its slot (a first slot's two teams,
/// otherwise the winners picked in its two feeder slots).
pub fn validate_payload(view: &Value, payload: &Value) -> std::result::Result<(), String> {
    match view["stage"]["kind"].as_str() {
        Some("swiss") => {
            let teams: BTreeSet<i64> = view["teams"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|t| t["id"].as_i64())
                .collect();
            let mut seen = BTreeSet::new();
            for (key, count) in [("three_nil", 2), ("zero_three", 2), ("advance", 6)] {
                let picked = payload[key].as_array().map_or(&[][..], Vec::as_slice);
                if picked.len() != count {
                    return Err(format!("{key} has {} teams, not {count}", picked.len()));
                }
                for id in picked {
                    let id = id.as_i64().ok_or("a team id is not a number")?;
                    if !teams.contains(&id) {
                        return Err(format!("team {id} is not in the Stage"));
                    }
                    if !seen.insert(id) {
                        return Err(format!("team {id} is picked twice"));
                    }
                }
            }
            Ok(())
        }
        Some("playoffs") => {
            let mut picked: BTreeMap<(u64, u64), i64> = BTreeMap::new();
            for p in payload["picks"].as_array().into_iter().flatten() {
                let key = (
                    p["rounds_before_final"].as_u64().ok_or("a pick has no round")?,
                    p["slot"].as_u64().ok_or("a pick has no slot")?,
                );
                let team = p["team_id"].as_i64().ok_or("a pick has no team")?;
                if picked.insert(key, team).is_some() {
                    return Err(format!("slot {key:?} is picked twice"));
                }
            }
            let slots: Vec<&Value> = view["bracket"]["slots"].as_array().into_iter().flatten().collect();
            if slots.len() != picked.len() {
                return Err(format!("{} picks for {} slots", picked.len(), slots.len()));
            }
            for s in slots {
                let key = (
                    s["rounds_before_final"].as_u64().ok_or("a slot has no round")?,
                    s["slot"].as_u64().ok_or("a slot has no slot")?,
                );
                let pick = picked.get(&key).ok_or(format!("slot {key:?} is not picked"))?;
                let participants: Vec<Option<i64>> = if s["leaf"].as_bool() == Some(true) {
                    vec![s["team_a"].as_i64(), s["team_b"].as_i64()]
                } else {
                    vec![
                        picked.get(&(key.0 + 1, 2 * key.1)).copied(),
                        picked.get(&(key.0 + 1, 2 * key.1 + 1)).copied(),
                    ]
                };
                if !participants.contains(&Some(*pick)) {
                    return Err(format!("team {pick} is not a participant of slot {key:?}"));
                }
            }
            Ok(())
        }
        other => Err(format!("no Pick'em for a {other:?} Stage")),
    }
}

fn mix(seed: u64, a: u64, b: u64) -> u64 {
    let mut x = seed ^ a.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ b.wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

fn normal(rng: &mut SeededRng) -> f64 {
    let u1 = rng.next_f64().max(1e-12);
    let u2 = rng.next_f64();
    (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
}

fn disturb(map: f64, sd: f64, rng: &mut SeededRng) -> f64 {
    let p = map.clamp(1e-6, 1.0 - 1e-6);
    let z = (p / (1.0 - p)).ln() + sd * normal(rng);
    1.0 / (1.0 + (-z).exp())
}

/// One pair of the Stage's teams (`first` id < `second` id) as a made-up or
/// opening Match, with its view at the Pick'em instant.
struct Pair {
    first: i64,
    second: i64,
    view: MatchView,
    /// The baseline's map probability for `first`, when it has one.
    baseline: Option<f64>,
}

impl Pair {
    /// A series probability for the view's team A as a map probability for `first`.
    fn map_for_first(&self, series_a: f64) -> f64 {
        let game = &self.view.game;
        let map_a = invert_series_prob(game.format, series_a);
        if game.team_a.id == self.first {
            map_a
        } else {
            1.0 - map_a
        }
    }
}

/// Every pair of the Stage's teams, viewed at `now`.
async fn pairs(
    lab: &Lab,
    stage: &ReplayStage,
    now: Instant,
    hidden: &BTreeSet<i64>,
) -> Result<Vec<Pair>> {
    let opening = |a: i64, b: i64| {
        stage
            .opening_matches
            .iter()
            .find(|m| {
                (m.team_a.id, m.team_b.id) == (a, b) || (m.team_a.id, m.team_b.id) == (b, a)
            })
    };
    let mut teams = stage.teams.clone();
    teams.sort_by_key(|t| t.id);
    let mut out = Vec::new();
    for (i, ta) in teams.iter().enumerate() {
        for tb in &teams[i + 1..] {
            let game = opening(ta.id, tb.id).cloned().unwrap_or_else(|| Match {
                event_id: stage.event_id,
                id: 0,
                team_a: ta.clone(),
                team_b: tb.clone(),
                format: Format::Bo1,
                scheduled_start: Some(stage.cutoff),
            });
            let context = lab.archive.match_context.stage_view(stage, now);
            let view = lab.view_of(&game, now, context, None, hidden).await?;
            let mut pair = Pair {
                first: ta.id,
                second: tb.id,
                view,
                baseline: None,
            };
            let from_market = pair
                .view
                .market()?
                .filter(|_| opening(ta.id, tb.id).is_some())
                .map(|q| pair.map_for_first(q));
            pair.baseline = match from_market {
                Some(map) => Some(map),
                None => pair.view.signal()?.map(|s| {
                    if pair.view.game.team_a.id == ta.id {
                        s.map
                    } else {
                        1.0 - s.map
                    }
                }),
            };
            out.push(pair);
        }
    }
    Ok(out)
}

fn ratings(
    pairs: &[Pair],
    maps: &[Option<f64>],
    now: Instant,
    cutoff: Instant,
) -> Result<Vec<AsOf<PairwiseProbability>>> {
    pairs
        .iter()
        .zip(maps)
        .filter_map(|(p, m)| {
            Some(AsOf::new(
                PairwiseProbability {
                    first: p.first,
                    second: p.second,
                    map: (*m)?,
                },
                now,
                cutoff,
            ))
        })
        .collect::<std::result::Result<_, _>>()
        .map_err(Into::into)
}

/// A fitted Candidate Signal for one Event, or why a variant is not testable.
enum Source {
    Fitted(Box<dyn FittedSignal>),
    Absent(String),
}

fn event_order(lab: &Lab) -> BTreeMap<i64, Vec<usize>> {
    let mut by_event: BTreeMap<i64, Vec<usize>> = BTreeMap::new();
    for (i, s) in lab.archive.stages.iter().enumerate() {
        by_event.entry(s.event_id).or_default().push(i);
    }
    for stages in by_event.values_mut() {
        stages.sort_by_key(|i| (lab.archive.stages[*i].cutoff, *i));
    }
    by_event
}

/// When the Stage's last result became known; `None` while any is unknown.
fn last_result(stage: &ReplayStage) -> Option<Instant> {
    stage
        .matches
        .iter()
        .map(|m| m.known_finish())
        .collect::<Option<Vec<_>>>()?
        .into_iter()
        .max()
}

#[derive(Default)]
struct Tally {
    rated: usize,
    fallback: usize,
}

/// Runs the comparison over the lab's archive.
pub async fn run(lab: &Lab, options: &PickemOptions) -> Result<PickemOutcome> {
    let seed = lab.protocol.seed;
    let split = lab.events().await?;
    // The Holdout's Stages are not scored, and their results stay out of
    // every view Pick'em reads; a Candidate Signal is told the archive holds
    // the training side only.
    let holdout_stages = lab.holdout_stages();
    let hidden = lab.holdout_matches(&holdout_stages);
    let training_archive = lab.archive.without_stages(&holdout_stages);
    let solver = MonteCarloPickemSolver::with_simulations(0, options.simulations);
    let candidates = [("calibrated", "rating_calibration"), ("learned", "learned_weights")];
    let by_event = event_order(lab);
    let mut rows: Vec<StageRow> = Vec::new();
    let mut excluded = Vec::new();
    let mut submitted = Vec::new();
    let mut tallies: [Tally; 4] = Default::default();
    // (event, variant) -> final Points, and the Members' final Points.
    let mut finals: BTreeMap<i64, ([usize; 4], Vec<usize>)> = BTreeMap::new();
    let mut not_testable: [Option<String>; 4] = Default::default();

    for (event_id, order) in &by_event {
        let Some(event) = split.training.iter().find(|e| e.info.id == *event_id) else {
            for i in order {
                let s = &lab.archive.stages[*i];
                excluded.push(format!(
                    "{}: in the Holdout, left unread until the selection is frozen",
                    s.title()
                ));
            }
            continue;
        };
        // The fits of this Event: only Matches of Events that ended before it began.
        let training = Lab::training(&split.training, event);
        let sources: Vec<Source> = candidates
            .iter()
            .map(|(_, name)| match lab.signals.iter().find(|s| s.name() == *name) {
                None => Ok(Source::Absent(format!("Candidate Signal {name} is not in this run"))),
                Some(signal) => match signal.missing_data(&training_archive) {
                    Some(reason) => Ok(Source::Absent(reason)),
                    None => signal.fit(&training).map(Source::Fitted),
                },
            })
            .collect::<Result<_>>()?;
        for (k, source) in sources.iter().enumerate() {
            if let Source::Absent(reason) = source {
                not_testable[k + 1].get_or_insert_with(|| reason.clone());
            }
        }

        // Pick'em Points the bot (the variance-seeking variant) and the Members
        // have on the Pick'em Leaderboard from Stages that ended before a Stage.
        let mut done: Vec<(Instant, usize, Vec<usize>)> = Vec::new();
        let mut totals = [0_usize; 4];
        let mut member_totals = vec![0_usize; options.members];
        for &i in order {
            let stage = &lab.archive.stages[i];
            let reason = if holdout_stages.contains(&stage.id) {
                // A later Stage of an Event whose earlier Stage is on the
                // training side (Singapore Stage 2 and on).
                Some("in the Holdout, left unread until the selection is frozen".to_string())
            } else if !stage.complete {
                Some("the Stage is not complete".to_string())
            } else if let Some(problem) = &stage.opening_problem {
                Some(format!("no opening view: {problem}"))
            } else if stage.opening.is_none() {
                Some("no opening view".to_string())
            } else if !matches!(stage.kind, StageKind::Swiss | StageKind::Playoffs) {
                Some("no Pick'em model for this Stage format".to_string())
            } else {
                None
            };
            if let Some(reason) = reason {
                excluded.push(format!("{}: {reason}", stage.title()));
                continue;
            }
            let opening = stage.opening.as_ref().expect("checked");
            let now = stage.cutoff - Duration::seconds(1);
            let mut view = opening.value.clone();
            view["stage"]["ordinal"] = (i as u64).into();
            let stage_view = AsOf::new(view, opening.observed_at, stage.cutoff)?;
            let pairs = pairs(lab, stage, now, &hidden).await?;
            let baseline_maps: Vec<Option<f64>> = pairs.iter().map(|p| p.baseline).collect();
            let stage_seed = mix(seed, stage.id as u64, 0);
            let solve = |maps: &[Option<f64>], avoid: Option<Avoid<'_>>, rng_seed: u64| -> Result<Option<Value>> {
                let rated = ratings(&pairs, maps, now, stage.cutoff)?;
                solver.solve_avoiding(
                    &stage_view,
                    &rated,
                    stage.cutoff,
                    &mut SeededRng::new(rng_seed),
                    avoid,
                )
            };
            let Some(base) = solve(&baseline_maps, None, stage_seed)? else {
                excluded.push(format!("{}: the solver gave no Pick'em", stage.title()));
                continue;
            };
            let mut payloads: Vec<Option<Value>> = vec![Some(base.clone()), None, None, None];
            for (k, source) in sources.iter().enumerate() {
                let Source::Fitted(fit) = source else { continue };
                let mut maps = Vec::with_capacity(pairs.len());
                for p in &pairs {
                    let predicted = match fit.predict(&p.view)? {
                        Some(series) => Some(p.map_for_first(*series.at(now)?)),
                        None => None,
                    };
                    if predicted.is_some() {
                        tallies[k + 1].rated += 1;
                    } else if p.baseline.is_some() {
                        tallies[k + 1].fallback += 1;
                    }
                    maps.push(predicted.or(p.baseline));
                }
                payloads[k + 1] = solve(&maps, None, stage_seed)?;
            }
            // The field: the baseline solver on ratings each Member disturbs.
            let mut member_points = Vec::new();
            for m in 0..options.members {
                let mut rng = SeededRng::new(mix(seed, stage.id as u64, m as u64 + 1));
                let maps: Vec<Option<f64>> = baseline_maps
                    .iter()
                    .map(|p| p.map(|p| disturb(p, options.member_noise, &mut rng)))
                    .collect();
                let payload = solve(&maps, None, mix(stage_seed, m as u64 + 1, 7))?
                    .ok_or("the solver gave a Member no Pick'em")?;
                member_points.push(pickem_points(stage, &payload).0);
            }
            // Public standing before this Stage: Stages whose last result is out.
            let public = |points: &dyn Fn(&(Instant, usize, Vec<usize>)) -> usize| {
                done.iter()
                    .filter(|(ended, _, _)| *ended < stage.cutoff)
                    .map(points)
                    .sum::<usize>()
            };
            let bot = public(&|d| d.1);
            let best = (0..options.members)
                .map(|m| public(&|d| d.2[m]))
                .max();
            let trailing = best.is_some_and(|best| best > bot);
            payloads[3] = if trailing {
                let avoid = Avoid {
                    picks: &base,
                    discount: options.discount,
                };
                solve(&baseline_maps, Some(avoid), stage_seed)?
            } else {
                Some(base.clone())
            };

            let mut points = Vec::new();
            let mut picks = 0;
            for (k, payload) in payloads.iter().enumerate() {
                match payload {
                    Some(payload) => {
                        validate_payload(&stage_view.at(stage.cutoff)?.clone(), payload)
                            .map_err(|e| format!("{} {}: invalid Pick'em: {e}", VARIANTS[k], stage.title()))?;
                        let (p, _, n) = pickem_points(stage, payload);
                        picks = n;
                        points.push(Some(p));
                        totals[k] += p;
                        submitted.push(SubmittedStagePickem {
                            variant: VARIANTS[k].into(),
                            stage_id: stage.id,
                            payload: payload.clone(),
                        });
                    }
                    None => points.push(None),
                }
            }
            for (m, p) in member_points.iter().enumerate() {
                member_totals[m] += p;
            }
            if let Some(ended) = last_result(stage) {
                done.push((ended, points[3].unwrap_or(0), member_points.clone()));
            }
            rows.push(StageRow {
                event_id: *event_id,
                stage_id: stage.id,
                event: stage.event.clone(),
                stage: stage.name.clone(),
                kind: format!("{:?}", stage.kind).to_lowercase(),
                picks,
                points,
                trailing,
                best_member: member_points.iter().copied().max().unwrap_or(0),
            });
        }
        finals.insert(*event_id, (totals, member_totals));
    }
    let begins = |id: i64| lab.archive.stages.iter().find(|s| s.id == id).map(|s| s.cutoff);
    rows.sort_by_key(|r| (r.event_id, begins(r.stage_id), r.stage_id));
    let report = build(lab, options, rows, excluded, tallies, not_testable, &finals);
    Ok(PickemOutcome { report, submitted })
}

fn build(
    lab: &Lab,
    options: &PickemOptions,
    rows: Vec<StageRow>,
    excluded: Vec<String>,
    tallies: [Tally; 4],
    not_testable: [Option<String>; 4],
    finals: &BTreeMap<i64, ([usize; 4], Vec<usize>)>,
) -> PickemReport {
    let protocol = &lab.protocol;
    let describe = [
        "the solver as it runs live: market line for the opening pairs, Valve's rating otherwise",
        "the solver fed by `rating_calibration`, fitted walk-forward",
        "the solver fed by `learned_weights`, fitted walk-forward",
        "the baseline, but when trailing on the Pick'em Leaderboard the picks the field shares are worth less",
    ];
    let variants = (0..4)
        .map(|k| {
            let mut v = VariantReport {
                name: VARIANTS[k].into(),
                description: describe[k].into(),
                verdict: Verdict::NoEvidence,
                reason: String::new(),
                total_points: None,
                stages_compared: 0,
                events: 0,
                difference: None,
                interval: None,
                std_error: None,
                detectable: None,
                rated_pairs: tallies[k].rated,
                fallback_pairs: tallies[k].fallback,
                mean_place: None,
                events_first: 0,
            };
            let mine: Vec<usize> = rows.iter().filter_map(|r| r.points[k]).collect();
            if mine.is_empty() {
                v.verdict = Verdict::NotTestable;
                v.reason = not_testable[k]
                    .clone()
                    .unwrap_or_else(|| "no Stage was scored".into());
                return v;
            }
            v.total_points = Some(mine.iter().sum());
            let places: Vec<usize> = finals
                .values()
                .filter(|(_, members)| !members.is_empty())
                .map(|(totals, members)| 1 + members.iter().filter(|m| **m > totals[k]).count())
                .collect();
            v.events_first = places.iter().filter(|p| **p == 1).count();
            v.mean_place = (!places.is_empty())
                .then(|| places.iter().sum::<usize>() as f64 / places.len() as f64);
            if k == 0 {
                v.stages_compared = mine.len();
                v.verdict = Verdict::NoEvidence;
                v.reason = "the reference".into();
                return v;
            }
            let diffs: Vec<(i64, f64)> = rows
                .iter()
                .filter_map(|r| Some((r.event_id, r.points[k]? as f64 - r.points[0]? as f64)))
                .collect();
            v.stages_compared = diffs.len();
            v.events = diffs.iter().map(|d| d.0).collect::<BTreeSet<_>>().len();
            let Some(boot) =
                stats::event_bootstrap(&diffs, protocol.resamples, protocol.level, protocol.seed)
            else {
                v.reason = "no Stage was scored by both".into();
                return v;
            };
            v.difference = Some(boot.mean);
            v.interval = Some(boot.interval);
            v.std_error = Some(boot.std_error);
            v.detectable = Some(stats::detectable_effect(boot.std_error));
            (v.verdict, v.reason) = verdict(diffs.len(), boot.interval, MIN_STAGES);
            v
        })
        .collect();
    PickemReport {
        protocol_version: PROTOCOL_VERSION.into(),
        seed: protocol.seed,
        resamples: protocol.resamples,
        level: protocol.level,
        min_stages: MIN_STAGES,
        simulations: options.simulations,
        members: options.members,
        member_noise: options.member_noise,
        discount: options.discount,
        archive_digest: lab.archive.digest.clone(),
        stages: rows,
        excluded,
        variants,
    }
}

impl PickemReport {
    pub fn markdown(&self) -> String {
        let mut out = String::new();
        let w = &mut out;
        let _ = writeln!(
            w,
            "# Pick'em variants in Pick'em Points\n\nProtocol `{}`, archive SHA-256 `{}`, seed {}, {} Event resamples, {:.0}% intervals. \
             Each Stage's Pick'em is made 1 s before its first Match from the Liquipedia opening view saved from before it and the ratings \
             the replay ports gave before that instant; the solver refuses a view that holds a result or a later-round pairing. \
             Pick'em Points are KallBet's (Swiss 1 per correct 3-0, 0-3 and advance pick; Playoffs 1 per correct slot). \
             The difference is the mean Points per Stage minus the baseline's, with the Event bootstrap; \
             under {} Stages the verdict is `no evidence`. Stages of the Holdout are not read.\n\n\
             The field is {} simulated Members who submit the baseline's solver on ratings disturbed by a normal noise of {} on the logit; \
             the bot trails when its public Pick'em Points (Stages that ended before) are below the best Member's; the variance-seeking \
             variant then discounts the field's shared picks by {:.0}%. {} simulations per Swiss Pick'em.",
            self.protocol_version,
            self.archive_digest,
            self.seed,
            self.resamples,
            self.level * 100.0,
            self.min_stages,
            self.members,
            self.member_noise,
            self.discount * 100.0,
            self.simulations
        );
        let _ = writeln!(
            w,
            "\n## Variants against the baseline\n\n| Variant | Total Points | Stages | Difference per Stage | {:.0}% interval | Detectable | Verdict | Mean place | Events first |\n|---|---:|---:|---:|---|---:|---|---:|---:|",
            self.level * 100.0
        );
        for v in &self.variants {
            let num = |x: Option<f64>, f: &dyn Fn(f64) -> String| x.map_or("—".into(), f);
            let _ = writeln!(
                w,
                "| `{}` | {} | {} | {} | {} | {} | **{}**: {} | {} | {} |",
                v.name,
                v.total_points.map_or("—".into(), |t| t.to_string()),
                v.stages_compared,
                num(v.difference, &|d| format!("{d:+.3}")),
                v.interval
                    .map_or("—".into(), |i| format!("[{:+.3}, {:+.3}]", i.low, i.high)),
                num(v.detectable, &|d| format!("{d:.3}")),
                v.verdict.as_str(),
                v.reason,
                num(v.mean_place, &|d| format!("{d:.2}")),
                v.events_first,
            );
        }
        for v in &self.variants[1..3] {
            let _ = writeln!(
                w,
                "\n`{}` rated {} pairs itself and left {} to the baseline's rating.",
                v.name, v.rated_pairs, v.fallback_pairs
            );
        }
        let _ = writeln!(
            w,
            "\n## Pick'em Points per Stage\n\n| Event | Stage | Picks | baseline | calibrated | learned | variance_seeking | trailing | best Member |\n|---|---|---:|---:|---:|---:|---:|---|---:|"
        );
        for r in &self.stages {
            let cell = |p: Option<usize>| p.map_or("—".into(), |p| p.to_string());
            let _ = writeln!(
                w,
                "| {} | {} | {} | {} | {} | {} | {} | {} | {} |",
                r.event,
                r.stage,
                r.picks,
                cell(r.points[0]),
                cell(r.points[1]),
                cell(r.points[2]),
                cell(r.points[3]),
                if r.trailing { "yes" } else { "no" },
                r.best_member
            );
        }
        if !self.excluded.is_empty() {
            let _ = writeln!(w, "\n## Stages not scored\n");
            for e in &self.excluded {
                let _ = writeln!(w, "- {e}");
            }
        }
        out
    }
}
