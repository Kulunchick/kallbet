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
