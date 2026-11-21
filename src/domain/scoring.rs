//! Scoring (pure): Match Points, Playoff Points, % correct, shared-place ranking.
//!
//! Points are never stored: the Leaderboard is computed on read from the
//! League's effective Match view and its Predictions, so a new result, or an
//! Override (#13), is reflected the next time anyone looks.

use super::{Match, MatchId, MatchStatus, TeamId, UserId};

/// What one Member's (possible) Prediction is worth on one Match.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    /// The picked team won: 1 Point.
    Correct,
    /// The other team won: 0 Points.
    Wrong,
    /// A decided Match without a Prediction: 0 Points.
    Missing,
    /// A Void Match counts for nothing, whatever was predicted.
    Void,
    /// No result yet.
    Pending,
}

impl Verdict {
    pub fn points(self) -> u32 {
        u32::from(self == Verdict::Correct)
    }
}

/// Judges `pick` against `game`'s result.
pub fn judge(game: &Match, pick: Option<TeamId>) -> Verdict {
    if game.is_void() {
        return Verdict::Void;
    }
    match (game.status, game.winner) {
        (MatchStatus::Finished, Some(winner)) => match pick {
            Some(team) if team == winner => Verdict::Correct,
            Some(_) => Verdict::Wrong,
            None => Verdict::Missing,
        },
        _ => Verdict::Pending,
    }
}

/// A Match to score, and whether it belongs to the Playoffs.
#[derive(Debug, Clone, Copy)]
pub struct ScoredMatch<'a> {
    pub game: &'a Match,
    pub playoffs: bool,
}

/// One Member's row of the Leaderboard.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Standing {
    /// 1-based; Members with equal Points share a place and the next place is
    /// skipped (1, 1, 3).
    pub place: u32,
    pub user_id: UserId,
    pub points: u32,
    /// The part of `points` earned on Playoffs Matches.
    pub playoff_points: u32,
    pub correct: u32,
    /// Predictions on decided, non-Void Matches that were wrong.
    pub wrong: u32,
}

impl Standing {
    /// Correct Predictions as a whole percentage of the Predictions made on
    /// decided, non-Void Matches. `None` before there is any.
    pub fn percent_correct(&self) -> Option<u32> {
        let made = self.correct + self.wrong;
        (made > 0).then(|| (self.correct * 100 + made / 2) / made)
    }
}

/// Ranks `members` by Points over `games`. `pick_of(user, match)` is the
/// team the Member predicted, if any. Pass a subset of `games` (e.g. one
/// match day) to get the Points gained over it. Rows come ordered by place,
/// then by user id.
pub fn leaderboard(
    members: &[UserId],
    games: &[ScoredMatch],
    pick_of: impl Fn(UserId, MatchId) -> Option<TeamId>,
) -> Vec<Standing> {
    let mut rows: Vec<Standing> = members
        .iter()
        .map(|&user_id| {
            let mut row = Standing {
                place: 0,
                user_id,
                points: 0,
                playoff_points: 0,
                correct: 0,
                wrong: 0,
            };
            for scored in games {
                let verdict = judge(scored.game, pick_of(user_id, scored.game.id));
                row.points += verdict.points();
                if scored.playoffs {
                    row.playoff_points += verdict.points();
                }
                match verdict {
                    Verdict::Correct => row.correct += 1,
                    Verdict::Wrong => row.wrong += 1,
                    _ => {}
                }
            }
            row
        })
        .collect();
    rows.sort_by_key(|r| (std::cmp::Reverse(r.points), r.user_id));
    let mut place = 0;
    for i in 0..rows.len() {
        if i == 0 || rows[i].points != rows[i - 1].points {
            place = i as u32 + 1;
        }
        rows[i].place = place;
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{MatchFormat, StageOrdinal};
    use crate::domain::MatchPlacement;

    fn game(status: MatchStatus, winner: Option<i64>) -> Match {
        Match {
            id: MatchId(1),
            stage: StageOrdinal(1),
            placement: MatchPlacement::Unplaced,
            team_a: Some(TeamId(10)),
            team_b: Some(TeamId(20)),
            format: MatchFormat::Bo3,
            scheduled_start: None,
            actual_start: None,
            status,
            winner: winner.map(TeamId),
            score: None,
        }
    }

    #[test]
    fn the_winning_pick_is_correct_and_earns_a_point() {
        let v = judge(&game(MatchStatus::Finished, Some(10)), Some(TeamId(10)));
        assert_eq!(v, Verdict::Correct);
        assert_eq!(v.points(), 1);
    }

    #[test]
    fn the_losing_pick_is_wrong_and_earns_nothing() {
        let v = judge(&game(MatchStatus::Finished, Some(10)), Some(TeamId(20)));
        assert_eq!(v, Verdict::Wrong);
        assert_eq!(v.points(), 0);
    }

    #[test]
    fn no_prediction_on_a_decided_match_is_missing() {
        let v = judge(&game(MatchStatus::Finished, Some(10)), None);
        assert_eq!(v, Verdict::Missing);
        assert_eq!(v.points(), 0);
    }

    #[test]
    fn void_matches_count_for_nothing_whatever_was_predicted() {
        let cancelled = game(MatchStatus::Cancelled, None);
        let no_winner = game(MatchStatus::Finished, None);
        for g in [&cancelled, &no_winner] {
            assert_eq!(judge(g, Some(TeamId(10))), Verdict::Void);
            assert_eq!(judge(g, None), Verdict::Void);
        }
    }

    #[test]
    fn a_match_without_a_result_is_pending() {
        assert_eq!(
            judge(&game(MatchStatus::Live, None), Some(TeamId(10))),
            Verdict::Pending
        );
        assert_eq!(
            judge(&game(MatchStatus::Upcoming, None), None),
            Verdict::Pending
        );
    }

    fn decided(id: i64, winner: i64) -> Match {
        Match {
            id: MatchId(id),
            ..game(MatchStatus::Finished, Some(winner))
        }
    }

    /// `picks`: (user, match, team).
    fn board(members: &[i64], games: &[(Match, bool)], picks: &[(i64, i64, i64)]) -> Vec<Standing> {
        let members: Vec<UserId> = members.iter().map(|&u| UserId(u)).collect();
        let games: Vec<ScoredMatch> = games
            .iter()
            .map(|(game, playoffs)| ScoredMatch {
                game,
                playoffs: *playoffs,
            })
            .collect();
        leaderboard(&members, &games, |user, game| {
            picks
                .iter()
                .find(|&&(u, g, _)| UserId(u) == user && MatchId(g) == game)
                .map(|&(_, _, t)| TeamId(t))
        })
    }

    fn places(board: &[Standing]) -> Vec<(i64, u32, u32)> {
        board
            .iter()
            .map(|s| (s.user_id.0, s.place, s.points))
            .collect()
    }

    #[test]
    fn equal_points_share_a_place_and_the_next_place_is_skipped() {
        let games = [(decided(1, 10), false), (decided(2, 10), false)];
        // users 1 and 2 get 2 points, user 3 gets 1, user 4 gets 1... but 5 gets 0
        let picks = [
            (1, 1, 10),
            (1, 2, 10),
            (2, 1, 10),
            (2, 2, 10),
            (3, 1, 10),
            (4, 2, 10),
        ];
        let b = board(&[5, 4, 3, 2, 1], &games, &picks);
        assert_eq!(
            places(&b),
            [(1, 1, 2), (2, 1, 2), (3, 3, 1), (4, 3, 1), (5, 5, 0)]
        );
    }

    #[test]
    fn playoff_points_count_only_playoffs_matches_but_also_add_to_points() {
        let games = [(decided(1, 10), false), (decided(2, 10), true)];
        let b = board(&[1], &games, &[(1, 1, 10), (1, 2, 10)]);
        assert_eq!((b[0].points, b[0].playoff_points), (2, 1));
    }

    #[test]
    fn a_member_without_predictions_scores_zero() {
        let b = board(&[1], &[(decided(1, 10), false)], &[]);
        assert_eq!((b[0].points, b[0].correct, b[0].wrong), (0, 0, 0));
        assert_eq!(b[0].percent_correct(), None);
    }

    #[test]
    fn percent_correct_excludes_void_pending_and_missing() {
        let games = [
            (decided(1, 10), false),
            (decided(2, 10), false),
            (decided(3, 10), false),
            (
                Match {
                    id: MatchId(4),
                    ..game(MatchStatus::Cancelled, None)
                },
                false,
            ),
            (
                Match {
                    id: MatchId(5),
                    ..game(MatchStatus::Live, None)
                },
                false,
            ),
            (decided(6, 10), false),
        ];
        // right, right, wrong, void, pending, missing: 2 of 3 predictions.
        let picks = [(1, 1, 10), (1, 2, 10), (1, 3, 20), (1, 4, 10), (1, 5, 10)];
        let b = board(&[1], &games, &picks);
        assert_eq!((b[0].correct, b[0].wrong), (2, 1));
        assert_eq!(b[0].percent_correct(), Some(67));
    }
}
