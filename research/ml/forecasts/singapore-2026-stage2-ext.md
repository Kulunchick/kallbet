# PGL Major Singapore 2026 Stage 2 — ext forecasters' slips (NOT pre-registered)

**Made 2026-11-27T23:13:12Z** (machine clock, UTC; Stage 2 starts 2026-11-28T04:00:00Z) at git `5a9a2b6f04d7d3eae57d6559c1a1f4479a4d8f79`, from **pre-Stage inputs only**: the PandaScore listings and the Liquipedia revision saved under `forecasts/data/` on 2026-11-27 (every Stage 2 Match `not_started`), the snapshot and the extended archive (latest result 2026-11-22T19:23:39Z; latest weekly HLTV ranking 2026-11-23). Nothing was fetched and no Stage 2 result enters. It is **not a pre-registered forecast**: it was made after the pre-registered forecast was fixed and after the 28-Stage replay results of these forecasters (FORECASTER.md) were seen, on request, so it must not be scored as one. The pre-registered forecast is `singapore-2026-stage2.md` (unchanged).

Reproduce: `<gpu python> -m kbml.pickem_forecast_ext --runs 5000000 --random-starts 8` from `research/ml/` (GPU environment, see README).

## Slips

| Model | Strategy | 3–0 | 0–3 | Advance (3–1 / 3–2) | Expected correct | P(≥5) |
|---|---|---|---|---|---:|---:|
| **logreg-ext** | B | FUT Esports, TheMongolz | FlyQuest, Lynn Vision | Nemiga, Natus Vincere, NIP, Liquid, PARIVISION, 100 Thieves | 3.806 | 30.9% |
| logreg-ext | A | FUT Esports, TheMongolz | FlyQuest, Lynn Vision | Nemiga, Natus Vincere, NIP, Liquid, PARIVISION, 100 Thieves | 3.806 | 30.9% |
| **lgbm-ext** | B | FUT Esports, TheMongolz | FlyQuest, Lynn Vision | Nemiga, G2, Natus Vincere, Astralis, NIP, PARIVISION | 3.673 | 28.1% |
| lgbm-ext | A | FUT Esports, Natus Vincere | FlyQuest, Lynn Vision | Nemiga, G2, Astralis, TheMongolz, NIP, PARIVISION | 3.690 | 28.0% |
| Anchor | B | FUT Esports, TheMongolz | FlyQuest, Lynn Vision | Nemiga, G2, THE ONE, Natus Vincere, Astralis, NIP | 4.814 | 59.6% |
| Anchor | A | FUT Esports, Nemiga | FlyQuest, Lynn Vision | G2, THE ONE, Natus Vincere, Astralis, TheMongolz, NIP | 4.840 | 59.5% |

Expected correct and P(≥5) are under the slip's own model, in-sample on its 5,000,000 simulated Stages.

## Cross table: each slip under each model

| Slip | under logreg-ext | under lgbm-ext | under Anchor |
|---|---:|---:|---:|
| logreg-ext B | 3.806 / 30.9% | 3.569 / 25.5% | 4.384 / 46.5% |
| logreg-ext A | 3.806 / 30.9% | 3.569 / 25.5% | 4.384 / 46.5% |
| lgbm-ext B | 3.698 / 28.5% | 3.673 / 28.1% | 4.736 / 57.1% |
| lgbm-ext A | 3.653 / 26.7% | 3.690 / 28.0% | 4.670 / 55.5% |
| Anchor B | 3.621 / 26.5% | 3.626 / 27.0% | 4.814 / 59.6% |
| Anchor A | 3.552 / 23.9% | 3.623 / 26.0% | 4.840 / 59.5% |

Cells: expected correct / P(≥5).

## Per team under logreg-ext

| Seed | Team | P(3–0) | P(0–3) | P(3–1 or 3–2) | lgbm-ext P(3–0) | lgbm-ext P(0–3) | lgbm-ext P(3–1/3–2) |
|---:|---|---:|---:|---:|---:|---:|---:|
| 1 | FUT Esports | 24.2% | 3.7% | 47.7% | 28.5% | 3.4% | 48.3% |
| 2 | Nemiga | 20.1% | 5.1% | 48.8% | 22.0% | 5.1% | 48.4% |
| 3 | G2 | 10.8% | 10.4% | 39.6% | 15.6% | 7.4% | 44.1% |
| 4 | THE ONE | 8.1% | 13.8% | 32.5% | 10.2% | 11.1% | 35.9% |
| 5 | Natus Vincere | 24.8% | 3.8% | 51.1% | 26.1% | 4.0% | 50.4% |
| 6 | Astralis | 9.6% | 12.0% | 35.0% | 10.9% | 10.2% | 38.8% |
| 7 | TheMongolz | 25.1% | 3.5% | 47.0% | 22.0% | 4.8% | 48.0% |
| 8 | MIBR | 7.8% | 14.5% | 32.9% | 9.5% | 11.8% | 36.8% |
| 9 | FlyQuest | 3.6% | 25.5% | 20.4% | 3.6% | 26.2% | 20.5% |
| 10 | NIP | 10.5% | 11.2% | 40.8% | 8.3% | 13.5% | 37.3% |
| 11 | Liquid | 14.5% | 8.0% | 44.3% | 9.1% | 12.9% | 37.3% |
| 12 | Eternal Fire | 11.0% | 10.7% | 38.0% | 7.4% | 15.1% | 32.3% |
| 13 | PARIVISION | 9.2% | 12.3% | 40.2% | 8.8% | 12.7% | 40.6% |
| 14 | Imperial | 7.0% | 16.0% | 29.8% | 6.8% | 16.6% | 30.4% |
| 15 | Lynn Vision | 1.5% | 39.3% | 10.7% | 3.0% | 31.0% | 15.6% |
| 16 | 100 Thieves | 12.1% | 10.1% | 41.1% | 8.1% | 14.3% | 35.2% |

## Method

- Forecasters: `kbml/forecaster.py` (phase 4, FORECASTER.md). `logreg-ext`: logreg on `p3_rw` (baseline + weekly HLTV ranking + tier). `lgbm-ext`: LightGBM on `p3_all`. Each fitted once on every non-qualifier Match known before the cutoff: 11,079 (in-scope, S/A 2023-24, tier B, tier C, Singapore Stage 1's 33 from the saved listing); hyperparameters by inner walk-forward on completed earlier Events. The 120 pairings in BO1 and BO3 are hypothetical: match-page features take the missing-data path. No Stage 2 market line exists in the inputs; the anchor is the Valve rating alone.
- Simulation and search: `pickem_forecast.analyse` (GPU Swiss simulator, seed 33; strategy A = exact expected-points optimum; B = local search for P(≥5) from A, the checkpoints' slips and 8 seeded random slips), the same code as the pre-registered forecast.

## Result

To be scored after Stage 2 ends.

