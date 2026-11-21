//! Leagues and membership: ids, roles and the rules that need no I/O.

use serde::{Deserialize, Serialize};

use super::EventId;
use super::ids::id;

id!(
    /// Our id of a League.
    LeagueId(i64)
);

/// A Member's role. A League Admin is the creator or an appointed co-admin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Member,
    Admin,
}

impl Role {
    pub fn code(self) -> &'static str {
        match self {
            Role::Member => "member",
            Role::Admin => "admin",
        }
    }

    pub fn from_code(code: &str) -> Option<Role> {
        match code {
            "member" => Some(Role::Member),
            "admin" => Some(Role::Admin),
            _ => None,
        }
    }

    pub fn is_admin(self) -> bool {
        self == Role::Admin
    }
}

pub const LEAGUE_NAME_MAX_CHARS: usize = 60;

/// Trims a League name; `None` when it is empty or too long.
pub fn clean_league_name(raw: &str) -> Option<String> {
    let name = raw.trim();
    let chars = name.chars().count();
    (1..=LEAGUE_NAME_MAX_CHARS)
        .contains(&chars)
        .then(|| name.to_string())
}

/// The Event id in an HLTV event link such as
/// `https://www.hltv.org/events/7148/iem-cologne-major-2026`. `None` when the
/// text is not an HLTV event link.
pub fn parse_hltv_event_link(link: &str) -> Option<EventId> {
    let link = link.trim();
    let rest = link
        .strip_prefix("https://")
        .or_else(|| link.strip_prefix("http://"))?;
    let (host, path) = rest.split_once('/')?;
    if !matches!(host, "www.hltv.org" | "hltv.org") {
        return None;
    }
    let path = path.split(['?', '#']).next()?;
    let mut parts = path.split('/');
    if parts.next()? != "events" {
        return None;
    }
    let id: i64 = parts.next()?.parse().ok()?;
    (id > 0).then_some(EventId(id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_event_links() {
        for link in [
            "https://www.hltv.org/events/7148/iem-cologne-major-2026",
            "http://hltv.org/events/7148",
            "  https://www.hltv.org/events/7148/x?tab=1#top ",
        ] {
            assert_eq!(parse_hltv_event_link(link), Some(EventId(7148)), "{link}");
        }
    }

    #[test]
    fn rejects_everything_else() {
        for link in [
            "",
            "hltv.org/events/7148/x",
            "https://www.hltv.org/events/0/x",
            "https://www.hltv.org/events/-3/x",
            "https://www.hltv.org/events/",
            "https://www.hltv.org/results",
            "https://notwww.hltv.org/events/7148/x",
        ] {
            assert_eq!(parse_hltv_event_link(link), None, "{link}");
        }
    }

    #[test]
    fn league_names_are_trimmed_and_bounded() {
        assert_eq!(clean_league_name("  Friends "), Some("Friends".into()));
        assert_eq!(clean_league_name(" "), None);
        assert_eq!(
            clean_league_name(&"я".repeat(60)).map(|n| n.chars().count()),
            Some(60)
        );
        assert_eq!(clean_league_name(&"я".repeat(61)), None);
    }
}
