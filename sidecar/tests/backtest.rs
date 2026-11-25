//! Seam 1 of ticket #31: the retrospective runner drives the live pipeline
//! (`Sidecar::tick`, policies, debate, Pick'em step) over saved real responses
//! of completed Stages (`tests/fixtures/backtest/`, provenance in its README).
//! Only the outside world is scripted: a feed that injects a datum, and the CLI
//! runner, which answers with the saved output of the real CLIs.
use async_trait::async_trait;
use chrono::Duration;
use kallbet_ai::{
    adapters::{
        backtest::{
            Backtest, BlindAnonymizer, Feeds, LlmPlan, ReplayArchive, Submitted,
            feeds::{MarketSource, ReplayOdds},
            runner::{DEBATE, MARKET, SINGLE, VRS},
        },
        llm::{Cli, CliConfig, Finished, Invocation, ProcessRunner, RunError},
    },
    *,
};
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

const SINGAPORE_STAGE_1: i64 = 22378;

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/backtest")
}

/// A replay archive of the saved files whose names start with one of `prefixes`.
fn archive_of(prefixes: &[&str]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    for entry in std::fs::read_dir(fixtures()).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        if prefixes.iter().any(|p| name.starts_with(p)) {
            std::fs::copy(&path, dir.path().join(&name)).unwrap();
        }
    }
    dir
}

/// Every saved response about PGL Major Singapore 2026 Stage 1.
fn singapore() -> tempfile::TempDir {
    archive_of(&[
        "panda-22378",
        "lp-22378",
        "roster-1789",
        "roster-1816",
        "standings_global_2026_11_02",
        "oddspapi-fixtures-pgl-singapore",
        "od1740001",
        "polymarket-",
        "kalshi-",
    ])
}

fn load(dir: &tempfile::TempDir) -> ReplayArchive {
    ReplayArchive::load(dir.path()).unwrap()
}

async fn fresh_audit() -> (tempfile::TempDir, Audit) {
    let dir = tempfile::tempdir().unwrap();
    let audit = Audit::open(&dir.path().join("audit.sqlite")).await.unwrap();
    (dir, audit)
}

fn saved_cli(name: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("tests/fixtures/debate/{name}")),
    )
    .unwrap()
}

fn prompt_of(invocation: &Invocation) -> String {
    invocation.stdin.clone().unwrap_or_else(|| {
        let print = invocation
            .args
            .iter()
            .find_map(|a| a.strip_prefix("--print="));
        print.unwrap_or_default().to_owned()
    })
}
fn role_of(invocation: &Invocation) -> String {
    let prompt = prompt_of(invocation);
    let rest = prompt.split("You are the `").nth(1).unwrap_or_default();
    rest.split('`').next().unwrap_or_default().to_owned()
}
fn program_of(invocation: &Invocation) -> String {
    let stem = invocation.program.file_stem().unwrap_or_default();
    stem.to_string_lossy().into_owned()
}

type Answer = Box<dyn Fn(&Invocation) -> Option<String> + Send + Sync>;

/// The CLIs: records every run and answers with what the real CLI printed for
/// that seat (`tests/fixtures/debate/`), or with `answer` when it gives one.
struct Clis {
    started: Mutex<Vec<Invocation>>,
    answer: Answer,
}
impl Clis {
    fn saved() -> Arc<Self> {
        Self::with(Box::new(|_| None))
    }
    fn with(answer: Answer) -> Arc<Self> {
        Arc::new(Self {
            started: Mutex::default(),
            answer,
        })
    }
    fn started(&self) -> Vec<Invocation> {
        self.started.lock().unwrap().clone()
    }
}
#[async_trait]
impl ProcessRunner for Clis {
    async fn run(&self, invocation: &Invocation) -> Result<Finished, RunError> {
        self.started.lock().unwrap().push(invocation.clone());
        if let Some(stdout) = (self.answer)(invocation) {
            return Ok(Finished {
                code: Some(0),
                stdout,
                stderr: String::new(),
            });
        }
        let (seat, cli) = (role_of(invocation), program_of(invocation));
        let seat = if seat == "predict" { "predict" } else { &seat };
        let stdout = match cli.as_str() {
            "codex" => saved("devil-codex.jsonl"),
            "agy" => saved("arbiter-agy.json"),
            _ if seat == "advocate" || seat == "devil" => saved("advocate-claude.json"),
            _ => saved("arbiter-claude.json"),
        };
        let stderr = if cli == "agy" {
            saved("arbiter-agy.stderr.txt")
        } else {
            String::new()
        };
        Ok(Finished {
            // `agy` left with 3 when its quota was exhausted (the saved run).
            code: Some(if stderr.is_empty() { 0 } else { 3 }),
            stdout,
            stderr,
        })
    }
}
fn saved(name: &str) -> String {
    saved_cli(name)
}

/// The `llm` section a profile could carry: all three subscriptions, and the
/// two LLM profiles listed as allowed to search the web in live runs.
fn llm_config() -> kallbet_ai::adapters::llm::LlmConfig {
    serde_json::from_value(json!({
        "providers": {
            "claude": {"kind": "claude"},
            "codex": {"kind": "codex"},
            "antigravity": {"kind": "antigravity"}
        },
        "web_search_bots": [SINGLE, DEBATE]
    }))
    .unwrap()
}

fn chains(plan: &mut LlmPlan, arbiter: &[&str]) {
    plan.single = vec!["claude".into()];
    plan.advocate = vec!["claude".into()];
    plan.devil = vec!["codex".into()];
    plan.arbiter = arbiter.iter().map(|s| (*s).into()).collect();
}

fn by_bot<'a>(predictions: &'a [Submitted], bot: &str) -> Vec<&'a Submitted> {
    predictions.iter().filter(|p| p.bot == bot).collect()
}

#[tokio::test]
async fn completed_singapore_matches_are_replayed_through_the_pipeline_and_scored() {
    let dir = singapore();
    let mut backtest = Backtest::new(load(&dir), Clis::saved()).unwrap();
    backtest.simulations = 500;
    let (_a, audit) = fresh_audit().await;
    let outcome = backtest.run(&audit).await.unwrap();

    // 24 Matches were finished when the PandaScore list was saved.
    let stage = &backtest.archive.stages[0];
    assert_eq!(
        (stage.id, stage.matches.len(), stage.complete),
        (SINGAPORE_STAGE_1, 24, false)
    );
    assert_eq!(by_bot(&outcome.predictions, VRS).len(), 24);
    assert_eq!(by_bot(&outcome.predictions, MARKET).len(), 24);
    // No LLM Match was planned, so the LLM profiles stayed silent.
    assert!(by_bot(&outcome.predictions, SINGLE).is_empty());

    let report = &outcome.report;
    let row = &report.stages[0];
    assert_eq!(
        (row.with_vrs, row.with_market, row.with_rosters),
        (24, 24, 24)
    );
    let all = &report.tables[0];
    let vrs = all
        .rows
        .iter()
        .find(|r| r.profile == "VrsOnly" && r.format == "all")
        .unwrap();
    assert_eq!(vrs.scores.matches, 24);
    let bo3 = all
        .rows
        .iter()
        .find(|r| r.profile == "VrsOnly" && r.format == "bo3")
        .unwrap();
    assert_eq!(bo3.scores.matches, 4);
    // The Stage is still being played: its Pick'em is submitted, its Points pending.
    let pickem = &report.pickem[0];
    assert_eq!((pickem.points, pickem.picks), (None, 10));
    assert!(pickem.status.starts_with("pending"), "{}", pickem.status);
    assert!(
        report
            .markdown()
            .contains("PGL Major Singapore 2026 / Stage 1")
    );
}

#[tokio::test]
async fn saved_market_histories_hold_in_play_prices_that_are_never_read() {
    let dir = singapore();
    let archive = load(&dir);
    let markets = ReplayOdds::new(&archive);
    // NIP vs M80, started 05:04:13 (Scheduled 05:00): cutoff 04:45.
    let game = archive.stages[0]
        .matches
        .iter()
        .find(|m| m.game.id == 1789258)
        .unwrap();
    let cutoff = game.cutoff();
    let after = |t: Instant| t >= cutoff;
    let polymarket = &archive.polymarket_histories["cs2-nip-m80-2026-11-24"];
    assert!(
        polymarket["history"]
            .as_array()
            .unwrap()
            .iter()
            .any(|p| { after(Instant::from_timestamp(p["t"].as_i64().unwrap(), 0).unwrap()) })
    );
    for source in MarketSource::ALL {
        let quote = markets
            .quote_from(source, &game.game, cutoff)
            .unwrap()
            .unwrap_or_else(|| panic!("{source:?} priced NIP vs M80"));
        assert!(quote.observed_at() < cutoff, "{source:?}");
        let p = *quote.at(cutoff).unwrap();
        assert!(p > 0.5 && p < 1.0, "{source:?} had NIP favoured: {p}");
    }
}

/// A feed that answers truthfully, except for one Match, where it hands the
/// pipeline a datum observed at `observed`.
struct Inject {
    game: i64,
    observed: Instant,
    kind: &'static str,
    feeds: Feeds,
}
fn far(at: Instant) -> Instant {
    at + Duration::days(365)
}
#[async_trait]
impl OddsSource for Inject {
    async fn quote(&self, game: &Match, cutoff: Instant) -> Result<Option<AsOf<f64>>> {
        if self.kind == "odds" && game.id == self.game {
            return Ok(Some(AsOf::new(0.6, self.observed, far(self.observed))?));
        }
        self.feeds.odds.quote(game, cutoff).await
    }
}
#[async_trait]
impl SignalEstimator for Inject {
    async fn signal(&self, game: &Match, cutoff: Instant) -> Result<Option<AsOf<Signal>>> {
        if self.kind == "vrs" && game.id == self.game {
            let s = Signal {
                map: 0.6,
                series: 0.6,
            };
            return Ok(Some(AsOf::new(s, self.observed, far(self.observed))?));
        }
        self.feeds.signal.signal(game, cutoff).await
    }
}
#[async_trait]
impl ContextEnricher for Inject {
    async fn facts(&self, game: &Match, cutoff: Instant) -> Result<Vec<AsOf<Value>>> {
        let mut facts = self.feeds.context.facts(game, cutoff).await?;
        if game.id == self.game {
            let fact = match self.kind {
                "liquipedia" => json!({"kind": "liquipedia_roster", "team": game.team_a.name,
                    "match_side": "team_a", "has_stand_in": true}),
                "news" => json!({"kind": "news_item", "title": "Stand-in announced",
                    "url": "https://www.hltv.org/news/1/x", "published_at": self.observed}),
                _ => return Ok(facts),
            };
            facts.push(AsOf::new(fact, self.observed, far(self.observed))?);
        }
        Ok(facts)
    }
}

async fn run_with_injected(kind: &'static str, offset: Duration) -> Result<(), String> {
    let dir = singapore();
    let mut backtest = Backtest::new(load(&dir), Clis::saved()).unwrap();
    backtest.simulations = 200;
    // Round 2, PARIVISION vs FlyQuest: Scheduled 11:00, started 11:03:28.
    let game = backtest.archive.stages[0]
        .matches
        .iter()
        .find(|m| m.game.id == 1816909)
        .unwrap();
    let observed = game.cutoff() + offset;
    let inject = Arc::new(Inject {
        game: 1816909,
        observed,
        kind,
        feeds: Feeds::replay(&backtest.archive).unwrap(),
    });
    backtest.feeds = Feeds {
        signal: inject.clone(),
        odds: inject.clone(),
        context: inject,
    };
    let (_a, audit) = fresh_audit().await;
    match backtest.run(&audit).await {
        Ok(_) => Ok(()),
        Err(e) => {
            let leak = e
                .downcast_ref::<TemporalLeakError>()
                .unwrap_or_else(|| panic!("{kind}: not a TemporalLeakError: {e}"));
            assert_eq!(leak.observed_at, observed, "{kind}");
            Err(e.to_string())
        }
    }
}

#[tokio::test]
async fn any_datum_observed_at_or_after_the_match_cutoff_aborts_the_run() {
    for kind in ["odds", "vrs", "liquipedia", "news"] {
        for offset in [Duration::zero(), Duration::seconds(1), Duration::hours(2)] {
            assert!(
                run_with_injected(kind, offset).await.is_err(),
                "{kind} at +{offset}"
            );
        }
        // The same datum two seconds before the cutoff is accepted.
        assert_eq!(
            run_with_injected(kind, -Duration::seconds(2)).await,
            Ok(()),
            "{kind}"
        );
    }
}

#[tokio::test]
async fn mid_stage_results_in_a_pickem_view_abort_the_run() {
    type Tamper = fn(&mut kallbet_ai::adapters::backtest::ReplayStage);
    let tampered: [(&str, Tamper); 4] = [
        ("a Round 1 winner", |s| {
            let opening = s.opening.as_mut().unwrap();
            let winner = opening.value["groups"][0]["matches"][0]["team_a"]["id"].clone();
            opening.value["groups"][0]["matches"][0]["winner"] = winner;
        }),
        ("a Round 2 pairing", |s| {
            let opening = s.opening.as_mut().unwrap();
            let mut round2 = opening.value["groups"][0].clone();
            round2["wins"] = json!(1);
            opening.value["groups"].as_array_mut().unwrap().push(round2);
        }),
        ("a Round 1 Match marked finished", |s| {
            let opening = s.opening.as_mut().unwrap();
            opening.value["groups"][0]["matches"][0]["status"] = json!("finished");
        }),
        ("a Stage revision saved at the Stage's start", |s| {
            let opening = s.opening.as_mut().unwrap();
            opening.observed_at = s.cutoff;
        }),
    ];
    for (what, tamper) in tampered {
        let dir = singapore();
        let mut backtest = Backtest::new(load(&dir), Clis::saved()).unwrap();
        backtest.simulations = 200;
        tamper(&mut backtest.archive.stages[0]);
        let (_a, audit) = fresh_audit().await;
        let err = backtest
            .run(&audit)
            .await
            .err()
            .unwrap_or_else(|| panic!("{what}: ran"));
        assert!(err.is::<TemporalLeakError>(), "{what}: {err}");
    }
}

fn plan_singapore(backtest: &mut Backtest, blind: bool) {
    let ids: std::collections::BTreeSet<i64> = backtest.archive.stages[0]
        .matches
        .iter()
        .map(|m| m.game.id)
        .collect();
    if blind {
        backtest.plan.blind = ids;
    } else {
        backtest.plan.direct = ids;
    }
}

#[tokio::test]
async fn backtest_cli_runs_have_no_tools_and_no_web_even_for_web_search_profiles() {
    let dir = singapore();
    let clis = Clis::saved();
    let mut backtest = Backtest::new(load(&dir), clis.clone()).unwrap();
    backtest.simulations = 200;
    backtest.llm = llm_config();
    chains(&mut backtest.plan, &["antigravity", "claude"]);
    plan_singapore(&mut backtest, false);
    backtest.plan.direct = backtest.plan.direct.iter().take(2).copied().collect();
    let (_a, audit) = fresh_audit().await;
    let outcome = backtest.run(&audit).await.unwrap();

    let started = clis.started();
    let programs: std::collections::BTreeSet<String> = started.iter().map(program_of).collect();
    assert_eq!(
        programs,
        ["agy", "claude", "codex"].map(String::from).into(),
        "every CLI ran"
    );
    let config: CliConfig = serde_json::from_value(json!({})).unwrap();
    for run in &started {
        let prompt = prompt_of(run);
        assert!(
            prompt.contains("Use no tools and no web access."),
            "{prompt}"
        );
        assert!(!prompt.contains("You may read web pages"));
        let cli = match program_of(run).as_str() {
            "claude" => Cli::Claude,
            "codex" => Cli::Codex,
            _ => Cli::Antigravity,
        };
        let schema: Value = serde_json::from_str(
            run.args
                .iter()
                .skip_while(|a| *a != "--json-schema")
                .nth(1)
                .map(String::as_str)
                .or(run.files.first().map(|f| f.1.as_str()))
                .unwrap(),
        )
        .unwrap();
        let _ = (&config, &schema);
        let args = &run.args;
        let has = |flag: &str| args.iter().any(|a| a == flag);
        let pair = |flag: &str, value: &str| args.windows(2).any(|w| w[0] == flag && w[1] == value);
        let joined = args.join(" ");
        // Nothing anywhere that grants the web.
        for never in ["WebFetch", "WebSearch(", "web_search=\"live\"", "allowed_domains", "read_url(hltv", "read_url(liquipedia", "--dangerously"] {
            assert!(!joined.contains(never), "{cli:?} ran with {never}: {joined}");
        }
        match cli {
            Cli::Claude => {
                assert!(pair("--tools", ""), "claude: no tool at all: {joined}");
                assert!(has("--strict-mcp-config") && has("--disable-slash-commands"), "{joined}");
                assert!(!has("--settings"), "claude: no permission settings to widen it");
            }
            Cli::Codex => {
                assert!(pair("--sandbox", "read-only"), "{joined}");
                assert!(pair("-c", "web_search=\"disabled\""), "{joined}");
                assert!(pair("-c", "mcp_servers={}"), "{joined}");
            }
            Cli::Antigravity => {
                assert!(has("--sandbox") && has("--disable-slash-commands"), "{joined}");
                let settings: Value = serde_json::from_str(
                    &args[args.iter().position(|a| a == "--settings").unwrap() + 1],
                )
                .unwrap();
                assert_eq!(settings["toolPermission"], "strict");
                assert_eq!(settings["permissions"]["allow"], json!([]));
                let deny = settings["permissions"]["deny"].as_array().unwrap();
                for tool in ["read_url(*)", "search_web(*)", "command(*)", "mcp(*)"] {
                    assert!(deny.contains(&json!(tool)), "agy must deny {tool}: {settings}");
                }
            }
        }
    }
    // Antigravity's saved run is its quota error: it was asked once, and the
    // Arbiter seat fell back to Claude for both Matches.
    assert_eq!(started.iter().filter(|r| program_of(r) == "agy").count(), 1);
    assert_eq!(by_bot(&outcome.predictions, DEBATE).len(), 2);
    assert_eq!(by_bot(&outcome.predictions, SINGLE).len(), 2);
    assert!(outcome.report.llm.exhausted["antigravity"].contains("quota"));
    assert_eq!(outcome.report.llm.stopped, None);
}

#[tokio::test]
async fn the_first_quota_error_stops_the_llm_track_and_the_benchmark_completes() {
    let dir = singapore();
    let clis = Clis::saved();
    let mut backtest = Backtest::new(load(&dir), clis.clone()).unwrap();
    backtest.simulations = 200;
    backtest.llm = llm_config();
    // Antigravity alone in the Arbiter seat: its quota error fails the debate.
    chains(&mut backtest.plan, &["antigravity"]);
    plan_singapore(&mut backtest, false);
    let (_a, audit) = fresh_audit().await;
    let outcome = backtest.run(&audit).await.unwrap();

    let agy_runs = clis
        .started()
        .iter()
        .filter(|r| program_of(r) == "agy")
        .count();
    assert_eq!(agy_runs, 1, "no retry after the quota error");
    let track = &outcome.report.llm;
    assert!(
        track.stopped.as_deref().unwrap().contains("quota"),
        "{:?}",
        track.stopped
    );
    // The first Matches' single agent answered before the debate failed; nothing after.
    assert!(by_bot(&outcome.predictions, SINGLE).len() <= 2);
    assert!(by_bot(&outcome.predictions, DEBATE).is_empty());
    assert_eq!(
        by_bot(&outcome.predictions, VRS).len(),
        24,
        "the rest went on"
    );
}

/// Real names a blind prompt must not carry: both teams as PandaScore and
/// Liquipedia write them, every player of the revision, the Event's title.
fn real_names(archive: &ReplayArchive) -> Vec<String> {
    let mut names = vec!["PGL".to_owned(), "Singapore".to_owned()];
    for team in &archive.stages[0].teams {
        names.push(team.name.clone());
    }
    let revision = archive.event_pages["PGL/2026/Singapore"].last().unwrap();
    let rosters = kallbet_ai::adapters::context::liquipedia::parse_revision_rosters(
        &revision.body,
        revision.timestamp + Duration::seconds(1),
    )
    .unwrap()
    .unwrap();
    for team in &rosters
        .at(revision.timestamp + Duration::seconds(1))
        .unwrap()
        .teams
    {
        names.push(team.team.clone());
        names.extend(team.active_players.clone());
        names.extend(team.coaches.iter().map(|c| c.player.clone()));
    }
    names.retain(|n| !n.trim().is_empty());
    names.sort();
    names.dedup();
    names
}

fn has_year(text: &str) -> Option<String> {
    let b = text.as_bytes();
    (0..b.len().saturating_sub(3)).find_map(|i| {
        let w = &b[i..i + 4];
        // Not part of a longer number such as a probability (0.1977).
        let alone = (i == 0 || !(b[i - 1].is_ascii_digit() || b[i - 1] == b'.'))
            && (i + 4 == b.len() || !b[i + 4].is_ascii_digit());
        (alone
            && w.iter().all(u8::is_ascii_digit)
            && (w.starts_with(b"19") || w.starts_with(b"20")))
        .then(|| text[i..i + 4].to_owned())
    })
}

/// `name` as a word of `text` (a JSON key such as `stand_ins` is one word).
fn word(text: &str, name: &str) -> bool {
    let (t, n) = (text.to_lowercase(), name.to_lowercase());
    let part = |c: char| c.is_alphanumeric() || c == '_';
    t.match_indices(&n).any(|(i, _)| {
        let before = t[..i].chars().next_back().is_none_or(|c| !part(c));
        let after = t[i + n.len()..].chars().next().is_none_or(|c| !part(c));
        before && after
    })
}

/// A news item for every dossier, naming in free text a team and a player
/// that no structured field of the dossier names (as a real headline would).
struct WithHeadline {
    inner: Arc<dyn ContextEnricher>,
    title: String,
}
#[async_trait]
impl ContextEnricher for WithHeadline {
    async fn facts(&self, game: &Match, cutoff: Instant) -> kallbet_ai::Result<Vec<AsOf<Value>>> {
        let mut facts = self.inner.facts(game, cutoff).await?;
        facts.push(AsOf::new(
            json!({"kind": "news_item", "source": "HLTV", "feed_url": "https://www.hltv.org/rss/news",
                   "title": self.title, "description": format!("{} (2026)", self.title),
                   "url": "https://www.hltv.org/news/1/x", "published_at": (cutoff - Duration::hours(30)).to_rfc3339(),
                   "matched_teams": [game.team_a.name]}),
            cutoff - Duration::hours(30),
            cutoff,
        )?);
        Ok(facts)
    }
}

/// A seat's brief written on the masked dossier (placeholders only), in the
/// real Claude envelope.
fn placeholder_brief(run: &Invocation) -> String {
    let brief = json!({
        "position": "TEAM_ALPHA is favoured by the baseline; PLAYER_03 is unchanged.",
        "claims": [{"claim": "No verified change for TEAM_BETA.", "kind": "none",
                    "evidence_url": "https://evidence.invalid/1", "favours": "team_a"}],
        "delta": 0
    });
    if program_of(run) == "codex" {
        // The real codex stream, its final agent message replaced.
        return saved("devil-codex.jsonl")
            .lines()
            .map(|line| {
                let mut event: Value = serde_json::from_str(line).unwrap();
                if event["type"] == "item.completed" && event["item"]["type"] == "agent_message" {
                    event["item"]["text"] = json!(brief.to_string());
                }
                event.to_string()
            })
            .collect::<Vec<_>>()
            .join("
");
    }
    let mut envelope: Value = serde_json::from_str(&saved("advocate-claude.json")).unwrap();
    envelope["structured_output"] = brief;
    envelope.to_string()
}

#[tokio::test]
async fn blind_prompts_carry_no_real_team_player_event_or_year() {
    let dir = singapore();
    // The seats answer what a model would on the masked dossier; the Arbiter
    // prompt carries their briefs, and is checked whole.
    let clis = Clis::with(Box::new(|run: &Invocation| {
        matches!(role_of(run).as_str(), "advocate" | "devil").then(|| placeholder_brief(run))
    }));
    let mut backtest = Backtest::new(load(&dir), clis.clone()).unwrap();
    // A headline naming a team and a player of the Stage that are not in
    // this Match (so no structured field of its dossier names them).
    let stage = &backtest.archive.stages[0];
    let third = stage.teams.last().unwrap().name.clone();
    let revision = backtest.archive.event_pages["PGL/2026/Singapore"].last().unwrap();
    let at = revision.timestamp + Duration::seconds(1);
    let rosters = kallbet_ai::adapters::context::liquipedia::parse_revision_rosters(&revision.body, at)
        .unwrap()
        .unwrap();
    let player = rosters.at(at).unwrap().teams.last().unwrap().active_players[0].clone();
    backtest.feeds.context = Arc::new(WithHeadline {
        inner: backtest.feeds.context.clone(),
        title: format!("{third} sign {player} after a strong year"),
    });
    backtest.simulations = 200;
    backtest.llm = llm_config();
    chains(&mut backtest.plan, &["claude"]);
    plan_singapore(&mut backtest, true);
    let (_a, audit) = fresh_audit().await;
    let outcome = backtest.run(&audit).await.unwrap();
    assert_eq!(by_bot(&outcome.predictions, DEBATE).len(), 24);

    let names = real_names(&backtest.archive);
    assert!(
        names
            .iter()
            .any(|n| n == "ZywOo" || n == "NiKo" || n == "s1mple" || n.len() > 3)
    );
    let started = clis.started();
    assert_eq!(
        started.len(),
        24 * 4,
        "one single agent and three seats per Match"
    );
    for run in &started {
        // The whole prompt, the Arbiter's attached briefs included.
        let prompt = prompt_of(run);
        let prompt = prompt.as_str();
        for name in names.iter().chain([&third, &player]) {
            assert!(!word(prompt, name), "{name:?} in a blind prompt");
        }
        assert_eq!(has_year(prompt), None, "a year in a blind prompt");
        for placeholder in [
            "TEAM_ALPHA",
            "TEAM_BETA",
            "PLAYER_01",
            "PLAYER_10",
            "https://evidence.invalid/1",
        ] {
            assert!(prompt.contains(placeholder), "{placeholder} missing");
        }
    }
}

#[tokio::test]
async fn a_blind_answer_is_mapped_back_to_the_real_winner_and_its_evidence() {
    let dir = singapore();
    // Claude's real single-agent envelope, its answer changed to back TEAM_BETA
    // with a stand-in cited from the masked dossier.
    let clis = Clis::with(Box::new(|run: &Invocation| {
        let prompt = prompt_of(run);
        let url = prompt
            .split("\"source_url\":\"")
            .nth(1)?
            .split('"')
            .next()?
            .to_owned();
        let mut envelope: Value = serde_json::from_str(&saved_cli("predict-claude.json")).ok()?;
        envelope["structured_output"] = json!({
            "delta": -8,
            "factors": [{"kind": "stand_in", "evidence_url": url}],
            "summary": {"en": "TEAM_BETA is favoured: TEAM_ALPHA fields a stand-in.",
                        "ru": "TEAM_BETA.", "uk": "TEAM_BETA."}
        });
        Some(envelope.to_string())
    }));
    let mut backtest = Backtest::new(load(&dir), clis.clone()).unwrap();
    backtest.simulations = 200;
    backtest.llm = llm_config();
    chains(&mut backtest.plan, &["claude"]);
    plan_singapore(&mut backtest, true);
    let (_a, audit) = fresh_audit().await;
    let outcome = backtest.run(&audit).await.unwrap();

    let stage = &backtest.archive.stages[0];
    let anchors: Vec<&Submitted> = by_bot(&outcome.predictions, "anchor");
    let mut flipped = 0;
    for p in by_bot(&outcome.predictions, SINGLE) {
        let game = &stage
            .matches
            .iter()
            .find(|m| m.game.id == p.match_id)
            .unwrap()
            .game;
        let anchor = anchors.iter().find(|a| a.match_id == p.match_id).unwrap();
        let p_team_a = |s: &Submitted| {
            let p = s.prediction.rationale["p_win"].as_f64().unwrap();
            if s.prediction.team_id == game.team_a.id {
                p
            } else {
                1.0 - p
            }
        };
        // Eight points toward the real team B, on the real dossier's anchor.
        assert!(
            (p_team_a(anchor) - 0.08 - p_team_a(p)).abs() < 1e-9 || p_team_a(p) == 0.01,
            "{} {} {}",
            p_team_a(anchor),
            p_team_a(p),
            p.prediction.rationale
        );
        let factor = &p.prediction.rationale["factors"][0];
        let url = factor["evidence_url"].as_str().unwrap();
        assert!(
            url.starts_with(
                "https://liquipedia.net/counterstrike/index.php?title=PGL/2026/Singapore&oldid="
            ),
            "{url}"
        );
        let summary = p.prediction.rationale["summary"]["en"].as_str().unwrap();
        assert!(
            summary.contains(&game.team_b.name) && !summary.contains("TEAM_"),
            "{summary}"
        );
        if anchor.prediction.team_id == game.team_a.id && p.prediction.team_id == game.team_b.id {
            flipped += 1;
        }
    }
    assert!(flipped > 0, "at least one pick moved to the real team B");

    let (masked, map) = BlindAnonymizer::mask(&Dossier {
        game: stage.matches[0].game.clone(),
        signal: None,
        market: None,
        context: vec![],
    });
    assert_eq!(
        map.team(&masked.game.team_b.name),
        Some(stage.matches[0].game.team_b.id)
    );
    assert_eq!(
        map.team(&masked.game.team_a.name),
        Some(stage.matches[0].game.team_a.id)
    );
}

#[tokio::test]
async fn recorded_answers_reproduce_the_llm_track_without_a_call() {
    let dir = singapore();
    let first = Clis::saved();
    let mut backtest = Backtest::new(load(&dir), first.clone()).unwrap();
    backtest.simulations = 200;
    backtest.llm = llm_config();
    chains(&mut backtest.plan, &["claude"]);
    plan_singapore(&mut backtest, false);
    let (_a, audit) = fresh_audit().await;
    let before = backtest.run(&audit).await.unwrap();
    let answers = kallbet_ai::adapters::backtest::export_answers(&audit)
        .await
        .unwrap();

    let again = Clis::saved();
    backtest.runner = again.clone();
    let (_b, fresh) = fresh_audit().await;
    kallbet_ai::adapters::backtest::import_answers(&fresh, &answers)
        .await
        .unwrap();
    let after = backtest.run(&fresh).await.unwrap();
    assert!(again.started().is_empty(), "no provider was called");
    assert_eq!(
        serde_json::to_value(&before.report.tables).unwrap(),
        serde_json::to_value(&after.report.tables).unwrap()
    );
}

#[test]
fn the_backtest_subcommand_writes_the_report() {
    let dir = archive_of(&["panda-22378", "lp-22378", "standings_global_2026_11_02"]);
    let out = tempfile::tempdir().unwrap();
    let run = std::process::Command::new(env!("CARGO_BIN_EXE_kallbet-ai"))
        .args(["backtest", "--archive"])
        .arg(dir.path())
        .arg("--out")
        .arg(out.path())
        .args(["--simulations", "200"])
        .env_remove("SIDECAR_CONFIG")
        .output()
        .unwrap();
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    let report = std::fs::read_to_string(out.path().join("report.md")).unwrap();
    assert!(
        report.contains("| PGL Major Singapore 2026 / Stage 1 | Swiss | no | 24 |"),
        "{report}"
    );
    let json: Value =
        serde_json::from_str(&std::fs::read_to_string(out.path().join("report.json")).unwrap())
            .unwrap();
    assert_eq!(json["stages"][0]["with_vrs"], 24);
    // The run is recorded in the audit file beside the report.
    let runs = tokio::runtime::Runtime::new().unwrap().block_on(async {
        let audit = Audit::open(&out.path().join("audit.sqlite")).await.unwrap();
        audit.backtest_runs().await.unwrap()
    });
    assert_eq!(runs.len(), 1);
    let run = &runs[0];
    assert_eq!(run.archive_sha256, json["archive_digest"].as_str().unwrap());
    assert_eq!(run.archive_files, 3);
    assert!(run.started_at <= run.finished_at);
    assert_eq!(run.profiles, json["profiles"]);
    assert_eq!(run.headline["predictions"]["vrs"], 24);
    assert_eq!(run.headline["stages"], 1);
    assert_eq!(run.headline["llm_calls"], serde_json::json!({}));

    let missing = std::process::Command::new(env!("CARGO_BIN_EXE_kallbet-ai"))
        .args(["backtest", "--archive", "no-such-directory"])
        .output()
        .unwrap();
    assert!(!missing.status.success());
    assert!(String::from_utf8_lossy(&missing.stderr).contains("no-such-directory"));
}

#[tokio::test]
async fn every_saved_major_stage_replays_without_a_leak() {
    let archive = ReplayArchive::load(&fixtures()).unwrap();
    assert_eq!(archive.stages.len(), 19);
    for stage in &archive.stages {
        assert!(
            stage.opening.is_some(),
            "{}: {:?}",
            stage.title(),
            stage.opening_problem
        );
        assert!(
            stage.opening.as_ref().unwrap().observed_at < stage.cutoff,
            "{}",
            stage.title()
        );
    }
    let mut backtest = Backtest::new(archive, Clis::saved()).unwrap();
    backtest.simulations = 100;
    let (_a, audit) = fresh_audit().await;
    let outcome = backtest.run(&audit).await.unwrap();
    // 488 scored Matches; Copenhagen 2024 (73) predates Valve's global standings.
    assert_eq!(by_bot(&outcome.predictions, VRS).len(), 488 - 73);
    assert_eq!(
        outcome
            .report
            .pickem
            .iter()
            .filter(|p| p.status == "final")
            .count(),
        18
    );
}
