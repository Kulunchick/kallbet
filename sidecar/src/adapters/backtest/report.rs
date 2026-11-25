//! Scores and the benchmark report: accuracy (by Bo1/Bo3/Bo5), log-loss and
//! Brier score of each profile's submitted Predictions, the override hit-rate,
//! and Pick'em Points by KallBet's rules.
use super::archive::{ReplayStage, StageKind};
use super::platform::Submitted;
use serde::Serialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

/// One scored Prediction: the probability given to the picked team.
#[derive(Clone, Copy)]
pub struct Outcome {
    pub p_pick: f64,
    pub hit: bool,
    pub override_market: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Scores {
    pub matches: usize,
    pub correct: usize,
    pub accuracy: Option<f64>,
    /// Mean negative natural log of the probability given to the winner.
    pub log_loss: Option<f64>,
    /// Mean squared error of the probability given to the winner.
    pub brier: Option<f64>,
    /// Predictions against the market favourite.
    pub overrides: usize,
    pub override_hits: usize,
    pub override_hit_rate: Option<f64>,
}
impl Scores {
    pub fn of(outcomes: &[Outcome]) -> Self {
        let n = outcomes.len();
        let mean = |f: &dyn Fn(&Outcome) -> f64| {
            (n > 0).then(|| outcomes.iter().map(f).sum::<f64>() / n as f64)
        };
        let winner_p = |o: &Outcome| if o.hit { o.p_pick } else { 1.0 - o.p_pick };
        let overrides: Vec<_> = outcomes.iter().filter(|o| o.override_market).collect();
        let override_hits = overrides.iter().filter(|o| o.hit).count();
        Self {
            matches: n,
            correct: outcomes.iter().filter(|o| o.hit).count(),
            accuracy: mean(&|o| f64::from(u8::from(o.hit))),
            log_loss: mean(&|o| -winner_p(o).clamp(1e-6, 1.0).ln()),
            brier: mean(&|o| (1.0 - winner_p(o)).powi(2)),
            overrides: overrides.len(),
            override_hits,
            override_hit_rate: (!overrides.is_empty())
                .then(|| override_hits as f64 / overrides.len() as f64),
        }
    }
}

/// The outcome of a submitted Prediction, or `None` when it carries no
/// probability in `[0, 1]` (it is then not scored, and the runner says so).
pub fn outcome(p: &Submitted, winner: i64) -> Option<Outcome> {
    let p_pick = p.prediction.rationale["p_win"]
        .as_f64()
        .filter(|p| (0.0..=1.0).contains(p))?;
    Some(Outcome {
        p_pick,
        hit: p.prediction.team_id == winner,
        override_market: p.prediction.rationale["override_market"] == true,
    })
}

/// Pick'em Points of one submission: Swiss by KallBet's rules (1 per correct
/// `three_nil` 3-0, `zero_three` 0-3, `advance` 3-1/3-2 among settled teams);
/// Playoffs 1 per bracket slot whose picked winner won it. Returns
/// (points, picks scored so far, picks).
pub fn pickem_points(stage: &ReplayStage, payload: &Value) -> (usize, usize, usize) {
    let mut wins: BTreeMap<i64, usize> = BTreeMap::new();
    let mut losses: BTreeMap<i64, usize> = BTreeMap::new();
    for (w, l) in &stage.results {
        *wins.entry(*w).or_default() += 1;
        *losses.entry(*l).or_default() += 1;
    }
    let record = |id: i64| {
        (
            wins.get(&id).copied().unwrap_or(0),
            losses.get(&id).copied().unwrap_or(0),
        )
    };
    match stage.kind {
        StageKind::Swiss => {
            let mut points = 0;
            let mut settled = 0;
            let mut picks = 0;
            for (bucket, ok) in [
                ("three_nil", &(|r: (usize, usize)| r == (3, 0)) as &dyn Fn((usize, usize)) -> bool),
                ("zero_three", &|r| r == (0, 3)),
                ("advance", &|r: (usize, usize)| r.0 == 3 && r.1 > 0),
            ] {
                for id in payload[bucket].as_array().into_iter().flatten() {
                    picks += 1;
                    let r = record(id.as_i64().unwrap_or_default());
                    if r.0 == 3 || r.1 == 3 {
                        settled += 1;
                        points += usize::from(ok(r));
                    }
                }
            }
            (points, settled, picks)
        }
        StageKind::Playoffs => {
            // In single elimination a team won its slot `rounds_before_final = r`
            // exactly when it won `depth - r + 1` series; picks are path-consistent,
            // so that slot is the one on its path.
            let slots: Vec<&Value> = payload["picks"].as_array().into_iter().flatten().collect();
            let depth = slots
                .iter()
                .filter_map(|s| s["rounds_before_final"].as_u64())
                .max()
                .unwrap_or(0) as usize;
            let mut points = 0;
            let mut decided = 0;
            for s in &slots {
                let id = s["team_id"].as_i64().unwrap_or_default();
                let r = s["rounds_before_final"].as_u64().unwrap_or(0) as usize;
                let needed = depth - r + 1;
                let (w, l) = record(id);
                if w >= needed {
                    points += 1;
                    decided += 1;
                } else if l > 0 {
                    decided += 1;
                }
            }
            (points, decided, slots.len())
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ScoreRow {
    pub profile: String,
    /// `all`, `bo1`, `bo3` or `bo5`.
    pub format: String,
    pub scores: Scores,
}

#[derive(Clone, Debug, Serialize)]
pub struct Table {
    pub title: String,
    pub note: String,
    pub rows: Vec<ScoreRow>,
}

/// Scores `profiles` on `matches`: every profile on the Matches it predicted
/// (`paired = false`), or all of them only on the Matches every one of them
/// predicted (`paired = true`).
pub fn table(
    title: &str,
    note: &str,
    stages: &[ReplayStage],
    matches: &BTreeSet<i64>,
    predictions: &[Submitted],
    profiles: &[(&str, &str)],
    paired: bool,
) -> Table {
    let games: BTreeMap<i64, (&str, i64)> = stages
        .iter()
        .flat_map(|s| s.matches.iter())
        .filter(|m| matches.contains(&m.game.id))
        .map(|m| {
            let format = match m.game.format {
                crate::Format::Bo1 => "bo1",
                crate::Format::Bo3 => "bo3",
                crate::Format::Bo5 => "bo5",
            };
            (m.game.id, (format, m.winner))
        })
        .collect();
    let by_bot = |bot: &str| -> BTreeMap<i64, &Submitted> {
        predictions
            .iter()
            .filter(|p| p.bot == bot && games.contains_key(&p.match_id))
            .map(|p| (p.match_id, p))
            .collect()
    };
    let all: Vec<BTreeMap<i64, &Submitted>> = profiles.iter().map(|(key, _)| by_bot(key)).collect();
    let common: BTreeSet<i64> = games
        .keys()
        .filter(|id| all.iter().all(|m| m.contains_key(id)))
        .copied()
        .collect();
    let mut rows = Vec::new();
    for ((_, label), mine) in profiles.iter().zip(&all) {
        for format in ["all", "bo1", "bo3", "bo5"] {
            let outcomes: Vec<Outcome> = mine
                .iter()
                .filter(|(id, _)| !paired || common.contains(id))
                .filter(|(id, _)| format == "all" || games[id].0 == format)
                .filter_map(|(id, p)| outcome(p, games[id].1))
                .collect();
            if !outcomes.is_empty() || format == "all" {
                rows.push(ScoreRow {
                    profile: (*label).to_owned(),
                    format: format.to_owned(),
                    scores: Scores::of(&outcomes),
                });
            }
        }
    }
    Table {
        title: title.to_owned(),
        note: note.to_owned(),
        rows,
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct StageRow {
    pub stage: String,
    pub kind: StageKind,
    pub complete: bool,
    pub scored_matches: usize,
    pub unscored_matches: usize,
    pub with_vrs: usize,
    pub with_market: usize,
    pub with_rosters: usize,
    pub opening_view: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct PickemRow {
    pub stage: String,
    pub kind: StageKind,
    /// `final`, `pending` (Stage not finished) or why there is no Pick'em.
    pub status: String,
    pub points: Option<usize>,
    pub settled_picks: usize,
    pub picks: usize,
    /// Pairwise ratings the solver had (0: it picked from even odds).
    pub rated_pairs: usize,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct LlmTrack {
    pub providers: Vec<String>,
    pub planned: BTreeMap<String, usize>,
    pub answered: BTreeMap<String, usize>,
    pub calls: BTreeMap<String, usize>,
    /// Matches without an LLM answer, by profile and error kind.
    pub failures: BTreeMap<String, usize>,
    pub stopped: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Report {
    pub archive_digest: String,
    pub archive_files: usize,
    pub profiles: Vec<(String, String)>,
    pub stages: Vec<StageRow>,
    pub tables: Vec<Table>,
    pub pickem: Vec<PickemRow>,
    pub llm: LlmTrack,
    pub notes: Vec<String>,
}

fn pct(v: Option<f64>) -> String {
    v.map_or("–".into(), |v| format!("{:.1}%", v * 100.0))
}
fn num(v: Option<f64>) -> String {
    v.map_or("–".into(), |v| format!("{v:.3}"))
}

impl Report {
    pub fn markdown(&self) -> String {
        let mut out = String::new();
        let mut line = |s: String| {
            out.push_str(&s);
            out.push('\n');
        };
        line("# Retrospective benchmark".into());
        line(String::new());
        line(format!(
            "Archive: {} files, SHA-256 `{}`. Regenerate with `cargo run -p kallbet-ai -- backtest` (see `docs/agents/codebase.md`, ticket #31).",
            self.archive_files, self.archive_digest
        ));
        line(String::new());
        line("Profiles:".into());
        for (key, label) in &self.profiles {
            line(format!("- `{key}`: {label}"));
        }
        line(String::new());
        line("## Data per Stage".into());
        line(String::new());
        line("| Stage | Kind | Finished | Scored Matches | Not scored | VRS | Market | Rosters | Pick'em view |".into());
        line("|---|---|---|--:|--:|--:|--:|--:|---|".into());
        for s in &self.stages {
            line(format!(
                "| {} | {:?} | {} | {} | {} | {} | {} | {} | {} |",
                s.stage,
                s.kind,
                if s.complete { "yes" } else { "no" },
                s.scored_matches,
                s.unscored_matches,
                s.with_vrs,
                s.with_market,
                s.with_rosters,
                s.opening_view.as_deref().unwrap_or("Liquipedia revision before the Stage")
            ));
        }
        for t in &self.tables {
            line(String::new());
            line(format!("## {}", t.title));
            line(String::new());
            if !t.note.is_empty() {
                line(t.note.clone());
                line(String::new());
            }
            line("| Profile | Format | Matches | Correct | Accuracy | Log-loss | Brier | Overrides | Override hits |".into());
            line("|---|---|--:|--:|--:|--:|--:|--:|--:|".into());
            for r in &t.rows {
                let s = &r.scores;
                line(format!(
                    "| {} | {} | {} | {} | {} | {} | {} | {} | {} |",
                    r.profile,
                    r.format,
                    s.matches,
                    s.correct,
                    pct(s.accuracy),
                    num(s.log_loss),
                    num(s.brier),
                    s.overrides,
                    if s.overrides == 0 {
                        "–".into()
                    } else {
                        format!("{} ({})", s.override_hits, pct(s.override_hit_rate))
                    }
                ));
            }
        }
        line(String::new());
        line("## Pick'em Points (PickemSolver)".into());
        line(String::new());
        line("Every profile submits the same Pick'em: the sidecar's Pick'em step is the Monte-Carlo solver over pre-Stage market and VRS ratings, with no LLM and no decision policy (ticket #28). Swiss: 10 picks, Playoffs: 7 slots.".into());
        line(String::new());
        line("| Stage | Kind | Status | Points | Decided picks | Rated pairs |".into());
        line("|---|---|---|--:|--:|--:|".into());
        for p in &self.pickem {
            line(format!(
                "| {} | {:?} | {} | {} | {}/{} | {} |",
                p.stage,
                p.kind,
                p.status,
                p.points.map_or("–".into(), |v| v.to_string()),
                p.settled_picks,
                p.picks,
                p.rated_pairs
            ));
        }
        line(String::new());
        line("## LLM track".into());
        line(String::new());
        line(format!(
            "Providers configured: {}.",
            if self.llm.providers.is_empty() {
                "none".into()
            } else {
                self.llm.providers.join(", ")
            }
        ));
        for (bot, planned) in &self.llm.planned {
            line(format!(
                "- `{bot}`: {} of {planned} planned Matches answered; provider calls by seat: {}.",
                self.llm.answered.get(bot).copied().unwrap_or(0),
                self.llm.calls.get(bot).copied().unwrap_or(0)
            ));
        }
        if let Some(stop) = &self.llm.stopped {
            line(format!("- Stopped: {stop}"));
        }
        for (f, n) in &self.llm.failures {
            line(format!("- {f}: {n} Matches without an answer"));
        }
        line(String::new());
        line("## Notes".into());
        line(String::new());
        for n in &self.notes {
            line(format!("- {n}"));
        }
        out
    }
}
