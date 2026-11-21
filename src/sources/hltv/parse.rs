//! Pure HLTV parser: rendered HTML in, canonical model out. No I/O.

use chrono::{DateTime, TimeZone, Utc};
use scraper::{ElementRef, Html, Selector};

use crate::domain::{Event, EventId};
use crate::ports::SourceError;

/// Current and upcoming top Events from `https://www.hltv.org/events`.
pub fn event_list(html: &str) -> Result<Vec<Event>, SourceError> {
    let doc = Html::parse_document(html);
    let mut events: Vec<(Event, String)> = Vec::new();
    for a in doc.select(&sel("a.ongoing-event, a.big-event")) {
        let href = a.value().attr("href").unwrap_or_default();
        let (id, slug) = event_href(href)
            .ok_or_else(|| parse_error(format!("event link without an id: {href:?}")))?;
        if events.iter().any(|(e, _)| e.id == id) {
            continue;
        }
        let name = first_text(a, ".event-name-small .text-ellipsis, .big-event-name")
            .ok_or_else(|| parse_error(format!("event {id} has no name")))?;
        let dates: Vec<DateTime<Utc>> = a
            .select(&sel("[data-unix]"))
            .map(|d| unix_ms(d.value().attr("data-unix").unwrap_or_default()))
            .collect::<Result<_, _>>()?;
        let event = Event {
            id,
            name,
            starts_at: dates.first().copied(),
            ends_at: dates.get(1).copied(),
        };
        events.push((event, slug));
    }
    if events.is_empty() {
        return Err(parse_error("the event list has no events".into()));
    }
    // A Stage of a Major is its own HLTV event; list only the Major itself.
    let slugs: Vec<String> = events.iter().map(|(_, s)| s.clone()).collect();
    Ok(events
        .into_iter()
        .filter(|(_, slug)| !slugs.iter().any(|parent| is_stage_slug_of(slug, parent)))
        .map(|(e, _)| e)
        .collect())
}

/// `pgl-major-singapore-2026-stage-1` is a Stage of `pgl-major-singapore-2026`.
fn is_stage_slug_of(slug: &str, parent: &str) -> bool {
    slug.strip_prefix(parent)
        .is_some_and(|rest| rest.starts_with("-stage-"))
}

pub(crate) fn parse_error(message: String) -> SourceError {
    SourceError::Parse(format!("hltv: {message}"))
}

fn sel(css: &str) -> Selector {
    Selector::parse(css).expect("static selector")
}

fn first_text(el: ElementRef<'_>, css: &str) -> Option<String> {
    el.select(&sel(css))
        .map(|e| e.text().collect::<String>().trim().to_string())
        .find(|t| !t.is_empty())
}

/// `/events/8302/pgl-major-singapore-2026` -> (8302, "pgl-major-singapore-2026").
fn event_href(href: &str) -> Option<(EventId, String)> {
    let rest = href
        .strip_prefix("https://www.hltv.org")
        .unwrap_or(href)
        .strip_prefix("/events/")?;
    let (id, slug) = rest.split_once('/')?;
    let slug = slug.split(['#', '?', '/']).next().unwrap_or_default();
    Some((EventId(id.parse().ok()?), slug.to_string()))
}

fn unix_ms(value: &str) -> Result<DateTime<Utc>, SourceError> {
    value
        .parse::<i64>()
        .ok()
        .and_then(|ms| Utc.timestamp_millis_opt(ms).single())
        .ok_or_else(|| parse_error(format!("bad data-unix timestamp {value:?}")))
}
