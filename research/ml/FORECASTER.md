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

## Conclusion (written after the run)

1. **Better calibrated for Pick'em: yes, by the declared rule.** The Brier score of the "≥5 correct"
   event is 0.282 for `lgbm-ext` B against 0.465 for anchor B: −0.183 [−0.329, −0.037], wholly below
   zero. The anchor's slips claim P(≥5) = 85% on average and reach five on 11 of 28 Stages (39%):
   −46 pp [−62, −30], badly over-confident, as Singapore Stage 2's 59% suggested. `lgbm-ext` claims
   39% and reaches five on 14 of 28 (50%): +11 pp [−5, +27], slightly under-confident and within
   noise. Expected minus realised correct picks: anchor +1.79, `lgbm-ext` −0.40.
2. **Reaches ≥5 more often than anchor B: not measurably.** 14 against 11 Stages: +3 [−2, +8]. The
   best point estimate is `logreg-ext` B, 16/28, +5 [+0, +10] (exploratory; its lower bound touches
   zero).
3. **The gain is the calibration fix, not the extended archive.** The in-scope models of PICKEM.md
   (logreg/lgbm, baseline/all) are about as well calibrated at the Stage level (Brier 0.24 to 0.28)
   and score about as often. What differs from the anchor is that any fitted model undoes the
   overconfident Valve rating.
4. **Per Match** (2087 scored Matches, walk-forward): the anchor is over-confident by +12.4 pp
   [+9.7, +15.0] on average (its 95-100% bin wins 76%); `lgbm-ext` by +0.6 pp [−1.4, +2.9] (ECE
   1.3 pp), `logreg-ext` +2.4 pp [+0.7, +4.3] (ECE 2.5 pp). `lgbm-ext` is the best-calibrated
   forecaster measured.

For KallBet: use `lgbm-ext` (or `logreg-ext`) probabilities wherever a probability is shown or a
decision depends on it (the Rationale's win percentage, the Pick'em P(≥5) and expected points);
picks of individual Matches stay as good as the anchor's (phase 3), no better.

## Catch-up in Leagues (exploratory)

`kbml/catchup.py` ports `sidecar/src/adapters/lab/leagues.rs`; with anchor bots on the snapshot it
reproduces `docs/lab/research/catch-up.md` exactly (all 24 numbers, `tests/test_catchup.py`). Below,
the same draws with the bot's probability taken from `lgbm-ext` (walk-forward, 2087 Matches; the
catch-up rule reverses a 50-60% favourite while trailing), opponents still following the anchor
(or the market).

| Opponents | Members | Anchor plain | Anchor catch-up | lgbm-ext plain | lgbm-ext catch-up | Δ first, lgbm-ext − anchor (plain) | Δ first, lgbm-ext − anchor (catch-up) |
|---|---:|---|---|---|---|---|---|
| favourite follower, 10% noise | 4 | 0.603 (place 1.49) | 0.684 (place 1.51) | 0.568 (place 1.69) | 0.571 (place 1.78) | −0.035 [−0.090, +0.020] | −0.113 [−0.168, −0.058] |
| favourite follower, 10% noise | 12 | 0.190 (place 2.84) | 0.398 (place 3.57) | 0.290 (place 3.70) | 0.296 (place 4.73) | +0.100 [+0.045, +0.155] | −0.101 [−0.156, −0.046] |
| favourite follower, 25% noise | 4 | 0.718 (place 1.34) | 0.738 (place 1.36) | 0.699 (place 1.45) | 0.688 (place 1.48) | −0.019 [−0.074, +0.036] | −0.050 [−0.105, +0.005] |
| favourite follower, 25% noise | 12 | 0.396 (place 2.26) | 0.443 (place 2.61) | 0.418 (place 2.78) | 0.383 (place 3.30) | +0.022 [−0.032, +0.077] | −0.060 [−0.115, −0.005] |
| market follower | 4 | 0.961 (place 1.12) | 0.979 (place 1.06) | 0.875 (place 1.38) | 0.915 (place 1.25) | −0.086 [−0.141, −0.032] | −0.064 [−0.118, −0.009] |
| market follower | 12 | 0.961 (place 1.43) | 0.979 (place 1.23) | 0.875 (place 2.38) | 0.915 (place 1.93) | −0.086 [−0.141, −0.032] | −0.064 [−0.118, −0.009] |

57 in-scope Events, 2000 paired trials, seed 33; brackets are 90% Hoeffding intervals (Monte Carlo
only, conditional on these Events and opponents). Reading: against opponents who copy the anchor, a
bot that picks differently wins outright more often in big Leagues (+0.10 with 12 members, plain) and
finishes worse on average. The fixed 40-60% catch-up window is worse with calibrated
probabilities (−0.05 to −0.11): `lgbm-ext` puts about twice as many Matches in that window (687 vs
363 in 50-60%), so it flips many more picks, and some of those flipped favourites really were
favourites. The window was tuned for the over-confident anchor; with `lgbm-ext` it would need to be
re-chosen (walk-forward) before use.

## What a Rust port for live KallBet would need (outline, not implemented)

- **Model export.** LightGBM: `booster.save_model()` text (or `dump_model()` JSON) per training
  date; Rust evaluates the trees itself (a tree walker over the dump is ~150 lines, no C library), and
  keeps the two-orientation average p = (f(x) + 1 − f(swap x)) / 2. Logreg: coefficients, scaler means
  and scales, and the feature order as JSON, like `report.json`'s `deployment`. A parity test: Rust
  predictions equal the Python ones on a saved fixture of feature rows.
- **Features available live today:** the anchor inputs (market line and Valve rating, the existing
  ports), format, tier (PandaScore). **Needing new live inputs:** results history for Elo, form,
  head-to-head and rest (PandaScore finished Matches); HLTV match records (maps, vetoes, player stats,
  half scores) for map Elo, map pool, players and sides; the weekly HLTV ranking page (the ml-data
  branch has a Rust collector, `hltv_research::ranking`); HLTV rank badges. `logreg-ext` on `p3_rw`
  needs fewer of them (no player or side stats) than `lgbm-ext` on `p3_all`.
- **Two ways to supply team state:** port the feature groups to Rust (exact, most work), or have an
  offline Python job export per-team state (ratings, form, map shares) with the model weekly, which
  is simpler but stale between exports. Either way every datum keeps the point-in-time rule
  (`AsOf` strictly before the cutoff), and a hypothetical pairing (Pick'em) uses the missing-data path
  for page features, as here.
- **Retraining:** weekly or after each Event, walk-forward, with the same inner-validation rule; the
  promotion gate stays the Signal Lab's.

<!-- generated by kbml.forecaster_report: everything below is rewritten -->

## Pick'em: calibration of the slip over the 28 Stages

| Forecaster | Strategy | Mean correct | Stages ≥5 | Mean predicted P(≥5) | Realised − predicted [90%] | Brier of ≥5 | Δ Brier vs anchor B [90%] | Δ stages ≥5 vs anchor B [90%] | Expected − realised correct |
|---|---|---:|---:|---:|---|---:|---|---|---:|
| Anchor | A | 4.25 | 13/28 | 84.7% | -38.2 pp [-55.1, -21.6] | 0.423 | -0.042 [-0.104, -0.006] | +2 stages [+0.0, +4.0] | +1.87 |
| Anchor | B | 4.25 | 11/28 | 85.4% | -46.1 pp [-61.5, -30.4] | 0.465 | — | — | +1.79 |
| lgbm ext | A | 4.46 | 14/28 | 38.6% | +11.4 pp [-4.5, +27.2] | 0.273 | -0.192 [-0.336, -0.048] | +3 stages [-2.0, +8.0] | -0.37 |
| lgbm ext | B | 4.50 | 14/28 | 38.7% | +11.3 pp [-5.2, +27.0] | 0.282 | -0.183 [-0.329, -0.037] | +3 stages [-2.0, +8.0] | -0.40 |
| logreg ext | A | 4.61 | 15/28 | 50.0% | +3.5 pp [-13.6, +19.8] | 0.280 | -0.184 [-0.318, -0.051] | +4 stages [+0.0, +8.0] | -0.09 |
| logreg ext | B | 4.57 | 16/28 | 50.1% | +7.0 pp [-10.1, +23.0] | 0.281 | -0.183 [-0.317, -0.050] | +5 stages [+0.0, +10.0] | -0.05 |
| lgbm all | A | 4.39 | 13/28 | 36.2% | +10.3 pp [-5.7, +26.4] | 0.266 | -0.199 [-0.354, -0.041] | +2 stages [-2.0, +6.0] | -0.39 |
| lgbm all | B | 4.57 | 14/28 | 36.3% | +13.7 pp [-2.2, +30.0] | 0.274 | -0.191 [-0.344, -0.036] | +3 stages [-1.0, +7.0] | -0.58 |
| logreg all | A | 4.32 | 12/28 | 29.8% | +13.1 pp [-2.0, +27.7] | 0.252 | -0.213 [-0.360, -0.064] | +1 stages [-4.0, +6.0] | -0.57 |
| logreg all | B | 4.39 | 12/28 | 29.9% | +12.9 pp [-2.3, +28.3] | 0.262 | -0.202 [-0.370, -0.034] | +1 stages [-3.0, +5.0] | -0.64 |
| lgbm baseline | A | 4.39 | 14/28 | 44.2% | +5.8 pp [-9.1, +21.3] | 0.244 | -0.220 [-0.355, -0.080] | +3 stages [+0.0, +7.0] | -0.09 |
| lgbm baseline | B | 4.36 | 15/28 | 44.4% | +9.2 pp [-6.1, +25.3] | 0.265 | -0.199 [-0.340, -0.057] | +4 stages [+0.0, +8.0] | -0.06 |
| logreg baseline | A | 4.46 | 14/28 | 42.9% | +7.1 pp [-8.2, +23.3] | 0.276 | -0.189 [-0.339, -0.044] | +3 stages [-1.0, +7.0] | -0.22 |
| logreg baseline | B | 4.43 | 14/28 | 43.1% | +6.9 pp [-8.3, +23.1] | 0.275 | -0.189 [-0.339, -0.044] | +3 stages [-1.0, +7.0] | -0.19 |

Intervals: paired Stage bootstrap (2000 resamples, 90%, seed 33) over the 28 Stages. "Realised − predicted": share of Stages with ≥5 correct minus the mean simulated P(≥5) (negative = over-confident). Brier of ≥5: mean of (simulated P(≥5) − [≥5 happened])².

### Per Stage (strategy B)

| Stage | Anchor B: correct (P≥5) | lgbm ext B: correct (P≥5) | logreg ext B: correct (P≥5) |
|---|---|---|---|
| Perfect World Major Shanghai Major 2024 / Opening Stage | 3 (87%) | 5 (25%) | 5 (35%) |
| Perfect World Major Shanghai Major 2024 / Elimination Stage | 6 (81%) | 6 (28%) | 5 (37%) |
| PGL Cluj-Napoca 2025 / Group Stage | 6 (94%) | 5 (35%) | 7 (49%) |
| ESL Pro League Season 21 2025 / Group Stage | 9 (97%) | 9 (39%) | 9 (64%) |
| PGL Bucharest 2025 / Group Stage | 2 (97%) | 5 (36%) | 5 (66%) |
| PGL Astana 2025 / Group Stage | 3 (98%) | 4 (41%) | 4 (73%) |
| BLAST.tv Major Austin 2025 / Stage 1 | 6 (77%) | 4 (32%) | 5 (43%) |
| BLAST.tv Major Austin 2025 / Stage 2 | 2 (92%) | 2 (29%) | 2 (38%) |
| BLAST.tv Major Austin 2025 / Stage 3 | 4 (95%) | 4 (45%) | 6 (58%) |
| FISSURE PLAYGROUND Season 2 2025 / Group Stage | 3 (93%) | 4 (29%) | 2 (35%) |
| ESL Pro League Season 22 2025 / Stage 1 | 4 (98%) | 4 (54%) | 4 (74%) |
| ESL Pro League Season 22 2025 / Stage 2 | 4 (96%) | 4 (46%) | 3 (62%) |
| PGL Masters Bucharest 2025 / Group Stage | 2 (83%) | 3 (32%) | 3 (37%) |
| StarLadder Major Budapest 2025 / Stage 1 | 6 (88%) | 5 (34%) | 4 (49%) |
| StarLadder Major Budapest 2025 / Stage 2 | 5 (83%) | 5 (25%) | 5 (27%) |
| StarLadder Major Budapest 2025 / Stage 3 | 4 (84%) | 6 (39%) | 5 (41%) |
| PGL Cluj-Napoca 2026 / Group Stage | 6 (75%) | 6 (38%) | 6 (50%) |
| ESL Pro League Season 23 2026 / Stage 1 | 5 (92%) | 4 (54%) | 4 (59%) |
| ESL Pro League Season 23 2026 / Stage 2 | 5 (86%) | 4 (39%) | 5 (43%) |
| PGL Bucharest 2026 / Group Stage | 2 (92%) | 2 (41%) | 2 (52%) |
| PGL Astana 2026 / Group Stage | 6 (93%) | 6 (59%) | 6 (69%) |
| IEM Cologne Major 2026 / Stage 1 | 4 (67%) | 5 (42%) | 5 (49%) |
| IEM Cologne Major 2026 / Stage 2 | 5 (77%) | 5 (36%) | 5 (40%) |
| IEM Cologne Major 2026 / Stage 3 | 4 (75%) | 2 (32%) | 3 (38%) |
| XSE Pro League 2026 / Group Stage | 2 (52%) | 6 (36%) | 6 (40%) |
| ESL Pro League Season 24 2026 / Group Stage | 4 (96%) | 5 (54%) | 5 (71%) |
| PGL Masters Bucharest 2026 / Group Stage | 4 (92%) | 3 (49%) | 3 (59%) |
| PGL Major Singapore 2026 / Stage 1 | 3 (50%) | 3 (34%) | 4 (44%) |

## Per-Match reliability on the 2087 scored Matches

Walk-forward by Event as in phase 3 (ext rows train). Bins by the favourite's probability; "won" = share of those Matches the favourite won.

| Bin (favourite's P) | Anchor: n, predicted → won | lgbm ext: n, predicted → won | logreg ext: n, predicted → won |
|---|---|---|---|
| 50–55% | 178, 52% → 52% | 331, 53% → 49% | 305, 53% → 51% |
| 55–60% | 185, 57% → 50% | 356, 57% → 56% | 322, 57% → 56% |
| 60–65% | 153, 62% → 59% | 347, 63% → 64% | 316, 62% → 63% |
| 65–70% | 188, 68% → 66% | 363, 67% → 67% | 298, 68% → 65% |
| 70–75% | 177, 73% → 67% | 293, 72% → 72% | 252, 72% → 68% |
| 75–80% | 179, 78% → 64% | 185, 77% → 78% | 190, 77% → 74% |
| 80–85% | 204, 82% → 69% | 131, 82% → 82% | 160, 83% → 80% |
| 85–90% | 207, 87% → 65% | 78, 87% → 85% | 144, 87% → 83% |
| 90–95% | 202, 93% → 74% | 3, 91% → 100% | 78, 92% → 81% |
| 95–100% | 414, 98% → 76% | 0 | 22, 97% → 86% |

| Forecaster | Log-loss | Brier | ECE | Over-confidence: mean(P favourite − favourite won) [90%] |
|---|---:|---:|---:|---|
| Anchor | 0.7943 | 0.2398 | 12.4 pp | +12.4 pp [+9.7, +15.0] |
| lgbm ext | 0.6198 | 0.2152 | 1.3 pp | +0.6 pp [-1.4, +2.9] |
| logreg ext | 0.6235 | 0.2158 | 2.5 pp | +2.4 pp [+0.7, +4.3] |
