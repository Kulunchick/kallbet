//! Team logos, downloaded once and served from our side (never hotlinked):
//! `<dir>/<team id>.<png|svg>`, plus `<team id>.source` naming the logo it
//! holds, so a team's new logo replaces the old one. Served by
//! `GET /api/teams/{id}/logo` (`crate::api::teams`).

use std::path::{Path, PathBuf};

use crate::domain::{Team, TeamId};

/// HLTV's image CDN refuses clients without a user agent it accepts.
const USER_AGENT: &str = "KallBet/0.1 (https://kallbet.kulunchick.tech)";

#[derive(Clone)]
pub struct LogoCache {
    dir: PathBuf,
    client: reqwest::Client,
}

#[derive(Debug, thiserror::Error)]
pub enum LogoError {
    #[error("download failed: {0}")]
    Download(String),
    #[error("not an image: {0}")]
    NotAnImage(String),
    #[error("writing the logo: {0}")]
    Io(#[from] std::io::Error),
}

impl LogoCache {
    pub fn new(dir: impl Into<PathBuf>) -> LogoCache {
        super::install_tls_provider();
        let client = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .timeout(std::time::Duration::from_secs(20))
            .build()
            .expect("HTTP client");
        LogoCache { dir: dir.into(), client }
    }

    /// Makes sure the team's current logo is on disk. Downloads only when it
    /// is not there yet or the team's logo changed. Teams without a logo
    /// (HLTV's letter placeholder) are skipped.
    pub async fn ensure(&self, team: &Team) -> Result<(), LogoError> {
        let Some(url) = team.logo_url.as_deref() else {
            return Ok(());
        };
        if url.contains("/dynamic-svg/") {
            return Ok(());
        }
        let source = url.split('?').next().unwrap_or(url);
        let marker = self.dir.join(format!("{}.source", team.id));
        if tokio::fs::read_to_string(&marker).await.ok().as_deref() == Some(source)
            && stored(&self.dir, team.id).await.is_some()
        {
            return Ok(());
        }

        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| LogoError::Download(e.to_string()))?;
        if !response.status().is_success() {
            return Err(LogoError::Download(format!("{url}: HTTP {}", response.status())));
        }
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default()
            .to_string();
        let ext = match content_type.split(';').next().unwrap_or_default().trim() {
            "image/svg+xml" => "svg",
            "image/png" => "png",
            other => return Err(LogoError::NotAnImage(format!("{url}: {other:?}"))),
        };
        let bytes = response
            .bytes()
            .await
            .map_err(|e| LogoError::Download(e.to_string()))?;

        tokio::fs::create_dir_all(&self.dir).await?;
        let tmp = self.dir.join(format!("{}.{ext}.tmp", team.id));
        tokio::fs::write(&tmp, &bytes).await?;
        for old in ["svg", "png"] {
            let _ = tokio::fs::remove_file(self.dir.join(format!("{}.{old}", team.id))).await;
        }
        tokio::fs::rename(&tmp, self.dir.join(format!("{}.{ext}", team.id))).await?;
        tokio::fs::write(&marker, source).await?;
        Ok(())
    }
}

/// The stored logo of a team: its bytes and content type.
pub async fn stored(dir: &Path, team: TeamId) -> Option<(Vec<u8>, &'static str)> {
    for (ext, content_type) in [("svg", "image/svg+xml"), ("png", "image/png")] {
        if let Ok(bytes) = tokio::fs::read(dir.join(format!("{team}.{ext}"))).await {
            return Some((bytes, content_type));
        }
    }
    None
}
