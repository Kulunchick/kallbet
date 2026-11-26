use crate::*;
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use std::path::Path;
pub struct Audit {
    pub(crate) db: SqlitePool,
}
impl Audit {
    pub async fn open(path: &Path) -> Result<Self> {
        let db = SqlitePool::connect_with(
            SqliteConnectOptions::new()
                .filename(path)
                .create_if_missing(true),
        )
        .await?;
        sqlx::migrate!("./migrations").run(&db).await?;
        Ok(Self { db })
    }
    /// Every Prediction the sidecar submitted, in order: decided ones and those
    /// submitted from the anchor alone while the LLM gave no answer.
    pub async fn submissions(&self) -> Result<Vec<Prediction>> {
        let rows = sqlx::query_scalar::<_, String>(
            "SELECT payload FROM (
                 SELECT run_id, 1 AS k, payload FROM submitted_predictions
                 UNION ALL SELECT run_id, 0 AS k, payload FROM unadjusted_submissions)
             ORDER BY run_id, k",
        )
        .fetch_all(&self.db)
        .await?;
        rows.iter().map(|s| Ok(serde_json::from_str(s)?)).collect()
    }
    /// Every raw record kept under `source` (e.g. `odds:oddspapi`), oldest first.
    pub async fn raw_records(&self, source: &str) -> Result<Vec<(Instant, Vec<u8>)>> {
        Ok(sqlx::query_as(
            "SELECT observed_at, payload FROM raw_feed_snapshots WHERE source=? ORDER BY id",
        )
        .bind(source)
        .fetch_all(&self.db)
        .await?)
    }
    /// Dated RSS articles both published and first fetched before the replay instant.
    pub async fn news_before(&self, cutoff: Instant) -> Result<Vec<AsOf<NewsItem>>> {
        let rows = sqlx::query_as::<_, (Instant, Vec<u8>)>(
            "SELECT observed_at, payload FROM raw_feed_snapshots
             WHERE source = 'rss:item' AND observed_at < ? ORDER BY observed_at, id",
        )
        .bind(cutoff)
        .fetch_all(&self.db)
        .await?;
        let mut items = Vec::new();
        for (seen, payload) in rows {
            let item: NewsItem = serde_json::from_slice(&payload)?;
            if item.published_at < cutoff {
                items.push(AsOf::new(item, seen, cutoff)?);
            }
        }
        Ok(items)
    }
    pub async fn record_vrs(&self, markdown: &str, observed_at: Instant) -> Result<()> {
        sqlx::query("INSERT INTO vrs_snapshots(observed_at,content_hash,markdown) VALUES(?,?,?) ON CONFLICT DO NOTHING").bind(observed_at).bind(crate::digest(markdown.as_bytes())).bind(markdown).execute(&self.db).await?;
        Ok(())
    }
}
impl Audit {
    /// The content address of a dossier: what `dossier_runs` and `debate_turns`
    /// key on. The market probability is rounded so that a move inside one
    /// rounding step does not count as a new input.
    pub fn input_hash(dossier: &Dossier) -> Result<String> {
        const MARKET_HASH_SCALE: f64 = 10_000.0;
        let mut hash_input = dossier.clone();
        hash_input.market = dossier
            .market
            .map(|p| (p * MARKET_HASH_SCALE).round() / MARKET_HASH_SCALE);
        Ok(crate::digest(&serde_json::to_vec(&hash_input)?))
    }
}
pub(crate) enum Decision {
    Pending,
    Submit(Prediction),
    Complete,
}

impl Audit {
    pub(crate) async fn begin(
        &self,
        bot: &str,
        league: i64,
        dossier: &Dossier,
        tier: Tier,
        cutoff: Instant,
    ) -> Result<(i64, Decision)> {
        let hash = Self::input_hash(dossier)?;
        let encoded = serde_json::to_string(dossier)?;
        let game = dossier.game.id;
        let mut tx = self.db.begin().await?;
        sqlx::query(
            "INSERT INTO dossier_runs(bot,league,match_id,tier,input_hash,cutoff,dossier)
            VALUES(?,?,?,?,?,?,?) ON CONFLICT DO NOTHING",
        )
        .bind(bot)
        .bind(league)
        .bind(game)
        .bind(tier.name())
        .bind(&hash)
        .bind(cutoff)
        .bind(encoded)
        .execute(&mut *tx)
        .await?;
        let (id, decided, payload, submitted): (i64, bool, Option<String>, bool) = sqlx::query_as(
            "SELECT d.id, p.run_id IS NOT NULL, p.prediction, s.run_id IS NOT NULL
             FROM dossier_runs d LEFT JOIN decisions p ON p.run_id=d.id
             LEFT JOIN submitted_predictions s ON s.run_id=d.id
             WHERE bot=? AND league=? AND match_id=? AND tier=? AND input_hash=?",
        )
        .bind(bot)
        .bind(league)
        .bind(game)
        .bind(tier.name())
        .bind(hash)
        .fetch_one(&mut *tx)
        .await?;
        tx.commit().await?;
        let state = match (decided, payload, submitted) {
            (false, _, _) => Decision::Pending,
            (true, Some(payload), false) => Decision::Submit(serde_json::from_str(&payload)?),
            _ => Decision::Complete,
        };
        Ok((id, state))
    }
    pub(crate) async fn decide(&self, id: i64, pick: Option<&Prediction>) -> Result<()> {
        sqlx::query("INSERT INTO decisions(run_id,prediction) VALUES(?,?)")
            .bind(id)
            .bind(pick.map(serde_json::to_string).transpose()?)
            .execute(&self.db)
            .await?;
        Ok(())
    }
    pub(crate) async fn submitted(&self, id: i64, now: Instant, pick: &Prediction) -> Result<()> {
        sqlx::query("INSERT INTO submitted_predictions(run_id,submitted_at,payload) VALUES(?,?,?)")
            .bind(id)
            .bind(now)
            .bind(serde_json::to_string(pick)?)
            .execute(&self.db)
            .await?;
        Ok(())
    }
}

/// One step of a tick that failed; the tick went on with the next one.
#[derive(Debug, Clone, PartialEq)]
pub struct Failure {
    pub at: Instant,
    pub bot: String,
    /// `login`, `join`, `leagues`, `matches`, `pickem`, `predict`.
    pub step: String,
    pub league: Option<i64>,
    pub match_id: Option<i64>,
    pub stage: Option<u8>,
    pub error: String,
}

impl Audit {
    pub(crate) async fn record_failure(&self, failure: &Failure) -> Result<()> {
        sqlx::query(
            "INSERT INTO tick_failures(at,bot,step,league,match_id,stage,error) VALUES(?,?,?,?,?,?,?)",
        )
        .bind(failure.at)
        .bind(&failure.bot)
        .bind(&failure.step)
        .bind(failure.league)
        .bind(failure.match_id)
        .bind(failure.stage.map(i64::from))
        .bind(&failure.error)
        .execute(&self.db)
        .await?;
        Ok(())
    }
    /// Every recorded failure, oldest first.
    pub async fn failures(&self) -> Result<Vec<Failure>> {
        type Row = (Instant, String, String, Option<i64>, Option<i64>, Option<i64>, String);
        let rows: Vec<Row> = sqlx::query_as(
            "SELECT at,bot,step,league,match_id,stage,error FROM tick_failures ORDER BY id",
        )
        .fetch_all(&self.db)
        .await?;
        Ok(rows
            .into_iter()
            .map(|(at, bot, step, league, match_id, stage, error)| Failure {
                at,
                bot,
                step,
                league,
                match_id,
                stage: stage.map(|s| s as u8),
                error,
            })
            .collect())
    }
    /// Whether this run already submitted a Prediction from the anchor alone.
    pub(crate) async fn unadjusted(&self, id: i64) -> Result<bool> {
        Ok(
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM unadjusted_submissions WHERE run_id=?")
                .bind(id)
                .fetch_one(&self.db)
                .await?
                > 0,
        )
    }
    pub(crate) async fn submitted_unadjusted(
        &self,
        id: i64,
        now: Instant,
        error: &str,
        pick: &Prediction,
    ) -> Result<()> {
        sqlx::query(
            "INSERT INTO unadjusted_submissions(run_id,submitted_at,llm_error,payload) VALUES(?,?,?,?)",
        )
        .bind(id)
        .bind(now)
        .bind(error)
        .bind(serde_json::to_string(pick)?)
        .execute(&self.db)
        .await?;
        Ok(())
    }
}

/// Keeps a raw external record once per `source` and content: an unchanged
/// record seen again is not stored again (`observed_at` is when it was first
/// seen). Returns its content hash. The rule for every adapter that keeps raw
/// data in `raw_feed_snapshots`.
pub(crate) async fn store_raw(
    db: &sqlx::SqlitePool,
    source: &str,
    observed_at: Instant,
    payload: &[u8],
) -> Result<String> {
    let hash = crate::digest(payload);
    sqlx::query(
        "INSERT INTO raw_feed_snapshots(source, observed_at, content_hash, payload)
         SELECT ?1, ?2, ?3, ?4
         WHERE NOT EXISTS (SELECT 1 FROM raw_feed_snapshots WHERE source = ?1 AND content_hash = ?3)",
    )
    .bind(source)
    .bind(observed_at)
    .bind(&hash)
    .bind(payload)
    .execute(db)
    .await?;
    Ok(hash)
}
/// A raw record by its content hash.
pub(crate) async fn load_raw(
    db: &sqlx::SqlitePool,
    source: &str,
    hash: &str,
) -> Result<Option<Vec<u8>>> {
    Ok(sqlx::query_scalar(
        "SELECT payload FROM raw_feed_snapshots WHERE source = ? AND content_hash = ? LIMIT 1",
    )
    .bind(source)
    .bind(hash)
    .fetch_optional(db)
    .await?)
}
