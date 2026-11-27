# PGL Major Singapore 2026 Stage 2 — Pick'em forecast

Made **2026-11-27T16:18:45Z** at git `26db20429d6de6737586acc5ab9ae10f29379a7b` (the commit this file is added on top of), before Stage 2 begins (2026-11-28T04:00:00Z, PandaScore tournament 22473). PandaScore listed every Stage 2 match as `not_started` with no winner (latest listing change 2026-11-27T15:30:48Z). No Stage 2 result is used anywhere. Reproduce with `uv run --no-default-groups --group gpu python -m kbml.pickem_forecast --runs 50000000 --checkpoints 100000,200000,500000,1000000,2000000,5000000,10000000,20000000 --check-runs 1000000 --random-starts 8` from `research/ml/` (GPU environment: see README); the script refuses to run if its saved Stage 2 listing shows a started match. Total wall time 1,770 s, almost all of it the B searches.

Primary slip: **LightGBM on all features, strategy B** (highest simulated chance of at least five correct), the best row of the 28-stage replay in `PICKEM.md` (mean 4.57, 14/28 stages ≥5; the intervals of all models overlap, so this ranking is weak evidence).

## Picks

| Model | Strategy | 3–0 | 0–3 | Advance (3–1 / 3–2) | Expected correct | P(≥5) |
|---|---|---|---|---|---:|---:|
| **LightGBM all** | B | FUT Esports, TheMongolz | FlyQuest, Lynn Vision | Nemiga, G2, THE ONE, Natus Vincere, Liquid, PARIVISION | 3.558 | 25.3% |
| LightGBM all | A | FUT Esports, TheMongolz | FlyQuest, Lynn Vision | Nemiga, G2, THE ONE, Natus Vincere, Liquid, PARIVISION | 3.558 | 25.3% |
| Anchor | A | FUT Esports, Nemiga | FlyQuest, Lynn Vision | G2, THE ONE, Natus Vincere, Astralis, TheMongolz, NIP | 4.839 | 59.5% |
| Anchor | B | FUT Esports, TheMongolz | FlyQuest, Lynn Vision | Nemiga, G2, THE ONE, Natus Vincere, Astralis, NIP | 4.813 | 59.6% |

Expected correct and P(≥5) are in-sample on the 50,000,000 simulated stages of that model (Monte Carlo standard error of P(≥5) ≤ 0.01%). Each model is scored on its own simulation, so the numbers measure how confident the model is, not which slip is better.

Cross-model view: each slip's expected correct / P(≥5) under the other model's simulation.

| Slip | under Anchor | under LightGBM all |
|---|---:|---:|
| LightGBM all B | 4.686 / 55.7% | 3.558 / 25.3% |
| LightGBM all A | 4.686 / 55.7% | 3.558 / 25.3% |
| Anchor A | 4.839 / 59.5% | 3.459 / 22.1% |
| Anchor B | 4.813 / 59.6% | 3.475 / 23.4% |

## Seeds and round 1

Seeds from the Liquipedia Stage 2 page (revision 2026-11-27T15:42:42Z): invited teams 1–8 by Valve global standing, Stage 1 qualifiers 9–16 by Stage 1 record and Buchholz. Round 1 pairs seed k with k+8, matching the PandaScore listing. Round 1, round 2 and the 1–1 round-3 matches are BO1; every match that can advance or eliminate a team is BO3.

| Round 1 (BO1) | Anchor P(first wins) | LightGBM all P(first wins) |
|---|---:|---:|
| 2. Nemiga vs 10. NIP | 61.8% | 52.0% |
| 1. FUT Esports vs 9. FlyQuest | 90.5% | 68.6% |
| 4. THE ONE vs 12. Eternal Fire | 61.7% | 52.3% |
| 3. G2 vs 11. Liquid | 62.1% | 56.8% |
| 5. Natus Vincere vs 13. PARIVISION | 54.1% | 53.1% |
| 6. Astralis vs 14. Imperial | 71.7% | 56.0% |
| 8. MIBR vs 16. 100 Thieves | 54.9% | 55.4% |
| 7. TheMongolz vs 15. Lynn Vision | 83.3% | 68.6% |

## Team probabilities — LightGBM all

From 50,000,000 simulated stages. Advance = any 3–x finish (3–0, 3–1 or 3–2); the advance pick scores only on 3–1 or 3–2.

| Seed | Team | P(3–0) | P(0–3) | P(advance) | P(3–1/3–2) | B pick | A pick |
|---:|---|---:|---:|---:|---:|---|---|
| 1 | FUT Esports | 27.9% | 3.7% | 76.2% | 48.3% | 3–0 | 3–0 |
| 5 | Natus Vincere | 18.7% | 6.0% | 68.0% | 49.3% | advance | advance |
| 3 | G2 | 19.1% | 6.3% | 66.1% | 47.1% | advance | advance |
| 7 | TheMongolz | 20.3% | 5.6% | 65.8% | 45.5% | 3–0 | 3–0 |
| 13 | PARIVISION | 12.6% | 10.4% | 55.2% | 42.6% | advance | advance |
| 2 | Nemiga | 14.1% | 9.4% | 55.0% | 40.9% | advance | advance |
| 11 | Liquid | 10.9% | 12.0% | 50.8% | 39.9% | advance | advance |
| 4 | THE ONE | 11.7% | 11.1% | 49.4% | 37.7% | advance | advance |
| 6 | Astralis | 11.9% | 11.2% | 49.1% | 37.2% | — | — |
| 8 | MIBR | 11.6% | 11.1% | 49.0% | 37.4% | — | — |
| 10 | NIP | 10.4% | 12.4% | 47.5% | 37.0% | — | — |
| 12 | Eternal Fire | 9.2% | 13.6% | 44.5% | 35.3% | — | — |
| 16 | 100 Thieves | 6.4% | 18.5% | 35.6% | 29.1% | — | — |
| 14 | Imperial | 6.6% | 18.7% | 34.2% | 27.6% | — | — |
| 9 | FlyQuest | 4.7% | 23.7% | 28.5% | 23.7% | 0–3 | 0–3 |
| 15 | Lynn Vision | 3.9% | 26.3% | 25.2% | 21.3% | 0–3 | 0–3 |

## Team probabilities — Anchor

From 50,000,000 simulated stages. Advance = any 3–x finish (3–0, 3–1 or 3–2); the advance pick scores only on 3–1 or 3–2.

| Seed | Team | P(3–0) | P(0–3) | P(advance) | P(3–1/3–2) | B pick | A pick |
|---:|---|---:|---:|---:|---:|---|---|
| 1 | FUT Esports | 54.5% | 0.2% | 95.1% | 40.6% | 3–0 | 3–0 |
| 2 | Nemiga | 26.4% | 1.6% | 82.4% | 56.0% | advance | 3–0 |
| 3 | G2 | 20.0% | 2.2% | 74.6% | 54.6% | advance | advance |
| 4 | THE ONE | 16.6% | 2.7% | 69.4% | 52.8% | advance | advance |
| 5 | Natus Vincere | 13.5% | 3.4% | 65.7% | 52.2% | advance | advance |
| 6 | Astralis | 14.9% | 3.4% | 62.8% | 47.9% | advance | advance |
| 7 | TheMongolz | 14.5% | 2.8% | 61.2% | 46.7% | 3–0 | advance |
| 10 | NIP | 8.1% | 6.7% | 54.6% | 46.5% | advance | advance |
| 13 | PARIVISION | 7.5% | 6.9% | 52.4% | 45.0% | — | — |
| 8 | MIBR | 8.6% | 5.6% | 51.6% | 43.0% | — | — |
| 11 | Liquid | 5.6% | 10.0% | 42.3% | 36.7% | — | — |
| 12 | Eternal Fire | 4.6% | 11.4% | 37.3% | 32.7% | — | — |
| 16 | 100 Thieves | 4.0% | 13.2% | 34.5% | 30.6% | — | — |
| 14 | Imperial | 1.0% | 27.4% | 11.7% | 10.7% | — | — |
| 9 | FlyQuest | 0.2% | 45.8% | 3.0% | 2.8% | 0–3 | 0–3 |
| 15 | Lynn Vision | 0.1% | 56.6% | 1.4% | 1.3% | 0–3 | 0–3 |

## Convergence

Simulator: torch 2.15.0+cu132 on cuda (NVIDIA GeForce RTX 3060 Laptop GPU). Each checkpoint is a prefix of the same run (torch, batches of 1,000,000, batch b seeded 33·1,000,003 + b): A by exact assignment on the prefix, B by the replay's best-improving swap search on the prefix (started from A, the previous checkpoint's B and two seeded random slips). The final B search on the full run restarts from every checkpoint's A and B and 8 seeded random slips. Changed = team/category selections that differ from the final slip of that strategy. Expected and P(≥5) are re-scored on the full run.

### LightGBM all

Simulation of 50,000,000 stages took 26.3 s; the final B search 483 s.

| Runs | Strategy | 3–0 | 0–3 | Advance | Changed vs final | Expected (full run) | P(≥5) (full run) | P(≥5) (prefix) |
|---:|---|---|---|---|---:|---:|---:|---:|
| 100,000 | A | FUT Esports, TheMongolz | FlyQuest, Lynn Vision | Nemiga, G2, THE ONE, Natus Vincere, Liquid, PARIVISION | 0 | 3.5577 | 25.265% | 25.333% |
| 100,000 | B | FUT Esports, TheMongolz | FlyQuest, Lynn Vision | Nemiga, G2, Natus Vincere, MIBR, Liquid, PARIVISION | 1 | 3.5542 | 25.157% | 25.352% |
| 200,000 | A | FUT Esports, TheMongolz | FlyQuest, Lynn Vision | Nemiga, G2, THE ONE, Natus Vincere, Liquid, PARIVISION | 0 | 3.5577 | 25.265% | 25.285% |
| 200,000 | B | FUT Esports, TheMongolz | FlyQuest, Lynn Vision | Nemiga, G2, THE ONE, Natus Vincere, Liquid, PARIVISION | 0 | 3.5577 | 25.265% | 25.285% |
| 500,000 | A | FUT Esports, TheMongolz | FlyQuest, Lynn Vision | Nemiga, G2, THE ONE, Natus Vincere, Liquid, PARIVISION | 0 | 3.5577 | 25.265% | 25.237% |
| 500,000 | B | FUT Esports, TheMongolz | FlyQuest, Lynn Vision | Nemiga, G2, THE ONE, Natus Vincere, Liquid, PARIVISION | 0 | 3.5577 | 25.265% | 25.237% |
| 1,000,000 | A | FUT Esports, TheMongolz | FlyQuest, Lynn Vision | Nemiga, G2, THE ONE, Natus Vincere, Liquid, PARIVISION | 0 | 3.5577 | 25.265% | 25.263% |
| 1,000,000 | B | FUT Esports, TheMongolz | FlyQuest, Lynn Vision | Nemiga, G2, THE ONE, Natus Vincere, Liquid, PARIVISION | 0 | 3.5577 | 25.265% | 25.263% |
| 2,000,000 | A | FUT Esports, TheMongolz | FlyQuest, Lynn Vision | Nemiga, G2, THE ONE, Natus Vincere, Liquid, PARIVISION | 0 | 3.5577 | 25.265% | 25.244% |
| 2,000,000 | B | FUT Esports, TheMongolz | FlyQuest, Lynn Vision | Nemiga, G2, THE ONE, Natus Vincere, Liquid, PARIVISION | 0 | 3.5577 | 25.265% | 25.244% |
| 5,000,000 | A | FUT Esports, TheMongolz | FlyQuest, Lynn Vision | Nemiga, G2, THE ONE, Natus Vincere, Liquid, PARIVISION | 0 | 3.5577 | 25.265% | 25.243% |
| 5,000,000 | B | FUT Esports, TheMongolz | FlyQuest, Lynn Vision | Nemiga, G2, THE ONE, Natus Vincere, Liquid, PARIVISION | 0 | 3.5577 | 25.265% | 25.243% |
| 10,000,000 | A | FUT Esports, TheMongolz | FlyQuest, Lynn Vision | Nemiga, G2, THE ONE, Natus Vincere, Liquid, PARIVISION | 0 | 3.5577 | 25.265% | 25.237% |
| 10,000,000 | B | FUT Esports, TheMongolz | FlyQuest, Lynn Vision | Nemiga, G2, THE ONE, Natus Vincere, Liquid, PARIVISION | 0 | 3.5577 | 25.265% | 25.237% |
| 20,000,000 | A | FUT Esports, TheMongolz | FlyQuest, Lynn Vision | Nemiga, G2, THE ONE, Natus Vincere, Liquid, PARIVISION | 0 | 3.5577 | 25.265% | 25.251% |
| 20,000,000 | B | FUT Esports, TheMongolz | FlyQuest, Lynn Vision | Nemiga, G2, THE ONE, Natus Vincere, Liquid, PARIVISION | 0 | 3.5577 | 25.265% | 25.251% |
| 50,000,000 | A | FUT Esports, TheMongolz | FlyQuest, Lynn Vision | Nemiga, G2, THE ONE, Natus Vincere, Liquid, PARIVISION | 0 | 3.5577 | 25.265% | 25.265% |
| 50,000,000 | B | FUT Esports, TheMongolz | FlyQuest, Lynn Vision | Nemiga, G2, THE ONE, Natus Vincere, Liquid, PARIVISION | 0 | 3.5577 | 25.265% | 25.265% |
| 50,000,000 | B (final search) | FUT Esports, TheMongolz | FlyQuest, Lynn Vision | Nemiga, G2, THE ONE, Natus Vincere, Liquid, PARIVISION | 0 | 3.5577 | 25.265% | — |

### Anchor

Simulation of 50,000,000 stages took 26.9 s; the final B search 522 s.

| Runs | Strategy | 3–0 | 0–3 | Advance | Changed vs final | Expected (full run) | P(≥5) (full run) | P(≥5) (prefix) |
|---:|---|---|---|---|---:|---:|---:|---:|
| 100,000 | A | FUT Esports, Nemiga | FlyQuest, Lynn Vision | G2, THE ONE, Natus Vincere, Astralis, TheMongolz, NIP | 0 | 4.8387 | 59.483% | 59.680% |
| 100,000 | B | FUT Esports, TheMongolz | FlyQuest, Lynn Vision | Nemiga, G2, THE ONE, Natus Vincere, Astralis, NIP | 0 | 4.8131 | 59.577% | 59.760% |
| 200,000 | A | FUT Esports, Nemiga | FlyQuest, Lynn Vision | G2, THE ONE, Natus Vincere, Astralis, TheMongolz, NIP | 0 | 4.8387 | 59.483% | 59.509% |
| 200,000 | B | FUT Esports, TheMongolz | FlyQuest, Lynn Vision | Nemiga, G2, THE ONE, Natus Vincere, Astralis, NIP | 0 | 4.8131 | 59.577% | 59.705% |
| 500,000 | A | FUT Esports, Nemiga | FlyQuest, Lynn Vision | G2, THE ONE, Natus Vincere, Astralis, TheMongolz, NIP | 0 | 4.8387 | 59.483% | 59.394% |
| 500,000 | B | FUT Esports, TheMongolz | FlyQuest, Lynn Vision | Nemiga, G2, THE ONE, Natus Vincere, Astralis, NIP | 0 | 4.8131 | 59.577% | 59.688% |
| 1,000,000 | A | FUT Esports, Nemiga | FlyQuest, Lynn Vision | G2, THE ONE, Natus Vincere, Astralis, TheMongolz, NIP | 0 | 4.8387 | 59.483% | 59.401% |
| 1,000,000 | B | FUT Esports, TheMongolz | FlyQuest, Lynn Vision | Nemiga, G2, THE ONE, Natus Vincere, Astralis, NIP | 0 | 4.8131 | 59.577% | 59.630% |
| 2,000,000 | A | FUT Esports, Nemiga | FlyQuest, Lynn Vision | G2, THE ONE, Natus Vincere, Astralis, TheMongolz, NIP | 0 | 4.8387 | 59.483% | 59.458% |
| 2,000,000 | B | FUT Esports, TheMongolz | FlyQuest, Lynn Vision | Nemiga, G2, THE ONE, Natus Vincere, Astralis, NIP | 0 | 4.8131 | 59.577% | 59.649% |
| 5,000,000 | A | FUT Esports, Nemiga | FlyQuest, Lynn Vision | G2, THE ONE, Natus Vincere, Astralis, TheMongolz, NIP | 0 | 4.8387 | 59.483% | 59.493% |
| 5,000,000 | B | FUT Esports, TheMongolz | FlyQuest, Lynn Vision | Nemiga, G2, THE ONE, Natus Vincere, Astralis, NIP | 0 | 4.8131 | 59.577% | 59.617% |
| 10,000,000 | A | FUT Esports, Nemiga | FlyQuest, Lynn Vision | G2, THE ONE, Natus Vincere, Astralis, TheMongolz, NIP | 0 | 4.8387 | 59.483% | 59.466% |
| 10,000,000 | B | FUT Esports, TheMongolz | FlyQuest, Lynn Vision | Nemiga, G2, THE ONE, Natus Vincere, Astralis, NIP | 0 | 4.8131 | 59.577% | 59.583% |
| 20,000,000 | A | FUT Esports, Nemiga | FlyQuest, Lynn Vision | G2, THE ONE, Natus Vincere, Astralis, TheMongolz, NIP | 0 | 4.8387 | 59.483% | 59.471% |
| 20,000,000 | B | FUT Esports, TheMongolz | FlyQuest, Lynn Vision | Nemiga, G2, THE ONE, Natus Vincere, Astralis, NIP | 0 | 4.8131 | 59.577% | 59.576% |
| 50,000,000 | A | FUT Esports, Nemiga | FlyQuest, Lynn Vision | G2, THE ONE, Natus Vincere, Astralis, TheMongolz, NIP | 0 | 4.8387 | 59.483% | 59.483% |
| 50,000,000 | B | FUT Esports, TheMongolz | FlyQuest, Lynn Vision | Nemiga, G2, THE ONE, Natus Vincere, Astralis, NIP | 0 | 4.8131 | 59.577% | 59.577% |
| 50,000,000 | B (final search) | FUT Esports, TheMongolz | FlyQuest, Lynn Vision | Nemiga, G2, THE ONE, Natus Vincere, Astralis, NIP | 0 | 4.8131 | 59.577% | — |

## GPU simulator check

`kbml/swiss_gpu.py` runs the same Swiss procedure as the replay's numpy simulator with every run advancing round by round in tensors (Buchholz pairing with the rematch tables, BO1/BO3 by bucket). Fed the same uniform draws it returns identical outcomes (`tests/test_swiss_gpu.py`, CPU and CUDA). Here both simulate each model independently (numpy seed 33, torch seed 34) and every team's 3–0, 0–3, 3–1/3–2 and other shares are compared (16 × 4 z-scores per model).

| Model | Runs each | numpy | torch | Largest share gap | max abs z | abs z > 3 |
|---|---:|---:|---:|---:|---:|---:|
| LightGBM all | 1,000,000 | 6.6 s | 1.0 s | 0.174% | 2.89 | 0/64 |
| Anchor | 1,000,000 | 6.4 s | 0.7 s | 0.108% | 2.27 | 0/64 |

Throughput: numpy about 0.15M runs/s (an aborted earlier CPU run of this forecast simulated 10,000,000 LightGBM stages in 66 s and needed 285 s for its final B search at 10M with 20 random starts); torch on the RTX 3060 Laptop about 1.9M runs/s in 1M-run batches (1.9 GiB peak; 2M batches spill out of the 6 GB card and slow to 0.55M runs/s). The 50M-run B searches run on the CPU (numpy, bit-packed) and dominate the wall time: 483 s and 522 s.

## Score distribution

| Slip | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| LightGBM all B | 0.7% | 5.9% | 17.4% | 26.0% | 24.7% | 16.2% | 6.8% | 1.8% | 0.4% | 0.1% | 0.0% |
| LightGBM all A | 0.7% | 5.9% | 17.4% | 26.0% | 24.7% | 16.2% | 6.8% | 1.8% | 0.4% | 0.1% | 0.0% |
| Anchor A | 0.0% | 0.6% | 4.1% | 12.8% | 22.9% | 27.5% | 19.9% | 9.0% | 2.7% | 0.4% | 0.0% |
| Anchor B | 0.0% | 0.6% | 4.0% | 12.8% | 23.0% | 28.7% | 20.9% | 7.2% | 2.2% | 0.6% | 0.0% |

## Inputs

- Training: 2,159 non-qualifier results with a known finish before the cutoff: the research archive (latest result 2026-11-15T11:00:12Z) plus all 33 Singapore Stage 1 results from PandaScore tournament 22378 (finish times, winners and map scores). The archive snapshot holds nothing between its latest result and Stage 1, so any other matches played in that gap are missing.
- Stage 1 rows carry the saved pre-match market lines where the replay fixtures have them (24/33) and Valve ratings otherwise. They have no HLTV match pages, so their map-level and page-derived features are missing; they still feed series Elo, form, head-to-head, rest and venue history and the training set.
- LightGBM hyperparameters from the inner walk-forward over completed events: `{'num_leaves': 4, 'n_estimators': 150, 'learning_rate': 0.03, 'min_child_samples': 40}`; 76 features (the `full` set).
- Round 1 market lines: none. The saved OddsPapi fixture lists stop at Stage 1 and a new fixtures listing is a billable request, so the anchor is the Valve-rating probability alone (the 80/20 blend with no market). Valve ratings cover 240/240 pair-format rows (standings of 2026-11-02); a pair without both ratings gets 50%.
- THE ONE is the 1win roster renamed after the 2026-11-02 Valve standings (same five players; seed 4 on Liquipedia matches 1win's 1,598 points). Its rating, archive history (as 1WIN) and rank use 1win.
- HLTV rank badges at the cutoff known for 6/16 teams.
- Stage 2 pages, lineups and match-specific panels do not exist before the stage and take the feature groups' missing-data behaviour, as in the replay.
- Pairings after round 1 follow the sidecar Buchholz rule with Liquipedia's initial seeds, the procedure that reproduced all 12 Major stages with published seeds in the replay.
- B is a local search (best-improving two-team role swaps) from A, every checkpoint's A and B and 8 seeded random slips; a better slip it did not reach can exist.
- Raw inputs saved under `forecasts/data/`: PandaScore `GET /tournaments/22378/matches?per_page=100` and `/tournaments/22473/matches?per_page=100`, and the Liquipedia revision of `PGL/2026/Singapore/Stage 2`, all fetched 2026-11-27.

## Result

To be scored after Stage 2 ends (2026-12-01).

