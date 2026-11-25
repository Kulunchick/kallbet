//! What each seat of the debate is asked, and the shape of a brief. The schemas
//! stay inside the subset every provider accepts (`adapters::llm::schema`): each
//! object closed, every property required, no numeric bounds.
use super::policy::{FACTOR_KINDS, MAX_SHIFT_PP};
use crate::*;
use serde_json::{Value, json};

pub const ADVOCATE: &str = "advocate";
pub const DEVIL: &str = "devil";
pub const ARBITER: &str = "arbiter";
/// The single fast agent (the scheduler's own prompt).
pub const PREDICT: &str = "predict";

/// A brief of the Market Advocate or the Devil's Advocate.
pub fn brief_schema() -> Value {
    let mut kinds: Vec<&str> = FACTOR_KINDS.to_vec();
    kinds.push("other");
    json!({
        "type": "object",
        "properties": {
            "position": {"type": "string"},
            "claims": {"type": "array", "items": {
                "type": "object",
                "properties": {
                    "claim": {"type": "string"},
                    "kind": {"type": "string", "enum": kinds},
                    "evidence_url": {"type": "string"},
                    "favours": {"type": "string", "enum": ["team_a", "team_b"]}
                },
                "required": ["claim", "kind", "evidence_url", "favours"],
                "additionalProperties": false
            }},
            "delta": {"type": "number"}
        },
        "required": ["position", "claims", "delta"],
        "additionalProperties": false
    })
}

const DOSSIER: &str = "In the dossier, `market` is the de-vigged bookmaker probability that team_a \
    wins the series and `signal.series` the estimate from the Valve Regional Standings; either \
    may be missing. The attached baseline is their blend (80% market, 20% standings, in logit \
    space) for team_a, with the risk flags found in `context`.";

const BRIEF: &str = "Answer with a brief: `position` in 2-4 sentences; `claims`, each with the \
    side it favours, its `kind` (`other` when it fits none of the factor kinds) and the evidence \
    URL copied exactly from the dossier (an empty string when the dossier gives none); and \
    `delta`, the shift of the baseline in percentage points for team_a that you argue for \
    (positive favours team_a, 0 for none).";

fn seat(role: &str, task: &str) -> RolePrompt {
    RolePrompt {
        role: role.into(),
        instructions: format!(
            "{task} {DOSSIER} A shift is at most {MAX_SHIFT_PP} points either way. {BRIEF}"
        ),
        schema: brief_schema(),
        attachments: vec![],
    }
}

pub fn advocate() -> RolePrompt {
    seat(
        ADVOCATE,
        "You are the Market Advocate in a debate about a CS2 match forecast. Defend the baseline: \
         bookmakers already price public ratings, form and known roster news. Go through every \
         roster, stand-in, coach, travel or illness item in the dossier and say which are stale, \
         already priced in, or unsupported rumour. Concede only what the dossier verifies.",
    )
}

pub fn devil() -> RolePrompt {
    seat(
        DEVIL,
        "You are the Devil's Advocate in a debate about a CS2 match forecast. Make the strongest \
         grounded case that the baseline is wrong: a stand-in or a roster change in the last 30 \
         days, an IGL or coach change, visa or travel trouble, illness, a technical or \
         disqualification issue, best-of-one variance, LAN and travel factors. Use only what the \
         dossier supports; do not invent news.",
    )
}

/// The Chief Arbiter answers what `asked` asks (the scheduler's adjustment), with
/// the two briefs attached. A missing brief is attached as `null`.
pub fn arbiter(asked: &RolePrompt, advocate: Value, devil: Value) -> RolePrompt {
    let mut attachments = asked.attachments.clone();
    attachments.push(("Market Advocate brief".into(), advocate));
    attachments.push(("Devil's Advocate brief".into(), devil));
    RolePrompt {
        role: ARBITER.into(),
        instructions: format!(
            "You are the Chief Arbiter of a debate about a CS2 match forecast. Two briefs are \
             attached: the Market Advocate defends the baseline, the Devil's Advocate attacks it \
             (a brief is null when its seat was unavailable). {DOSSIER} Weigh both briefs against \
             the numbers. Discard every claim that has no evidence URL from the dossier or whose \
             kind is not one of the factor kinds; a claim both seats accept weighs more than one \
             only its author makes. Then give the final answer. {}",
            asked.instructions
        ),
        schema: asked.schema.clone(),
        attachments,
    }
}
