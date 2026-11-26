//! Optional `wreq` + `wreq-util` iOS-impersonating HTTP transport (feature `hltv-impersonate`).
//! Never compiled into the server crate or the default live sidecar build.

use super::collector::{CollectorError, HttpResponse, ResearchHttp};
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
            .redirect(redirect::Policy::limited(5))
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
        let body = resp
            .text()
            .await
            .map_err(|e| CollectorError::Transport(e.to_string()))?;
        Ok(HttpResponse {
            status,
            cf_mitigated,
            body,
        })
    }
}
