# Signal Lab report

Archive SHA-256 `25459d150e1fac87409e38d17a73ab2110444401703a056b1c4d4c846c4df782` (272 files), protocol `signal-lab/2`, seed 33, 2000 Event resamples, 90% intervals, floor 100 paired Matches.

Baseline: the anchor: HybridAnchored without LLM, 0.80 market and 0.20 Valve rating logit (`debate::base`). Each Candidate Signal is fitted, for each Event, only on the Matches of the Events that ended before it began, and every datum it reads is observed before the Match cutoff. Differences are variant − anchor on the Matches both predicted (log-loss and Brier: below zero is better). The interval resamples whole Events. Verdict: `promote` when the interval of the log-loss difference is below zero and the Matches with a market line are not worse; `harmful` when it is above zero; `no evidence` otherwise or under the floor; `not testable` when the data could not be had. "Detectable" is the smallest log-loss difference these Matches would show with 80% probability: an effect smaller than it can hide as `no evidence`.

| Candidate Signal | Verdict | Computed on | Paired | Events | Δ log-loss | 90% interval | Detectable | Δ Brier | Δ accuracy | Market subset (n, Δ log-loss) |
|---|---|---|---|---|---|---|---|---|---|---|
| `rating_calibration` | no evidence | 342 | 342 | 4 | -0.1066 | [-0.1779, +0.0037] | 0.1274 | -0.0230 | +0.0058 | 112, -0.0005 |
| `learned_weights` | no evidence | 415 | 415 | 5 | -0.0783 | [-0.1361, +0.0051] | 0.0975 | -0.0167 | +0.0000 | 112, +0.0020 |
| `market_source_pinnacle` | no evidence | 19 | 19 | 1 | +0.0000 | [+0.0000, +0.0000] | 0.0000 | +0.0000 | +0.0000 | 19, +0.0000 |
| `market_source_polymarket` | no evidence | 19 | 19 | 1 | -0.0003 | [-0.0003, -0.0003] | 0.0000 | -0.0001 | +0.0000 | 19, -0.0003 |
| `market_source_kalshi` | no evidence | 19 | 19 | 1 | -0.0049 | [-0.0049, -0.0049] | 0.0000 | -0.0024 | +0.0526 | 19, -0.0049 |
| `market_source_average` | no evidence | 19 | 19 | 1 | -0.0018 | [-0.0018, -0.0018] | 0.0000 | -0.0009 | +0.0526 | 19, -0.0018 |
| `line_movement` | no evidence | 24 | 24 | 1 | -0.0349 | [-0.0349, -0.0349] | 0.0000 | -0.0149 | +0.0417 | 24, -0.0349 |
| `favourite_longshot` | no evidence | 24 | 24 | 1 | +0.0441 | [+0.0441, +0.0441] | 0.0000 | +0.0169 | +0.0000 | 24, +0.0441 |
| `own_rating` | no evidence | 441 | 384 | 5 | -0.0694 | [-0.1539, +0.0262] | 0.1261 | -0.0049 | -0.0703 | 103, +0.0020 |
| `recent_form` | no evidence | 262 | 262 | 4 | +0.0018 | [-0.0004, +0.0045] | 0.0040 | +0.0005 | +0.0000 | 81, +0.0009 |
| `head_to_head` | no evidence | 64 | 64 | 3 | +0.0058 | [-0.0057, +0.0164] | 0.0188 | +0.0014 | -0.0156 | 32, -0.0042 |
| `llm_adjustment` | not testable | 0 | 0 | 0 | — | — | — | — | — | — |
| `lineup_strength` | not testable | 0 | 0 | 0 | — | — | — | — | — | — |
| `player_form` | not testable | 0 | 0 | 0 | — | — | — | — | — | — |
| `per_map_rating` | not testable | 0 | 0 | 0 | — | — | — | — | — | — |
| `bo1_veto_sim` | not testable | 0 | 0 | 0 | — | — | — | — | — | — |
| `roster_stand_in` | no evidence | 203 | 203 | 3 | -0.0008 | [-0.0040, +0.0068] | 0.0112 | -0.0005 | +0.0099 | 112, -0.0004 |
| `roster_change_30d` | no evidence | 14 | 14 | 2 | -0.0711 | [-0.3319, +0.0000] | 0.3128 | -0.0345 | +0.0714 | 13, -0.0766 |
| `roster_stability` | no evidence | 115 | 115 | 2 | +0.0066 | [+0.0000, +0.0362] | 0.0346 | +0.0031 | -0.0087 | 100, +0.0076 |
| `roster_coach_change` | no evidence | 69 | 69 | 2 | -0.0074 | [-0.0243, +0.0000] | 0.0220 | -0.0030 | +0.0000 | 62, -0.0082 |
| `rest_schedule_load` | no evidence | 318 | 318 | 4 | -0.0039 | [-0.0125, +0.0065] | 0.0141 | +0.0003 | +0.0000 | 103, +0.0090 |
| `stakes` | no evidence | 342 | 342 | 4 | -0.0336 | [-0.0593, -0.0009] | 0.0481 | -0.0053 | +0.0000 | 112, +0.0086 |
| `lan_online` | no evidence | 120 | 120 | 2 | -0.0000 | [-0.0002, +0.0000] | 0.0003 | -0.0000 | +0.0000 | 103, -0.0001 |
| `series_format` | no evidence | 254 | 254 | 4 | +0.0013 | [-0.0001, +0.0027] | 0.0022 | +0.0005 | +0.0000 | 87, +0.0009 |
| `region_travel` | not testable | 0 | 0 | 0 | — | — | — | — | — | — |

## `rating_calibration`: no evidence

Valve rating's map probability rescaled in logit space by a factor fitted per series format, then blended with the market as the anchor does.

Verdict: the interval of the log-loss difference includes zero.

| On the 342 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.7696 | 0.2550 | 61.4% |
| rating_calibration | 0.6629 | 0.2320 | 62.0% |

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| PGL Major Copenhagen 2024 | 0 | 0 | 0 | — | — |
| Perfect World Major Shanghai Major 2024 | 73 | 0 | 0 | — | — |
| BLAST.tv Major Austin 2025 | 146 | 106 | 106 | -0.2052 | — |
| StarLadder Major Budapest 2025 | 252 | 106 | 106 | -0.1506 | — |
| IEM Cologne Major 2026 | 358 | 106 | 106 | +0.0146 | — |
| PGL Major Singapore 2026 | 464 | 24 | 24 | -0.0124 | — |

## `learned_weights`: no evidence

the anchor's market and Valve rating weights (logit space) learned by the combiner on the Events that ended before, one set per availability of the two inputs, regularised towards 0.80 / 0.20.

Verdict: the interval of the log-loss difference includes zero.

| On the 415 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.7484 | 0.2485 | 62.2% |
| learned_weights | 0.6701 | 0.2318 | 62.2% |

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| PGL Major Copenhagen 2024 | 0 | 0 | 0 | — | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":0,"rating":1.0}}` |
| Perfect World Major Shanghai Major 2024 | 73 | 73 | 73 | -0.0000 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":0,"rating":1.0}}` |
| BLAST.tv Major Austin 2025 | 146 | 106 | 106 | -0.1701 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":73,"rating":0.608448}}` |
| StarLadder Major Budapest 2025 | 252 | 106 | 106 | -0.1489 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":179,"rating":0.329579}}` |
| IEM Cologne Major 2026 | 358 | 106 | 106 | +0.0102 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":285,"rating":0.317669}}` |
| PGL Major Singapore 2026 | 464 | 24 | 24 | +0.0093 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.948886,"matches":88,"rating":0.131999},"rating":{"matches":303,"rating":0.330835}}` |

## `market_source_pinnacle`: no evidence

the anchor with Pinnacle's last price before the cutoff as the market, on Matches every source priced.

Verdict: computed on 19 paired Matches, under the floor of 100.

| On the 19 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.7797 | 0.2923 | 42.1% |
| market_source_pinnacle | 0.7797 | 0.2923 | 42.1% |

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| PGL Major Copenhagen 2024 | 0 | 0 | 0 | — | — |
| Perfect World Major Shanghai Major 2024 | 73 | 0 | 0 | — | — |
| BLAST.tv Major Austin 2025 | 146 | 0 | 0 | — | — |
| StarLadder Major Budapest 2025 | 252 | 0 | 0 | — | — |
| IEM Cologne Major 2026 | 358 | 0 | 0 | — | — |
| PGL Major Singapore 2026 | 464 | 19 | 19 | +0.0000 | — |

## `market_source_polymarket`: no evidence

the anchor with Polymarket's last price before the cutoff as the market, on Matches every source priced.

Verdict: computed on 19 paired Matches, under the floor of 100.

| On the 19 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.7797 | 0.2923 | 42.1% |
| market_source_polymarket | 0.7794 | 0.2921 | 42.1% |

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| PGL Major Copenhagen 2024 | 0 | 0 | 0 | — | — |
| Perfect World Major Shanghai Major 2024 | 73 | 0 | 0 | — | — |
| BLAST.tv Major Austin 2025 | 146 | 0 | 0 | — | — |
| StarLadder Major Budapest 2025 | 252 | 0 | 0 | — | — |
| IEM Cologne Major 2026 | 358 | 0 | 0 | — | — |
| PGL Major Singapore 2026 | 464 | 19 | 19 | -0.0003 | — |

## `market_source_kalshi`: no evidence

the anchor with Kalshi's last price before the cutoff as the market, on Matches every source priced.

Verdict: computed on 19 paired Matches, under the floor of 100.

| On the 19 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.7797 | 0.2923 | 42.1% |
| market_source_kalshi | 0.7748 | 0.2899 | 47.4% |

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| PGL Major Copenhagen 2024 | 0 | 0 | 0 | — | — |
| Perfect World Major Shanghai Major 2024 | 73 | 0 | 0 | — | — |
| BLAST.tv Major Austin 2025 | 146 | 0 | 0 | — | — |
| StarLadder Major Budapest 2025 | 252 | 0 | 0 | — | — |
| IEM Cologne Major 2026 | 358 | 0 | 0 | — | — |
| PGL Major Singapore 2026 | 464 | 19 | 19 | -0.0049 | — |

## `market_source_average`: no evidence

the anchor with the mean of Pinnacle, Polymarket and Kalshi as the market, on Matches every source priced.

Verdict: computed on 19 paired Matches, under the floor of 100.

| On the 19 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.7797 | 0.2923 | 42.1% |
| market_source_average | 0.7779 | 0.2914 | 47.4% |

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| PGL Major Copenhagen 2024 | 0 | 0 | 0 | — | — |
| Perfect World Major Shanghai Major 2024 | 73 | 0 | 0 | — | — |
| BLAST.tv Major Austin 2025 | 146 | 0 | 0 | — | — |
| StarLadder Major Budapest 2025 | 252 | 0 | 0 | — | — |
| IEM Cologne Major 2026 | 358 | 0 | 0 | — | — |
| PGL Major Singapore 2026 | 464 | 19 | 19 | -0.0018 | — |

## `line_movement`: no evidence

the anchor's logit moved by how the line moved (closing against opening, and closing against six hours before the cutoff), coefficients fitted walk-forward.

Verdict: computed on 24 paired Matches, under the floor of 100.

| On the 24 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.7473 | 0.2753 | 50.0% |
| line_movement | 0.7123 | 0.2604 | 54.2% |

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| PGL Major Copenhagen 2024 | 0 | 0 | 0 | — | — |
| Perfect World Major Shanghai Major 2024 | 73 | 0 | 0 | — | — |
| BLAST.tv Major Austin 2025 | 146 | 0 | 0 | — | — |
| StarLadder Major Budapest 2025 | 252 | 0 | 0 | — | — |
| IEM Cologne Major 2026 | 358 | 0 | 0 | — | — |
| PGL Major Singapore 2026 | 464 | 24 | 24 | -0.0349 | — |

## `favourite_longshot`: no evidence

the market's price rescaled in logit space by a factor fitted walk-forward on earlier Events (favourite-longshot bias), then blended as the anchor does.

Verdict: computed on 24 paired Matches, under the floor of 100.

| On the 24 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.7473 | 0.2753 | 50.0% |
| favourite_longshot | 0.7914 | 0.2923 | 50.0% |

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| PGL Major Copenhagen 2024 | 0 | 0 | 0 | — | — |
| Perfect World Major Shanghai Major 2024 | 73 | 0 | 0 | — | — |
| BLAST.tv Major Austin 2025 | 146 | 0 | 0 | — | — |
| StarLadder Major Budapest 2025 | 252 | 0 | 0 | — | — |
| IEM Cologne Major 2026 | 358 | 0 | 0 | — | — |
| PGL Major Singapore 2026 | 464 | 24 | 24 | +0.0441 | — |

## `own_rating`: no evidence

Glicko-2 team rating updated after every Match finished before the cutoff (teams joined by team_key), with format-calibrated map scale blended with the market as the anchor does.

Verdict: the interval of the log-loss difference includes zero.

| On the 384 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.7458 | 0.2468 | 62.5% |
| own_rating | 0.6764 | 0.2419 | 55.5% |

### Direct comparison with Valve rating (384 paired Matches, 5 Events)

Reference: Valve Regional Standings (unblended). Δ log-loss -0.0846 (90% interval [-0.1626, -0.0056], detectable 0.1141), Δ Brier -0.0090, Δ accuracy -0.0677.

| On the 384 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| Valve rating | 0.7743 | 0.2572 | 61.5% |
| own_rating | 0.6897 | 0.2483 | 54.7% |

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| PGL Major Copenhagen 2024 | 0 | 57 | 0 | — | — |
| Perfect World Major Shanghai Major 2024 | 73 | 66 | 66 | +0.0490 | — |
| BLAST.tv Major Austin 2025 | 146 | 96 | 96 | -0.2216 | — |
| StarLadder Major Budapest 2025 | 252 | 102 | 102 | -0.1114 | — |
| IEM Cologne Major 2026 | 358 | 99 | 99 | +0.0333 | — |
| PGL Major Singapore 2026 | 464 | 21 | 21 | -0.0251 | — |

## `recent_form`: no evidence

Recent results over the last 45 days (14-day half-life) weighted by opponent strength from the point-in-time Glicko-2 rating, added as a fitted logit offset to the anchor.

Verdict: the interval of the log-loss difference includes zero.

| On the 262 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.7333 | 0.2432 | 62.6% |
| recent_form | 0.7351 | 0.2436 | 62.6% |

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| PGL Major Copenhagen 2024 | 0 | 0 | 0 | — | — |
| Perfect World Major Shanghai Major 2024 | 73 | 0 | 0 | — | — |
| BLAST.tv Major Austin 2025 | 146 | 82 | 82 | -0.0009 | — |
| StarLadder Major Budapest 2025 | 252 | 82 | 82 | +0.0059 | — |
| IEM Cologne Major 2026 | 358 | 82 | 82 | +0.0002 | — |
| PGL Major Singapore 2026 | 464 | 16 | 16 | +0.0018 | — |

## `head_to_head`: no evidence

Recency-weighted head-to-head series record between the two teams (180-day half-life, shrunk toward even), added as a fitted logit offset to the anchor.

Verdict: computed on 64 paired Matches, under the floor of 100.

| On the 64 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.6626 | 0.2293 | 62.5% |
| head_to_head | 0.6684 | 0.2307 | 60.9% |

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| PGL Major Copenhagen 2024 | 0 | 0 | 0 | — | — |
| Perfect World Major Shanghai Major 2024 | 73 | 0 | 0 | — | — |
| BLAST.tv Major Austin 2025 | 146 | 0 | 0 | — | — |
| StarLadder Major Budapest 2025 | 252 | 30 | 30 | +0.0177 | — |
| IEM Cologne Major 2026 | 358 | 27 | 27 | -0.0072 | — |
| PGL Major Singapore 2026 | 464 | 7 | 7 | +0.0054 | — |

## `llm_adjustment`: not testable

The three-seat debate's adjustment (HybridAnchored, debate mode `always`, the first seating) on the Matches with a roster flag, from recorded answers.

Verdict: no LLM answers in this run; produce them with `cargo run -p kallbet-ai -- lab --llm <LLM_FILE> --answers <ANSWERS_FILE> --budget claude=<RUNS> --budget codex=<RUNS> --budget antigravity=<RUNS>`.

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| PGL Major Copenhagen 2024 | 0 | 0 | 0 | — | — |
| Perfect World Major Shanghai Major 2024 | 73 | 0 | 0 | — | — |
| BLAST.tv Major Austin 2025 | 146 | 0 | 0 | — | — |
| StarLadder Major Budapest 2025 | 252 | 0 | 0 | — | — |
| IEM Cologne Major 2026 | 358 | 0 | 0 | — | — |
| PGL Major Singapore 2026 | 464 | 0 | 0 | — | — |

## `lineup_strength`: not testable

Mean HLTV player rating of the 5 playing starters from the pre-cutoff Liquipedia roster (90-day half-life, shrunk toward prior 1.00 with weight 6 maps; debutants/unknown stand-ins take the 1.00 prior when at least 3 of 5 starters on each team have HLTV history, else uncovered), added as a fitted logit offset to the anchor.

Verdict: the archive holds no HLTV player statistics records (hltv/), so lineup strength cannot be evaluated.

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| PGL Major Copenhagen 2024 | 0 | 0 | 0 | — | — |
| Perfect World Major Shanghai Major 2024 | 73 | 0 | 0 | — | — |
| BLAST.tv Major Austin 2025 | 146 | 0 | 0 | — | — |
| StarLadder Major Budapest 2025 | 252 | 0 | 0 | — | — |
| IEM Cologne Major 2026 | 358 | 0 | 0 | — | — |
| PGL Major Singapore 2026 | 464 | 0 | 0 | — | — |

## `player_form`: not testable

Mean recent-minus-longer-run HLTV rating form of the 5 playing starters from the pre-cutoff Liquipedia roster (30d/14d half-life vs 180d/90d half-life, shrunk toward neutral 0.0 with weight 4 maps; debutants/players without 2+ baseline & recent Matches take 0.0 when at least 3 of 5 starters on each team have form, else uncovered), added as a fitted logit offset to the anchor.

Verdict: the archive holds no HLTV player statistics records (hltv/), so player form cannot be evaluated.

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| PGL Major Copenhagen 2024 | 0 | 0 | 0 | — | — |
| Perfect World Major Shanghai Major 2024 | 73 | 0 | 0 | — | — |
| BLAST.tv Major Austin 2025 | 146 | 0 | 0 | — | — |
| StarLadder Major Budapest 2025 | 252 | 0 | 0 | — | — |
| IEM Cologne Major 2026 | 358 | 0 | 0 | — | — |
| PGL Major Singapore 2026 | 464 | 0 | 0 | — | — |

## `per_map_rating`: not testable

Per-map Elo from completed Liquipedia series, averaged over equally likely map sets and blended with the market.

Verdict: No played Liquipedia maps linked to completed PandaScore Matches in the archive.

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| PGL Major Copenhagen 2024 | 0 | 0 | 0 | — | — |
| Perfect World Major Shanghai Major 2024 | 73 | 0 | 0 | — | — |
| BLAST.tv Major Austin 2025 | 146 | 0 | 0 | — | — |
| StarLadder Major Budapest 2025 | 252 | 0 | 0 | — | — |
| IEM Cologne Major 2026 | 358 | 0 | 0 | — | — |
| PGL Major Singapore 2026 | 464 | 0 | 0 | — | — |

## `bo1_veto_sim`: not testable

Bo1 ban simulation from past HLTV vetoes and map results, blended with the market.

Verdict: No complete HLTV Bo1 vetoes linked to finished PandaScore Matches in the archive.

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| PGL Major Copenhagen 2024 | 0 | 0 | 0 | — | — |
| Perfect World Major Shanghai Major 2024 | 73 | 0 | 0 | — | — |
| BLAST.tv Major Austin 2025 | 146 | 0 | 0 | — | — |
| StarLadder Major Budapest 2025 | 252 | 0 | 0 | — | — |
| IEM Cologne Major 2026 | 358 | 0 | 0 | — | — |
| PGL Major Singapore 2026 | 464 | 0 | 0 | — | — |

## `roster_stand_in`: no evidence

A stand-in in the active five, from the pre-cutoff Liquipedia revision..

Verdict: the interval of the log-loss difference includes zero.

| On the 203 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.6319 | 0.2179 | 64.5% |
| roster_stand_in | 0.6311 | 0.2174 | 65.5% |

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| PGL Major Copenhagen 2024 | 0 | 0 | 0 | — | — |
| Perfect World Major Shanghai Major 2024 | 73 | 73 | 73 | +0.0000 | — |
| BLAST.tv Major Austin 2025 | 146 | 0 | 0 | — | — |
| StarLadder Major Budapest 2025 | 252 | 0 | 0 | — | — |
| IEM Cologne Major 2026 | 358 | 106 | 106 | -0.0054 | — |
| PGL Major Singapore 2026 | 464 | 24 | 24 | +0.0171 | — |

## `roster_change_30d`: no evidence

An active-five change bounded to the last 30 days by dated lineup observations..

Verdict: computed on 14 paired Matches, under the floor of 100.

| On the 14 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.5978 | 0.2101 | 57.1% |
| roster_change_30d | 0.5267 | 0.1757 | 64.3% |

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| PGL Major Copenhagen 2024 | 0 | 0 | 0 | — | — |
| Perfect World Major Shanghai Major 2024 | 73 | 0 | 0 | — | — |
| BLAST.tv Major Austin 2025 | 146 | 0 | 0 | — | — |
| StarLadder Major Budapest 2025 | 252 | 0 | 0 | — | — |
| IEM Cologne Major 2026 | 358 | 11 | 11 | +0.0000 | — |
| PGL Major Singapore 2026 | 464 | 3 | 3 | -0.3319 | — |

## `roster_stability`: no evidence

Days since the current five was first observed after a different five..

Verdict: the interval of the log-loss difference includes zero.

| On the 115 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.6230 | 0.2177 | 63.5% |
| roster_stability | 0.6296 | 0.2208 | 62.6% |

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| PGL Major Copenhagen 2024 | 0 | 0 | 0 | — | — |
| Perfect World Major Shanghai Major 2024 | 73 | 0 | 0 | — | — |
| BLAST.tv Major Austin 2025 | 146 | 0 | 0 | — | — |
| StarLadder Major Budapest 2025 | 252 | 0 | 0 | — | — |
| IEM Cologne Major 2026 | 358 | 94 | 94 | +0.0000 | — |
| PGL Major Singapore 2026 | 464 | 21 | 21 | +0.0362 | — |

## `roster_coach_change`: no evidence

An emergency coach substitute or a change from the previously observed active coach set..

Verdict: computed on 69 paired Matches, under the floor of 100.

| On the 69 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.6717 | 0.2397 | 58.0% |
| roster_coach_change | 0.6643 | 0.2367 | 58.0% |

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| PGL Major Copenhagen 2024 | 0 | 0 | 0 | — | — |
| Perfect World Major Shanghai Major 2024 | 73 | 0 | 0 | — | — |
| BLAST.tv Major Austin 2025 | 146 | 0 | 0 | — | — |
| StarLadder Major Budapest 2025 | 252 | 0 | 0 | — | — |
| IEM Cologne Major 2026 | 358 | 48 | 48 | +0.0000 | — |
| PGL Major Singapore 2026 | 464 | 21 | 21 | -0.0243 | — |

## `rest_schedule_load`: no evidence

Team differences in rest, Matches and maps in the preceding 48 hours, and a second Match on the same UTC day.

Verdict: the interval of the log-loss difference includes zero.

| On the 318 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.7670 | 0.2539 | 61.3% |
| rest_schedule_load | 0.7630 | 0.2542 | 61.3% |

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| PGL Major Copenhagen 2024 | 0 | 0 | 0 | — | — |
| Perfect World Major Shanghai Major 2024 | 73 | 0 | 0 | — | — |
| BLAST.tv Major Austin 2025 | 146 | 96 | 96 | -0.0155 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[-0.09372474094717755,-0.0030431316368061175,-0.02507012692978926,0.0]}` |
| StarLadder Major Budapest 2025 | 252 | 102 | 102 | -0.0075 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[-0.237982022292172,0.2758352617064106,0.04301966877458919,-0.10565986671777168]}` |
| IEM Cologne Major 2026 | 358 | 99 | 99 | +0.0098 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[-0.24331508418462305,0.19225605235362733,-0.05342903507826699,-0.10567688199224834]}` |
| PGL Major Singapore 2026 | 464 | 21 | 21 | +0.0014 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[-0.15936026141570886,0.23859314892808786,0.07921324429061659,-0.10540798424237699]}` |

## `stakes`: no evidence

Pre-round Swiss records, advancement and elimination Matches, and Playoffs rounds.

Verdict: better overall, but worse on the Matches with a market line (+0.0086).

| On the 342 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.7696 | 0.2550 | 61.4% |
| stakes | 0.7360 | 0.2497 | 61.4% |

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| PGL Major Copenhagen 2024 | 0 | 0 | 0 | — | — |
| Perfect World Major Shanghai Major 2024 | 73 | 0 | 0 | — | — |
| BLAST.tv Major Austin 2025 | 146 | 106 | 106 | -0.0649 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[0.03503880224435155,-0.21032204809400631,0.0962303208744153,-0.1645373243367784,0.0,-0.12046679772306142,-0.2304432498571695,0.025935243342570934]}` |
| StarLadder Major Budapest 2025 | 252 | 106 | 106 | -0.0536 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.3059621320474534,-0.2582393250178,0.2124915787572432,0.009474477161904875,0.0,-0.20632640528260016,-0.19257173283961138,0.03847993893064559]}` |
| IEM Cologne Major 2026 | 358 | 106 | 106 | +0.0167 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.2994936079963877,-0.2980324611912847,0.3080874202441326,0.07812472277435409,0.0,-0.5227806166667834,-0.23722003884365658,0.04419534972045481]}` |
| PGL Major Singapore 2026 | 464 | 24 | 24 | -0.0285 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.2928661757790597,-0.25998213154628275,0.35070873831341143,0.006946411873897213,0.0,-0.492296525492899,-0.24472937590952007,0.06577503323092188]}` |

## `lan_online`: no evidence

Team win rates in the current environment, with separate weights for LAN and online.

Verdict: the interval of the log-loss difference includes zero.

| On the 120 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.6197 | 0.2168 | 64.2% |
| lan_online | 0.6197 | 0.2168 | 64.2% |

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| PGL Major Copenhagen 2024 | 0 | 0 | 0 | — | — |
| Perfect World Major Shanghai Major 2024 | 73 | 0 | 0 | — | — |
| BLAST.tv Major Austin 2025 | 146 | 0 | 0 | — | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.03681690121195903,0.0]}` |
| StarLadder Major Budapest 2025 | 252 | 0 | 0 | — | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.03681690121195903,0.0]}` |
| IEM Cologne Major 2026 | 358 | 99 | 99 | +0.0000 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.03681690121195903,0.0]}` |
| PGL Major Singapore 2026 | 464 | 21 | 21 | -0.0002 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.029475172721946642,0.0]}` |

## `series_format`: no evidence

Team differences in Bo1 versus Bo3 performance, conditioned on the current format.

Verdict: the interval of the log-loss difference includes zero.

| On the 254 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.7720 | 0.2520 | 61.4% |
| series_format | 0.7732 | 0.2525 | 61.4% |

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| PGL Major Copenhagen 2024 | 0 | 0 | 0 | — | — |
| Perfect World Major Shanghai Major 2024 | 73 | 0 | 0 | — | — |
| BLAST.tv Major Austin 2025 | 146 | 70 | 70 | -0.0004 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[-0.005298534126694724,0.016332395086951893]}` |
| StarLadder Major Budapest 2025 | 252 | 88 | 88 | +0.0033 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[-0.14783608343067722,0.14096686874673678]}` |
| IEM Cologne Major 2026 | 358 | 81 | 81 | +0.0005 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[-0.04465918655513589,0.06216679918861895]}` |
| PGL Major Singapore 2026 | 464 | 15 | 15 | +0.0012 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[-0.0628498665391053,-0.00961728700069253]}` |

## `region_travel`: not testable

Difference in playing a LAN outside the team's published VRS qualifying region.

Verdict: computed on no Match: it needs a dated LAN host region and dated VRS qualifying regions for both teams, plus 20 anchored training Matches.

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| PGL Major Copenhagen 2024 | 0 | 0 | 0 | — | — |
| Perfect World Major Shanghai Major 2024 | 73 | 0 | 0 | — | — |
| BLAST.tv Major Austin 2025 | 146 | 0 | 0 | — | — |
| StarLadder Major Budapest 2025 | 252 | 0 | 0 | — | — |
| IEM Cologne Major 2026 | 358 | 0 | 0 | — | — |
| PGL Major Singapore 2026 | 464 | 0 | 0 | — | `{"features":["away_region"],"weights":[0.06862636822015585]}` |

## Forward selection

The Candidate Signals marked `promote` enter one combination, one at a time: at each step the one that lowers the walk-forward log-loss most (on the 415 training-side Matches the anchor predicted) enters, and selection stops when none lowers it. The combination is the combiner (a regularised logistic model without intercept, penalty 10 towards the anchor's weights) over the logits of the market line and the rating plus each member's features, refitted for every Event on the Events that ended before it. "Added" is the log-loss after the step − before it (the anchor before the first).

No Candidate Signal is marked `promote`: nothing enters, and the frozen combination is the anchor.

## Holdout: pending

The Holdout is every Stage that begins at or after 2026-11-25 23:46:46 UTC, or whose Event has a result at or after it (except PandaScore tournament 22378, kept on the training side). No fitting and no selection step reads it; the frozen combination is evaluated on it once, last.

The Holdout is pending: no Holdout Match has been played in this archive, so the frozen combination was not evaluated.

## Events (walk-forward order)

| Event | Begins | Ends | Scored Matches |
|---|---|---|---|
| PGL Major Copenhagen 2024 | 2024-03-17 12:00 | 2024-03-31 20:37 | 73 |
| Perfect World Major Shanghai Major 2024 | 2024-11-30 03:00 | 2024-12-15 12:23 | 73 |
| BLAST.tv Major Austin 2025 | 2025-06-03 15:15 | 2025-06-22 22:15 | 106 |
| StarLadder Major Budapest 2025 | 2025-11-24 12:00 | 2025-12-14 21:14 | 106 |
| IEM Cologne Major 2026 | 2026-06-02 10:30 | 2026-06-21 18:06 | 106 |
| PGL Major Singapore 2026 | 2026-11-24 04:00 | 2026-11-25 13:10 | 24 |

[Catch-up mode in simulated Leagues](catch-up.md)

[LLM track: single agent, debate and seatings](llm.md)

[Pick'em variants in Pick'em Points](pickem.md)
