# Signal Lab report

Archive SHA-256 `25459d150e1fac87409e38d17a73ab2110444401703a056b1c4d4c846c4df782` (272 files), protocol `signal-lab/1`, seed 33, 2000 Event resamples, 90% intervals, floor 100 paired Matches.

Baseline: the anchor: HybridAnchored without LLM, 0.80 market and 0.20 Valve rating logit (`debate::base`). Each Candidate Signal is fitted, for each Event, only on the Matches of the Events that ended before it began, and every datum it reads is observed before the Match cutoff. Differences are variant − anchor on the Matches both predicted (log-loss and Brier: below zero is better). The interval resamples whole Events. Verdict: `promote` when the interval of the log-loss difference is below zero and the Matches with a market line are not worse; `harmful` when it is above zero; `no evidence` otherwise or under the floor; `not testable` when the data could not be had. "Detectable" is the smallest log-loss difference these Matches would show with 80% probability: an effect smaller than it can hide as `no evidence`.

| Candidate Signal | Verdict | Computed on | Paired | Events | Δ log-loss | 90% interval | Detectable | Δ Brier | Δ accuracy | Market subset (n, Δ log-loss) |
|---|---|---|---|---|---|---|---|---|---|---|
| `rating_calibration` | no evidence | 342 | 342 | 4 | -0.1066 | [-0.1779, +0.0037] | 0.1274 | -0.0230 | +0.0058 | 112, -0.0005 |
| `own_rating` | no evidence | 441 | 384 | 5 | -0.0694 | [-0.1539, +0.0262] | 0.1261 | -0.0049 | -0.0703 | 103, +0.0020 |
| `recent_form` | no evidence | 262 | 262 | 4 | +0.0018 | [-0.0004, +0.0045] | 0.0040 | +0.0005 | +0.0000 | 81, +0.0009 |
| `head_to_head` | no evidence | 64 | 64 | 3 | +0.0058 | [-0.0057, +0.0164] | 0.0188 | +0.0014 | -0.0156 | 32, -0.0042 |
| `llm_adjustment` | not testable | 0 | 0 | 0 | — | — | — | — | — | — |

## `rating_calibration`: no evidence

Valve rating's map probability rescaled in logit space by a factor fitted per series format, then blended with the market as the anchor does.

Verdict: the interval of the log-loss difference includes zero.

| On the 342 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.7696 | 0.2550 | 61.4% |
| rating_calibration | 0.6629 | 0.2320 | 62.0% |

| Event | Fitted on | Computed on | Paired | Δ log-loss |
|---|---|---|---|---|
| PGL Major Copenhagen 2024 | 0 | 0 | 0 | — |
| Perfect World Major Shanghai Major 2024 | 73 | 0 | 0 | — |
| BLAST.tv Major Austin 2025 | 146 | 106 | 106 | -0.2052 |
| StarLadder Major Budapest 2025 | 252 | 106 | 106 | -0.1506 |
| IEM Cologne Major 2026 | 358 | 106 | 106 | +0.0146 |
| PGL Major Singapore 2026 | 464 | 24 | 24 | -0.0124 |

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

| Event | Fitted on | Computed on | Paired | Δ log-loss |
|---|---|---|---|---|
| PGL Major Copenhagen 2024 | 0 | 57 | 0 | — |
| Perfect World Major Shanghai Major 2024 | 73 | 66 | 66 | +0.0490 |
| BLAST.tv Major Austin 2025 | 146 | 96 | 96 | -0.2216 |
| StarLadder Major Budapest 2025 | 252 | 102 | 102 | -0.1114 |
| IEM Cologne Major 2026 | 358 | 99 | 99 | +0.0333 |
| PGL Major Singapore 2026 | 464 | 21 | 21 | -0.0251 |

## `recent_form`: no evidence

Recent results over the last 45 days (14-day half-life) weighted by opponent strength from the point-in-time Glicko-2 rating, added as a fitted logit offset to the anchor.

Verdict: the interval of the log-loss difference includes zero.

| On the 262 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.7333 | 0.2432 | 62.6% |
| recent_form | 0.7351 | 0.2436 | 62.6% |

| Event | Fitted on | Computed on | Paired | Δ log-loss |
|---|---|---|---|---|
| PGL Major Copenhagen 2024 | 0 | 0 | 0 | — |
| Perfect World Major Shanghai Major 2024 | 73 | 0 | 0 | — |
| BLAST.tv Major Austin 2025 | 146 | 82 | 82 | -0.0009 |
| StarLadder Major Budapest 2025 | 252 | 82 | 82 | +0.0059 |
| IEM Cologne Major 2026 | 358 | 82 | 82 | +0.0002 |
| PGL Major Singapore 2026 | 464 | 16 | 16 | +0.0018 |

## `head_to_head`: no evidence

Recency-weighted head-to-head series record between the two teams (180-day half-life, shrunk toward even), added as a fitted logit offset to the anchor.

Verdict: computed on 64 paired Matches, under the floor of 100.

| On the 64 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.6626 | 0.2293 | 62.5% |
| head_to_head | 0.6684 | 0.2307 | 60.9% |

| Event | Fitted on | Computed on | Paired | Δ log-loss |
|---|---|---|---|---|
| PGL Major Copenhagen 2024 | 0 | 0 | 0 | — |
| Perfect World Major Shanghai Major 2024 | 73 | 0 | 0 | — |
| BLAST.tv Major Austin 2025 | 146 | 0 | 0 | — |
| StarLadder Major Budapest 2025 | 252 | 30 | 30 | +0.0177 |
| IEM Cologne Major 2026 | 358 | 27 | 27 | -0.0072 |
| PGL Major Singapore 2026 | 464 | 7 | 7 | +0.0054 |

## `llm_adjustment`: not testable

The three-seat debate's adjustment (HybridAnchored, debate mode `always`, the first seating) on the Matches with a roster flag, from recorded answers.

Verdict: no LLM answers in this run; produce them with `cargo run -p kallbet-ai -- lab --llm <LLM_FILE> --answers <ANSWERS_FILE> --budget claude=<RUNS> --budget codex=<RUNS> --budget antigravity=<RUNS>`.

| Event | Fitted on | Computed on | Paired | Δ log-loss |
|---|---|---|---|---|
| PGL Major Copenhagen 2024 | 0 | 0 | 0 | — |
| Perfect World Major Shanghai Major 2024 | 73 | 0 | 0 | — |
| BLAST.tv Major Austin 2025 | 146 | 0 | 0 | — |
| StarLadder Major Budapest 2025 | 252 | 0 | 0 | — |
| IEM Cologne Major 2026 | 358 | 0 | 0 | — |
| PGL Major Singapore 2026 | 464 | 0 | 0 | — |

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
