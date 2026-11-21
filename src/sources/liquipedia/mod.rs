//! Liquipedia Data Source adapter (#11): a pure parser ([`parse`], Seam 2)
//! and a fetch layer over the MediaWiki API (`liquipedia.net/counterstrike/api.php`).
//!
//! A fallback adapter: it implements
//! [`ExternalSource`](crate::sources::fallback::ExternalSource) with
//! Liquipedia's own ids (an Event is its root page title, e.g.
//! `Intel Extreme Masters/2026/Cologne`); a Match template's `hltv=` goes
//! into `ExternalMatch::canonical` and the fallback chain does the mapping.
//!
//! API rules we follow (<https://liquipedia.net/api-terms-of-use>): one
//! request per 2 seconds, [`limiter`]; no `action=parse` and no scraping of
//! rendered pages, only `action=query` (wikitext) plus one
//! `action=expandtemplates` per batch of unseen team keys, held to 30 seconds
//! apart; a User-Agent naming the project; gzip. Team names are cached for
//! the life of the process. The terms also require attributing Liquipedia
//! (the SPA shows the Data Source of each Match, story 34).

use std::collections::{BTreeSet, HashMap};
use std::sync::{Arc, Mutex};
use std::time::Duration as StdDuration;

use async_trait::async_trait;
use chrono::Duration;

use crate::domain::DataSourceKind;
use crate::ports::{Clock, SourceError};
use crate::sources::fallback::{EventHint, ExternalEvent, ExternalSnapshot, ExternalSource};

pub mod limiter;
pub mod parse;
pub mod wikitext;

use limiter::{Class, RateLimiter};
use parse::{Page, TeamNames};

pub const USER_AGENT: &str = "KallBet/0.1 (https://kallbet.kulunchick.tech)";
const API_URL: &str = "https://liquipedia.net/counterstrike/api.php";
/// A request that would have to wait longer than this fails instead: the
/// poller is not held for minutes (it tries again on its next pass).
const MAX_WAIT: Duration = Duration::seconds(60);
/// Pages per `titles=` request the API accepts.
const MAX_TITLES: usize = 50;

/// Why a request did not return a body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WikiFailure {
    /// HTTP 429, with `Retry-After` if sent.
    RateLimited(Option<Duration>),
    /// HTTP 403: banned or blocked.
    Refused,
    Other(String),
}

/// One GET against the wiki API.
#[async_trait]
pub trait Wiki: Send + Sync {
    async fn get(&self, params: &[(&str, &str)]) -> Result<String, WikiFailure>;
}

/// Waits; a port so tests do not sleep.
#[async_trait]
pub trait Pause: Send + Sync {
    async fn pause(&self, for_: Duration);
}

pub struct TokioPause;

#[async_trait]
impl Pause for TokioPause {
    async fn pause(&self, for_: Duration) {
        if let Ok(d) = for_.to_std() {
            tokio::time::sleep(d).await;
        }
    }
}

/// The real API over HTTPS.
pub struct HttpWiki {
    client: reqwest::Client,
    api_url: String,
}

impl HttpWiki {
    pub fn new() -> HttpWiki {
        HttpWiki::with_api_url(API_URL)
    }

    /// Another endpoint (a local stand-in, in tests).
    pub fn with_api_url(api_url: impl Into<String>) -> HttpWiki {
        // reqwest is built on rustls without a bundled crypto provider.
        let _ = rustls::crypto::ring::default_provider().install_default();
        HttpWiki {
            api_url: api_url.into(),
            client: reqwest::Client::builder()
                .user_agent(USER_AGENT)
                .gzip(true)
                .timeout(StdDuration::from_secs(30))
                .build()
                .expect("an HTTP client"),
        }
    }
}

impl Default for HttpWiki {
    fn default() -> HttpWiki {
        HttpWiki::new()
    }
}

#[async_trait]
impl Wiki for HttpWiki {
    async fn get(&self, params: &[(&str, &str)]) -> Result<String, WikiFailure> {
        let url = reqwest::Url::parse_with_params(
            &self.api_url,
            [("format", "json"), ("formatversion", "2")]
                .into_iter()
                .chain(params.iter().copied()),
        )
        .map_err(|e| WikiFailure::Other(e.to_string()))?;
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| WikiFailure::Other(e.without_url().to_string()))?;
        let status = response.status();
        if status.as_u16() == 429 {
            let retry = response
                .headers()
                .get(reqwest::header::RETRY_AFTER)
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<i64>().ok())
                .map(Duration::seconds);
            return Err(WikiFailure::RateLimited(retry));
        }
        if status.as_u16() == 403 {
            return Err(WikiFailure::Refused);
        }
        if !status.is_success() {
            return Err(WikiFailure::Other(format!("HTTP {status}")));
        }
        response
            .text()
            .await
            .map_err(|e| WikiFailure::Other(e.without_url().to_string()))
    }
}

pub struct LiquipediaSource {
    wiki: Arc<dyn Wiki>,
    pause: Arc<dyn Pause>,
    clock: Arc<dyn Clock>,
    limiter: Mutex<RateLimiter>,
    /// Team template key -> name, for good: a team's template does not change
    /// during an Event, and every lookup costs a slow request.
    names: Mutex<TeamNames>,
}

impl LiquipediaSource {
    /// The production adapter: real API, real waiting.
    pub fn new(clock: Arc<dyn Clock>) -> LiquipediaSource {
        LiquipediaSource::with(Arc::new(HttpWiki::new()), Arc::new(TokioPause), clock)
    }

    pub fn with(wiki: Arc<dyn Wiki>, pause: Arc<dyn Pause>, clock: Arc<dyn Clock>) -> Self {
        LiquipediaSource {
            wiki,
            pause,
            clock,
            limiter: Mutex::new(RateLimiter::new()),
            names: Mutex::new(HashMap::new()),
        }
    }

    /// One request, after waiting for its slot.
    async fn request(&self, class: Class, params: &[(&str, &str)]) -> Result<String, SourceError> {
        let wait = self
            .limiter
            .lock()
            .unwrap()
            .reserve(self.clock.now(), class, MAX_WAIT)
            .ok_or_else(|| {
                SourceError::Unavailable("Liquipedia rate limit: next slot too far off".into())
            })?;
        if wait > Duration::zero() {
            self.pause.pause(wait).await;
        }
        match self.wiki.get(params).await {
            Ok(body) => Ok(body),
            Err(failure) => {
                let (back_off, message) = match failure {
                    WikiFailure::RateLimited(retry) => (
                        Some(retry.unwrap_or(Duration::minutes(2))),
                        "rate limited (HTTP 429)".to_string(),
                    ),
                    WikiFailure::Refused => (
                        Some(Duration::minutes(10)),
                        "refused (HTTP 403)".to_string(),
                    ),
                    WikiFailure::Other(m) => (None, m),
                };
                if let Some(for_) = back_off {
                    self.limiter.lock().unwrap().back_off(self.clock.now(), for_);
                }
                Err(SourceError::Unavailable(message))
            }
        }
    }

    async fn wikitext(&self, titles: &[String], section0: bool) -> Result<Vec<Page>, SourceError> {
        let joined = titles.join("|");
        let mut params = vec![
            ("action", "query"),
            ("prop", "revisions"),
            ("rvprop", "content"),
            ("rvslots", "main"),
            ("titles", joined.as_str()),
        ];
        if section0 {
            params.push(("rvsection", "0"));
        }
        parse::query_pages(&self.request(Class::Normal, &params).await?)
    }

    /// Names for every key not yet known; one request.
    async fn learn_names(&self, keys: &BTreeSet<String>) -> Result<(), SourceError> {
        let unknown: BTreeSet<String> = {
            let known = self.names.lock().unwrap();
            keys.iter().filter(|k| !known.contains_key(*k)).cloned().collect()
        };
        if unknown.is_empty() {
            return Ok(());
        }
        let text = parse::team_name_query(&unknown);
        let body = self
            .request(
                Class::Heavy,
                &[
                    ("action", "expandtemplates"),
                    ("prop", "wikitext"),
                    ("text", text.as_str()),
                ],
            )
            .await?;
        let found = parse::team_names(&body)?;
        let mut known = self.names.lock().unwrap();
        for key in unknown {
            // A key the wiki could not name is remembered under its own key,
            // so a broken template does not cost a request on every poll.
            let name = found.get(&key).cloned().unwrap_or_else(|| key.clone());
            known.insert(key, name);
        }
        Ok(())
    }
}

#[async_trait]
impl ExternalSource for LiquipediaSource {
    fn kind(&self) -> DataSourceKind {
        DataSourceKind::Liquipedia
    }

    /// Liquipedia's tournament lists are generated by Lua modules, readable
    /// only through `action=parse` (limited to one request per 30 seconds and
    /// not meant for this), so this adapter does not list Events. It answers
    /// `NotConfigured`, which the chain skips without counting a failure; it
    /// serves Events HLTV (or a League) already named, through `find_events`.
    async fn list_events(&self) -> Result<Vec<ExternalEvent>, SourceError> {
        Err(SourceError::NotConfigured)
    }

    async fn find_events(&self, hint: &EventHint) -> Result<Vec<ExternalEvent>, SourceError> {
        // Liquipedia knows no HLTV event id, so without a name there is
        // nothing to search for.
        if hint.name.trim().is_empty() {
            return Ok(Vec::new());
        }
        let found = self
            .request(
                Class::Normal,
                &[
                    ("action", "query"),
                    ("list", "search"),
                    ("srsearch", hint.name.as_str()),
                    ("srnamespace", "0"),
                    ("srlimit", "8"),
                    ("srprop", ""),
                ],
            )
            .await?;
        let titles = parse::listed_titles(&found)?;
        if titles.is_empty() {
            return Ok(Vec::new());
        }
        // The infobox sits in section 0; only league pages have one.
        let pages = self.wikitext(&titles, true).await?;
        Ok(pages
            .iter()
            .filter_map(|p| parse::event_header(&p.title, &p.wikitext))
            .collect())
    }

    async fn fetch_event(&self, external_id: &str) -> Result<ExternalSnapshot, SourceError> {
        let prefix = format!("{external_id}/");
        let listing = self
            .request(
                Class::Normal,
                &[
                    ("action", "query"),
                    ("list", "allpages"),
                    ("apprefix", prefix.as_str()),
                    ("aplimit", "50"),
                ],
            )
            .await?;
        let subpages = parse::stage_subpages(&parse::listed_titles(&listing)?, external_id);

        let mut titles = vec![external_id.to_string()];
        titles.extend(subpages.into_iter().take(MAX_TITLES - 1));
        let pages = self.wikitext(&titles, false).await?;
        // Root first, then the Stage pages in play order.
        let mut ordered: Vec<Page> = Vec::new();
        for title in &titles {
            if let Some(p) = pages.iter().find(|p| &p.title == title) {
                ordered.push(p.clone());
            }
        }
        if ordered.first().map(|p| p.title.as_str()) != Some(external_id) {
            return Err(SourceError::NotFound);
        }

        let parsed = parse::parse_event(external_id, &ordered)?;
        self.learn_names(&parsed.team_keys()).await?;
        let names = self.names.lock().unwrap().clone();
        Ok(parsed.into_snapshot(&names, self.clock.now()))
    }
}
