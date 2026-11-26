//! Pre-cutoff Event and team context from saved archive observations.
use super::{Dated, ReplayMatch, ReplayStage, StageKind, archive::WikiRevision};
use crate::adapters::context::wikitext::{find_templates, strip_comments};
use crate::{Instant, Result, team_names::team_key};
use serde_json::Value;
use std::{collections::BTreeMap, sync::Arc};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Environment {
    Lan,
    Online,
}
impl Environment {
    fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "lan" | "offline" => Some(Self::Lan),
            "online" => Some(Self::Online),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Region {
    Americas,
    Europe,
    AsiaPacific,
}
impl Region {
    fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "america" | "americas" | "am" | "na" | "sa" | "north america" | "south america" => {
                Some(Self::Americas)
            }
            "eu" | "eeu" | "weu" | "europe" => Some(Self::Europe),
            "as" | "asia" | "apac" | "oce" | "oceania" | "asia-pacific" => Some(Self::AsiaPacific),
            _ => None,
        }
    }

    fn host_country(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "denmark" | "germany" | "hungary" | "france" | "sweden" | "poland"
            | "united kingdom" | "portugal" | "spain" | "romania" | "serbia" | "finland"
            | "switzerland" | "netherlands" | "malta" | "czechia" => Some(Self::Europe),
            "united states" | "canada" | "brazil" | "argentina" | "mexico" => Some(Self::Americas),
            "china" | "singapore" | "australia" | "mongolia" | "japan" | "south korea" => {
                Some(Self::AsiaPacific)
            }
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BracketRound {
    RoundOf16,
    Quarterfinal,
    Semifinal,
    Final,
}
impl BracketRound {
    pub(super) fn parse(name: Option<&str>) -> Option<Self> {
        let round = name?.split(':').next()?.trim().to_ascii_lowercase();
        if round == "round of 16" {
            return Some(Self::RoundOf16);
        }
        let label = round
            .trim_end_matches(|c: char| c.is_ascii_digit())
            .trim_end();
        match label {
            "quarterfinal" | "quarter-final" => Some(Self::Quarterfinal),
            "semifinal" | "semi-final" => Some(Self::Semifinal),
            "grand final" | "final" => Some(Self::Final),
            _ => None,
        }
    }
}

#[derive(Clone, Default)]
struct Venue {
    environments: Vec<Dated<Environment>>,
    regions: Vec<Dated<Region>>,
}
impl Venue {
    fn add(&mut self, at: Instant, environment: Option<Environment>, region: Option<Region>) {
        if let Some(value) = environment {
            self.environments.push(Dated {
                observed_at: at,
                value,
            });
        }
        if let Some(value) = region {
            self.regions.push(Dated {
                observed_at: at,
                value,
            });
        }
    }

    fn sort(&mut self) {
        self.environments.sort_by_key(|d| d.observed_at);
        self.regions.sort_by_key(|d| d.observed_at);
    }
}

#[derive(Default)]
struct Index {
    stages: BTreeMap<i64, Venue>,
    events: BTreeMap<i64, Venue>,
    team_regions: BTreeMap<(i64, String), Vec<Dated<Region>>>,
}

#[derive(Default)]
pub struct ArchiveContext(Arc<Index>);

fn before<T>(data: &[Dated<T>], cutoff: Instant) -> Option<&Dated<T>> {
    data[..data.partition_point(|d| d.observed_at < cutoff)].last()
}

impl ArchiveContext {
    pub fn pandascore(&mut self, stage_id: i64, row: &Value) {
        let tournament = &row["tournament"];
        let Some(at) = tournament["modified_at"]
            .as_str()
            .and_then(|s| s.parse().ok())
        else {
            return;
        };
        Arc::get_mut(&mut self.0)
            .expect("archive is still loading")
            .stages
            .entry(stage_id)
            .or_default()
            .add(
                at,
                tournament["type"].as_str().and_then(Environment::parse),
                tournament["region"].as_str().and_then(Region::parse),
            );
    }

    pub fn wiki(&mut self, event_id: i64, page: &WikiRevision) -> Result<()> {
        let json: Value = serde_json::from_str(&page.body)?;
        let Some(text) =
            json["query"]["pages"][0]["revisions"][0]["slots"]["main"]["content"].as_str()
        else {
            return Ok(());
        };
        let text = strip_comments(text);
        let index = Arc::get_mut(&mut self.0).expect("archive is still loading");
        if let Some(infobox) = find_templates(&text, "Infobox league").first() {
            let environment = infobox.get("type").and_then(Environment::parse);
            let region = infobox.get("region").and_then(Region::parse).or_else(|| {
                (environment == Some(Environment::Lan))
                    .then(|| infobox.get("country").and_then(Region::host_country))
                    .flatten()
            });
            index
                .events
                .entry(event_id)
                .or_default()
                .add(page.timestamp, environment, region);
        }
        for card in find_templates(&text, "TeamCard") {
            let Some(team) = card.get("team") else {
                continue;
            };
            let region = card.get("qualifier").and_then(|qualifier| {
                let vrs = find_templates(qualifier, "VRS");
                Region::parse(vrs.first()?.positional().next()?)
            });
            if let Some(value) = region {
                index
                    .team_regions
                    .entry((event_id, team_key(team)))
                    .or_default()
                    .push(Dated {
                        observed_at: page.timestamp,
                        value,
                    });
            }
        }
        Ok(())
    }

    pub fn sort(&mut self) {
        let index = Arc::get_mut(&mut self.0).expect("archive is still loading");
        for venue in index.stages.values_mut().chain(index.events.values_mut()) {
            venue.sort();
        }
        for regions in index.team_regions.values_mut() {
            regions.sort_by_key(|d| d.observed_at);
        }
    }

    pub fn view(&self, stage: &ReplayStage, game: &ReplayMatch) -> MatchContext {
        MatchContext {
            index: self.0.clone(),
            event_id: stage.event_id,
            stage_id: stage.id,
            stage_kind: stage.kind,
            round: game.round,
            bracket_round: game.bracket_round,
        }
    }
}

#[derive(Clone)]
pub struct MatchContext {
    index: Arc<Index>,
    event_id: i64,
    pub stage_id: i64,
    pub stage_kind: StageKind,
    pub round: Option<u32>,
    pub bracket_round: Option<BracketRound>,
}

impl MatchContext {
    fn venue<T>(
        &self,
        stage: i64,
        event: i64,
        cutoff: Instant,
        field: impl Fn(&Venue) -> &[Dated<T>],
    ) -> Option<&Dated<T>> {
        self.index
            .stages
            .get(&stage)
            .and_then(|v| before(field(v), cutoff))
            .or_else(|| {
                self.index
                    .events
                    .get(&event)
                    .and_then(|v| before(field(v), cutoff))
            })
    }

    pub fn environment(&self, cutoff: Instant) -> Option<&Dated<Environment>> {
        self.history_environment(self.stage_id, self.event_id, cutoff)
    }

    pub fn history_environment(
        &self,
        stage_id: i64,
        event_id: i64,
        cutoff: Instant,
    ) -> Option<&Dated<Environment>> {
        self.venue(stage_id, event_id, cutoff, |v| &v.environments)
    }

    pub fn region(&self, cutoff: Instant) -> Option<&Dated<Region>> {
        self.venue(self.stage_id, self.event_id, cutoff, |v| &v.regions)
    }

    pub fn team_region(&self, team: &str, cutoff: Instant) -> Option<&Dated<Region>> {
        before(
            self.index
                .team_regions
                .get(&(self.event_id, team_key(team)))?,
            cutoff,
        )
    }
}
