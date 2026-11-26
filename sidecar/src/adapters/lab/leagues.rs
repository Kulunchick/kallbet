//! Catch-up policy experiment over point-in-time archive views.

#[cfg(test)]
mod tests {
    use super::*;

    fn game(cutoff: i64, end: i64, p: Option<f64>, won: bool) -> Game {
        Game {
            cutoff: Instant::from_timestamp(cutoff, 0).unwrap(),
            known: Instant::from_timestamp(end, 0),
            anchor: p,
            market: Some(0.4),
            won,
        }
    }

    #[test]
    fn hand_scored_league_and_strict_reveal_boundary() {
        // Two market followers pick B three times. The bot picks A, misses
        // a prediction, then takes B only if the first result is public.
        let mut games = vec![game(0, 9, Some(0.8), false),
                             game(1, 8, None, true),
                             game(10, 20, Some(0.54), false)];
        let play = |games: &[Game]| simulate(games, 3, Model::Market, &mut SeededRng::new(3));
        let early = play(&games);
        assert_eq!(early.points, [0, 1]);
        assert_eq!(early.places, [3, 3]);
        assert_eq!(early.first_change, Some((1, 1)));
        for known in [Instant::from_timestamp(10, 0), Instant::from_timestamp(11, 0), None] {
            games[0].known = known;
            let late = play(&games);
            assert_eq!(late.first_change, None);
            assert_eq!(late.points, [0, 0]);
        }
        // A shared first (1,1,1), including when the anchor has no pick.
        games.truncate(1);
        games[0].anchor = Some(0.4);
        assert_eq!(play(&games).places, [1, 1]);
    }
}
