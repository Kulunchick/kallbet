//! The retrospective runner: the live pipeline (`Sidecar::tick`, the policies,
//! the debate, the Pick'em step) driven over the replay, one virtual instant at
//! a time, with the LLM runtimes built in `Mode::Backtest`.
use super::anonymizer::Blind;
use super::archive::{ReplayArchive, ReplayStage};
use super::feeds::{MarketSource, ReplayContext, ReplayOdds, ReplaySignal};
use super::platform::{Due, ReplayClock, ReplayPlatform, Submitted};
use super::report::{self, LlmTrack, PickemRow, Report, StageRow};
use crate::adapters::{
    baseline::Noop,
    debate::{self, DebateConfig},
    llm::{BreakerState, LlmConfig, Mode, ProcessRunner, Providers},
    pickem::{self, DEFAULT_SEED, MonteCarloPickemSolver},
};
use crate::*;
use chrono::Duration;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

/// The profiles of the benchmark, by bot key.
pub const VRS: &str = "vrs";
pub const MARKET: &str = "market";
pub const ANCHOR: &str = "anchor";
pub const SINGLE: &str = "single";
pub const DEBATE: &str = "debate";
pub const PROFILES: [(&str, &str); 5] = [
    (VRS, "VrsOnly"),
    (MARKET, "MarketOnly"),
    (ANCHOR, "HybridAnchored without LLM (the anchor)"),
    (
        SINGLE,
        "Single LLM agent (HybridAnchored, debate mode `never`)",
    ),
    (
        DEBATE,
        "3-agent AdversarialDebateLlm (HybridAnchored, debate mode `always`)",
    ),
];
const LLM_BOTS: [&str; 2] = [SINGLE, DEBATE];
/// Ten years: in a backtest a provider's breaker never closes again.
const NEVER_AGAIN_MINUTES: i64 = 60 * 24 * 3650;

/// Each Match is evaluated this long before its cutoff: the scheduler skips a
/// Match whose Scheduled Start is 15 minutes or less away, and every datum must
/// be observed strictly before the evaluation instant.
pub fn evaluation_lead() -> Duration {
    Duration::seconds(1)
}

/// The external world of the replay. Tests replace a feed to inject a datum.
pub struct Feeds {
    pub signal: Arc<dyn SignalEstimator>,
    pub odds: Arc<dyn OddsSource>,
    pub context: Arc<dyn ContextEnricher>,
}
impl Feeds {
    pub fn replay(archive: &ReplayArchive) -> Result<Self> {
        Ok(Self {
            signal: Arc::new(ReplaySignal::new(&archive.standings)?),
            odds: Arc::new(ReplayOdds::new(archive)),
            context: Arc::new(ReplayContext::new(archive)),
        })
    }
}

/// Which Matches the LLM profiles evaluate (the budget) and who sits where.
#[derive(Clone, Default)]
pub struct LlmPlan {
    /// Matches sent with real names (after the models' training cutoff).
    pub direct: BTreeSet<i64>,
    /// Matches sent through the `BlindAnonymizer`.
    pub blind: BTreeSet<i64>,
    /// The single agent's provider chain.
    pub single: Vec<String>,
    pub advocate: Vec<String>,
    pub devil: Vec<String>,
    pub arbiter: Vec<String>,
}
impl LlmPlan {
    pub fn matches(&self) -> BTreeSet<i64> {
        self.direct.union(&self.blind).copied().collect()
    }
    fn debate_config(&self) -> Result<DebateConfig> {
        Ok(serde_json::from_value(json!({"bots": {
            SINGLE: {"mode": "never", "fast": self.single},
            DEBATE: {"mode": "always", "advocate": self.advocate,
                     "devil": self.devil, "arbiter": self.arbiter},
        }}))?)
    }
}

pub struct Backtest {
    pub archive: ReplayArchive,
    pub feeds: Feeds,
    /// The `llm` section: providers and, ignored here, `web_search_bots`.
    pub llm: LlmConfig,
    pub plan: LlmPlan,
    pub runner: Arc<dyn ProcessRunner>,
    /// Swiss Monte-Carlo runs per Pick'em (the solver's default is 25,000).
    pub simulations: usize,
    /// The saved market histories, for the per-source comparison.
    pub markets: Option<Arc<ReplayOdds>>,
}

/// What one run produced besides the report.
pub struct Outcome {
    pub report: Report,
    pub predictions: Vec<Submitted>,
}

fn bot(key: &str, policy: Policy, chain: Vec<String>) -> BotProfile {
    BotProfile {
        bot_key: key.into(),
        nickname: key.into(),
        language: None,
        avatar_url: None,
        invite_links: vec![],
        policy,
        provider_chain: chain,
    }
}

enum Step {
    Stage(usize),
    Matches(Vec<Match>),
}

impl Backtest {
    pub fn new(archive: ReplayArchive, runner: Arc<dyn ProcessRunner>) -> Result<Self> {
        let markets = Arc::new(ReplayOdds::new(&archive));
        let mut feeds = Feeds::replay(&archive)?;
        feeds.odds = markets.clone();
        Ok(Self {
            markets: Some(markets),
            feeds,
            archive,
            llm: LlmConfig::default(),
            plan: LlmPlan::default(),
            runner,
            simulations: pickem::DEFAULT_SIMULATIONS,
        })
    }

    /// Runs the replay. `audit` must be a fresh audit file (it may hold LLM
    /// answers imported with `import_answers`). Any datum observed at or after
    /// its cutoff aborts the run with `TemporalLeakError`; an LLM quota or
    /// authentication failure stops the LLM profiles and the rest goes on.
    pub async fn run(&self, audit: &Audit) -> Result<Outcome> {
        let stages = &self.archive.stages;
        let mut steps: Vec<(Instant, Step)> = Vec::new();
        for (i, stage) in stages.iter().enumerate() {
            steps.push((stage.cutoff - evaluation_lead(), Step::Stage(i)));
            let mut by_instant: BTreeMap<Instant, Vec<Match>> = BTreeMap::new();
            for m in &stage.matches {
                by_instant
                    .entry(m.cutoff() - evaluation_lead())
                    .or_default()
                    .push(m.game.clone());
            }
            steps.extend(by_instant.into_iter().map(|(t, g)| (t, Step::Matches(g))));
        }
        steps.sort_by_key(|(t, s)| (*t, matches!(s, Step::Matches(_))));
        let start = steps.first().map(|s| s.0).unwrap_or_default();
        let clock = Arc::new(ReplayClock::new(start));

        // A provider whose breaker opens (quota, timeout, transient failure) is
        // not tried again in this run: the replay clock moves days between
        // Matches, so an ordinary cool-down would retry an exhausted quota.
        let mut llm_config = self.llm.clone();
        llm_config.cooldown_minutes = NEVER_AGAIN_MINUTES;
        let providers = Arc::new(Providers::new(
            &llm_config,
            Mode::Backtest,
            clock.clone(),
            self.runner.clone(),
        ));
        let debate = debate::build(&self.plan.debate_config()?, providers.clone(), audit)?;
        let llm: Arc<dyn LlmRuntime> = Arc::new(Required(Arc::new(Blind::new(
            debate,
            self.plan.blind.clone(),
        ))));
        let scope: BTreeMap<String, BTreeSet<i64>> = LLM_BOTS
            .iter()
            .map(|b| ((*b).to_owned(), self.plan.matches()))
            .collect();
        let platform = Arc::new(ReplayPlatform::new(stages.clone(), scope));
        let seats: Vec<String> = [&self.plan.advocate, &self.plan.devil, &self.plan.arbiter]
            .into_iter()
            .flatten()
            .cloned()
            .collect();
        let solver = Arc::new(MonteCarloPickemSolver::with_simulations(
            DEFAULT_SEED,
            self.simulations,
        ));
        let pickem_step = pickem::with_solver(
            clock.clone(),
            self.feeds.signal.clone(),
            self.feeds.odds.clone(),
            solver,
        );
        let lineup = [
            (bot(VRS, Policy::VrsOnly, vec![]), false, true),
            (bot(MARKET, Policy::MarketOnly, vec![]), false, false),
            (bot(ANCHOR, Policy::HybridAnchored, vec![]), false, false),
            (
                bot(SINGLE, Policy::HybridAnchored, self.plan.single.clone()),
                true,
                false,
            ),
            (bot(DEBATE, Policy::HybridAnchored, seats), true, false),
        ];
        let mut sidecars: Vec<Sidecar> = lineup
            .into_iter()
            .map(|(profile, uses_llm, solves_pickem)| {
                let ports = Ports {
                    platform: platform.clone(),
                    clock: clock.clone(),
                    signal: self.feeds.signal.clone(),
                    odds: self.feeds.odds.clone(),
                    context: self.feeds.context.clone(),
                    llm: if uses_llm {
                        llm.clone()
                    } else {
                        Arc::new(Noop)
                    },
                    policy: Arc::new(crate::registry::ConfiguredPolicy),
                    pickem: if solves_pickem {
                        pickem_step.clone()
                    } else {
                        Arc::new(Noop)
                    },
                };
                let audit = Audit {
                    db: audit.db.clone(),
                };
                // The replay reads every failure from the tick: an LLM profile
                // without an answer must not submit the anchor's pick.
                Sidecar::new(vec![profile], audit, ports).on_failure(OnFailure::Abort)
            })
            .collect();

        let imported = audit.debate_turns().await?.len();
        let imported_id: i64 = sqlx::query_scalar("SELECT COALESCE(MAX(id), 0) FROM debate_turns")
            .fetch_one(&audit.db)
            .await?;
        let mut track = LlmTrack {
            providers: self.llm.providers.keys().cloned().collect(),
            ..LlmTrack::default()
        };
        for b in LLM_BOTS {
            track.planned.insert(b.into(), self.plan.matches().len());
        }
        let mut halted = false;
        for (at, step) in steps {
            clock.set(at);
            let games = match &step {
                Step::Stage(i) => {
                    platform.show(Due::Stage(*i));
                    vec![]
                }
                Step::Matches(games) => {
                    platform.show(Due::Matches(games.clone()));
                    games.iter().map(|g| g.id).collect()
                }
            };
            for sidecar in &mut sidecars {
                let key = sidecar.bots[0].bot_key.clone();
                let llm_bot = LLM_BOTS.contains(&key.as_str());
                if llm_bot && (halted || !games.iter().any(|g| self.plan.matches().contains(g))) {
                    continue;
                }
                let Err(e) = sidecar.tick().await else {
                    continue;
                };
                if let Some(leak) = e.downcast_ref::<TemporalLeakError>() {
                    return Err(Box::new(TemporalLeakError {
                        observed_at: leak.observed_at,
                        cutoff: leak.cutoff,
                    }));
                }
                match e.downcast_ref::<LlmError>() {
                    Some(kind) if llm_bot => {
                        *track
                            .failures
                            .entry(format!("`{key}`: {kind}"))
                            .or_default() += games.len();
                        if matches!(kind, LlmError::QuotaExceeded | LlmError::Auth) && !halted {
                            halted = true;
                            track.stopped = Some(format!(
                                "first {kind} at {at}; no further LLM call was made"
                            ));
                        }
                    }
                    _ => return Err(format!("{key} at {at}: {e}").into()),
                }
            }
            // A seat whose provider ran out of quota fails quietly (its brief is
            // null and the Arbiter still decides); that, too, ends the LLM track.
            if !halted {
                let exhausted: Option<(String, String)> = sqlx::query_as(
                    "SELECT provider, role FROM debate_turns
                     WHERE id > ? AND status IN ('quota_exceeded', 'auth') ORDER BY id LIMIT 1",
                )
                .bind(imported_id)
                .fetch_optional(&audit.db)
                .await?;
                if let Some((provider, role)) = exhausted {
                    halted = true;
                    track.stopped = Some(format!(
                        "first quota or sign-in failure ({provider}, seat {role}) at {at}; no further LLM call was made"
                    ));
                }
            }
            // One that a fallback provider absorbed is seen on the breaker: that
            // provider is not asked again (its cool-down outlasts the run) and the
            // rest of its chain answers, as in a live run. Recorded, not stopped.
            for name in self.llm.providers.keys() {
                if let Some(BreakerState::Open { cause, .. }) = providers.breaker(name) {
                    track
                        .exhausted
                        .entry(name.clone())
                        .or_insert_with(|| format!("{cause} at {at}; not asked again"));
                }
            }
        }
        platform.show(Due::Nothing);

        let predictions = platform.predictions();
        let turns = audit.debate_turns().await?;
        for t in turns.iter().skip(imported) {
            *track.calls.entry(t.bot.clone()).or_default() += 1;
        }
        for t in turns.iter().filter(|t| t.status == "ok") {
            *track
                .answered_by
                .entry(t.bot.clone())
                .or_default()
                .entry(format!("{} by {}", t.role, t.provider))
                .or_default() += 1;
        }
        for b in LLM_BOTS {
            let answered = predictions.iter().filter(|p| p.bot == b).count();
            track.answered.insert(b.into(), answered);
        }
        let report = self
            .report(audit, &predictions, &platform.pickems(), track)
            .await?;
        Ok(Outcome {
            report,
            predictions,
        })
    }

    async fn report(
        &self,
        audit: &Audit,
        predictions: &[Submitted],
        pickems: &[super::platform::SubmittedPickem],
        llm: LlmTrack,
    ) -> Result<Report> {
        let stages = &self.archive.stages;
        // What each Match's dossier held, read from the anchor's audited dossiers.
        let dossiers: Vec<(i64, String)> =
            sqlx::query_as("SELECT match_id, dossier FROM dossier_runs WHERE bot = ?")
                .bind(ANCHOR)
                .fetch_all(&audit.db)
                .await?;
        let held: BTreeMap<i64, Value> = dossiers
            .into_iter()
            .filter_map(|(id, d)| Some((id, serde_json::from_str(&d).ok()?)))
            .collect();
        let count = |stage: &ReplayStage, f: &dyn Fn(&Value) -> bool| {
            stage
                .matches
                .iter()
                .filter(|m| held.get(&m.game.id).is_some_and(f))
                .count()
        };
        let stage_rows = stages
            .iter()
            .map(|s| StageRow {
                stage: s.title(),
                kind: s.kind,
                complete: s.complete,
                scored_matches: s.matches.len(),
                unscored_matches: s.unscored,
                with_vrs: count(s, &|d| !d["signal"].is_null()),
                with_market: count(s, &|d| !d["market"].is_null()),
                with_rosters: count(s, &|d| {
                    d["context"].as_array().is_some_and(|c| !c.is_empty())
                }),
                opening_view: s.opening_problem.clone(),
            })
            .collect();

        let all: BTreeSet<i64> = stages
            .iter()
            .flat_map(|s| s.matches.iter().map(|m| m.game.id))
            .collect();
        let baselines = &PROFILES[..3];
        let mut tables = vec![
            report::table(
                "Non-LLM track: every completed Match",
                "Each profile on every Match it predicted. VrsOnly needs both teams in a Valve snapshot before the cutoff, MarketOnly a pre-match market line (OddsPapi, else Polymarket, else Kalshi; without one it falls back to VRS, so its rows include those Matches), the anchor either.",
                stages,
                &all,
                predictions,
                baselines,
                false,
            ),
            report::table(
                "Non-LLM track: Matches with both a market line and a rating",
                "The same profiles, only on the Matches where every one of them predicted and the dossier held both a market line and a VRS rating.",
                stages,
                &all.iter()
                    .filter(|id| {
                        held.get(id)
                            .is_some_and(|d| !d["market"].is_null() && !d["signal"].is_null())
                    })
                    .copied()
                    .collect(),
                predictions,
                baselines,
                true,
            ),
        ];
        if let Some(markets) = &self.markets
            && !self.plan.direct.is_empty()
        {
            let mut quotes = Vec::new();
            for m in stages.iter().flat_map(|s| &s.matches) {
                if !self.plan.direct.contains(&m.game.id) {
                    continue;
                }
                for source in MarketSource::ALL {
                    let at = m.cutoff() - evaluation_lead();
                    if let Some(q) = markets.quote_from(source, &m.game, at)? {
                        let p = *q.at(at)?;
                        let (team, p_win) = if p >= 0.5 {
                            (m.game.team_a.id, p)
                        } else {
                            (m.game.team_b.id, 1.0 - p)
                        };
                        quotes.push(Submitted {
                            bot: source.name().into(),
                            match_id: m.game.id,
                            prediction: Prediction {
                                team_id: team,
                                rationale: json!({"p_win": p_win, "override_market": false}),
                            },
                        });
                    }
                }
            }
            let sources: Vec<(&str, &str)> = MarketSource::ALL
                .iter()
                .map(|s| (s.name(), s.name()))
                .collect();
            tables.push(report::table(
                "Market sources on the post-cutoff Matches",
                "The favourite of each source's last pre-cutoff price (Pinnacle de-vigged; Polymarket's last trade price; Kalshi's two bid/ask mids normalised), on the Matches every source priced. Not a profile: no policy, no fallback.",
                stages,
                &self.plan.direct,
                &quotes,
                &sources,
                true,
            ));
        }
        if !self.plan.direct.is_empty() {
            tables.push(report::table(
                "LLM track: post-cutoff Matches, real names",
                "All five profiles, only on the Matches every one of them predicted.",
                stages,
                &self.plan.direct,
                predictions,
                &PROFILES,
                true,
            ));
        }
        if !self.plan.blind.is_empty() {
            let (rosters, news) = self.blind_context(stages).await?;
            let n = self.plan.blind.len();
            let note = format!(
                "All five profiles, only on the Matches every one of them predicted. The LLM seats saw TEAM_ALPHA/TEAM_BETA, {}, {}, no Event title and no dates.",
                match rosters {
                    0 => "no roster (no player names to mask)".to_string(),
                    r if r == n => format!("PLAYER_01..10 for the rosters of all {n} Matches"),
                    r => format!("PLAYER_01..10 for the rosters of {r} of {n} Matches"),
                },
                match news {
                    0 => "no news item".to_string(),
                    k => format!("news items on {k} of {n} Matches"),
                },
            );
            tables.push(report::table(
                "LLM track: anonymized historical slice (BlindAnonymizer)",
                &note,
                stages,
                &self.plan.blind,
                predictions,
                &PROFILES,
                true,
            ));
        }

        let mut pickem = Vec::new();
        for s in stages {
            let sub = pickems.iter().find(|p| p.stage_id == s.id);
            let rated = self.rated_pairs(s).await?;
            pickem.push(match sub {
                    None => PickemRow {
                        stage: s.title(),
                        kind: s.kind,
                        status: s
                            .opening_problem
                            .clone()
                            .unwrap_or_else(|| "no Pick'em submitted".into()),
                        points: None,
                        settled_picks: 0,
                        picks: 0,
                        rated_pairs: rated,
                    },
                    Some(p) => {
                        let (points, settled, picks) = report::pickem_points(s, &p.payload);
                        PickemRow {
                            stage: s.title(),
                            kind: s.kind,
                            status: if s.complete {
                                "final".into()
                            } else {
                                format!("pending: Stage not finished ({points} Points on {settled} decided picks so far)")
                            },
                            points: s.complete.then_some(points),
                            settled_picks: settled,
                            picks,
                            rated_pairs: rated,
                        }
                    }
            });
        }

        let anchors: BTreeMap<i64, &Submitted> = predictions
            .iter()
            .filter(|p| p.bot == ANCHOR)
            .map(|p| (p.match_id, p))
            .collect();
        let mut llm_moves = Vec::new();
        for p in predictions
            .iter()
            .filter(|p| LLM_BOTS.contains(&p.bot.as_str()))
        {
            let (Some(anchor), Some(m)) = (
                anchors.get(&p.match_id),
                stages
                    .iter()
                    .flat_map(|s| &s.matches)
                    .find(|m| m.game.id == p.match_id),
            ) else {
                continue;
            };
            let p_a = |s: &Submitted| {
                let w = s.prediction.rationale["p_win"].as_f64().unwrap_or(0.5);
                if s.prediction.team_id == m.game.team_a.id {
                    w
                } else {
                    1.0 - w
                }
            };
            if (p_a(anchor) - p_a(p)).abs() > 1e-9 {
                let name = |id: i64| {
                    if id == m.game.team_a.id {
                        m.game.team_a.name.clone()
                    } else {
                        m.game.team_b.name.clone()
                    }
                };
                let format = serde_json::to_value(m.game.format)?;
                llm_moves.push(report::LlmMove {
                    profile: p.bot.clone(),
                    game: format!(
                        "{} vs {} ({})",
                        m.game.team_a.name,
                        m.game.team_b.name,
                        format.as_str().unwrap_or("")
                    ),
                    anchor_team_a: p_a(anchor),
                    llm_team_a: p_a(p),
                    pick: name(p.prediction.team_id),
                    winner: name(m.winner),
                    factors: p.prediction.rationale["factors"].clone(),
                });
            }
        }
        let mut notes = vec![
            format!(
                "Each Match is evaluated {} s before its cutoff min(Scheduled Start − 15 min, actual start); every odds line, VRS snapshot and Liquipedia revision in its dossier was observed strictly before that instant. Each Stage's Pick'em is made 1 s before its first Match starts, from the Liquipedia Stage revision saved from before that instant (teams, Round 1 pairings, seeds).",
                evaluation_lead().num_seconds()
            ),
            "Scheduled Start and actual start come from PandaScore's final records (`scheduled_at`, `begin_at`).".into(),
            "Market lines: OddsPapi historical Pinnacle series-winner prices, which exist for settled fixtures since January 2026 only (IEM Cologne Major 2026 and PGL Major Singapore 2026), then Polymarket and Kalshi price histories, saved for PGL Major Singapore 2026 Stage 1 only.".into(),
            "Valve's global standings start in August 2024, so PGL Major Copenhagen 2024 has no rating: its Matches are not predicted and its Pick'em is drawn from even odds.".into(),
            "Rosters: Liquipedia Event page revisions saved for some Matches only (see the Rosters column). No dated news archive exists, so no dossier holds news.".into(),
        ];
        notes.extend(stages.iter().filter_map(|s| {
            s.opening_problem
                .as_ref()
                .map(|p| format!("{}: no Pick'em view ({p}).", s.title()))
        }));
        Ok(Report {
            archive_digest: self.archive.digest.clone(),
            archive_files: self.archive.files,
            profiles: PROFILES
                .iter()
                .map(|(k, l)| ((*k).to_owned(), (*l).to_owned()))
                .collect(),
            stages: stage_rows,
            tables,
            llm_moves,
            pickem,
            llm,
            notes,
        })
    }

    /// Of the anonymized slice, how many Matches' dossiers held a roster fact
    /// and how many a news item: what the report can say was masked.
    async fn blind_context(&self, stages: &[ReplayStage]) -> Result<(usize, usize)> {
        let (mut rosters, mut news) = (0, 0);
        for m in stages.iter().flat_map(|s| &s.matches) {
            if !self.plan.blind.contains(&m.game.id) {
                continue;
            }
            let at = m.cutoff() - evaluation_lead();
            let facts = self.feeds.context.facts(&m.game, at).await?;
            let kinds: Vec<String> = facts
                .iter()
                .filter_map(|f| f.at(at).ok()?["kind"].as_str().map(str::to_owned))
                .collect();
            rosters += usize::from(kinds.iter().any(|k| k == "liquipedia_roster"));
            news += usize::from(kinds.iter().any(|k| k == "news_item"));
        }
        Ok((rosters, news))
    }

    /// How many pairs of the Stage's teams had a rating or market line before it.
    async fn rated_pairs(&self, stage: &ReplayStage) -> Result<usize> {
        let Some(opening) = &stage.opening else {
            return Ok(0);
        };
        let ids: Vec<i64> = opening.value["teams"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|t| t["id"].as_i64())
            .collect();
        let teams: BTreeMap<i64, &Team> = stage.teams.iter().map(|t| (t.id, t)).collect();
        let at = stage.cutoff - evaluation_lead();
        let mut rated = 0;
        for (i, a) in ids.iter().enumerate() {
            for b in &ids[i + 1..] {
                let (Some(a), Some(b)) = (teams.get(a), teams.get(b)) else {
                    continue;
                };
                let game = Match {
                    event_id: stage.event_id,
                    id: 0,
                    team_a: (*a).clone(),
                    team_b: (*b).clone(),
                    format: Format::Bo1,
                    scheduled_start: Some(stage.cutoff),
                };
                if self.feeds.signal.signal(&game, at).await?.is_some() {
                    rated += 1;
                }
            }
        }
        Ok(rated)
    }
}

/// An LLM profile gives no Prediction without an LLM answer: where the debate
/// runtime would let the policy decide on its anchor alone (no provider in the
/// chain and no recorded answer), this fails as `NotConfigured` instead, so
/// the anchor is never scored as an LLM's pick.
struct Required(Arc<dyn LlmRuntime>);
#[async_trait::async_trait]
impl LlmRuntime for Required {
    async fn evaluate(
        &self,
        bot: &BotProfile,
        dossier: &Dossier,
        tier: Tier,
        prompt: &RolePrompt,
    ) -> Result<Option<Value>, LlmError> {
        match self.0.evaluate(bot, dossier, tier, prompt).await? {
            None if debate::base(dossier).is_some() => Err(LlmError::NotConfigured),
            answer => Ok(answer),
        }
    }
}

/// Loads recorded LLM answers (an export of `debate_turns`) into a fresh audit,
/// so that a rerun answers every Match it covers without calling a provider.
pub async fn import_answers(audit: &Audit, turns: &[Value]) -> Result<usize> {
    let mut n = 0;
    for t in turns.iter().filter(|t| t["status"] == "ok") {
        sqlx::query(
            "INSERT INTO debate_turns(bot, match_id, tier, input_hash, reuse_key, role, provider, latency_ms, status, brief)
             VALUES(?,?,?,?,?,?,?,?,?,?)",
        )
        .bind(t["bot"].as_str().ok_or("answer without bot")?)
        .bind(t["match_id"].as_i64().ok_or("answer without match_id")?)
        .bind(t["tier"].as_str().ok_or("answer without tier")?)
        .bind(t["input_hash"].as_str().unwrap_or_default())
        .bind(t["reuse_key"].as_str().ok_or("answer without reuse_key")?)
        .bind(t["role"].as_str().ok_or("answer without role")?)
        .bind(t["provider"].as_str().unwrap_or_default())
        .bind(t["latency_ms"].as_i64().unwrap_or_default())
        .bind("ok")
        .bind(t["brief"].to_string())
        .execute(&audit.db)
        .await?;
        n += 1;
    }
    Ok(n)
}

/// Every answered seat in the audit, in the form `import_answers` reads.
pub async fn export_answers(audit: &Audit) -> Result<Vec<Value>> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for t in audit.debate_turns().await? {
        if t.status != "ok"
            || !seen.insert((
                t.bot.clone(),
                t.tier.clone(),
                t.reuse_key.clone(),
                t.role.clone(),
            ))
        {
            continue;
        }
        out.push(json!({
            "bot": t.bot, "match_id": t.match_id, "tier": t.tier, "input_hash": t.input_hash,
            "reuse_key": t.reuse_key, "role": t.role, "provider": t.provider,
            "latency_ms": t.latency_ms, "status": t.status, "brief": t.brief,
        }));
    }
    Ok(out)
}
