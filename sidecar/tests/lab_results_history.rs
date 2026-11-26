//! Ticket #40 (spec #33): point-in-time results history and its three Candidate
//! Signals (`own_rating`, `recent_form`, `head_to_head`), plus direct comparison
//! of `own_rating` against the Valve rating.
use chrono::Duration;
use kallbet_ai::{
    adapters::{
        backtest::{Feeds, ReplayArchive, ReplayHistory},
        lab::{
            Lab, LabOutcome,
            signals::{
                self,
                head_to_head::{self, h2h_feature},
                own_rating::{self, Glicko2Table},
                recent_form::{self, fit_logit_slope, team_form},
            },
            stats::{self, Verdict},
        },
    },
    team_names::team_key,
    *,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    sync::Arc,
};

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/backtest")
}

fn whole() -> ReplayArchive {
    ReplayArchive::load(&fixtures()).unwrap()
}

fn close(a: f64, b: f64, eps: f64) -> bool {
    (a - b).abs() <= eps
}

// ---------------------------------------------------------------------------
// Acceptance criterion 1: point-in-time history before instant T
// ---------------------------------------------------------------------------

#[tokio::test]
async fn history_answers_only_matches_finished_strictly_before_t() {
    let archive = whole();
    let history = ReplayHistory::new(&archive);

    // Pick a team that plays multiple Matches across the archive (Vitality).
    let far_future: Instant = "2030-01-01T00:00:00Z".parse().unwrap();
    let all_vit = history.team_matches("Team Vitality", far_future).await.unwrap();
    let all_vit = all_vit.at(far_future).unwrap();
    assert!(all_vit.len() >= 10, "expected many Vitality Matches, got {}", all_vit.len());

    // Choose a Match strictly in the middle of Vitality's timeline.
    let pivot = all_vit[all_vit.len() / 2];
    let t = pivot.finished_at;

    // At T == pivot.finished_at: pivot itself (finished AT T) and anything after T MUST NOT appear.
    let at_t = history.team_matches("Vitality", t).await.unwrap();
    let at_t = at_t.at(t).unwrap();
    assert!(
        at_t.iter().all(|m| m.finished_at < t),
        "every returned Match must finish strictly before T"
    );
    assert!(
        at_t.iter().all(|m| m.match_id != pivot.match_id),
        "a Match finishing at T must not appear in history before T"
    );
    let expected_before = all_vit.iter().filter(|m| m.finished_at < t).count();
    assert_eq!(at_t.len(), expected_before);

    // At T + 1 second: pivot IS now included (finished 1s before cutoff).
    let after_t = t + Duration::seconds(1);
    let after_slice = history.team_matches("Vitality", after_t).await.unwrap();
    let after_matches = after_slice.at(after_t).unwrap();
    assert!(
        after_matches.iter().any(|m| m.match_id == pivot.match_id),
        "pivot must appear once cutoff passes finished_at"
    );
    assert!(after_matches.iter().all(|m| m.finished_at < after_t));

    // Same strict < T invariant holds for matches_before(T).
    let global_at_t = history.matches_before(t).await.unwrap();
    let global_matches = global_at_t.at(t).unwrap();
    assert!(global_matches.iter().all(|m| m.finished_at < t));
    assert!(global_matches.iter().all(|m| m.match_id != pivot.match_id));

    // Re-checking a slice obtained for a later cutoff against an earlier cutoff
    // fails with TemporalLeakError.
    let leak = after_slice.at(t).unwrap_err();
    assert!(leak.observed_at >= leak.cutoff);
}

/// A leaky ResultsHistory port returning a Match observed at the cutoff itself.
struct LeakyHistory;

#[async_trait::async_trait]
impl ResultsHistory for LeakyHistory {
    async fn team_matches(&self, team: &str, cutoff: Instant) -> Result<HistorySlice> {
        let m = HistoricalMatch {
            match_id: 999_999,
            event_id: 1,
            event_name: "Leak Event".into(),
            stage_id: 1,
            stage_name: "Stage 1".into(),
            stage_kind: HistoryStageKind::Swiss,
            round: Some(1),
            team_a_key: team_key(team),
            team_a_name: team.into(),
            team_b_key: "other".into(),
            team_b_name: "Other".into(),
            winner_key: team_key(team),
            format: Format::Bo1,
            maps_score: Some((1, 0)),
            scheduled_start: Some(cutoff),
            actual_start: cutoff,
            finished_at: cutoff,
        };
        Ok(HistorySlice::from_vec(vec![AsOf::new(
            m,
            cutoff,
            cutoff + Duration::days(1),
        )?]))
    }

    async fn matches_before(&self, cutoff: Instant) -> Result<HistorySlice> {
        self.team_matches("Vitality", cutoff).await
    }
}

#[tokio::test]
async fn a_leaky_results_history_port_aborts_the_lab_run() {
    let dir = tempfile::tempdir().unwrap();
    for entry in std::fs::read_dir(fixtures()).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        if name.starts_with("panda-22378") {
            std::fs::copy(&path, dir.path().join(&name)).unwrap();
        }
    }
    let archive = ReplayArchive::load(dir.path()).unwrap();
    let mut lab = Lab::new(archive).unwrap();
    let replay = Feeds::replay(&lab.archive).unwrap();
    lab.feeds = Feeds {
        history: Arc::new(LeakyHistory),
        ..replay
    };
    let err = lab.run().await.err().expect("leaky history must abort");
    assert!(err.is::<TemporalLeakError>(), "{err}");
}

// ---------------------------------------------------------------------------
// Acceptance criterion 3: rename / alias join on a real archive case
// ---------------------------------------------------------------------------

#[tokio::test]
async fn rename_or_alias_does_not_split_history_on_real_parivision_pvision_case() {
    let archive = whole();
    let history = ReplayHistory::new(&archive);

    // In the saved archive, PARIVISION (PandaScore id 133719) played in
    // StarLadder Major Budapest 2025 and IEM Cologne Major 2026, and then
    // appeared as PVISION (PandaScore id 139565) in PGL Major Singapore 2026.
    assert_eq!(team_key("PARIVISION"), team_key("PVISION"));

    // Find PVISION's very first Match in PGL Major Singapore 2026.
    let singapore_first = archive
        .stages
        .iter()
        .filter(|s| s.event.contains("Singapore"))
        .flat_map(|s| &s.matches)
        .filter(|m| {
            m.game.team_a.name == "PVISION" || m.game.team_b.name == "PVISION"
        })
        .min_by_key(|m| m.cutoff())
        .expect("PVISION plays in PGL Major Singapore 2026");

    // Asking the history for "PVISION" before its very first Singapore Match
    // MUST return PARIVISION's earlier Budapest 2025 and Cologne 2026 Matches.
    let cutoff = singapore_first.cutoff();
    let prior_as_pvision = history.team_matches("PVISION", cutoff).await.unwrap();
    let prior_as_parivision = history.team_matches("PARIVISION", cutoff).await.unwrap();
    let pvision_matches = prior_as_pvision.at(cutoff).unwrap();
    let parivision_matches = prior_as_parivision.at(cutoff).unwrap();

    assert_eq!(pvision_matches, parivision_matches);
    assert!(
        pvision_matches.len() >= 5,
        "PVISION must inherit PARIVISION's prior Matches, got {}",
        pvision_matches.len()
    );
    assert!(
        pvision_matches
            .iter()
            .any(|m| m.event_name.contains("Budapest") && (m.team_a_name == "PARIVISION" || m.team_b_name == "PARIVISION")),
        "Budapest PARIVISION Matches must be present under PVISION"
    );
    assert!(
        pvision_matches
            .iter()
            .any(|m| m.event_name.contains("Cologne") && (m.team_a_name == "PARIVISION" || m.team_b_name == "PARIVISION")),
        "Cologne PARIVISION Matches must be present under PVISION"
    );

    // After Singapore finishes, querying by "PVISION", "PARIVISION", or "PV"
    // returns one continuous history containing both names.
    let far_future: Instant = "2030-01-01T00:00:00Z".parse().unwrap();
    let full = history.team_matches("PV", far_future).await.unwrap();
    let full = full.at(far_future).unwrap();
    assert!(full.iter().any(|m| m.team_a_name == "PARIVISION" || m.team_b_name == "PARIVISION"));
    assert!(full.iter().any(|m| m.team_a_name == "PVISION" || m.team_b_name == "PVISION"));

    // Also verify another real alias pair: "NAVI" <-> "Natus Vincere", "MongolZ" <-> "TheMongolz".
    assert_eq!(
        history.team_matches("NAVI", far_future).await.unwrap().len(),
        history.team_matches("Natus Vincere", far_future).await.unwrap().len()
    );
    assert!(history.team_matches("NAVI", far_future).await.unwrap().len() > 20);
    assert_eq!(
        history.team_matches("MongolZ", far_future).await.unwrap().len(),
        history.team_matches("TheMongolz", far_future).await.unwrap().len()
    );
}

// ---------------------------------------------------------------------------
// Acceptance criterion 2: rating updated only with Matches finished before cutoff
// ---------------------------------------------------------------------------

#[tokio::test]
async fn own_rating_uses_only_matches_finished_before_each_match_cutoff() {
    let mut base_own = Lab::new(whole()).unwrap();
    base_own.signals = vec![
        Arc::new(own_rating::OwnRating),
        Arc::new(recent_form::RecentForm),
        Arc::new(head_to_head::HeadToHead),
    ];
    let base_out = base_own.run().await.unwrap();

    // Reverse a Match in the middle of IEM Cologne Major 2026 (not the first, not the last).
    let mut mutated = whole();
    let cologne_id = mutated
        .stages
        .iter()
        .find(|s| s.event.contains("Cologne") && s.event.contains("2026"))
        .unwrap()
        .event_id;
    let mut cologne_matches: Vec<(i64, Instant, Instant)> = mutated
        .stages
        .iter()
        .filter(|s| s.event_id == cologne_id)
        .flat_map(|s| &s.matches)
        .map(|m| (m.game.id, m.cutoff(), m.finished_at))
        .collect();
    cologne_matches.sort_by_key(|x| (x.2, x.0));
    let (target_id, _target_cutoff, target_finished) = cologne_matches[cologne_matches.len() / 2];

    for stage in &mut mutated.stages {
        for m in &mut stage.matches {
            if m.game.id == target_id {
                m.winner = if m.winner == m.game.team_a.id {
                    m.game.team_b.id
                } else {
                    m.game.team_a.id
                };
            }
        }
    }

    let mut cutoffs: BTreeMap<i64, Instant> = BTreeMap::new();
    for s in &mutated.stages {
        for m in &s.matches {
            cutoffs.insert(m.game.id, m.cutoff());
        }
    }

    let mut mut_lab = Lab::new(mutated).unwrap();
    mut_lab.signals = vec![
        Arc::new(own_rating::OwnRating),
        Arc::new(recent_form::RecentForm),
        Arc::new(head_to_head::HeadToHead),
    ];
    let mut_out = mut_lab.run().await.unwrap();

    let key = |o: &LabOutcome| -> BTreeMap<(String, i64), Option<f64>> {
        o.predictions
            .iter()
            .map(|p| ((p.signal.clone(), p.match_id), p.variant))
            .collect()
    };
    let (before_map, after_map) = (key(&base_out), key(&mut_out));
    assert_eq!(before_map.len(), after_map.len());

    let mut own_moved_after = 0usize;
    for ((sig, mid), v_before) in &before_map {
        let v_after = &after_map[&(sig.clone(), *mid)];
        if cutoffs[mid] <= target_finished {
            // Every Match whose cutoff is at or before target's finished_at
            // (including earlier Matches in Cologne itself!) MUST NOT move.
            assert_eq!(
                v_before, v_after,
                "{sig} on Match {mid} (cutoff {} <= target finished {}) moved!",
                cutoffs[mid], target_finished
            );
        } else if sig == own_rating::NAME && v_before != v_after {
            own_moved_after += 1;
        }
    }
    assert!(
        own_moved_after > 0,
        "reversing a finished Match must update own_rating for subsequent Matches"
    );
}

// ---------------------------------------------------------------------------
// Pure model / math unit tests (Glicko-2, recent form, H2H, logit slope)
// ---------------------------------------------------------------------------

fn synthetic_match(
    id: i64,
    a: &str,
    b: &str,
    winner: &str,
    format: Format,
    finished_at: Instant,
) -> HistoricalMatch {
    HistoricalMatch {
        match_id: id,
        event_id: 1,
        event_name: "Test Event".into(),
        stage_id: 1,
        stage_name: "Stage 1".into(),
        stage_kind: HistoryStageKind::Swiss,
        round: Some(1),
        team_a_key: team_key(a),
        team_a_name: a.into(),
        team_b_key: team_key(b),
        team_b_name: b.into(),
        winner_key: team_key(winner),
        format,
        maps_score: None,
        scheduled_start: Some(finished_at - Duration::hours(2)),
        actual_start: finished_at - Duration::hours(2),
        finished_at,
    }
}

#[test]
fn glicko2_ratings_update_monotonically_and_are_symmetric() {
    let t0: Instant = "2026-01-01T12:00:00Z".parse().unwrap();
    let m1 = synthetic_match(1, "Alpha", "Beta", "Alpha", Format::Bo3, t0);
    let m2 = synthetic_match(2, "Alpha", "Gamma", "Alpha", Format::Bo3, t0 + Duration::days(1));
    let m3 = synthetic_match(3, "Beta", "Gamma", "Gamma", Format::Bo1, t0 + Duration::days(2));
    let refs = [&m1, &m2, &m3];

    let table = Glicko2Table::from_matches(&refs);
    let cutoff = t0 + Duration::days(5);

    let ra = table.get("alpha").unwrap();
    let rb = table.get("beta").unwrap();
    let rg = table.get("gamma").unwrap();
    assert!(ra.mu > rg.mu && rg.mu > rb.mu, "alpha > gamma > beta: {} {} {}", ra.mu, rg.mu, rb.mu);

    // Exact anti-symmetry when swapping team A and team B.
    let x_ab = table.logit_diff("alpha", "beta", cutoff).unwrap();
    let x_ba = table.logit_diff("beta", "alpha", cutoff).unwrap();
    assert!(close(x_ab, -x_ba, 1e-12));
    assert!(x_ab > 0.0);

    // Unseen team has no rating.
    assert!(table.logit_diff("alpha", "unknown", cutoff).is_none());

    // Inactivity increases rating deviation phi, shrinking the expected logit toward 0.
    let x_soon = table.logit_diff("alpha", "beta", t0 + Duration::days(3)).unwrap();
    let x_late = table.logit_diff("alpha", "beta", t0 + Duration::days(180)).unwrap();
    assert!(x_soon > x_late && x_late > 0.0, "{x_soon} vs {x_late}");
}

#[test]
fn recent_form_weights_opponent_strength_and_ignores_stale_matches() {
    let t0: Instant = "2026-01-01T12:00:00Z".parse().unwrap();
    // Build a strong team (Strong) and a weak team (Weak).
    let mut matches = Vec::new();
    for i in 0..6 {
        matches.push(synthetic_match(
            i + 1,
            "Strong",
            "Weak",
            "Strong",
            Format::Bo3,
            t0 + Duration::days(i),
        ));
    }
    let recent_day = t0 + Duration::days(60);
    // Team X beats Strong recently; Team Y beats Weak on the exact same day.
    let mx = synthetic_match(100, "TeamX", "Strong", "TeamX", Format::Bo3, recent_day);
    let my = synthetic_match(101, "TeamY", "Weak", "TeamY", Format::Bo3, recent_day);
    // Stale loss for Team X 90 days before cutoff (outside 45-day window).
    let stale = synthetic_match(99, "TeamX", "Weak", "Weak", Format::Bo3, t0);

    let all: Vec<&HistoricalMatch> = matches
        .iter()
        .chain([&stale, &mx, &my])
        .collect();
    let table = Glicko2Table::from_matches(&all);
    let cutoff = recent_day + Duration::days(2);

    let form_x = team_form("teamx", &[&stale, &mx], &table, cutoff).unwrap();
    let form_y = team_form("teamy", &[&my], &table, cutoff).unwrap();
    assert!(
        form_x > form_y && form_y > 0.0,
        "beating Strong must give higher form than beating Weak: {form_x} vs {form_y}"
    );
    // A team with only stale Matches has None.
    assert!(team_form("teamx", &[&stale], &table, cutoff).is_none());
}

#[test]
fn head_to_head_feature_is_antisymmetric_and_decays_with_age() {
    let t0: Instant = "2026-01-01T12:00:00Z".parse().unwrap();
    let old_win_a = synthetic_match(1, "Alpha", "Beta", "Alpha", Format::Bo3, t0);
    let new_win_b = synthetic_match(
        2,
        "Beta",
        "Alpha",
        "Beta",
        Format::Bo3,
        t0 + Duration::days(300),
    );
    let cutoff = t0 + Duration::days(310);

    let (fa, latest_a) = h2h_feature("alpha", "beta", &[&old_win_a, &new_win_b], cutoff).unwrap();
    let (fb, latest_b) = h2h_feature("beta", "alpha", &[&old_win_a, &new_win_b], cutoff).unwrap();
    assert!(close(fa, -fb, 1e-12));
    assert_eq!(latest_a, new_win_b.finished_at);
    assert_eq!(latest_b, new_win_b.finished_at);
    // Beta's win is 300 days fresher than Alpha's win, so Alpha's H2H is negative.
    assert!(fa < 0.0, "more recent Beta win should outweigh older Alpha win: {fa}");
    // No meetings -> None.
    assert!(h2h_feature("alpha", "gamma", &[&old_win_a, &new_win_b], cutoff).is_none());
}

#[test]
fn fit_logit_slope_recovers_sign_and_is_symmetric() {
    let data: Vec<(f64, f64, bool)> = (0..100)
        .map(|i| {
            let x = if i % 2 == 0 { 1.0 } else { -1.0 };
            let won = if x > 0.0 { i % 10 != 0 } else { i % 10 == 1 };
            (0.0, x, won)
        })
        .collect();
    let beta = fit_logit_slope(&data, 1.0).unwrap();
    assert!(beta > 0.5, "positive feature correlation should give positive slope: {beta}");

    let mirrored: Vec<(f64, f64, bool)> = data.iter().map(|(b, x, w)| (-b, -x, !w)).collect();
    let beta_m = fit_logit_slope(&mirrored, 1.0).unwrap();
    assert!(close(beta, beta_m, 1e-6), "{beta} vs {beta_m}");
    assert!(fit_logit_slope(&[], 1.0).is_none());
}

// ---------------------------------------------------------------------------
// Acceptance criterion 4: all three signals reported + own_rating vs Valve
// ---------------------------------------------------------------------------

#[tokio::test]
async fn all_three_signals_are_reported_and_own_rating_is_compared_directly_with_valve() {
    let names: Vec<_> = signals::registry().iter().map(|s| s.name()).collect();
    for expected in [own_rating::NAME, recent_form::NAME, head_to_head::NAME] {
        assert!(names.contains(&expected), "{expected} missing from registry: {names:?}");
    }

    let lab = Lab::new(whole()).unwrap();
    let outcome = lab.run().await.unwrap();
    let report = &outcome.report;

    let winners: BTreeMap<i64, (i64, bool)> = lab
        .archive
        .stages
        .iter()
        .flat_map(|s| &s.matches)
        .map(|m| (m.game.id, (m.game.event_id, m.winner == m.game.team_a.id)))
        .collect();

    for name in [own_rating::NAME, recent_form::NAME, head_to_head::NAME] {
        let c = report
            .candidates
            .iter()
            .find(|c| c.name == name)
            .unwrap_or_else(|| panic!("{name} missing from report"));
        assert!(c.computed_on > 0, "{name} computed_on == 0");
        assert!(c.paired > 0, "{name} paired == 0");
        assert!(c.difference.is_some(), "{name} missing difference");
        assert!(c.interval.is_some(), "{name} missing interval");
        assert!(c.detectable.is_some(), "{name} missing detectable");
        assert_ne!(c.verdict, Verdict::NotTestable, "{name} unexpectedly not testable: {}", c.reason);

        // Verify the verdict matches the written protocol rule.
        let (verdict, reason) = stats::verdict(
            c.paired,
            c.interval.unwrap(),
            c.accuracy_interval,
            c.market.as_ref().map(|m| m.log_loss_difference),
            &lab.protocol,
        );
        assert_eq!(c.verdict, verdict, "{name}");
        assert_eq!(c.reason, reason, "{name}");
    }

    // Verify own_rating's direct comparison with Valve rating (unblended).
    let own = report
        .candidates
        .iter()
        .find(|c| c.name == own_rating::NAME)
        .unwrap();
    let vs_valve = own
        .vs_valve
        .as_ref()
        .expect("own_rating must include a direct comparison with the Valve rating");
    assert!(vs_valve.paired > 100, "paired vs Valve: {}", vs_valve.paired);
    assert!(vs_valve.events >= 4, "events vs Valve: {}", vs_valve.events);
    assert!(close(
        vs_valve.difference.log_loss,
        vs_valve.variant.log_loss - vs_valve.baseline.log_loss,
        1e-12
    ));
    assert!(close(
        vs_valve.difference.brier,
        vs_valve.variant.brier - vs_valve.baseline.brier,
        1e-12
    ));
    assert!(close(
        vs_valve.difference.accuracy,
        vs_valve.variant.accuracy - vs_valve.baseline.accuracy,
        1e-12
    ));
    assert_eq!(
        vs_valve.detectable,
        stats::detectable_effect(vs_valve.std_error)
    );
    assert!(vs_valve.interval.low <= vs_valve.interval.high);

    // Other signals do not emit a vs_valve block.
    for other in ["rating_calibration", recent_form::NAME, head_to_head::NAME] {
        let c = report.candidates.iter().find(|c| c.name == other).unwrap();
        assert!(c.vs_valve.is_none(), "{other} should not have vs_valve");
    }

    // Recompute own_rating's paired metrics against the anchor from predictions.
    let own_rows: Vec<_> = outcome
        .predictions
        .iter()
        .filter(|p| p.signal == own_rating::NAME)
        .collect();
    assert_eq!(own_rows.len(), winners.len());
    let paired_own: Vec<_> = own_rows
        .iter()
        .filter_map(|p| Some((p.match_id, p.baseline?, p.variant?)))
        .collect();
    assert_eq!(own.paired, paired_own.len());
    let own_events: BTreeSet<i64> = paired_own.iter().map(|(id, _, _)| winners[id].0).collect();
    assert_eq!(own.events, own_events.len());

    // Markdown and JSON include all three signals and the direct Valve comparison.
    let md = report.markdown();
    for needle in [
        "own_rating",
        "recent_form",
        "head_to_head",
        "Direct comparison with Valve rating",
    ] {
        assert!(md.contains(needle), "{needle} missing from markdown:\n{md}");
    }
    let json = serde_json::to_value(report).unwrap();
    let own_json = json["candidates"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == own_rating::NAME)
        .unwrap();
    assert!(own_json["vs_valve"].is_object(), "vs_valve missing in JSON");
}

// Finding 1 (review of 6eea194): a Match whose finish is not reported is not
// yet visible. A start time is not a finish: play may last for hours.
#[tokio::test]
async fn a_match_without_a_reported_finish_never_enters_the_history() {
    let mut archive = whole();
    let m = &mut archive.stages[0].matches[0];
    let id = m.game.id;
    // What the loader makes of a finished row without `end_at`.
    m.reported_finish = None;
    m.finished_at = m.actual_start;
    let kickoff = m.actual_start;
    let history = ReplayHistory::new(&archive);
    for cutoff in [
        kickoff + Duration::minutes(5),
        "2030-01-01T00:00:00Z".parse().unwrap(),
    ] {
        let all = history.matches_before(cutoff).await.unwrap();
        assert!(all.at(cutoff).unwrap().iter().all(|h| h.match_id != id));
        let team = archive.stages[0].matches[0].game.team_a.name.clone();
        let own = history.team_matches(&team, cutoff).await.unwrap();
        assert!(own.at(cutoff).unwrap().iter().all(|h| h.match_id != id));
    }
}
