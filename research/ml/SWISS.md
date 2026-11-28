# Pick'em Swiss simulator with correlated tournament form (phase 5)

The phase 4 replay (FORECASTER.md) simulates every Match of a Swiss Stage as an independent draw
from the model's pairwise probability. Real Stages are not independent: a team that peaks (or
falls apart) does so for all its Matches, which fattens the tails of the score distribution. The
under-confident `lgbm-ext` P(≥5) (38.7% predicted, 14/28 = 50% realised) and the over-confident
anchor (85% predicted, 11/28) both hint at a miscalibrated spread. This page tests four ideas from
an external review whose effect sizes came from synthetic simulations (treated here as hypotheses,
none of its numbers is reused).

## Primary configuration (declared before its evaluation)

Written and committed before any variant was run on the replay. FORECASTER.md and PICKEM.md (the
28-Stage replay, the cached pairwise probabilities of `lgbm ext` and `logreg ext`) had been seen.

- **Probabilities:** logit ensemble 0.5·`logreg-ext` + 0.5·`lgbm-ext` (the cached pairwise
  probabilities of the phase 4 replay; no refit).
- **Correlated form:** per run, team i gets one shock η_i ~ N(0, σ_form² + σ_i²) added in logit to
  every Match of the Stage: P(i beats j) = σ(logit p_ij + η_i − η_j), BO1 and BO3 alike.
  - σ_i = 0.3 · u_i (fixed, not tuned), u_i the mean of three point-in-time scores in [0, 1]:
    thin recent play = clip((8 − n90)/8, 0, 1), n90 the team's Matches finished in the 90 days before
    the Stage cutoff (extended archive, any tier, qualifiers included); new roster = min(1, new/2),
    new = players of its latest known HLTV lineup absent from its lineups in the 120 days before that
    Match (no earlier lineup = 1); isolation = the share of the Stage's 15 other teams it did not meet
    in the 365 days before the cutoff (rarely met region).
  - σ_form ∈ {0, 0.15, 0.30, 0.45} chosen walk-forward for each Stage: the value with the lowest mean
    Brier of the "≥5 correct" event (strategy B's slip under that σ_form) over the Stages that
    finished before this Stage's cutoff. With fewer than 5 such Stages, σ_form = 0.30.
- **Strategy:** B, `pickem_check`'s exchange search for the slip with the highest simulated P(≥5)
  (λ = 0), on 200,000 runs per Stage, seed 33, the GPU simulator (`kbml/swiss_gpu.py`).
- **Data:** the 28 Stages of PICKEM.md (Singapore Stage 1 included, as in FORECASTER.md; a 27-Stage
  sensitivity without it is reported too).
- **Rules** (paired Stage bootstrap, 2000 resamples, 90%, seed 33):
  1. *Calibration:* primary is "better calibrated" than `logreg-ext` B only if the interval of the
     Brier(≥5) difference (primary − `logreg-ext` B) lies wholly below zero.
  2. *Score:* primary "reaches ≥5 in more Stages" than anchor B only if the interval of the
     difference in the number of Stages with ≥5 lies wholly above zero.

28 Stages give little power; a "no" can mean "not measurable here". Every other variant is
exploratory: the same ensemble or models without shocks, ensemble weight chosen walk-forward
(grid {0, .25, .5, .75, 1} on the log-loss of the earlier Stages' actual Matches), regularised
strategy B (maximise P(≥5) + λ·E[correct], λ ∈ {0, 0.02, 0.05}, chosen walk-forward by the earlier
Stages' realised share of ≥5, then realised mean correct, then smaller λ), and the between-stage
update (Stage 2/3 of a Major, and Shanghai's Elimination Stage, shift each team's logit by κ·Σ over
its Matches in the previous Stage of the same Event of (result − the model's probability), κ ∈
{0, .1, .2, .4} chosen walk-forward on the earlier eligible Stages' Match log-loss).

## Conclusion (written after the run)

1. **Primary (ensemble 0.5 + walk-forward σ_form, B): neither declared rule is met.** Brier of ≥5
   0.290 against 0.281 for `logreg-ext` B: +0.009 [−0.019, +0.036], so not "better calibrated"
   (the interval straddles zero, slightly on the worse side). Stages with ≥5: 15/28 against the
   anchor's 11/28: +4 [+0, +8], the lower bound sits at zero, so not "more often" by the rule
   (same verdict as `logreg-ext` B in phase 4, +5 [+0, +10]). Without Singapore Stage 1 the verdicts
   are the same (+0.011 [−0.017, +0.038]; +4 [+0, +8]).
2. **Correlated form does not help here.** It lowers the predicted P(≥5) (the shocks spread the score
   distribution but the mean sits below 5, so the P(≥5) of an under-confident model falls further
   from the 50% realised); the mean predicted P(≥5) goes from 44.9% (independent ensemble) to 43.6%
   (primary) and 39% at σ_form = 0.45. Brier of ≥5 is flat or worse as σ_form grows (0.282, 0.283,
   0.288, 0.292 at 0, .15, .30, .45). Only `logreg-ext` improves slightly with the walk-forward σ
   (Brier 0.270 against 0.281 independent; Δ vs logreg-ext B −0.011 [−0.046, +0.008]), inside noise.
   Walk-forward σ_form mostly picks 0 or 0.15; for `lgbm-ext` it picks 0 in 22 of 28 Stages.
3. **The ensemble is the only piece that moves Brier**, and only as walk-forward weight (0.265,
   −0.016 [−0.046, +0.013] vs logreg-ext B): about 4 to 6 thousandths of Brier from a 28-Stage
   selection, not distinguishable from zero. The 0.5 ensemble alone equals the single models
   (0.282). It gains in mean correct (4.68 against 4.50 and 4.54) but not in Stages ≥5.
4. **Regularised B is a no-op:** λ = 0 was selected in every Stage (the earlier Stages never favoured
   λ > 0 on realised ≥5), so those rows equal their λ = 0 twins. The λ > 0 slips were computed and cached but never entered a row; their separate effect was not
   inspected.
5. **Between-stage update:** κ = 0.2 chosen in 6 of 28 Stages (the 22 others had too few eligible
   earlier Stages or κ = 0); mean correct 4.68 → 4.71 but no change in Stages ≥5 or Brier. Not
   measurable on 9 eligible Stages.
6. **Noise floor:** re-drawing `logreg-ext` B with the GPU simulator gives 15/28 where the numpy
   draws gave 16/28; ≥5 counts differ by ±1 Stage from the draws alone, so differences of one Stage
   between rows mean nothing.

Net: with 28 Stages nothing here beats `logreg-ext` B or the independent simulator measurably; the
under-confidence of `lgbm-ext` (38.7% predicted, 50% realised) is not repaired by tournament-level
correlation, which pushes P(≥5) down, not up. For KallBet nothing changes from FORECASTER.md. All
variants except the primary are exploratory (the walk-forward choices are listed below).

## Method and cost

`kbml/swiss_form.py` (inputs, walk-forward engine, regularised search), `kbml/swiss_report.py`
(this table), `kbml/swiss_gpu.py` (optional per-run logit shocks from a second generator, so the
uniforms are identical with or without shocks; `tests/test_swiss_form.py` checks that zero shocks
reproduce the independent outcomes and that λ = 0 is `pickem_check.search`). The probabilities are
the phase 4 cached pairs (no refit). Calls: `<gpu python> -m kbml.swiss_report`. Timings for one Stage at
200,000 runs: numpy simulator 1.4 s, torch on the RTX 3060 0.15 s (batch of 200k, well under 1 GB);
the slip search (`pickem_check.search`, numpy bit-packed) stays on the CPU, 3 to 7 s for the three
λ values together, and dominates the run (about 1,100 simulations and searches, 2 h wall clock).

<!-- generated by kbml.swiss_report: everything below is rewritten -->

## All variants on the 28 Stages (strategy B unless stated)

| Variant | Mean correct | Stages ≥5 | Predicted P(≥5) | Realised − predicted [90%] | Brier of ≥5 | Δ Brier vs logreg-ext B [90%] | Δ Stages ≥5 vs anchor B [90%] | Δ Stages ≥5 vs logreg-ext B [90%] | Expected − realised correct |
|---|---:|---:|---:|---|---:|---|---|---|---:|
| Anchor B (phase 4, CPU draws) | 4.25 | 11/28 | 85.4% | -46.1 [-61.5, -30.4] pp | 0.465 | +0.183 [+0.050, +0.317] | +0.0 [+0.0, +0.0] | -5.0 [-10.0, +0.0] | +1.79 |
| logreg-ext B (phase 4, CPU draws) | 4.57 | 16/28 | 50.1% | +7.0 [-10.1, +23.0] pp | 0.281 | +0.000 [+0.000, +0.000] | +5.0 [+0.0, +10.0] | +0.0 [+0.0, +0.0] | -0.05 |
| lgbm-ext B (phase 4, CPU draws) | 4.50 | 14/28 | 38.7% | +11.3 [-5.2, +27.0] pp | 0.282 | +0.001 [-0.047, +0.049] | +3.0 [-2.0, +8.0] | -2.0 [-5.0, +1.0] | -0.40 |
| PRIMARY ensemble 0.5, σ_form walk-forward, B | 4.64 | 15/28 | 43.6% | +9.9 [-6.8, +26.7] pp | 0.290 | +0.009 [-0.019, +0.036] | +4.0 [+0.0, +8.0] | -1.0 [-4.0, +2.0] | -0.36 |
| logreg-ext, independent, B (GPU draws) | 4.54 | 15/28 | 50.1% | +3.4 [-13.7, +19.7] pp | 0.281 | -0.000 [-0.002, +0.001] | +4.0 [+0.0, +8.0] | -1.0 [-3.0, +0.0] | -0.02 |
| lgbm-ext, independent, B (GPU draws) | 4.50 | 14/28 | 38.7% | +11.3 [-5.2, +27.0] pp | 0.282 | +0.001 [-0.047, +0.048] | +3.0 [-2.0, +8.0] | -2.0 [-5.0, +1.0] | -0.40 |
| ensemble 0.5, independent, B | 4.68 | 15/28 | 44.9% | +8.7 [-7.8, +25.3] pp | 0.282 | +0.001 [-0.022, +0.024] | +4.0 [+0.0, +8.0] | -1.0 [-4.0, +2.0] | -0.35 |
| logreg-ext, σ_form walk-forward | 4.61 | 15/28 | 49.3% | +4.2 [-12.4, +20.4] pp | 0.270 | -0.011 [-0.046, +0.008] | +4.0 [-1.0, +9.0] | -1.0 [-3.0, +0.0] | -0.12 |
| lgbm-ext, σ_form walk-forward | 4.50 | 14/28 | 38.0% | +12.0 [-4.9, +28.0] pp | 0.290 | +0.008 [-0.043, +0.061] | +3.0 [-2.0, +8.0] | -2.0 [-5.0, +1.0] | -0.43 |
| ensemble 0.5, σ_form = 0 (σ_i only) | 4.61 | 15/28 | 44.7% | +8.8 [-7.6, +25.4] pp | 0.282 | +0.001 [-0.022, +0.025] | +4.0 [+0.0, +8.0] | -1.0 [-4.0, +2.0] | -0.29 |
| ensemble 0.5, σ_form = 0.15 | 4.71 | 16/28 | 44.0% | +13.1 [-3.2, +29.3] pp | 0.283 | +0.002 [-0.023, +0.027] | +5.0 [+0.0, +10.0] | +0.0 [-2.0, +2.0] | -0.42 |
| ensemble 0.5, σ_form = 0.30 | 4.64 | 16/28 | 42.0% | +15.2 [-1.2, +31.3] pp | 0.288 | +0.007 [-0.025, +0.038] | +5.0 [+0.0, +10.0] | +0.0 [-2.0, +2.0] | -0.43 |
| ensemble 0.5, σ_form = 0.45 | 4.57 | 15/28 | 39.1% | +14.4 [-1.9, +30.9] pp | 0.292 | +0.011 [-0.028, +0.050] | +4.0 [+0.0, +8.0] | -1.0 [-4.0, +2.0] | -0.46 |
| ensemble weight walk-forward, independent, B | 4.61 | 14/28 | 43.9% | +6.1 [-9.9, +21.9] pp | 0.265 | -0.016 [-0.046, +0.013] | +3.0 [-2.0, +8.0] | -2.0 [-5.0, +1.0] | -0.32 |
| ensemble weight walk-forward, σ_form walk-forward, B | 4.57 | 14/28 | 42.6% | +7.4 [-8.9, +23.5] pp | 0.273 | -0.009 [-0.042, +0.025] | +3.0 [-2.0, +8.0] | -2.0 [-5.0, +1.0] | -0.33 |
| ensemble 0.5, independent, λ walk-forward | 4.68 | 15/28 | 44.9% | +8.7 [-7.8, +25.3] pp | 0.282 | +0.001 [-0.022, +0.024] | +4.0 [+0.0, +8.0] | -1.0 [-4.0, +2.0] | -0.35 |
| ensemble 0.5, σ_form walk-forward, λ walk-forward | 4.64 | 15/28 | 43.6% | +9.9 [-6.8, +26.7] pp | 0.290 | +0.009 [-0.019, +0.036] | +4.0 [+0.0, +8.0] | -1.0 [-4.0, +2.0] | -0.36 |
| ensemble 0.5, independent, between-stage κ walk-forward | 4.71 | 15/28 | 44.8% | +8.8 [-7.6, +25.3] pp | 0.282 | +0.001 [-0.022, +0.024] | +4.0 [+0.0, +8.0] | -1.0 [-4.0, +2.0] | -0.39 |
| ensemble 0.5, σ_form walk-forward, between-stage κ walk-forward | 4.68 | 15/28 | 43.6% | +10.0 [-6.7, +26.8] pp | 0.289 | +0.008 [-0.019, +0.036] | +4.0 [+0.0, +8.0] | -1.0 [-4.0, +2.0] | -0.40 |
| all four walk-forward (w, κ, σ_form, λ) | 4.61 | 14/28 | 42.5% | +7.5 [-8.8, +23.5] pp | 0.272 | -0.009 [-0.043, +0.025] | +3.0 [-2.0, +8.0] | -2.0 [-5.0, +1.0] | -0.37 |

Intervals: paired Stage bootstrap (2000 resamples, 90%, seed 33). Realised − predicted: share of Stages with ≥5 correct minus mean simulated P(≥5) (negative = over-confident). Brier of ≥5: mean (simulated P(≥5) − [≥5 happened])². Deltas in Stages are counts. The three reference rows are the phase 4 numbers (numpy draws); the other rows use the GPU simulator (seed 33, 200,000 runs per Stage).

## Same, without Singapore Stage 1 (27 Stages; its last results postdate the holdout line)

| Variant | Mean correct | Stages ≥5 | Predicted P(≥5) | Realised − predicted [90%] | Brier of ≥5 | Δ Brier vs logreg-ext B [90%] | Δ Stages ≥5 vs anchor B [90%] | Δ Stages ≥5 vs logreg-ext B [90%] | Expected − realised correct |
|---|---:|---:|---:|---|---:|---|---|---|---:|
| Anchor B (phase 4, CPU draws) | 4.30 | 11/27 | 86.7% | -46.0 [-62.2, -29.2] pp | 0.472 | +0.188 [+0.050, +0.330] | +0.0 [+0.0, +0.0] | -5.0 [-10.0, +0.0] | +1.80 |
| logreg-ext B (phase 4, CPU draws) | 4.59 | 16/27 | 50.3% | +8.9 [-8.1, +25.9] pp | 0.285 | +0.000 [+0.000, +0.000] | +5.0 [+0.0, +10.0] | +0.0 [+0.0, +0.0] | -0.06 |
| lgbm-ext B (phase 4, CPU draws) | 4.56 | 14/27 | 38.9% | +13.0 [-3.1, +29.0] pp | 0.288 | +0.004 [-0.045, +0.052] | +3.0 [-2.0, +8.0] | -2.0 [-5.0, +1.0] | -0.45 |
| PRIMARY ensemble 0.5, σ_form walk-forward, B | 4.67 | 15/27 | 43.8% | +11.7 [-5.7, +28.1] pp | 0.295 | +0.011 [-0.017, +0.038] | +4.0 [+0.0, +8.0] | -1.0 [-4.0, +2.0] | -0.38 |
| logreg-ext, independent, B (GPU draws) | 4.56 | 15/27 | 50.4% | +5.2 [-12.4, +22.2] pp | 0.284 | -0.000 [-0.002, +0.001] | +4.0 [+0.0, +9.0] | -1.0 [-3.0, +0.0] | -0.03 |
| lgbm-ext, independent, B (GPU draws) | 4.56 | 14/27 | 38.9% | +12.9 [-3.1, +29.0] pp | 0.288 | +0.003 [-0.045, +0.052] | +3.0 [-2.0, +8.0] | -2.0 [-5.0, +1.0] | -0.45 |
| ensemble 0.5, independent, B | 4.70 | 15/27 | 45.1% | +10.4 [-6.9, +26.5] pp | 0.287 | +0.003 [-0.021, +0.026] | +4.0 [+0.0, +8.0] | -1.0 [-4.0, +2.0] | -0.37 |
| logreg-ext, σ_form walk-forward | 4.63 | 15/27 | 49.6% | +6.0 [-11.1, +22.2] pp | 0.273 | -0.011 [-0.047, +0.009] | +4.0 [-1.0, +9.0] | -1.0 [-3.0, +0.0] | -0.13 |
| lgbm-ext, σ_form walk-forward | 4.59 | 14/27 | 38.2% | +13.6 [-2.6, +30.1] pp | 0.296 | +0.012 [-0.041, +0.064] | +3.0 [-2.0, +8.0] | -2.0 [-5.0, +1.0] | -0.52 |
| ensemble 0.5, σ_form = 0 (σ_i only) | 4.63 | 15/27 | 45.0% | +10.6 [-6.7, +26.6] pp | 0.287 | +0.003 [-0.022, +0.026] | +4.0 [+0.0, +8.0] | -1.0 [-4.0, +2.0] | -0.30 |
| ensemble 0.5, σ_form = 0.15 | 4.78 | 16/27 | 44.2% | +15.0 [-1.6, +30.9] pp | 0.288 | +0.003 [-0.023, +0.028] | +5.0 [+0.0, +10.0] | +0.0 [-2.0, +2.0] | -0.48 |
| ensemble 0.5, σ_form = 0.30 | 4.70 | 16/27 | 42.2% | +17.1 [+0.5, +32.9] pp | 0.294 | +0.009 [-0.023, +0.040] | +5.0 [+0.0, +10.0] | +0.0 [-2.0, +2.0] | -0.48 |
| ensemble 0.5, σ_form = 0.45 | 4.63 | 15/27 | 39.4% | +16.2 [-0.9, +32.1] pp | 0.299 | +0.014 [-0.027, +0.053] | +4.0 [+0.0, +8.0] | -1.0 [-4.0, +2.0] | -0.51 |
| ensemble weight walk-forward, independent, B | 4.67 | 14/27 | 44.2% | +7.7 [-8.4, +23.8] pp | 0.270 | -0.015 [-0.044, +0.016] | +3.0 [-2.0, +8.0] | -2.0 [-5.0, +1.0] | -0.37 |
| ensemble weight walk-forward, σ_form walk-forward, B | 4.63 | 14/27 | 42.9% | +9.0 [-7.4, +25.3] pp | 0.278 | -0.007 [-0.039, +0.027] | +3.0 [-2.0, +8.0] | -2.0 [-5.0, +1.0] | -0.38 |
| ensemble 0.5, independent, λ walk-forward | 4.70 | 15/27 | 45.1% | +10.4 [-6.9, +26.5] pp | 0.287 | +0.003 [-0.021, +0.026] | +4.0 [+0.0, +8.0] | -1.0 [-4.0, +2.0] | -0.37 |
| ensemble 0.5, σ_form walk-forward, λ walk-forward | 4.67 | 15/27 | 43.8% | +11.7 [-5.7, +28.1] pp | 0.295 | +0.011 [-0.017, +0.038] | +4.0 [+0.0, +8.0] | -1.0 [-4.0, +2.0] | -0.38 |
| ensemble 0.5, independent, between-stage κ walk-forward | 4.74 | 15/27 | 45.0% | +10.5 [-6.7, +26.5] pp | 0.287 | +0.002 [-0.022, +0.025] | +4.0 [+0.0, +8.0] | -1.0 [-4.0, +2.0] | -0.41 |
| ensemble 0.5, σ_form walk-forward, between-stage κ walk-forward | 4.70 | 15/27 | 43.8% | +11.8 [-5.6, +28.1] pp | 0.295 | +0.010 [-0.018, +0.038] | +4.0 [+0.0, +8.0] | -1.0 [-4.0, +2.0] | -0.42 |
| all four walk-forward (w, κ, σ_form, λ) | 4.67 | 14/27 | 42.8% | +9.1 [-7.2, +25.3] pp | 0.278 | -0.007 [-0.040, +0.027] | +3.0 [-2.0, +8.0] | -2.0 [-5.0, +1.0] | -0.42 |

## Choices made walk-forward

| Variant | weights used (lgbm share) | κ used | σ_form used | λ used |
|---|---|---|---|---|
| PRIMARY ensemble 0.5, σ_form walk-forward, B | 0.5: 28 | 0.0: 28 | 0.0: 11, 0.15: 10, 0.3: 5, 0.45: 2 | 0.0: 28 |
| logreg-ext, σ_form walk-forward | 0.0: 28 | 0.0: 28 | 0.0: 17, 0.15: 6, 0.3: 5 | 0.0: 28 |
| lgbm-ext, σ_form walk-forward | 1.0: 28 | 0.0: 28 | 0.0: 22, 0.3: 5, 0.45: 1 | 0.0: 28 |
| ensemble weight walk-forward, independent, B | 0.0: 4, 0.25: 1, 0.5: 2, 0.75: 11, 1.0: 10 | 0.0: 28 | None: 28 | 0.0: 28 |
| ensemble weight walk-forward, σ_form walk-forward, B | 0.0: 4, 0.25: 1, 0.5: 2, 0.75: 11, 1.0: 10 | 0.0: 28 | 0.0: 17, 0.15: 1, 0.3: 8, 0.45: 2 | 0.0: 28 |
| ensemble 0.5, σ_form walk-forward, λ walk-forward | 0.5: 28 | 0.0: 28 | 0.0: 11, 0.15: 10, 0.3: 5, 0.45: 2 | 0.0: 28 |
| ensemble 0.5, independent, between-stage κ walk-forward | 0.5: 28 | 0.0: 22, 0.2: 6 | None: 28 | 0.0: 28 |
| ensemble 0.5, σ_form walk-forward, between-stage κ walk-forward | 0.5: 28 | 0.0: 22, 0.2: 6 | 0.0: 11, 0.15: 10, 0.3: 5, 0.45: 2 | 0.0: 28 |
| all four walk-forward (w, κ, σ_form, λ) | 0.0: 4, 0.25: 1, 0.5: 2, 0.75: 11, 1.0: 10 | 0.0: 22, 0.2: 6 | 0.0: 17, 0.15: 1, 0.3: 8, 0.45: 2 | 0.0: 28 |

## Per Stage: primary against the references

| Stage | σ_form chosen | Correct (P≥5) primary | logreg-ext B | lgbm-ext B | Anchor B | mean σ_i |
|---|---:|---|---|---|---|---:|
| Perfect World Major Shanghai Major 2024 / Opening Stage | 0.30 | 6 (28%) | 5 (35%) | 5 (25%) | 3 (87%) | 0.062 |
| Perfect World Major Shanghai Major 2024 / Elimination Stage | 0.30 | 5 (30%) | 5 (37%) | 6 (28%) | 6 (81%) | 0.035 |
| PGL Cluj-Napoca 2025 / Group Stage | 0.30 | 7 (39%) | 7 (49%) | 5 (35%) | 6 (94%) | 0.040 |
| ESL Pro League Season 21 2025 / Group Stage | 0.30 | 9 (50%) | 9 (64%) | 9 (39%) | 9 (97%) | 0.020 |
| PGL Bucharest 2025 / Group Stage | 0.30 | 5 (50%) | 5 (66%) | 5 (36%) | 2 (97%) | 0.055 |
| PGL Astana 2025 / Group Stage | 0.00 | 4 (60%) | 4 (73%) | 4 (41%) | 3 (98%) | 0.045 |
| BLAST.tv Major Austin 2025 / Stage 1 | 0.00 | 4 (37%) | 5 (43%) | 4 (32%) | 6 (77%) | 0.067 |
| BLAST.tv Major Austin 2025 / Stage 2 | 0.00 | 2 (33%) | 2 (38%) | 2 (29%) | 2 (92%) | 0.040 |
| BLAST.tv Major Austin 2025 / Stage 3 | 0.00 | 4 (52%) | 6 (58%) | 4 (45%) | 4 (95%) | 0.028 |
| FISSURE PLAYGROUND Season 2 2025 / Group Stage | 0.15 | 4 (31%) | 2 (35%) | 4 (29%) | 3 (93%) | 0.026 |
| ESL Pro League Season 22 2025 / Stage 1 | 0.15 | 4 (65%) | 4 (74%) | 4 (54%) | 4 (98%) | 0.091 |
| ESL Pro League Season 22 2025 / Stage 2 | 0.15 | 4 (54%) | 3 (62%) | 4 (46%) | 4 (96%) | 0.036 |
| PGL Masters Bucharest 2025 / Group Stage | 0.45 | 3 (30%) | 3 (37%) | 3 (32%) | 2 (83%) | 0.033 |
| StarLadder Major Budapest 2025 / Stage 1 | 0.45 | 5 (36%) | 4 (49%) | 5 (34%) | 6 (88%) | 0.075 |
| StarLadder Major Budapest 2025 / Stage 2 | 0.15 | 5 (25%) | 5 (27%) | 5 (25%) | 5 (83%) | 0.038 |
| StarLadder Major Budapest 2025 / Stage 3 | 0.15 | 5 (39%) | 5 (41%) | 6 (39%) | 4 (84%) | 0.028 |
| PGL Cluj-Napoca 2026 / Group Stage | 0.15 | 6 (44%) | 6 (50%) | 6 (38%) | 6 (75%) | 0.023 |
| ESL Pro League Season 23 2026 / Stage 1 | 0.00 | 4 (57%) | 4 (59%) | 4 (54%) | 5 (92%) | 0.062 |
| ESL Pro League Season 23 2026 / Stage 2 | 0.15 | 5 (40%) | 5 (43%) | 4 (39%) | 5 (86%) | 0.025 |
| PGL Bucharest 2026 / Group Stage | 0.15 | 2 (47%) | 2 (52%) | 2 (41%) | 2 (92%) | 0.060 |
| PGL Astana 2026 / Group Stage | 0.15 | 6 (64%) | 6 (69%) | 6 (59%) | 6 (93%) | 0.071 |
| IEM Cologne Major 2026 / Stage 1 | 0.15 | 5 (45%) | 5 (49%) | 5 (42%) | 4 (67%) | 0.087 |
| IEM Cologne Major 2026 / Stage 2 | 0.00 | 5 (38%) | 5 (40%) | 5 (36%) | 5 (77%) | 0.047 |
| IEM Cologne Major 2026 / Stage 3 | 0.00 | 3 (35%) | 3 (38%) | 2 (32%) | 4 (75%) | 0.018 |
| XSE Pro League 2026 / Group Stage | 0.00 | 6 (38%) | 6 (40%) | 6 (36%) | 2 (52%) | 0.067 |
| ESL Pro League Season 24 2026 / Group Stage | 0.00 | 5 (64%) | 5 (71%) | 5 (54%) | 4 (96%) | 0.035 |
| PGL Masters Bucharest 2026 / Group Stage | 0.00 | 3 (55%) | 3 (59%) | 3 (49%) | 4 (92%) | 0.030 |
| PGL Major Singapore 2026 / Stage 1 | 0.00 | 4 (38%) | 4 (44%) | 3 (34%) | 3 (50%) | 0.073 |
