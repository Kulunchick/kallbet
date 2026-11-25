//! Seam 1: a Prediction's Rationale (spec #21, ticket #23). It is accepted
//! with the Prediction, validated, and leaves the server only with the
//! revealed Prediction it belongs to: never before Reveal, never to a Bot
//! User before Hard Lock. It follows the Prediction through updates,
//! re-keying, team Overrides and League deletion.

use std::sync::Arc;

use axum::http::StatusCode;
use chrono::{DateTime, Duration, Utc};
use kallbet::domain::{DataSourceKind, EventId, StageKind};
use kallbet::poller;
use kallbet::ports::{Clock, SourceError};
use kallbet::sources::fallback::{FallbackSettings, FallbackSource};
use kallbet::testing::{
    EventSnapshotBuilder, ExternalFake, ScriptedSource, TestApp, TestClient, TestResponse,
};
use serde_json::{Value, json};

const EVENT: i64 = 7148;
const NAVI_FAZE: i64 = 101;
// Teams are numbered in order of appearance.
const NAVI: i64 = 1;
const FAZE: i64 = 2;
const VITALITY: i64 = 3;

const SUMMARY_EN: &str = "FaZe field a stand-in after a late illness. The market still \
                          favours NAVI, and the stand-in widens the gap.";

/// A Rationale as the sidecar sends it, for the picked team.
fn rationale(p_win: f64, summary: &str) -> Value {
    json!({
        "p_win": p_win,
        "override_market": false,
        "preliminary": true,
        "summary": {
            "en": summary,
            "ru": "FaZe выходят со стендином после болезни игрока.",
            "uk": "FaZe грають зі стендіном після хвороби гравця."
        },
        "factors": [
            { "kind": "stand_in", "evidence_url": "https://www.hltv.org/news/1/faze-stand-in" },
            { "kind": "illness_or_emergency", "evidence_url": "https://liquipedia.net/counterstrike/FaZe_Clan" }
        ]
    })
}

struct Setup {
    app: TestApp,
    alice: TestClient,
    bob: TestClient,
    /// The Bot User "Oracle", a Member.
    oracle: TestClient,
    league: i64,
    token: String,
    /// Scheduled Start of NAVI vs FaZe.
    start: DateTime<Utc>,
}

/// A League of alice (creator), bob and the Bot User Oracle on an Event with
/// NAVI vs FaZe four hours from now and Vitality vs G2 a day later; polled.
async fn setup() -> Setup {
    let app = TestApp::new().await;
    let now = app.clock.now();
    let start = now + Duration::hours(4);
    app.source.put_event(
        EventSnapshotBuilder::new(EVENT, "Test Major")
            .stage(StageKind::Swiss, "Stage 1")
            .swiss_match(NAVI_FAZE, (1, 0, 0), "NAVI", "FaZe", start)
            .swiss_match(102, (1, 0, 0), "Vitality", "G2", start + Duration::days(1))
            .build(now),
    );
    let alice = app.user("alice").await;
    let bob = app.user("bob").await;
    let oracle = app.bot("oracle", "Oracle").await;
    let (league, token) = create_league(&alice, "Friends").await;
    join(&bob, &token).await;
    join(&oracle, &token).await;
    poller::tick(&app.state).await;
    Setup {
        app,
        alice,
        bob,
        oracle,
        league,
        token,
        start,
    }
}

async fn create_league(client: &TestClient, name: &str) -> (i64, String) {
    let res = client
        .post("/api/leagues", &json!({ "event_id": EVENT, "name": name }))
        .await;
    assert_eq!(res.status, StatusCode::CREATED, "{}", res.text());
    let league = res.json();
    (
        league["id"].as_i64().unwrap(),
        league["invite_token"].as_str().unwrap().to_string(),
    )
}

async fn join(client: &TestClient, token: &str) {
    let res = client
        .post_empty(&format!("/api/invites/{token}/join"))
        .await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
}

async fn put_prediction(client: &TestClient, league: i64, game: i64, body: &Value) -> TestResponse {
    client
        .put(
            &format!("/api/leagues/{league}/matches/{game}/prediction"),
            body,
        )
        .await
}

async fn matches_text(client: &TestClient, league: i64) -> String {
    let res = client.get(&format!("/api/leagues/{league}/matches")).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    res.text()
}

async fn card_in(client: &TestClient, league: i64, game: i64) -> Option<Value> {
    let view: Value = serde_json::from_str(&matches_text(client, league).await).unwrap();
    view["stages"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|s| s["groups"].as_array().unwrap())
        .flat_map(|g| g["matches"].as_array().unwrap())
        .find(|m| m["id"] == game)
        .cloned()
}

/// The revealed Prediction of the Member with this Nickname, if shown.
fn pick_of<'a>(card: &'a Value, nickname: &str) -> Option<&'a Value> {
    card["predictions"]["revealed"]
        .as_array()?
        .iter()
        .find(|p| p["nickname"] == nickname)
}

impl Setup {
    async fn predict(&self, client: &TestClient, team: i64, rationale: Option<Value>) {
        let mut body = json!({ "team_id": team });
        if let Some(rationale) = rationale {
            body["rationale"] = rationale;
        }
        let res = put_prediction(client, self.league, NAVI_FAZE, &body).await;
        assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    }

    async fn card(&self, client: &TestClient) -> Value {
        card_in(client, self.league, NAVI_FAZE).await.unwrap()
    }

    /// alice and bob predict NAVI vs FaZe and their final edit window runs
    /// out: the humans' early Reveal, hours before the Hard Lock.
    async fn humans_reveal_early(&self) {
        self.predict(&self.alice, NAVI, None).await;
        self.predict(&self.bob, FAZE, None).await;
        self.app.clock.advance(Duration::minutes(5));
        let card = self.card(&self.alice).await;
        assert!(card["predictions"]["revealed"].is_array(), "{card}");
    }

    fn minutes_before_start(&self, minutes: i64) {
        self.app.clock.set(self.start - Duration::minutes(minutes));
    }
}

// --- Hiding ----------------------------------------------------------------

#[tokio::test]
async fn before_reveal_no_member_is_shown_a_rationale() {
    let s = setup().await;
    s.predict(&s.oracle, NAVI, Some(rationale(0.63, SUMMARY_EN)))
        .await;
    s.predict(&s.alice, NAVI, None).await;

    for client in [&s.alice, &s.bob] {
        let text = matches_text(client, s.league).await;
        assert!(!text.contains("stand-in"), "leaked before Reveal: {text}");
        assert!(
            !text.contains("hltv.org/news"),
            "leaked before Reveal: {text}"
        );
        let card = s.card(client).await;
        assert!(card["predictions"]["revealed"].is_null(), "{card}");
    }
}

#[tokio::test]
async fn after_reveal_each_revealed_prediction_carries_its_rationale() {
    let s = setup().await;
    s.predict(&s.oracle, NAVI, Some(rationale(0.63, SUMMARY_EN)))
        .await;

    s.humans_reveal_early().await;

    let card = s.card(&s.bob).await;
    let oracle = pick_of(&card, "Oracle").expect("Oracle's pick is revealed");
    assert_eq!(oracle["team_id"], NAVI);
    assert_eq!(oracle["preliminary"], true, "before Hard Lock");
    assert_eq!(oracle["rationale"], rationale(0.63, SUMMARY_EN), "{card}");
    let alice = pick_of(&card, "alice").unwrap();
    assert!(alice["rationale"].is_null(), "alice gave none: {card}");
}

#[tokio::test]
async fn a_bot_user_is_shown_no_rationale_before_hard_lock() {
    let s = setup().await;
    let market = s.app.bot("market", "MarketBot").await;
    join(&market, &s.token).await;
    s.predict(&s.oracle, NAVI, Some(rationale(0.63, SUMMARY_EN)))
        .await;
    s.predict(&market, FAZE, Some(rationale(0.55, "Market favours FaZe.")))
        .await;
    s.humans_reveal_early().await;

    s.minutes_before_start(22);
    for bot in [&s.oracle, &market] {
        let text = matches_text(bot, s.league).await;
        assert!(!text.contains("stand-in"), "leaked to a bot: {text}");
        assert!(!text.contains("Market favours"), "leaked to a bot: {text}");
        let card = s.card(bot).await;
        assert!(card["predictions"]["revealed"].is_null(), "{card}");
    }

    s.minutes_before_start(15);
    let card = s.card(&market).await;
    let oracle = pick_of(&card, "Oracle").expect("shown from Hard Lock on");
    assert_eq!(oracle["rationale"]["summary"]["en"], SUMMARY_EN);
    assert_eq!(oracle["preliminary"], false);
}

// --- Updating ----------------------------------------------------------------

#[tokio::test]
async fn a_bot_users_update_before_hard_lock_replaces_its_pick_and_rationale() {
    let s = setup().await;
    s.predict(&s.oracle, NAVI, Some(rationale(0.63, SUMMARY_EN)))
        .await;
    s.humans_reveal_early().await;

    // T-22: a late stand-in for NAVI turns the pick around.
    s.minutes_before_start(22);
    let update = json!({
        "p_win": 0.52,
        "override_market": true,
        "preliminary": false,
        "summary": { "en": "NAVI's IGL is out; FaZe edge it." },
        "factors": [{ "kind": "igl_or_coach_change", "evidence_url": "https://www.hltv.org/news/2" }]
    });
    s.predict(&s.oracle, FAZE, Some(update.clone())).await;

    let card = s.card(&s.alice).await;
    let oracle = pick_of(&card, "Oracle").unwrap();
    assert_eq!(oracle["team_id"], FAZE);
    assert_eq!(oracle["rationale"], update, "{card}");
    assert!(!matches_text(&s.alice, s.league).await.contains("stand-in"));

    // T-15: Hard Lock. The pick and its Rationale stay as they were.
    s.minutes_before_start(15);
    let res = put_prediction(
        &s.oracle,
        s.league,
        NAVI_FAZE,
        &json!({ "team_id": NAVI, "rationale": rationale(0.7, SUMMARY_EN) }),
    )
    .await;
    assert_eq!(res.status, StatusCode::CONFLICT, "{}", res.text());
    assert_eq!(res.error_code(), "match_locked");
    let card = s.card(&s.alice).await;
    let oracle = pick_of(&card, "Oracle").unwrap();
    assert_eq!(oracle["team_id"], FAZE);
    assert_eq!(oracle["rationale"], update);
}

#[tokio::test]
async fn a_prediction_sent_without_a_rationale_has_none() {
    let s = setup().await;
    s.predict(&s.oracle, NAVI, Some(rationale(0.63, SUMMARY_EN)))
        .await;
    // The Prediction is replaced as a whole: no Rationale this time.
    s.predict(&s.oracle, NAVI, None).await;
    s.humans_reveal_early().await;

    let card = s.card(&s.alice).await;
    assert!(
        pick_of(&card, "Oracle").unwrap()["rationale"].is_null(),
        "{card}"
    );
}

#[tokio::test]
async fn a_rationale_accepts_the_bounds_and_a_single_language() {
    let s = setup().await;
    let minimal = json!({
        "p_win": 1.0,
        "override_market": false,
        "preliminary": false,
        "summary": { "uk": "Лише українською." },
        "factors": []
    });
    s.predict(&s.oracle, NAVI, Some(minimal.clone())).await;
    let zero = json!({
        "p_win": 0,
        "override_market": true,
        "preliminary": false,
        "summary": { "en": "x".repeat(1000) },
        "factors": []
    });
    s.predict(&s.alice, FAZE, Some(zero)).await;
    s.predict(&s.bob, FAZE, None).await;
    s.app.clock.advance(Duration::minutes(5));

    let card = s.card(&s.bob).await;
    assert_eq!(pick_of(&card, "Oracle").unwrap()["rationale"], minimal);
    assert_eq!(pick_of(&card, "alice").unwrap()["rationale"]["p_win"], 0.0);
}

#[tokio::test]
async fn a_malformed_rationale_is_rejected_and_nothing_changes() {
    let s = setup().await;
    s.predict(&s.oracle, NAVI, Some(rationale(0.63, SUMMARY_EN)))
        .await;
    let valid = rationale(0.4, "Changed my mind.");
    let with = |pointer: &str, value: Value| {
        let mut body = valid.clone();
        *body.pointer_mut(pointer).unwrap() = value;
        body
    };
    let without = |field: &str| {
        let mut body = valid.clone();
        body.as_object_mut().unwrap().remove(field);
        body
    };
    let factor = |kind: &str, url: &str| json!([{ "kind": kind, "evidence_url": url }]);
    let cases = [
        ("p_win above 1", with("/p_win", json!(1.01))),
        ("p_win below 0", with("/p_win", json!(-0.01))),
        ("p_win a string", with("/p_win", json!("0.6"))),
        ("p_win missing", without("p_win")),
        ("override_market not a bool", with("/override_market", json!("yes"))),
        ("summary missing", without("summary")),
        ("summary too long", with("/summary/en", json!("x".repeat(1001)))),
        ("summary blank", with("/summary/ru", json!("   "))),
        ("summary in an unknown language", with("/summary", json!({ "de": "Hallo" }))),
        ("summary empty", with("/summary", json!({}))),
        ("unknown field", with("/factors", json!([])).tap_insert("confidence", json!(1))),
        ("unknown factor kind", with("/factors", factor("vibes", "https://x.com/1"))),
        ("factor without a url", with("/factors", json!([{ "kind": "stand_in" }]))),
        ("javascript url", with("/factors", factor("stand_in", "javascript:alert(1)"))),
        ("ftp url", with("/factors", factor("stand_in", "ftp://hltv.org/x"))),
        ("bare scheme", with("/factors", factor("stand_in", "https://"))),
        ("url with a space", with("/factors", factor("stand_in", "https://hltv.org/a b"))),
        (
            "url too long",
            with(
                "/factors",
                factor("stand_in", &format!("https://hltv.org/{}", "a".repeat(600))),
            ),
        ),
        (
            "too many factors",
            with(
                "/factors",
                json!(
                    (0..9)
                        .map(|i| json!({ "kind": "stand_in", "evidence_url": format!("https://hltv.org/{i}") }))
                        .collect::<Vec<_>>()
                ),
            ),
        ),
        ("not an object", json!("NAVI look strong")),
    ];
    for (what, bad) in cases {
        let res = put_prediction(
            &s.oracle,
            s.league,
            NAVI_FAZE,
            &json!({ "team_id": FAZE, "rationale": bad }),
        )
        .await;
        assert_eq!(
            res.status,
            StatusCode::UNPROCESSABLE_ENTITY,
            "{what}: {}",
            res.text()
        );
        assert_eq!(res.error_code(), "rationale_invalid", "{what}");
    }

    // Nothing was written: the pick and Rationale are the first ones.
    s.humans_reveal_early().await;
    s.minutes_before_start(15);
    let card = s.card(&s.alice).await;
    let oracle = pick_of(&card, "Oracle").unwrap();
    assert_eq!(oracle["team_id"], NAVI);
    assert_eq!(oracle["rationale"], rationale(0.63, SUMMARY_EN));
}

trait TapInsert {
    fn tap_insert(self, key: &str, value: Value) -> Value;
}

impl TapInsert for Value {
    fn tap_insert(mut self, key: &str, value: Value) -> Value {
        self.as_object_mut().unwrap().insert(key.into(), value);
        self
    }
}

// --- Overrides and League deletion -----------------------------------------

#[tokio::test]
async fn a_teams_override_hides_an_invalidated_prediction_with_its_rationale() {
    let s = setup().await;
    s.predict(&s.oracle, NAVI, Some(rationale(0.63, SUMMARY_EN)))
        .await;
    s.predict(&s.alice, FAZE, None).await;
    // alice fixes the team mapping: it is Vitality, not NAVI, that plays.
    let res = s
        .alice
        .post(
            &format!("/api/leagues/{}/matches/{NAVI_FAZE}/overrides", s.league),
            &json!({ "field": "teams", "team_a": VITALITY, "team_b": FAZE }),
        )
        .await;
    assert_eq!(res.status, StatusCode::CREATED, "{}", res.text());
    s.minutes_before_start(15);

    let card = s.card(&s.bob).await;
    assert!(pick_of(&card, "Oracle").is_none(), "invalidated: {card}");
    assert!(!matches_text(&s.bob, s.league).await.contains("stand-in"));

    // Removing the Override restores the Prediction, Rationale and all.
    let res = s
        .alice
        .delete(&format!(
            "/api/leagues/{}/matches/{NAVI_FAZE}/overrides/teams",
            s.league
        ))
        .await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    let card = s.card(&s.bob).await;
    let oracle = pick_of(&card, "Oracle").expect("restored");
    assert_eq!(oracle["rationale"], rationale(0.63, SUMMARY_EN));
}

#[tokio::test]
async fn deleting_a_league_deletes_its_rationales() {
    let s = setup().await;
    s.predict(&s.oracle, NAVI, Some(rationale(0.63, SUMMARY_EN)))
        .await;
    s.humans_reveal_early().await;

    let res = s
        .alice
        .post(
            &format!("/api/leagues/{}/delete", s.league),
            &json!({ "confirm_name": "Friends" }),
        )
        .await;
    assert_eq!(res.status, StatusCode::NO_CONTENT, "{}", res.text());
    let res = s
        .oracle
        .get(&format!("/api/leagues/{}/matches", s.league))
        .await;
    assert_eq!(res.status, StatusCode::NOT_FOUND);

    // A new League on the same Event starts without any of it.
    let (league, token) = create_league(&s.alice, "Friends again").await;
    join(&s.oracle, &token).await;
    s.minutes_before_start(15);
    let card = card_in(&s.alice, league, NAVI_FAZE).await.unwrap();
    assert_eq!(card["predictions"]["revealed"], json!([]), "{card}");
    assert!(!matches_text(&s.alice, league).await.contains("stand-in"));
}

// --- Re-keying ---------------------------------------------------------------

#[tokio::test]
async fn a_rationale_on_a_local_match_follows_it_when_hltv_adopts_it() {
    // HLTV, then a PandaScore fallback that knows a Match HLTV does not.
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
    let now = app.clock.now();
    let hltv_major = || {
        EventSnapshotBuilder::new(EVENT, "Test Major")
            .stage(StageKind::Swiss, "Stage 1")
            .swiss_match(101, (1, 0, 0), "NAVI", "FaZe", now + Duration::hours(2))
            .swiss_match(102, (1, 0, 0), "Vitality", "G2", now + Duration::hours(3))
    };
    let panda_major = || {
        EventSnapshotBuilder::new(555, "Test Major 2026")
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
    };
    hltv.put_event(hltv_major().build(now));
    panda.put_event(panda_major().build(now));
    let alice = app.user("alice").await;
    let oracle = app.bot("oracle", "Oracle").await;
    let (league, token) = create_league(&alice, "Friends").await;
    join(&oracle, &token).await;
    poller::tick(&app.state).await;

    // HLTV goes down; only PandaScore knows round 2: NAVI vs Vitality.
    hltv.fail_with(SourceError::Unavailable("cloudflare".into()));
    panda.put_event(
        panda_major()
            .swiss_match(
                9201,
                (2, 1, 0),
                "Natus Vincere",
                "Team Vitality",
                now + Duration::hours(26),
            )
            .build(now),
    );
    app.clock.advance(Duration::minutes(5));
    poller::tick(&app.state).await;
    let view: Value = serde_json::from_str(&matches_text(&alice, league).await).unwrap();
    let local = view["stages"][0]["groups"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|g| g["matches"].as_array().unwrap())
        .find(|m| m["id"].as_i64().unwrap() < 0)
        .expect("a local Match")["id"]
        .as_i64()
        .unwrap();
    let res = put_prediction(
        &oracle,
        league,
        local,
        &json!({ "team_id": NAVI, "rationale": rationale(0.58, SUMMARY_EN) }),
    )
    .await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    let res = put_prediction(&alice, league, local, &json!({ "team_id": VITALITY })).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());

    // HLTV comes back and lists round 2 as Match 201.
    hltv.recover();
    hltv.update_event(EventId(EVENT), |snapshot| {
        let round2 = hltv_major()
            .swiss_match(
                201,
                (2, 1, 0),
                "Vitality",
                "NAVI",
                now + Duration::hours(25),
            )
            .build(now);
        snapshot.matches = round2.matches;
        snapshot.teams = round2.teams;
    });
    app.clock.advance(Duration::minutes(5));
    poller::tick(&app.state).await;
    // Past the adopted Match's Hard Lock, so the Bot User's pick is final.
    app.clock
        .set(now + Duration::hours(25) - Duration::minutes(15));

    assert!(card_in(&alice, league, local).await.is_none(), "re-keyed");
    let card = card_in(&alice, league, 201).await.expect("Match 201");
    let oracle_pick = pick_of(&card, "Oracle").expect("kept through the re-key");
    assert_eq!(oracle_pick["team_id"], NAVI);
    assert_eq!(
        oracle_pick["rationale"],
        rationale(0.58, SUMMARY_EN),
        "{card}"
    );
}
