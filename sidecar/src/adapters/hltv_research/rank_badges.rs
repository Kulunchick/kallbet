//! The rank badges of an HLTV match page (ticket #57): under each team's name,
//! `ranking-badge hltv-ranking` (HLTV world ranking) and `ranking-badge
//! vrs-ranking` (Valve Regional Standings), each linking to the dated ranking
//! it shows (`/ranking/teams/2026/june/8/<team>`, `/valve-ranking/teams/2026/june/11?teamId=<team>`).
//!
//! The leakage audit (docs/signal-lab/criteria-research.md, "Rank badge audit")
//! found the HLTV badge to be the weekly ranking published on or before the
//! Match day, and the VRS badge to be dated by the Match day itself and to
//! already move with that Match's own result. Only the HLTV badge may be read
//! before a Match (`backtest::rank_history`); the VRS badge is parsed for the
//! audit and never used.
use chrono::NaiveDate;

/// One badge: the rank shown and the date of the ranking it links to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RankBadge {
    pub rank: u32,
    pub date: NaiveDate,
}

/// One team's badges, with the HLTV team id the badge links carry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TeamBadges {
    pub team_id: i64,
    pub hltv: Option<RankBadge>,
    pub vrs: Option<RankBadge>,
}

/// The teams' badges in page order (team A, then team B); empty when the page
/// has none (a walkover page, or an unranked pair). A team listed twice keeps
/// its first badges (the match header comes before any lineup block).
pub fn parse_rank_badges(html: &str) -> Vec<TeamBadges> {
    let mut teams: Vec<TeamBadges> = vec![];
    let mut rest = html;
    while let Some(at) = rest.find("class=\"ranking-badge ") {
        // The anchor's href precedes its class: `<a href="..." class="ranking-badge ...">`.
        let start = rest[..at].rfind("<a ").unwrap_or(at);
        let end = rest[at..].find("</a>").map_or(rest.len(), |e| at + e);
        if let Some((team_id, kind, badge)) = parse_anchor(&rest[start..end]) {
            let team = match teams.iter().position(|t| t.team_id == team_id) {
                Some(i) => &mut teams[i],
                None => {
                    teams.push(TeamBadges { team_id, hltv: None, vrs: None });
                    teams.last_mut().unwrap()
                }
            };
            let slot = match kind {
                Kind::Hltv => &mut team.hltv,
                Kind::Vrs => &mut team.vrs,
            };
            slot.get_or_insert(badge);
        }
        rest = &rest[end.max(at + 1)..];
    }
    teams.truncate(2);
    teams
}

enum Kind {
    Hltv,
    Vrs,
}

fn parse_anchor(anchor: &str) -> Option<(i64, Kind, RankBadge)> {
    let href = anchor.split("href=\"").nth(1)?.split('"').next()?;
    let (kind, path) = if let Some(p) = href.strip_prefix("/ranking/teams/") {
        (Kind::Hltv, p)
    } else if let Some(p) = href.strip_prefix("/valve-ranking/teams/") {
        (Kind::Vrs, p)
    } else {
        return None;
    };
    let (path, query) = path.split_once('?').unwrap_or((path, ""));
    let mut parts = path.split('/');
    let year: i32 = parts.next()?.parse().ok()?;
    let month = month(parts.next()?)?;
    let day: u32 = parts.next()?.parse().ok()?;
    let team_id: i64 = match kind {
        Kind::Hltv => parts.next()?.parse().ok()?,
        Kind::Vrs => query
            .split('&')
            .find_map(|kv| kv.strip_prefix("teamId="))?
            .parse()
            .ok()?,
    };
    let text = anchor.rsplit("</span>").next()?;
    let rank: u32 = text.trim().strip_prefix('#')?.trim().parse().ok()?;
    let date = NaiveDate::from_ymd_opt(year, month, day)?;
    Some((team_id, kind, RankBadge { rank, date }))
}

fn month(name: &str) -> Option<u32> {
    const MONTHS: [&str; 12] = [
        "january", "february", "march", "april", "may", "june", "july", "august", "september",
        "october", "november", "december",
    ];
    MONTHS.iter().position(|m| *m == name).map(|i| i as u32 + 1)
}
