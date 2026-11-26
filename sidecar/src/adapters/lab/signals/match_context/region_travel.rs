use super::{ContextSignal, Features, MatchView};
use crate::Result;
use crate::adapters::backtest::match_context::Environment;

pub(super) const SIGNAL: ContextSignal = ContextSignal {
    name: "region_travel",
    description: "Difference in playing a LAN outside the team's published VRS qualifying region",
    needs: "a dated LAN host region and dated VRS qualifying regions for both teams, plus 20 anchored training Matches",
    columns: &["away_region"],
    read: features,
};

fn features(view: &MatchView) -> Result<Option<Features>> {
    let context = &view.context;
    let cutoff = view.cutoff;
    let (Some(environment), Some(host), Some(a), Some(b)) = (
        context.environment(cutoff),
        context.region(cutoff),
        context.team_region(&view.game.team_a.name, cutoff),
        context.team_region(&view.game.team_b.name, cutoff),
    ) else { return Ok(None) };
    if environment.value != Environment::Lan { return Ok(None); }
    let away = |team| f64::from(u8::from(team != host.value));
    let at = host.observed_at.max(environment.observed_at).max(a.observed_at).max(b.observed_at);
    Ok(Some((vec![away(a.value) - away(b.value)], at)))
}
