//! Ticket #45 (spec #33): player-level Candidate Signals (`lineup_strength`
//! and `player_form`).
//!
//! Per the #36 data research (`docs/signal-lab/map-veto-player-research.md`)
//! and operator decision, no permitted source of point-in-time player
//! statistics exists (PandaScore free plan returns 403, Liquipedia carries
//! rosters without per-Match player statistics, GRID Open Access excludes
//! Majors/ESL/BLAST, and HLTV scraping is not authorized by the operator).
//! Both signals therefore register cleanly as `not testable` with that reason
//! and build no model, leaving the extension seam intact for a future source.
use kallbet_ai::{
    adapters::{
        backtest::ReplayArchive,
        lab::{
            CandidateSignal, Lab, PROTOCOL_VERSION,
            signals::{self, lineup_strength, player_form},
            stats::Verdict,
        },
    },
    *,
};
use serde_json::Value;
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/backtest")
}

fn whole() -> ReplayArchive {
    ReplayArchive::load(&fixtures()).unwrap()
}

fn run_lab(archive: &Path, out: &Path, extra: &[&str]) -> std::process::Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_kallbet-ai"))
        .arg("lab")
        .arg("--archive")
        .arg(archive)
        .arg("--out")
        .arg(out)
        .args(extra)
        .env_remove("SIDECAR_CONFIG")
        .env_remove("ODDSPAPI_API_KEY")
        .output()
        .unwrap()
}

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

#[test]
fn both_player_signals_report_missing_data_with_research_reason_and_build_no_model() {
    let archive = whole();
    for signal in [
        &lineup_strength::LineupStrength as &dyn CandidateSignal,
        &player_form::PlayerForm as &dyn CandidateSignal,
    ] {
        let reason = signal
            .missing_data(&archive)
            .expect("missing_data must explain why player stats are unavailable");
        let lower = reason.to_lowercase();
        assert!(
            lower.contains("player statistics") || lower.contains("player stats"),
            "{}: {reason}",
            signal.name()
        );
        assert!(
            reason.contains("403") && reason.contains("PandaScore"),
            "{}: {reason}",
            signal.name()
        );
        assert!(
            reason.contains("HLTV") && lower.contains("not authorized"),
            "{}: {reason}",
            signal.name()
        );

        // Even if fit() is called directly, no model is built and predict() yields None.
        let fitted = signal.fit(&[]).unwrap();
        let stage = &archive.stages[0];
        let rm = &stage.matches[0];
        let view = kallbet_ai::adapters::lab::MatchView {
            game: rm.game.clone(),
            cutoff: rm.cutoff(),
            signal: None,
            market: None,
            history_before: HistorySlice::default(),
            team_a_history: HistorySlice::default(),
            team_b_history: HistorySlice::default(),
        };
        assert!(fitted.predict(&view).unwrap().is_none(), "{}", signal.name());
        assert!(!signal.needs().is_empty());
    }
}

#[tokio::test]
async fn lab_run_reports_lineup_strength_and_player_form_as_not_testable() {
    let lab = Lab::new(whole()).unwrap();
    let total_matches: usize = lab.archive.stages.iter().map(|s| s.matches.len()).sum();
    let outcome = lab.run().await.unwrap();

    assert_eq!(outcome.report.protocol_version, PROTOCOL_VERSION);

    for name in [lineup_strength::NAME, player_form::NAME] {
        let candidate = outcome
            .report
            .candidates
            .iter()
            .find(|c| c.name == name)
            .unwrap_or_else(|| panic!("{name} missing from lab report"));

        assert_eq!(candidate.verdict, Verdict::NotTestable, "{name}");
        assert_eq!(candidate.computed_on, 0, "{name}");
        assert_eq!(candidate.paired, 0, "{name}");
        assert_eq!(candidate.events, 0, "{name}");
        assert!(candidate.baseline.is_none(), "{name}");
        assert!(candidate.variant.is_none(), "{name}");
        assert!(candidate.difference.is_none(), "{name}");
        assert!(candidate.interval.is_none(), "{name}");
        assert!(candidate.std_error.is_none(), "{name}");
        assert!(candidate.detectable.is_none(), "{name}");
        assert!(candidate.market.is_none(), "{name}");
        assert!(candidate.vs_valve.is_none(), "{name}");
        assert!(
            candidate.reason.contains("403") && candidate.reason.contains("HLTV"),
            "{name} reason: {}",
            candidate.reason
        );

        let rows: Vec<_> = outcome
            .predictions
            .iter()
            .filter(|p| p.signal == name)
            .collect();
        assert_eq!(rows.len(), total_matches, "{name}");
        assert!(rows.iter().all(|r| r.variant.is_none()), "{name}");
    }

    let md = outcome.report.markdown();
    for needle in [
        "`lineup_strength` | not testable | 0 | 0 | 0",
        "`player_form` | not testable | 0 | 0 | 0",
        "## `lineup_strength`: not testable",
        "## `player_form`: not testable",
    ] {
        assert!(md.contains(needle), "{needle} missing from markdown:\n{md}");
    }
}

#[test]
fn lab_cli_accepts_player_signals_and_records_not_testable_in_json_and_audit() {
    let out = tempfile::tempdir().unwrap();
    let run = run_lab(
        &fixtures(),
        out.path(),
        &[
            "--seed",
            "33",
            "--signal",
            "lineup_strength",
            "--signal",
            "player_form",
        ],
    );
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );

    let json: Value =
        serde_json::from_slice(&std::fs::read(out.path().join("report.json")).unwrap()).unwrap();
    assert_eq!(json["protocol_version"], PROTOCOL_VERSION);
    let candidates = json["candidates"].as_array().unwrap();
    assert_eq!(candidates.len(), 2);
    for name in ["lineup_strength", "player_form"] {
        let entry = candidates
            .iter()
            .find(|c| c["name"] == name)
            .unwrap_or_else(|| panic!("{name} missing from report.json"));
        assert_eq!(entry["verdict"], "not testable");
        assert_eq!(entry["computed_on"], 0);
        assert!(entry["reason"].as_str().unwrap().contains("HLTV"));
    }

    let runs = tokio::runtime::Runtime::new().unwrap().block_on(async {
        let audit = Audit::open(&out.path().join("audit.sqlite")).await.unwrap();
        audit.lab_runs().await.unwrap()
    });
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].verdicts["lineup_strength"], "not testable");
    assert_eq!(runs[0].verdicts["player_form"], "not testable");
}
