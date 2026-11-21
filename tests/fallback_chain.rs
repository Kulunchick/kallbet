//! Seam 1: the `FallbackSource` (HLTV -> PandaScore -> Liquipedia) composed
//! of fake sources, behind the `MatchSource` port. Driven through the poller
//! and observed through the Matches API, the fakes' call counters and the
//! data-quality flags League Admins will see (#13).

use std::sync::Arc;

use axum::http::StatusCode;
use chrono::{DateTime, Duration, Utc};
use kallbet::domain::{DataSourceKind, EventId, MatchId, StageKind};
use kallbet::poller;
use kallbet::ports::{Clock, SourceError};
use kallbet::sources::fallback::{FallbackSettings, FallbackSource};
use kallbet::testing::{EventSnapshotBuilder, ExternalFake, ScriptedSource, TestApp, TestClient};
use serde_json::{Value, json};

const EVENT: i64 = 7148;
/// PandaScore's own id of the same Event.
const PS_EVENT: i64 = 555;
/// Liquipedia's own id of the same Event.
const LP_EVENT: i64 = 777;

struct Chain {
    app: TestApp,
    hltv: Arc<ScriptedSource>,
    panda: Arc<ScriptedSource>,
    liquipedia: Arc<ScriptedSource>,
}

struct Options {
    hltv: bool,
    panda: bool,
    liquipedia_canonical_ids: bool,
}

impl Default for Options {
    fn default() -> Options {
        Options {
            hltv: true,
            panda: true,
            liquipedia_canonical_ids: false,
        }
    }
}

async fn chain() -> Chain {
    chain_with(Options::default()).await
}

async fn chain_with(options: Options) -> Chain {
    let hltv = Arc::new(ScriptedSource::new());
    let panda = Arc::new(ScriptedSource::of(DataSourceKind::PandaScore));
    let liquipedia = Arc::new(ScriptedSource::of(DataSourceKind::Liquipedia));
    if !options.hltv {
        hltv.fail_with(SourceError::NotConfigured);
    }
    if !options.panda {
        panda.fail_with(SourceError::NotConfigured);
    }
    let (h, p, l) = (hltv.clone(), panda.clone(), liquipedia.clone());
    let app = TestApp::builder()
        .source_with(move |db, clock| {
            let lp = ExternalFake::new(l);
            let lp = if options.liquipedia_canonical_ids {
                lp.with_canonical_ids()
            } else {
                lp
            };
            Arc::new(
                FallbackSource::new(db, clock, FallbackSettings::default())
                    .with_primary(h)
                    .with_fallback(Arc::new(ExternalFake::new(p)))
                    .with_fallback(Arc::new(lp)),
            )
        })
        .build()
        .await;
    Chain {
        app,
        hltv,
        panda,
        liquipedia,
    }
}

/// The Event as HLTV has it: NAVI=1, FaZe=2, Vitality=3, G2=4.
fn hltv_major(now: DateTime<Utc>) -> EventSnapshotBuilder {
    EventSnapshotBuilder::new(EVENT, "Test Major")
        .stage(StageKind::Swiss, "Stage 1")
        .swiss_match(101, (1, 0, 0), "NAVI", "FaZe", now + Duration::hours(2))
        .swiss_match(102, (1, 0, 0), "Vitality", "G2", now + Duration::hours(3))
}

/// The same Event as PandaScore has it: its own ids, teams in another order
/// and spelling, start times off by half an hour.
fn panda_major(now: DateTime<Utc>) -> EventSnapshotBuilder {
    EventSnapshotBuilder::new(PS_EVENT, "Test Major 2026")
        .stage(StageKind::Swiss, "Swiss Stage")
        .swiss_match(
            9101,
            (1, 0, 0),
            "FaZe Clan",
            "Natus Vincere",
            now + Duration::minutes(150),
        )
        .swiss_match(
            9102,
            (1, 0, 0),
            "Team Vitality",
            "G2 Esports",
            now + Duration::minutes(210),
        )
}

async fn create_league(client: &TestClient) -> i64 {
    let res = client
        .post(
            "/api/leagues",
            &json!({ "event_id": EVENT, "name": "Friends" }),
        )
        .await;
    assert_eq!(res.status, StatusCode::CREATED, "{}", res.text());
    res.json()["id"].as_i64().unwrap()
}

async fn matches(client: &TestClient, league: i64) -> Value {
    let res = client.get(&format!("/api/leagues/{league}/matches")).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    res.json()
}

fn all_matches(view: &Value) -> Vec<Value> {
    view["stages"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|s| s["groups"].as_array().unwrap())
        .flat_map(|g| g["matches"].as_array().unwrap())
        .cloned()
        .collect()
}

fn find_match(view: &Value, id: i64) -> Value {
    all_matches(view)
        .into_iter()
        .find(|m| m["id"] == id)
        .unwrap_or_else(|| panic!("Match {id} not in the view: {view}"))
}

/// Next poll of the Event is due (5 minutes later), then one poller pass.
async fn poll_again(app: &TestApp) {
    app.clock.advance(Duration::minutes(5));
    poller::tick(&app.state).await;
}

#[tokio::test]
async fn when_hltv_fails_the_next_source_updates_the_hltv_matches_by_team_name_and_time() {
    let c = chain().await;
    let now = c.app.clock.now();
    c.hltv.put_event(hltv_major(now).build(now));
    c.panda.put_event(panda_major(now).build(now));
    let alice = c.app.user("alice").await;
    let league = create_league(&alice).await;
    poller::tick(&c.app.state).await;

    c.hltv
        .fail_with(SourceError::Unavailable("cloudflare".into()));
    c.panda
        .finish(EventId(PS_EVENT), MatchId(9101), "Natus Vincere", (0, 2));
    poll_again(&c.app).await;

    let view = matches(&alice, league).await;
    let navi_faze = find_match(&view, 101);
    assert_eq!(navi_faze["source"], "pandascore");
    assert_eq!(navi_faze["status"], "finished");
    assert_eq!(navi_faze["team_a"]["name"], "NAVI", "HLTV names are kept");
    assert_eq!(navi_faze["winner"], navi_faze["team_a"]["id"]);
    assert_eq!(navi_faze["score"], json!({ "team_a": 2, "team_b": 0 }));
    assert_eq!(all_matches(&view).len(), 2, "no duplicate Matches: {view}");
}

#[tokio::test]
async fn when_pandascore_fails_too_liquipedia_serves() {
    let c = chain().await;
    let (alice, league) = hltv_then_panda(&c).await;
    let now = c.app.clock.now();
    c.panda
        .fail_with(SourceError::Unavailable("HTTP 503".into()));
    c.liquipedia.put_event(
        EventSnapshotBuilder::new(LP_EVENT, "Test Major")
            .stage(StageKind::Swiss, "Stage 1")
            .swiss_match(1, (1, 0, 0), "Vitality", "G2", now + Duration::hours(3))
            .build(now),
    );
    c.liquipedia
        .finish(EventId(LP_EVENT), MatchId(1), "G2", (1, 2));
    poll_again(&c.app).await;

    let view = matches(&alice, league).await;
    let game = find_match(&view, 102);
    assert_eq!(game["source"], "liquipedia");
    assert_eq!(game["winner"], game["team_b"]["id"]);
    assert_eq!(
        find_match(&view, 101)["source"],
        "hltv",
        "not reported: left as it was"
    );
}

#[tokio::test]
async fn a_failing_source_is_skipped_after_three_failures_and_tried_again_after_fifteen_minutes() {
    let c = chain().await;
    let now = c.app.clock.now();
    c.hltv.put_event(hltv_major(now).build(now));
    c.panda.put_event(panda_major(now).build(now));
    let alice = c.app.user("alice").await;
    let league = create_league(&alice).await;
    poller::tick(&c.app.state).await;
    let calls = |c: &Chain| c.hltv.fetch_event_calls(EventId(EVENT));
    assert_eq!(calls(&c), 2);

    c.hltv
        .fail_with(SourceError::Parse("standings row missing".into()));
    for _ in 0..3 {
        poll_again(&c.app).await;
    }
    assert_eq!(calls(&c), 5, "three failed tries");

    poll_again(&c.app).await;
    poll_again(&c.app).await;
    assert_eq!(calls(&c), 5, "the breaker is open: HLTV is not asked");
    let view = matches(&alice, league).await;
    assert_eq!(
        find_match(&view, 101)["source"],
        "pandascore",
        "the fallback still serves"
    );

    poll_again(&c.app).await;
    assert_eq!(calls(&c), 6, "15 minutes later one more try (half-open)");
    poll_again(&c.app).await;
    assert_eq!(calls(&c), 6, "it failed, so the breaker is open again");

    c.hltv.recover();
    poll_again(&c.app).await;
    poll_again(&c.app).await;
    assert_eq!(calls(&c), 7, "the next try after 15 more minutes succeeds");
    poll_again(&c.app).await;
    assert_eq!(calls(&c), 8, "closed again: asked on every poll");
    let view = matches(&alice, league).await;
    assert_eq!(find_match(&view, 101)["source"], "hltv");
}

#[tokio::test]
async fn unconfigured_sources_are_skipped_and_ids_a_source_carries_are_used_directly() {
    let c = chain_with(Options {
        hltv: false,
        panda: false,
        liquipedia_canonical_ids: true,
    })
    .await;
    let now = c.app.clock.now();
    // Liquipedia's records carry the HLTV ids (its hltv= field), and spell
    // the teams its own way.
    c.liquipedia.put_event(
        EventSnapshotBuilder::new(EVENT, "Test Major")
            .stage(StageKind::Swiss, "Stage 1")
            .swiss_match(
                101,
                (1, 0, 0),
                "Natus Vincere",
                "FaZe Clan",
                now + Duration::hours(2),
            )
            .build(now),
    );
    let alice = c.app.user("alice").await;
    let league = create_league(&alice).await;
    poller::tick(&c.app.state).await;

    let view = matches(&alice, league).await;
    let game = find_match(&view, 101);
    assert_eq!(game["source"], "liquipedia");
    assert_eq!(game["team_a"]["name"], "Natus Vincere");
    for _ in 0..5 {
        poll_again(&c.app).await;
    }
    assert_eq!(
        c.liquipedia.fetch_event_calls(EventId(EVENT)),
        7,
        "Liquipedia keeps serving: unconfigured sources never tripped anything"
    );
}

/// HLTV served the Event once, then fails for good.
async fn hltv_then_panda(c: &Chain) -> (TestClient, i64) {
    let now = c.app.clock.now();
    c.hltv.put_event(hltv_major(now).build(now));
    c.panda.put_event(panda_major(now).build(now));
    let alice = c.app.user("alice").await;
    let league = create_league(&alice).await;
    poller::tick(&c.app.state).await;
    c.hltv
        .fail_with(SourceError::Unavailable("cloudflare".into()));
    (alice, league)
}

/// The Event's unmatched records, as a League Admin sees them
/// (`GET /api/leagues/{id}/data-warnings`): (source, kind, description).
async fn unmatched(admin: &TestClient, league: i64) -> Vec<(String, String, String)> {
    let text = |v: &Value| v.as_str().unwrap().to_string();
    warnings(admin, league).await["unmatched"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| (text(&f["source"]), text(&f["kind"]), text(&f["description"])))
        .collect()
}

fn flagged(
    flags: &[(String, String, String)],
    source: &str,
    kind: &str,
    description: &str,
) -> bool {
    flags
        .iter()
        .any(|(s, k, d)| s == source && k == kind && d.starts_with(description))
}

#[tokio::test]
async fn a_record_that_cannot_be_linked_is_flagged_and_left_out() {
    let c = chain().await;
    let (alice, league) = hltv_then_panda(&c).await;
    let now = c.app.clock.now();
    c.panda.put_event(
        panda_major(now)
            .swiss_match(9103, (1, 0, 0), "Spirit", "MOUZ", now + Duration::hours(30))
            .build(now),
    );
    poll_again(&c.app).await;

    let view = matches(&alice, league).await;
    assert_eq!(
        all_matches(&view).len(),
        2,
        "the unknown Match is not added: {view}"
    );
    let flags = unmatched(&alice, league).await;
    assert!(flagged(&flags, "pandascore", "team", "Spirit"), "{flags:?}");
    assert!(flagged(&flags, "pandascore", "team", "MOUZ"), "{flags:?}");
    assert!(
        flagged(&flags, "pandascore", "match", "Spirit vs MOUZ"),
        "{flags:?}"
    );
    assert_eq!(flags.len(), 3, "{flags:?}");
}

#[tokio::test]
async fn a_manual_mapping_links_what_names_and_times_could_not() {
    let c = chain().await;
    let now = c.app.clock.now();
    c.hltv.put_event(
        hltv_major(now)
            .swiss_match(103, (1, 0, 0), "Spirit", "MOUZ", now + Duration::hours(30))
            .build(now),
    );
    // PandaScore has Spirit vs MOUZ a whole day off.
    c.panda.put_event(
        panda_major(now)
            .swiss_match(
                9103,
                (1, 0, 0),
                "Team Spirit",
                "MOUZ",
                now + Duration::hours(54),
            )
            .build(now),
    );
    let alice = c.app.user("alice").await;
    let league = create_league(&alice).await;
    poller::tick(&c.app.state).await;
    c.hltv
        .fail_with(SourceError::Unavailable("cloudflare".into()));
    poll_again(&c.app).await;

    let view = matches(&alice, league).await;
    assert_eq!(
        all_matches(&view).len(),
        3,
        "no second Spirit vs MOUZ: {view}"
    );
    assert_eq!(find_match(&view, 103)["source"], "hltv");
    let flags = unmatched(&alice, league).await;
    assert!(
        flagged(&flags, "pandascore", "match", "Team Spirit vs MOUZ"),
        "{flags:?}"
    );

    kallbet::store::external_ids::set_manual(
        &c.app.state.db,
        DataSourceKind::PandaScore,
        kallbet::sources::fallback::mapping::IdKind::Match,
        "m9103",
        103,
        c.app.clock.now(),
    )
    .await
    .unwrap();
    poll_again(&c.app).await;

    let view = matches(&alice, league).await;
    let game = find_match(&view, 103);
    assert_eq!(game["source"], "pandascore");
    assert_eq!(game["scheduled_start"], json!(now + Duration::hours(54)));
    assert!(unmatched(&alice, league).await.is_empty(), "the flag clears");
}

/// The Event's winner conflicts, as a League Admin sees them:
/// (match, kept source, kept winner, other source, other winner).
async fn conflicts(
    admin: &TestClient,
    league: i64,
) -> Vec<(i64, String, Option<i64>, String, Option<i64>)> {
    let text = |v: &Value| v.as_str().unwrap().to_string();
    warnings(admin, league).await["conflicts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| {
            (
                f["match_id"].as_i64().unwrap(),
                text(&f["kept_source"]),
                f["kept_winner"]["id"].as_i64(),
                text(&f["other_source"]),
                f["other_winner"]["id"].as_i64(),
            )
        })
        .collect()
}

const NAVI: i64 = 1;
const FAZE: i64 = 2;

#[tokio::test]
async fn a_lower_priority_source_never_overrules_or_rolls_back_a_higher_one() {
    let c = chain().await;
    let now = c.app.clock.now();
    c.hltv.put_event(hltv_major(now).build(now));
    c.panda.put_event(panda_major(now).build(now));
    let alice = c.app.user("alice").await;
    let league = create_league(&alice).await;
    c.hltv.finish(EventId(EVENT), MatchId(101), "NAVI", (2, 1));
    c.hltv.start(EventId(EVENT), MatchId(102), now);
    poller::tick(&c.app.state).await;

    c.hltv
        .fail_with(SourceError::Parse("standings row missing".into()));
    c.panda
        .finish(EventId(PS_EVENT), MatchId(9101), "FaZe Clan", (2, 0));
    // PandaScore still has Vitality vs G2 as upcoming.
    poll_again(&c.app).await;

    let view = matches(&alice, league).await;
    let navi_faze = find_match(&view, 101);
    assert_eq!(navi_faze["winner"], NAVI);
    assert_eq!(navi_faze["source"], "hltv");
    assert_eq!(navi_faze["score"], json!({ "team_a": 2, "team_b": 1 }));
    assert_eq!(find_match(&view, 102)["status"], "live", "not rolled back");
    assert_eq!(
        conflicts(&alice, league).await,
        vec![(
            101,
            "hltv".into(),
            Some(NAVI),
            "pandascore".into(),
            Some(FAZE)
        )]
    );
}

#[tokio::test]
async fn a_match_only_a_fallback_knows_gets_a_local_id_and_keeps_its_predictions_once_hltv_shows_it()
 {
    let c = chain().await;
    let (alice, league) = hltv_then_panda(&c).await;
    let now = c.app.clock.now();
    // Round 2 is paired; HLTV cannot parse the page, PandaScore has it.
    c.panda.put_event(
        panda_major(now)
            .swiss_match(
                9201,
                (2, 1, 0),
                "Natus Vincere",
                "Team Vitality",
                now + Duration::hours(26),
            )
            .build(now),
    );
    poll_again(&c.app).await;

    let view = matches(&alice, league).await;
    let local = all_matches(&view)
        .into_iter()
        .find(|m| m["id"].as_i64().unwrap() < 0)
        .unwrap_or_else(|| panic!("a local Match: {view}"));
    assert_eq!(local["team_a"]["name"], "NAVI", "linked to the HLTV teams");
    assert_eq!(local["team_b"]["name"], "Vitality");
    assert_eq!(local["source"], "pandascore");
    let local_id = local["id"].as_i64().unwrap();
    let res = alice
        .put(
            &format!("/api/leagues/{league}/matches/{local_id}/prediction"),
            &json!({ "team_id": local["team_b"]["id"] }),
        )
        .await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    poll_again(&c.app).await;
    assert_eq!(
        all_matches(&matches(&alice, league).await).len(),
        3,
        "stable across polls"
    );

    c.hltv.recover();
    c.hltv.update_event(EventId(EVENT), |s| {
        let round2 = EventSnapshotBuilder::new(EVENT, "Test Major")
            .stage(StageKind::Swiss, "Stage 1")
            .swiss_match(101, (1, 0, 0), "NAVI", "FaZe", now + Duration::hours(2))
            .swiss_match(102, (1, 0, 0), "Vitality", "G2", now + Duration::hours(3))
            .swiss_match(
                201,
                (2, 1, 0),
                "Vitality",
                "NAVI",
                now + Duration::hours(25),
            )
            .build(now);
        s.matches = round2.matches;
        s.teams = round2.teams;
    });
    poll_again(&c.app).await;

    let view = matches(&alice, league).await;
    let ids: Vec<i64> = all_matches(&view)
        .iter()
        .map(|m| m["id"].as_i64().unwrap())
        .collect();
    assert_eq!(ids.len(), 3, "the local Match became HLTV's: {ids:?}");
    let game = find_match(&view, 201);
    assert_eq!(game["source"], "hltv");
    assert_eq!(
        game["my_prediction"], 3,
        "the Prediction (Vitality) followed it: {game}"
    );
}

#[tokio::test]
async fn an_event_hltv_never_served_is_found_by_name_and_adopted_when_hltv_returns() {
    let c = chain().await;
    let now = c.app.clock.now();
    c.hltv.put_event(hltv_major(now).build(now));
    c.panda.put_event(panda_major(now).build(now));
    poller::tick(&c.app.state).await; // the Event list knows "Test Major"
    c.hltv
        .fail_with(SourceError::Unavailable("cloudflare".into()));

    let alice = c.app.user("alice").await;
    let league = create_league(&alice).await;
    poller::tick(&c.app.state).await;
    let view = matches(&alice, league).await;
    let games = all_matches(&view);
    assert_eq!(games.len(), 2, "{view}");
    assert!(
        games.iter().all(|m| m["id"].as_i64().unwrap() < 0),
        "local ids: {view}"
    );
    let navi = games
        .iter()
        .find(|m| m["team_b"]["name"] == "Natus Vincere")
        .unwrap();
    assert!(navi["team_b"]["id"].as_i64().unwrap() < 0, "a local team");
    let res = alice
        .put(
            &format!("/api/leagues/{league}/matches/{}/prediction", navi["id"]),
            &json!({ "team_id": navi["team_b"]["id"] }),
        )
        .await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());

    c.hltv.recover();
    poll_again(&c.app).await;
    let view = matches(&alice, league).await;
    let ids: Vec<i64> = all_matches(&view)
        .iter()
        .map(|m| m["id"].as_i64().unwrap())
        .collect();
    assert_eq!(ids, vec![101, 102], "{view}");
    let game = find_match(&view, 101);
    assert_eq!(game["team_a"]["name"], "NAVI");
    assert_eq!(
        game["my_prediction"], NAVI,
        "the Prediction follows the team: {game}"
    );
}

#[tokio::test]
async fn a_link_to_one_stage_of_a_major_resolves_to_the_major_also_while_hltv_is_down() {
    let c = chain().await;
    let now = c.app.clock.now();
    const STAGE_EVENT: i64 = 9028;
    c.hltv.put_event(hltv_major(now).build(now));
    c.hltv.alias(EventId(STAGE_EVENT), EventId(EVENT));
    c.panda.put_event(panda_major(now).build(now));
    let alice = c.app.user("alice").await;
    let link = json!({ "link": format!("https://www.hltv.org/events/{STAGE_EVENT}/stage-1") });

    let res = alice.post("/api/events/resolve", &link).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    assert_eq!(res.json()["id"], EVENT);
    let res = alice
        .post(
            "/api/leagues",
            &json!({ "event_id": STAGE_EVENT, "name": "Friends" }),
        )
        .await;
    assert_eq!(res.status, StatusCode::CREATED, "{}", res.text());
    assert_eq!(
        res.json()["event"]["id"],
        EVENT,
        "the League is on the Major"
    );

    c.hltv
        .fail_with(SourceError::Unavailable("cloudflare".into()));
    let res = alice.post("/api/events/resolve", &link).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    assert_eq!(res.json()["id"], EVENT, "the Stage id is remembered");
    let res = alice
        .post(
            "/api/leagues",
            &json!({ "event_id": STAGE_EVENT, "name": "Others" }),
        )
        .await;
    assert_eq!(res.status, StatusCode::CREATED, "{}", res.text());
    assert_eq!(res.json()["event"]["id"], EVENT);
}

#[tokio::test]
async fn when_hltv_returns_its_result_wins_over_a_fallback_result_with_a_conflict_flag() {
    let c = chain().await;
    let (alice, league) = hltv_then_panda(&c).await;
    c.panda
        .finish(EventId(PS_EVENT), MatchId(9101), "FaZe Clan", (2, 0));
    poll_again(&c.app).await;
    assert_eq!(
        find_match(&matches(&alice, league).await, 101)["winner"],
        FAZE
    );

    c.hltv.recover();
    c.hltv.finish(EventId(EVENT), MatchId(101), "NAVI", (2, 1));
    poll_again(&c.app).await;

    let game = find_match(&matches(&alice, league).await, 101);
    assert_eq!(game["winner"], NAVI);
    assert_eq!(game["source"], "hltv");
    assert_eq!(
        conflicts(&alice, league).await,
        vec![(
            101,
            "hltv".into(),
            Some(NAVI),
            "pandascore".into(),
            Some(FAZE)
        )]
    );
}

/// The Event's data warnings as a League Admin sees them.
async fn warnings(admin: &TestClient, league: i64) -> Value {
    let res = admin
        .get(&format!("/api/leagues/{league}/data-warnings"))
        .await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    res.json()
}

/// A7: a fallback record already linked to a stored Match, whose teams then
/// change at the source (one team agrees, the other is a different known
/// team), is not merged when its winner does not play the stored Match: it
/// is flagged instead.
#[tokio::test]
async fn a_linked_record_won_by_a_team_not_in_the_stored_match_is_flagged_not_merged() {
    let c = chain().await;
    let (alice, league) = hltv_then_panda(&c).await;
    poll_again(&c.app).await; // PandaScore's records are linked to 101 and 102.
    let now = c.app.clock.now();
    // PandaScore now has Vitality vs FaZe where HLTV has Vitality vs G2.
    c.panda.put_event(
        EventSnapshotBuilder::new(PS_EVENT, "Test Major 2026")
            .stage(StageKind::Swiss, "Swiss Stage")
            .swiss_match(9101, (1, 0, 0), "FaZe Clan", "Natus Vincere", now + Duration::minutes(150))
            .swiss_match(9102, (1, 0, 0), "Team Vitality", "FaZe Clan", now + Duration::minutes(210))
            .build(now),
    );
    c.panda
        .finish(EventId(PS_EVENT), MatchId(9102), "FaZe Clan", (0, 2));
    poll_again(&c.app).await;

    let view = matches(&alice, league).await;
    let game = find_match(&view, 102);
    assert_eq!(game["status"], "upcoming", "FaZe's win is not Vitality vs G2's result: {game}");
    assert_eq!(game["winner"], Value::Null, "{game}");
    assert_eq!(all_matches(&view).len(), 2, "{view}");
    let flags = warnings(&alice, league).await;
    assert!(
        flags["unmatched"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f["source"] == "pandascore" && f["kind"] == "match"),
        "{flags}"
    );
}

/// A8: a fallback record of one Stage never links to a stored Match of
/// another Stage. NAVI and FaZe meet in Stage 1 (stored) and again in
/// Stage 2, which HLTV has not shown yet; PandaScore's Stage 2 record has no
/// Scheduled Start (postponed), so time cannot tell them apart.
#[tokio::test]
async fn a_record_of_one_stage_is_not_linked_to_a_match_of_another_stage() {
    let c = chain().await;
    let now = c.app.clock.now();
    c.hltv.put_event(
        hltv_major(now)
            .stage(StageKind::Swiss, "Stage 2")
            .build(now),
    );
    c.hltv.finish(EventId(EVENT), MatchId(101), "NAVI", (2, 1));
    let alice = c.app.user("alice").await;
    let league = create_league(&alice).await;
    poller::tick(&c.app.state).await;
    c.hltv.fail_with(SourceError::Unavailable("cloudflare".into()));
    c.panda.put_event(
        EventSnapshotBuilder::new(PS_EVENT, "Test Major 2026")
            .stage(StageKind::Swiss, "Swiss Stage 1")
            .swiss_match(9102, (1, 0, 0), "Team Vitality", "G2 Esports", now + Duration::minutes(210))
            .stage(StageKind::Swiss, "Swiss Stage 2")
            .swiss_match(9201, (1, 0, 0), "FaZe Clan", "NAVI", now + Duration::days(3))
            .build(now),
    );
    c.panda.update_match(EventId(PS_EVENT), MatchId(9201), |m| m.scheduled_start = None);
    c.panda.finish(EventId(PS_EVENT), MatchId(9201), "FaZe Clan", (2, 0));
    poll_again(&c.app).await;

    let view = matches(&alice, league).await;
    let stage_one = find_match(&view, 101);
    assert_eq!(stage_one["winner"], NAVI, "{stage_one}");
    assert_eq!(stage_one["source"], "hltv", "{stage_one}");
    let warned = warnings(&alice, league).await;
    assert!(warned["conflicts"].as_array().unwrap().is_empty(), "{warned}");
    let stage_two: Vec<Value> = view["stages"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|s| s["ordinal"] == 2)
        .flat_map(|s| s["groups"].as_array().unwrap().clone())
        .flat_map(|g| g["matches"].as_array().unwrap().clone())
        .collect();
    assert_eq!(stage_two.len(), 1, "Stage 2's Match is added there: {view}");
    assert_eq!(stage_two[0]["winner"], FAZE);
}

/// A9: when HLTV adopts a local Match whose fallback result differs from
/// HLTV's, the winner conflict is recorded (under the HLTV id).
#[tokio::test]
async fn a_winner_conflict_is_recorded_when_hltv_adopts_a_local_match() {
    let c = chain().await;
    let (alice, league) = hltv_then_panda(&c).await;
    let now = c.app.clock.now();
    c.panda.put_event(
        panda_major(now)
            .swiss_match(9201, (2, 1, 0), "Natus Vincere", "Team Vitality", now + Duration::hours(26))
            .build(now),
    );
    c.panda.finish(EventId(PS_EVENT), MatchId(9201), "Team Vitality", (0, 2));
    poll_again(&c.app).await;
    assert!(
        all_matches(&matches(&alice, league).await)
            .iter()
            .any(|m| m["id"].as_i64().unwrap() < 0 && m["winner"] == 3),
        "a local Match won by Vitality"
    );

    c.hltv.recover();
    c.hltv.update_event(EventId(EVENT), |s| {
        let round2 = EventSnapshotBuilder::new(EVENT, "Test Major")
            .stage(StageKind::Swiss, "Stage 1")
            .swiss_match(101, (1, 0, 0), "NAVI", "FaZe", now + Duration::hours(2))
            .swiss_match(102, (1, 0, 0), "Vitality", "G2", now + Duration::hours(3))
            .swiss_match(201, (2, 1, 0), "Vitality", "NAVI", now + Duration::hours(25))
            .build(now);
        s.matches = round2.matches;
        s.teams = round2.teams;
    });
    c.hltv.finish(EventId(EVENT), MatchId(201), "NAVI", (2, 1));
    poll_again(&c.app).await;

    let game = find_match(&matches(&alice, league).await, 201);
    assert_eq!(game["winner"], NAVI, "HLTV's result stands: {game}");
    let warned = warnings(&alice, league).await;
    let conflicts = warned["conflicts"].as_array().unwrap();
    assert_eq!(conflicts.len(), 1, "{warned}");
    assert_eq!(conflicts[0]["match_id"], 201);
    assert_eq!(conflicts[0]["kept_source"], "hltv");
    assert_eq!(conflicts[0]["kept_winner"]["id"], NAVI);
    assert_eq!(conflicts[0]["other_source"], "pandascore");
    assert_eq!(conflicts[0]["other_winner"]["id"], 3);
}
