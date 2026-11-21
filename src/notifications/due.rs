//! What is due now: reminders, Reveal announcements and daily digests, as
//! plain data. Pure: reads the League's `LeagueMatches`, touches nothing.

use std::collections::BTreeMap;

use chrono::{DateTime, Duration, NaiveDate, NaiveTime, Utc};

use super::messages;
use crate::api::leaderboard::ranking;
use crate::domain::scoring::{self, ScoredMatch};
use crate::domain::{MatchStatus, UserId};
use crate::league_matches::LeagueMatches;
use crate::ports::NotificationKind;
use crate::store::leagues::League;

/// The reminder goes out this long before Lock.
pub const REMINDER_LEAD: Duration = Duration::minutes(30);
/// A Reveal older than this is not announced: it happened before the feature
/// (or the poller) was running, or while the bot was down for hours.
pub const REVEAL_ANNOUNCE_WINDOW: Duration = Duration::hours(3);
/// A match day's digest is sent until this long after the day ended (UTC).
pub const DIGEST_GRACE: Duration = Duration::hours(6);
/// An upcoming Match whose Scheduled Start passed this long ago no longer
/// holds back its match day's digest (stale data, not a pending Match).
pub const STALE_AFTER_START: Duration = Duration::hours(12);

/// One notification to one User, not yet rendered (the language is theirs).
#[derive(Debug, Clone)]
pub struct Due {
    /// Dedupe key in the notification log.
    pub key: String,
    pub user: UserId,
    pub kind: NotificationKind,
    /// A key of the message catalog.
    pub message: &'static str,
    pub values: Vec<(&'static str, String)>,
}

pub fn due(league: &League, matches: &LeagueMatches, now: DateTime<Utc>) -> Vec<Due> {
    let mut items = Vec::new();
    reminders(league, matches, now, &mut items);
    reveals(league, matches, now, &mut items);
    digests(league, matches, now, &mut items);
    items
}

/// When a Match began or begins: the earlier of its actual start and its
/// current Scheduled Start. A postponed Match thus moves to its new day, an
/// early start counts at the early time, and a Match the poller saw late
/// (the poller stamps "now" as its actual start) stays on its scheduled day.
fn start_of(game: &crate::domain::Match) -> Option<DateTime<Utc>> {
    match (game.actual_start, game.scheduled_start) {
        (Some(actual), Some(scheduled)) => Some(actual.min(scheduled)),
        (actual, scheduled) => actual.or(scheduled),
    }
}

/// The match day a Match belongs to: the UTC date of [`start_of`]. Events
/// span time zones and KallBet knows no User's zone, so one fixed zone keeps
/// the day stable and the rule explainable.
fn match_day(game: &crate::domain::Match) -> Option<NaiveDate> {
    start_of(game).map(|start| start.date_naive())
}

/// The digest of a match day: Points gained over the day's Matches and the
/// current place, once every Match of that day is done (finished or
/// cancelled; a Match whose start passed over [`STALE_AFTER_START`] ago
/// without a result is dead data and does not hold the day back). Days with
/// no scored Match, and days that ended more than [`DIGEST_GRACE`] ago, get
/// none. Sent to the Members who joined before the day ended.
fn digests(league: &League, matches: &LeagueMatches, now: DateTime<Utc>, out: &mut Vec<Due>) {
    let mut days: BTreeMap<NaiveDate, Vec<&crate::domain::Match>> = BTreeMap::new();
    for m in &matches.view.matches {
        if let Some(day) = match_day(&m.data) {
            days.entry(day).or_default().push(&m.data);
        }
    }
    let member_ids: Vec<UserId> = matches.profiles.iter().map(|p| p.user_id).collect();
    let total = ranking(matches, &member_ids);

    for (day, games) in days {
        let day_end = day
            .succ_opt()
            .unwrap_or(day)
            .and_time(NaiveTime::MIN)
            .and_utc();
        if now >= day_end + DIGEST_GRACE {
            continue;
        }
        let done = |g: &&crate::domain::Match| match g.status {
            MatchStatus::Finished | MatchStatus::Cancelled => true,
            MatchStatus::Upcoming | MatchStatus::Live => {
                start_of(g).is_some_and(|start| start + STALE_AFTER_START <= now)
            }
        };
        if !games.iter().all(done) {
            continue;
        }
        let scored: Vec<ScoredMatch> = games
            .iter()
            .filter(|g| g.status == MatchStatus::Finished && !g.is_void())
            .map(|&game| ScoredMatch {
                game,
                playoffs: false,
            })
            .collect();
        if scored.is_empty() {
            continue;
        }
        let gained = scoring::leaderboard(&member_ids, &scored, |user, game| {
            matches.prediction_of(game, user).map(|p| p.team_id)
        });
        for profile in matches.profiles.iter().filter(|p| p.joined_at < day_end) {
            let user = profile.user_id;
            let (Some(today), Some(row)) = (
                gained.iter().find(|s| s.user_id == user),
                total.iter().find(|s| s.user_id == user),
            ) else {
                continue;
            };
            out.push(Due {
                key: format!("digest:{}:{}:{}", league.id, day, user),
                user,
                kind: NotificationKind::DailyDigest,
                message: messages::DIGEST,
                values: vec![
                    ("league", league.name.clone()),
                    ("date", day.to_string()),
                    ("gained", today.points.to_string()),
                    ("place", row.place.to_string()),
                    ("total", row.points.to_string()),
                ],
            });
        }
    }
}

/// Every Match whose Predictions were revealed within the last
/// [`REVEAL_ANNOUNCE_WINDOW`], to each Eligible Member (those who joined
/// after the Lock, or left since, are not in `outcome.eligible`). The moment
/// is the stored `revealed_at`, because the Prediction endpoint may have
/// stored an early Reveal before any poller pass saw it. Skipped: Void
/// Matches, and Matches where no Eligible Member predicted (nothing to show).
fn reveals(league: &League, matches: &LeagueMatches, now: DateTime<Utc>, out: &mut Vec<Due>) {
    for m in &matches.view.matches {
        let game = &m.data;
        let Some(outcome) = matches.outcome(game.id) else {
            continue;
        };
        let Some(revealed_at) = outcome.stamps.revealed_at else {
            continue;
        };
        if now - revealed_at > REVEAL_ANNOUNCE_WINDOW || game.is_void() {
            continue;
        }
        let picks: Vec<_> = outcome
            .eligible
            .iter()
            .filter_map(|&user| matches.prediction_of(game.id, user))
            .collect();
        if picks.is_empty() {
            continue;
        }
        let count = |team| picks.iter().filter(|p| Some(p.team_id) == team).count();
        let (team_a, team_b) = match_title(matches, game);
        for &user in &outcome.eligible {
            out.push(Due {
                key: format!("reveal:{}:{}:{}", league.id, game.id, user),
                user,
                kind: NotificationKind::Reveal,
                message: messages::REVEAL,
                values: vec![
                    ("league", league.name.clone()),
                    ("match", format!("{team_a} vs {team_b}")),
                    ("team_a", team_a.clone()),
                    ("team_b", team_b.clone()),
                    ("count_a", count(game.team_a).to_string()),
                    ("count_b", count(game.team_b).to_string()),
                ],
            });
        }
    }
}

fn match_title(matches: &LeagueMatches, game: &crate::domain::Match) -> (String, String) {
    let name = |team: Option<crate::domain::TeamId>| {
        team.and_then(|id| matches.view.team(id))
            .map_or_else(|| "TBD".to_string(), |t| t.name.clone())
    };
    (name(game.team_a), name(game.team_b))
}

/// Eligible Members without a Prediction, from 30 minutes before the Match's
/// Lock until it Locks. A Match first seen inside that window still gets its
/// reminder (with the real minutes left): a late reminder beats none while
/// the Match is open. Keyed without the Lock time, so a postponed Match does
/// not remind twice.
fn reminders(league: &League, matches: &LeagueMatches, now: DateTime<Utc>, out: &mut Vec<Due>) {
    for m in &matches.view.matches {
        let game = &m.data;
        let Some(outcome) = matches.outcome(game.id) else {
            continue;
        };
        let (Some(_), Some(_), Some(lock_at)) = (game.team_a, game.team_b, outcome.lock_time)
        else {
            continue;
        };
        if outcome.is_locked()
            || game.status != MatchStatus::Upcoming
            || now < lock_at - REMINDER_LEAD
        {
            continue;
        }
        let (team_a, team_b) = match_title(matches, game);
        let minutes = ((lock_at - now).num_seconds() + 59) / 60;
        for &user in &outcome.eligible {
            if matches.prediction_of(game.id, user).is_some() {
                continue;
            }
            out.push(Due {
                key: format!("reminder:{}:{}:{}", league.id, game.id, user),
                user,
                kind: NotificationKind::Reminder,
                message: messages::REMINDER,
                values: vec![
                    ("league", league.name.clone()),
                    ("match", format!("{team_a} vs {team_b}")),
                    ("minutes", minutes.max(1).to_string()),
                ],
            });
        }
    }
}
