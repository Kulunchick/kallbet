//! Catch-up policy experiment over point-in-time archive views.

use super::Lab;
use crate::{
    Instant, Result,
    ports::{RandomSource, SeededRng},
};
use std::{collections::BTreeMap, fmt::Write};

struct Game {
    cutoff: Instant,
    known: Option<Instant>,
    anchor: Option<f64>,
    market: Option<f64>,
    won: bool,
}

#[derive(Clone, Copy)]
enum Model {
    Follower(f64),
    Market,
}
impl Model {
    fn pick(self, g: &Game, rng: &mut SeededRng) -> Option<bool> {
        match self {
            Self::Follower(noise) => g.anchor.map(|p| (p >= 0.5) ^ (rng.next_f64() < noise)),
            Self::Market => g.market.map(|p| p >= 0.5),
        }
    }
}

fn catch_up(p: f64, deficit: usize) -> bool {
    let favourite = p >= 0.5;
    if deficit > 0 && p != 0.5 && (0.40..=0.60).contains(&p) {
        !favourite
    } else {
        favourite
    }
}

struct Finish {
    #[cfg(test)]
    points: [usize; 2],
    places: [usize; 2],
    first_change: Option<(usize, usize)>,
}

fn simulate(games: &[Game], size: usize, model: Model, rng: &mut SeededRng) -> Finish {
    // Parallel counterfactuals: slots 0 and 1 compete against the same opponents,
    // never against each other. Award one point per correct default-stage pick.
    let mut final_points = vec![0; size + 1];
    let mut visible = vec![0_usize; size + 1];
    let mut pending = BTreeMap::<(Instant, usize), Vec<usize>>::new();
    let mut first_change = None;
    for (index, g) in games.iter().enumerate() {
        while pending
            .first_key_value()
            .is_some_and(|((end, _), _)| *end < g.cutoff)
        {
            for member in pending.pop_first().unwrap().1 {
                visible[member] += 1;
            }
        }
        let leader = visible[2..].iter().copied().max().unwrap();
        let deficit = leader.saturating_sub(visible[1]);
        let plain = g.anchor.map(|p| p >= 0.5);
        let variant = g.anchor.map(|p| catch_up(p, deficit));
        if plain != variant && first_change.is_none() {
            // Retrospective stratification only. Include all simultaneous cutoffs.
            let already_locked = games.partition_point(|m| m.cutoff < g.cutoff);
            first_change = Some((deficit, games.len() - already_locked));
        }
        let picks = [plain, variant]
            .into_iter()
            .chain((1..size).map(|_| model.pick(g, rng)));
        let mut correct = Vec::new();
        for (member, pick) in picks.enumerate() {
            if pick == Some(g.won) {
                final_points[member] += 1;
                correct.push(member);
            }
        }
        if let Some(end) = g.known {
            pending.insert((end, index), correct);
        }
    }
    let place = |points| {
        1 + final_points[2..]
            .iter()
            .filter(|&&other| other > points)
            .count()
    };
    Finish {
        #[cfg(test)]
        points: [final_points[0], final_points[1]],
        places: [place(final_points[0]), place(final_points[1])],
        first_change,
    }
}

#[derive(Default)]
struct Totals {
    n: usize,
    first: [usize; 2],
    place: [usize; 2],
}
impl Totals {
    fn add(&mut self, finish: &Finish) {
        self.n += 1;
        for (i, p) in finish.places.iter().copied().enumerate() {
            self.first[i] += usize::from(p == 1);
            self.place[i] += p;
        }
    }
    // Two-sided 90% Hoeffding bound for independent bounded League draws.
    fn interval(&self, sum: f64, low: f64, high: f64) -> (f64, f64, f64) {
        let mean = sum / self.n as f64;
        let radius = (high - low) * (20.0_f64.ln() / (2.0 * self.n as f64)).sqrt();
        (mean, (mean - radius).max(low), (mean + radius).min(high))
    }
}
fn display((v, lo, hi): (f64, f64, f64)) -> String {
    format!("{v:.3} [{lo:.3}, {hi:.3}]")
}

/// Seeded paired simulations conditional on the supplied archive.
pub async fn report(lab: &Lab, seed: u64, trials: usize) -> Result<String> {
    if trials == 0 {
        return Err("league experiment needs at least one trial".into());
    }
    let mut events = BTreeMap::<i64, Vec<Game>>::new();
    for stage in &lab.archive.stages {
        for m in &stage.matches {
            let v = lab.view(&m.game, m.cutoff(), None).await?;
            events.entry(stage.event_id).or_default().push(Game {
                cutoff: m.cutoff(),
                known: (m.finished_at > m.actual_start).then_some(m.finished_at),
                anchor: v.anchor()?,
                market: v.market()?,
                won: m.winner == m.game.team_a.id,
            });
        }
    }
    let mut events: Vec<_> = events.into_values().collect();
    if events.is_empty() {
        return Ok("# Catch-up mode\n\nNot testable. No scored Matches.\n".into());
    }
    for event in &mut events {
        event.sort_by_key(|g| g.cutoff);
    }
    let games = events.iter().flatten();
    let digest = &lab.archive.digest;
    let mut out = format!(
        "# Catch-up mode\n\nArchive SHA-256 `{digest}`. Seed {seed}. Each of {trials} paired trials draws one of {} archived Events uniformly and replays all its scored Matches. Of {} Matches, {} have an anchor and {} have a market line. An unavailable input means a missed pick. Favourite followers independently reverse the anchor favourite with the stated probability. Market followers always take the market favourite. Exact ties pick team A.\n\nBoth bots face identical opponents. The variant reverses a favourite of over 50% through 60% whenever it trails. Only results published strictly before the cutoff enter standings. Missing end times enter final scoring only. Default Stage weights award one Match Point per correct pick. Equal points share place, including first.\n\nBrackets are pointwise 90% Hoeffding intervals for the simulated means conditional on these Events and opponent models. They measure Monte Carlo uncertainty only.\n",
        events.len(),
        games.clone().count(),
        games.clone().filter(|g| g.anchor.is_some()).count(),
        games.filter(|g| g.market.is_some()).count()
    );
    for (label, model) in [
        ("Favourite follower (10% noise)", Model::Follower(0.1)),
        ("Favourite follower (25% noise)", Model::Follower(0.25)),
        ("Market only", Model::Market),
    ] {
        writeln!(
            out,
            "\n## {label}\n\n| Members | Policy | First place probability | Expected place |\n|---:|---|---:|---:|"
        )?;
        let mut verdicts = String::new();
        for size in [4, 12] {
            let mut rng = SeededRng::new(seed);
            let mut all = Totals::default();
            let mut groups = BTreeMap::<(&str, &str), Totals>::new();
            for _ in 0..trials {
                let event = (rng.next_f64() * events.len() as f64) as usize;
                let f = simulate(&events[event], size, model, &mut rng);
                all.add(&f);
                if let Some((deficit, left)) = f.first_change {
                    let key = (
                        if deficit == 1 { "1" } else { "2+" },
                        if left <= 10 { "1-10" } else { "11+" },
                    );
                    groups.entry(key).or_default().add(&f);
                }
            }
            for (i, policy) in ["plain", "catch-up"].iter().enumerate() {
                writeln!(
                    out,
                    "| {size} | {policy} | {} | {} |",
                    display(all.interval(all.first[i] as f64, 0., 1.)),
                    display(all.interval(all.place[i] as f64, 1., size as f64))
                )?;
            }
            if groups.is_empty() {
                writeln!(
                    verdicts,
                    "| {size} | — | — | {trials} | 0 | 0 | identical in every trial (no changed pick) |"
                )?;
            }
            for ((gap, left), group) in groups {
                let delta = group.interval(group.first[1] as f64 - group.first[0] as f64, -1., 1.);
                let max_shift = (size - 1) as f64;
                let place_delta = group.interval(
                    group.place[1] as f64 - group.place[0] as f64,
                    -max_shift,
                    max_shift,
                );
                let verdict = if delta.1 > 0. {
                    "higher"
                } else if delta.2 < 0. {
                    "lower"
                } else {
                    "inconclusive"
                };
                writeln!(
                    verdicts,
                    "| {size} | {gap} | {left} | {} | {} | {} | {verdict} |",
                    group.n,
                    display(delta),
                    display(place_delta)
                )?;
            }
        }
        writeln!(
            out,
            "\n### Verdict\n\nConditions at the first changed pick. Matches left includes the current cutoff. A higher chance of first and a lower expected place favour catch-up. Trials without a changed pick have identical finishes.\n\n| Members | points behind | Matches left | Trials | Difference in first-place probability | Difference in expected place | Chance of first |\n|---:|---:|---:|---:|---:|---:|---|\n{verdicts}"
        )?;
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interval_for_a_hand_counted_first_place_frequency() {
        let totals = Totals {
            n: 200,
            ..Default::default()
        };
        let (mean, lo, hi) = totals.interval(50., 0., 1.);
        assert_eq!(mean, 0.25);
        assert!((lo - 0.16346).abs() < 0.00001);
        assert!((hi - 0.33654).abs() < 0.00001);
        assert_eq!(totals.interval(0., 0., 1.).1, 0.);
    }

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
        let mut games = vec![
            game(0, 9, Some(0.8), false),
            game(1, 8, None, true),
            game(10, 20, Some(0.54), false),
        ];
        let play = |games: &[Game]| simulate(games, 3, Model::Market, &mut SeededRng::new(3));
        let early = play(&games);
        assert_eq!(early.points, [0, 1]);
        assert_eq!(early.places, [3, 3]);
        // Cross-check the same League against the server's ranking entry point.
        use kallbet::domain::{
            Match, MatchFormat, MatchId, MatchPlacement, MatchStatus, StageOrdinal, TeamId, UserId,
            scoring,
        };
        let matches: Vec<_> = games
            .iter()
            .enumerate()
            .map(|(i, g)| Match {
                id: MatchId(i as i64),
                stage: StageOrdinal(1),
                placement: MatchPlacement::Unplaced,
                team_a: Some(TeamId(10)),
                team_b: Some(TeamId(20)),
                format: MatchFormat::Bo3,
                scheduled_start: None,
                actual_start: None,
                status: MatchStatus::Finished,
                winner: Some(TeamId(if g.won { 10 } else { 20 })),
                score: None,
            })
            .collect();
        let scored: Vec<_> = matches
            .iter()
            .map(|game| scoring::ScoredMatch {
                game,
                playoffs: false,
                points: 1,
            })
            .collect();
        for policy in 0..2 {
            let board =
                scoring::leaderboard(&[UserId(0), UserId(1), UserId(2)], &scored, |user, m| {
                    if user != UserId(0) {
                        return Some(TeamId(20));
                    }
                    match m.0 {
                        1 => None,
                        2 if policy == 1 => Some(TeamId(20)),
                        _ => Some(TeamId(10)),
                    }
                });
            let bot = board.iter().find(|r| r.user_id == UserId(0)).unwrap();
            assert_eq!(
                (bot.points as usize, bot.place as usize),
                (early.points[policy], early.places[policy])
            );
        }
        assert_eq!(early.first_change, Some((1, 1)));
        for known in [
            Instant::from_timestamp(10, 0),
            Instant::from_timestamp(11, 0),
            None,
        ] {
            games[0].known = known;
            let late = play(&games);
            assert_eq!(late.first_change, None);
            assert_eq!(late.points, [0, 0]);
        }
        // All three Members pick the winner and share first.
        games.truncate(1);
        games[0].anchor = Some(0.4);
        assert_eq!(play(&games).places, [1, 1]);
    }

    #[test]
    fn catch_up_flips_only_close_matches_when_trailing() {
        // When not trailing, always takes the favourite (team A at >= 0.5).
        for p in [0.39, 0.40, 0.45, 0.50, 0.55, 0.60, 0.61] {
            assert_eq!(catch_up(p, 0), p >= 0.5, "deficit=0 p={p}");
        }
        // When trailing, flips strictly inside (0.40..=0.60) excluding 0.50.
        assert!(!catch_up(0.39, 1));
        assert!(catch_up(0.40, 1));
        assert!(catch_up(0.45, 1));
        assert!(catch_up(0.50, 1));
        assert!(!catch_up(0.55, 1));
        assert!(!catch_up(0.60, 1));
        assert!(catch_up(0.61, 2));
    }

    #[test]
    fn future_results_never_change_earlier_decisions() {
        let base = vec![
            game(0, 5, Some(0.8), false),
            game(10, 15, Some(0.54), true),
            game(20, 25, Some(0.54), true),
        ];
        let mut flipped_future = vec![
            game(0, 5, Some(0.8), false),
            game(10, 15, Some(0.54), true),
            game(20, 25, Some(0.54), false),
        ];
        flipped_future[2].known = None;
        let r1 = simulate(&base, 4, Model::Market, &mut SeededRng::new(9));
        let r2 = simulate(&flipped_future, 4, Model::Market, &mut SeededRng::new(9));
        // Both diverge on the same second match with the same live deficit.
        assert_eq!(r1.first_change, Some((1, 2)));
        assert_eq!(r2.first_change, r1.first_change);
    }
}
