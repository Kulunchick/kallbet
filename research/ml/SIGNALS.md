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

## Conclusion (written after the run)

1. **Primary: no.** Two-stage market fusion on `lgbm-ext`, on the 777 priced scored Matches (21
   Events): Δ log-loss +0.0083 [−0.0098, +0.0331], not wholly below zero. Its Brier and resolution
   do not move either (−0.0003, −0.0005). It picks 1.2 pp more winners [+0.2, +2.2], but accuracy
   is not the declared metric, and after multiple testing (E) this does not survive. `lgbm-ext`
   already reads the chain's market logit, so a second-stage recombination has little left to add.
   With three per-source weights (0 where a source has no price), stage 2 is noisier than one chain
   logit: the variant `fund_market` (logit(p_fund) + the chain's market logit) is the best A
   variant, −0.0058 [−0.0135, +0.0018]. It is exploratory and its interval touches zero.
2. **De-vig method does not matter here.** On 301 Pinnacle prices (mean overround 1.050), power and
   Shin are within ±0.001 log-loss of proportional, and every interval spans zero. Fusion results
   are the same to the fourth decimal whichever method is used.
3. **Liquidity / spread: barely testable.** Kalshi spreads are 1¢ on nearly every scored Match
   (mean 1.1¢). Polymarket's only volume field is the after-the-Match total, so it is not point in
   time and is not used. Pinnacle stake limits and Kalshi 24 h volume as interaction weights
   (`liquidity`) change nothing measurable.
4. **Line movement / CLV.** Over the last ~12 h, lines move slightly toward the stage-1 model
   (no market input): mean CLV +0.016 logit [+0.006, +0.026]. Of the moved lines, 53% [49%, 57%]
   moved toward the model. So the model holds a little information the early market lacked and the
   close absorbed. The close forecasts no better than the early price (Δ log-loss +0.0008
   [−0.0026, +0.0040]). Adding the movement to stage 2 does not help.
5. **B, Bo3 veto with empirical-Bayes map strength.** The pool changes were derived from our own
   vetoes (table below; the detector sees each change within a few weeks of the first vetoes on the new
   pool). On its own, the simulation picks −2.1 pp [−3.6, −0.7] versus the anchor on 1,774 Bo3s
   (the lab's `bo3_veto_sim`: −2.8 pp). Blended 0.8 market + 0.2 simulation, it picks −1.7 pp
   [−3.0, −0.5]. Without shrinkage it is clearly worse (−4.0 pp [−5.4, −2.6]). So shrinkage removes
   most of the damage that noisy per-map ratings do, but a map simulation still picks fewer winners
   than the market. The pool reset changes little. As an add-on to `lgbm-ext`, log-loss is
   unchanged (+0.0006 [−0.0010, +0.0022]) and accuracy is +0.6 pp [+0.1, +1.1] (exploratory; does
   not survive E).
6. **C, OpenSkill player ratings.** This is the one add-on whose log-loss interval lies below zero:
   `lgbm-ext` + `openskill` −0.0019 [−0.0036, −0.0001], with resolution +0.0015 [+0.0001, +0.0028].
   Replacing the `roster` group by it instead (−0.0006 [−0.0026, +0.0014]) loses most of that gain.
   The two carry different information, so OpenSkill does not replace the roster flags. On
   `logreg-ext` it is neutral (+0.0002). Exploratory, and it does not survive the SPA test (p = 0.30
   for the add-on family).
7. **D, travel: no gain, slightly harmful in logreg.** Inputs: the Stage's country (PandaScore,
   else the Liquipedia infobox `|country=`, 45 Stages filled) and each team's PandaScore location.
   The location is a fetch-time attribute of the team, not dated, so treat it as approximate. On
   `lgbm-ext`: +0.0010 [−0.0006, +0.0027]. On `logreg-ext`: +0.0031 [+0.0006, +0.0057].
8. **E, multiple testing.** Hansen SPA over 112 configurations of phases 2-5 against the anchor
   (Event block bootstrap). Log-loss: every configuration beats the anchor (p < 0.001, all 112
   survive the stepwise test), as known: the anchor is over-confident. Accuracy: no configuration
   picks more winners than the anchor (best +0.6 pp, p = 0.82, no survivor). Phase 5 add-ons
   against their own base (25 configurations): none survives on log-loss (p = 0.30) or accuracy
   (p = 0.24). A variants against the base on the priced Matches: none survives (p = 0.36 log-loss,
   p = 0.11 accuracy).

For KallBet: none of the five ideas earns a change to `lgbm-ext`. The ones worth a pre-registered
re-test when more priced Events exist: OpenSkill as an add-on (C), and fusion with a single
chain-market logit (A `fund_market`).

## Phase 5b: rolling player history (F), primary declared before its evaluation

Written and committed before F was computed or scored, and before the baselines were rerun on the
recovered links. Phase 5 above had been seen.

- **Data (W5, `docs/research-hltv-w5.md` on `Kulunchick/ml-data2` @ 7be0a75):** the extended
  archive with W5's 11,214 HLTV pages (1,798 new; 566 earlier raw-only pages now parsed) and its
  Match → page list `derived/recovery/links.tsv` (9,306 links) in place of the name-and-time
  linker (`KBML_HLTV_LINKS`). The Match table and HLTV caches are rebuilt under a separate cache
  root (`KBML_CACHE=D:/Projects/KallBet/target/ml-cache-w5`). The scored set stays the 2,087
  anchor Matches.
- **Bases, rerun on the recovered links:** `lgbm-ext` (LightGBM, `p3_all`) and `logreg-ext`
  (logreg, `p3_rw`). Both keep phase 3's training blocks, walk-forward and inner validation.
- **F (`player_hist`):** W5's per-player per-map table `hltv_player_match_stats.parquet`. Only
  rows of Matches whose linked PandaScore Match finished strictly before the cutoff are used
  (released with the HLTV record). Rating 3.0, Round Swing, eK−eD per round, eADR and eKAST come
  only from Rating 3.0 pages (Matches from 2023-10-16). The pages have no "Eco Swing" column, so
  eK−eD stands in for it.
  - Per player: the last 20 maps, shrunk toward the team's recent mean (its last 100
    player-maps) with 5 pseudo-maps.
  - Per lineup: the own page's five, else the team's last known five. Features are the mean and
    the max of each stat, core-five continuity (log of the maps those five played together for the
    team), and coverage.
  - F is added to both bases.
- **Primary and rule:** `lgbm-ext` + F against `lgbm-ext` (both on the recovered links) on the
  2,087 scored Matches. Δ log-loss, Event bootstrap 90% interval (2000 resamples, seed 33). **F
  improves `lgbm-ext` only if the interval lies wholly below zero.** Everything else is
  exploratory: `logreg-ext` + F, the market subset, the effect of the recovered links on the bases,
  F without the existing `players` group, and F in the SPA set.

<!-- generated by kbml.signals_report: everything below is rewritten -->

## Primary result

| Configuration | Subset | Matches (Events) | Log-loss base → variant | Δ log-loss [90%] | Δ Brier [90%] | Δ resolution [90%] | Accuracy base → variant | Δ accuracy [90%] |
|---|---|---:|---|---|---|---|---|---|
| lgbm-ext + A (core) |  | 777 (21) | 0.6153 → 0.6235 | +0.0083 [-0.0098, +0.0331] | -0.0003 [-0.0054, +0.0059] | -0.0005 [-0.0066, +0.0050] | 65.8% → 66.9% | +1.2 pp [+0.2, +2.2] |

By the declared rule, two-stage market fusion **does not measurably improve** `lgbm-ext` on the priced Matches (Δ log-loss interval [-0.0098, +0.0331]). Stage 2 fell back to `lgbm-ext` for 2 of the 21 priced Events (fewer than 100 earlier priced training Matches).

## A: market fusion (exploratory variants)

| Configuration | Subset | Matches (Events) | Log-loss base → variant | Δ log-loss [90%] | Δ Brier [90%] | Δ resolution [90%] | Accuracy base → variant | Δ accuracy [90%] |
|---|---|---:|---|---|---|---|---|---|
| lgbm-ext + A[core] | market | 777 (21) | 0.6153 → 0.6235 | +0.0083 [-0.0098, +0.0331] | -0.0003 [-0.0054, +0.0059] | -0.0005 [-0.0066, +0.0050] | 65.8% → 66.9% | +1.2 pp [+0.2, +2.2] |
| lgbm-ext + A[core] | all | 2087 (56) | 0.6198 → 0.6224 | +0.0026 [-0.0042, +0.0124] | -0.0003 [-0.0023, +0.0022] | +0.0006 [-0.0014, +0.0027] | 65.3% → 65.8% | +0.5 pp [+0.0, +1.1] |
| lgbm-ext + A[proportional] | market | 777 (21) | 0.6153 → 0.6234 | +0.0081 [-0.0099, +0.0329] | -0.0003 [-0.0054, +0.0058] | -0.0005 [-0.0064, +0.0050] | 65.8% → 66.9% | +1.2 pp [+0.2, +2.2] |
| lgbm-ext + A[proportional] | all | 2087 (56) | 0.6198 → 0.6224 | +0.0025 [-0.0042, +0.0123] | -0.0003 [-0.0023, +0.0022] | +0.0006 [-0.0013, +0.0027] | 65.3% → 65.8% | +0.5 pp [+0.0, +1.1] |
| lgbm-ext + A[shin] | market | 777 (21) | 0.6153 → 0.6234 | +0.0082 [-0.0098, +0.0329] | -0.0003 [-0.0054, +0.0058] | -0.0005 [-0.0064, +0.0050] | 65.8% → 66.9% | +1.2 pp [+0.2, +2.2] |
| lgbm-ext + A[shin] | all | 2087 (56) | 0.6198 → 0.6224 | +0.0026 [-0.0042, +0.0123] | -0.0003 [-0.0023, +0.0022] | +0.0006 [-0.0013, +0.0027] | 65.3% → 65.8% | +0.5 pp [+0.0, +1.1] |
| lgbm-ext + A[liquidity] | market | 777 (21) | 0.6153 → 0.6204 | +0.0051 [-0.0120, +0.0281] | -0.0013 [-0.0061, +0.0043] | +0.0008 [-0.0049, +0.0064] | 65.8% → 66.9% | +1.2 pp [+0.2, +2.2] |
| lgbm-ext + A[liquidity] | all | 2087 (56) | 0.6198 → 0.6213 | +0.0014 [-0.0049, +0.0102] | -0.0007 [-0.0025, +0.0015] | +0.0012 [-0.0008, +0.0032] | 65.3% → 65.8% | +0.5 pp [+0.0, +1.1] |
| lgbm-ext + A[movement] | market | 777 (21) | 0.6153 → 0.6248 | +0.0095 [-0.0093, +0.0350] | +0.0000 [-0.0056, +0.0070] | +0.0000 [-0.0057, +0.0056] | 65.8% → 66.9% | +1.2 pp [+0.2, +2.2] |
| lgbm-ext + A[movement] | all | 2087 (56) | 0.6198 → 0.6229 | +0.0031 [-0.0040, +0.0133] | -0.0002 [-0.0024, +0.0025] | +0.0008 [-0.0012, +0.0029] | 65.3% → 65.8% | +0.5 pp [+0.0, +1.1] |
| lgbm-ext + A[pin_only] | market | 777 (21) | 0.6153 → 0.6174 | +0.0021 [-0.0042, +0.0100] | +0.0005 [-0.0015, +0.0028] | -0.0022 [-0.0066, +0.0019] | 65.8% → 66.8% | +1.0 pp [+0.0, +2.2] |
| lgbm-ext + A[pin_only] | all | 2087 (56) | 0.6198 → 0.6201 | +0.0003 [-0.0023, +0.0035] | -0.0000 [-0.0009, +0.0010] | -0.0001 [-0.0016, +0.0014] | 65.3% → 65.7% | +0.5 pp [-0.1, +1.0] |
| lgbm-ext + A[fund_market] | market | 777 (21) | 0.6153 → 0.6094 | -0.0058 [-0.0135, +0.0018] | -0.0029 [-0.0062, +0.0003] | -0.0001 [-0.0044, +0.0050] | 65.8% → 67.2% | +1.4 pp [+0.1, +2.8] |
| lgbm-ext + A[fund_market] | all | 2087 (56) | 0.6198 → 0.6172 | -0.0026 [-0.0057, +0.0003] | -0.0013 [-0.0026, -0.0000] | +0.0009 [-0.0008, +0.0028] | 65.3% → 65.9% | +0.6 pp [+0.0, +1.3] |
| logreg-ext + A[core] | market | 777 (21) | 0.6117 → 0.6238 | +0.0121 [-0.0066, +0.0389] | +0.0021 [-0.0029, +0.0088] | -0.0005 [-0.0046, +0.0034] | 66.7% → 66.7% | +0.0 pp [-1.3, +1.0] |
| logreg-ext + A[core] | all | 2087 (56) | 0.6235 → 0.6279 | +0.0043 [-0.0028, +0.0146] | +0.0008 [-0.0012, +0.0034] | -0.0000 [-0.0017, +0.0015] | 65.8% → 65.7% | -0.1 pp [-0.6, +0.3] |
| logreg-ext + A[proportional] | market | 777 (21) | 0.6117 → 0.6236 | +0.0119 [-0.0067, +0.0387] | +0.0021 [-0.0029, +0.0088] | -0.0006 [-0.0048, +0.0032] | 66.7% → 66.7% | +0.0 pp [-1.3, +1.0] |
| logreg-ext + A[proportional] | all | 2087 (56) | 0.6235 → 0.6278 | +0.0043 [-0.0028, +0.0146] | +0.0007 [-0.0012, +0.0034] | -0.0001 [-0.0018, +0.0015] | 65.8% → 65.7% | -0.1 pp [-0.6, +0.3] |
| logreg-ext + A[shin] | market | 777 (21) | 0.6117 → 0.6237 | +0.0120 [-0.0067, +0.0388] | +0.0021 [-0.0029, +0.0088] | -0.0008 [-0.0048, +0.0029] | 66.7% → 66.7% | +0.0 pp [-1.3, +1.0] |
| logreg-ext + A[shin] | all | 2087 (56) | 0.6235 → 0.6278 | +0.0043 [-0.0028, +0.0146] | +0.0007 [-0.0012, +0.0034] | -0.0001 [-0.0018, +0.0014] | 65.8% → 65.7% | -0.1 pp [-0.6, +0.3] |
| logreg-ext + A[liquidity] | market | 777 (21) | 0.6117 → 0.6205 | +0.0088 [-0.0080, +0.0327] | +0.0012 [-0.0033, +0.0069] | -0.0001 [-0.0038, +0.0034] | 66.7% → 66.9% | +0.3 pp [-0.9, +1.2] |
| logreg-ext + A[liquidity] | all | 2087 (56) | 0.6235 → 0.6266 | +0.0031 [-0.0032, +0.0123] | +0.0004 [-0.0013, +0.0027] | +0.0001 [-0.0012, +0.0015] | 65.8% → 65.8% | +0.0 pp [-0.4, +0.4] |
| logreg-ext + A[movement] | market | 777 (21) | 0.6117 → 0.6253 | +0.0137 [-0.0054, +0.0412] | +0.0026 [-0.0028, +0.0100] | -0.0007 [-0.0049, +0.0035] | 66.7% → 66.8% | +0.1 pp [-1.1, +1.2] |
| logreg-ext + A[movement] | all | 2087 (56) | 0.6235 → 0.6284 | +0.0049 [-0.0024, +0.0157] | +0.0009 [-0.0012, +0.0038] | -0.0001 [-0.0018, +0.0015] | 65.8% → 65.7% | -0.0 pp [-0.5, +0.4] |
| logreg-ext + A[pin_only] | market | 777 (21) | 0.6117 → 0.6167 | +0.0050 [-0.0053, +0.0149] | +0.0026 [-0.0008, +0.0061] | -0.0009 [-0.0038, +0.0021] | 66.7% → 66.8% | +0.1 pp [-1.0, +1.3] |
| logreg-ext + A[pin_only] | all | 2087 (56) | 0.6235 → 0.6252 | +0.0017 [-0.0021, +0.0053] | +0.0009 [-0.0003, +0.0022] | -0.0003 [-0.0015, +0.0010] | 65.8% → 65.7% | -0.0 pp [-0.5, +0.4] |
| logreg-ext + A[fund_market] | market | 777 (21) | 0.6117 → 0.6086 | -0.0031 [-0.0123, +0.0066] | -0.0005 [-0.0039, +0.0033] | -0.0004 [-0.0037, +0.0027] | 66.7% → 66.8% | +0.1 pp [-1.0, +1.1] |
| logreg-ext + A[fund_market] | all | 2087 (56) | 0.6235 → 0.6222 | -0.0013 [-0.0048, +0.0023] | -0.0002 [-0.0016, +0.0012] | +0.0001 [-0.0011, +0.0014] | 65.8% → 65.7% | -0.0 pp [-0.5, +0.4] |

De-vig of the last pre-cutoff Pinnacle price as a forecast on its own (301 scored Matches, mean overround 1.050):

| Method | Log-loss | Brier | Δ log-loss vs proportional [90%] |
|---|---:|---:|---|
| proportional | 0.6023 | 0.2069 | +0.00000 [+0.00000, +0.00000] |
| power | 0.6036 | 0.2068 | +0.00124 [-0.00403, +0.00895] |
| shin | 0.6024 | 0.2067 | +0.00003 [-0.00267, +0.00401] |

CLV-style metric (767 scored Matches with an early and a closing price; early = about 12 h before the cutoff, close = last pre-cutoff price, chain order of sources). CLV = sign(model − early) × (close − early) in logit units; the model is stage 1 (no market input).

| Stage 1 of | Mean CLV (logit) [90%] | Share of moved lines that moved toward the model [90%] | Lines moved | Log-loss close − early [90%] |
|---|---|---|---:|---|
| lgbm-ext | +0.0155 [+0.0056, +0.0262] | +53.1% [+49.4, +56.6] | 670 | +0.0008 [-0.0026, +0.0040] |
| logreg-ext | +0.0142 [+0.0049, +0.0246] | +51.5% [+48.7, +54.8] | 670 | +0.0008 [-0.0026, +0.0040] |

## B: Bo3 veto, empirical Bayes

| Configuration | Subset | Matches (Events) | Log-loss base → variant | Δ log-loss [90%] | Δ Brier [90%] | Δ resolution [90%] | Accuracy base → variant | Δ accuracy [90%] |
|---|---|---:|---|---|---|---|---|---|
| lgbm-ext + veto_eb | all | 2087 (56) | 0.6198 → 0.6204 | +0.0006 [-0.0010, +0.0022] | +0.0000 [-0.0005, +0.0006] | +0.0011 [-0.0001, +0.0023] | 65.3% → 65.8% | +0.6 pp [+0.1, +1.1] |
| lgbm-ext + veto_eb | market | 777 (21) | 0.6153 → 0.6155 | +0.0003 [-0.0027, +0.0033] | -0.0004 [-0.0013, +0.0005] | +0.0009 [-0.0018, +0.0044] | 65.8% → 66.7% | +0.9 pp [+0.0, +1.9] |
| lgbm-ext + veto_eb | bo3 | 1774 (56) | 0.6166 → 0.6171 | +0.0005 [-0.0012, +0.0023] | +0.0001 [-0.0006, +0.0007] | +0.0013 [+0.0001, +0.0028] | 66.1% → 66.7% | +0.6 pp [+0.1, +1.1] |
| lgbm-ext + veto_eb_noreset | all | 2087 (56) | 0.6198 → 0.6202 | +0.0004 [-0.0014, +0.0024] | +0.0002 [-0.0004, +0.0009] | +0.0005 [-0.0008, +0.0018] | 65.3% → 65.2% | -0.1 pp [-0.6, +0.3] |
| lgbm-ext + veto_eb_noshrink | all | 2087 (56) | 0.6198 → 0.6206 | +0.0008 [-0.0010, +0.0026] | +0.0004 [-0.0002, +0.0012] | +0.0005 [-0.0008, +0.0018] | 65.3% → 65.4% | +0.1 pp [-0.2, +0.5] |
| logreg-ext + veto_eb | all | 2087 (56) | 0.6235 → 0.6242 | +0.0007 [+0.0001, +0.0013] | +0.0002 [-0.0000, +0.0004] | +0.0002 [-0.0004, +0.0009] | 65.8% → 65.8% | +0.0 pp [-0.1, +0.2] |
| logreg-ext + veto_eb | market | 777 (21) | 0.6117 → 0.6119 | +0.0002 [-0.0002, +0.0006] | +0.0001 [-0.0000, +0.0003] | +0.0002 [-0.0009, +0.0015] | 66.7% → 66.7% | +0.0 pp [+0.0, +0.0] |
| logreg-ext + veto_eb | bo3 | 1774 (56) | 0.6210 → 0.6211 | +0.0000 [-0.0005, +0.0004] | -0.0000 [-0.0002, +0.0002] | +0.0005 [-0.0001, +0.0011] | 66.3% → 66.3% | +0.1 pp [-0.2, +0.3] |
| logreg-ext + veto_eb_noreset | all | 2087 (56) | 0.6235 → 0.6240 | +0.0004 [-0.0003, +0.0012] | +0.0001 [-0.0002, +0.0004] | +0.0003 [-0.0003, +0.0009] | 65.8% → 66.1% | +0.3 pp [+0.0, +0.7] |
| logreg-ext + veto_eb_noshrink | all | 2087 (56) | 0.6235 → 0.6251 | +0.0016 [+0.0005, +0.0026] | +0.0006 [+0.0001, +0.0010] | -0.0005 [-0.0014, +0.0005] | 65.8% → 65.7% | -0.0 pp [-0.5, +0.5] |

The simulation alone against the anchor on the 1774 scored Bo3s with a simulated veto (the lab's `bo3_veto_sim`, which blends its simulation with the market, picked −2.8 pp [−4.5, −1.4] on 1,771 Bo3s):

| Forecast vs anchor | Subset | Matches (Events) | Log-loss base → variant | Δ log-loss [90%] | Δ Brier [90%] | Δ resolution [90%] | Accuracy base → variant | Δ accuracy [90%] |
|---|---|---:|---|---|---|---|---|---|
| simulation | bo3 | 1774 (56) | 0.7979 → 0.6358 | -0.1622 [-0.2376, -0.0878] | -0.0146 [-0.0240, -0.0055] | -0.0054 [-0.0101, -0.0011] | 66.5% → 64.3% | -2.1 pp [-3.6, -0.7] |
| market blend | bo3 | 1774 (56) | 0.7979 → 0.6229 | -0.1750 [-0.2488, -0.1045] | -0.0194 [-0.0280, -0.0112] | -0.0020 [-0.0065, +0.0020] | 66.5% → 64.8% | -1.7 pp [-3.0, -0.5] |
| simulation veto_eb_noreset | bo3 | 1774 (56) | 0.7979 → 0.6357 | -0.1622 [-0.2376, -0.0878] | -0.0146 [-0.0240, -0.0055] | -0.0054 [-0.0100, -0.0010] | 66.5% → 64.4% | -2.1 pp [-3.5, -0.6] |
| simulation veto_eb_noshrink | bo3 | 1774 (56) | 0.7979 → 0.6664 | -0.1316 [-0.2064, -0.0574] | -0.0025 [-0.0124, +0.0073] | -0.0119 [-0.0172, -0.0071] | 66.5% → 62.5% | -4.0 pp [-5.4, -2.6] |

Active map-pool changes the point-in-time pool detector found in our own vetoes (the seven maps most frequent in the last 14 days of released vetoes; date = first cutoff that sees the change):

| Seen from | Map in | Map out |
|---|---|---|
| 2024-06-05 | Dust2 | Overpass |
| 2025-01-16 | Train | Vertigo |
| 2025-07-28 | Overpass | Anubis |
| 2026-01-20 | Anubis | Train |
| 2026-07-09 | Cache | Overpass |

## C: OpenSkill player ratings

| Configuration | Subset | Matches (Events) | Log-loss base → variant | Δ log-loss [90%] | Δ Brier [90%] | Δ resolution [90%] | Accuracy base → variant | Δ accuracy [90%] |
|---|---|---:|---|---|---|---|---|---|
| lgbm-ext + openskill | all | 2087 (56) | 0.6198 → 0.6179 | -0.0019 [-0.0036, -0.0001] | -0.0007 [-0.0013, +0.0000] | +0.0015 [+0.0001, +0.0028] | 65.3% → 65.7% | +0.4 pp [-0.3, +1.2] |
| lgbm-ext + openskill | market | 777 (21) | 0.6153 → 0.6128 | -0.0025 [-0.0065, +0.0013] | -0.0007 [-0.0021, +0.0007] | -0.0000 [-0.0030, +0.0036] | 65.8% → 66.7% | +0.9 pp [-0.3, +2.3] |
| lgbm-ext + openskill − roster | all | 2087 (56) | 0.6198 → 0.6193 | -0.0006 [-0.0026, +0.0014] | -0.0003 [-0.0011, +0.0005] | +0.0014 [-0.0002, +0.0029] | 65.3% → 65.4% | +0.1 pp [-0.5, +0.7] |
| lgbm-ext + openskill − roster | market | 777 (21) | 0.6153 → 0.6142 | -0.0010 [-0.0055, +0.0034] | -0.0006 [-0.0022, +0.0010] | -0.0009 [-0.0047, +0.0026] | 65.8% → 66.7% | +0.9 pp [+0.3, +1.6] |
| logreg-ext + openskill | all | 2087 (56) | 0.6235 → 0.6237 | +0.0002 [-0.0029, +0.0030] | -0.0002 [-0.0013, +0.0008] | +0.0005 [-0.0009, +0.0021] | 65.8% → 65.8% | +0.0 pp [-0.6, +0.6] |
| logreg-ext + openskill | market | 777 (21) | 0.6117 → 0.6102 | -0.0015 [-0.0049, +0.0020] | -0.0006 [-0.0018, +0.0006] | +0.0031 [+0.0012, +0.0060] | 66.7% → 67.3% | +0.6 pp [-0.4, +1.5] |

## D: travel

| Configuration | Subset | Matches (Events) | Log-loss base → variant | Δ log-loss [90%] | Δ Brier [90%] | Δ resolution [90%] | Accuracy base → variant | Δ accuracy [90%] |
|---|---|---:|---|---|---|---|---|---|
| lgbm-ext + travel | all | 2087 (56) | 0.6198 → 0.6208 | +0.0010 [-0.0006, +0.0027] | +0.0006 [-0.0000, +0.0012] | +0.0001 [-0.0012, +0.0015] | 65.3% → 65.5% | +0.2 pp [-0.2, +0.7] |
| lgbm-ext + travel | market | 777 (21) | 0.6153 → 0.6142 | -0.0010 [-0.0048, +0.0027] | -0.0001 [-0.0014, +0.0011] | -0.0003 [-0.0027, +0.0024] | 65.8% → 66.0% | +0.3 pp [-0.4, +1.0] |
| logreg-ext + travel | all | 2087 (56) | 0.6235 → 0.6266 | +0.0031 [+0.0006, +0.0057] | +0.0010 [-0.0001, +0.0021] | -0.0015 [-0.0026, -0.0001] | 65.8% → 65.2% | -0.6 pp [-1.2, +0.0] |
| logreg-ext + travel | market | 777 (21) | 0.6117 → 0.6136 | +0.0019 [+0.0010, +0.0028] | +0.0007 [+0.0003, +0.0011] | -0.0001 [-0.0020, +0.0019] | 66.7% → 67.2% | +0.5 pp [+0.1, +1.0] |

## E: multiple-testing control (Hansen SPA, Event block bootstrap)

Configurations: phase 2 40, phase 3 45 (which include phase 4's two forecasters), phase 5 27. 2000 Event resamples, seed 33; survivors = stepwise SPA at α = 0.10.

| Family | Loss | K | Best (mean loss gain) | p (consistent) | p (lower) | p (upper, RC) | Survivors |
|---|---|---:|---|---:|---:|---:|---|
| all_vs_anchor | log_loss | 112 | `logreg-ext + A[fund_market]` (+0.1721) | 0.000 | 0.000 | 0.000 | `p2 logreg full`, `p2 logreg baseline`, `p2 logreg baseline+rank`, `p2 logreg baseline+opp_form`, `p2 logreg baseline+past_form`, `p2 logreg baseline+hltv_h2h`, `p2 logreg baseline+players`, `p2 logreg baseline+map_pool` (+104 more) (112) |
| addons_vs_base | log_loss | 25 | `lgbm-ext + openskill` (+0.0019) | 0.296 | 0.174 | 0.315 | none (0) |
| A_vs_base_market | log_loss | 14 | `lgbm-ext + A[fund_market]` (+0.0058) | 0.363 | 0.220 | 0.363 | none (0) |
| all_vs_anchor | accuracy | 112 | `p3 logreg p3_all / +S/A+B+C` (+0.0062) | 0.816 | 0.622 | 0.841 | none (0) |
| addons_vs_base | accuracy | 25 | `lgbm-ext + veto_eb` (+0.0057) | 0.240 | 0.213 | 0.240 | none (0) |
| A_vs_base_market | accuracy | 14 | `lgbm-ext + A[core]` (+0.0116) | 0.112 | 0.112 | 0.112 | none (0) |

Coverage on the 2,087 scored Matches: OpenSkill lineups 100%, travel (both teams located) 100%, simulated Bo3 vetoes 1774 Matches.
