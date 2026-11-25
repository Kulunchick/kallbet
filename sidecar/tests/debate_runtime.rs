//! `TieredDebateLlm` behind the `LlmRuntime` port, over three scripted providers
//! with real breakers and a real audit file: which pass a run gets, the order of
//! the seats, failover inside a seat's chain, and what the audit keeps.
use async_trait::async_trait;
use chrono::Duration;
use kallbet_ai::{
    adapters::{
        context::{liquipedia, rss},
        debate::{self, DebateConfig, DebateTurn, RiskFlag, risk_flags},
        llm::{BreakerState, Providers},
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
                "factors": [],
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
fn seats(turns: &[DebateTurn]) -> Vec<(&str, &str, &str)> {
    turns
        .iter()
        .map(|t| (t.role.as_str(), t.provider.as_str(), t.status.as_str()))
        .collect()
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
    assert_eq!(turns[2].brief.as_ref(), Some(&answer));
    assert_eq!(
        turns[1].brief.as_ref().unwrap()["position"],
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
