//! Ticket #43 (spec #33): the LLM track of the Signal Lab. See
//! docs/agents/codebase.md, "LLM track of the Signal Lab (ticket #43)".
//!
//! The single agent and the three-seat debate are run, through the backtest's
//! live pipeline (`Backtest`: `Mode::Backtest`, so no CLI run gets a tool or the
//! web whatever a profile says; `Blind` for the Events before the models'
//! training cutoff), on every archived Match whose pre-cutoff dossier carries a
//! roster flag, once per seating. What they answered becomes Candidate Signals
//! (`LlmAnswers`) evaluated by the unchanged protocol: `llm_adjustment` (the
//! debate under the first seating) in the lab's own table, and every profile
//! in this track's comparison.
//!
//! Spending is bounded before anything runs: a run that names a provider needs
//! a budget of CLI runs for each one (`Budgeted` refuses the run after the last
//! one, which the pipeline reads as a quota error). The first real quota or
//! sign-in failure stops the whole track; a spent budget stops only the
//! seatings that need that subscription. Answers are recorded per seating
//! (`Recorded`) and reused, so a rerun from them calls no provider.
use super::{CandidateReport, CandidateSignal, FittedSignal, Lab, MatchView, Observed, stats};
use crate::adapters::backtest::{
    Backtest, ReplayArchive, export_answers,
    feeds::ReplayContext,
    import_answers,
    runner::{DEBATE, SINGLE, evaluation_lead},
};
use crate::adapters::llm::{
    Finished, Invocation, LlmConfig, ProcessRunner, ProviderConfig, RunError,
};
use crate::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write as _,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

/// The registry name of the LLM adjustment as a Candidate Signal.
pub const NAME: &str = "llm_adjustment";
/// Events that began before this instant are sent through the
/// `BlindAnonymizer` (the latest training cutoff of the seated models, kept
/// conservative: PGL Major Singapore 2026 is the first Event after it).
pub const DEFAULT_TRAINING_CUTOFF: &str = "2026-11-01T00:00:00Z";
/// Swiss Monte-Carlo runs of the Pick'em step inside a seating's replay (the
/// lab reads no Pick'em of it).
const SIMULATIONS: usize = 100;
/// The options of `kallbet-ai lab` this module reads.
pub const FLAGS: [&str; 4] = ["--llm", "--answers", "--budget", "--training-cutoff"];
pub const USAGE: &str =
    "[--llm FILE --budget NAME=RUNS...] [--answers FILE] [--training-cutoff RFC3339]";

/// Who sits where in the debate.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Seating {
    pub name: String,
    pub advocate: Vec<String>,
    pub devil: Vec<String>,
    pub arbiter: Vec<String>,
}

/// The `--llm` file: the sidecar config's `llm` section, the single agent's
/// chain and the seatings compared (the first one is `llm_adjustment`).
#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LlmFile {
    #[serde(default)]
    pub llm: LlmConfig,
    #[serde(default)]
    pub single: Vec<String>,
    #[serde(default)]
    pub seatings: Vec<Seating>,
}

/// The `--answers` file: every recorded answer, by profile, in the form
/// `backtest::import_answers` reads, and the seatings they were given under.
#[derive(Clone, Default, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Recorded {
    #[serde(default)]
    pub seatings: Vec<Seating>,
    #[serde(default)]
    pub single: Vec<Value>,
    /// Seating name → the debate's answered seats.
    #[serde(default)]
    pub debate: BTreeMap<String, Vec<Value>>,
}

/// The command-line options of the track.
#[derive(Debug, PartialEq)]
pub struct Options {
    pub llm: Option<PathBuf>,
    pub answers: Option<PathBuf>,
    /// `None`: no `--budget` was given, and no provider may be called.
    pub budget: Option<BTreeMap<String, usize>>,
    pub training_cutoff: Instant,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            llm: None,
            answers: None,
            budget: None,
            training_cutoff: DEFAULT_TRAINING_CUTOFF.parse().expect("a valid instant"),
        }
    }
}
impl Options {
    /// Takes one of `FLAGS` and its value.
    pub fn set(&mut self, flag: &str, value: String) -> Result<()> {
        match flag {
            "--llm" => self.llm = Some(value.into()),
            "--answers" => self.answers = Some(value.into()),
            "--training-cutoff" => {
                self.training_cutoff = chrono::DateTime::parse_from_rfc3339(&value)
                    .map_err(|e| format!("--training-cutoff {value:?}: {e}"))?
                    .to_utc()
            }
            "--budget" => {
                let (name, runs) = value
                    .split_once('=')
                    .ok_or(format!("--budget takes NAME=RUNS, not {value:?}"))?;
                let runs: usize = runs
                    .parse()
                    .map_err(|_| format!("--budget {value:?}: RUNS is a whole number"))?;
                let budget = self.budget.get_or_insert_with(BTreeMap::new);
                if budget.insert(name.to_owned(), runs).is_some() {
                    return Err(format!("--budget {name} given twice").into());
                }
            }
            other => return Err(format!("not an LLM track option: {other}").into()),
        }
        Ok(())
    }
}

/// What a track run is given. `work_dir` receives one audit file per seating.
pub struct Setup {
    pub llm: LlmConfig,
    pub single: Vec<String>,
    pub seatings: Vec<Seating>,
    pub budget: Option<BTreeMap<String, usize>>,
    pub recorded: Recorded,
    pub training_cutoff: Instant,
    pub work_dir: PathBuf,
}

/// The CLI program of every configured provider, after the checks that must
/// pass before any provider may be called.
fn programs(setup: &Setup) -> Result<BTreeMap<PathBuf, String>> {
    let providers = &setup.llm.providers;
    if providers.is_empty() {
        return Ok(BTreeMap::new());
    }
    let names: Vec<&str> = providers.keys().map(String::as_str).collect();
    let Some(budget) = &setup.budget else {
        return Err(format!(
            "refusing to call any provider: a lab run that calls one needs a budget of CLI runs \
             per subscription (--budget NAME=RUNS for each of {})",
            names.join(", ")
        )
        .into());
    };
    let unbudgeted: Vec<&str> = names
        .iter()
        .copied()
        .filter(|n| !budget.contains_key(*n))
        .collect();
    if !unbudgeted.is_empty() {
        return Err(format!(
            "refusing to call any provider: no --budget for {}",
            unbudgeted.join(", ")
        )
        .into());
    }
    if let Some(unknown) = budget.keys().find(|n| !providers.contains_key(*n)) {
        return Err(format!("--budget {unknown}: no such provider in the llm file").into());
    }
    let mut programs: BTreeMap<PathBuf, String> = BTreeMap::new();
    for (name, provider) in providers {
        let program = match provider {
            ProviderConfig::Claude(c) => c.binary_or("claude"),
            ProviderConfig::Codex(c) => c.binary_or("codex"),
            ProviderConfig::Antigravity(c) => c.binary_or("agy"),
            ProviderConfig::Http(_) => {
                return Err(format!(
                    "provider {name}: the lab budgets CLI runs, an http provider cannot be budgeted"
                )
                .into());
            }
        };
        if let Some(other) = programs.insert(program.clone(), name.clone()) {
            return Err(format!(
                "providers {other} and {name} run the same CLI ({}): a budget is per \
                 subscription, give each its own binary",
                program.display()
            )
            .into());
        }
    }
    Ok(programs)
}

/// The process runner of a track run: counts the CLI runs of every
/// subscription and refuses to start one past its budget (the CLI runtime
/// reads `RunError::OverBudget` as a quota error). A program that belongs to
/// no budgeted provider is never started.
pub struct Budgeted {
    inner: Arc<dyn ProcessRunner>,
    programs: BTreeMap<PathBuf, String>,
    budget: BTreeMap<String, usize>,
    used: Mutex<BTreeMap<String, (usize, usize)>>,
}
impl Budgeted {
    pub fn new(
        inner: Arc<dyn ProcessRunner>,
        programs: BTreeMap<PathBuf, String>,
        budget: BTreeMap<String, usize>,
    ) -> Self {
        Self {
            inner,
            programs,
            budget,
            used: Mutex::default(),
        }
    }
    /// (runs started, runs refused) by subscription.
    pub fn used(&self) -> BTreeMap<String, (usize, usize)> {
        self.used.lock().expect("budget lock").clone()
    }
    fn refused(&self) -> usize {
        self.used().values().map(|u| u.1).sum()
    }
}
#[async_trait::async_trait]
impl ProcessRunner for Budgeted {
    async fn run(&self, invocation: &Invocation) -> Result<Finished, RunError> {
        let Some(name) = self.programs.get(&invocation.program) else {
            return Err(RunError::OverBudget);
        };
        {
            let mut used = self.used.lock().expect("budget lock");
            let entry = used.entry(name.clone()).or_default();
            if entry.0 >= self.budget.get(name).copied().unwrap_or(0) {
                entry.1 += 1;
                return Err(RunError::OverBudget);
            }
            entry.0 += 1;
        }
        self.inner.run(invocation).await
    }
}

/// A Match the track asks about.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Flagged {
    pub match_id: i64,
    pub event_id: i64,
    /// Sent through the `BlindAnonymizer`: its Event began before the
    /// models' training cutoff.
    pub blind: bool,
}

/// Every scored Match whose dossier, read at its evaluation instant, holds a
/// roster fact of either team with a stand-in (`has_stand_in`): the roster
/// flag, the case the LLM adjustment's factors are about.
pub async fn flagged(archive: &ReplayArchive, training_cutoff: Instant) -> Result<Vec<Flagged>> {
    let context = ReplayContext::new(archive);
    let mut begins: BTreeMap<i64, Instant> = BTreeMap::new();
    for s in &archive.stages {
        let b = begins.entry(s.event_id).or_insert(s.cutoff);
        *b = (*b).min(s.cutoff);
    }
    let mut out = Vec::new();
    for stage in &archive.stages {
        for m in &stage.matches {
            let at = m.cutoff() - evaluation_lead();
            let mut flag = false;
            for fact in context.facts(&m.game, at).await? {
                let fact = fact.at(at)?;
                flag |= fact["kind"] == "liquipedia_roster" && fact["has_stand_in"] == true;
            }
            if flag {
                out.push(Flagged {
                    match_id: m.game.id,
                    event_id: stage.event_id,
                    blind: begins[&stage.event_id] < training_cutoff,
                });
            }
        }
    }
    Ok(out)
}

/// The archive with only the Stages that hold a Match of `ids`.
fn subset(archive: &ReplayArchive, ids: &BTreeSet<i64>) -> ReplayArchive {
    ReplayArchive {
        stages: archive
            .stages
            .iter()
            .filter(|s| s.matches.iter().any(|m| ids.contains(&m.game.id)))
            .cloned()
            .collect(),
        match_context: archive.match_context.clone(),
        standings: archive.standings.clone(),
        fixture_lists: archive.fixture_lists.clone(),
        histories: archive.histories.clone(),
        polymarket_events: archive.polymarket_events.clone(),
        polymarket_histories: archive.polymarket_histories.clone(),
        kalshi_markets: archive.kalshi_markets.clone(),
        kalshi_candles: archive.kalshi_candles.clone(),
        event_pages: archive.event_pages.clone(),
        event_titles: archive.event_titles.clone(),
        hltv_matches: archive.hltv_matches.clone(),
        map_history: archive.map_history.clone(),
        veto_history: archive.veto_history.clone(),
        digest: archive.digest.clone(),
        files: archive.files,
    }
}

/// One seating's run.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SeatingRun {
    pub seating: Seating,
    /// `false`: not run, the track had stopped before it.
    pub ran: bool,
    /// Flagged Matches the debate answered, recorded or new.
    pub answered: usize,
    /// Provider calls this run made (recorded answers are not calls).
    pub calls: usize,
    /// Why it stopped early, if it did.
    pub stopped: Option<String>,
}

/// A subscription's spending.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct BudgetRow {
    pub subscription: String,
    pub budget: usize,
    /// CLI runs started.
    pub used: usize,
    /// CLI runs refused because the budget was spent.
    pub refused: usize,
}

/// What a track run produced.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Track {
    pub training_cutoff: Instant,
    pub flagged: Vec<Flagged>,
    pub single_chain: Vec<String>,
    pub seatings: Vec<SeatingRun>,
    /// Team A's probability by Match: the single agent's, and each seating's debate's.
    pub single: BTreeMap<i64, f64>,
    pub debate: BTreeMap<String, BTreeMap<i64, f64>>,
    pub budget: Vec<BudgetRow>,
    /// The first quota or sign-in failure, after which nothing was asked.
    pub stopped: Option<String>,
    /// Every answer, recorded or new: the `--answers` file to write.
    #[serde(skip)]
    pub recorded: Recorded,
}

fn p_team_a(game: &Match, prediction: &Prediction) -> Option<f64> {
    let p = prediction.rationale["p_win"].as_f64()?;
    Some(if prediction.team_id == game.team_a.id {
        p
    } else {
        1.0 - p
    })
}

/// Adds `new` answers to `into`, once per (tier, reuse key, role).
fn merge(into: &mut Vec<Value>, new: impl IntoIterator<Item = Value>) {
    let key = |t: &Value| {
        (
            t["tier"].to_string(),
            t["reuse_key"].to_string(),
            t["role"].to_string(),
        )
    };
    let mut seen: BTreeSet<_> = into.iter().map(key).collect();
    for t in new {
        if seen.insert(key(&t)) {
            into.push(t);
        }
    }
}

/// Runs the track over the flagged Matches of `archive`: one replay per
/// seating, each with its own fresh audit file in `setup.work_dir` holding the
/// recorded answers of the single agent and of that seating.
pub async fn run(
    archive: &ReplayArchive,
    setup: &Setup,
    runner: Arc<dyn ProcessRunner>,
) -> Result<Track> {
    let programs = programs(setup)?;
    let budgeted = Arc::new(Budgeted::new(
        runner,
        programs,
        setup.budget.clone().unwrap_or_default(),
    ));
    let flagged = flagged(archive, setup.training_cutoff).await?;
    let mut recorded = setup.recorded.clone();
    let seatings = if setup.seatings.is_empty() {
        recorded.seatings.clone()
    } else {
        setup.seatings.clone()
    };
    for s in &seatings {
        match recorded.seatings.iter_mut().find(|r| r.name == s.name) {
            Some(r) => *r = s.clone(),
            None => recorded.seatings.push(s.clone()),
        }
    }
    let ids: BTreeSet<i64> = flagged.iter().map(|f| f.match_id).collect();
    let games: BTreeMap<i64, Match> = archive
        .stages
        .iter()
        .flat_map(|s| &s.matches)
        .map(|m| (m.game.id, m.game.clone()))
        .collect();
    let mut track = Track {
        training_cutoff: setup.training_cutoff,
        flagged: flagged.clone(),
        single_chain: setup.single.clone(),
        seatings: vec![],
        single: BTreeMap::new(),
        debate: BTreeMap::new(),
        budget: vec![],
        stopped: None,
        recorded: Recorded::default(),
    };
    std::fs::create_dir_all(&setup.work_dir)?;
    for (i, seating) in seatings.iter().enumerate() {
        if let Some(why) = &track.stopped {
            track.seatings.push(SeatingRun {
                seating: seating.clone(),
                ran: false,
                answered: 0,
                calls: 0,
                stopped: Some(format!("not run: the LLM track stopped at {why}")),
            });
            continue;
        }
        let mut backtest = Backtest::new(subset(archive, &ids), budgeted.clone())?;
        backtest.llm = setup.llm.clone();
        backtest.simulations = SIMULATIONS;
        backtest.plan.direct = flagged
            .iter()
            .filter(|f| !f.blind)
            .map(|f| f.match_id)
            .collect();
        backtest.plan.blind = flagged
            .iter()
            .filter(|f| f.blind)
            .map(|f| f.match_id)
            .collect();
        backtest.plan.single = setup.single.clone();
        backtest.plan.advocate = seating.advocate.clone();
        backtest.plan.devil = seating.devil.clone();
        backtest.plan.arbiter = seating.arbiter.clone();

        let path = setup.work_dir.join(format!("seating-{i}.sqlite"));
        for suffix in ["", "-shm", "-wal"] {
            let p = PathBuf::from(format!("{}{suffix}", path.display()));
            if p.exists() {
                std::fs::remove_file(p)?;
            }
        }
        let audit = Audit::open(&path).await?;
        import_answers(&audit, &recorded.single).await?;
        if let Some(turns) = recorded.debate.get(&seating.name) {
            import_answers(&audit, turns).await?;
        }
        let refused_before = budgeted.refused();
        let outcome = backtest.run(&audit).await?;
        let mut debate_turns = recorded.debate.remove(&seating.name).unwrap_or_default();
        let (single, debate): (Vec<Value>, Vec<Value>) = export_answers(&audit)
            .await?
            .into_iter()
            .partition(|t| t["bot"] == SINGLE);
        merge(&mut recorded.single, single);
        merge(
            &mut debate_turns,
            debate.into_iter().filter(|t| t["bot"] == DEBATE),
        );
        recorded.debate.insert(seating.name.clone(), debate_turns);
        audit.db.close().await;

        let mut answers = BTreeMap::new();
        for p in &outcome.predictions {
            let Some(game) = games.get(&p.match_id) else {
                continue;
            };
            let Some(prob) = p_team_a(game, &p.prediction) else {
                continue;
            };
            match p.bot.as_str() {
                SINGLE => {
                    track.single.entry(p.match_id).or_insert(prob);
                }
                DEBATE => {
                    answers.insert(p.match_id, prob);
                }
                _ => {}
            }
        }
        let llm = &outcome.report.llm;
        let stopped = llm.stopped.clone().map(|why| {
            if budgeted.refused() > refused_before {
                format!("a subscription's budget was spent ({why})")
            } else {
                track.stopped = Some(why.clone());
                why
            }
        });
        track.seatings.push(SeatingRun {
            seating: seating.clone(),
            ran: true,
            answered: answers.len(),
            calls: llm.calls.values().sum(),
            stopped,
        });
        track.debate.insert(seating.name.clone(), answers);
    }
    let used = budgeted.used();
    track.budget = setup
        .llm
        .providers
        .keys()
        .map(|name| {
            let (u, r) = used.get(name).copied().unwrap_or_default();
            BudgetRow {
                subscription: name.clone(),
                budget: setup
                    .budget
                    .as_ref()
                    .and_then(|b| b.get(name))
                    .copied()
                    .unwrap_or(0),
                used: u,
                refused: r,
            }
        })
        .collect();
    track.recorded = recorded;
    Ok(track)
}

/// Recorded LLM answers as a Candidate Signal: nothing is fitted (the LLM's
/// adjustment is taken as it was given), and a Match's probability is dated
/// at its evaluation instant, one second before the cutoff.
pub struct LlmAnswers {
    pub name: &'static str,
    pub description: &'static str,
    pub answers: Arc<BTreeMap<i64, f64>>,
    /// Why there are none, when there are none.
    pub missing: Option<String>,
}
impl LlmAnswers {
    /// `llm_adjustment` before any track ran: `not testable` with how to run it.
    pub fn unrun() -> Self {
        Self {
            name: NAME,
            description: DESCRIPTION,
            answers: Arc::default(),
            missing: Some(format!(
                "no LLM answers in this run; produce them with `{}`",
                command(None, None, &[])
            )),
        }
    }
}
const DESCRIPTION: &str = "The three-seat debate's adjustment (HybridAnchored, debate mode \
`always`, the first seating) on the Matches with a roster flag, from recorded answers";

impl CandidateSignal for LlmAnswers {
    fn name(&self) -> &'static str {
        self.name
    }
    fn description(&self) -> &'static str {
        self.description
    }
    fn needs(&self) -> &'static str {
        "a recorded LLM answer for a Match whose dossier has a roster flag"
    }
    fn missing_data(&self, _archive: &ReplayArchive) -> Option<String> {
        self.missing.clone()
    }
    fn fit(&self, _training: &[Observed]) -> Result<Box<dyn FittedSignal>> {
        Ok(Box::new(Fitted(self.answers.clone())))
    }
}
struct Fitted(Arc<BTreeMap<i64, f64>>);
impl FittedSignal for Fitted {
    fn predict(&self, m: &MatchView) -> Result<Option<AsOf<f64>>> {
        self.0
            .get(&m.game.id)
            .map(|p| Ok(AsOf::new(*p, m.cutoff - evaluation_lead(), m.cutoff)?))
            .transpose()
    }
}

fn leak(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

impl Track {
    /// `llm_adjustment`: the debate under the first seating.
    pub fn adjustment(&self) -> LlmAnswers {
        let first = self.seatings.first();
        let answers = first
            .and_then(|s| self.debate.get(&s.seating.name))
            .cloned()
            .unwrap_or_default();
        let missing = match first {
            None => Some("no seating was given or recorded".to_owned()),
            Some(s) if !s.ran => s.stopped.clone(),
            _ => None,
        };
        LlmAnswers {
            name: NAME,
            description: DESCRIPTION,
            answers: Arc::new(answers),
            missing,
        }
    }
    /// Every profile as a Candidate Signal: the single agent, then the debate
    /// under each seating.
    pub fn profiles(&self) -> Vec<Arc<dyn CandidateSignal>> {
        let mut out: Vec<Arc<dyn CandidateSignal>> = vec![Arc::new(LlmAnswers {
            name: "llm_single",
            description: "The single agent (HybridAnchored, debate mode `never`)",
            answers: Arc::new(self.single.clone()),
            missing: None,
        })];
        for s in &self.seatings {
            out.push(Arc::new(LlmAnswers {
                name: leak(format!("llm_debate_{}", s.seating.name)),
                description: leak(format!(
                    "The debate with {} as Market Advocate, {} as Devil's Advocate, {} as Chief Arbiter",
                    s.seating.advocate.join("/"),
                    s.seating.devil.join("/"),
                    s.seating.arbiter.join("/")
                )),
                answers: Arc::new(self.debate.get(&s.seating.name).cloned().unwrap_or_default()),
                missing: (!s.ran).then(|| s.stopped.clone().unwrap_or_default()),
            }));
        }
        out
    }
    /// What the flagged Matches lack, one line each (empty: nothing).
    pub fn missing(&self) -> Vec<String> {
        let n = self.flagged.len();
        let mut out = Vec::new();
        if self.seatings.is_empty() {
            out.push("no seating was given (`seatings` in the --llm file) or recorded".into());
        }
        if self.single.len() < n {
            out.push(format!(
                "single agent: {} of {n} flagged Matches have no answer",
                n - self.single.len()
            ));
        }
        for s in &self.seatings {
            let answered = self.debate.get(&s.seating.name).map_or(0, BTreeMap::len);
            if answered < n {
                out.push(format!(
                    "seating `{}`: {} of {n} flagged Matches have no answer{}",
                    s.seating.name,
                    n - answered,
                    s.stopped
                        .as_deref()
                        .map(|w| format!(" ({w})"))
                        .unwrap_or_default()
                ));
            }
        }
        out
    }
}

/// The command that produces what is missing. The budget is the operator's:
/// `<RUNS>` stands for it.
pub fn command(llm: Option<&Path>, answers: Option<&Path>, subscriptions: &[String]) -> String {
    let path = |p: Option<&Path>, placeholder: &str| {
        p.map_or(placeholder.to_owned(), |p| p.display().to_string())
    };
    let mut cmd = format!(
        "cargo run -p kallbet-ai -- lab --llm {} --answers {}",
        path(llm, "<LLM_FILE>"),
        path(answers, "<ANSWERS_FILE>")
    );
    let subscriptions: Vec<String> = if subscriptions.is_empty() {
        ["claude", "codex", "antigravity"]
            .map(String::from)
            .to_vec()
    } else {
        subscriptions.to_vec()
    };
    for s in subscriptions {
        let _ = write!(cmd, " --budget {s}=<RUNS>");
    }
    cmd
}

/// One pairwise comparison of two LLM profiles on the Matches both answered.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Pairwise {
    pub seating: String,
    pub paired: usize,
    /// Mean log-loss, debate − single agent.
    pub log_loss_difference: Option<f64>,
    pub interval: Option<stats::Interval>,
}

/// What the track's report shows.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TrackReport {
    pub track: Track,
    /// Every profile against the anchor, by the lab's protocol.
    pub profiles: Vec<CandidateReport>,
    /// The debate of each seating against the single agent.
    pub debate_vs_single: Vec<Pairwise>,
    pub missing: Vec<String>,
    pub command: Option<String>,
}

/// Evaluates every profile of `track` by the lab's protocol (its runner, with
/// the profiles as the Candidate Signals) and compares each seating's debate
/// with the single agent.
pub async fn compare(lab: &mut Lab, track: &Track, command: String) -> Result<TrackReport> {
    let saved = std::mem::replace(&mut lab.signals, track.profiles());
    let outcome = lab.run().await;
    lab.signals = saved;
    let profiles = outcome?.report.candidates;
    let winners: BTreeMap<i64, bool> = lab
        .archive
        .stages
        .iter()
        .flat_map(|s| &s.matches)
        .map(|m| (m.game.id, m.winner == m.game.team_a.id))
        .collect();
    let events: BTreeMap<i64, i64> = track
        .flagged
        .iter()
        .map(|f| (f.match_id, f.event_id))
        .collect();
    let mut debate_vs_single = Vec::new();
    for s in &track.seatings {
        let debate = track.debate.get(&s.seating.name);
        let diffs: Vec<(i64, f64)> = track
            .single
            .iter()
            .filter_map(|(id, single)| {
                let d = debate?.get(id)?;
                let won = *winners.get(id)?;
                Some((
                    *events.get(id)?,
                    stats::log_loss(*d, won) - stats::log_loss(*single, won),
                ))
            })
            .collect();
        let boot = stats::event_bootstrap(
            &diffs,
            lab.protocol.resamples,
            lab.protocol.level,
            lab.protocol.seed,
        );
        debate_vs_single.push(Pairwise {
            seating: s.seating.name.clone(),
            paired: diffs.len(),
            log_loss_difference: (!diffs.is_empty())
                .then(|| diffs.iter().map(|d| d.1).sum::<f64>() / diffs.len() as f64),
            interval: boot.map(|b| b.interval),
        });
    }
    let missing = track.missing();
    Ok(TrackReport {
        track: track.clone(),
        profiles,
        debate_vs_single,
        command: (!missing.is_empty()).then_some(command),
        missing,
    })
}

fn num(v: Option<f64>) -> String {
    v.map_or("—".into(), |v| format!("{v:+.4}"))
}
fn interval(i: Option<stats::Interval>) -> String {
    i.map_or("—".into(), |i| format!("[{:+.4}, {:+.4}]", i.low, i.high))
}

impl TrackReport {
    pub fn markdown(&self) -> String {
        let t = &self.track;
        let mut md = String::new();
        let blind = t.flagged.iter().filter(|f| f.blind).count();
        let _ = writeln!(md, "# LLM track of the Signal Lab\n");
        let _ = writeln!(
            md,
            "The single agent and the three-seat debate on every archived Match whose dossier \
             holds a roster fact with a stand-in, read at its evaluation instant: {} Matches, {} \
             with real names and {blind} through the BlindAnonymizer (Events that began before \
             {}). Every CLI run is a backtest run: no tool and no web, whatever a profile says. \
             Each profile is compared with the anchor by the lab's protocol, on the Matches both \
             predicted.\n",
            t.flagged.len(),
            t.flagged.len() - blind,
            t.training_cutoff.to_rfc3339()
        );
        let _ = writeln!(md, "## Against the anchor\n");
        let _ = writeln!(
            md,
            "| Profile | Paired | Anchor log-loss | Profile log-loss | Δ log-loss | 90% interval | Δ Brier | Δ accuracy | Verdict |"
        );
        let _ = writeln!(md, "|---|---:|---:|---:|---:|---|---:|---:|---|");
        for c in &self.profiles {
            let _ = writeln!(
                md,
                "| `{}` | {} | {} | {} | {} | {} | {} | {} | {}: {} |",
                c.name,
                c.paired,
                c.baseline
                    .map_or("—".into(), |m| format!("{:.4}", m.log_loss)),
                c.variant
                    .map_or("—".into(), |m| format!("{:.4}", m.log_loss)),
                num(c.difference.map(|d| d.log_loss)),
                interval(c.interval),
                num(c.difference.map(|d| d.brier)),
                num(c.difference.map(|d| d.accuracy)),
                c.verdict.as_str(),
                c.reason
            );
        }
        let _ = writeln!(md, "\n## Debate against the single agent\n");
        let _ = writeln!(
            md,
            "| Seating | Paired | Δ log-loss (debate − single) | 90% interval |"
        );
        let _ = writeln!(md, "|---|---:|---:|---|");
        for p in &self.debate_vs_single {
            let _ = writeln!(
                md,
                "| `{}` | {} | {} | {} |",
                p.seating,
                p.paired,
                num(p.log_loss_difference),
                interval(p.interval)
            );
        }
        let _ = writeln!(md, "\n## Seatings\n");
        let _ = writeln!(
            md,
            "Single agent: {}.\n",
            if t.single_chain.is_empty() {
                "recorded answers only".to_owned()
            } else {
                t.single_chain.join(" → ")
            }
        );
        let _ = writeln!(
            md,
            "| Seating | Market Advocate | Devil's Advocate | Chief Arbiter | Answered | Status |"
        );
        let _ = writeln!(md, "|---|---|---|---|---:|---|");
        for s in &t.seatings {
            let _ = writeln!(
                md,
                "| `{}` | {} | {} | {} | {} | {} |",
                s.seating.name,
                s.seating.advocate.join(" → "),
                s.seating.devil.join(" → "),
                s.seating.arbiter.join(" → "),
                s.answered,
                s.stopped
                    .as_deref()
                    .unwrap_or(if s.ran { "complete" } else { "not run" })
            );
        }
        let _ = writeln!(md, "\n## Budget\n");
        if t.budget.is_empty() {
            let _ = writeln!(md, "No provider was configured: no CLI was run.");
        } else {
            let _ = writeln!(md, "| Subscription | Budget (CLI runs) | Used | Refused |");
            let _ = writeln!(md, "|---|---:|---:|---:|");
            for b in &t.budget {
                let _ = writeln!(
                    md,
                    "| {} | {} | {} | {} |",
                    b.subscription, b.budget, b.used, b.refused
                );
            }
        }
        if let Some(why) = &t.stopped {
            let _ = writeln!(md, "\nThe LLM track stopped at {why}.");
        }
        let _ = writeln!(md, "\n## Missing\n");
        if self.missing.is_empty() {
            let _ = writeln!(
                md,
                "Nothing: every flagged Match has an answer from every profile."
            );
        } else {
            for m in &self.missing {
                let _ = writeln!(md, "- {m}");
            }
            if let Some(cmd) = &self.command {
                let _ = writeln!(
                    md,
                    "\nTo produce it (recorded answers are reused; `<RUNS>` is the budget of CLI \
                     runs you choose for each subscription):\n\n```\n{cmd}\n```"
                );
            }
        }
        md
    }
}

/// `kallbet-ai lab`'s use of the track: runs it when `--llm` or `--answers`
/// is given, puts `llm_adjustment` in the lab, writes the answers file.
pub async fn prepare(
    lab: &mut Lab,
    o: &Options,
    runner: Arc<dyn ProcessRunner>,
    out: &Path,
) -> Result<Option<Track>> {
    if o.llm.is_none() && o.answers.is_none() {
        return Ok(None);
    }
    let file: LlmFile = match &o.llm {
        Some(path) => serde_json::from_slice(&std::fs::read(path)?)
            .map_err(|e| format!("{}: {e}", path.display()))?,
        None => LlmFile::default(),
    };
    let recorded: Recorded = match &o.answers {
        Some(path) if path.exists() => serde_json::from_slice(&std::fs::read(path)?)
            .map_err(|e| format!("{}: {e}", path.display()))?,
        _ => Recorded::default(),
    };
    let setup = Setup {
        llm: file.llm,
        single: file.single,
        seatings: file.seatings,
        budget: o.budget.clone(),
        recorded,
        training_cutoff: o.training_cutoff,
        work_dir: out.join("llm-audit"),
    };
    let track = run(&lab.archive, &setup, runner).await?;
    if let Some(path) = &o.answers {
        std::fs::write(path, serde_json::to_string_pretty(&track.recorded)? + "\n")?;
    }
    for s in &mut lab.signals {
        if s.name() == NAME {
            *s = Arc::new(track.adjustment());
        }
    }
    Ok(Some(track))
}

/// Writes `llm.md` and `llm.json` into `out`.
pub async fn write(lab: &mut Lab, track: Option<&Track>, o: &Options, out: &Path) -> Result<()> {
    let subscriptions: Vec<String> = track
        .map(|t| t.budget.iter().map(|b| b.subscription.clone()).collect())
        .unwrap_or_default();
    let cmd = command(o.llm.as_deref(), o.answers.as_deref(), &subscriptions);
    let Some(track) = track else {
        std::fs::write(
            out.join("llm.md"),
            format!(
                "# LLM track of the Signal Lab\n\nNot run: no `--llm` file and no `--answers`. \
                 To run it (`<RUNS>` is the budget of CLI runs you choose for each \
                 subscription):\n\n```\n{cmd}\n```\n"
            ),
        )?;
        return Ok(());
    };
    let report = compare(lab, track, cmd).await?;
    std::fs::write(out.join("llm.md"), report.markdown())?;
    std::fs::write(
        out.join("llm.json"),
        serde_json::to_string_pretty(&report)? + "\n",
    )?;
    Ok(())
}
