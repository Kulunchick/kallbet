# PGL Major Singapore 2026 Stage 2 — LIVE tracking (post-start)

**Live, made 2026-11-28T13:50:07Z during Stage 2** at git `e69bfae381f6782ecf58f8157c7212f639bb61c2` from the PandaScore listing saved as `forecasts/data/panda-22473-matches-live-2026-11-28T134659Z.json.gz` (latest listing change 2026-11-28T13:43:44Z). This is live tracking, separate from the Holdout evaluation, which scores only the pre-registered forecast (`singapore-2026-stage2.md`, unchanged). Probabilities of the remaining Matches come from the pre-Stage models only (inputs as of the cutoff 2026-11-28T04:00:00Z, no refit on any Stage 2 result); finished Matches are fixed, running ones count as unplayed. 5,000,000 GPU runs per model, seed 33.

## Results so far

| Round | Match | Winner | Status |
|---:|---|---|---|
| 1 | Nemiga vs NIP | NIP | finished |
| 1 | FUT Esports vs FlyQuest | FUT Esports | finished |
| 1 | THE ONE vs Eternal Fire | THE ONE | finished |
| 1 | G2 vs Liquid | G2 | finished |
| 1 | Natus Vincere vs PARIVISION | Natus Vincere | finished |
| 1 | Astralis vs Imperial | Astralis | finished |
| 1 | MIBR vs 100 Thieves | 100 Thieves | finished |
| 1 | TheMongolz vs Lynn Vision | TheMongolz | finished |
| 2 | G2 vs NIP | G2 | finished |
| 2 | Nemiga vs Lynn Vision | Nemiga | finished |
| 2 | THE ONE vs TheMongolz | THE ONE | finished |
| 2 | Liquid vs Eternal Fire | Liquid | finished |
| 2 | FUT Esports vs 100 Thieves | FUT Esports | finished |
| 2 | MIBR vs Imperial | MIBR | finished |
| 2 | Natus Vincere vs Astralis | Astralis | finished |
| 2 | FlyQuest vs PARIVISION | — | running |

Pairing checks: round 2: listed pairing = simulator's pairing.

## Current records

| Team | Record |
|---|---|
| Astralis | 2–0 |
| FUT Esports | 2–0 |
| G2 | 2–0 |
| THE ONE | 2–0 |
| 100 Thieves | 1–1 |
| Liquid | 1–1 |
| MIBR | 1–1 |
| NIP | 1–1 |
| Natus Vincere | 1–1 |
| Nemiga | 1–1 |
| TheMongolz | 1–1 |
| FlyQuest | 0–1 |
| PARIVISION | 0–1 |
| Eternal Fire | 0–2 |
| Imperial | 0–2 |
| Lynn Vision | 0–2 |

## lgbm all B (pre-registered)

3–0: FUT Esports, TheMongolz; 0–3: FlyQuest, Lynn Vision; advance: Nemiga, G2, THE ONE, Natus Vincere, Liquid, PARIVISION.

| Model | P(≥5) | P(≥6) | P(≥7) | Expected correct | Score distribution 0…10 |
|---|---:|---:|---:|---:|---|
| lgbm all | 49.1% | 20.9% | 4.3% | 4.46 | 1: 0.5% · 2: 4.9% · 3: 17.1% · 4: 28.4% · 5: 28.2% · 6: 16.6% · 7: 4.2% · 8: 0.2% |
| lgbm-ext | 50.3% | 21.6% | 4.3% | 4.49 | 1: 0.4% · 2: 4.5% · 3: 16.7% · 4: 28.1% · 5: 28.7% · 6: 17.3% · 7: 4.2% · 8: 0.2% |
| logreg-ext | 49.7% | 20.7% | 4.0% | 4.47 | 1: 0.4% · 2: 4.5% · 3: 16.8% · 4: 28.6% · 5: 28.9% · 6: 16.8% · 7: 3.8% · 8: 0.1% |
| Anchor | 65.1% | 31.6% | 7.0% | 4.89 | 1: 0.1% · 2: 2.3% · 3: 10.1% · 4: 22.4% · 5: 33.5% · 6: 24.6% · 7: 6.7% · 8: 0.3% |

Per pick (probability it comes true; settled = 0% or 100% under every model):

| Pick | Team | Record | lgbm all | lgbm-ext | logreg-ext | Anchor | State |
|---|---|---|---:|---:|---:|---:|---|
| 3–0 | FUT Esports | 2–0 | 65.1% | 65.2% | 64.2% | 74.8% | open |
| 3–0 | TheMongolz | 1–1 | 0.0% | 0.0% | 0.0% | 0.0% | wrong |
| 0–3 | FlyQuest | 0–1 | 28.0% | 30.0% | 25.1% | 31.4% | open |
| 0–3 | Lynn Vision | 0–2 | 59.4% | 59.1% | 64.3% | 66.5% | open |
| advance | Nemiga | 1–1 | 54.9% | 63.7% | 61.8% | 75.5% | open |
| advance | G2 | 2–0 | 35.2% | 33.5% | 33.3% | 35.6% | open |
| advance | THE ONE | 2–0 | 46.6% | 43.5% | 40.7% | 56.0% | open |
| advance | Natus Vincere | 1–1 | 66.7% | 72.3% | 70.0% | 63.4% | open |
| advance | Liquid | 1–1 | 48.9% | 42.1% | 50.9% | 42.5% | open |
| advance | PARIVISION | 0–1 | 41.3% | 39.9% | 37.1% | 43.2% | open |

## Anchor A (pre-registered file)

3–0: FUT Esports, Nemiga; 0–3: FlyQuest, Lynn Vision; advance: G2, THE ONE, Natus Vincere, Astralis, TheMongolz, NIP.

| Model | P(≥5) | P(≥6) | P(≥7) | Expected correct | Score distribution 0…10 |
|---|---:|---:|---:|---:|---|
| lgbm all | 53.1% | 23.9% | 3.9% | 4.54 | 1: 0.6% · 2: 4.8% · 3: 15.6% · 4: 25.9% · 5: 29.3% · 6: 20.0% · 7: 3.9% |
| lgbm-ext | 54.7% | 24.7% | 4.1% | 4.59 | 1: 0.5% · 2: 4.2% · 3: 14.8% · 4: 25.9% · 5: 30.0% · 6: 20.6% · 7: 4.1% |
| logreg-ext | 52.6% | 22.8% | 3.7% | 4.53 | 1: 0.5% · 2: 4.6% · 3: 15.6% · 4: 26.6% · 5: 29.9% · 6: 19.1% · 7: 3.7% |
| Anchor | 66.1% | 33.6% | 5.5% | 4.90 | 1: 0.1% · 2: 2.3% · 3: 10.3% · 4: 21.1% · 5: 32.4% · 6: 28.1% · 7: 5.5% |

Per pick (probability it comes true; settled = 0% or 100% under every model):

| Pick | Team | Record | lgbm all | lgbm-ext | logreg-ext | Anchor | State |
|---|---|---|---:|---:|---:|---:|---|
| 3–0 | FUT Esports | 2–0 | 65.1% | 65.2% | 64.2% | 74.8% | open |
| 3–0 | Nemiga | 1–1 | 0.0% | 0.0% | 0.0% | 0.0% | wrong |
| 0–3 | FlyQuest | 0–1 | 28.0% | 30.0% | 25.1% | 31.4% | open |
| 0–3 | Lynn Vision | 0–2 | 59.4% | 59.1% | 64.3% | 66.5% | open |
| advance | G2 | 2–0 | 35.2% | 33.5% | 33.3% | 35.6% | open |
| advance | THE ONE | 2–0 | 46.6% | 43.5% | 40.7% | 56.0% | open |
| advance | Natus Vincere | 1–1 | 66.7% | 72.3% | 70.0% | 63.4% | open |
| advance | Astralis | 2–0 | 43.9% | 42.5% | 40.0% | 48.8% | open |
| advance | TheMongolz | 1–1 | 60.4% | 64.3% | 62.3% | 54.4% | open |
| advance | NIP | 1–1 | 48.5% | 48.5% | 52.7% | 58.9% | open |

## logreg-ext B (not pre-registered)

3–0: FUT Esports, TheMongolz; 0–3: FlyQuest, Lynn Vision; advance: Nemiga, Natus Vincere, NIP, Liquid, PARIVISION, 100 Thieves.

| Model | P(≥5) | P(≥6) | P(≥7) | Expected correct | Score distribution 0…10 |
|---|---:|---:|---:|---:|---|
| lgbm all | 51.9% | 17.0% | 2.0% | 4.51 | 1: 0.2% · 2: 2.8% · 3: 13.5% · 4: 31.6% · 5: 34.9% · 6: 15.0% · 7: 2.0% |
| lgbm-ext | 56.8% | 20.3% | 2.9% | 4.64 | 1: 0.1% · 2: 2.0% · 3: 11.3% · 4: 29.9% · 5: 36.4% · 6: 17.5% · 7: 2.8% · 8: 0.1% |
| logreg-ext | 60.4% | 23.5% | 3.8% | 4.75 | 1: 0.1% · 2: 1.5% · 3: 9.8% · 4: 28.2% · 5: 36.9% · 6: 19.7% · 7: 3.7% · 8: 0.1% |
| Anchor | 69.5% | 27.0% | 3.2% | 4.94 | 2: 0.3% · 3: 5.4% · 4: 24.7% · 5: 42.5% · 6: 23.9% · 7: 3.1% · 8: 0.1% |

Per pick (probability it comes true; settled = 0% or 100% under every model):

| Pick | Team | Record | lgbm all | lgbm-ext | logreg-ext | Anchor | State |
|---|---|---|---:|---:|---:|---:|---|
| 3–0 | FUT Esports | 2–0 | 65.1% | 65.2% | 64.2% | 74.8% | open |
| 3–0 | TheMongolz | 1–1 | 0.0% | 0.0% | 0.0% | 0.0% | wrong |
| 0–3 | FlyQuest | 0–1 | 28.0% | 30.0% | 25.1% | 31.4% | open |
| 0–3 | Lynn Vision | 0–2 | 59.4% | 59.1% | 64.3% | 66.5% | open |
| advance | Nemiga | 1–1 | 54.9% | 63.7% | 61.8% | 75.5% | open |
| advance | Natus Vincere | 1–1 | 66.7% | 72.3% | 70.0% | 63.4% | open |
| advance | NIP | 1–1 | 48.5% | 48.5% | 52.7% | 58.9% | open |
| advance | Liquid | 1–1 | 48.9% | 42.1% | 50.9% | 42.5% | open |
| advance | PARIVISION | 0–1 | 41.3% | 39.9% | 37.1% | 43.2% | open |
| advance | 100 Thieves | 1–1 | 38.4% | 43.7% | 48.6% | 37.5% | open |

## Method

- Models and inputs: as in `singapore-2026-stage2.md` (`lgbm all`, anchor) and `singapore-2026-stage2-ext.md` (`lgbm-ext`, `logreg-ext`), rebuilt from the same saved pre-Stage files; the code refuses to run if those show a started Match.
- Conditioning: each finished Match's pair gets probability 1 (or 0) in both formats; the Swiss simulator (`kbml/swiss_gpu.py`) then forms every later pairing from the results by its Buchholz rule. Round 1 is checked against the opening, each listed later round against the simulator's own pairing, and no simulated run contradicts a known result (asserted).

