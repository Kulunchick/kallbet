//! PandaScore Data Source adapter: pure parser ([`parse`]) + REST fetch layer
//! (this file). Owner: #10.
//!
//! A fallback adapter: it implements
//! [`ExternalSource`](crate::sources::fallback::ExternalSource) (records with
//! PandaScore's own ids), and the fallback chain maps them to canonical ids.
//! `runtime::match_source` puts it in the chain. Without `PANDASCORE_TOKEN`
//! it answers `NotConfigured` (skipped, not a failure) and makes no request.
//!
//! Endpoints (all on every plan, CS2 lives under `/csgo/`; see the research
//! notes in `docs/agents/codebase.md`): `/csgo/series/running|upcoming` for
//! the Event list, `/csgo/series` (filtered by id or by date range) to find
//! one, `/series/{id}/tournaments` for its Stages and
//! `/tournaments/{id}/matches` (`/brackets` for a bracket Stage, which adds
//! `previous_matches`) for the Matches.
//!
//! Verified against the real API (2026-11-22, the owner's plan, see
//! `tests/fixtures/pandascore/README.md`): every request form used here
//! answers 200 (`/csgo/series?filter[id]=`, `...?search[name]=`,
//! `...?range[begin_at]=<date>,<date>`, `/csgo/series/running|upcoming`,
//! `/series/{id}/tournaments`, `/tournaments/{id}/matches|brackets`, each
//! with `page[size]`). `search[name]` matches the serie's own `name`
//! ("Cologne Major"), not the league ("IEM") or `full_name`, so an undated
//! search by an HLTV name finds little; Events always have dates in
//! practice. `/series/{id}/tournaments` embeds matches too, but without
//! opponents or results, so they are not used.
//!
//! UNVERIFIED: page numbering beyond page 1 (`page[number]`), and
//! `/brackets` of a tournament that is not a single-elimination tree.

pub mod parse;

use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::domain::{DataSourceKind, StageKind};
use crate::ports::SourceError;
use crate::sources::fallback::{EventHint, ExternalEvent, ExternalSnapshot, ExternalSource};
use parse::TournamentInput;

const API_BASE: &str = "https://api.pandascore.co";
const TIMEOUT: Duration = Duration::from_secs(15);
const PER_PAGE: usize = 100;
/// Safety stop: the free plan allows 1000 requests an hour.
const MAX_PAGES: usize = 10;
/// How far a serie's start may be from the canonical Event's start.
const SEARCH_WINDOW_DAYS: i64 = 7;

pub struct PandaScoreSource {
    /// `PANDASCORE_TOKEN`; `None` means the adapter is skipped.
    token: Option<String>,
    base_url: String,
    client: reqwest::Client,
}

impl PandaScoreSource {
    pub fn new(token: Option<String>) -> PandaScoreSource {
        PandaScoreSource::with_base_url(token, API_BASE)
    }

    /// Talks to another server than PandaScore's (tests use a local one).
    pub fn with_base_url(token: Option<String>, base_url: &str) -> PandaScoreSource {
        // reqwest is built on rustls without a bundled crypto provider (see
        // `sources::hltv`): install ring once; a second install is a no-op.
        let _ = rustls::crypto::ring::default_provider().install_default();
        PandaScoreSource {
            token: token.filter(|t| !t.is_empty()),
            base_url: base_url.trim_end_matches('/').to_string(),
            client: reqwest::Client::builder()
                .timeout(TIMEOUT)
                .build()
                .expect("an HTTP client with default TLS settings"),
        }
    }

    fn token(&self) -> Result<&str, SourceError> {
        self.token.as_deref().ok_or(SourceError::NotConfigured)
    }

    /// One GET, answered as JSON. The token travels in a header, and no
    /// error text ever contains it or the URL.
    async fn get(&self, path: &str, query: &[(&str, String)]) -> Result<Value, SourceError> {
        let token = self.token()?;
        let response = self
            .client
            .get(format!("{}{path}{}", self.base_url, query_string(query)))
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| SourceError::Unavailable(format!("PandaScore request failed: {}", e.without_url())))?;
        let status = response.status();
        match status.as_u16() {
            200..=299 => {}
            404 => return Err(SourceError::NotFound),
            code => {
                return Err(SourceError::Unavailable(format!(
                    "PandaScore answered {code} for {path}"
                )));
            }
        }
        response
            .json::<Value>()
            .await
            .map_err(|e| SourceError::Parse(format!("PandaScore: not JSON ({})", e.without_url())))
    }

    /// Every page of a list endpoint.
    async fn get_all(
        &self,
        path: &str,
        query: &[(&str, String)],
    ) -> Result<Vec<Value>, SourceError> {
        let mut all = Vec::new();
        for page in 1..=MAX_PAGES {
            let mut q = query.to_vec();
            q.push(("page[number]", page.to_string()));
            q.push(("page[size]", PER_PAGE.to_string()));
            let body = self.get(path, &q).await?;
            let items = body.as_array().ok_or_else(|| {
                SourceError::Parse(format!("PandaScore: {path} did not answer a JSON array"))
            })?;
            let last = items.len() < PER_PAGE;
            all.extend(items.iter().cloned());
            if last {
                return Ok(all);
            }
        }
        Err(SourceError::Parse(format!("PandaScore: {path} has more than {MAX_PAGES} pages")))
    }

    /// A list of series; a missing endpoint is a failure here, not "no such Event".
    async fn series(&self, path: &str, query: &[(&str, String)]) -> Result<Vec<ExternalEvent>, SourceError> {
        let items = match self.get_all(path, query).await {
            Err(SourceError::NotFound) => {
                return Err(SourceError::Unavailable(format!("PandaScore has no {path}")));
            }
            other => other?,
        };
        parse::events(&Value::Array(items))
    }
}

/// `?a=b&c=d`, percent-encoded (reqwest's `query` feature is not enabled).
fn query_string(query: &[(&str, String)]) -> String {
    fn encode(s: &str) -> String {
        s.bytes()
            .map(|b| match b {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    (b as char).to_string()
                }
                _ => format!("%{b:02X}"),
            })
            .collect()
    }
    if query.is_empty() {
        return String::new();
    }
    let pairs: Vec<String> =
        query.iter().map(|(k, v)| format!("{}={}", encode(k), encode(v))).collect();
    format!("?{}", pairs.join("&"))
}

fn day(t: DateTime<Utc>) -> String {
    t.format("%Y-%m-%d").to_string()
}

#[async_trait]
impl ExternalSource for PandaScoreSource {
    fn kind(&self) -> DataSourceKind {
        DataSourceKind::PandaScore
    }

    async fn list_events(&self) -> Result<Vec<ExternalEvent>, SourceError> {
        self.token()?;
        let mut events = self.series("/csgo/series/running", &[]).await?;
        for e in self.series("/csgo/series/upcoming", &[]).await? {
            if !events.iter().any(|known| known.external_id == e.external_id) {
                events.push(e);
            }
        }
        Ok(events)
    }

    async fn find_events(&self, hint: &EventHint) -> Result<Vec<ExternalEvent>, SourceError> {
        self.token()?;
        let window = chrono::Duration::days(SEARCH_WINDOW_DAYS);
        let query = match (hint.starts_at, hint.name.is_empty()) {
            // Series starting around the Event's start; the chain picks by name.
            (Some(start), _) => vec![
                ("range[begin_at]", format!("{},{}", day(start - window), day(start + window))),
            ],
            (None, false) => vec![("search[name]", hint.name.clone())],
            (None, true) => return Ok(vec![]),
        };
        self.series("/csgo/series", &query).await
    }

    async fn fetch_event(&self, external_id: &str) -> Result<ExternalSnapshot, SourceError> {
        self.token()?;
        let Ok(id) = external_id.parse::<i64>() else {
            return Err(SourceError::NotFound);
        };
        let found = self
            .get_all("/csgo/series", &[("filter[id]", id.to_string())])
            .await?;
        let serie = found.first().ok_or(SourceError::NotFound)?;

        let tournaments = self.get_all(&format!("/series/{id}/tournaments"), &[]).await?;
        let mut inputs = Vec::new();
        for tournament in tournaments {
            let tid = tournament["id"].as_i64().ok_or_else(|| {
                SourceError::Parse("PandaScore: a tournament has no id".to_string())
            })?;
            // The Playoffs come from /brackets: it carries `previous_matches`.
            // A group phase flagged `has_bracket` is read as plain matches.
            let endpoint = if parse::stage_kind(&tournament)? == StageKind::Playoffs {
                "brackets"
            } else {
                "matches"
            };
            let matches = self.get_all(&format!("/tournaments/{tid}/{endpoint}"), &[]).await?;
            inputs.push(TournamentInput { tournament, matches });
        }
        parse::snapshot(serie, &inputs)
    }
}
