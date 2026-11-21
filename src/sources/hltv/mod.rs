//! HLTV Data Source adapter (ADR-0001): a pure parser ([`parse`], Seam 2)
//! and a fetch layer that loads pages through headless Chrome over CDP
//! ([`ChromePages`]). Team logos are downloaded once ([`LogoCache`]).
//!
//! `fetch_event` of a Major's Stage event returns the Major (the parent
//! HLTV event): the canonical Event, whose Stages are those Stage events.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::domain::{Event, EventId, EventSnapshot};
use crate::ports::{Clock, MatchSource, SourceError};

pub mod chrome;
pub mod logos;
pub mod parse;

pub use chrome::ChromePages;
pub use logos::LogoCache;
use parse::EventPage;

const BASE: &str = "https://www.hltv.org";

/// reqwest (ours, and chromiumoxide's for `/json/version`) is built on
/// rustls without a bundled crypto provider; install ring once.
fn install_tls_provider() {
    let _ = rustls::crypto::ring::default_provider().install_default();
}

/// Loads one page and returns its rendered HTML.
#[async_trait]
pub trait PageLoader: Send + Sync {
    async fn load(&self, url: &str) -> Result<String, SourceError>;
}

pub struct HltvSource {
    pages: Arc<dyn PageLoader>,
    clock: Arc<dyn Clock>,
    logos: Option<LogoCache>,
    /// Pages of Stage events HLTV marks as finished: they no longer change.
    finished_stages: Mutex<HashMap<EventId, EventPage>>,
}

impl HltvSource {
    pub fn new(pages: Arc<dyn PageLoader>, clock: Arc<dyn Clock>) -> HltvSource {
        HltvSource {
            pages,
            clock,
            logos: None,
            finished_stages: Mutex::new(HashMap::new()),
        }
    }

    /// Downloads the logo of every team it sees into `logos`, once.
    pub fn with_logos(mut self, logos: LogoCache) -> HltvSource {
        self.logos = Some(logos);
        self
    }

    async fn event_page(&self, id: EventId, slug: &str) -> Result<EventPage, SourceError> {
        let html = self.pages.load(&format!("{BASE}/events/{id}/{slug}")).await?;
        let page = parse::event_page(&html)?;
        if page.event.id != id {
            return Err(parse::parse_error(format!(
                "asked for event {id}, HLTV showed event {}",
                page.event.id
            )));
        }
        Ok(page)
    }

    async fn stage_page(&self, id: EventId, slug: &str) -> Result<EventPage, SourceError> {
        if let Some(page) = self.finished_stages.lock().await.get(&id) {
            return Ok(page.clone());
        }
        let page = self.event_page(id, slug).await?;
        if page.ended {
            self.finished_stages.lock().await.insert(id, page.clone());
        }
        Ok(page)
    }
}

#[async_trait]
impl MatchSource for HltvSource {
    async fn list_events(&self) -> Result<Vec<Event>, SourceError> {
        let html = self.pages.load(&format!("{BASE}/events")).await?;
        parse::event_list(&html)
    }

    async fn fetch_event(&self, id: EventId) -> Result<EventSnapshot, SourceError> {
        // HLTV redirects any slug to the event's own one.
        let mut main = self.event_page(id, "event").await?;
        if let Some(parent) = main.parent().cloned() {
            main = self.event_page(parent.id, &parent.slug).await?;
        }
        let mut stage_pages = Vec::new();
        for link in main.stage_events() {
            stage_pages.push(self.stage_page(link.id, &link.slug).await?);
        }
        let live = parse::live_matches(&self.pages.load(&format!("{BASE}/matches")).await?)?;
        let snapshot = parse::snapshot(&main, &stage_pages, &live, self.clock.now())?;

        if let Some(logos) = &self.logos {
            for team in &snapshot.teams {
                if let Err(e) = logos.ensure(team).await {
                    tracing::warn!("logo of team {} ({}): {e}", team.id, team.name);
                }
            }
        }
        Ok(snapshot)
    }
}
