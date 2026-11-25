//! A Prediction's Rationale (spec #21, ticket #23): the optional structured
//! explanation a Member, in practice a Bot User, attaches to a Prediction.
//!
//! It is stored with the Prediction and shown only with it: in
//! `api::matches::predictions_view`, once revealed to that viewer. It names
//! no team: `p_win` is the win probability of the picked team, so a team
//! re-key or a teams Override never makes it point at the wrong side.

use serde::{Deserialize, Serialize};

/// Longest summary per language, in characters (2-4 sentences fit easily).
pub const SUMMARY_MAX: usize = 1000;
/// Most adjustment factors in one Rationale.
pub const FACTORS_MAX: usize = 8;
/// Longest evidence URL, in bytes.
pub const EVIDENCE_URL_MAX: usize = 512;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rationale {
    /// Estimated win probability of the picked team, `0.0..=1.0`.
    pub p_win: f64,
    /// The pick goes against the market favourite.
    #[serde(default)]
    pub override_market: bool,
    /// The author's own draft flag: an early analysis it means to refine.
    /// (Not the same as a revealed Prediction's `preliminary`, which the
    /// server sets while a Bot User may still change its pick.)
    #[serde(default)]
    pub preliminary: bool,
    pub summary: Summary,
    #[serde(default)]
    pub factors: Vec<Factor>,
}

/// The 2-4 sentence summary in one or more of the interface languages. The
/// SPA shows the viewer's language and falls back to another.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Summary {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub en: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ru: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uk: Option<String>,
}

impl Summary {
    fn texts(&self) -> impl Iterator<Item = &String> {
        [&self.en, &self.ru, &self.uk].into_iter().flatten()
    }
}

/// A verified adjustment factor and where it is reported.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Factor {
    pub kind: FactorKind,
    pub evidence_url: String,
}

/// The closed vocabulary of adjustment factors (spec #21, Decision Policy).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FactorKind {
    StandIn,
    #[serde(rename = "roster_change_30d")]
    RosterChange30d,
    IglOrCoachChange,
    VisaOrTravel,
    IllnessOrEmergency,
    TechnicalOrDq,
}

/// Why a Rationale was refused. The API answers every case with
/// 422 `rationale_invalid`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidRationale(pub String);

impl Rationale {
    /// Reads and validates a Rationale from the request body's `rationale`
    /// value. `null` is no Rationale.
    pub fn parse(value: serde_json::Value) -> Result<Option<Rationale>, InvalidRationale> {
        if value.is_null() {
            return Ok(None);
        }
        let rationale: Rationale =
            serde_json::from_value(value).map_err(|e| InvalidRationale(e.to_string()))?;
        rationale.validate()?;
        Ok(Some(rationale))
    }

    pub fn validate(&self) -> Result<(), InvalidRationale> {
        let invalid = |why: &str| Err(InvalidRationale(why.into()));
        if !(0.0..=1.0).contains(&self.p_win) {
            return invalid("p_win out of 0..=1");
        }
        if self.summary.texts().next().is_none() {
            return invalid("summary has no language");
        }
        for text in self.summary.texts() {
            if text.trim().is_empty() || text.chars().count() > SUMMARY_MAX {
                return invalid("summary blank or too long");
            }
        }
        if self.factors.len() > FACTORS_MAX {
            return invalid("too many factors");
        }
        if !self
            .factors
            .iter()
            .all(|f| is_evidence_url(&f.evidence_url))
        {
            return invalid("evidence_url is not an http(s) URL");
        }
        Ok(())
    }
}

/// An absolute http(s) URL with a host part, of sane length, without
/// whitespace: the SPA renders it as an external link, so nothing else
/// (`javascript:`, `data:`) may pass.
fn is_evidence_url(url: &str) -> bool {
    url.len() <= EVIDENCE_URL_MAX
        && ["https://", "http://"].iter().any(|scheme| {
            url.strip_prefix(scheme)
                .is_some_and(|rest| !rest.is_empty() && !rest.starts_with('/'))
        })
        && !url.chars().any(|c| c.is_whitespace() || c.is_control())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evidence_urls_are_absolute_http_urls() {
        assert!(is_evidence_url("https://www.hltv.org/news/1"));
        assert!(is_evidence_url("http://liquipedia.net"));
        assert!(!is_evidence_url("https://"));
        assert!(!is_evidence_url("https:///path"));
        assert!(!is_evidence_url("javascript:alert(1)"));
        assert!(!is_evidence_url("HTTPS://hltv.org"));
    }
}
