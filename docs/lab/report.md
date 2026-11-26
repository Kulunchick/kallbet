# Signal Lab report

Archive SHA-256 `25459d150e1fac87409e38d17a73ab2110444401703a056b1c4d4c846c4df782` (272 files), protocol `signal-lab/2`, seed 33, 2000 Event resamples, 90% intervals, floor 100 paired Matches.

Baseline: the anchor: HybridAnchored without LLM, 0.80 market and 0.20 Valve rating logit (`debate::base`). Each Candidate Signal is fitted, for each Event, only on the Matches of the Events that ended before it began, and every datum it reads is observed before the Match cutoff. Differences are variant − anchor on the Matches both predicted (log-loss and Brier: below zero is better). The interval resamples whole Events. Verdict: `promote` when the interval of the log-loss difference is below zero and the Matches with a market line are not worse; `harmful` when it is above zero; `no evidence` otherwise or under the floor; `not testable` when the data could not be had. "Detectable" is the smallest log-loss difference these Matches would show with 80% probability: an effect smaller than it can hide as `no evidence`.

| Candidate Signal | Verdict | Computed on | Paired | Events | Δ log-loss | 90% interval | Detectable | Δ Brier | Δ accuracy | Market subset (n, Δ log-loss) |
|---|---|---|---|---|---|---|---|---|---|---|
| `rating_calibration` | no evidence | 342 | 342 | 4 | -0.1066 | [-0.1779, +0.0037] | 0.1274 | -0.0230 | +0.0058 | 112, -0.0005 |
| `learned_weights` | no evidence | 415 | 415 | 5 | -0.0783 | [-0.1361, +0.0051] | 0.0975 | -0.0167 | +0.0000 | 112, +0.0020 |

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
