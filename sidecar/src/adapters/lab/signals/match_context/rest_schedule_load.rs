use super::{ContextSignal, Features, MatchView, team_features};
use crate::{HistoricalMatch, Instant, Result};
use chrono::Duration;

pub(super) const SIGNAL: ContextSignal = ContextSignal {
    name: "rest_schedule_load",
    description: "Team differences in rest, Matches and maps in the preceding 48 hours, and a second Match on the same UTC day",
    needs: "prior results and known map counts for both teams within the load window, plus 20 anchored training Matches",
    columns: &["log_rest_hours", "matches_last_48h", "maps_last_48h", "second_match_today"],
    read: features,
};

#[derive(Debug, PartialEq)]
pub struct Load {
    pub rest_hours: f64,
    pub matches: u32,
    pub maps: u32,
    pub second_match: bool,
}

pub fn team_load(
    history: &[&HistoricalMatch],
    cutoff: Instant,
    scheduled_start: Instant,
) -> Option<Load> {
    let previous = history.iter().rev().copied().filter(|m| m.finished_at < cutoff);
    let last = previous.clone().next()?;
    let rest_hours = (scheduled_start - last.finished_at).num_seconds() as f64 / 3_600.0;
    let mut load = Load { rest_hours, matches: 0, maps: 0, second_match: false };
    for m in previous.take_while(|m| m.finished_at >= cutoff - Duration::days(2)) {
        load.matches += 1;
        load.maps += m.maps_played()?;
        load.second_match |= m.actual_start.date_naive() == scheduled_start.date_naive();
    }
    Some(load)
}

fn features(view: &MatchView) -> Result<Option<Features>> {
    let Some(start) = view.game.scheduled_start else { return Ok(None) };
    Ok(team_features(view, |_, history| team_load(history, view.cutoff, start))?
        .map(|(a, b, at)| (vec![
            a.rest_hours.ln_1p() - b.rest_hours.ln_1p(),
            f64::from(a.matches) - f64::from(b.matches),
            (f64::from(a.maps) - f64::from(b.maps)) / 3.0,
            f64::from(u8::from(a.second_match)) - f64::from(u8::from(b.second_match)),
        ], at)))
}
