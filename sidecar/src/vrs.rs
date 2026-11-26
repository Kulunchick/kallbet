//! Global Valve Markdown snapshots. A date-only publication is available after its UTC day.
use crate::*;
use chrono::NaiveDate;
use std::collections::BTreeMap;

struct Snapshot {
    observed: Instant,
    points: BTreeMap<String, i32>,
}
#[derive(Default)]
pub struct Standings(Vec<Snapshot>);
impl Standings {
    pub fn parse<'a>(documents: impl IntoIterator<Item = &'a str>) -> Result<Self> {
        let mut standings = Self::default();
        for doc in documents {
            standings.add(doc)?;
        }
        Ok(standings)
    }
    pub fn add(&mut self, doc: &str) -> Result<Instant> {
        let mut lines = doc.lines();
        let header = lines
            .next()
            .and_then(|s| s.strip_prefix("### Standings as of "))
            .ok_or("missing Valve header")?;
        let date = NaiveDate::parse_from_str(header.split('<').next().unwrap(), "%Y_%m_%d")?;
        let observed = date
            .and_hms_nano_opt(23, 59, 59, 999999999)
            .unwrap()
            .and_utc();
        let mut points = BTreeMap::new();
        for row in lines.filter(|s| s.starts_with('|')) {
            let mut cells = row.trim_matches('|').split('|').map(str::trim);
            let rank = cells.next().unwrap_or("");
            if matches!(rank, "Standing" | ":-") {
                continue;
            }
            rank.parse::<usize>()?;
            let rating = cells.next().ok_or("missing points")?.parse()?;
            let team = cells.next().ok_or("missing team")?.to_lowercase();
            points.entry(team).or_insert(rating);
        }
        if points.is_empty() {
            return Err("empty standings".into());
        }
        self.0.push(Snapshot { observed, points });
        self.0.sort_by_key(|s| s.observed);
        Ok(observed)
    }
    pub fn estimate(
        &self,
        a: &str,
        b: &str,
        format: Format,
        cutoff: Instant,
    ) -> Result<Option<AsOf<Signal>>> {
        let Some(s) = self.0.iter().rev().find(|s| s.observed < cutoff) else {
            return Ok(None);
        };
        let (Some(a), Some(b)) = (
            s.points.get(&a.to_lowercase()),
            s.points.get(&b.to_lowercase()),
        ) else {
            return Ok(None);
        };
        let rd = 75.0 * 10f64.ln() / (400.0 * std::f64::consts::PI);
        let g = (1.0 + 3.0 * rd.powi(2)).sqrt().recip();
        let p = (1.0 + 10f64.powf(g * (f64::from(*b) - f64::from(*a)) / 400.0)).recip();
        Ok(Some(AsOf::new(
            Signal {
                map: p,
                series: series_probability(p, format),
            },
            s.observed,
            cutoff,
        )?))
    }
}

/// Team A's series-win probability from its map-win probability `p`, maps
/// taken as independent: Bo1 `p`, Bo3 first to two, Bo5 first to three.
pub fn series_probability(p: f64, format: Format) -> f64 {
    match format {
        Format::Bo1 => p,
        Format::Bo3 => p * p * (3. - 2. * p),
        Format::Bo5 => p.powi(3) * (10. - 15. * p + 6. * p * p),
    }
}
