use super::{ContextSignal, Features, MatchView, history_at, win_rate};
use crate::{HistoricalMatch, Result, team_names::team_key};

pub(super) const SIGNAL: ContextSignal = ContextSignal {
    name: "lan_online",
    description: "Team win rates in the current environment, with separate weights for LAN and online",
    needs: "dated Event types, same-environment results for both teams, and 20 anchored training Matches",
    columns: &["lan_win_rate", "online_win_rate"],
    read: features,
};

fn features(view: &MatchView) -> Result<Option<Features>> {
    use crate::adapters::backtest::match_context::Environment;
    let Some(environment) = view.context.environment(view.cutoff) else { return Ok(None) };
    let mut at = Some(environment.observed_at).max(history_at(view)).unwrap();
    let mut same_environment_rate = |team: &str, history: Vec<&HistoricalMatch>| {
        let team = team_key(team);
        win_rate(history.into_iter().filter_map(|m| {
            let source = view.context.history_environment(m.stage_id, m.event_id, view.cutoff)?;
            at = at.max(source.observed_at);
            (source.value == environment.value).then(|| m.won_by(&team))
        }))
    };
    let a = same_environment_rate(&view.game.team_a.name, view.team_a_history()?);
    let b = same_environment_rate(&view.game.team_b.name, view.team_b_history()?);
    Ok(a.zip(b).map(|(a, b)| {
        let columns = match environment.value {
            Environment::Lan => vec![a - b, 0.0],
            Environment::Online => vec![0.0, a - b],
        };
        (columns, at)
    }))
}
