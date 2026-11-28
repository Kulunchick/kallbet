//! Optional `wreq` + `wreq-util` iOS-impersonating HTTP transport (feature `hltv-impersonate`).
//! Never compiled into the server crate or the default live sidecar build.
//! Disables automatic redirects so the collector counts every HTTP request against
//! the daily budget and enforces `/matches/*` on redirect targets.

use super::collector::{CollectorError, HttpResponse, ResearchHttp};
use super::demo::{DownloadResponse, ResearchDownload};
use futures_util::StreamExt;
use std::io::Write;
use async_trait::async_trait;
use std::time::Duration;
use wreq::{Client, header, redirect};
use wreq_util::{Emulation, Platform, Profile};

pub struct WreqIosHttp {
    client: Client,
}

impl WreqIosHttp {
    pub fn for_ios_major(major: u8) -> Result<Self, CollectorError> {
        let profile = if major <= 16 {
            Profile::SafariIos16_5
        } else {
            Profile::SafariIos17_2
        };
        let emulation = Emulation::builder()
            .profile(profile)
            .platform(Platform::IOS)
            .headers(false)
            .build();
        let client = Client::builder()
            .emulation(emulation)
            .redirect(redirect::Policy::none())
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| CollectorError::Transport(e.to_string()))?;
        Ok(Self { client })
    }
}

#[async_trait]
impl ResearchHttp for WreqIosHttp {
    async fn get(&self, url: &str, user_agent: &str) -> Result<HttpResponse, CollectorError> {
        let resp = self
            .client
            .get(url)
            .header(header::USER_AGENT, user_agent)
            .header(header::ACCEPT, "*/*")
            .header(header::ACCEPT_LANGUAGE, "en-US;q=1.0")
            .header(header::ACCEPT_ENCODING, "br;q=1.0, gzip;q=0.9, deflate;q=0.8")
            .send()
            .await
            .map_err(|e| CollectorError::Transport(e.to_string()))?;
        let status = resp.status().as_u16();
        let cf_mitigated = resp
            .headers()
            .get("cf-mitigated")
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned);
        let location = resp
            .headers()
            .get(header::LOCATION)
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned);
        let body = resp
            .text()
            .await
            .map_err(|e| CollectorError::Transport(e.to_string()))?;
        Ok(HttpResponse {
            status,
            cf_mitigated,
            location,
            body,
        })
    }
}

#[async_trait]
impl ResearchDownload for WreqIosHttp {
    async fn download(
        &self,
        url: &str,
        user_agent: &str,
        dest: &std::path::Path,
    ) -> Result<DownloadResponse, CollectorError> {
        let resp = self
            .client
            .get(url)
            .header(header::USER_AGENT, user_agent)
            .header(header::ACCEPT, "*/*")
            .header(header::ACCEPT_LANGUAGE, "en-US;q=1.0")
            // A demo archive is hundreds of MB: the 30 s page timeout would cut it off.
            .timeout(Duration::from_secs(1800))
            .send()
            .await
            .map_err(|e| CollectorError::Transport(e.to_string()))?;
        let status = resp.status().as_u16();
        let h = |name: &str| {
            resp.headers()
                .get(name)
                .and_then(|v| v.to_str().ok())
                .map(str::to_owned)
        };
        let cf_mitigated = h("cf-mitigated");
        let location = h("location");
        let content_type = h("content-type");
        let content_disposition = h("content-disposition");
        let is_html = content_type
            .as_deref()
            .is_some_and(|c| c.to_ascii_lowercase().starts_with("text/"));
        if status != 200 || is_html {
            // Small page (challenge, error, redirect body): keep the text for the stop-rule check.
            let body = resp
                .text()
                .await
                .map_err(|e| CollectorError::Transport(e.to_string()))?;
            return Ok(DownloadResponse {
                status,
                cf_mitigated,
                location,
                content_type,
                content_disposition,
                bytes_written: 0,
                body,
            });
        }
        let mut file = std::fs::File::create(dest)?;
        let mut written = 0u64;
        let mut stream = resp.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| CollectorError::Transport(e.to_string()))?;
            file.write_all(&chunk)?;
            written += chunk.len() as u64;
        }
        file.flush()?;
        Ok(DownloadResponse {
            status,
            cf_mitigated,
            location,
            content_type,
            content_disposition,
            bytes_written: written,
            body: String::new(),
        })
    }
}
