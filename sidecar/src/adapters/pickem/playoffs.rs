use super::swiss::{parse_format, series_win_prob};
use crate::*;
use serde_json::{Value, json};
use std::cmp::Reverse;
use std::collections::HashMap;

#[derive(Clone)]
struct SlotNode {
    rounds_before_final: u8,
    slot: u8,
    leaf: bool,
    team_a: Option<i64>,
    team_b: Option<i64>,
    format: Format,
}

impl SlotNode {
    fn id(&self) -> u16 {
        (1_u16 << self.rounds_before_final) + self.slot as u16
    }
}

pub(crate) fn solve_playoffs(
    stage_val: &Value,
    ratings: &[AsOf<PairwiseProbability>],
    cutoff: Instant,
    avoid: Option<super::Avoid<'_>>,
) -> Result<Option<Value>> {
    let bracket = &stage_val["bracket"];
    if bracket["ready"].as_bool() != Some(true) {
        return Ok(None);
    }
    let Some(slots_arr) = bracket["slots"].as_array() else {
        return Ok(None);
    };
    if slots_arr.is_empty() {
        return Ok(None);
    }

    // Collect optional per-slot Match formats from `groups` if present.
    let mut group_formats: HashMap<(u8, u8), Format> = HashMap::new();
    if let Some(groups) = stage_val["groups"].as_array() {
        for g in groups {
            if g["kind"].as_str() == Some("playoff_round")
                && let Some(r) = g["rounds_before_final"].as_u64()
                && let Some(matches) = g["matches"].as_array()
            {
                for m in matches {
                    if let (Some(s), Some(fmt)) = (
                        m["bracket_slot"].as_u64(),
                        parse_format(m["format"].as_str()),
                    ) {
                        group_formats.insert((r as u8, s as u8), fmt);
                    }
                }
            }
        }
    }

    let mut nodes: HashMap<u16, SlotNode> = HashMap::with_capacity(slots_arr.len());
    for s in slots_arr {
        let r = s["rounds_before_final"]
            .as_u64()
            .ok_or("missing rounds_before_final")? as u8;
        let slot = s["slot"].as_u64().ok_or("missing slot")? as u8;
        let leaf = s["leaf"].as_bool().ok_or("missing leaf")?;
        let team_a = s["team_a"].as_i64();
        let team_b = s["team_b"].as_i64();
        if leaf && (team_a.is_none() || team_b.is_none()) {
            return Ok(None);
        }
        let format = parse_format(s["format"].as_str())
            .or_else(|| group_formats.get(&(r, slot)).copied())
            .unwrap_or(Format::Bo3);
        let node = SlotNode {
            rounds_before_final: r,
            slot,
            leaf,
            team_a,
            team_b,
            format,
        };
        nodes.insert(node.id(), node);
    }

    if !nodes.contains_key(&1) {
        return Ok(None);
    }

    // Map pairwise probabilities keyed by ordered (min_id, max_id) -> P(min_id beats max_id on 1 map).
    let mut map_probs: HashMap<(i64, i64), f64> = HashMap::new();
    for rating in ratings {
        let r = rating.at(cutoff)?;
        if r.first < r.second {
            map_probs.insert((r.first, r.second), r.map.clamp(0.0, 1.0));
        } else if r.second < r.first {
            map_probs.insert((r.second, r.first), (1.0 - r.map).clamp(0.0, 1.0));
        }
    }

    let match_win_prob = |fmt: Format, a: i64, b: i64| -> f64 {
        if a == b {
            return 0.5;
        }
        let p_map = if a < b {
            map_probs.get(&(a, b)).copied().unwrap_or(0.5)
        } else {
            1.0 - map_probs.get(&(b, a)).copied().unwrap_or(0.5)
        };
        series_win_prob(fmt, p_map)
    };

    // A pick the field shares is worth less (`Avoid`): the weight of picking
    // `team` to win the slot with heap id `id`.
    let shared: HashMap<u16, i64> = avoid
        .map(|a| {
            a.picks["picks"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|p| {
                    let r = p["rounds_before_final"].as_u64()? as u16;
                    let slot = p["slot"].as_u64()? as u16;
                    Some(((1_u16 << r) + slot, p["team_id"].as_i64()?))
                })
                .collect()
        })
        .unwrap_or_default();
    let keep = |id: u16, team: i64| match avoid {
        Some(a) if shared.get(&id) == Some(&team) => 1.0 - a.discount,
        _ => 1.0,
    };

    // Process slots bottom-up (largest rounds_before_final / largest heap id first).
    let mut order: Vec<u16> = nodes.keys().copied().collect();
    order.sort_by_key(|&id| Reverse(id));

    // win_probs[slot_id] = Vec<(team_id, P(team reaches and wins slot_id))>
    // dp[slot_id] = Vec<(team_id, max expected Points in subtree of slot_id when slot_id picks team_id)>
    let mut win_probs: HashMap<u16, Vec<(i64, f64)>> = HashMap::with_capacity(order.len());
    let mut dp: HashMap<u16, Vec<(i64, f64)>> = HashMap::with_capacity(order.len());

    let argmax = |entries: &[(i64, f64)]| -> (i64, f64) {
        *entries
            .iter()
            .max_by(|(ta, va), (tb, vb)| va.total_cmp(vb).then_with(|| tb.cmp(ta)))
            .expect("non-empty candidate list")
    };

    for &id in &order {
        let node = &nodes[&id];
        if node.leaf {
            let a = node.team_a.expect("checked leaf team_a");
            let b = node.team_b.expect("checked leaf team_b");
            let pa = match_win_prob(node.format, a, b);
            let pb = 1.0 - pa;
            let w = vec![(a, pa), (b, pb)];
            dp.insert(
                id,
                w.iter().map(|&(t, p)| (t, p * keep(id, t))).collect(),
            );
            win_probs.insert(id, w);
        } else {
            let left_id = id * 2;
            let right_id = id * 2 + 1;
            let left_w = win_probs
                .get(&left_id)
                .ok_or("missing left feeder slot in bracket")?
                .clone();
            let right_w = win_probs
                .get(&right_id)
                .ok_or("missing right feeder slot in bracket")?
                .clone();
            let left_dp = dp[&left_id].clone();
            let right_dp = dp[&right_id].clone();

            let (_, best_left_ev) = argmax(&left_dp);
            let (_, best_right_ev) = argmax(&right_dp);

            let mut w_cur = Vec::with_capacity(left_w.len() + right_w.len());
            let mut dp_cur = Vec::with_capacity(left_w.len() + right_w.len());

            for &(x, wx) in &left_w {
                let win_vs_right: f64 = right_w
                    .iter()
                    .map(|&(y, wy)| wy * match_win_prob(node.format, x, y))
                    .sum();
                let w_u_x = wx * win_vs_right;
                let dp_l_x = left_dp.iter().find(|&&(t, _)| t == x).unwrap().1;
                w_cur.push((x, w_u_x));
                dp_cur.push((x, w_u_x * keep(id, x) + dp_l_x + best_right_ev));
            }

            for &(y, wy) in &right_w {
                let win_vs_left: f64 = left_w
                    .iter()
                    .map(|&(x, wx)| wx * match_win_prob(node.format, y, x))
                    .sum();
                let w_u_y = wy * win_vs_left;
                let dp_r_y = right_dp.iter().find(|&&(t, _)| t == y).unwrap().1;
                w_cur.push((y, w_u_y));
                dp_cur.push((y, w_u_y * keep(id, y) + best_left_ev + dp_r_y));
            }

            win_probs.insert(id, w_cur);
            dp.insert(id, dp_cur);
        }
    }

    // Reconstruct optimal path-consistent picks top-down starting at the Final (id = 1).
    let mut chosen: HashMap<u16, i64> = HashMap::with_capacity(order.len());
    let (champion, _) = argmax(&dp[&1]);
    let mut stack = vec![(1_u16, champion)];
    while let Some((id, pick)) = stack.pop() {
        chosen.insert(id, pick);
        let node = &nodes[&id];
        if !node.leaf {
            let left_id = id * 2;
            let right_id = id * 2 + 1;
            let in_left = dp[&left_id].iter().any(|&(t, _)| t == pick);
            if in_left {
                let (best_r, _) = argmax(&dp[&right_id]);
                stack.push((left_id, pick));
                stack.push((right_id, best_r));
            } else {
                let (best_l, _) = argmax(&dp[&left_id]);
                stack.push((left_id, best_l));
                stack.push((right_id, pick));
            }
        }
    }

    let mut sorted_nodes: Vec<&SlotNode> = nodes.values().collect();
    sorted_nodes.sort_by_key(|n| (Reverse(n.rounds_before_final), n.slot));

    let picks: Vec<Value> = sorted_nodes
        .into_iter()
        .map(|n| {
            json!({
                "rounds_before_final": n.rounds_before_final,
                "slot": n.slot,
                "team_id": chosen[&n.id()],
            })
        })
        .collect();

    Ok(Some(json!({ "picks": picks })))
}
