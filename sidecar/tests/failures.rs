//! Seam 1: a failure is local to its Match, League, Stage or Invite Link, and
//! a Prediction is never dropped because the LLM gave no answer (spec #21,
//! story 32).
mod support;
use async_trait::async_trait;
use chrono::Duration;
use kallbet::{domain::StageKind, poller, testing::EventSnapshotBuilder};
use kallbet_ai::{adapters::debate::HybridAnchored, *};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};
use support::Harness;

const NO_LLM: &str = "No LLM adjustment was available";

/// A League with MOUZ vs Vitality at `start` and the Invite Link of a bot.
async fn league(h: &Harness, now: Instant, start: Instant) -> (i64, String) {
    h.app.clock.set(now);
    h.app.source.put_event(
        EventSnapshotBuilder::new(99, "Major")
            .stage(StageKind::Swiss, "Stage 1")
            .swiss_match(42, (1, 0, 0), "MOUZ", "Vitality", start)
            .build(now),
    );
    let human = h.app.user("Alice").await;
    let league = human
        .post("/api/leagues", &json!({"name":"Friends","event_id":99}))
        .await
        .json();
    poller::tick(&h.app.state).await;
    (
        league["id"].as_i64().unwrap(),
        format!(
            "{}/join/{}",
            h.base,
            league["invite_token"].as_str().unwrap()
        ),
    )
}

/// The Oracle's policy over the harness's scripted LLM.
async fn oracle(h: &Harness, invite: &str) -> Sidecar {
    let mut sidecar = h.sidecar(invite).await;
    sidecar.bots[0].policy = Policy::HybridAnchored;
    sidecar.bots[0].provider_chain = vec!["claude".into(), "codex".into()];
    sidecar.ports.policy = Arc::new(HybridAnchored);
    sidecar
}

fn summary(p: &Prediction) -> String {
    p.rationale["summary"]["en"].as_str().unwrap().to_owned()
}

#[tokio::test]
async fn every_llm_chain_failing_on_a_draft_still_submits_the_anchor() {
    let h = Harness::new().await;
    let now: Instant = "2026-11-24T00:00:00Z".parse().unwrap();
    let (_, invite) = league(&h, now, now + Duration::hours(4)).await;
    let mut sidecar = oracle(&h, &invite).await;
    h.scripts
        .llm_errors
        .lock()
        .unwrap()
        .push_back(LlmError::QuotaExceeded);

    assert_eq!(h.tick(&mut sidecar, now).await.unwrap(), 1);
    let submitted = sidecar.audit.submissions().await.unwrap();
    assert_eq!(submitted.len(), 1);
    let draft = &submitted[0];
    assert_eq!(draft.team_id, 1, "the VRS favourite MOUZ, unadjusted");
    assert_eq!(draft.rationale["preliminary"], true);
    assert!(summary(draft).contains(NO_LLM), "{}", summary(draft));
    for language in ["ru", "uk"] {
        assert!(draft.rationale["summary"][language].is_string());
    }
    assert_eq!(draft.rationale["factors"], json!([]));
    // The bot's own view shows the Prediction on the server.
    let bot = h.app.bot("vrs", "Baseline").await;
    let view: Value = bot.get("/api/leagues").await.json();
    let league = view[0]["id"].as_i64().unwrap();
    let matches: Value = bot
        .get(&format!("/api/leagues/{league}/matches"))
        .await
        .json();
    assert_eq!(
        matches["stages"][0]["groups"][0]["matches"][0]["my_prediction"],
        1
    );
}

#[tokio::test]
async fn in_the_pre_lock_window_the_anchor_is_submitted_and_a_later_tick_asks_the_llm_again() {
    let h = Harness::new().await;
    let now: Instant = "2026-11-24T00:00:00Z".parse().unwrap();
    let start = now + Duration::hours(4);
    let (_, invite) = league(&h, now, start).await;
    let mut sidecar = oracle(&h, &invite).await;
    *h.scripts.adjustment.lock().unwrap() = Some(json!({
        "delta": 0, "factors": [],
        "summary": {"en": "Model view.", "ru": "Мнение модели.", "uk": "Думка моделі."}
    }));
    assert_eq!(h.tick(&mut sidecar, now).await.unwrap(), 1, "the Draft");

    h.scripts
        .llm_errors
        .lock()
        .unwrap()
        .extend([LlmError::Timeout, LlmError::Transient]);
    let window = start - Duration::minutes(24);
    assert_eq!(h.tick(&mut sidecar, window).await.unwrap(), 1);
    let submitted = sidecar.audit.submissions().await.unwrap();
    let anchored = submitted.last().unwrap();
    assert_eq!(anchored.rationale["preliminary"], false);
    assert!(summary(anchored).contains(NO_LLM));

    // Still failing: the LLM is asked again, the same anchor is not resent.
    let later = window + Duration::minutes(1);
    assert_eq!(h.tick(&mut sidecar, later).await.unwrap(), 0);
    // Answering again: the run is decided and submitted as usual.
    let last = later + Duration::minutes(1);
    assert_eq!(h.tick(&mut sidecar, last).await.unwrap(), 1);
    assert_eq!(
        *h.scripts.llm_calls.lock().unwrap(),
        [Tier::Draft, Tier::PreLock, Tier::PreLock, Tier::PreLock]
    );
    let submitted = sidecar.audit.submissions().await.unwrap();
    assert_eq!(submitted.len(), 3);
    assert_eq!(summary(submitted.last().unwrap()), "Model view.");
    assert!(
        sidecar.audit.failures().await.unwrap().is_empty(),
        "an LLM that gives no answer is not a failed step"
    );
}

/// Two Leagues: listing the first one's Matches fails, and in the second one
/// the first submission is refused.
struct Flaky {
    submitted: Mutex<Vec<i64>>,
}
fn game(id: i64, a: &str, b: &str, start: Instant) -> Match {
    Match {
        event_id: 99,
        id,
        team_a: Team {
            id: id * 10 + 1,
            name: a.into(),
        },
        team_b: Team {
            id: id * 10 + 2,
            name: b.into(),
        },
        format: Format::Bo3,
        scheduled_start: Some(start),
    }
}
#[async_trait]
impl LeaguePlatform for Flaky {
    async fn login(&self, bot: &BotProfile) -> Result<String> {
        Ok(bot.bot_key.clone())
    }
    async fn join(&self, _: &str, _: &str) -> Result<Join> {
        Ok(Join::Member)
    }
    async fn leagues(&self, _: &str) -> Result<Vec<i64>> {
        Ok(vec![1, 2])
    }
    async fn matches(&self, _: &str, league: i64) -> Result<Vec<(Match, bool)>> {
        let start = "2026-11-24T04:00:00Z".parse().unwrap();
        match league {
            1 => Err("HTTP 500 listing the Matches".into()),
            _ => Ok(vec![
                (game(7, "MOUZ", "Vitality", start), false),
                (game(8, "Spirit", "FaZe", start), false),
            ]),
        }
    }
    async fn submit(&self, _: &str, _: i64, game: i64, _: &Prediction) -> Result<()> {
        if game == 7 {
            return Err("HTTP 502 on the write".into());
        }
        self.submitted.lock().unwrap().push(game);
        Ok(())
    }
}

#[tokio::test]
async fn a_failure_in_one_league_or_match_does_not_stop_the_others() {
    let h = Harness::new().await;
    let now: Instant = "2026-11-24T00:00:00Z".parse().unwrap();
    let mut sidecar = h.sidecar("unused").await;
    let platform = Arc::new(Flaky {
        submitted: Mutex::default(),
    });
    sidecar.ports.platform = platform.clone();

    assert_eq!(h.tick(&mut sidecar, now).await.unwrap(), 1);
    assert_eq!(*platform.submitted.lock().unwrap(), [8]);
    let failures = sidecar.audit.failures().await.unwrap();
    let seen: Vec<_> = failures
        .iter()
        .map(|f| (f.step.as_str(), f.league, f.match_id))
        .collect();
    assert_eq!(
        seen,
        [("matches", Some(1), None), ("predict", Some(2), Some(7))]
    );
    assert!(failures[1].error.contains("502"), "{}", failures[1].error);
    assert!(failures.iter().all(|f| f.bot == "vrs" && f.at == now));

    // The decided Prediction of Match 7 is sent again on the next tick.
    assert_eq!(h.tick(&mut sidecar, now + Duration::minutes(1)).await.unwrap(), 0);
}

#[tokio::test]
async fn a_reissued_invite_link_is_skipped_and_the_bots_other_league_goes_on() {
    let h = Harness::new().await;
    let now: Instant = "2026-11-24T00:00:00Z".parse().unwrap();
    let (_, valid) = league(&h, now, now + Duration::hours(4)).await;
    // A second League of the same Event whose admin reissues its link after
    // giving it to the sidecar.
    let human = h.app.user("Bob").await;
    let other = human
        .post("/api/leagues", &json!({"name":"Others","event_id":99}))
        .await
        .json();
    let id = other["id"].as_i64().unwrap();
    let dead = format!(
        "{}/join/{}",
        h.base,
        other["invite_token"].as_str().unwrap()
    );
    let reissued = human
        .post_empty(&format!("/api/leagues/{id}/invite/reissue"))
        .await;
    assert!(reissued.status.is_success());

    let mut sidecar = h.sidecar(&dead).await;
    sidecar.bots[0].invite_links.push(valid);
    assert_eq!(h.tick(&mut sidecar, now).await.unwrap(), 1, "the valid League is predicted");
    assert_eq!(
        h.tick(&mut sidecar, now + Duration::minutes(1)).await.unwrap(),
        0
    );
    let failures = sidecar.audit.failures().await.unwrap();
    assert_eq!(failures.len(), 1, "logged once per run: {failures:?}");
    assert_eq!(failures[0].step, "join");
    assert_eq!(failures[0].error, "Invite Link 1: invite_invalid");
    let bot = h.app.bot("vrs", "Baseline").await;
    let leagues: Value = bot.get("/api/leagues").await.json();
    assert_eq!(leagues.as_array().unwrap().len(), 1);
}
