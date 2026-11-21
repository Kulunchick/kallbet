//! Seam 1: a Match with a local id (known only to a fallback Data Source)
//! that HLTV later lists is re-keyed to its HLTV id (#9). A League's
//! Overrides on it and its notification log follow (#13, #16), and the
//! conflict and unmatched warnings reach League Admins.

use std::sync::Arc;

use axum::http::StatusCode;
use chrono::{DateTime, Duration, Utc};
use kallbet::domain::{DataSourceKind, EventId, MatchId, StageKind, UserId};
use kallbet::poller;
use kallbet::ports::{Clock, NotificationKind, SourceError};
use kallbet::sources::fallback::{FallbackSettings, FallbackSource};
use kallbet::testing::{EventSnapshotBuilder, ExternalFake, ScriptedSource, TestApp, TestClient};
use serde_json::{Value, json};

const EVENT: i64 = 7148;
const PS_EVENT: i64 = 555;

struct Chain {
    app: TestApp,
    hltv: Arc<ScriptedSource>,
    panda: Arc<ScriptedSource>,
}

async fn chain() -> Chain {
    let hltv = Arc::new(ScriptedSource::new());
    let panda = Arc::new(ScriptedSource::of(DataSourceKind::PandaScore));
    let liquipedia = Arc::new(ScriptedSource::of(DataSourceKind::Liquipedia));
    liquipedia.fail_with(SourceError::NotConfigured);
    let (h, p, l) = (hltv.clone(), panda.clone(), liquipedia.clone());
    let app = TestApp::builder()
        .source_with(move |db, clock| {
            Arc::new(
                FallbackSource::new(db, clock, FallbackSettings::default())
                    .with_primary(h)
                    .with_fallback(Arc::new(ExternalFake::new(p)))
                    .with_fallback(Arc::new(ExternalFake::new(l))),
            )
        })
        .build()
        .await;
    Chain { app, hltv, panda }
}

/// NAVI=1, FaZe=2, Vitality=3, G2=4.
fn hltv_major(now: DateTime<Utc>) -> EventSnapshotBuilder {
    EventSnapshotBuilder::new(EVENT, "Test Major")
        .stage(StageKind::Swiss, "Stage 1")
        .swiss_match(101, (1, 0, 0), "NAVI", "FaZe", now + Duration::hours(2))
        .swiss_match(102, (1, 0, 0), "Vitality", "G2", now + Duration::hours(3))
}

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

async fn poll_again(app: &TestApp) {
    app.clock.advance(Duration::minutes(5));
    poller::tick(&app.state).await;
}

async fn matches(client: &TestClient, league: i64) -> Vec<Value> {
    let res = client.get(&format!("/api/leagues/{league}/matches")).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    res.json()["stages"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|s| s["groups"].as_array().unwrap().clone())
        .flat_map(|g| g["matches"].as_array().unwrap().clone())
        .collect()
}

/// alice's League, polled once from HLTV; then HLTV goes down and only
/// PandaScore knows round 2 (NAVI vs Vitality): a local Match.
async fn with_local_match(c: &Chain) -> (TestClient, i64, i64) {
    let now = c.app.clock.now();
    c.hltv.put_event(hltv_major(now).build(now));
    c.panda.put_event(panda_major(now).build(now));
    let alice = c.app.user("alice").await;
    let res = alice
        .post(
            "/api/leagues",
            &json!({ "event_id": EVENT, "name": "Friends" }),
        )
        .await;
    assert_eq!(res.status, StatusCode::CREATED, "{}", res.text());
    let league = res.json()["id"].as_i64().unwrap();
    poller::tick(&c.app.state).await;
    c.hltv
        .fail_with(SourceError::Unavailable("cloudflare".into()));
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
    let local = matches(&alice, league)
        .await
        .into_iter()
        .find(|m| m["id"].as_i64().unwrap() < 0)
        .expect("a local Match")["id"]
        .as_i64()
        .unwrap();
    (alice, league, local)
}

/// HLTV comes back and lists round 2 as Match 201.
async fn hltv_returns(c: &Chain) {
    let now = c.app.clock.now();
    c.hltv.recover();
    c.hltv.update_event(EventId(EVENT), |s| {
        let round2 = hltv_major(now)
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
}

#[tokio::test]
async fn overrides_on_a_local_match_follow_it_when_hltv_adopts_it() {
    let c = chain().await;
    let (alice, league, local) = with_local_match(&c).await;
    let mine = c.app.clock.now() + Duration::hours(30);
    let res = alice
        .post(
            &format!("/api/leagues/{league}/matches/{local}/overrides"),
            &json!({ "field": "scheduled_start", "scheduled_start": mine }),
        )
        .await;
    assert_eq!(res.status, StatusCode::CREATED, "{}", res.text());

    hltv_returns(&c).await;

    let game = matches(&alice, league)
        .await
        .into_iter()
        .find(|m| m["id"] == 201)
        .expect("Match 201");
    assert_eq!(game["scheduled_start"], json!(mine), "{game}");
    let history = alice
        .get(&format!("/api/leagues/{league}/overrides"))
        .await
        .json();
    assert_eq!(history["entries"][0]["match_id"], 201, "{history}");
}

#[tokio::test]
async fn a_reveal_announced_before_a_re_key_is_not_announced_again() {
    let c = chain().await;
    let (alice, league, local) = with_local_match(&c).await;
    let alice_id = UserId(alice.get("/api/me").await.json()["id"].as_i64().unwrap());
    // alice is the only Member: her Prediction Reveals the Match.
    let res = alice
        .put(
            &format!("/api/leagues/{league}/matches/{local}/prediction"),
            &json!({ "team_id": 1 }),
        )
        .await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    poll_again(&c.app).await;
    let reveals = || {
        c.app
            .notifier
            .sent_to(alice_id)
            .into_iter()
            .filter(|n| n.kind == NotificationKind::Reveal)
            .count()
    };
    assert_eq!(reveals(), 1);

    hltv_returns(&c).await;
    poll_again(&c.app).await;

    assert!(
        matches(&alice, league).await.iter().any(|m| m["id"] == 201),
        "re-keyed"
    );
    assert_eq!(reveals(), 1, "announced once");
}

#[tokio::test]
async fn league_admins_see_the_events_conflict_and_unmatched_warnings() {
    let c = chain().await;
    let now = c.app.clock.now();
    c.hltv.put_event(hltv_major(now).build(now));
    c.panda.put_event(panda_major(now).build(now));
    let alice = c.app.user("alice").await;
    let res = alice
        .post(
            "/api/leagues",
            &json!({ "event_id": EVENT, "name": "Friends" }),
        )
        .await;
    let league = res.json()["id"].as_i64().unwrap();
    let invite = res.json()["invite_token"].as_str().unwrap().to_string();
    let bob = c.app.user("bob").await;
    bob.post_empty(&format!("/api/invites/{invite}/join")).await;
    poller::tick(&c.app.state).await;

    // HLTV down: PandaScore reports FaZe winning 101 and a Match it cannot link.
    c.hltv
        .fail_with(SourceError::Unavailable("cloudflare".into()));
    c.panda.put_event(
        panda_major(now)
            .swiss_match(9103, (1, 0, 0), "Spirit", "MOUZ", now + Duration::hours(30))
            .build(now),
    );
    c.panda
        .finish(EventId(PS_EVENT), MatchId(9101), "FaZe Clan", (2, 0));
    poll_again(&c.app).await;
    // HLTV returns with NAVI winning.
    c.hltv.recover();
    c.hltv.finish(EventId(EVENT), MatchId(101), "NAVI", (2, 1));
    poll_again(&c.app).await;

    let url = format!("/api/leagues/{league}/data-warnings");
    assert_eq!(bob.get(&url).await.status, StatusCode::FORBIDDEN);
    let res = alice.get(&url).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    let body = res.json();
    let conflict = &body["conflicts"][0];
    assert_eq!(conflict["match_id"], 101, "{body}");
    assert_eq!(conflict["kept_source"], "hltv");
    assert_eq!(conflict["kept_winner"]["name"], "NAVI");
    assert_eq!(conflict["other_source"], "pandascore");
    assert_eq!(conflict["other_winner"]["name"], "FaZe");
    let unmatched: Vec<&str> = body["unmatched"]
        .as_array()
        .unwrap()
        .iter()
        .map(|u| u["description"].as_str().unwrap())
        .collect();
    assert!(
        unmatched.iter().any(|d| d.starts_with("Spirit vs MOUZ")),
        "{body}"
    );
}
