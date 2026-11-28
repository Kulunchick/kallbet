use super::{ContextSignal, Features, MatchView, team_features};
use crate::adapters::{backtest::{StageKind, match_context::BracketRound}, lab::combiner::logit};
use crate::{HistoricalMatch, Instant, Result};

pub(super) const SIGNAL: ContextSignal = ContextSignal {
    name: "stakes",
    description: "Pre-round Swiss records, advancement and elimination Matches, and Playoffs rounds",
    needs: "known Swiss round and prior-round results, or a named Playoffs round, plus 20 anchored training Matches",
    columns: &["wins", "losses", "advancement", "elimination", "round_of_16", "quarterfinal", "semifinal", "final"],
    read: features,
};

pub fn swiss_record(
    team: &str,
    history: &[&HistoricalMatch],
    stage_id: i64,
    round: u32,
    cutoff: Instant,
) -> Option<(u32, u32)> {
    let mut prior = [None; 4];
    let played = usize::try_from(round.checked_sub(1)?).ok()?;
    let needed = prior.get_mut(..played)?;
    for m in history {
        if m.stage_id != stage_id || m.finished_at >= cutoff || !m.played(team) {
            continue;
        }
        if let Some(slot) = m.round.and_then(|n| n.checked_sub(1)).and_then(|n| needed.get_mut(n as usize))
            && slot.replace(m.won_by(team)).is_some()
        {
            return None;
        }
    }
    let wins = needed.iter().try_fold(0, |wins, &result| Some(wins + u32::from(result?)))?;
    Some((wins, round - 1 - wins))
}

fn features(view: &MatchView) -> Result<Option<Features>> {
    let Some(anchor) = view.anchor()? else { return Ok(None) };
    let anchor_logit = logit(anchor);
    let Some(anchor_at) = view.observed_at() else { return Ok(None) };
    let mut values = vec![0.0; SIGNAL.columns.len()];
    let mut at = anchor_at;
    match (view.context.stage_kind, view.context.round, view.context.bracket_round) {
        (StageKind::Swiss, Some(round), _) => {
            let ((a_wins, a_losses), (b_wins, b_losses)) = if round == 1 {
                ((0, 0), (0, 0))
            } else {
                let Some((a, b, history_at)) = team_features(view, |team, history| {
                    swiss_record(team, history, view.context.stage_id, round, view.cutoff)
                })? else { return Ok(None) };
                at = at.max(history_at);
                (a, b)
            };
            values[0] = anchor_logit * f64::from(a_wins + b_wins) / 2.0;
            values[1] = anchor_logit * f64::from(a_losses + b_losses) / 2.0;
            if a_wins == 2 && b_wins == 2 { values[2] = anchor_logit; }
            if a_losses == 2 && b_losses == 2 { values[3] = anchor_logit; }
        }
        (StageKind::Playoffs, _, Some(round)) => {
            let index = match round {
                BracketRound::RoundOf16 => 4,
                BracketRound::Quarterfinal => 5,
                BracketRound::Semifinal => 6,
                BracketRound::Final => 7,
            };
            values[index] = anchor_logit;
        }
        _ => return Ok(None),
    }
    Ok(Some((values, at)))
}
