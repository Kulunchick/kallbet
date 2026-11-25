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
    pub async fn submissions(&self) -> Result<Vec<Prediction>> {
        let rows = sqlx::query_scalar::<_, String>(
            "SELECT payload FROM submitted_predictions ORDER BY run_id",
        )
        .fetch_all(&self.db)
        .await?;
        rows.iter().map(|s| Ok(serde_json::from_str(s)?)).collect()
    }
    pub async fn record_vrs(&self, markdown: &str, observed_at: Instant) -> Result<()> {
        sqlx::query("INSERT INTO vrs_snapshots(observed_at,content_hash,markdown) VALUES(?,?,?) ON CONFLICT DO NOTHING").bind(observed_at).bind(crate::digest(markdown.as_bytes())).bind(markdown).execute(&self.db).await?;
        Ok(())
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
        let encoded = serde_json::to_string(dossier)?;
        let hash = crate::digest(encoded.as_bytes());
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
