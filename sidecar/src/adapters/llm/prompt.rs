//! The text every provider receives: seat, instructions, tool rule, schema,
//! dossier and attachments. The schema is also enforced by each provider's own
//! structured-output switch and checked again in Rust.
use super::{ToolAccess, WEB_DOMAINS};
use crate::*;

pub fn render(dossier: &Dossier, tier: Tier, role: &RolePrompt, access: ToolAccess) -> String {
    let tools = match access {
        ToolAccess::None => {
            "Use no tools and no web access. Work only from the dossier below.".to_owned()
        }
        ToolAccess::WhitelistedWeb => format!(
            "You may read web pages on {} only, to check late roster, stand-in, travel or illness news \
             published before the match. Use no other tools or sites.",
            WEB_DOMAINS.join(" and ")
        ),
    };
    let mut text = format!(
        "You are the `{role}` seat of a CS2 match forecaster (run tier: {tier}).\n{instructions}\n{tools}\n\n\
         Answer with one JSON value that matches this JSON Schema, and nothing else:\n{schema}\n\n\
         Match dossier (JSON):\n{dossier}\n",
        role = role.role,
        tier = tier.name(),
        instructions = role.instructions,
        schema = role.schema,
        dossier = serde_json::to_string(dossier).unwrap_or_default(),
    );
    for (label, value) in &role.attachments {
        text.push_str(&format!("\n{label} (JSON):\n{value}\n"));
    }
    text
}
