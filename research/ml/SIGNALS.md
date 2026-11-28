# New match-model signals (phase 5)

Five ideas from an external review, tested as add-ons to the two phase 3/4 forecasters on the 2,087
scored Matches. The review's effect sizes came from synthetic simulations: here every idea is a
hypothesis, and every factual claim it makes (for example the dates of map-pool changes) is
derived from our own archive or left out.

## Primary configuration (declared before its evaluation)

Written and committed before any of the signals below was computed or scored. Phases 1-4
(REPORT.md, PICKEM.md, FORECASTER.md) had been seen.

- **Base:** `lgbm-ext`: LightGBM on `p3_all`, extended archive, training rows scope + S/A 2023-24 +
  tier B + tier C (never a qualifier), walk-forward by Event exactly as phase 3 (`p3_all / +S/A+B+C`,
  log-loss 0.6198).
- **Signal A, two-stage market fusion:**
  - *Stage 1 (fundamental):* the same LightGBM, training rows and walk-forward, on `p3_all` without
    any market-derived column (`logit_anchor`, `logit_market`, `has_market` and the four
    `*_x_anchor` stakes interactions are dropped; the Valve rating columns stay). It is predicted
    out of sample for every Event that has a scored Match and for every non-qualifier Event of the
    training blocks that has a priced Match, each from a fit on Events that ended before that Event
    began.
  - *Stage 2 (calibrator):* for an evaluated Event E, a logistic regression without intercept
    (L2, C = 1) fitted only on priced Matches (a market price strictly before the cutoff) of
    non-qualifier Events of the training blocks that ended before E began, each carrying its own
    out-of-sample stage 1 probability. Inputs, all antisymmetric, 0 when the source has no price:
    `logit(p_fund)`; `logit` of Pinnacle de-vigged by the **power method** (last pre-cutoff price,
    the chain's OddsPapi rule: `pinnacle`, else `pinnacle+30`); `logit` of the last pre-cutoff
    Polymarket price; `logit` of the normalised Kalshi mid. Fewer than 100 stage 2 training rows →
    the Event keeps `lgbm-ext`'s own prediction.
  - A Match without a price gets the stage 1 probability.
- **Data and rule:** the priced scored Matches (`market_p` set: 777 Matches, 21 Events, 2026).
  Δ log-loss = (`lgbm-ext` + A) − `lgbm-ext` on the same Matches, Event-bootstrap 90% interval (2000
  resamples, seed 33). **A improves `lgbm-ext` on the market subset only if the interval lies wholly
  below zero.**

Everything else is exploratory: A on `logreg-ext` and on all 2,087 Matches; de-vig by
proportional and Shin; Kalshi spread / liquidity weighting; line movement; the CLV-style metric;
signals B (Bo3 veto with empirical-Bayes map strength), C (OpenSkill player ratings), D (travel)
and E (Hansen SPA over the configurations of phases 2-5).

<!-- results are added below after the run -->
