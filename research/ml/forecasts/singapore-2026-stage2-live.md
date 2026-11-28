# PGL Major Singapore 2026 Stage 2 — LIVE tracking (post-start)

**Live, made 2026-11-28T14:51:35Z during Stage 2** at git `0bd98228bdac6faa87e9a893a5cd0c5125b018a9` from the PandaScore listing saved as `forecasts/data/panda-22473-matches-live-2026-11-28T144954Z.json.gz` (latest listing change 2026-11-28T14:47:50Z). This is live tracking, separate from the Holdout evaluation, which scores only the pre-registered forecast (`singapore-2026-stage2.md`, unchanged). Probabilities of the remaining Matches come from the pre-Stage models only (inputs as of the cutoff 2026-11-28T04:00:00Z, no refit on any Stage 2 result); finished Matches are fixed, running ones count as unplayed. 5,000,000 GPU runs per model, seed 33.

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
| 2 | FlyQuest vs PARIVISION | FlyQuest | finished |

Pairing checks: round 2: listed pairing = simulator's pairing.

## Current records

| Team | Record |
|---|---|
| Astralis | 2–0 |
| FUT Esports | 2–0 |
| G2 | 2–0 |
| THE ONE | 2–0 |
| 100 Thieves | 1–1 |
| FlyQuest | 1–1 |
| Liquid | 1–1 |
| MIBR | 1–1 |
| NIP | 1–1 |
| Natus Vincere | 1–1 |
| Nemiga | 1–1 |
| TheMongolz | 1–1 |
| Eternal Fire | 0–2 |
| Imperial | 0–2 |
| Lynn Vision | 0–2 |
| PARIVISION | 0–2 |

## lgbm all B (pre-registered)

3–0: FUT Esports, TheMongolz; 0–3: FlyQuest, Lynn Vision; advance: Nemiga, G2, THE ONE, Natus Vincere, Liquid, PARIVISION.

| Model | P(≥5) | P(≥6) | P(≥7) | Expected correct | Score distribution 0…10 |
|---|---:|---:|---:|---:|---|
| lgbm all | 31.4% | 6.0% | 0.2% | 3.98 | 1: 0.8% · 2: 6.7% · 3: 23.9% · 4: 37.3% · 5: 25.4% · 6: 5.8% · 7: 0.2% |
| lgbm-ext | 31.7% | 5.8% | 0.2% | 3.99 | 1: 0.7% · 2: 6.4% · 3: 23.5% · 4: 37.7% · 5: 25.8% · 6: 5.6% · 7: 0.2% |
| logreg-ext | 31.8% | 5.8% | 0.2% | 4.00 | 1: 0.7% · 2: 6.4% · 3: 23.4% · 4: 37.7% · 5: 26.0% · 6: 5.6% · 7: 0.2% |
| Anchor | 53.9% | 13.6% | 0.6% | 4.54 | 1: 0.1% · 2: 1.4% · 3: 11.0% · 4: 33.7% · 5: 40.2% · 6: 13.0% · 7: 0.6% |

Per pick (probability it comes true; settled = 0% or 100% under every model):

| Pick | Team | Record | lgbm all | lgbm-ext | logreg-ext | Anchor | State |
|---|---|---|---:|---:|---:|---:|---|
| 3–0 | FUT Esports | 2–0 | 64.7% | 65.4% | 63.0% | 78.0% | open |
| 3–0 | TheMongolz | 1–1 | 0.0% | 0.0% | 0.0% | 0.0% | wrong |
| 0–3 | FlyQuest | 1–1 | 0.0% | 0.0% | 0.0% | 0.0% | wrong |
| 0–3 | Lynn Vision | 0–2 | 67.0% | 68.4% | 70.9% | 89.5% | open |
| advance | Nemiga | 1–1 | 55.7% | 65.1% | 62.9% | 77.6% | open |
| advance | G2 | 2–0 | 34.5% | 31.2% | 33.2% | 38.8% | open |
| advance | THE ONE | 2–0 | 43.6% | 40.7% | 36.3% | 43.1% | open |
| advance | Natus Vincere | 1–1 | 65.9% | 72.1% | 68.1% | 63.6% | open |
| advance | Liquid | 1–1 | 48.6% | 41.2% | 50.8% | 43.1% | open |
| advance | PARIVISION | 0–2 | 17.9% | 15.1% | 14.2% | 20.4% | open |

## Anchor A (pre-registered file)

3–0: FUT Esports, Nemiga; 0–3: FlyQuest, Lynn Vision; advance: G2, THE ONE, Natus Vincere, Astralis, TheMongolz, NIP.

| Model | P(≥5) | P(≥6) | P(≥7) | Expected correct | Score distribution 0…10 |
|---|---:|---:|---:|---:|---|
| lgbm all | 47.9% | 19.9% | 3.0% | 4.37 | 0: 0.1% · 1: 1.1% · 2: 6.5% · 3: 17.5% · 4: 26.9% · 5: 28.0% · 6: 16.8% · 7: 3.0% |
| lgbm-ext | 49.6% | 20.7% | 3.2% | 4.42 | 1: 1.0% · 2: 6.0% · 3: 16.8% · 4: 26.6% · 5: 28.9% · 6: 17.5% · 7: 3.2% |
| logreg-ext | 47.4% | 19.1% | 2.8% | 4.36 | 1: 1.0% · 2: 6.4% · 3: 17.8% · 4: 27.3% · 5: 28.3% · 6: 16.3% · 7: 2.8% |
| Anchor | 67.8% | 34.2% | 5.8% | 4.94 | 1: 0.2% · 2: 2.0% · 3: 9.3% · 4: 20.7% · 5: 33.6% · 6: 28.4% · 7: 5.8% |

Per pick (probability it comes true; settled = 0% or 100% under every model):

| Pick | Team | Record | lgbm all | lgbm-ext | logreg-ext | Anchor | State |
|---|---|---|---:|---:|---:|---:|---|
| 3–0 | FUT Esports | 2–0 | 64.7% | 65.4% | 63.0% | 78.0% | open |
| 3–0 | Nemiga | 1–1 | 0.0% | 0.0% | 0.0% | 0.0% | wrong |
| 0–3 | FlyQuest | 1–1 | 0.0% | 0.0% | 0.0% | 0.0% | wrong |
| 0–3 | Lynn Vision | 0–2 | 67.0% | 68.4% | 70.9% | 89.5% | open |
| advance | G2 | 2–0 | 34.5% | 31.2% | 33.2% | 38.8% | open |
| advance | THE ONE | 2–0 | 43.6% | 40.7% | 36.3% | 43.1% | open |
| advance | Natus Vincere | 1–1 | 65.9% | 72.1% | 68.1% | 63.6% | open |
| advance | Astralis | 2–0 | 47.5% | 47.8% | 44.0% | 60.9% | open |
| advance | TheMongolz | 1–1 | 64.0% | 66.9% | 66.1% | 62.4% | open |
| advance | NIP | 1–1 | 49.4% | 49.0% | 53.8% | 57.5% | open |

## logreg-ext B (not pre-registered)

3–0: FUT Esports, TheMongolz; 0–3: FlyQuest, Lynn Vision; advance: Nemiga, Natus Vincere, NIP, Liquid, PARIVISION, 100 Thieves.

| Model | P(≥5) | P(≥6) | P(≥7) | Expected correct | Score distribution 0…10 |
|---|---:|---:|---:|---:|---|
| lgbm all | 35.2% | 8.0% | 0.6% | 4.08 | 1: 0.6% · 2: 6.0% · 3: 22.0% · 4: 36.2% · 5: 27.1% · 6: 7.5% · 7: 0.6% |
| lgbm-ext | 39.5% | 9.8% | 0.8% | 4.20 | 1: 0.4% · 2: 4.6% · 3: 19.6% · 4: 36.0% · 5: 29.7% · 6: 9.1% · 7: 0.8% |
| logreg-ext | 44.0% | 12.3% | 1.1% | 4.32 | 1: 0.2% · 2: 3.5% · 3: 17.3% · 4: 35.0% · 5: 31.7% · 6: 11.2% · 7: 1.1% |
| Anchor | 59.2% | 17.3% | 1.3% | 4.67 | 2: 0.9% · 3: 8.9% · 4: 31.0% · 5: 42.0% · 6: 16.0% · 7: 1.2% |

Per pick (probability it comes true; settled = 0% or 100% under every model):

| Pick | Team | Record | lgbm all | lgbm-ext | logreg-ext | Anchor | State |
|---|---|---|---:|---:|---:|---:|---|
| 3–0 | FUT Esports | 2–0 | 64.7% | 65.4% | 63.0% | 78.0% | open |
| 3–0 | TheMongolz | 1–1 | 0.0% | 0.0% | 0.0% | 0.0% | wrong |
| 0–3 | FlyQuest | 1–1 | 0.0% | 0.0% | 0.0% | 0.0% | wrong |
| 0–3 | Lynn Vision | 0–2 | 67.0% | 68.4% | 70.9% | 89.5% | open |
| advance | Nemiga | 1–1 | 55.7% | 65.1% | 62.9% | 77.6% | open |
| advance | Natus Vincere | 1–1 | 65.9% | 72.1% | 68.1% | 63.6% | open |
| advance | NIP | 1–1 | 49.4% | 49.0% | 53.8% | 57.5% | open |
| advance | Liquid | 1–1 | 48.6% | 41.2% | 50.8% | 43.1% | open |
| advance | PARIVISION | 0–2 | 17.9% | 15.1% | 14.2% | 20.4% | open |
| advance | 100 Thieves | 1–1 | 38.6% | 43.9% | 48.6% | 37.4% | open |

## Method

- Models and inputs: as in `singapore-2026-stage2.md` (`lgbm all`, anchor) and `singapore-2026-stage2-ext.md` (`lgbm-ext`, `logreg-ext`), rebuilt from the same saved pre-Stage files; the code refuses to run if those show a started Match.
- Conditioning: each finished Match's pair gets probability 1 (or 0) in both formats; the Swiss simulator (`kbml/swiss_gpu.py`) then forms every later pairing from the results by its Buchholz rule. Round 1 is checked against the opening, each listed later round against the simulator's own pairing, and no simulated run contradicts a known result (asserted).

