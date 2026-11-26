# Pick'em variants in Pick'em Points

Protocol `signal-lab/3`, archive SHA-256 `25459d150e1fac87409e38d17a73ab2110444401703a056b1c4d4c846c4df782`, seed 33, 2000 Event resamples, 90% intervals. Each Stage's Pick'em is made 1 s before its first Match from the Liquipedia opening view saved from before it and the ratings the replay ports gave before that instant; the solver refuses a view that holds a result or a later-round pairing. Pick'em Points are KallBet's (Swiss 1 per correct 3-0, 0-3 and advance pick; Playoffs 1 per correct slot). The difference is the mean Points per Stage minus the baseline's, with the Event bootstrap; under 10 Stages the verdict is `no evidence`. Stages of the Holdout are not read.

The field is 6 simulated Members who submit the baseline's solver on ratings disturbed by a normal noise of 0.5 on the logit; the bot trails when its public Pick'em Points (Stages that ended before) are below the best Member's; the variance-seeking variant then discounts the field's shared picks by 50%. 5000 simulations per Swiss Pick'em.

## Variants against the baseline

| Variant | Total Points | Stages | Difference per Stage | 90% interval | Detectable | Verdict | Mean place | Events first |
|---|---:|---:|---:|---|---:|---|---:|---:|
| `baseline` | 70 | 18 | — | — | — | **no evidence**: the reference | 3.17 | 2 |
| `calibrated` | 67 | 18 | -0.167 | [-0.450, +0.000] | 0.358 | **no evidence**: the interval of the Pick'em Points difference includes zero | 3.83 | 2 |
| `learned` | 69 | 18 | -0.056 | [-0.150, +0.000] | 0.119 | **no evidence**: the interval of the Pick'em Points difference includes zero | 3.33 | 2 |
| `variance_seeking` | 65 | 18 | -0.278 | [-0.688, +0.111] | 0.608 | **no evidence**: the interval of the Pick'em Points difference includes zero | 3.83 | 3 |

`calibrated` rated 1180 pairs itself and left 268 to the baseline's rating.

`learned` rated 1448 pairs itself and left 0 to the baseline's rating.

## Pick'em Points per Stage

| Event | Stage | Picks | baseline | calibrated | learned | variance_seeking | trailing | best Member |
|---|---|---:|---:|---:|---:|---:|---|---:|
| PGL Major Copenhagen 2024 | Opening Stage | 10 | 3 | 3 | 3 | 3 | no | 3 |
| PGL Major Copenhagen 2024 | Elimination Stage | 10 | 0 | 0 | 0 | 0 | no | 4 |
| PGL Major Copenhagen 2024 | Playoffs | 7 | 4 | 4 | 4 | 1 | yes | 4 |
| Perfect World Major Shanghai Major 2024 | Opening Stage | 10 | 3 | 3 | 3 | 3 | no | 6 |
| Perfect World Major Shanghai Major 2024 | Elimination Stage | 10 | 6 | 6 | 6 | 4 | yes | 6 |
| Perfect World Major Shanghai Major 2024 | Playoffs | 7 | 3 | 3 | 3 | 4 | yes | 4 |
| BLAST.tv Major Austin 2025 | Stage 1 | 10 | 6 | 6 | 6 | 6 | no | 6 |
| BLAST.tv Major Austin 2025 | Stage 2 | 10 | 2 | 2 | 2 | 2 | no | 3 |
| BLAST.tv Major Austin 2025 | Stage 3 | 10 | 4 | 4 | 4 | 4 | yes | 4 |
| BLAST.tv Major Austin 2025 | Playoffs | 7 | 6 | 6 | 6 | 6 | no | 6 |
| StarLadder Major Budapest 2025 | Stage 1 | 10 | 6 | 6 | 6 | 6 | no | 6 |
| StarLadder Major Budapest 2025 | Stage 2 | 10 | 5 | 5 | 5 | 5 | no | 6 |
| StarLadder Major Budapest 2025 | Stage 3 | 10 | 4 | 4 | 4 | 6 | yes | 5 |
| StarLadder Major Budapest 2025 | Playoffs | 7 | 1 | 1 | 1 | 1 | no | 2 |
| IEM Cologne Major 2026 | Stage 1 | 10 | 4 | 2 | 4 | 4 | no | 5 |
| IEM Cologne Major 2026 | Stage 2 | 10 | 5 | 6 | 6 | 6 | yes | 7 |
| IEM Cologne Major 2026 | Stage 3 | 10 | 4 | 2 | 2 | 1 | yes | 4 |
| IEM Cologne Major 2026 | Playoffs | 7 | 4 | 4 | 4 | 3 | yes | 6 |

## Stages not scored

- PGL Major Singapore 2026 / Stage 1: the Stage is not complete
