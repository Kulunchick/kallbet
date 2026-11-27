# `bo1_pool_depth` — Bo1 map-pool depth and first-ban features (ticket #62)

**Parent:** #33. **Source:** `docs/signal-lab/criteria-research.md` (#56).  
**Status:** Candidate Signal added after spec #33's candidate list was fixed (`added_after_list_fixed = true`), evaluated under protocol `signal-lab/3` alongside `bo1_veto_sim`.  
**Implementation:** `sidecar/src/adapters/lab/signals/bo1_pool_depth.rs` (registered as `bo1_pool_depth` in `signals::registry()`).  
**Tests:** `sidecar/tests/lab_bo1_pool_depth.rs`.

## Scope and data inputs

- **Evaluated Matches:** `Format::Bo1` Matches only (`Bo3` and `Bo5` Matches return `None`).
- **History source:** All linked complete seven-step HLTV vetoes (`Bo1`, `Bo3`, and `Bo5`) finished strictly before the Match cutoff (`view.veto_history.all_at(view.cutoff)`), each verified with `AsOf`. Holdout Stages are stripped from training-side views (`veto_history.without_stages`). A Match never reads its own veto or any concurrent/future veto.
- **Active pool:** The seven canonical maps `active = [m_0, ..., m_6]` from the latest complete seven-step HLTV veto finished strictly before `cutoff`, so map-pool rotations apply immediately.
- **Availability requirement:** Both teams must have at least one prior complete seven-step veto (`vetoes >= 1`) and the Match must have an anchor probability (`view.anchor()`).

## Feature definitions

Every prior complete veto finished at `t < cutoff` receives an exponential recency weight:

```text
w(t) = 2^(-(cutoff - t) / HALF_LIFE_DAYS)    (HALF_LIFE_DAYS = 90.0 days)
```

For each team `T` and each active map `m in active`, accumulate from all prior vetoes where `T` participated:

- `first_bans[m]`: recency-weighted count of vetoes where `m` was `T`'s opening ban (the earliest `VetoStepKind::Ban` step by `T` in steps `1..=6`, across `Bo1`, `Bo3`, and `Bo5`).
- `first_ban_total`: sum of `w` over all prior vetoes where `T` made a first ban (including maps that may have rotated out of `active`).
- `fb_share(T, m) = first_bans[m] / first_ban_total` in `[0, 1]`.
- `played[m]`, `wins[m]`: recency-weighted played and won map counts for `T` on map `m` from `veto.played_maps`.

From these counts we define four per-team / pairwise quantities:

1. **Per-map competence** (`COMPETENCE_PRIOR = 1.0`):
   ```text
   c(T, m) = (wins[m] / (played[m] + 1.0)) * (1 - fb_share(T, m))   if wins[m] > 0
           = 0                                                      otherwise
   ```
   Unplayed maps and maps with zero wins always contribute `0`. Small win samples are smoothly shrunk toward `0` (`1-0 -> 0.50`, `2-0 -> 0.67`, `5-0 -> 0.83`), converging to the team's win rate on larger samples, and maps the team normally first-bans are discounted.

2. **Map-pool depth** (`depth(T) in [0, 7)`):
   ```text
   depth(T) = sum_{m in active} c(T, m)
   ```
   Effective number of active maps team `T` plays competently.

3. **First-ban stability** (`STABILITY_PRIOR = 2.0`):
   ```text
   stability(T) = max_{m in active} (first_bans[m] + 2/7) / (first_ban_total + 2.0)
   ```
   Dirichlet-smoothed top-1 active-pool opening-ban share. Approaches `1.0` for a strict permaban in the active pool, equals `1/7` for uniform bans, and drops below `1/7` when a team's historical permaban has rotated out of the active pool.

4. **Map-pool overlap** (`overlap(A, B) in [0, 1]`):
   Weighted Jaccard (Ruzicka) similarity of `c(A, ·)` and `c(B, ·)` over the seven active maps (`0` if neither team has won an active map):
   ```text
   overlap(A, B) = (sum_{m in active} min(c(A, m), c(B, m)))
                 / (sum_{m in active} max(c(A, m), c(B, m)))
   ```

5. **First-ban disruption** (`disrupt(A on B) in [0, 1)`):
   ```text
   disrupt(A on B) = sum_{m in active} fb_share(A, m) * c(B, m)
   ```
   Expected active-pool competence of team `B` removed by team `A`'s opening ban.

### Combiner features (strictly anti-symmetric under Team A ↔ Team B)

| # | Feature name | Definition | Symmetry |
|---|---|---|---|
| 1 | `pool_depth_diff` | `(depth(A) - depth(B)) / 7` | Anti-symmetric in `(-1, 1)` |
| 2 | `first_ban_stability_diff` | `stability(A) - stability(B)` | Anti-symmetric in `(-1, 1)` |
| 3 | `pool_overlap_x_anchor` | `(overlap(A, B) - 0.5) * logit(anchor)` | Anti-symmetric (symmetric overlap × anti-symmetric anchor logit) |
| 4 | `first_ban_disruption_diff` | `disrupt(A on B) - disrupt(B on A)` | Anti-symmetric in `(-1, 1)` |

## Walk-forward fitting

For each evaluated Event, the four feature weights `w` are fitted on Bo1 training Matches from earlier completed Events by strictly convex L2-regularised logistic regression (`ridge_logistic_with_offset`, `LAMBDA = 10.0`, prior `0`) with `logit(anchor)` as a fixed offset:

```text
p_variant = sigmoid(logit(anchor) + w · x)
```

With zero prior Bo1 training Matches (e.g., the first Event), `w = [0, 0, 0, 0]` and the signal outputs the anchor probability so Bo1 coverage is still measured. If promoted under `signal-lab/3`, the four features enter forward selection through `FittedSignal::features` in the lab's `Combiner`.

## Bo1 coverage and research-archive run (`signal-lab/3`)

Read-only offline run over `D:/Projects/KallBet/target/research-archive` (SHA-256 `a556cc446e2f5e75d4494b0fee8b4240e58e8147b3954e23cd959d5cdc0c7c8a`, `15,405` files, `62` Events, `342` Bo1 Matches, `1,970` complete HLTV vetoes), seed `33`, `2,000` Event resamples, 90% intervals:

```sh
cargo run --release -p kallbet-ai -- lab --archive D:/Projects/KallBet/target/research-archive --signal bo1_pool_depth --signal bo1_veto_sim --out target/t62-bo1-pool-depth-report --seed 33 --catch-up-trials 8 --pickem-simulations 20
```

| Candidate Signal | Verdict | Computed / Bo1s | Paired | Events | Δ log-loss (90% CI) | Δ Brier | Δ accuracy (90% CI) | Market subset (n, Δ log-loss) |
|---|---|---:|---:|---:|---|---:|---|---|
| `bo1_pool_depth` | `no evidence (added after list fixed)` | 312 / 342 (91.2%) | 312 | 14 | −0.0240 [−0.0413, −0.0077] | −0.0052 | +0.32 pp [−1.32, +1.50] | 49, +0.0212 |
| `bo1_veto_sim` | `promote` | 323 / 342 (94.4%) | 319 | 13 | −0.0877 [−0.1315, −0.0342] | −0.0173 | +0.94 pp [−0.88, +2.80] | 54, −0.0025 |

- On the 312 paired Bo1 Matches: anchor log-loss `0.8020` → `0.7780`, Brier `0.2698` → `0.2647`, accuracy `61.2%` → `61.5%`, detectable log-loss effect `0.0256`.
- Verdict reason under `signal-lab/3`: *"better overall, but worse on the Matches with a market line (+0.0212)"*.
- Secondary `s`-tier slice: 217 paired Bo1 Matches across 6 Events, Δ log-loss `−0.0138` (90% CI `[−0.0266, +0.0071]`), `no evidence`.
- Final fitted weights on 312 Bo1 training Matches: `pool_depth_diff = −0.2269`, `first_ban_stability_diff = −0.3134`, `pool_overlap_x_anchor = +0.8099`, `first_ban_disruption_diff = +0.3861`.
- Holdout: pending.

