use super::{ContextSignal, Features, MatchView, team_features, win_rate};
use crate::{Format, HistoricalMatch, Result};

pub(super) const SIGNAL: ContextSignal = ContextSignal {
    name: "series_format",
    description: "Team differences in Bo1 versus Bo3 performance, conditioned on the current format",
    needs: "both Bo1 and Bo3 results for each team, a Bo1 or Bo3 Match, and 20 anchored training Matches",
    columns: &["bo1_relative_form", "bo3_relative_form"],
    read: features,
};

pub fn format_rates(team: &str, history: &[&HistoricalMatch]) -> Option<(f64, f64)> {
    let rate = |format| win_rate(history.iter().filter(|m| m.format == format).map(|m| m.won_by(team)));
    rate(Format::Bo1).zip(rate(Format::Bo3))
}

fn features(view: &MatchView) -> Result<Option<Features>> {
    if view.game.format == Format::Bo5 { return Ok(None); }
    Ok(team_features(view, format_rates)?.map(|((a1, a3), (b1, b3), at)| {
        let shift = (a1 - a3) - (b1 - b3);
        let columns = match view.game.format {
            Format::Bo1 => vec![shift, 0.0],
            Format::Bo3 => vec![0.0, -shift],
            Format::Bo5 => unreachable!("Bo5 has no feature"),
        };
        (columns, at)
    }))
}
