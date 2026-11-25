//! Which pass a run gets: one fast agent or the three-seat debate. Pure: the
//! decision reads only the dossier, the tier and the bot's debate settings.
use crate::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;

/// Something in the dossier's context that may make the anchor wrong.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskFlag {
    /// A team fields a stand-in, or its coach plays (Liquipedia roster).
    StandIn,
    /// The Event roster differs from the team's latest VRS roster.
    RosterChange,
    /// A headline about one of the teams speaks of a roster, travel or health matter.
    News,
}

/// Words that make a team's headline a risk flag. Results and interviews, which
/// are most of a Major's news, carry none of them.
const NEWS_WORDS: [&str; 19] = [
    "stand-in",
    "stand in",
    "standin",
    "substitut",
    "replac",
    "bench",
    "roster",
    "lineup",
    "line-up",
    "coach",
    "visa",
    "travel",
    "illness",
    "sick",
    "hospital",
    "emergency",
    "withdraw",
    "forfeit",
    "disqualif",
];

/// The risk flags of a dossier, read from the context facts of ticket #26
/// (`ContextFact`). A fact of another shape raises none.
pub fn risk_flags(dossier: &Dossier) -> BTreeSet<RiskFlag> {
    let mut flags = BTreeSet::new();
    for fact in &dossier.context {
        match fact["kind"].as_str() {
            Some("liquipedia_roster") => {
                let coach_plays = fact["coaches"]
                    .as_array()
                    .is_some_and(|c| c.iter().any(|c| c["emergency_sub"] == Value::Bool(true)));
                let stand_ins = fact["stand_ins"].as_array().is_some_and(|s| !s.is_empty());
                if fact["has_stand_in"] == Value::Bool(true) || stand_ins || coach_plays {
                    flags.insert(RiskFlag::StandIn);
                }
            }
            Some("vrs_roster_diff") if fact["changed"] == Value::Bool(true) => {
                flags.insert(RiskFlag::RosterChange);
            }
            Some("news_item") => {
                let text = format!(
                    "{} {}",
                    fact["title"].as_str().unwrap_or_default(),
                    fact["description"].as_str().unwrap_or_default()
                )
                .to_lowercase();
                if NEWS_WORDS.iter().any(|word| text.contains(word)) {
                    flags.insert(RiskFlag::News);
                }
            }
            _ => {}
        }
    }
    flags
}

/// When a bot debates. `tiered` is the default.
#[derive(Debug, Clone, Copy, PartialEq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DebateMode {
    /// Fast on Draft runs and on clean mismatches, debate otherwise.
    #[default]
    Tiered,
    /// Every run is a debate, Draft runs too.
    Always,
    /// Every run is one fast agent.
    Never,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Pass {
    /// One agent: the bot's `fast` chain.
    Fast,
    /// Market Advocate and Devil's Advocate, then the Chief Arbiter.
    Debate,
}

/// The pass for a run. In `tiered` mode a Draft run is fast; a PreLock run (the
/// window before Hard Lock) is a debate unless the Match is a clean mismatch:
/// `p_base` at or beyond `lopsided` on either side, no risk flag, not a Bo1.
pub fn plan(
    mode: DebateMode,
    tier: Tier,
    p_base: f64,
    format: Format,
    flags: &BTreeSet<RiskFlag>,
    lopsided: f64,
) -> Pass {
    match (mode, tier) {
        (DebateMode::Always, _) => Pass::Debate,
        (DebateMode::Never, _) | (DebateMode::Tiered, Tier::Draft) => Pass::Fast,
        (DebateMode::Tiered, Tier::PreLock) => {
            // The same distance from even on either side; the margin keeps a
            // `p_base` of exactly 0.20 on the mismatch side of `1 - 0.80`.
            let mismatch = (p_base - 0.5).abs() + 1e-9 >= lopsided - 0.5;
            if mismatch && flags.is_empty() && !matches!(format, Format::Bo1) {
                Pass::Fast
            } else {
                Pass::Debate
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use DebateMode::*;
    use Pass::*;

    fn none() -> BTreeSet<RiskFlag> {
        BTreeSet::new()
    }
    fn flag(f: RiskFlag) -> BTreeSet<RiskFlag> {
        [f].into()
    }

    #[test]
    fn a_draft_run_is_one_agent_unless_the_bot_always_debates() {
        for p in [0.52, 0.85] {
            let stand_in = flag(RiskFlag::StandIn);
            assert_eq!(
                plan(Tiered, Tier::Draft, p, Format::Bo1, &stand_in, 0.8),
                Fast
            );
            assert_eq!(
                plan(Always, Tier::Draft, p, Format::Bo3, &none(), 0.8),
                Debate
            );
            assert_eq!(
                plan(Never, Tier::PreLock, p, Format::Bo1, &stand_in, 0.8),
                Fast
            );
        }
    }

    #[test]
    fn before_hard_lock_only_a_clean_mismatch_stays_with_one_agent() {
        let pre = |p, format, flags: &BTreeSet<RiskFlag>| {
            plan(Tiered, Tier::PreLock, p, format, flags, 0.8)
        };
        // Competitive Matches debate.
        assert_eq!(pre(0.52, Format::Bo3, &none()), Debate);
        assert_eq!(pre(0.79, Format::Bo3, &none()), Debate);
        assert_eq!(pre(0.21, Format::Bo5, &none()), Debate);
        // A mismatch on either side, with nothing flagged, does not.
        assert_eq!(pre(0.80, Format::Bo3, &none()), Fast);
        assert_eq!(pre(0.20, Format::Bo3, &none()), Fast);
        assert_eq!(pre(0.93, Format::Bo5, &none()), Fast);
        // A Bo1 or any flag upgrades even a mismatch.
        assert_eq!(pre(0.93, Format::Bo1, &none()), Debate);
        for f in [RiskFlag::StandIn, RiskFlag::RosterChange, RiskFlag::News] {
            assert_eq!(pre(0.93, Format::Bo3, &flag(f)), Debate);
        }
        // The threshold is the bot's own.
        assert_eq!(
            plan(Tiered, Tier::PreLock, 0.72, Format::Bo3, &none(), 0.7),
            Fast
        );
    }
}
