use crate::*;
use serde_json::{Value, json};
use std::cmp::Reverse;
use std::sync::OnceLock;

const SWISS_TEAMS: usize = 16;
const THREE_NIL_SLOTS: usize = 2;
const ZERO_THREE_SLOTS: usize = 2;
const ADVANCE_SLOTS: usize = 6;

pub(crate) fn series_win_prob(format: Format, map_prob: f64) -> f64 {
    let p = map_prob.clamp(0.0, 1.0);
    match format {
        Format::Bo1 => p,
        Format::Bo3 => p * p * (3.0 - 2.0 * p),
        Format::Bo5 => p.powi(3) * (10.0 - 15.0 * p + 6.0 * p * p),
    }
}

pub(crate) fn parse_format(s: Option<&str>) -> Option<Format> {
    match s? {
        "bo1" => Some(Format::Bo1),
        "bo3" => Some(Format::Bo3),
        "bo5" => Some(Format::Bo5),
        _ => None,
    }
}

struct PairingTables {
    four: Vec<[(usize, usize); 2]>,
    six: Vec<[(usize, usize); 3]>,
    eight: Vec<[(usize, usize); 4]>,
}

fn generate_matchings<const K: usize>(n: usize) -> Vec<[(usize, usize); K]> {
    fn rec(used: u16, n: usize, cur: &mut Vec<(usize, usize)>, out: &mut Vec<Vec<(usize, usize)>>) {
        let Some(first) = (0..n).find(|&i| (used & (1 << i)) == 0) else {
            out.push(cur.clone());
            return;
        };
        for second in ((first + 1)..n).rev() {
            if (used & (1 << second)) == 0 {
                cur.push((first, second));
                rec(used | (1 << first) | (1 << second), n, cur, out);
                cur.pop();
            }
        }
    }

    let mut raw = Vec::new();
    rec(0, n, &mut Vec::with_capacity(K), &mut raw);

    raw.into_iter()
        .map(|m| m.try_into().expect("matching size K"))
        .collect()
}

fn pairing_tables() -> &'static PairingTables {
    static TABLES: OnceLock<PairingTables> = OnceLock::new();
    TABLES.get_or_init(|| PairingTables {
        four: generate_matchings::<2>(4),
        // Valve Major Supplemental Rulebook, Swiss Bracket priority table.
        // Keep the published one-based seed numbers visible for review.
        six: [
            [(1,6),(2,5),(3,4)], [(1,6),(2,4),(3,5)],
            [(1,5),(2,6),(3,4)], [(1,5),(2,4),(3,6)],
            [(1,4),(2,6),(3,5)], [(1,4),(2,5),(3,6)],
            [(1,6),(2,3),(4,5)], [(1,5),(2,3),(4,6)],
            [(1,3),(2,6),(4,5)], [(1,3),(2,5),(4,6)],
            [(1,4),(2,3),(5,6)], [(1,3),(2,4),(5,6)],
            [(1,2),(3,6),(4,5)], [(1,2),(3,5),(4,6)],
            [(1,2),(3,4),(5,6)],
        ].map(|row| row.map(|(a,b)| (a-1,b-1))).to_vec(),
        eight: generate_matchings::<4>(8),
    })
}

pub(crate) fn solve_swiss(
    stage_val: &Value,
    ratings: &[AsOf<PairwiseProbability>],
    cutoff: Instant,
    simulations: usize,
    rng: &mut dyn RandomSource,
) -> Result<Option<Value>> {
    let Some(teams_arr) = stage_val["teams"].as_array() else {
        return Ok(None);
    };
    if teams_arr.len() != SWISS_TEAMS {
        return Ok(None);
    }

    let mut team_ids = [0_i64; SWISS_TEAMS];
    for (idx, t) in teams_arr.iter().enumerate() {
        team_ids[idx] = t["id"].as_i64().ok_or("invalid team id in Swiss stage")?;
    }

    let index_of = |id: i64| -> Option<usize> { team_ids.iter().position(|&tid| tid == id) };

    let mut map_prob = [[0.5_f64; SWISS_TEAMS]; SWISS_TEAMS];
    for (i, row) in map_prob.iter_mut().enumerate() {
        row[i] = 0.0;
    }
    for rating in ratings {
        let r = rating.at(cutoff)?;
        if let (Some(a), Some(b)) = (index_of(r.first), index_of(r.second))
            && a != b
        {
            let p = r.map.clamp(0.0, 1.0);
            map_prob[a][b] = p;
            map_prob[b][a] = 1.0 - p;
        }
    }

    // Determine initial seeds 0..16 (0 = strongest seed).
    let mut initial_seed = [0_u8; SWISS_TEAMS];
    let explicit_seeds = stage_val["seeds"].as_array().and_then(|arr| {
        if arr.len() != SWISS_TEAMS {
            return None;
        }
        let mut order = Vec::with_capacity(SWISS_TEAMS);
        let mut seen = 0_u16;
        for v in arr {
            let idx = index_of(v.as_i64()?)?;
            if (seen & (1 << idx)) != 0 {
                return None;
            }
            seen |= 1 << idx;
            order.push(idx);
        }
        Some(order)
    });

    let seed_order: Vec<usize> = if let Some(order) = explicit_seeds {
        order
    } else {
        let mut order: Vec<usize> = (0..SWISS_TEAMS).collect();
        order.sort_by(|&a, &b| {
            let sum_a: f64 = (0..SWISS_TEAMS)
                .filter(|&j| j != a)
                .map(|j| map_prob[a][j])
                .sum();
            let sum_b: f64 = (0..SWISS_TEAMS)
                .filter(|&j| j != b)
                .map(|j| map_prob[b][j])
                .sum();
            let qa = (sum_a * 1e12).round() as i64;
            let qb = (sum_b * 1e12).round() as i64;
            qb.cmp(&qa).then_with(|| team_ids[a].cmp(&team_ids[b]))
        });
        order
    };
    for (rank, &team_idx) in seed_order.iter().enumerate() {
        initial_seed[team_idx] = rank as u8;
    }

    let stage_name = stage_val["stage"]["name"].as_str().unwrap_or("");
    let explicit_all_bo3 = stage_val["all_bo3"].as_bool().unwrap_or(false);

    // Parse Round 1 pairings and any bucket format hints from `groups`.
    let mut r1_matches: Vec<(usize, usize, Format)> = Vec::new();
    let mut bucket_formats: [[Option<Format>; 3]; 3] = [[None; 3]; 3];
    if let Some(groups) = stage_val["groups"].as_array() {
        for g in groups {
            let wins = g["wins"].as_u64().unwrap_or(0) as usize;
            let losses = g["losses"].as_u64().unwrap_or(0) as usize;
            let Some(matches) = g["matches"].as_array() else {
                continue;
            };
            for m in matches {
                let fmt = parse_format(m["format"].as_str());
                if wins < 3 && losses < 3 && fmt.is_some() {
                    bucket_formats[wins][losses] = fmt;
                }
                if wins == 0 && losses == 0 {
                    let ta = m["team_a"]["id"].as_i64().and_then(index_of);
                    let tb = m["team_b"]["id"].as_i64().and_then(index_of);
                    if let (Some(a), Some(b)) = (ta, tb)
                        && a != b
                    {
                        r1_matches.push((a, b, fmt.unwrap_or(Format::Bo1)));
                    }
                }
            }
        }
    }

    let valid_r1 = r1_matches.len() == SWISS_TEAMS / 2 && {
        let mut mask = 0_u16;
        for &(a, b, _) in &r1_matches {
            mask |= (1 << a) | (1 << b);
        }
        mask == u16::MAX
    };

    let all_bo3 = explicit_all_bo3
        || (valid_r1 && r1_matches.iter().all(|&(_, _, f)| matches!(f, Format::Bo3)))
        || (!valid_r1
            && bucket_formats[0][0].is_none()
            && stage_name.trim().eq_ignore_ascii_case("stage 3"));

    let default_r1_format = bucket_formats[0][0].unwrap_or(if all_bo3 {
        Format::Bo3
    } else {
        Format::Bo1
    });

    let round1: [(usize, usize, Format); 8] = if valid_r1 {
        std::array::from_fn(|k| r1_matches[k])
    } else {
        std::array::from_fn(|k| (seed_order[k], seed_order[k + 8], default_r1_format))
    };

    let format_for_bucket = |w: usize, l: usize| -> Format {
        if let Some(f) = bucket_formats[w][l] {
            return f;
        }
        if w == 2 || l == 2 || all_bo3 {
            Format::Bo3
        } else {
            Format::Bo1
        }
    };

    let mut prob_bo1 = [[0.0_f64; SWISS_TEAMS]; SWISS_TEAMS];
    let mut prob_bo3 = [[0.0_f64; SWISS_TEAMS]; SWISS_TEAMS];
    let mut prob_bo5 = [[0.0_f64; SWISS_TEAMS]; SWISS_TEAMS];
    for a in 0..SWISS_TEAMS {
        for b in 0..SWISS_TEAMS {
            if a != b {
                prob_bo1[a][b] = series_win_prob(Format::Bo1, map_prob[a][b]);
                prob_bo3[a][b] = series_win_prob(Format::Bo3, map_prob[a][b]);
                prob_bo5[a][b] = series_win_prob(Format::Bo5, map_prob[a][b]);
            }
        }
    }

    let win_prob = |fmt: Format, a: usize, b: usize| -> f64 {
        match fmt {
            Format::Bo1 => prob_bo1[a][b],
            Format::Bo3 => prob_bo3[a][b],
            Format::Bo5 => prob_bo5[a][b],
        }
    };

    let tables = pairing_tables();
    let sims = simulations.max(1);
    let mut count_3_0 = [0_u32; SWISS_TEAMS];
    let mut count_0_3 = [0_u32; SWISS_TEAMS];
    let mut count_adv = [0_u32; SWISS_TEAMS];

    const ROUND_BUCKETS: [&[(u8, u8)]; 4] = [
        &[(1, 0), (0, 1)],
        &[(2, 0), (1, 1), (0, 2)],
        &[(2, 1), (1, 2)],
        &[(2, 2)],
    ];

    for _ in 0..sims {
        let mut wins = [0_u8; SWISS_TEAMS];
        let mut losses = [0_u8; SWISS_TEAMS];
        let mut played = [0_u16; SWISS_TEAMS];

        // Round 1
        for &(a, b, fmt) in &round1 {
            played[a] |= 1 << b;
            played[b] |= 1 << a;
            if rng.next_f64() < win_prob(fmt, a, b) {
                wins[a] += 1;
                losses[b] += 1;
            } else {
                wins[b] += 1;
                losses[a] += 1;
            }
        }

        // Rounds 2..=5
        for buckets in ROUND_BUCKETS {
            let mut buchholz = [0_i8; SWISS_TEAMS];
            for t in 0..SWISS_TEAMS {
                if wins[t] < 3 && losses[t] < 3 {
                    let mut mask = played[t];
                    let mut score = 0_i8;
                    while mask != 0 {
                        let opp = mask.trailing_zeros() as usize;
                        score += wins[opp] as i8 - losses[opp] as i8;
                        mask &= mask - 1;
                    }
                    buchholz[t] = score;
                }
            }

            for &(bw, bl) in buckets {
                let mut pool = [0_usize; 8];
                let mut len = 0;
                for t in 0..SWISS_TEAMS {
                    if wins[t] == bw && losses[t] == bl {
                        pool[len] = t;
                        len += 1;
                    }
                }
                pool[..len].sort_unstable_by_key(|&t| (Reverse(buchholz[t]), initial_seed[t]));
                let fmt = format_for_bucket(bw as usize, bl as usize);

                let pairs: &[(usize, usize)] = match len {
                    4 => tables
                        .four
                        .iter()
                        .find(|m| {
                            m.iter()
                                .all(|&(i, j)| (played[pool[i]] & (1 << pool[j])) == 0)
                        })
                        .unwrap_or(&tables.four[0])
                        .as_slice(),
                    6 => tables
                        .six
                        .iter()
                        .find(|m| {
                            m.iter()
                                .all(|&(i, j)| (played[pool[i]] & (1 << pool[j])) == 0)
                        })
                        .unwrap_or(&tables.six[0])
                        .as_slice(),
                    8 => tables
                        .eight
                        .iter()
                        .find(|m| {
                            m.iter()
                                .all(|&(i, j)| (played[pool[i]] & (1 << pool[j])) == 0)
                        })
                        .unwrap_or(&tables.eight[0])
                        .as_slice(),
                    _ => &[],
                };

                for &(i, j) in pairs {
                    let (a, b) = (pool[i], pool[j]);
                    played[a] |= 1 << b;
                    played[b] |= 1 << a;
                    if rng.next_f64() < win_prob(fmt, a, b) {
                        wins[a] += 1;
                        losses[b] += 1;
                    } else {
                        wins[b] += 1;
                        losses[a] += 1;
                    }
                }
            }
        }

        for t in 0..SWISS_TEAMS {
            if wins[t] == 3 {
                if losses[t] == 0 {
                    count_3_0[t] += 1;
                } else {
                    count_adv[t] += 1;
                }
            } else if losses[t] == 3 && wins[t] == 0 {
                count_0_3[t] += 1;
            }
        }
    }

    let inv = 1.0 / (sims as f64);
    let p30: [f64; SWISS_TEAMS] = std::array::from_fn(|i| count_3_0[i] as f64 * inv);
    let p03: [f64; SWISS_TEAMS] = std::array::from_fn(|i| count_0_3[i] as f64 * inv);
    let padv: [f64; SWISS_TEAMS] = std::array::from_fn(|i| count_adv[i] as f64 * inv);

    let (mut three_nil_idx, mut zero_three_idx, mut advance_idx) =
        optimal_swiss_assignment(&p30, &p03, &padv);

    three_nil_idx.sort_by(|&a, &b| {
        p30[b]
            .total_cmp(&p30[a])
            .then_with(|| team_ids[a].cmp(&team_ids[b]))
    });
    zero_three_idx.sort_by(|&a, &b| {
        p03[b]
            .total_cmp(&p03[a])
            .then_with(|| team_ids[a].cmp(&team_ids[b]))
    });
    advance_idx.sort_by(|&a, &b| {
        padv[b]
            .total_cmp(&padv[a])
            .then_with(|| team_ids[a].cmp(&team_ids[b]))
    });

    Ok(Some(json!({
        "three_nil": three_nil_idx.into_iter().map(|i| team_ids[i]).collect::<Vec<_>>(),
        "zero_three": zero_three_idx.into_iter().map(|i| team_ids[i]).collect::<Vec<_>>(),
        "advance": advance_idx.into_iter().map(|i| team_ids[i]).collect::<Vec<_>>(),
    })))
}

fn optimal_swiss_assignment(
    p30: &[f64; SWISS_TEAMS],
    p03: &[f64; SWISS_TEAMS],
    padv: &[f64; SWISS_TEAMS],
) -> (Vec<usize>, Vec<usize>, Vec<usize>) {
    // Role: 0 = unpicked, 1 = three_nil, 2 = zero_three, 3 = advance
    let neg_inf = f64::NEG_INFINITY;
    let mut dp =
        [[[[neg_inf; ADVANCE_SLOTS + 1]; ZERO_THREE_SLOTS + 1]; THREE_NIL_SLOTS + 1]; SWISS_TEAMS + 1];
    let mut choice =
        [[[[0_u8; ADVANCE_SLOTS + 1]; ZERO_THREE_SLOTS + 1]; THREE_NIL_SLOTS + 1]; SWISS_TEAMS + 1];
    dp[0][0][0][0] = 0.0;

    for i in 0..SWISS_TEAMS {
        for c30 in 0..=THREE_NIL_SLOTS {
            for c03 in 0..=ZERO_THREE_SLOTS {
                for cadv in 0..=ADVANCE_SLOTS {
                    let cur = dp[i][c30][c03][cadv];
                    if cur == neg_inf {
                        continue;
                    }
                    // 0: leave unpicked
                    if cur > dp[i + 1][c30][c03][cadv] {
                        dp[i + 1][c30][c03][cadv] = cur;
                        choice[i + 1][c30][c03][cadv] = 0;
                    }
                    // 1: three_nil
                    if c30 < THREE_NIL_SLOTS {
                        let cand = cur + p30[i];
                        if cand > dp[i + 1][c30 + 1][c03][cadv] {
                            dp[i + 1][c30 + 1][c03][cadv] = cand;
                            choice[i + 1][c30 + 1][c03][cadv] = 1;
                        }
                    }
                    // 2: zero_three
                    if c03 < ZERO_THREE_SLOTS {
                        let cand = cur + p03[i];
                        if cand > dp[i + 1][c30][c03 + 1][cadv] {
                            dp[i + 1][c30][c03 + 1][cadv] = cand;
                            choice[i + 1][c30][c03 + 1][cadv] = 2;
                        }
                    }
                    // 3: advance (3-1 or 3-2)
                    if cadv < ADVANCE_SLOTS {
                        let cand = cur + padv[i];
                        if cand > dp[i + 1][c30][c03][cadv + 1] {
                            dp[i + 1][c30][c03][cadv + 1] = cand;
                            choice[i + 1][c30][c03][cadv + 1] = 3;
                        }
                    }
                }
            }
        }
    }

    let mut three_nil = Vec::with_capacity(THREE_NIL_SLOTS);
    let mut zero_three = Vec::with_capacity(ZERO_THREE_SLOTS);
    let mut advance = Vec::with_capacity(ADVANCE_SLOTS);
    let (mut c30, mut c03, mut cadv) = (THREE_NIL_SLOTS, ZERO_THREE_SLOTS, ADVANCE_SLOTS);
    for i in (0..SWISS_TEAMS).rev() {
        match choice[i + 1][c30][c03][cadv] {
            1 => {
                three_nil.push(i);
                c30 -= 1;
            }
            2 => {
                zero_three.push(i);
                c03 -= 1;
            }
            3 => {
                advance.push(i);
                cadv -= 1;
            }
            _ => {}
        }
    }
    (three_nil, zero_three, advance)
}
