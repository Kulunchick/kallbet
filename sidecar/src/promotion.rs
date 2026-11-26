//! A promoted Candidate Signal, or the frozen combination, run live (ticket
//! #47). A bot selects it with `Policy::Promoted { name, report }`: `report`
//! is a Signal Lab `report.json`, whose verdicts decide what may be selected
//! and whose `deployment` section holds the fitted parameters.
use crate::adapters::lab::{
    self, CandidateSignal, FittedSignal, MatchView,
    combiner::{Combiner, Inputs},
    signals,
};
use crate::*;
use serde_json::{Value, json};
use std::{fs, path::Path, sync::Arc};

/// The name that selects the frozen combination of a lab run.
pub const COMBINATION: &str = "combination";

pub struct Promotion {
    members: Vec<(Arc<dyn CandidateSignal>, Box<dyn FittedSignal>)>,
    combination: Option<Combiner>,
    /// `name/<sha-256 of the report>`: recorded in every dossier the bot decides on.
    pub identity: String,
    /// What the Rationale names: the signal, or the combination's members.
    pub label: String,
}

/// Team A's probability from a promotion, and whether its own data was there.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Live {
    /// `None`: neither the signal nor the anchor could predict this Match.
    pub probability: Option<f64>,
    /// `false`: the signal's live data was missing and this is the anchor.
    pub available: bool,
}

/// The Candidate Signals whose inputs the live ports give (the market line and
/// the Valve rating). The others read the archive's Match history, which the
/// live sidecar has no port for.
pub const LIVE: [&str; 2] = [signals::calibration::NAME, signals::learned_weights::NAME];

fn decode(name: &str, parameters: Value) -> Result<Box<dyn FittedSignal>> {
    use signals::{calibration, learned_weights};
    Ok(match name {
        calibration::NAME => Box::new(serde_json::from_value::<calibration::Calibrated>(
            parameters,
        )?),
        learned_weights::NAME => Box::new(serde_json::from_value::<learned_weights::Learned>(
            parameters,
        )?),
        _ => unreachable!("checked against LIVE"),
    })
}

impl Promotion {
    /// Reads `name` from a Signal Lab report. A name the lab did not promote
    /// (or, for `combination`, a run that froze nothing) is an error saying why.
    pub fn load(name: &str, path: &Path) -> Result<Self> {
        let context = |e: Box<dyn std::error::Error + Send + Sync>| -> Box<dyn std::error::Error + Send + Sync> {
            format!("promoted criterion {name} ({}): {e}", path.display()).into()
        };
        Self::read(name, path).map_err(context)
    }

    fn read(name: &str, path: &Path) -> Result<Self> {
        let bytes = fs::read(path)?;
        let report: Value = serde_json::from_slice(&bytes)?;
        if report["protocol_version"] != lab::PROTOCOL_VERSION {
            return Err(format!(
                "the report's protocol is {}, not {}",
                report["protocol_version"],
                lab::PROTOCOL_VERSION
            )
            .into());
        }
        let names: Vec<String> = if name == COMBINATION {
            let frozen: Vec<String> =
                serde_json::from_value(report["selection"]["frozen"].clone())?;
            if frozen.is_empty() {
                return Err(
                    "the lab froze no Candidate Signal: the combination is the anchor".into(),
                );
            }
            frozen
        } else {
            vec![name.to_owned()]
        };
        let mut members = Vec::new();
        for key in &names {
            let candidate = report["candidates"]
                .as_array()
                .and_then(|cs| cs.iter().find(|c| c["name"] == key.as_str()))
                .ok_or_else(|| format!("{key} was not evaluated by this lab run"))?;
            if candidate["verdict"] != "promote" {
                return Err(format!(
                    "{key} was not promoted: {} ({})",
                    candidate["verdict"].as_str().unwrap_or("?"),
                    candidate["reason"].as_str().unwrap_or("")
                )
                .into());
            }
            if !LIVE.contains(&key.as_str()) {
                return Err(format!(
                    "{key} cannot run live: it reads the archive's Match history, which the live                      sidecar has no port for"
                )
                .into());
            }
            let signal = signals::by_name(key).ok_or_else(|| format!("{key} is not registered"))?;
            let parameters = report["deployment"][key.as_str()].clone();
            if parameters.is_null() {
                return Err(format!("the report has no fitted parameters for {key}").into());
            }
            members.push((signal, decode(key, parameters)?));
        }
        let combination = if name == COMBINATION {
            let combiner: Combiner =
                serde_json::from_value(report["deployment"][COMBINATION].clone())?;
            if combiner.extras.len()
                != members
                    .iter()
                    .map(|(s, _)| s.feature_names().len())
                    .sum::<usize>()
            {
                return Err("the combination's weights do not match its members".into());
            }
            Some(combiner)
        } else {
            None
        };
        Ok(Self {
            label: names.join(" + "),
            members,
            combination,
            identity: format!("{name}/{}", crate::digest(&bytes)),
        })
    }

    /// Team A's probability for the Match, computed as the lab computes it;
    /// the anchor when the signal has no value for it.
    pub fn live(&self, view: &MatchView) -> Result<Live> {
        let value = match &self.combination {
            Some(combiner) => {
                let mut inputs = Inputs::base(view.market()?, view.signal()?.map(|s| s.series));
                for (signal, fit) in &self.members {
                    inputs.extras.extend(lab::selection::features_of(
                        signal.as_ref(),
                        fit.as_ref(),
                        view,
                    )?);
                }
                if inputs.extras.iter().all(Option::is_some) {
                    combiner.predict(&inputs)
                } else {
                    None
                }
            }
            None => match self.members[0].1.predict(view)? {
                Some(p) => Some(*p.at(view.cutoff)?),
                None => None,
            },
        };
        Ok(match value {
            Some(p) => Live {
                probability: Some(p),
                available: true,
            },
            None => Live {
                probability: view.anchor()?,
                available: false,
            },
        })
    }

    /// The Prediction and its Rationale, which names the criterion in the three
    /// interface languages (or says its data was missing and the anchor decided).
    pub fn prediction(
        &self,
        view: &MatchView,
        live: Live,
        tier: Tier,
    ) -> Result<Option<Prediction>> {
        let Some(p) = live.probability else {
            return Ok(None);
        };
        let label = &self.label;
        let summary = if live.available {
            json!({
                "en": format!("Signal Lab criterion: {label}."),
                "ru": format!("Критерий лаборатории сигналов: {label}."),
                "uk": format!("Критерій лабораторії сигналів: {label}."),
            })
        } else {
            json!({
                "en": format!("No live data for the Signal Lab criterion {label}: the anchor decided."),
                "ru": format!("Нет данных для критерия лаборатории сигналов {label}: решила базовая оценка."),
                "uk": format!("Немає даних для критерію лабораторії сигналів {label}: вирішила базова оцінка."),
            })
        };
        let a = p >= 0.5;
        Ok(Some(Prediction {
            team_id: if a {
                view.game.team_a.id
            } else {
                view.game.team_b.id
            },
            rationale: json!({
                "summary": summary,
                "p_win": if a { p } else { 1.0 - p },
                "override_market": view.market()?.is_some_and(|m| (m >= 0.5) != a),
                "preliminary": tier == Tier::Draft,
                "factors": [],
            }),
        }))
    }
}
