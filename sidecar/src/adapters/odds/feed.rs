use super::{kalshi, lines::Line, oddspapi, polymarket};
use crate::{Clock, Result};
use async_trait::async_trait;
use chrono::Duration;
use reqwest::{Client, StatusCode, Url};

#[derive(Debug, thiserror::Error)]
pub enum FetchError {
    #[error("odds feed is rate limited")]
    RateLimited,
    #[error("odds feed is unavailable")]
    Unavailable,
}

#[async_trait]
pub trait OddsTransport: Send + Sync {
    async fn get(&self, url: Url) -> Result<Vec<u8>, FetchError>;
}

pub(super) struct HttpTransport(Client);

impl HttpTransport {
    pub fn new() -> Result<Self> {
        let _ = rustls::crypto::ring::default_provider().install_default();
        Ok(Self(
            Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .timeout(std::time::Duration::from_secs(5))
                .user_agent("KallBet/0.1 (https://kallbet.kulunchick.tech)")
                .build()?,
        ))
    }
}

#[async_trait]
impl OddsTransport for HttpTransport {
    async fn get(&self, url: Url) -> Result<Vec<u8>, FetchError> {
        // OddsPapi puts its key in the URL. Discard transport errors and bodies
        // on non-success responses so credentials never enter errors or logs.
        let response = self
            .0
            .get(url)
            .send()
            .await
            .map_err(|_| FetchError::Unavailable)?;
        if response.status() == StatusCode::TOO_MANY_REQUESTS {
            return Err(FetchError::RateLimited);
        }
        if !response.status().is_success() {
            return Err(FetchError::Unavailable);
        }
        response
            .bytes()
            .await
            .map(|body| body.to_vec())
            .map_err(|_| FetchError::Unavailable)
    }
}

pub(super) enum Feed<'a> {
    OddsPapi { ids: &'a str, key: &'a str },
    Polymarket { tag: &'a str },
    Kalshi,
}

impl Feed<'_> {
    pub fn ttl(&self) -> Duration {
        match self {
            Self::OddsPapi { .. } => Duration::hours(6),
            _ => Duration::minutes(2),
        }
    }

    pub async fn fetch(&self, http: &dyn OddsTransport, clock: &dyn Clock) -> Result<Vec<Line>> {
        const PAGE_LIMIT: usize = 100;
        // Bound pagination in the live tick. A full final page is treated as
        // unavailable instead of saving an incomplete snapshot.
        const MAX_PAGES: usize = 10;
        match self {
            Self::OddsPapi { ids, key } => {
                let mut url = Url::parse("https://api.oddspapi.io/v4/odds-by-tournaments")?;
                url.query_pairs_mut().extend_pairs([
                    ("tournamentIds", *ids),
                    ("bookmaker", oddspapi::FREE_PINNACLE),
                    ("language", "en"),
                    ("verbosity", "3"),
                    ("apiKey", *key),
                ]);
                oddspapi::parse_lines(&http.get(url).await?)
            }
            Self::Polymarket { tag } => {
                let mut base = Url::parse("https://gamma-api.polymarket.com/events")?;
                base.query_pairs_mut()
                    .append_pair("tag_slug", tag)
                    .append_pair("closed", "false")
                    .append_pair("limit", &PAGE_LIMIT.to_string());
                let mut lines = Vec::new();
                for page in 0..MAX_PAGES {
                    let mut url = base.clone();
                    url.query_pairs_mut()
                        .append_pair("offset", &(page * PAGE_LIMIT).to_string());
                    let (quotes, count) = polymarket::parse_page(&http.get(url).await?)?;
                    lines.extend(quotes);
                    if count < PAGE_LIMIT {
                        return Ok(lines);
                    }
                }
                Err("Polymarket events exceeded the page limit".into())
            }
            Self::Kalshi => {
                let mut base = Url::parse("https://api.elections.kalshi.com/trade-api/v2/events")?;
                base.query_pairs_mut().extend_pairs([
                    ("series_ticker", kalshi::SERIES),
                    ("status", "open"),
                    ("with_nested_markets", "true"),
                    ("limit", &PAGE_LIMIT.to_string()),
                ]);
                let mut cursor = String::new();
                let mut lines = Vec::new();
                for _ in 0..MAX_PAGES {
                    let mut url = base.clone();
                    if !cursor.is_empty() {
                        url.query_pairs_mut().append_pair("cursor", &cursor);
                    }
                    let body = http.get(url).await?;
                    let (quotes, next) = kalshi::parse_page(&body, clock.now())?;
                    lines.extend(quotes);
                    if next.is_empty() {
                        return Ok(lines);
                    }
                    if next == cursor {
                        return Err("Kalshi repeated its events cursor".into());
                    }
                    cursor = next;
                }
                Err("Kalshi events exceeded the page limit".into())
            }
        }
    }
}
