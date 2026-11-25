use super::*;
use crate::adapters::{
    baseline::VrsOnly,
    context::liquipedia,
    odds::{MarketOnly, oddspapi},
    pickem::MonteCarloPickemSolver,
};
use serde::Deserialize;
use serde_json::json;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

#[derive(Deserialize)]
struct Row {
    game: Match,
    actual: Instant,
    winner: i64,
}
#[derive(Deserialize)]
struct Participant {
    id: i64,
    rating_name: String,
}
#[derive(Deserialize)]
struct View {
    observed: Instant,
    payload: Value,
}
#[derive(Deserialize)]
struct Stage {
    id: i64,
    name: String,
    cutoff: Instant,
    complete: bool,
    teams: Vec<Participant>,
    matches: Vec<Row>,
    view: Option<View>,
}

#[derive(Default)]
struct Metrics {
    matches: usize,
    correct: usize,
    log_loss: f64,
    brier: f64,
    overrides: usize,
    correct_overrides: usize,
}
impl Metrics {
    fn score(&mut self, pick: &Prediction, winner: i64) -> Result<()> {
        let p = pick.rationale["p_win"]
            .as_f64()
            .filter(|p| (0.0..=1.0).contains(p))
            .ok_or("invalid prediction probability")?;
        let hit = pick.team_id == winner;
        self.matches += 1;
        self.correct += usize::from(hit);
        let truth = if hit { p } else { 1.0 - p };
        self.log_loss -= truth.max(1e-15).ln();
        self.brier += (1.0 - truth).powi(2);
        if pick.rationale["override_market"] == true {
            self.overrides += 1;
            self.correct_overrides += usize::from(hit);
        }
        Ok(())
    }
    fn report(&self) -> Value {
        let n = self.matches as f64;
        json!({"matches":self.matches,"correct":self.correct,"accuracy":self.correct as f64/n,
            "log_loss":self.log_loss/n,"brier":self.brier/n,"overrides":self.overrides,
            "override_hit_rate":if self.overrides==0 {None} else {Some(self.correct_overrides as f64/self.overrides as f64)}})
    }
}

async fn pickem(stage: &Stage, vrs: &vrs::Standings, panel: &super::panel::Panel) -> Result<Value> {
    let Some(view) = &stage.view else {
        return Ok(Value::Null);
    };
    let snapshot = AsOf::new(&view.payload, view.observed, stage.cutoff)?;
    let teams = snapshot.at(stage.cutoff)?["teams"]
        .as_array()
        .ok_or("stage teams missing")?;
    let mut ratings = Vec::new();
    let names: BTreeMap<_, _> = stage.teams.iter().map(|t| (t.id, &t.rating_name)).collect();
    for (i, a) in teams.iter().enumerate() {
        for b in &teams[..i] {
            let first = a["id"].as_i64().ok_or("invalid team")?;
            let second = b["id"].as_i64().ok_or("invalid team")?;
            if let Some(s) = vrs.estimate(
                names.get(&first).ok_or("unknown team")?,
                names.get(&second).ok_or("unknown team")?,
                Format::Bo1,
                stage.cutoff,
            )? {
                ratings.push(AsOf::new(
                    PairwiseProbability {
                        first,
                        second,
                        map: s.at(stage.cutoff)?.map,
                    },
                    s.observed_at(),
                    stage.cutoff,
                )?);
            }
        }
    }
    let frozen = AsOf::new(view.payload.clone(), view.observed, stage.cutoff)?;
    let Some(picks) = MonteCarloPickemSolver::default().solve(&frozen, &ratings, stage.cutoff)?
    else {
        return Ok(Value::Null);
    };
    let points = stage
        .complete
        .then(|| score_pickem(stage, &picks))
        .transpose()?;
    let mut output = json!({"picks":picks,"points":points,"rated_pairs":ratings.len()});
    if stage.id == 22378 {
        let frozen_ratings = ratings
            .iter()
            .map(|r| r.at(stage.cutoff))
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let opening = json!({"teams":teams,"seeds":view.payload["seeds"],"groups":view.payload["groups"],
            "pairwise_map_probabilities":frozen_ratings,"solver_picks":picks});
        for (label, p) in panel.swiss(&opening).await? {
            output[label] = json!({"picks":p,"points":stage.complete.then(||score_pickem(stage,&p)).transpose()?});
        }
    }
    Ok(output)
}
fn score_pickem(stage: &Stage, picks: &Value) -> Result<usize> {
    let mut wins = BTreeMap::<i64, usize>::new();
    let mut losses = BTreeMap::<i64, usize>::new();
    for m in &stage.matches {
        *wins.entry(m.winner).or_default() += 1;
        let loser = if m.winner == m.game.team_a.id {
            m.game.team_b.id
        } else {
            m.game.team_a.id
        };
        *losses.entry(loser).or_default() += 1;
    }
    if let Some(slots) = picks["picks"].as_array() {
        // In single elimination a team wins a slot in round r exactly when
        // it wins enough rounds from the leaves to reach the next round.
        let depth = slots
            .iter()
            .filter_map(|s| s["rounds_before_final"].as_u64())
            .max()
            .ok_or("empty bracket")?;
        return Ok(slots
            .iter()
            .filter(|s| {
                let id = s["team_id"].as_i64().unwrap_or_default();
                wins.get(&id).copied().unwrap_or(0) as u64
                    > depth - s["rounds_before_final"].as_u64().unwrap_or_default()
            })
            .count());
    }
    let mut points = 0;
    for group in ["three_nil", "zero_three", "advance"] {
        for id in picks[group].as_array().ok_or("missing pick bucket")? {
            let id = id.as_i64().ok_or("invalid pick")?;
            let record = (
                wins.get(&id).copied().unwrap_or(0),
                losses.get(&id).copied().unwrap_or(0),
            );
            points += usize::from(matches!(
                (group, record),
                ("three_nil", (3, 0)) | ("zero_three", (0, 3)) | ("advance", (3, 1 | 2))
            ));
        }
    }
    Ok(points)
}

pub async fn run(dir: &Path, live: bool, blind_singapore: bool) -> Result<Value> {
    let started_at = chrono::Utc::now();
    let data = fs::read(dir.join("stages.json")).map_err(|e| format!("stages.json: {e}"))?;
    let mut stages: Vec<Stage> = serde_json::from_slice(&data)?;
    stages.sort_by_key(|s| s.cutoff);
    let bot: BotProfile = serde_json::from_value(
        json!({"bot_key":"replay","nickname":"Replay","invite_links":[],"policy":"HybridAnchored"}),
    )?;
    let mut standings = vrs::Standings::default();
    let mut indexes = vec![];
    let mut history = BTreeMap::new();
    for path in fs::read_dir(dir)? {
        let path = path?.path();
        let name = path.file_name().and_then(|p| p.to_str()).unwrap_or("");
        if name.starts_with("standings_global_") {
            let doc = fs::read_to_string(&path)?;
            let (head, rows) = doc.split_once('\n').ok_or("invalid standings")?;
            standings.add(&format!("{}\n{rows}", head.replace('-', "_")))?;
        } else if name.ends_with("settled.json") || name == "cologne-fixtures.json" {
            indexes.push(fs::read(&path)?);
        } else if name.starts_with("od1")
            || name.starts_with("id1")
            || name.starts_with("oddspapi-historical-")
        {
            let bytes = fs::read(&path)?;
            let tick: Value = serde_json::from_slice(&bytes)?;
            if let Some(id) = tick["fixtureId"].as_str() {
                history.insert(id.to_owned(), bytes);
            }
        }
    }

    let audit = Audit::open(&dir.join("backtest.sqlite")).await?;
    let llm = super::panel::Panel::load(dir, live, &audit).await?;
    let mut metrics = BTreeMap::<(String, String), Metrics>::new();
    let mut paired = BTreeMap::<String, Metrics>::new();
    let mut reports = vec![];
    let mut seen = BTreeSet::new();
    for stage in &mut stages {
        if let Some(first) = stage.matches.iter().map(|row| row.actual).min() {
            if stage.cutoff > first {
                return Err(TemporalLeakError {
                    observed_at: stage.cutoff,
                    cutoff: first,
                }
                .into());
            }
        }
        let mut stage_prediction = None;
        for m in &stage.matches {
            match_cutoff(&m.game, m.actual)?;
        }
        stage.matches.sort_by_key(|m| {
            (
                match_cutoff(&m.game, m.actual).expect("validated replay cutoff"),
                m.game.id,
            )
        });
        let mut counts = BTreeMap::<String, usize>::new();
        for row in &stage.matches {
            let g = &row.game;
            if !seen.insert(g.id) || ![g.team_a.id, g.team_b.id].contains(&row.winner) {
                return Err("invalid completed match".into());
            }
            let cutoff = match_cutoff(g, row.actual)?;
            if stage_prediction.is_none() && stage.cutoff <= cutoff {
                stage_prediction = Some(pickem(stage, &standings, &llm).await?);
            }
            let name = |id| {
                stage
                    .teams
                    .iter()
                    .find(|t| t.id == id)
                    .map(|t| t.rating_name.as_str())
                    .ok_or("unknown team")
            };
            let signal =
                standings.estimate(name(g.team_a.id)?, name(g.team_b.id)?, g.format, cutoff)?;
            let mut market = None;
            for index in &indexes {
                if let Some(f) = oddspapi::match_settled_fixture(index, g)?
                    && let Some(body) = history.get(f.id())
                {
                    market = f.quote(body, cutoff)?;
                    break;
                }
            }
            let mut facts = vec![];
            let roster_path = dir.join(format!("roster-{}.json", g.id));
            if roster_path.exists() {
                let body = fs::read_to_string(roster_path)?;
                let raw: Value = serde_json::from_str(&body)?;
                let observed: Instant = serde_json::from_value(
                    raw["query"]["pages"][0]["revisions"][0]["timestamp"].clone(),
                )?;
                AsOf::new((), observed, cutoff)?;
                let rev = liquipedia::parse_revision_rosters(&body, cutoff)?
                    .ok_or("roster revision unavailable")?;
                let event = rev.at(cutoff)?;
                for (side, t) in [("team_a", &g.team_a), ("team_b", &g.team_b)] {
                    if let Some(r) = liquipedia::find_team_roster(event, &t.name) {
                        let fact = ContextFact::LiquipediaRoster(liquipedia::build_roster_fact(
                            event, r, side,
                        ));
                        facts.push(AsOf::new(fact.to_value()?, rev.observed_at(), cutoff)?);
                    }
                }
            }
            let dossier = replay_match(g.clone(), row.actual, signal, market, facts)?;
            let mut predictions = vec![
                (
                    "VrsOnly",
                    VrsOnly.decide(&bot, &dossier, Tier::PreLock, None)?,
                ),
                (
                    "MarketOnly",
                    if dossier.market.is_some() {
                        MarketOnly.decide(&bot, &dossier, Tier::PreLock, None)?
                    } else {
                        None
                    },
                ),
            ];
            if stage.id == 22378
                || (stage.id == 15477 && stage.matches.iter().take(2).any(|m| m.game.id == g.id))
            {
                predictions.extend(
                    llm.predict(&bot, &dossier, stage.id != 22378 || blind_singapore, cutoff)
                        .await?,
                );
            }
            let format = serde_json::to_value(g.format)?.as_str().unwrap().to_owned();
            for (model, pick) in predictions {
                if let Some(pick) = pick {
                    if dossier.market.is_some()
                        && dossier.signal.is_some()
                        && matches!(model, "VrsOnly" | "MarketOnly")
                    {
                        paired
                            .entry(model.into())
                            .or_default()
                            .score(&pick, row.winner)?;
                    }
                    *counts.entry(model.into()).or_default() += 1;
                    for bucket in [&format, "all"] {
                        metrics
                            .entry((model.into(), bucket.into()))
                            .or_default()
                            .score(&pick, row.winner)?;
                    }
                }
            }
        }
        let picks = match stage_prediction {
            Some(p) => p,
            None => pickem(stage, &standings, &llm).await?,
        };
        reports.push(json!({"stage":stage.name,"completed_matches":stage.matches.len(),"coverage":counts,"pickem":picks}));
    }
    let scores: Vec<_> = metrics
        .iter()
        .map(|((model, format), m)| json!({"model":model,"format":format,"metrics":m.report()}))
        .collect();
    let paired: BTreeMap<_, _> = paired.iter().map(|(name, m)| (name, m.report())).collect();
    let report = json!({"paired_baselines":paired,"scores":scores,"stages":reports,"blind_singapore":blind_singapore,"llm_status":llm.status().await});
    sqlx::query("INSERT INTO backtest_runs(started_at,profile,report) VALUES(?,?,?)")
        .bind(started_at.to_rfc3339())
        .bind("retrospective")
        .bind(report.to_string())
        .execute(&audit.db)
        .await?;
    Ok(report)
}
