//! Seam 1 of ticket #43: the Signal Lab's LLM track over the saved replay
//! archive (`tests/fixtures/backtest/`). No provider is called: the CLI runner
//! is scripted and answers with what the real CLIs printed
//! (`tests/fixtures/debate/`), including Antigravity's saved quota error.
use async_trait::async_trait;
use chrono::Duration;
use kallbet_ai::{
    adapters::{
        backtest::ReplayArchive,
        lab::{
            Lab,
            llm::{self, Recorded, Seating, Setup, Track},
            stats::{self, Verdict},
        },
        llm::{Finished, Invocation, LlmConfig, ProcessRunner, RunError},
    },
    *,
};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/backtest")
}
fn whole() -> ReplayArchive {
    ReplayArchive::load(&fixtures()).unwrap()
}
fn saved(name: &str) -> String {
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

/// The CLIs: every run recorded, each answered with what the real CLI printed
/// for that seat; `agy` with its saved quota error. With `masked_briefs` the
/// two Advocates answer what a model would on a masked dossier (placeholders
/// only), so the Arbiter's prompt, which attaches their briefs, can be checked
/// whole.
#[derive(Default)]
struct Clis {
    started: Mutex<Vec<Invocation>>,
    masked_briefs: bool,
}
impl Clis {
    fn started(&self) -> Vec<Invocation> {
        self.started.lock().unwrap().clone()
    }
    fn runs_of(&self, program: &str) -> usize {
        self.started()
            .iter()
            .filter(|r| program_of(r) == program)
            .count()
    }
}
#[async_trait]
impl ProcessRunner for Clis {
    async fn run(&self, invocation: &Invocation) -> Result<Finished, RunError> {
        self.started.lock().unwrap().push(invocation.clone());
        let (seat, cli) = (role_of(invocation), program_of(invocation));
        if self.masked_briefs && (seat == "advocate" || seat == "devil") {
            return Ok(Finished {
                code: Some(0),
                stdout: placeholder_brief(invocation),
                stderr: String::new(),
            });
        }
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
            code: Some(if stderr.is_empty() { 0 } else { 3 }),
            stdout,
            stderr,
        })
    }
}

/// A seat's brief written on the masked dossier, in the real CLI envelope.
fn placeholder_brief(run: &Invocation) -> String {
    let brief = json!({
        "position": "TEAM_ALPHA is favoured by the baseline; PLAYER_03 is unchanged.",
        "claims": [{"claim": "No verified change for TEAM_BETA.", "kind": "none",
                    "evidence_url": "https://evidence.invalid/1", "favours": "team_a"}],
        "delta": 0
    });
    if program_of(run) == "codex" {
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
            .join("\n");
    }
    let mut envelope: Value = serde_json::from_str(&saved("advocate-claude.json")).unwrap();
    envelope["structured_output"] = brief;
    envelope.to_string()
}

/// All three subscriptions, and both LLM profiles listed as allowed to search
/// the web in live runs.
fn llm_config() -> LlmConfig {
    serde_json::from_value(json!({
        "providers": {
            "claude": {"kind": "claude"},
            "codex": {"kind": "codex"},
            "antigravity": {"kind": "antigravity"}
        },
        "web_search_bots": ["single", "debate"]
    }))
    .unwrap()
}
fn seating(name: &str, advocate: &str, devil: &str, arbiter: &str) -> Seating {
    Seating {
        name: name.into(),
        advocate: vec![advocate.into()],
        devil: vec![devil.into()],
        arbiter: vec![arbiter.into()],
    }
}
fn budget(entries: &[(&str, usize)]) -> Option<std::collections::BTreeMap<String, usize>> {
    Some(entries.iter().map(|(n, b)| ((*n).into(), *b)).collect())
}
fn setup(work: &tempfile::TempDir, seatings: Vec<Seating>) -> Setup {
    Setup {
        llm: llm_config(),
        single: vec!["claude".into()],
        seatings,
        budget: budget(&[
            ("claude", 10_000),
            ("codex", 10_000),
            ("antigravity", 10_000),
        ]),
        recorded: Recorded::default(),
        training_cutoff: llm::Options::default().training_cutoff,
        work_dir: work.path().to_path_buf(),
    }
}
/// The two-seating track over the saved archive with the scripted CLIs, run
/// once for the tests that only read it, with how many CLI runs it started.
fn shared_track() -> &'static (Track, usize) {
    static TRACK: std::sync::OnceLock<(Track, usize)> = std::sync::OnceLock::new();
    TRACK.get_or_init(|| {
        std::thread::spawn(|| {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let work = tempfile::tempdir().unwrap().keep();
                let clis = Arc::new(Clis::default());
                let s = Setup {
                    work_dir: work,
                    ..setup(&tempfile::tempdir().unwrap(), two_seatings())
                };
                let track = llm::run(&whole(), &s, clis.clone()).await.unwrap();
                (track, clis.started().len())
            })
        })
        .join()
        .unwrap()
    })
}

fn two_seatings() -> Vec<Seating> {
    vec![
        seating("claude-codex-claude", "claude", "codex", "claude"),
        seating("claude-only", "claude", "claude", "claude"),
    ]
}

#[tokio::test]
async fn the_flagged_matches_are_those_with_a_pre_cutoff_stand_in_and_old_events_go_blind() {
    let archive = whole();
    let flagged = llm::flagged(&archive, llm::Options::default().training_cutoff)
        .await
        .unwrap();
    // Shanghai 2024 and Cologne 2026 (before the training cutoff) and
    // PGL Singapore 2026 Stage 1 (after it).
    assert_eq!(flagged.len(), 57);
    assert_eq!(flagged.iter().filter(|f| f.blind).count(), 54);
    let direct: BTreeSet<i64> = flagged
        .iter()
        .filter(|f| !f.blind)
        .map(|f| f.event_id)
        .collect();
    assert_eq!(direct, [11176].into(), "Singapore only");
    // The NAVI/Falcons Match of Cologne had a stand-in; Vitality/FUT had none.
    assert!(flagged.iter().any(|f| f.match_id == 1513148));
    assert!(!flagged.iter().any(|f| f.match_id == 1513120));
    // Everything blind when the cutoff is later than every Event.
    let later = "2100-01-01T00:00:00Z".parse().unwrap();
    assert!(
        llm::flagged(&archive, later)
            .await
            .unwrap()
            .iter()
            .all(|f| f.blind)
    );
}

#[tokio::test]
async fn a_run_without_a_budget_refuses_to_call_any_provider() {
    let archive = whole();
    let work = tempfile::tempdir().unwrap();
    let clis = Arc::new(Clis::default());
    let mut s = setup(&work, two_seatings());
    s.budget = None;
    let Err(e) = llm::run(&archive, &s, clis.clone()).await else {
        panic!("a run without a budget went ahead");
    };
    assert!(e.to_string().contains("--budget"), "{e}");
    // One subscription without a budget is refused too.
    s.budget = budget(&[("claude", 5), ("codex", 5)]);
    let Err(e) = llm::run(&archive, &s, clis.clone()).await else {
        panic!("a run with an unbudgeted provider went ahead");
    };
    assert!(e.to_string().contains("antigravity"), "{e}");
    assert!(clis.started().is_empty(), "no CLI was started");

    // The subcommand refuses before reading anything else.
    let file = work.path().join("llm.json");
    std::fs::write(
        &file,
        json!({"llm": llm_config_json(), "single": ["claude"],
               "seatings": [{"name": "s", "advocate": ["claude"], "devil": ["codex"], "arbiter": ["claude"]}]})
        .to_string(),
    )
    .unwrap();
    let out = tempfile::tempdir().unwrap();
    let run = run_lab(out.path(), &["--llm", file.to_str().unwrap()]);
    assert!(!run.status.success());
    assert!(
        String::from_utf8_lossy(&run.stderr).contains("--budget"),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
}

fn llm_config_json() -> Value {
    json!({"providers": {"claude": {"kind": "claude"}, "codex": {"kind": "codex"},
                         "antigravity": {"kind": "antigravity"}}})
}

#[tokio::test]
async fn a_spent_budget_stops_calling_that_subscription_and_each_ones_runs_are_reported() {
    let archive = whole();
    let work = tempfile::tempdir().unwrap();
    let clis = Arc::new(Clis::default());
    let mut s = setup(&work, two_seatings());
    // Codex sits only as the first seating's Devil's Advocate: three runs.
    s.budget = budget(&[("claude", 10_000), ("codex", 3), ("antigravity", 0)]);
    let track = llm::run(&archive, &s, clis.clone()).await.unwrap();

    assert_eq!(
        clis.runs_of("codex"),
        3,
        "not one codex run past its budget"
    );
    assert_eq!(clis.runs_of("agy"), 0);
    let row = |name: &str| {
        track
            .budget
            .iter()
            .find(|b| b.subscription == name)
            .unwrap()
    };
    assert_eq!((row("codex").budget, row("codex").used), (3, 3));
    assert!(row("codex").refused >= 1, "{:?}", track.budget);
    assert_eq!(row("claude").used, clis.runs_of("claude"));
    assert_eq!(
        (row("antigravity").used, row("antigravity").refused),
        (0, 0)
    );

    // The first seating stopped on its spent budget; the second, which needs
    // no codex, answered every flagged Match; the track itself did not stop.
    let first = &track.seatings[0];
    assert!(
        first.stopped.as_deref().unwrap().contains("budget"),
        "{first:?}"
    );
    assert!(first.answered < 57, "{first:?}");
    assert_eq!(track.seatings[1].answered, 57, "{:?}", track.seatings[1]);
    assert_eq!(track.seatings[1].stopped, None);
    assert_eq!(track.stopped, None);
    assert_eq!(track.single.len(), 57);

    let report = compare(&archive, &track).await;
    let md = report.markdown();
    assert!(md.contains("| codex | 3 | 3 |"), "{md}");
    assert!(
        report
            .missing
            .iter()
            .any(|m| m.contains("claude-codex-claude")),
        "{:?}",
        report.missing
    );
}

async fn compare(archive: &ReplayArchive, track: &Track) -> llm::TrackReport {
    let _ = archive;
    let mut lab = Lab::new(whole()).unwrap();
    lab.protocol.resamples = 300;
    llm::compare(&mut lab, track, llm::command(None, None, &[]))
        .await
        .unwrap()
}

#[tokio::test]
async fn the_first_quota_error_stops_the_llm_track_and_the_rest_of_the_lab_completes() {
    let archive = whole();
    let work = tempfile::tempdir().unwrap();
    let clis = Arc::new(Clis::default());
    // Antigravity alone in the first seating's Arbiter seat: its saved quota error.
    let s = setup(
        &work,
        vec![
            seating("agy-arbiter", "claude", "codex", "antigravity"),
            seating("claude-only", "claude", "claude", "claude"),
        ],
    );
    let track = llm::run(&archive, &s, clis.clone()).await.unwrap();
    assert_eq!(clis.runs_of("agy"), 1, "no retry after the quota error");
    assert!(
        track.stopped.as_deref().unwrap().contains("quota"),
        "{:?}",
        track.stopped
    );
    assert!(!track.seatings[1].ran, "nothing asked after it");
    assert!(
        track.seatings[1]
            .stopped
            .as_deref()
            .unwrap()
            .contains("quota")
    );

    // The rest of the lab: every Candidate Signal evaluated; the adjustment
    // has no answer and says so.
    let mut lab = Lab::new(whole()).unwrap();
    lab.protocol.resamples = 300;
    lab.signals = vec![
        kallbet_ai::adapters::lab::signals::by_name("rating_calibration").unwrap(),
        Arc::new(track.adjustment()),
    ];
    let outcome = lab.run().await.unwrap();
    let names: Vec<_> = outcome
        .report
        .candidates
        .iter()
        .map(|c| c.name.as_str())
        .collect();
    assert!(
        names.contains(&"rating_calibration") && names.contains(&llm::NAME),
        "{names:?}"
    );
    let adjustment = outcome
        .report
        .candidates
        .iter()
        .find(|c| c.name == llm::NAME)
        .unwrap();
    assert_eq!(adjustment.verdict, Verdict::NotTestable);

    let report = llm::compare(
        &mut lab,
        &track,
        llm::command(
            Some(Path::new("llm.json")),
            Some(Path::new("answers.json")),
            &["antigravity".into(), "claude".into(), "codex".into()],
        ),
    )
    .await
    .unwrap();
    let md = report.markdown();
    assert!(md.contains("The LLM track stopped at first"), "{md}");
    assert!(
        md.contains("seating `claude-only`: 57 of 57 flagged Matches have no answer"),
        "{md}"
    );
    assert!(
        md.contains(
            "cargo run -p kallbet-ai -- lab --archive <ARCHIVE> --llm llm.json --answers answers.json \
             --budget antigravity=<RUNS> --budget claude=<RUNS> --budget codex=<RUNS>"
        ),
        "{md}"
    );
}

#[tokio::test]
async fn recorded_answers_reproduce_the_tables_without_a_provider_call() {
    let archive = whole();
    let (track, started) = shared_track();
    assert!(*started > 0);
    let recorded: Recorded =
        serde_json::from_str(&serde_json::to_string(&track.recorded).unwrap()).unwrap();

    // No provider configured, no budget: only the recorded answers.
    let again = Arc::new(Clis::default());
    let rerun = Setup {
        llm: LlmConfig::default(),
        single: vec![],
        seatings: vec![],
        budget: None,
        recorded,
        training_cutoff: llm::Options::default().training_cutoff,
        work_dir: tempfile::tempdir().unwrap().keep(),
    };
    let replayed = llm::run(&archive, &rerun, again.clone()).await.unwrap();
    assert!(again.started().is_empty(), "no provider was called");
    assert_eq!(&replayed.single, &track.single);
    assert_eq!(&replayed.debate, &track.debate);

    let (a, b) = (
        compare(&archive, track).await,
        compare(&archive, &replayed).await,
    );
    assert_eq!(
        serde_json::to_value(&a.profiles).unwrap(),
        serde_json::to_value(&b.profiles).unwrap()
    );
    assert_eq!(a.debate_vs_single, b.debate_vs_single);
}

#[tokio::test]
async fn the_report_compares_single_debate_and_anchor_with_intervals_and_judges_the_adjustment() {
    let (track, _) = shared_track();
    assert_eq!(track.single.len(), 57);
    assert!(track.seatings.iter().all(|s| s.answered == 57 && s.ran));

    let mut lab = Lab::new(whole()).unwrap();
    lab.protocol.resamples = 300;
    let report = llm::compare(&mut lab, track, String::new()).await.unwrap();
    let names: Vec<_> = report.profiles.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(
        names,
        [
            "llm_single",
            "llm_debate_claude-codex-claude",
            "llm_debate_claude-only"
        ]
    );
    for c in &report.profiles {
        assert!(c.paired > 0 && c.paired <= 57, "{}: {}", c.name, c.paired);
        let interval = c.interval.expect("an interval");
        assert!(interval.low <= interval.high);
        // The verdict is the protocol's rule on these numbers.
        let (verdict, _) = stats::verdict(
            c.paired,
            interval,
            c.market.as_ref().map(|m| m.log_loss_difference),
            &lab.protocol,
        );
        assert_eq!(c.verdict, verdict, "{}", c.name);
    }
    for p in &report.debate_vs_single {
        assert_eq!(p.paired, 57);
        assert!(p.interval.is_some() && p.log_loss_difference.is_some());
    }
    assert!(report.missing.is_empty(), "{:?}", report.missing);

    // `llm_adjustment` in the lab's own table is the first seating's debate,
    // judged by the same runner.
    lab.signals = vec![Arc::new(track.adjustment())];
    let outcome = lab.run().await.unwrap();
    let adjustment = outcome
        .report
        .candidates
        .iter()
        .find(|c| c.name == llm::NAME)
        .unwrap();
    let debate = &report.profiles[1];
    assert_eq!(
        (
            adjustment.paired,
            adjustment.difference,
            adjustment.interval,
            adjustment.verdict
        ),
        (
            debate.paired,
            debate.difference,
            debate.interval,
            debate.verdict
        )
    );

    let md = report.markdown();
    for needle in [
        "## Against the anchor",
        "`llm_single`",
        "`llm_debate_claude-only`",
        "## Debate against the single agent",
        "## Seatings",
        "## Budget",
        "Nothing: every flagged Match has an answer",
    ] {
        assert!(md.contains(needle), "{needle}: {md}");
    }
}

/// `name` as a word of `text`.
fn word(text: &str, name: &str) -> bool {
    let (t, n) = (text.to_lowercase(), name.to_lowercase());
    let part = |c: char| c.is_alphanumeric() || c == '_';
    t.match_indices(&n).any(|(i, _)| {
        let before = t[..i].chars().next_back().is_none_or(|c| !part(c));
        let after = t[i + n.len()..].chars().next().is_none_or(|c| !part(c));
        before && after
    })
}
fn has_year(text: &str) -> Option<String> {
    let b = text.as_bytes();
    (0..b.len().saturating_sub(3)).find_map(|i| {
        let w = &b[i..i + 4];
        let alone = (i == 0 || !(b[i - 1].is_ascii_digit() || b[i - 1] == b'.'))
            && (i + 4 == b.len() || !b[i + 4].is_ascii_digit());
        (alone
            && w.iter().all(u8::is_ascii_digit)
            && (w.starts_with(b"19") || w.starts_with(b"20")))
        .then(|| text[i..i + 4].to_owned())
    })
}

#[tokio::test]
async fn every_cli_run_has_no_tools_and_pre_cutoff_prompts_hold_no_real_names() {
    let archive = whole();
    let work = tempfile::tempdir().unwrap();
    let clis = Arc::new(Clis {
        masked_briefs: true,
        ..Clis::default()
    });
    let s = setup(
        &work,
        vec![seating("all-three", "claude", "codex", "claude")],
    );
    let track = llm::run(&archive, &s, clis.clone()).await.unwrap();
    assert_eq!(track.seatings[0].answered, 57);
    let started = clis.started();
    assert_eq!(
        started.len(),
        57 * 4,
        "one single agent and three seats per Match"
    );

    // No tools and no web, although both LLM profiles are web-search bots.
    for run in &started {
        let prompt = prompt_of(run);
        assert!(
            prompt.contains("Use no tools and no web access."),
            "{prompt}"
        );
        assert!(!prompt.contains("You may read web pages"));
        let args = &run.args;
        let joined = args.join(" ");
        let pair = |flag: &str, value: &str| args.windows(2).any(|w| w[0] == flag && w[1] == value);
        for never in [
            "WebFetch",
            "WebSearch(",
            "web_search=\"live\"",
            "allowed_domains",
            "--dangerously",
        ] {
            assert!(!joined.contains(never), "{never}: {joined}");
        }
        match program_of(run).as_str() {
            "claude" => assert!(pair("--tools", ""), "{joined}"),
            "codex" => {
                assert!(pair("--sandbox", "read-only"), "{joined}");
                assert!(pair("-c", "web_search=\"disabled\""), "{joined}");
            }
            other => panic!("unexpected CLI {other}"),
        }
    }

    // The blind Matches' prompts (Shanghai 2024, Cologne 2026) hold no real
    // team, player, coach, Event title or year.
    let blind_events: BTreeSet<i64> = track
        .flagged
        .iter()
        .filter(|f| f.blind)
        .map(|f| f.event_id)
        .collect();
    let mut names: BTreeSet<String> = BTreeSet::new();
    for stage in archive
        .stages
        .iter()
        .filter(|s| blind_events.contains(&s.event_id))
    {
        names.extend(stage.teams.iter().map(|t| t.name.clone()));
        names.insert(stage.event.clone());
    }
    for event in &blind_events {
        let title = &archive.event_titles[event];
        names.extend(
            title
                .split('/')
                .filter(|s| s.parse::<u32>().is_err() && *s != "Major")
                .map(|s| s.replace('_', " ")),
        );
        for revision in &archive.event_pages[title] {
            let at = revision.timestamp + Duration::seconds(1);
            let Ok(Some(rosters)) =
                kallbet_ai::adapters::context::liquipedia::parse_revision_rosters(
                    &revision.body,
                    at,
                )
            else {
                continue;
            };
            for team in &rosters.at(at).unwrap().teams {
                names.insert(team.team.clone());
                names.extend(team.active_players.iter().cloned());
                names.extend(team.coaches.iter().map(|c| c.player.clone()));
            }
        }
    }
    names.retain(|n| !n.trim().is_empty());
    assert!(names.len() > 100, "{}", names.len());
    let blind_runs: Vec<String> = started
        .iter()
        .map(prompt_of)
        .filter(|p| p.contains("TEAM_ALPHA"))
        .collect();
    assert_eq!(
        blind_runs.len(),
        54 * 4,
        "every run of a blind Match is masked"
    );
    for prompt in &blind_runs {
        for name in &names {
            assert!(!word(prompt, name), "{name:?} in a blind prompt");
        }
        assert_eq!(has_year(prompt), None, "a year in a blind prompt");
    }
}

fn run_lab(out: &Path, extra: &[&str]) -> std::process::Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_kallbet-ai"))
        .arg("lab")
        .arg("--archive")
        .arg(fixtures())
        .arg("--out")
        .arg(out)
        // The catch-up and Pick'em simulations are not what these tests check.
        .args(["--catch-up-trials", "8", "--pickem-simulations", "20"])
        .args(extra)
        .env_remove("SIDECAR_CONFIG")
        .env_remove("ODDSPAPI_API_KEY")
        .output()
        .unwrap()
}

#[tokio::test]
async fn the_lab_subcommand_replays_recorded_answers_and_writes_the_llm_report() {
    let (track, _) = shared_track();
    let work = tempfile::tempdir().unwrap();
    let answers = work.path().join("answers.json");
    std::fs::write(&answers, serde_json::to_string(&track.recorded).unwrap()).unwrap();
    let out = tempfile::tempdir().unwrap();
    let run = run_lab(
        out.path(),
        &[
            "--resamples",
            "200",
            "--signal",
            llm::NAME,
            "--answers",
            answers.to_str().unwrap(),
        ],
    );
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    let read = |f: &str| std::fs::read_to_string(out.path().join(f)).unwrap();
    let md = read("llm.md");
    assert!(md.contains("`llm_debate_claude-only`"), "{md}");
    assert!(
        md.contains("No provider was configured: no CLI was run."),
        "{md}"
    );
    let report = read("report.md");
    assert!(
        report.contains("llm_adjustment") && report.contains("(llm.md)"),
        "{report}"
    );
    let json: Value = serde_json::from_str(&read("report.json")).unwrap();
    let adjustment = json["candidates"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == llm::NAME)
        .unwrap();
    assert!(adjustment["paired"].as_u64().unwrap() > 0, "{adjustment}");
    // The answers file was rewritten with the same answers.
    let back: Recorded = serde_json::from_str(&std::fs::read_to_string(&answers).unwrap()).unwrap();
    assert_eq!(back, track.recorded);

    // Without --llm or --answers the track is not run and says how to run it.
    let plain = tempfile::tempdir().unwrap();
    let run = run_lab(plain.path(), &["--resamples", "100", "--signal", llm::NAME]);
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    let md = std::fs::read_to_string(plain.path().join("llm.md")).unwrap();
    assert!(md.contains("--budget claude=<RUNS>"), "{md}");
}

#[tokio::test]
async fn the_example_llm_file_loads_and_a_zero_budget_starts_no_cli() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../docs/lab/llm.example.json");
    let file: llm::LlmFile = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(file.seatings.len(), 3);
    let work = tempfile::tempdir().unwrap();
    let clis = Arc::new(Clis::default());
    let s = Setup {
        llm: file.llm,
        single: file.single,
        seatings: file.seatings,
        budget: budget(&[("claude", 0), ("codex", 0), ("antigravity", 0)]),
        recorded: Recorded::default(),
        training_cutoff: llm::Options::default().training_cutoff,
        work_dir: work.path().to_path_buf(),
    };
    let track = llm::run(&whole(), &s, clis.clone()).await.unwrap();
    assert!(clis.started().is_empty());
    assert!(track.budget.iter().all(|b| b.used == 0), "{:?}", track.budget);
    assert!(track.budget.iter().any(|b| b.refused > 0), "{:?}", track.budget);
    assert!(track.seatings.iter().all(|s| s.answered == 0));
}

/// The replay's Valve rating, failing for the Matches in `broken`.
struct BrokenFor {
    inner: Arc<dyn SignalEstimator>,
    broken: BTreeSet<i64>,
}
#[async_trait]
impl SignalEstimator for BrokenFor {
    async fn signal(&self, game: &Match, cutoff: Instant) -> Result<Option<AsOf<Signal>>> {
        if self.broken.contains(&game.id) {
            return Err("the rating port is down for this Match".into());
        }
        self.inner.signal(game, cutoff).await
    }
}

// Finding 6 (review of 6eea194): the LLM comparison is a training-side
// experiment. It does not evaluate the Holdout (a port failing for a
// Holdout Match does not stop it), and a Holdout answer changes no seating
// comparison.
#[tokio::test]
async fn the_llm_comparison_reads_no_holdout_match() {
    let archive = whole();
    let cologne = archive
        .stages
        .iter()
        .filter(|s| s.event.contains("Cologne"))
        .map(|s| s.cutoff)
        .min()
        .unwrap();
    let pick = |needle: &str| {
        let s = archive.stages.iter().find(|s| s.event.contains(needle)).unwrap();
        (s.matches[0].game.id, s.event_id)
    };
    let (train, train_event) = pick("Copenhagen");
    let (held, held_event) = pick("Cologne");
    let seating = seating("s", "claude", "claude", "claude");
    let track = |held_debate: f64| Track {
        training_cutoff: "2026-11-01T00:00:00Z".parse().unwrap(),
        flagged: vec![
            llm::Flagged { match_id: train, event_id: train_event, blind: true },
            llm::Flagged { match_id: held, event_id: held_event, blind: false },
        ],
        single_chain: vec!["claude".into()],
        seatings: vec![llm::SeatingRun {
            seating: seating.clone(),
            ran: true,
            answered: 2,
            calls: 0,
            stopped: None,
        }],
        single: [(train, 0.6), (held, 0.6)].into(),
        debate: [("s".to_string(), [(train, 0.7), (held, held_debate)].into())].into(),
        budget: vec![],
        stopped: None,
        recorded: Recorded::default(),
    };
    let lab = || {
        let mut lab = Lab::new(whole()).unwrap();
        lab.holdout = kallbet_ai::adapters::lab::Holdout {
            from: cologne,
            training_stages: vec![],
        };
        lab.protocol.resamples = 100;
        let broken = lab.holdout_matches(&lab.holdout_stages());
        let replay = kallbet_ai::adapters::backtest::Feeds::replay(&lab.archive).unwrap();
        lab.feeds = kallbet_ai::adapters::backtest::Feeds {
            signal: Arc::new(BrokenFor {
                inner: replay.signal.clone(),
                broken,
            }),
            ..replay
        };
        lab
    };
    let a = llm::compare(&mut lab(), &track(0.1), String::new()).await.unwrap();
    let b = llm::compare(&mut lab(), &track(0.9), String::new()).await.unwrap();
    assert_eq!(a.debate_vs_single[0].paired, 1);
    assert_eq!(a.debate_vs_single, b.debate_vs_single);
    for c in &a.profiles {
        assert!(c.paired <= 1, "{}: {}", c.name, c.paired);
    }
}

// Finding 8 (review of 6eea194): the first real quota failure stops every
// provider call of the track, also within the seating where it happens.
// Antigravity's quota error is in the Advocate seat: no Arbiter is asked
// after it, and nothing else is started but a seat already in flight.
#[tokio::test]
async fn a_quota_error_in_a_non_final_seat_stops_every_later_call() {
    let archive = whole();
    let work = tempfile::tempdir().unwrap();
    let clis = Arc::new(Clis::default());
    let s = setup(
        &work,
        vec![
            seating("agy-advocate", "antigravity", "codex", "claude"),
            seating("claude-only", "claude", "claude", "claude"),
        ],
    );
    let track = llm::run(&archive, &s, clis.clone()).await.unwrap();
    let started = clis.started();
    let quota = started
        .iter()
        .position(|r| program_of(r) == "agy")
        .expect("the Advocate was asked");
    let after: Vec<_> = started[quota + 1..].iter().map(|r| (program_of(r), role_of(r))).collect();
    assert!(after.iter().all(|(_, role)| role != "arbiter"), "{after:?}");
    assert!(after.len() <= 1, "only a seat already in flight: {after:?}");
    assert_eq!(clis.runs_of("agy"), 1);
    assert!(track.stopped.as_deref().unwrap().contains("quota"), "{:?}", track.stopped);
    assert!(!track.seatings[1].ran);
}

// Finding 16 (review of 6eea194): a run that calls providers names the
// archive it spends them on; the default is the small test archive.
#[test]
fn calling_providers_needs_an_explicit_archive() {
    let args = |a: &[&str]| a.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    let e = kallbet_ai::adapters::lab::cli::parse(&args(&["--llm", "llm.json", "--budget", "claude=5"]))
        .err()
        .expect("refused");
    assert!(e.to_string().contains("--archive"), "{e}");
    assert!(
        kallbet_ai::adapters::lab::cli::parse(&args(&[
            "--archive", "research", "--llm", "llm.json", "--budget", "claude=5"
        ]))
        .is_ok()
    );
    // Recorded answers alone call nothing: the default archive is fine.
    assert!(kallbet_ai::adapters::lab::cli::parse(&args(&["--answers", "a.json"])).is_ok());
}
