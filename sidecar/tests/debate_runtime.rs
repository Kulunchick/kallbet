//! `TieredDebateLlm` behind the `LlmRuntime` port, over three scripted providers
//! with real breakers and a real audit file: which pass a run gets, the order of
//! the seats, failover inside a seat's chain, and what the audit keeps.
use async_trait::async_trait;
use chrono::Duration;
use kallbet_ai::{
    adapters::{
        context::{liquipedia, rss},
        debate::{self, DebateConfig, DebateTurn, RiskFlag, risk_flags},
        llm::{
            BreakerState, Finished, Invocation, LlmConfig, Mode, ProcessRunner, Providers, RunError,
        },
    },
    *,
};
use serde_json::{Value, json};
use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
};

const SINGAPORE_REV: &str =
    include_str!("fixtures/context/liquipedia/pgl-singapore-2026-rev-before-start.json");
const HLTV_RSS: &str = include_str!("fixtures/context/rss/hltv-news.xml");

fn now() -> Instant {
    "2026-11-24T00:00:00Z".parse().unwrap()
}
struct Fixed;
impl Clock for Fixed {
    fn now(&self) -> Instant {
        now()
    }
}

/// What the three scripted providers share: the order of events, the prompts
/// they were given, and the failures still to play, per provider.
#[derive(Default)]
struct World {
    events: Mutex<Vec<String>>,
    prompts: Mutex<Vec<(String, RolePrompt)>>,
    failures: Mutex<HashMap<&'static str, VecDeque<LlmError>>>,
    /// The factors the predict and arbiter seats answer with (none by default).
    factors: Mutex<Vec<Value>>,
}
impl World {
    fn fail(&self, provider: &'static str, errors: &[LlmError]) {
        self.failures
            .lock()
            .unwrap()
            .insert(provider, errors.iter().copied().collect());
    }
    /// `provider:role` of every call, in the order the calls started.
    fn calls(&self) -> Vec<String> {
        self.events
            .lock()
            .unwrap()
            .iter()
            .filter_map(|e| e.strip_suffix(":start").map(str::to_owned))
            .collect()
    }
    fn prompt_of(&self, role: &str) -> RolePrompt {
        let prompts = self.prompts.lock().unwrap();
        prompts
            .iter()
            .find(|(_, p)| p.role == role)
            .unwrap()
            .1
            .clone()
    }
}
struct Scripted {
    name: &'static str,
    world: Arc<World>,
}
#[async_trait]
impl LlmRuntime for Scripted {
    async fn evaluate(
        &self,
        _: &BotProfile,
        _: &Dossier,
        _: Tier,
        prompt: &RolePrompt,
    ) -> Result<Option<Value>, LlmError> {
        let (name, role) = (self.name, prompt.role.as_str());
        self.world
            .events
            .lock()
            .unwrap()
            .push(format!("{name}:{role}:start"));
        self.world
            .prompts
            .lock()
            .unwrap()
            .push((name.into(), prompt.clone()));
        // Gives a seat running beside this one the chance to start.
        tokio::task::yield_now().await;
        self.world
            .events
            .lock()
            .unwrap()
            .push(format!("{name}:{role}:end"));
        let failure = self
            .world
            .failures
            .lock()
            .unwrap()
            .get_mut(name)
            .and_then(VecDeque::pop_front);
        if let Some(error) = failure {
            return Err(error);
        }
        Ok(Some(match role {
            "advocate" | "devil" => json!({
                "position": format!("{role} brief by {name}"),
                "claims": [],
                "delta": 0
            }),
            _ => json!({
                "delta": -4,
                "factors": *self.world.factors.lock().unwrap(),
                "summary": {"en": format!("{role} by {name}"), "ru": "ру", "uk": "ук"}
            }),
        }))
    }
}

struct Rig {
    world: Arc<World>,
    providers: Arc<Providers>,
    llm: Arc<dyn LlmRuntime>,
    audit: Audit,
    _dir: tempfile::TempDir,
}
async fn rig(config: Value) -> Rig {
    let world = Arc::new(World::default());
    let providers = Arc::new(Providers::from_runtimes(
        ["claude", "codex", "antigravity"].map(|name| {
            let runtime: Arc<dyn LlmRuntime> = Arc::new(Scripted {
                name,
                world: world.clone(),
            });
            (name.to_string(), runtime)
        }),
        Duration::minutes(30),
        Arc::new(Fixed),
    ));
    let dir = tempfile::tempdir().unwrap();
    let audit = Audit::open(&dir.path().join("audit.db")).await.unwrap();
    let config: DebateConfig = serde_json::from_value(config).unwrap();
    let llm = debate::build(&config, providers.clone(), &audit).unwrap();
    Rig {
        world,
        providers,
        llm,
        audit,
        _dir: dir,
    }
}
fn bot() -> BotProfile {
    BotProfile {
        bot_key: "oracle".into(),
        nickname: "Oracle".into(),
        language: None,
        avatar_url: None,
        invite_links: vec![],
        policy: Policy::HybridAnchored,
        provider_chain: vec!["claude".into(), "codex".into(), "antigravity".into()],
    }
}
fn dossier(p: f64, format: Format, context: Vec<Value>) -> Dossier {
    let team = |id, name: &str| Team {
        id,
        name: name.into(),
    };
    Dossier {
        game: Match {
            event_id: 99,
            id: 42,
            team_a: team(1, "Vitality"),
            team_b: team(2, "MOUZ"),
            format,
            scheduled_start: Some(now() + Duration::hours(4)),
        },
        signal: Some(Signal { map: p, series: p }),
        market: Some(p),
        context,
    }
}
/// The roster fact the Liquipedia enricher emits for `team`, from the saved revision.
fn roster_fact(team: &str) -> Value {
    let rosters = liquipedia::parse_revision_rosters(SINGAPORE_REV, now())
        .unwrap()
        .unwrap();
    let event = rosters.at(now()).unwrap();
    let roster = liquipedia::find_team_roster(event, team).unwrap();
    ContextFact::LiquipediaRoster(liquipedia::build_roster_fact(event, roster, "team_a"))
        .to_value()
        .unwrap()
}
/// The news facts the RSS enricher emits for a pair of teams, from the saved HLTV feed.
fn news_facts(team_a: &str, team_b: &str) -> Vec<Value> {
    let cutoff: Instant = "2026-11-26T00:00:00Z".parse().unwrap();
    let items = rss::parse_rss_items(HLTV_RSS, "https://www.hltv.org/rss/news", cutoff).unwrap();
    rss::filter_match_news(
        &items,
        team_a,
        &[],
        team_b,
        &[],
        cutoff,
        Duration::days(14),
        10,
    )
    .unwrap()
    .iter()
    .map(|item| {
        ContextFact::NewsItem(item.at(cutoff).unwrap().clone())
            .to_value()
            .unwrap()
    })
    .collect()
}
async fn run(rig: &Rig, bot: &BotProfile, dossier: &Dossier, tier: Tier) -> Option<Value> {
    rig.llm
        .evaluate(bot, dossier, tier, &RolePrompt::adjustment())
        .await
        .unwrap()
}
/// (seat, provider, status) of the recorded turns, the two advocates first: they
/// run side by side, so which of them is written first is not fixed.
fn seats(turns: &[DebateTurn]) -> Vec<(&str, &str, &str)> {
    let mut seats: Vec<_> = turns
        .iter()
        .map(|t| (t.role.as_str(), t.provider.as_str(), t.status.as_str()))
        .collect();
    let rank = |role: &str| {
        ["advocate", "devil", "arbiter", "predict"]
            .iter()
            .position(|r| *r == role)
    };
    seats.sort_by_key(|(role, _, _)| rank(role));
    seats
}
fn turn<'a>(turns: &'a [DebateTurn], role: &str) -> &'a DebateTurn {
    turns.iter().find(|t| t.role == role).unwrap()
}

#[tokio::test]
async fn an_early_draft_is_one_call_even_on_a_close_bo1_with_a_stand_in() {
    let rig = rig(json!({})).await;
    let close = dossier(0.52, Format::Bo1, vec![roster_fact("Team Vitality")]);
    let answer = run(&rig, &bot(), &close, Tier::Draft).await.unwrap();
    assert_eq!(rig.world.calls(), ["claude:predict"]);
    assert_eq!(answer["summary"]["en"], "predict by claude");
    let turns = rig.audit.debate_turns().await.unwrap();
    assert_eq!(seats(&turns), [("predict", "claude", "ok")]);
    assert_eq!((turns[0].bot.as_str(), turns[0].match_id), ("oracle", 42));
    assert_eq!(turns[0].tier, "Draft");
    assert_eq!(turns[0].brief.as_ref(), Some(&answer));
}

#[tokio::test]
async fn before_hard_lock_a_clean_mismatch_is_one_call_and_everything_else_a_debate() {
    let debate = ["claude:advocate", "codex:devil", "antigravity:arbiter"];
    let cases: Vec<(&str, Dossier, &[&str])> = vec![
        (
            "a clean 90/10 Bo3",
            dossier(0.90, Format::Bo3, vec![roster_fact("MOUZ")]),
            &["claude:predict"],
        ),
        (
            "a clean 10/90 Bo3",
            dossier(0.10, Format::Bo3, vec![]),
            &["claude:predict"],
        ),
        (
            "a competitive Match",
            dossier(0.60, Format::Bo3, vec![]),
            &debate,
        ),
        ("a Bo1", dossier(0.90, Format::Bo1, vec![]), &debate),
        (
            "a stand-in",
            dossier(0.90, Format::Bo3, vec![roster_fact("Team Vitality")]),
            &debate,
        ),
    ];
    for (what, dossier, expected) in cases {
        let rig = rig(json!({})).await;
        run(&rig, &bot(), &dossier, Tier::PreLock).await.unwrap();
        assert_eq!(rig.world.calls(), expected, "{what}");
    }
}

#[tokio::test]
async fn the_risk_flags_come_from_the_context_facts() {
    // Vitality field apEX for the benched jL in the saved revision; MOUZ play their five.
    let vitality = dossier(0.9, Format::Bo3, vec![roster_fact("Team Vitality")]);
    assert_eq!(risk_flags(&vitality), [RiskFlag::StandIn].into());
    assert!(risk_flags(&dossier(0.9, Format::Bo3, vec![roster_fact("MOUZ")])).is_empty());
    // "TYLOO bench Moseyuh" is roster news; "NIP beat Liquid to reach Stage 2" is a result.
    let tyloo = news_facts("TYLOO", "Lynn Vision");
    assert!(!tyloo.is_empty());
    assert_eq!(
        risk_flags(&dossier(0.9, Format::Bo3, tyloo)),
        [RiskFlag::News].into()
    );
    let nip = news_facts("NIP", "Liquid");
    assert!(!nip.is_empty());
    assert!(risk_flags(&dossier(0.9, Format::Bo3, nip)).is_empty());
    let diff = json!({"kind": "vrs_roster_diff", "changed": true});
    assert_eq!(
        risk_flags(&dossier(0.9, Format::Bo3, vec![diff])),
        [RiskFlag::RosterChange].into()
    );
}

#[tokio::test]
async fn the_mode_and_the_threshold_are_set_per_bot() {
    let config = json!({"bots": {
        "eager": {"mode": "always"},
        "frugal": {"mode": "never"},
        "strict": {"lopsided": 0.95}
    }});
    let named = |key: &str| BotProfile {
        bot_key: key.into(),
        ..bot()
    };
    let rig = rig(config).await;
    let close = dossier(0.52, Format::Bo3, vec![]);
    run(&rig, &named("eager"), &close, Tier::Draft)
        .await
        .unwrap();
    assert_eq!(
        rig.world.calls(),
        ["claude:advocate", "codex:devil", "antigravity:arbiter"],
        "always: a Draft run debates"
    );
    rig.world.events.lock().unwrap().clear();
    run(&rig, &named("frugal"), &close, Tier::PreLock)
        .await
        .unwrap();
    assert_eq!(rig.world.calls(), ["claude:predict"], "never: one agent");
    rig.world.events.lock().unwrap().clear();
    let ninety = dossier(0.90, Format::Bo3, vec![]);
    run(&rig, &named("strict"), &ninety, Tier::PreLock)
        .await
        .unwrap();
    assert_eq!(rig.world.calls().len(), 3, "90% is not a mismatch at 0.95");
    assert!(
        serde_json::from_value::<DebateConfig>(json!({"bots": {"x": {"mode": "sometimes"}}}))
            .is_err()
    );
    for bad in [
        json!({"market_step_pp": 0}),
        json!({"market_step_pp": -1}),
        json!({"market_step_pp": 60}),
    ] {
        let config: DebateConfig = serde_json::from_value(json!({"bots": {"x": bad}})).unwrap();
        assert!(debate::build(&config, rig.providers.clone(), &rig.audit).is_err());
    }
    let half: DebateConfig =
        serde_json::from_value(json!({"bots": {"x": {"lopsided": 0.5}}})).unwrap();
    assert!(debate::build(&half, rig.providers.clone(), &rig.audit).is_err());
}

#[tokio::test]
async fn the_two_advocates_run_side_by_side_and_the_arbiter_weighs_both_briefs() {
    let rig = rig(json!({})).await;
    let d = dossier(0.52, Format::Bo3, vec![roster_fact("Team Vitality")]);
    let answer = run(&rig, &bot(), &d, Tier::PreLock).await.unwrap();
    assert_eq!(
        *rig.world.events.lock().unwrap(),
        [
            "claude:advocate:start",
            "codex:devil:start",
            "claude:advocate:end",
            "codex:devil:end",
            "antigravity:arbiter:start",
            "antigravity:arbiter:end",
        ],
        "both advocates are running before either has answered; the arbiter starts after both"
    );
    assert_eq!(answer["summary"]["en"], "arbiter by antigravity");
    assert_eq!(answer["delta"], -4);

    // The arbiter is asked for the scheduler's adjustment, with both briefs attached.
    let arbiter = rig.world.prompt_of("arbiter");
    assert_eq!(arbiter.schema, adjustment_schema());
    let attached: HashMap<_, _> = arbiter.attachments.iter().cloned().collect();
    assert_eq!(
        attached["Market Advocate brief"]["position"],
        "advocate brief by claude"
    );
    assert_eq!(
        attached["Devil's Advocate brief"]["position"],
        "devil brief by codex"
    );
    // Every seat is told the anchor and the flags Rust computed.
    for role in ["advocate", "devil", "arbiter"] {
        let prompt = rig.world.prompt_of(role);
        let baseline = &prompt
            .attachments
            .iter()
            .find(|(l, _)| l == "Baseline")
            .unwrap()
            .1;
        assert_eq!(baseline["p_base_team_a"], 0.52, "{role}");
        assert_eq!(baseline["risk_flags"], json!(["stand_in"]), "{role}");
    }
    assert_eq!(
        rig.world.prompt_of("advocate").schema,
        debate::brief_schema()
    );

    let turns = rig.audit.debate_turns().await.unwrap();
    assert_eq!(
        seats(&turns),
        [
            ("advocate", "claude", "ok"),
            ("devil", "codex", "ok"),
            ("arbiter", "antigravity", "ok"),
        ]
    );
    assert_eq!(turn(&turns, "arbiter").brief.as_ref(), Some(&answer));
    assert_eq!(
        turn(&turns, "devil").brief.as_ref().unwrap()["position"],
        "devil brief by codex"
    );
    assert!(
        turns
            .iter()
            .all(|t| t.tier == "PreLock" && t.input_hash == turns[0].input_hash)
    );
}

#[tokio::test]
async fn a_quota_on_one_seat_fails_over_inside_that_seats_chain_and_the_debate_finishes() {
    let rig = rig(json!({})).await;
    // Codex, the Devil's Advocate, is out of quota in the middle of the debate.
    rig.world.fail("codex", &[LlmError::QuotaExceeded]);
    let d = dossier(0.52, Format::Bo3, vec![]);
    let answer = run(&rig, &bot(), &d, Tier::PreLock).await.unwrap();
    assert_eq!(answer["summary"]["en"], "arbiter by antigravity");
    assert_eq!(
        rig.world.calls(),
        [
            "claude:advocate",
            "codex:devil",
            "antigravity:devil",
            "antigravity:arbiter"
        ],
        "the devil's chain is codex, antigravity, claude"
    );
    assert_eq!(
        seats(&rig.audit.debate_turns().await.unwrap()),
        [
            ("advocate", "claude", "ok"),
            ("devil", "antigravity", "ok"),
            ("arbiter", "antigravity", "ok"),
        ],
        "all three turns are kept, with the provider that answered"
    );
    assert!(matches!(
        rig.providers.breaker("codex"),
        Some(BreakerState::Open {
            cause: LlmError::QuotaExceeded,
            ..
        })
    ));
}

#[tokio::test]
async fn each_seat_has_its_own_configured_chain() {
    let rig = rig(json!({"bots": {"oracle": {
        "fast": ["codex"],
        "advocate": ["antigravity"],
        "devil": ["claude", "codex"],
        "arbiter": ["codex", "claude"]
    }}}))
    .await;
    rig.world.fail("claude", &[LlmError::Timeout]);
    run(
        &rig,
        &bot(),
        &dossier(0.52, Format::Bo3, vec![]),
        Tier::PreLock,
    )
    .await
    .unwrap();
    assert_eq!(
        rig.world.calls(),
        [
            "antigravity:advocate",
            "claude:devil",
            "codex:devil",
            "codex:arbiter"
        ]
    );
    rig.world.events.lock().unwrap().clear();
    run(
        &rig,
        &bot(),
        &dossier(0.52, Format::Bo3, vec![]),
        Tier::Draft,
    )
    .await
    .unwrap();
    assert_eq!(rig.world.calls(), ["codex:predict"]);
}

#[tokio::test]
async fn a_run_that_was_answered_is_never_asked_again() {
    let rig = rig(json!({})).await;
    let d = dossier(0.52, Format::Bo3, vec![]);
    let first = run(&rig, &bot(), &d, Tier::PreLock).await;
    let again = run(&rig, &bot(), &d, Tier::PreLock).await;
    assert_eq!(first, again);
    assert_eq!(rig.world.calls().len(), 3, "the second run cost no call");
    assert_eq!(rig.audit.debate_turns().await.unwrap().len(), 3);
    // A changed input is a new run.
    let moved = dossier(0.55, Format::Bo3, vec![]);
    run(&rig, &bot(), &moved, Tier::PreLock).await.unwrap();
    assert_eq!(rig.world.calls().len(), 6);
    // So is the same dossier for another bot, and another tier.
    let other = BotProfile {
        bot_key: "second".into(),
        ..bot()
    };
    run(&rig, &other, &d, Tier::PreLock).await.unwrap();
    assert_eq!(rig.world.calls().len(), 9);
    run(&rig, &bot(), &d, Tier::Draft).await.unwrap();
    assert_eq!(rig.world.calls().len(), 10);
}

#[tokio::test]
async fn a_failed_arbiter_fails_the_run_and_the_retry_keeps_the_briefs() {
    let rig = rig(json!({"bots": {"oracle": {
        "advocate": ["claude"], "devil": ["codex"], "arbiter": ["antigravity"]
    }}}))
    .await;
    rig.world.fail("antigravity", &[LlmError::InvalidOutput]);
    let d = dossier(0.52, Format::Bo3, vec![]);
    let failed = rig
        .llm
        .evaluate(&bot(), &d, Tier::PreLock, &RolePrompt::adjustment())
        .await;
    assert_eq!(failed, Err(LlmError::InvalidOutput));
    assert_eq!(
        seats(&rig.audit.debate_turns().await.unwrap()),
        [
            ("advocate", "claude", "ok"),
            ("devil", "codex", "ok"),
            ("arbiter", "antigravity", "invalid_output"),
        ]
    );
    rig.world.events.lock().unwrap().clear();
    let answer = run(&rig, &bot(), &d, Tier::PreLock).await.unwrap();
    assert_eq!(answer["summary"]["en"], "arbiter by antigravity");
    assert_eq!(rig.world.calls(), ["antigravity:arbiter"]);
    let briefs: HashMap<_, _> = rig
        .world
        .prompt_of("arbiter")
        .attachments
        .into_iter()
        .collect();
    assert_eq!(
        briefs["Devil's Advocate brief"]["position"],
        "devil brief by codex"
    );
}

#[tokio::test]
async fn a_seat_with_no_provider_left_does_not_stop_the_arbiter() {
    let rig = rig(json!({"bots": {"oracle": {
        "advocate": ["claude"], "devil": ["codex"], "arbiter": ["antigravity"]
    }}}))
    .await;
    rig.world.fail("codex", &[LlmError::QuotaExceeded]);
    let d = dossier(0.52, Format::Bo3, vec![]);
    let answer = run(&rig, &bot(), &d, Tier::PreLock).await.unwrap();
    assert_eq!(answer["summary"]["en"], "arbiter by antigravity");
    let briefs: HashMap<_, _> = rig
        .world
        .prompt_of("arbiter")
        .attachments
        .into_iter()
        .collect();
    assert_eq!(
        briefs["Market Advocate brief"]["position"],
        "advocate brief by claude"
    );
    assert_eq!(briefs["Devil's Advocate brief"], Value::Null);
    assert_eq!(
        seats(&rig.audit.debate_turns().await.unwrap()),
        [
            ("advocate", "claude", "ok"),
            ("devil", "codex", "quota_exceeded"),
            ("arbiter", "antigravity", "ok"),
        ]
    );
}

#[tokio::test]
async fn a_bot_without_providers_or_without_an_anchor_asks_nobody() {
    let rig = rig(json!({})).await;
    let silent = BotProfile {
        provider_chain: vec![],
        ..bot()
    };
    let d = dossier(0.52, Format::Bo3, vec![]);
    assert_eq!(run(&rig, &silent, &d, Tier::PreLock).await, None);
    assert_eq!(run(&rig, &silent, &d, Tier::Draft).await, None);
    // Neither a market line nor a rating: the policy cannot pick, so no call is spent.
    let mut blank = d.clone();
    blank.market = None;
    blank.signal = None;
    assert_eq!(run(&rig, &bot(), &blank, Tier::PreLock).await, None);
    assert!(rig.world.calls().is_empty());
    assert!(rig.audit.debate_turns().await.unwrap().is_empty());
}

/// The same Match and rating with the market line at `market`.
fn line(market: f64) -> Dossier {
    let mut d = dossier(0.52, Format::Bo3, vec![]);
    d.market = Some(market);
    d
}

#[tokio::test]
async fn a_line_moving_inside_its_step_is_answered_from_the_audit() {
    let rig = rig(json!({"bots": {"wide": {"market_step_pp": 5}}})).await;
    let first = run(&rig, &bot(), &line(0.52), Tier::PreLock).await;
    // 52.3%, 51.6% and 52.49% all round to 52: the three answers are reused.
    for market in [0.523, 0.516, 0.5249] {
        let again = run(&rig, &bot(), &line(market), Tier::PreLock).await;
        assert_eq!(again, first, "{market}");
    }
    assert_eq!(rig.world.calls().len(), 3, "one debate for the whole step");
    let turns = rig.audit.debate_turns().await.unwrap();
    assert_eq!(turns.len(), 3);
    assert!(
        turns
            .iter()
            .all(|t| !t.reuse_key.is_empty() && t.reuse_key == turns[0].reuse_key)
    );

    // 52.7% rounds to 53: a new evaluation.
    run(&rig, &bot(), &line(0.527), Tier::PreLock)
        .await
        .unwrap();
    assert_eq!(rig.world.calls().len(), 6);
    // 50.4% and 49.6% both round to 50, but the favourite is another team.
    run(&rig, &bot(), &line(0.504), Tier::PreLock)
        .await
        .unwrap();
    assert_eq!(rig.world.calls().len(), 9);
    run(&rig, &bot(), &line(0.496), Tier::PreLock)
        .await
        .unwrap();
    assert_eq!(
        rig.world.calls().len(),
        12,
        "a favourite flip is a new evaluation"
    );
    // Any other input is compared exactly: a new fact, another rating, the tier.
    let mut flagged = line(0.52);
    flagged.context = vec![roster_fact("Team Vitality")];
    run(&rig, &bot(), &flagged, Tier::PreLock).await.unwrap();
    assert_eq!(rig.world.calls().len(), 15);
    let mut rerated = line(0.52);
    rerated.signal = Some(Signal {
        map: 0.52,
        series: 0.521,
    });
    run(&rig, &bot(), &rerated, Tier::PreLock).await.unwrap();
    assert_eq!(rig.world.calls().len(), 18);

    // The Draft's one answer is reused the same way.
    let draft = run(&rig, &bot(), &line(0.52), Tier::Draft).await;
    assert_eq!(run(&rig, &bot(), &line(0.523), Tier::Draft).await, draft);
    assert_eq!(rig.world.calls().len(), 19);

    // The step is the bot's own: at 5 points 51% and 52% are one step, at 1 they are two.
    let wide = BotProfile {
        bot_key: "wide".into(),
        ..bot()
    };
    run(&rig, &wide, &line(0.51), Tier::PreLock).await.unwrap();
    run(&rig, &wide, &line(0.52), Tier::PreLock).await.unwrap();
    assert_eq!(rig.world.calls().len(), 22);
    run(&rig, &bot(), &line(0.51), Tier::PreLock).await.unwrap();
    assert_eq!(rig.world.calls().len(), 25);
}

#[tokio::test]
async fn a_move_across_the_mismatch_threshold_inside_one_step_asks_nobody() {
    let rig = rig(json!({})).await;
    let at = |market: f64| {
        let mut d = dossier(0.80, Format::Bo3, vec![]);
        d.market = Some(market);
        d
    };
    // 79.6% with an 80% rating anchors just under 0.80: competitive, a debate.
    let debated = run(&rig, &bot(), &at(0.796), Tier::PreLock).await;
    assert_eq!(rig.world.calls().len(), 3);
    // 80.4% anchors just over it: a mismatch now, but the same step and favourite.
    assert_eq!(run(&rig, &bot(), &at(0.804), Tier::PreLock).await, debated);
    assert_eq!(rig.world.calls().len(), 3);
}

#[tokio::test]
async fn only_factors_citing_the_dossier_reach_the_policy() {
    let rig = rig(json!({})).await;
    let fact = roster_fact("Team Vitality");
    let source = fact["source_url"].as_str().unwrap().to_owned();
    assert!(source.starts_with("https://liquipedia.net/"));
    let invented = "https://www.hltv.org/news/99999/vitality-field-a-stand-in";
    *rig.world.factors.lock().unwrap() = vec![
        json!({"kind": "stand_in", "evidence_url": source}),
        json!({"kind": "visa_or_travel", "evidence_url": invented}),
        json!({"kind": "illness_or_emergency", "evidence_url": ""}),
    ];
    let d = dossier(0.52, Format::Bo3, vec![fact]);
    let answer = run(&rig, &bot(), &d, Tier::PreLock).await.unwrap();
    assert_eq!(
        answer["factors"],
        json!([{"kind": "stand_in", "evidence_url": source}]),
        "the invented URL is on a whitelisted site, but this run had no web access"
    );
    assert_eq!(answer["delta"], -4, "the rest of the answer is untouched");
    // The audit keeps what the model said.
    let turns = rig.audit.debate_turns().await.unwrap();
    let said = turn(&turns, "arbiter").brief.as_ref().unwrap();
    assert_eq!(said["factors"].as_array().unwrap().len(), 3);
    // An answer read back from the audit goes through the same check.
    assert_eq!(run(&rig, &bot(), &d, Tier::PreLock).await.unwrap(), answer);
    assert_eq!(rig.world.calls().len(), 3);

    // The same answer for a dossier without that roster fact: nothing is traceable.
    let bare = dossier(0.55, Format::Bo3, vec![]);
    let answer = run(&rig, &bot(), &bare, Tier::PreLock).await.unwrap();
    assert_eq!(answer["factors"], json!([]));
}

/// `claude` answering every run with `answer`: the saved envelope of a real run
/// (`fixtures/debate/predict-claude.json`) with its structured output replaced.
struct Claude {
    answer: Value,
}
#[async_trait]
impl ProcessRunner for Claude {
    async fn run(&self, _: &Invocation) -> Result<Finished, RunError> {
        let mut envelope: Value =
            serde_json::from_str(include_str!("fixtures/debate/predict-claude.json")).unwrap();
        envelope["structured_output"] = self.answer.clone();
        Ok(Finished {
            code: Some(0),
            stdout: envelope.to_string(),
            stderr: String::new(),
        })
    }
}

#[tokio::test]
async fn a_whitelisted_page_is_evidence_only_for_a_run_that_could_read_the_web() {
    let on_hltv = "https://www.hltv.org/news/99999/vitality-field-a-stand-in";
    let answer = json!({
        "delta": -6,
        "factors": [
            {"kind": "stand_in", "evidence_url": on_hltv},
            {"kind": "visa_or_travel", "evidence_url": "https://x.com/rumour/status/1"}
        ],
        "summary": {"en": "Read on HLTV.", "ru": "Прочитано на HLTV.", "uk": "Прочитано на HLTV."}
    });
    // The `llm` section as an operator writes it: `scout` may search the web.
    let config: LlmConfig = serde_json::from_value(json!({
        "providers": {"claude": {"kind": "claude"}},
        "web_search_bots": ["scout"]
    }))
    .unwrap();
    let named = |key: &str| BotProfile {
        bot_key: key.into(),
        provider_chain: vec!["claude".into()],
        ..bot()
    };
    let d = dossier(0.52, Format::Bo3, vec![]);
    let cited = |answer: Option<Value>| answer.unwrap()["factors"].clone();
    for (mode, expected) in [
        (
            Mode::Live,
            json!([{"kind": "stand_in", "evidence_url": on_hltv}]),
        ),
        // A backtest never reads the web, whatever the config says.
        (Mode::Backtest, json!([])),
    ] {
        let runner = Arc::new(Claude {
            answer: answer.clone(),
        });
        let providers = Arc::new(Providers::new(&config, mode, Arc::new(Fixed), runner));
        let dir = tempfile::tempdir().unwrap();
        let audit = Audit::open(&dir.path().join("audit.db")).await.unwrap();
        let llm = debate::build(&DebateConfig::default(), providers, &audit).unwrap();
        let ask = |bot: BotProfile, tier: Tier| {
            let llm = llm.clone();
            let d = d.clone();
            async move {
                llm.evaluate(&bot, &d, tier, &RolePrompt::adjustment())
                    .await
                    .unwrap()
            }
        };
        // Web search is for the run before Hard Lock, and for the opted-in bot only.
        assert_eq!(
            cited(ask(named("scout"), Tier::PreLock).await),
            expected,
            "{mode:?}"
        );
        assert_eq!(cited(ask(named("scout"), Tier::Draft).await), json!([]));
        assert_eq!(cited(ask(named("oracle"), Tier::PreLock).await), json!([]));
    }
}
