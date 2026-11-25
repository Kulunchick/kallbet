use super::swiss::series_win_prob;
use crate::*;
use chrono::Duration;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SubmittedPickem {
    pub bot: String,
    pub league: i64,
    pub stage: u8,
    pub submitted_at: Instant,
    pub input_hash: String,
    pub payload: Value,
}

pub async fn submitted_pickems(audit: &Audit) -> Result<Vec<SubmittedPickem>> {
    let rows: Vec<(String, i64, i64, Instant, String, String)> = sqlx::query_as(
        "SELECT bot, league, stage, submitted_at, input_hash, payload
         FROM submitted_pickems
         ORDER BY id",
    )
    .fetch_all(&audit.db)
    .await?;
    rows.into_iter()
        .map(|(bot, league, stage, submitted_at, input_hash, payload)| {
            Ok(SubmittedPickem {
                bot,
                league,
                stage: stage as u8,
                submitted_at,
                input_hash,
                payload: serde_json::from_str(&payload)?,
            })
        })
        .collect()
}

pub(crate) fn invert_series_prob(format: Format, series_prob: f64) -> f64 {
    let target = series_prob.clamp(0.0, 1.0);
    match format {
        Format::Bo1 => target,
        Format::Bo3 | Format::Bo5 => {
            if target <= 0.0 {
                return 0.0;
            }
            if target >= 1.0 {
                return 1.0;
            }
            let mut lo = 0.0_f64;
            let mut hi = 1.0_f64;
            for _ in 0..60 {
                let mid = 0.5 * (lo + hi);
                if series_win_prob(format, mid) < target {
                    lo = mid;
                } else {
                    hi = mid;
                }
            }
            0.5 * (lo + hi)
        }
    }
}

fn canonical_stage_fingerprint(stage_val: &Value) -> Value {
    let mut teams: Vec<(i64, String)> = stage_val["teams"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|t| Some((t["id"].as_i64()?, t["name"].as_str()?.to_string())))
                .collect()
        })
        .unwrap_or_default();
    teams.sort_by_key(|(id, _)| *id);

    let bracket_slots: Vec<Value> = stage_val["bracket"]["slots"]
        .as_array()
        .map(|arr| {
            let mut slots: Vec<Value> = arr
                .iter()
                .map(|s| {
                    json!({
                        "r": s["rounds_before_final"],
                        "s": s["slot"],
                        "leaf": s["leaf"],
                        "a": s["team_a"],
                        "b": s["team_b"],
                        "f": s["format"],
                    })
                })
                .collect();
            slots.sort_by_cached_key(Value::to_string);
            slots
        })
        .unwrap_or_default();

    let mut r1_pairs: Vec<Value> = Vec::new();
    if let Some(groups) = stage_val["groups"].as_array() {
        for g in groups {
            let wins = g["wins"].as_u64().unwrap_or(0);
            let losses = g["losses"].as_u64().unwrap_or(0);
            let rbf = &g["rounds_before_final"];
            if let Some(matches) = g["matches"].as_array() {
                for m in matches {
                    if g["kind"].as_str() == Some("record") && wins == 0 && losses == 0 {
                        r1_pairs.push(json!({
                            "a": m["team_a"]["id"],
                            "b": m["team_b"]["id"],
                            "f": m["format"],
                        }));
                    } else if g["kind"].as_str() == Some("playoff_round") {
                        r1_pairs.push(json!({
                            "r": rbf,
                            "s": m["bracket_slot"],
                            "f": m["format"],
                        }));
                    }
                }
            }
        }
    }
    r1_pairs.sort_by_cached_key(Value::to_string);

    json!({
        "stage": {
            "ordinal": stage_val["stage"]["ordinal"],
            "kind": stage_val["stage"]["kind"],
            "name": stage_val["stage"]["name"],
        },
        "all_bo3": stage_val["all_bo3"],
        "teams": teams,
        "bracket_ready": stage_val["bracket"]["ready"],
        "bracket_slots": bracket_slots,
        "pairings": r1_pairs,
    })
}

fn compute_input_hash(
    stage_val: &Value,
    ratings: &[AsOf<PairwiseProbability>],
    cutoff: Instant,
) -> Result<String> {
    let mut quantised: Vec<(i64, i64, i64)> = Vec::with_capacity(ratings.len());
    for rating in ratings {
        let r = rating.at(cutoff)?;
        let (a, b, p) = if r.first <= r.second {
            (r.first, r.second, r.map)
        } else {
            (r.second, r.first, 1.0 - r.map)
        };
        quantised.push((a, b, (p.clamp(0.0, 1.0) * 1_000_000.0).round() as i64));
    }
    quantised.sort_unstable();
    let doc = json!({
        "stage": canonical_stage_fingerprint(stage_val),
        "ratings": quantised,
    });
    let bytes = serde_json::to_vec(&doc)?;
    Ok(crate::digest(&bytes))
}

pub struct PickemRunner {
    clock: Arc<dyn Clock>,
    signal: Arc<dyn SignalEstimator>,
    odds: Arc<dyn OddsSource>,
    solver: Arc<dyn PickemSolver>,
}

impl PickemRunner {
    pub fn new(
        clock: Arc<dyn Clock>,
        signal: Arc<dyn SignalEstimator>,
        odds: Arc<dyn OddsSource>,
        solver: Arc<dyn PickemSolver>,
    ) -> Self {
        Self {
            clock,
            signal,
            odds,
            solver,
        }
    }

    async fn gather_ratings(
        &self,
        sp: &StagePickem,
        now: Instant,
        stage_cutoff: Instant,
    ) -> Result<Vec<AsOf<PairwiseProbability>>> {
        let mut opening_by_pair: HashMap<(i64, i64), &Match> = HashMap::new();
        for m in &sp.opening_matches {
            let key = if m.team_a.id <= m.team_b.id {
                (m.team_a.id, m.team_b.id)
            } else {
                (m.team_b.id, m.team_a.id)
            };
            opening_by_pair.insert(key, m);
        }

        let mut teams = sp.teams.clone();
        teams.sort_by_key(|t| t.id);

        let mut ratings = Vec::new();
        for i in 0..teams.len() {
            for j in (i + 1)..teams.len() {
                let ta = &teams[i];
                let tb = &teams[j];
                let key = (ta.id, tb.id);

                let synth = Match {
                    event_id: sp.event_id,
                    id: 0,
                    team_a: ta.clone(),
                    team_b: tb.clone(),
                    format: Format::Bo1,
                    scheduled_start: sp.locks_at,
                };

                let opening = opening_by_pair.get(&key).copied();
                let game = opening.unwrap_or(&synth);
                let market = if opening.is_none() {
                    None
                } else if game
                    .scheduled_start
                    .is_some_and(|start| now >= start - Duration::minutes(15))
                {
                    // Match odds collection ends at Match Hard Lock. Other Event
                    // batches can still supply a dated line before Stage Hard Lock.
                    self.odds.quote(game, now).await?.map(|q| (now, q))
                } else {
                    let (as_of, quote) =
                        self.odds
                            .live_quote(game, now, Tier::Draft, self.clock.as_ref())
                            .await?;
                    quote.map(|q| (as_of, q))
                };
                let (map, observed_at) = if let Some((as_of, q)) = market {
                    (
                        invert_series_prob(game.format, *q.at(as_of)?),
                        q.observed_at(),
                    )
                } else if let Some(sig) = self.signal.signal(game, now).await? {
                    (sig.at(now)?.map, sig.observed_at())
                } else {
                    continue;
                };
                ratings.push(AsOf::new(
                    PairwiseProbability {
                        first: ta.id,
                        second: tb.id,
                        map: if game.team_a.id == ta.id {
                            map
                        } else {
                            1.0 - map
                        },
                    },
                    observed_at,
                    stage_cutoff,
                )?);
            }
        }

        Ok(ratings)
    }
}

impl PickemRunner {
    /// One Stage's Pick'em: submit it if missing, refresh it once in the last
    /// 30 minutes if its inputs changed.
    async fn stage(
        &self,
        platform: &dyn LeaguePlatform,
        token: &str,
        bot: &BotProfile,
        league: i64,
        now: Instant,
        audit: &Audit,
        sp: StagePickem,
    ) -> Result<()> {
        if !sp.can_edit || sp.locked || sp.locks_at.is_some_and(|l| now >= l) {
            return Ok(());
        }

        let kind = sp.stage["stage"]["kind"].as_str().unwrap_or("");
        match kind {
            "swiss" if sp.teams.len() == 16 => {}
            "playoffs" if sp.stage["bracket"]["ready"].as_bool() == Some(true) => {}
            _ => return Ok(()),
        }

        let previous = if let Some(last_write) = sp.my_updated_at {
            let Some(start) = sp.locks_at else { return Ok(()) };
            let refresh_open = start - Duration::minutes(30);
            if now < refresh_open {
                return Ok(());
            }
            let hashes: Vec<String> = sqlx::query_scalar(
                "SELECT input_hash FROM submitted_pickems
                 WHERE bot = ? AND league = ? AND stage = ? ORDER BY id DESC LIMIT 2",
            )
            .bind(&bot.bot_key)
            .bind(league)
            .bind(i64::from(sp.ordinal))
            .fetch_all(&audit.db)
            .await?;
            // Two successful writes are the draft and its refresh. If the audit was
            // lost, use the server timestamp to avoid repeating an in-window write.
            if hashes.len() == 2 || (hashes.is_empty() && last_write >= refresh_open) {
                return Ok(());
            }
            hashes.into_iter().next()
        } else {
            None
        };

        let stage_cutoff = sp
            .locks_at
            .unwrap_or_else(|| now + Duration::nanoseconds(1));
        let ratings = self.gather_ratings(&sp, now, stage_cutoff).await?;

        let input_hash = compute_input_hash(&sp.stage, &ratings, stage_cutoff)?;
        if previous.as_deref() == Some(input_hash.as_str()) {
            return Ok(());
        }

        let stage_as_of = AsOf::new(sp.stage, now, stage_cutoff)?;
        let Some(payload) = self.solver.solve(&stage_as_of, &ratings, stage_cutoff)? else {
            return Ok(());
        };

        if platform
            .submit_pickem(token, league, sp.ordinal, &payload)
            .await?
        {
            sqlx::query(
                "INSERT INTO submitted_pickems(bot, league, stage, submitted_at, input_hash, payload)
                 VALUES(?, ?, ?, ?, ?, ?)",
            )
            .bind(&bot.bot_key)
            .bind(league)
            .bind(i64::from(sp.ordinal))
            .bind(now)
            .bind(&input_hash)
            .bind(serde_json::to_string(&payload)?)
            .execute(&audit.db)
            .await?;
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl PickemStep for PickemRunner {
    async fn tick(
        &self,
        platform: &dyn LeaguePlatform,
        token: &str,
        bot: &BotProfile,
        league: i64,
        now: Instant,
        audit: &Audit,
    ) -> Result<Vec<StageFailure>> {
        let mut failures = Vec::new();
        for sp in platform.pickem_stages(token, league).await? {
            let stage = sp.ordinal;
            match self.stage(platform, token, bot, league, now, audit, sp).await {
                Ok(()) => {}
                Err(e) if e.is::<TemporalLeakError>() => return Err(e),
                Err(error) => failures.push(StageFailure { stage, error }),
            }
        }
        Ok(failures)
    }
}
