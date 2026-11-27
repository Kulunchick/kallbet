# Calibrated pairwise forecaster for Pick'em (phase 4)

KallBet's Pick'em and catch-up decisions need honest probabilities, not only the right favourite.
This page tests whether the phase 3 best-log-loss model, used as a pairwise forecaster (any two
teams, a format and a cutoff), gives better calibrated Pick'em slips than the anchor.

## Primary configuration (declared before its evaluation)

Written and committed before the ext forecasters were run on the Pick'em replay. PICKEM.md (the
in-scope models and the anchor on the same 28 Stages) and phases 1-3 had been seen.

- **Forecaster:** `lgbm-ext` (`kbml/forecaster.py`): LightGBM on `p3_all`, fitted for each Stage on
  every non-qualifier Match of the extended archive (in-scope, S/A 2023-24, tier B, tier C) whose
  result was known strictly before the Stage opened; hyperparameters by inner walk-forward on
  completed earlier Events, validated on in-scope Matches. Hypothetical pairings have no match page:
  page-specific features take the missing-data path.
- **Strategy:** B (maximise the simulated chance of ≥5 correct picks), with `pickem_check`'s
  simulation (200,000 runs per Stage, seed 33) and search, unchanged.
- **Data:** the 28 completed Swiss Stages of PICKEM.md.
- **Questions and rules** (paired Stage bootstrap over the 28 Stages, 2000 resamples, 90%, seed 33):
  1. *Calibration (the point of this phase):* the Brier score of the "≥5 correct" event, simulated
     P(≥5) against what happened, compared with anchor B. `lgbm-ext` B is "better calibrated for
     Pick'em" only if the interval of the Brier difference (lgbm-ext B − anchor B) lies wholly below
     zero.
  2. *Score:* `lgbm-ext` B "reaches ≥5 more often than anchor B" only if the interval of the
     difference in the share of Stages with ≥5 lies wholly above zero.

28 Stages give little power; a "no" on either rule can mean "not measurable here". Everything else
(strategy A, `logreg-ext`, the in-scope models, per-Match reliability bins, expected vs realised
points) is exploratory.
