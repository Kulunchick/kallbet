//! Ticket #45 (spec #33): player-level Candidate Signals (`lineup_strength`
//! and `player_form`) backed by point-in-time Liquipedia rosters (before Match
//! cutoff) and HLTV `/matches/*` per-player statistics (from Matches finished
//! strictly before cutoff).
use chrono::Duration;
use kallbet_ai::{
    adapters::{
        backtest::{Feeds, ReplayArchive, ReplayPlayerHistory},
        hltv_research::archive::{HltvMatchRecord, MapResult, PlayerMatchStats, TeamPlayerStats, TeamRef},
        lab::{
            Lab, LabOutcome, MatchView, PROTOCOL_VERSION,
            signals::{
                self,
                lineup_strength::{
                    self, LineupStrength, MIN_KNOWN_PLAYERS, PRIOR_RATING, lineup_strength_feature,
                },
                player_form::{self, PlayerForm, player_form_feature},
            },
            stats::{self, Verdict},
        },
    },
    ports::{
        ContextFact, PlayerAppearance, PlayerStatsHistory, PlayerStatsSlice, TeamRosterFact,
        Attribution, player_key,
    },
    *,
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    sync::Arc,
};

fn backtest_fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/backtest")
}

fn research_fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/research")
}

fn copy_backtest_prefixes(prefixes: &[&str]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    for entry in std::fs::read_dir(backtest_fixtures()).unwrap() {
        let path = entry.unwrap().path();
        if !path.is_file() {
            continue;
        }
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        if prefixes.iter().any(|p| name.starts_with(p)) {
            std::fs::copy(&path, dir.path().join(&name)).unwrap();
        }
    }
    dir
}

fn close(a: f64, b: f64, eps: f64) -> bool {
    (a - b).abs() <= eps
}

// ---------------------------------------------------------------------------
// Registry & fallback when archive lacks HLTV player stats
// ---------------------------------------------------------------------------

#[test]
fn lineup_strength_and_player_form_are_registered_with_distinct_names() {
    let names: Vec<_> = signals::registry().iter().map(|s| s.name()).collect();
    assert!(
        names.contains(&lineup_strength::NAME),
        "lineup_strength missing from registry: {names:?}"
    );
    assert!(
        names.contains(&player_form::NAME),
        "player_form missing from registry: {names:?}"
    );
    let unique: BTreeSet<_> = names.iter().collect();
    assert_eq!(unique.len(), names.len());

    assert!(signals::by_name("lineup_strength").is_some());
    assert!(signals::by_name("player_form").is_some());
}

#[tokio::test]
async fn archive_without_hltv_records_reports_player_signals_not_testable() {
    let archive = ReplayArchive::load(&backtest_fixtures()).unwrap();
    assert!(archive.hltv_matches.is_empty());

    let mut lab = Lab::new(archive).unwrap();
    lab.signals = vec![Arc::new(LineupStrength), Arc::new(PlayerForm)];
    let outcome = lab.run().await.unwrap();

    for name in [lineup_strength::NAME, player_form::NAME] {
        let c = outcome
            .report
            .candidates
            .iter()
            .find(|c| c.name == name)
            .unwrap();
        assert_eq!(c.verdict, Verdict::NotTestable, "{name}");
        assert_eq!(c.computed_on, 0, "{name}");
        assert!(
            c.reason.to_lowercase().contains("hltv"),
            "{name} reason should mention missing HLTV stats: {}",
            c.reason
        );
    }
}

// ---------------------------------------------------------------------------
// Acceptance criteria 2 & 4: real HLTV pages + strict finished_at < cutoff
// ---------------------------------------------------------------------------

#[tokio::test]
async fn replay_archive_loads_real_hltv_pages_and_enforces_strict_finished_before_cutoff() {
    let dir = copy_backtest_prefixes(&[
        "panda-20708",
        "panda-21115",
        "lp-20708",
        "lp-21115",
        "event-iem-cologne-2026-before-stage1",
    ]);
    let hltv_dir = dir.path().join("hltv");
    std::fs::create_dir_all(&hltv_dir).unwrap();

    // Real saved HLTV pages from #36 (Heroic vs Sharks Bo1, Vitality vs FUT Bo3)
    // alongside manifest/tsv/blocked metadata files as produced by HltvCollector.
    std::fs::copy(
        research_fixtures().join("hltv-match-2394774-heroic-vs-sharks.html.gz"),
        hltv_dir.join("hltv-match-2394774-heroic-vs-sharks.html.gz"),
    )
    .unwrap();
    std::fs::copy(
        research_fixtures().join("hltv-match-2394898-vitality-vs-fut.html.gz"),
        hltv_dir.join("hltv-match-2394898-vitality-vs-fut.html.gz"),
    )
    .unwrap();
    std::fs::write(hltv_dir.join("manifest.json"), r#"{"matches":{}}"#).unwrap();
    std::fs::write(hltv_dir.join("sources.tsv"), "file\turl\tfetched_at\n").unwrap();
    std::fs::write(hltv_dir.join("hltv-match-1-blocked.html"), "cf").unwrap();

    let archive = ReplayArchive::load(dir.path()).unwrap();
    assert_eq!(archive.hltv_matches.len(), 2);

    let history = ReplayPlayerHistory::new(&archive);

    // Heroic vs Sharks (HLTV 2394774) corresponds to PandaScore 1494593 in panda-20708,
    // whose verified end_at is 2026-06-02T13:39:47Z.
    let heroic_sharks_finished: Instant = "2026-06-02T13:39:47Z".parse().unwrap();

    // At T == finished_at: must NOT appear.
    let at_finish = history
        .player_matches("xfl0ud", heroic_sharks_finished)
        .await
        .unwrap();
    assert!(
        at_finish.at(heroic_sharks_finished).unwrap().is_empty(),
        "stats at exact finished_at must be excluded"
    );

    // At T == finished_at + 1s: MUST appear with xfl0ud's real HLTV rating (1.55).
    let after_finish = heroic_sharks_finished + Duration::seconds(1);
    let after_slice = history.player_matches("xfl0ud", after_finish).await.unwrap();
    let apps = after_slice.at(after_finish).unwrap();
    assert_eq!(apps.len(), 1);
    assert_eq!(apps[0].hltv_match_id, 2394774);
    assert!(close(apps[0].rating, 1.55, 1e-6));

    // Re-checking a slice from a later cutoff at an earlier cutoff fails with TemporalLeakError.
    let leak = after_slice.at(heroic_sharks_finished).unwrap_err();
    assert!(leak.observed_at >= leak.cutoff);
}

struct LeakyPlayerStats;
#[async_trait::async_trait]
impl PlayerStatsHistory for LeakyPlayerStats {
    async fn player_matches(&self, nickname: &str, cutoff: Instant) -> Result<PlayerStatsSlice> {
        let app = PlayerAppearance {
            hltv_match_id: 999_001,
            player_id: 42,
            nickname_key: player_key(nickname),
            nickname: nickname.to_string(),
            team_key: "alpha".to_string(),
            format: Format::Bo3,
            maps_played: 2,
            rating: 1.20,
            rating_version: "3.0".to_string(),
            finished_at: cutoff,
        };
        Ok(PlayerStatsSlice::from_vec(vec![AsOf::new(
            app,
            cutoff,
            cutoff + Duration::days(1),
        )?]))
    }
}

struct LeakyContext;
#[async_trait::async_trait]
impl ContextEnricher for LeakyContext {
    async fn facts(&self, game: &Match, cutoff: Instant) -> Result<Vec<AsOf<Value>>> {
        let fact = ContextFact::LiquipediaRoster(TeamRosterFact {
            team: game.team_a.name.clone(),
            match_side: "team_a".into(),
            page_title: "Leak".into(),
            revid: 1,
            revision_timestamp: cutoff,
            source_url: "https://liquipedia.net/counterstrike/Leak".into(),
            attribution: Attribution {
                source: "Liquipedia".into(),
                license: "CC-BY-SA 3.0".into(),
                license_url: "https://creativecommons.org/licenses/by-sa/3.0/".into(),
            },
            active_players: vec!["p1".into(), "p2".into(), "p3".into(), "p4".into(), "p5".into()],
            stand_ins: vec![],
            benched_players: vec![],
            unused_substitutes: vec![],
            coaches: vec![],
            has_stand_in: false,
            notes: vec![],
        });
        Ok(vec![AsOf::new(
            fact.to_value()?,
            cutoff,
            cutoff + Duration::days(1),
        )?])
    }
}

#[tokio::test]
async fn leaky_player_stats_or_context_port_aborts_the_lab_run() {
    let dir = copy_backtest_prefixes(&[
        "panda-20708",
        "lp-20708",
        "event-iem-cologne-2026-before-stage1",
    ]);
    let archive = ReplayArchive::load(dir.path()).unwrap();

    // 1. Leaky ContextEnricher aborts.
    let mut lab = Lab::new(archive).unwrap();
    let replay = Feeds::replay(&lab.archive).unwrap();
    lab.feeds = Feeds {
        context: Arc::new(LeakyContext),
        ..replay
    };
    let err = lab.run().await.err().expect("leaky context must abort");
    assert!(err.is::<TemporalLeakError>(), "{err}");

    // 2. Leaky PlayerStatsHistory aborts (when rosters exist).
    let archive = ReplayArchive::load(dir.path()).unwrap();
    let mut lab = Lab::new(archive).unwrap();
    let replay = Feeds::replay(&lab.archive).unwrap();
    lab.feeds = Feeds {
        player_stats: Arc::new(LeakyPlayerStats),
        ..replay
    };
    let err = lab.run().await.err().expect("leaky player stats must abort");
    assert!(err.is::<TemporalLeakError>(), "{err}");
}

// ---------------------------------------------------------------------------
// Pure math unit tests: stand-ins, debut priors, form, recency, symmetry
// ---------------------------------------------------------------------------

fn roster_fact(side: &str, team: &str, players: [&str; 5], observed: Instant) -> AsOf<TeamRosterFact> {
    AsOf::new(
        TeamRosterFact {
            team: team.to_string(),
            match_side: side.to_string(),
            page_title: "Test/Event".into(),
            revid: 1,
            revision_timestamp: observed,
            source_url: "https://liquipedia.net/counterstrike/Test".into(),
            attribution: Attribution {
                source: "Liquipedia".into(),
                license: "CC-BY-SA 3.0".into(),
                license_url: "https://creativecommons.org/licenses/by-sa/3.0/".into(),
            },
            active_players: players.iter().map(|s| (*s).to_string()).collect(),
            stand_ins: vec![],
            benched_players: vec![],
            unused_substitutes: vec![],
            coaches: vec![],
            has_stand_in: false,
            notes: vec![],
        },
        observed,
        observed + Duration::days(30),
    )
    .unwrap()
}

fn app_slice(items: Vec<(i64, i64, &str, &str, u8, f64, Instant)>) -> PlayerStatsSlice {
    let mut out = Vec::with_capacity(items.len());
    for (mid, pid, nick, team, maps, rating, finished_at) in items {
        let a = PlayerAppearance {
            hltv_match_id: mid,
            player_id: pid,
            nickname_key: player_key(nick),
            nickname: nick.to_string(),
            team_key: player_key(team),
            format: Format::Bo3,
            maps_played: maps,
            rating,
            rating_version: "3.0".to_string(),
            finished_at,
        };
        out.push(AsOf::new(a, finished_at, finished_at + Duration::days(365)).unwrap());
    }
    PlayerStatsSlice::from_vec(out)
}

fn make_view(
    team_a: &str,
    team_b: &str,
    roster_a: Option<AsOf<TeamRosterFact>>,
    roster_b: Option<AsOf<TeamRosterFact>>,
    player_stats: BTreeMap<String, PlayerStatsSlice>,
    cutoff: Instant,
) -> MatchView {
    MatchView {
        game: Match {
            event_id: 1,
            id: 100,
            team_a: Team {
                id: 1,
                name: team_a.to_string(),
            },
            team_b: Team {
                id: 2,
                name: team_b.to_string(),
            },
            format: Format::Bo3,
            scheduled_start: Some(cutoff + Duration::minutes(15)),
        },
        cutoff,
        signal: None,
        market: None,
        history_before: HistorySlice::default(),
        team_a_history: HistorySlice::default(),
        team_b_history: HistorySlice::default(),
        market_history: BTreeMap::new(),
        map_history: Default::default(),
        veto_history: Default::default(),
        context: kallbet_ai::adapters::backtest::match_context::MatchContext::unknown(),
        team_a_roster: roster_a,
        team_b_roster: roster_b,
        player_stats,
    }
}

#[test]
fn lineup_strength_prices_stand_ins_uses_stated_prior_and_is_antisymmetric() {
    let t0: Instant = "2026-06-01T12:00:00Z".parse().unwrap();
    let roster_ts = t0 - Duration::hours(12);

    let mut stats = BTreeMap::new();
    // Team A regulars: star1 (1.35), a2..a5 (1.05). Stand-in sub_weak (0.82).
    stats.insert(
        "star1".into(),
        app_slice(vec![(1, 101, "star1", "Alpha", 3, 1.35, t0 - Duration::days(5))]),
    );
    for (i, n) in ["a2", "a3", "a4", "a5"].iter().enumerate() {
        stats.insert(
            (*n).into(),
            app_slice(vec![(
                1,
                102 + i as i64,
                n,
                "Alpha",
                3,
                1.05,
                t0 - Duration::days(5),
            )]),
        );
    }
    stats.insert(
        "subweak".into(),
        app_slice(vec![(2, 199, "subweak", "Academy", 3, 0.82, t0 - Duration::days(4))]),
    );
    // Team B: b1..b5 (1.00).
    for (i, n) in ["b1", "b2", "b3", "b4", "b5"].iter().enumerate() {
        stats.insert(
            (*n).into(),
            app_slice(vec![(
                3,
                201 + i as i64,
                n,
                "Beta",
                3,
                1.00,
                t0 - Duration::days(5),
            )]),
        );
    }

    let ra_full = roster_fact("team_a", "Alpha", ["star1", "a2", "a3", "a4", "a5"], roster_ts);
    let ra_standin = roster_fact("team_a", "Alpha", ["subweak", "a2", "a3", "a4", "a5"], roster_ts);
    let ra_debut = roster_fact("team_a", "Alpha", ["rookie", "a2", "a3", "a4", "a5"], roster_ts);
    let rb = roster_fact("team_b", "Beta", ["b1", "b2", "b3", "b4", "b5"], roster_ts);

    let v_full = make_view("Alpha", "Beta", Some(ra_full.clone()), Some(rb.clone()), stats.clone(), t0);
    let v_standin = make_view("Alpha", "Beta", Some(ra_standin), Some(rb.clone()), stats.clone(), t0);
    let v_debut = make_view("Alpha", "Beta", Some(ra_debut), Some(rb.clone()), stats.clone(), t0);

    let (f_full, _) = lineup_strength_feature(&v_full).unwrap().unwrap();
    let (f_standin, _) = lineup_strength_feature(&v_standin).unwrap().unwrap();
    let (f_debut, _) = lineup_strength_feature(&v_debut).unwrap().unwrap();

    assert!(f_full > f_debut, "star1 (1.35) > rookie prior ({PRIOR_RATING}): {f_full} vs {f_debut}");
    assert!(f_debut > f_standin, "rookie prior ({PRIOR_RATING}) > subweak (0.82): {f_debut} vs {f_standin}");

    // Anti-symmetry: swapping team A and team B negates the feature.
    let ra_swapped = roster_fact("team_b", "Alpha", ["star1", "a2", "a3", "a4", "a5"], roster_ts);
    let rb_swapped = roster_fact("team_a", "Beta", ["b1", "b2", "b3", "b4", "b5"], roster_ts);
    let v_swapped = make_view("Beta", "Alpha", Some(rb_swapped), Some(ra_swapped), stats.clone(), t0);
    let (f_swapped, _) = lineup_strength_feature(&v_swapped).unwrap().unwrap();
    assert!(close(f_full, -f_swapped, 1e-12));

    // Uncovered if fewer than MIN_KNOWN_PLAYERS have prior history.
    assert_eq!(MIN_KNOWN_PLAYERS, 3);
    let ra_too_few = roster_fact("team_a", "Alpha", ["r1", "r2", "r3", "a4", "a5"], roster_ts);
    let v_too_few = make_view("Alpha", "Beta", Some(ra_too_few), Some(rb), stats, t0);
    assert!(lineup_strength_feature(&v_too_few).unwrap().is_none());
}

#[test]
fn player_form_contrasts_recent_against_longer_run_and_is_antisymmetric() {
    let t0: Instant = "2026-06-01T12:00:00Z".parse().unwrap();
    let roster_ts = t0 - Duration::hours(12);

    let mut stats = BTreeMap::new();
    // Alpha players: long-run ~1.00 (100d ago), recent ~1.30 (3d ago) -> positive form.
    for (i, n) in ["a1", "a2", "a3", "a4", "a5"].iter().enumerate() {
        stats.insert(
            (*n).into(),
            app_slice(vec![
                (10 + i as i64, 100 + i as i64, n, "Alpha", 3, 1.00, t0 - Duration::days(100)),
                (20 + i as i64, 100 + i as i64, n, "Alpha", 3, 1.30, t0 - Duration::days(3)),
            ]),
        );
    }
    // Beta players: long-run ~1.20 (100d ago), recent ~0.85 (3d ago) -> negative form.
    for (i, n) in ["b1", "b2", "b3", "b4", "b5"].iter().enumerate() {
        stats.insert(
            (*n).into(),
            app_slice(vec![
                (30 + i as i64, 200 + i as i64, n, "Beta", 3, 1.20, t0 - Duration::days(100)),
                (40 + i as i64, 200 + i as i64, n, "Beta", 3, 0.85, t0 - Duration::days(3)),
            ]),
        );
    }

    let ra = roster_fact("team_a", "Alpha", ["a1", "a2", "a3", "a4", "a5"], roster_ts);
    let rb = roster_fact("team_b", "Beta", ["b1", "b2", "b3", "b4", "b5"], roster_ts);

    let v = make_view("Alpha", "Beta", Some(ra.clone()), Some(rb.clone()), stats.clone(), t0);
    let (form_ab, _) = player_form_feature(&v).unwrap().unwrap();
    assert!(form_ab > 0.05, "hot Alpha vs cold Beta must be positive: {form_ab}");

    let ra_swap = roster_fact("team_b", "Alpha", ["a1", "a2", "a3", "a4", "a5"], roster_ts);
    let rb_swap = roster_fact("team_a", "Beta", ["b1", "b2", "b3", "b4", "b5"], roster_ts);
    let v_swap = make_view("Beta", "Alpha", Some(rb_swap), Some(ra_swap), stats.clone(), t0);
    let (form_ba, _) = player_form_feature(&v_swap).unwrap().unwrap();
    assert!(close(form_ab, -form_ba, 1e-12));

    // Only 1 prior appearance per player -> insufficient baseline -> uncovered.
    let mut single_stats = BTreeMap::new();
    for (i, n) in ["a1", "a2", "a3", "a4", "a5"].iter().enumerate() {
        single_stats.insert(
            (*n).into(),
            app_slice(vec![(1, 100 + i as i64, n, "Alpha", 3, 1.25, t0 - Duration::days(3))]),
        );
    }
    for (i, n) in ["b1", "b2", "b3", "b4", "b5"].iter().enumerate() {
        single_stats.insert(
            (*n).into(),
            app_slice(vec![(2, 200 + i as i64, n, "Beta", 3, 0.90, t0 - Duration::days(3))]),
        );
    }
    let v_single = make_view("Alpha", "Beta", Some(ra), Some(rb), single_stats, t0);
    assert!(player_form_feature(&v_single).unwrap().is_none());
}

// ---------------------------------------------------------------------------
// Acceptance criterion 1: walk-forward Lab run + report + isolation
// ---------------------------------------------------------------------------

fn write_gz(path: &Path, bytes: &[u8]) {
    use flate2::{Compression, write::GzEncoder};
    use std::io::Write;
    let mut enc = GzEncoder::new(Vec::new(), Compression::default());
    enc.write_all(bytes).unwrap();
    std::fs::write(path, enc.finish().unwrap()).unwrap();
}

fn seed_multi_event_player_archive(invert_last_event_hltv: bool) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let hltv_dir = dir.path().join("hltv").join("matches");
    std::fs::create_dir_all(&hltv_dir).unwrap();

    // Valve standings snapshot before all synthetic events.
    let vrs_ts = "2026_01_01";
    let mut vrs = format!(
        "### Standings as of {vrs_ts}\n| Standing | Points | Team Name | Roster |\n| :- | :- | :- | :- |\n"
    );
    let teams: Vec<(i64, &str, [&str; 5])> = vec![
        (1, "Team Alpha", ["a1", "a2", "a3", "a4", "a5"]),
        (2, "Team Beta", ["b1", "b2", "b3", "b4", "b5"]),
        (3, "Team Gamma", ["g1", "g2", "g3", "g4", "g5"]),
        (4, "Team Delta", ["d1", "d2", "d3", "d4", "d5"]),
    ];
    for (rank, name, roster) in &teams {
        let pts = 1800 - rank * 50;
        vrs.push_str(&format!("| {rank} | {pts} | {name} | {} |\n", roster.join(", ")));
    }
    std::fs::write(dir.path().join("standings_global_2026_01_01.md"), vrs).unwrap();

    // 4 Events before the Holdout boundary (2026-11-25), spaced ~20 days apart
    // so both recent (30d) and baseline (180d) windows are populated:
    // Event 1 (Apr 1): 6 matches (history seed).
    // Event 2 (Apr 20): 32 matches (>= MIN_FIT 30 training rows for later Events).
    // Event 3 (May 10): 16 matches (evaluated; fitted on Events 1+2).
    // Event 4 (Jun 01): 12 matches (evaluated; fitted on Events 1+2+3).
    let event_specs: Vec<(i64, &str, &str, usize)> = vec![
        (9001, "Event One 2026", "2026-04-01", 6),
        (9002, "Event Two 2026", "2026-04-20", 32),
        (9003, "Event Three 2026", "2026-05-10", 16),
        (9004, "Event Four 2026", "2026-06-01", 12),
    ];

    let mut hltv_id = 500_000i64;
    let mut match_id = 600_000i64;

    for (eid, ename, start_date, count) in event_specs {
        let base_ts: Instant = format!("{start_date}T10:00:00Z").parse().unwrap();
        let rev_ts = base_ts - Duration::days(1);

        // Liquipedia roster revision for this Event before its start.
        let mut wikitext = String::from("{{TeamParticipants|\n");
        for (_, tname, players) in &teams {
            wikitext.push_str(&format!("{{{{Opponent|{tname}|players="));
            for p in players {
                wikitext.push_str(&format!("{{{{Person|{p}}}}}"));
            }
            wikitext.push_str("}}\n");
        }
        wikitext.push_str("}}");
        let roster_json = json!({
            "query": {
                "pages": [{
                    "title": format!("Test/{eid}"),
                    "revisions": [{
                        "revid": eid,
                        "timestamp": rev_ts.to_rfc3339(),
                        "slots": { "main": { "content": wikitext } }
                    }]
                }]
            }
        });
        write_gz(
            &dir.path().join(format!("roster-{eid}-{start_date}.json.gz")),
            serde_json::to_string(&roster_json).unwrap().as_bytes(),
        );

        let mut panda_rows = Vec::new();
        for idx in 0..count {
            match_id += 1;
            hltv_id += 1;
            let (ta_idx, tb_idx) = match idx % 4 {
                0 => (0, 1),
                1 => (2, 3),
                2 => (0, 2),
                _ => (1, 3),
            };
            let (ta_id, ta_name, ta_players) = teams[ta_idx];
            let (tb_id, tb_name, tb_players) = teams[tb_idx];
            // Alpha (0) and Gamma (2) are stronger than Beta (1) and Delta (3).
            let a_won = ta_idx < tb_idx;
            let winner_id = if a_won { ta_id } else { tb_id };
            let begin = base_ts + Duration::hours((idx as i64) * 5);
            let sched = begin - Duration::minutes(5);
            let end = begin + Duration::hours(2);

            panda_rows.push(json!({
                "id": match_id,
                "tournament_id": eid * 10,
                "serie_id": eid,
                "name": format!("Match {idx}"),
                "status": "finished",
                "forfeit": false,
                "number_of_games": 3,
                "scheduled_at": sched.to_rfc3339(),
                "begin_at": begin.to_rfc3339(),
                "end_at": end.to_rfc3339(),
                "winner_id": winner_id,
                "tournament": { "name": "Group Stage" },
                "league": { "name": "Test" },
                "serie": { "full_name": ename },
                "opponents": [
                    { "opponent": { "id": ta_id, "name": ta_name } },
                    { "opponent": { "id": tb_id, "name": tb_name } }
                ],
                "results": [
                    { "team_id": ta_id, "score": if a_won { 2 } else { 0 } },
                    { "team_id": tb_id, "score": if a_won { 0 } else { 2 } }
                ]
            }));

            let boost_last = invert_last_event_hltv && eid == 9004;
            let make_players = |t_idx: usize, plist: [&str; 5], won: bool| -> Vec<PlayerMatchStats> {
                let base_r = if won { 1.24 } else { 0.88 };
                let r = if boost_last { 2.0 - base_r } else { base_r };
                plist
                    .iter()
                    .enumerate()
                    .map(|(pi, nick)| PlayerMatchStats {
                        player_id: (t_idx as i64) * 100 + pi as i64 + 1,
                        nickname: (*nick).to_string(),
                        kills: 40,
                        deaths: 30,
                        ek_kills: None,
                        ek_deaths: None,
                        swing_pct: None,
                        adr: Some(80.0),
                        eadr: None,
                        kast_pct: Some(72.0),
                        ekast_pct: None,
                        rating: r,
                        rating_version: "3.0".to_string(),
                    })
                    .collect()
            };

            let rec = HltvMatchRecord {
                match_id: hltv_id,
                url: format!("https://www.hltv.org/matches/{hltv_id}/match"),
                fetched_at: end + Duration::minutes(10),
                event_id: Some(eid),
                event_name: Some(ename.to_string()),
                started_at: begin,
                format: "bo3".to_string(),
                is_lan: Some(true),
                team_a: TeamRef { id: ta_id, name: ta_name.to_string() },
                team_b: TeamRef { id: tb_id, name: tb_name.to_string() },
                maps: vec![
                    MapResult {
                        position: 1,
                        map_name: "Mirage".into(),
                        mapstats_id: None,
                        team_a_score: Some(13),
                        team_b_score: Some(8),
                        winner: None,
                        half_scores: None,
                        played: true,
                    },
                    MapResult {
                        position: 2,
                        map_name: "Nuke".into(),
                        mapstats_id: None,
                        team_a_score: Some(13),
                        team_b_score: Some(9),
                        winner: None,
                        half_scores: None,
                        played: true,
                    },
                ],
                veto: vec![],
                overall_stats: vec![
                    TeamPlayerStats {
                        team_id: Some(ta_id),
                        team_name: ta_name.to_string(),
                        players: make_players(ta_idx, ta_players, a_won),
                    },
                    TeamPlayerStats {
                        team_id: Some(tb_id),
                        team_name: tb_name.to_string(),
                        players: make_players(tb_idx, tb_players, !a_won),
                    },
                ],
                per_map_stats: vec![],
            };
            std::fs::write(
                hltv_dir.join(format!("hltv-match-{hltv_id}.json")),
                serde_json::to_string(&rec).unwrap(),
            )
            .unwrap();
        }
        write_gz(
            &dir.path().join(format!("panda-{}.json.gz", eid * 10)),
            serde_json::to_string(&panda_rows).unwrap().as_bytes(),
        );
    }
    dir
}

#[tokio::test]
async fn walk_forward_lab_run_evaluates_player_signals_and_isolates_later_events() {
    let base_dir = seed_multi_event_player_archive(false);
    let mut lab = Lab::new(ReplayArchive::load(base_dir.path()).unwrap()).unwrap();
    lab.signals = vec![Arc::new(LineupStrength), Arc::new(PlayerForm)];
    let outcome = lab.run().await.unwrap();

    assert_eq!(outcome.report.protocol_version, PROTOCOL_VERSION);
    for name in [lineup_strength::NAME, player_form::NAME] {
        let c = outcome
            .report
            .candidates
            .iter()
            .find(|c| c.name == name)
            .unwrap();
        assert_ne!(
            c.verdict,
            Verdict::NotTestable,
            "{name} should be testable: {}",
            c.reason
        );
        assert!(c.computed_on > 0, "{name} computed_on == 0");
        assert!(c.paired > 0, "{name} paired == 0");
        assert!(c.events >= 2, "{name} events: {}", c.events);
        assert!(c.difference.is_some(), "{name}");
        assert!(c.interval.is_some(), "{name}");
        assert!(c.detectable.is_some(), "{name}");

        let (verdict, reason) = stats::verdict(
            c.paired,
            c.interval.unwrap(),
            c.market.as_ref().map(|m| m.log_loss_difference),
            &lab.protocol,
        );
        assert_eq!(c.verdict, verdict, "{name}");
        assert_eq!(c.reason, reason, "{name}");
    }

    let md = outcome.report.markdown();
    assert!(md.contains("lineup_strength") && md.contains("player_form"));
    assert!(md.contains("1.00") || md.contains("prior"), "report should state the debut prior:\n{md}");

    // Walk-forward isolation: altering HLTV ratings in Event 4 (9004) must move
    // zero predictions in Events 1..=3.
    let inv_dir = seed_multi_event_player_archive(true);
    let mut inv_lab = Lab::new(ReplayArchive::load(inv_dir.path()).unwrap()).unwrap();
    inv_lab.signals = vec![Arc::new(LineupStrength), Arc::new(PlayerForm)];
    let inv_outcome = inv_lab.run().await.unwrap();

    let key = |o: &LabOutcome| -> BTreeMap<(String, i64), (i64, Option<f64>)> {
        o.predictions
            .iter()
            .map(|p| ((p.signal.clone(), p.match_id), (p.event_id, p.variant)))
            .collect()
    };
    let (before_map, after_map) = (key(&outcome), key(&inv_outcome));
    for (k, (eid, v_before)) in &before_map {
        if *eid < 9004 {
            assert_eq!(v_before, &after_map[k].1, "earlier event moved for {k:?}");
        }
    }
}
