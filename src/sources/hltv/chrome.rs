//! The fetch layer: loads HLTV pages in a headless Chrome over CDP
//! (chromiumoxide), which gets past Cloudflare where a plain HTTP client
//! cannot.
//!
//! - Connects to an already running browser (the `chromedp/headless-shell`
//!   sidecar, `KALLBET_CHROME_URL`), lazily, and reconnects after a failure.
//! - Chrome's DevTools HTTP endpoint answers 500 to a `Host` header that is
//!   neither an IP nor `localhost`, so a hostname (`http://chrome:9222` in
//!   compose) is resolved to an IP before connecting.
//! - One page at a time, at least [`MIN_GAP`] apart: polling stays sparse.
//! - Waits for Cloudflare's "Just a moment..." interstitial to clear; a
//!   challenge that does not clear is `SourceError::Unavailable`.
//! - Never closes the browser itself (it is shared): only its own tabs.

use std::net::IpAddr;
use std::time::Duration;

use async_trait::async_trait;
use chromiumoxide::browser::Browser;
use futures::StreamExt;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time::{Instant, sleep, timeout};

use super::PageLoader;
use crate::ports::SourceError;

/// Minimum time between two page loads.
pub const MIN_GAP: Duration = Duration::from_secs(5);
/// How long a Cloudflare challenge may take to clear.
const CHALLENGE_WAIT: Duration = Duration::from_secs(45);
/// Upper bound for one whole page load.
const LOAD_TIMEOUT: Duration = Duration::from_secs(90);

pub struct ChromePages {
    endpoint: String,
    state: Mutex<State>,
}

#[derive(Default)]
struct State {
    browser: Option<Connected>,
    last_load: Option<Instant>,
}

struct Connected {
    browser: Browser,
    handler: JoinHandle<()>,
    /// Set when the browser announces itself as `HeadlessChrome`, which
    /// Cloudflare never lets through.
    user_agent: Option<String>,
}

impl Drop for Connected {
    fn drop(&mut self) {
        self.handler.abort();
    }
}

impl ChromePages {
    /// `endpoint`: `http://host:port` of the DevTools endpoint, or a
    /// `ws://.../devtools/browser/...` URL.
    pub fn new(endpoint: impl Into<String>) -> ChromePages {
        super::install_tls_provider();
        ChromePages {
            endpoint: endpoint.into(),
            state: Mutex::new(State::default()),
        }
    }

    async fn connect(&self) -> Result<Connected, SourceError> {
        let url = resolve_endpoint(&self.endpoint).await?;
        let (browser, mut handler) = timeout(Duration::from_secs(20), Browser::connect(url.clone()))
            .await
            .map_err(|_| unavailable(format!("connecting to {url} timed out")))?
            .map_err(|e| unavailable(format!("connecting to {url}: {e}")))?;
        let handler = tokio::spawn(async move {
            while let Some(event) = handler.next().await {
                if event.is_err() {
                    break;
                }
            }
        });
        let mut connected = Connected { browser, handler, user_agent: None };
        let agent = connected.browser.user_agent().await.map_err(cdp)?;
        if agent.contains("HeadlessChrome") {
            connected.user_agent = Some(agent.replace("HeadlessChrome", "Chrome"));
        }
        Ok(connected)
    }
}

#[async_trait]
impl PageLoader for ChromePages {
    async fn load(&self, url: &str) -> Result<String, SourceError> {
        let mut state = self.state.lock().await;
        if let Some(last) = state.last_load {
            let next = last + MIN_GAP;
            if next > Instant::now() {
                sleep(next - Instant::now()).await;
            }
        }
        if state.browser.is_none() {
            state.browser = Some(self.connect().await?);
        }
        let connected = state.browser.as_ref().expect("connected above");
        let result = timeout(LOAD_TIMEOUT, load_page(connected, url))
            .await
            .unwrap_or_else(|_| Err(Failure::Browser(unavailable(format!("loading {url} timed out")))));
        state.last_load = Some(Instant::now());
        match result {
            Err(Failure::Browser(e)) => {
                // Reconnect next time.
                state.browser = None;
                Err(e)
            }
            Err(Failure::Page(e)) => Err(e),
            Ok(html) => Ok(html),
        }
    }
}

enum Failure {
    /// The browser connection is broken.
    Browser(SourceError),
    /// This page failed (e.g. a challenge that did not clear).
    Page(SourceError),
}

impl From<SourceError> for Failure {
    fn from(e: SourceError) -> Failure {
        Failure::Browser(e)
    }
}

async fn load_page(connected: &Connected, url: &str) -> Result<String, Failure> {
    let page = connected.browser.new_page("about:blank").await.map_err(cdp)?;
    let result = async {
        if let Some(agent) = &connected.user_agent {
            page.set_user_agent(agent.as_str()).await.map_err(cdp)?;
        }
        page.goto(url).await.map_err(cdp)?;
        let deadline = Instant::now() + CHALLENGE_WAIT;
        loop {
            let title = page.get_title().await.map_err(cdp)?.unwrap_or_default();
            if !title.is_empty() && !title.starts_with("Just a moment") {
                break;
            }
            if Instant::now() > deadline {
                return Err(Failure::Page(unavailable(format!(
                    "Cloudflare challenge on {url} did not clear"
                ))));
            }
            sleep(Duration::from_millis(500)).await;
        }
        // Let the page finish rendering after the challenge's redirect.
        sleep(Duration::from_secs(1)).await;
        Ok(page.content().await.map_err(cdp)?)
    }
    .await;
    if let Err(e) = page.close().await {
        tracing::debug!("closing tab: {e}");
    }
    result
}

/// `http://chrome:9222` -> `http://172.18.0.2:9222`; IPs, `localhost` and
/// `ws://` URLs are kept.
pub async fn resolve_endpoint(endpoint: &str) -> Result<String, SourceError> {
    let Some(rest) = endpoint.strip_prefix("http://") else {
        return Ok(endpoint.to_string());
    };
    let authority = rest.split('/').next().unwrap_or_default();
    let (host, port) = match authority.rsplit_once(':') {
        Some((h, p)) if !h.ends_with(']') || authority.starts_with('[') => {
            let port: u16 = p
                .parse()
                .map_err(|_| unavailable(format!("bad Chrome endpoint {endpoint:?}")))?;
            (h, port)
        }
        _ => (authority, 80),
    };
    let bare = host.trim_start_matches('[').trim_end_matches(']');
    if host == "localhost" || bare.parse::<IpAddr>().is_ok() {
        return Ok(endpoint.to_string());
    }
    let addr = tokio::net::lookup_host((host, port))
        .await
        .map_err(|e| unavailable(format!("resolving {host}: {e}")))?
        .next()
        .ok_or_else(|| unavailable(format!("{host} has no address")))?;
    Ok(format!("http://{addr}"))
}

fn unavailable(message: String) -> SourceError {
    SourceError::Unavailable(format!("hltv: {message}"))
}

fn cdp(e: chromiumoxide::error::CdpError) -> SourceError {
    unavailable(format!("chrome: {e}"))
}
