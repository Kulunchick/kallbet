# Catch-up in Leagues: a Bellman DP policy instead of the fixed 40-60% window

FORECASTER.md found that with `lgbm-ext` probabilities the plain bot wins 12-member Leagues more
often than the anchor bot (+0.10), while the static catch-up rule of `docs/lab/research/catch-up.md`
(reverse a 50-60% favourite whenever trailing) gets worse (−0.05 to −0.11): calibrated probabilities
put about twice as many Matches in the window. This page tests a policy that decides each flip by
backward induction on the chance of finishing first.

## What a live bot can know at decision time (from the KallBet code)

- **Standings.** `GET /api/leagues/{id}/leaderboard` (`src/api/leaderboard.rs`) ranks the current
  Members by Match Points over decided Matches of the League's effective view. Points come from
  `scoring::judge`: one Point per correct Prediction times the Stage's points (`league_stage_points`,
  default 1); Missing, Void and Pending earn nothing. Equal Points share a place, including first
  (`scoring::leaderboard`: 1, 1, 3). So a tie for most Points is a first place.
- **Others' Predictions.** A Bot User sees no revealed Prediction of a Match before its Hard Lock
  (`api::matches`, `blind = viewer.is_bot && !outcome.is_hard_locked()`; ADR-0003 amendment), and it
  may change its own until Hard Lock. So when the bot decides a Match it does not know how anyone
  picked *that* Match. It can see others' picks of earlier, Hard-Locked Matches (revealed), but the
  sidecar does not read them today, and the lab simulation (`leagues.rs`) gives the bot only the
  published standings. The policy here uses only the standings.
- **Matches left.** The League's Match list (`api::matches`) shows the Event's known Matches; later
  Swiss rounds and Playoff slots appear once teams are known, so the exact count is not listed early
  but follows from the format (a 16-team Swiss Stage has 33 Matches, an 8-team bracket 7). The replay
  gives the bot the true number of the Event's remaining Matches (the format is known); results not
  yet published count as remaining.
- **Members.** The League's member list (`leagues::members`), so the League size is known.
- **The live sidecar** (`sidecar/src/scheduler.rs`, `ports::decision_policy`) has no catch-up rule
  and does not read the leaderboard today: catch-up exists only in the lab (`adapters/lab/leagues.rs`).

## The DP policy (`kbml/catchup_dp.py`)

State before a Match: r = Matches whose result is not yet published (this one included), d = the
bot's published Points minus the leader's (clipped to ±10), c0 = opponents tied for the lead, c1 =
opponents one Point behind the lead (the rest are treated as two behind). Value V_r = P(finish with
at least the leader's Points) = P(first, ties shared). Opponent model as the scenario's opponents:
each opponent independently takes the consensus side (the anchor favourite, or the market favourite)
and, for the favourite follower, reverses it with the noise probability; no consensus (missing
input) means every opponent misses. Our probability p decides the outcome. Future Matches are drawn
from the empirical distribution of P(consensus side wins) under our p over the scored Matches of
Events that ended before this Event began (walk-forward; a uniform 50-90% grid while fewer than 100
exist). V_0 = [d ≥ 0]; V_r is the expectation over the next Match type of the better of the two
picks (backward recursion, tables per League size, opponent model and Event). At a real Match the
bot computes both picks' values from its own p for that Match and V_{r−1}, and takes the underdog
only if that is strictly higher (ties: its favourite).

## Primary configuration (declared before its evaluation)

Committed before the DP or the re-tuned window were run on the archive. FORECASTER.md's catch-up
table (plain and static 40-60 bots, anchor and `lgbm-ext`) had been seen.

- **Policy:** the DP above, built with the true opponent model of the scenario (favourite follower,
  10% noise) and the League's size.
- **Probabilities:** `lgbm-ext` (walk-forward, phase 3 protocol, the 2087 scored Matches).
- **Comparison:** against the plain `lgbm-ext` bot (always its favourite), 12 members, opponents
  favourite followers with 10% noise.
- **Data and trials:** the 57 in-scope Events, 2000 paired trials, seed 33, the draws of
  `kbml/catchup.py` (both bots face identical opponents).
- **Rule:** DP "wins more 12-member Leagues than plain" only if the 90% interval of the paired
  difference in the first-place indicator (DP − plain), Hoeffding over the 2000 trials as in
  catch-up.md, lies wholly above 0.

Everything else (anchor probabilities, 4 members, 25% noise, market followers, the static and
re-tuned windows, expected place, Event-clustered bootstrap intervals, a DP built with the wrong
noise) is exploratory.

## Conclusion (written after the run)

**Primary: yes, by the declared rule.** With `lgbm-ext` probabilities, 12 members and favourite
followers with 10% noise, the DP bot finishes first in 0.474 of trials against 0.290 for the plain
bot: **+0.184 [+0.129, +0.239]** (Hoeffding), wholly above 0. The Event-clustered bootstrap agrees
(+0.184 [+0.122, +0.244]). The plain and 40-60 rows reproduce FORECASTER.md's table exactly (same
draws).

Comparison table: P(1st) and Δ against plain with the same probabilities, 90% Hoeffding.

| Opponents | Members | lgbm-ext plain | lgbm-ext 40-60 | lgbm-ext re-tuned | **lgbm-ext DP** | anchor 40-60 (catch-up.md) | anchor DP |
|---|---:|---|---|---|---|---|---|
| fav. 10% | 4 | 0.568 | +0.004 [−0.051, +0.058] | +0.008 [−0.047, +0.063] | **+0.135 [+0.080, +0.190]** (0.703) | 0.684 | 0.686 |
| fav. 10% | 12 | 0.290 | +0.006 [−0.048, +0.061] | +0.025 [−0.030, +0.080] | **+0.184 [+0.129, +0.239]** (0.474) | 0.398 | 0.381 |
| fav. 25% | 4 | 0.699 | −0.011 [−0.066, +0.043] | −0.005 [−0.060, +0.050] | +0.034 [−0.021, +0.089] (0.733) | 0.738 | 0.722 |
| fav. 25% | 12 | 0.418 | −0.035 [−0.090, +0.019] | −0.019 [−0.074, +0.036] | +0.049 [−0.006, +0.103] (0.467) | 0.443 | 0.429 |
| market | 4 | 0.875 | +0.041 [−0.014, +0.096] | +0.048 [−0.007, +0.103] | **+0.105 [+0.050, +0.160]** (0.980) | 0.979 | 0.941 |
| market | 12 | 0.875 | +0.041 [−0.014, +0.096] | +0.048 [−0.007, +0.103] | **+0.105 [+0.050, +0.160]** (0.980) | 0.979 | 0.941 |

Reading (all exploratory beyond the primary):

1. **The DP fixes what the static window broke.** With calibrated probabilities the 40-60 rule
   gains nothing (−0.035 to +0.041), and re-tuning the window walk-forward does not rescue it
   (−0.019 to +0.048): one window cannot tell "one behind with 40 left" from "one behind with 3
   left". The DP gains in every scenario: +0.03 to +0.18, measurably in four of six. With
   `lgbm-ext` it is the best bot measured in every row, the anchor bots included (12 members, 10%:
   0.474 against 0.398 for anchor 40-60).
2. **It needs honest probabilities.** The DP with the over-confident anchor is no better than the
   40-60 rule (and −0.021 against market followers): it trusts 90% favourites that win 74%.
   Calibration and the DP go together.
3. **It pays in mean place, as it should.** It maximises P(1st), not place: in 12-member Leagues
   mean place gets worse by +0.43 (10%) and +0.47 (25%), intervals touching 0. Against market
   followers both improve (mean place −1.16 [−1.76, −0.55] with 12 members).
4. **Robust to the wrong noise level.** A DP built for 25% noise playing 10% followers still gains
   +0.147 [+0.092, +0.202]; built for 10% playing 25%, +0.060 [+0.006, +0.115]. The bot does not
   need to know the field's noise well. Against the market follower the DP was built with the right
   model only (not tested with a wrong one).
5. **Few flips.** The `lgbm-ext` DP changes 0.8 to 5.5 picks per Event on average, the anchor DP 0.3
   to 3.7.

Limits: the opponents are simulated (copy the anchor or the market), not real friends; Hoeffding
brackets are Monte Carlo only, conditional on these 57 Events; the DP's field summary (lead group,
one behind, the rest two behind) is an approximation (exact against brute force for two opponents,
`tests/test_catchup_dp.py`); the bot is told the true number of the Event's unpublished results.

## Rust change (outline, not implemented)

**Sidecar lab (`sidecar/src/adapters/lab/leagues.rs`).** Add a `CatchUp::Dp` variant next to the
40-60 rule: `dp.rs` with `transitions(n_opp, eps)`, `build(...) -> Tables` (backward recursion over
r, d ∈ ±10, (c0, c1)) and `Tables::values(r, d, c0, c1, q)`, ported from `kbml/catchup_dp.py`. The
replay passes each policy the published standings, the opponent counts at and one below the lead,
and r = Matches whose result is not yet published. Match types: the histogram of P(consensus wins)
over scored Matches of Events that ended before the drawn Event began. Parity test: the Rust tables
and the replay's places equal the Python ones on a fixture (as `tests/test_catchup.py` does for the
40-60 rule); the report gains the DP rows and the verdict table by situation.

**Live bot.** (1) `LeaguePlatform::leaderboard(token, league)` reading
`GET /api/leagues/{id}/leaderboard` (points per Member) and the member count. (2) r from the
League's Match list plus the format's remaining Matches (Swiss 33 per 16-team Stage, bracket 7),
counting decided-but-unpublished as remaining. (3) A `DecisionPolicy` wrapper: after the base policy
gives p (from `lgbm-ext`, see FORECASTER.md: the anchor's p makes the DP useless), compute q
against the consensus side (the anchor or market favourite, which is what other Members most
likely pick) and flip iff `values` says so; the Rationale's "against the market favourite" flag
and stated win probability stay the honest p, so a flipped pick shows its true chance. Tables are
small (≈ 120 × 21 × 66 floats per size) and can be built at start-up per League size. (4) Opponent
model default: favourite follower, 10% noise (robust above). Stage weights other than 1 need the
DP over weighted points (d in Points, steps of the Stage's weight); not covered here.

<!-- generated by kbml.catchup_report: everything below is rewritten -->

57 in-scope Events, 2000 paired trials, seed 33 (the draws of `kbml.catchup`). P(1st) counts a shared first place. Brackets: 90% Hoeffding over the trials (Monte Carlo only, conditional on these Events and opponents, as catch-up.md); Δ columns also give an Event-clustered bootstrap (trials grouped by drawn Event, 2000 resamples, seed 33). Δ is against the plain bot with the same probabilities.

### favourite follower, 10% noise, 4 members

| Probabilities | Policy | P(1st) | Mean place | Δ P(1st) vs plain (Hoeffding) | Δ P(1st) (Event bootstrap) | Δ mean place (Hoeffding) |
|---|---|---|---|---|---|---|
| anchor | plain | 0.603 [0.575, 0.630] | 1.49 [1.40, 1.57] | — | — | — |
| anchor | static 40-60 | 0.684 [0.657, 0.711] | 1.51 [1.43, 1.60] | +0.082 [+0.027, +0.136] | +0.082 [+0.042, +0.123] | +0.03 [-0.14, +0.19] |
| anchor | re-tuned window | 0.674 [0.647, 0.701] | 1.53 [1.45, 1.61] | +0.071 [+0.017, +0.126] | +0.071 [+0.032, +0.112] | +0.04 [-0.12, +0.21] |
| anchor | DP | 0.686 [0.659, 0.713] | 1.56 [1.48, 1.64] | +0.084 [+0.029, +0.138] | +0.084 [+0.048, +0.118] | +0.07 [-0.09, +0.24] |
| lgbm ext | plain | 0.568 [0.540, 0.595] | 1.69 [1.61, 1.78] | — | — | — |
| lgbm ext | static 40-60 | 0.571 [0.544, 0.598] | 1.78 [1.70, 1.86] | +0.004 [-0.051, +0.058] | +0.004 [-0.043, +0.053] | +0.09 [-0.08, +0.25] |
| lgbm ext | re-tuned window | 0.576 [0.548, 0.603] | 1.67 [1.59, 1.76] | +0.008 [-0.047, +0.063] | +0.008 [-0.021, +0.037] | -0.02 [-0.18, +0.15] |
| lgbm ext | DP | 0.703 [0.675, 0.730] | 1.61 [1.53, 1.69] | +0.135 [+0.080, +0.190] | +0.135 [+0.084, +0.189] | -0.08 [-0.25, +0.08] |
| lgbm ext | DP built for 25% noise | 0.686 [0.659, 0.714] | 1.59 [1.51, 1.68] | +0.119 [+0.064, +0.174] | +0.119 [+0.073, +0.169] | -0.10 [-0.26, +0.06] |

DP flips per trial (all DP bots, in order): anchor 1.2, lgbm ext 2.6. Re-tuned windows chosen (upper bound of the favourite's probability, 0.50 = never flip; Events): anchor: 0.50×5, 0.56×2, 0.60×33, 0.62×10, 0.68×1, 0.70×6; lgbm ext: 0.50×9, 0.52×35, 0.54×2, 0.56×10, 0.60×1.

### favourite follower, 10% noise, 12 members

| Probabilities | Policy | P(1st) | Mean place | Δ P(1st) vs plain (Hoeffding) | Δ P(1st) (Event bootstrap) | Δ mean place (Hoeffding) |
|---|---|---|---|---|---|---|
| anchor | plain | 0.190 [0.163, 0.217] | 2.84 [2.54, 3.14] | — | — | — |
| anchor | static 40-60 | 0.398 [0.370, 0.425] | 3.57 [3.27, 3.87] | +0.207 [+0.153, +0.262] | +0.207 [+0.144, +0.275] | +0.73 [+0.13, +1.33] |
| anchor | re-tuned window | 0.395 [0.368, 0.422] | 3.58 [3.27, 3.88] | +0.205 [+0.150, +0.260] | +0.205 [+0.142, +0.268] | +0.74 [+0.14, +1.34] |
| anchor | DP | 0.381 [0.354, 0.408] | 4.17 [3.87, 4.47] | +0.191 [+0.136, +0.246] | +0.191 [+0.142, +0.243] | +1.33 [+0.73, +1.94] |
| lgbm ext | plain | 0.290 [0.263, 0.317] | 3.70 [3.40, 4.00] | — | — | — |
| lgbm ext | static 40-60 | 0.296 [0.269, 0.324] | 4.73 [4.43, 5.03] | +0.006 [-0.048, +0.061] | +0.006 [-0.057, +0.069] | +1.03 [+0.43, +1.63] |
| lgbm ext | re-tuned window | 0.315 [0.288, 0.342] | 4.18 [3.88, 4.48] | +0.025 [-0.030, +0.080] | +0.025 [-0.030, +0.079] | +0.48 [-0.12, +1.08] |
| lgbm ext | DP | 0.474 [0.447, 0.501] | 4.13 [3.83, 4.43] | +0.184 [+0.129, +0.239] | +0.184 [+0.122, +0.244] | +0.43 [-0.17, +1.03] |
| lgbm ext | DP built for 25% noise | 0.437 [0.410, 0.464] | 4.23 [3.93, 4.53] | +0.147 [+0.092, +0.202] | +0.147 [+0.090, +0.205] | +0.53 [-0.08, +1.13] |

DP flips per trial (all DP bots, in order): anchor 3.7, lgbm ext 4.3. Re-tuned windows chosen (upper bound of the favourite's probability, 0.50 = never flip; Events): anchor: 0.50×3, 0.60×43, 0.66×1, 0.70×10; lgbm ext: 0.50×7, 0.52×2, 0.54×2, 0.56×32, 0.60×1, 0.62×9, 0.64×4.

### favourite follower, 25% noise, 4 members

| Probabilities | Policy | P(1st) | Mean place | Δ P(1st) vs plain (Hoeffding) | Δ P(1st) (Event bootstrap) | Δ mean place (Hoeffding) |
|---|---|---|---|---|---|---|
| anchor | plain | 0.718 [0.691, 0.745] | 1.34 [1.26, 1.42] | — | — | — |
| anchor | static 40-60 | 0.738 [0.710, 0.765] | 1.36 [1.28, 1.45] | +0.019 [-0.035, +0.074] | +0.019 [-0.014, +0.054] | +0.02 [-0.14, +0.19] |
| anchor | re-tuned window | 0.731 [0.703, 0.758] | 1.36 [1.28, 1.44] | +0.013 [-0.042, +0.067] | +0.013 [-0.016, +0.042] | +0.02 [-0.14, +0.19] |
| anchor | DP | 0.722 [0.695, 0.749] | 1.39 [1.31, 1.48] | +0.004 [-0.051, +0.059] | +0.004 [-0.011, +0.017] | +0.05 [-0.11, +0.22] |
| lgbm ext | plain | 0.699 [0.672, 0.726] | 1.45 [1.37, 1.53] | — | — | — |
| lgbm ext | static 40-60 | 0.688 [0.660, 0.715] | 1.48 [1.40, 1.56] | -0.011 [-0.066, +0.043] | -0.011 [-0.060, +0.040] | +0.03 [-0.13, +0.20] |
| lgbm ext | re-tuned window | 0.694 [0.667, 0.721] | 1.45 [1.37, 1.53] | -0.005 [-0.060, +0.050] | -0.005 [-0.031, +0.021] | +0.00 [-0.16, +0.17] |
| lgbm ext | DP | 0.733 [0.706, 0.760] | 1.43 [1.35, 1.51] | +0.034 [-0.021, +0.089] | +0.034 [-0.002, +0.075] | -0.02 [-0.18, +0.14] |
| lgbm ext | DP built for 10% noise | 0.727 [0.700, 0.754] | 1.47 [1.39, 1.55] | +0.028 [-0.027, +0.083] | +0.028 [-0.017, +0.079] | +0.02 [-0.14, +0.19] |

DP flips per trial (all DP bots, in order): anchor 0.5, lgbm ext 3.8. Re-tuned windows chosen (upper bound of the favourite's probability, 0.50 = never flip; Events): anchor: 0.50×24, 0.60×29, 0.62×4; lgbm ext: 0.50×20, 0.52×19, 0.54×15, 0.56×2, 0.60×1.

### favourite follower, 25% noise, 12 members

| Probabilities | Policy | P(1st) | Mean place | Δ P(1st) vs plain (Hoeffding) | Δ P(1st) (Event bootstrap) | Δ mean place (Hoeffding) |
|---|---|---|---|---|---|---|
| anchor | plain | 0.396 [0.369, 0.423] | 2.26 [1.96, 2.56] | — | — | — |
| anchor | static 40-60 | 0.443 [0.416, 0.470] | 2.61 [2.31, 2.91] | +0.047 [-0.008, +0.102] | +0.047 [-0.014, +0.106] | +0.35 [-0.25, +0.95] |
| anchor | re-tuned window | 0.440 [0.412, 0.467] | 2.51 [2.21, 2.81] | +0.043 [-0.011, +0.098] | +0.043 [-0.006, +0.094] | +0.25 [-0.36, +0.85] |
| anchor | DP | 0.429 [0.402, 0.457] | 2.84 [2.54, 3.14] | +0.034 [-0.021, +0.088] | +0.034 [+0.005, +0.061] | +0.58 [-0.02, +1.19] |
| lgbm ext | plain | 0.418 [0.391, 0.446] | 2.78 [2.48, 3.08] | — | — | — |
| lgbm ext | static 40-60 | 0.383 [0.356, 0.410] | 3.30 [3.00, 3.60] | -0.035 [-0.090, +0.019] | -0.035 [-0.107, +0.038] | +0.52 [-0.08, +1.12] |
| lgbm ext | re-tuned window | 0.400 [0.372, 0.427] | 2.77 [2.47, 3.07] | -0.019 [-0.074, +0.036] | -0.019 [-0.057, +0.015] | -0.01 [-0.62, +0.59] |
| lgbm ext | DP | 0.467 [0.440, 0.494] | 3.25 [2.95, 3.55] | +0.049 [-0.006, +0.103] | +0.049 [+0.005, +0.092] | +0.47 [-0.13, +1.07] |
| lgbm ext | DP built for 10% noise | 0.479 [0.452, 0.506] | 3.21 [2.90, 3.51] | +0.060 [+0.006, +0.115] | +0.060 [+0.011, +0.112] | +0.42 [-0.18, +1.03] |

DP flips per trial (all DP bots, in order): anchor 2.0, lgbm ext 5.5. Re-tuned windows chosen (upper bound of the favourite's probability, 0.50 = never flip; Events): anchor: 0.50×17, 0.60×37, 0.62×2, 0.70×1; lgbm ext: 0.50×13, 0.52×31, 0.54×2, 0.56×10, 0.60×1.

### market follower, 4 members

| Probabilities | Policy | P(1st) | Mean place | Δ P(1st) vs plain (Hoeffding) | Δ P(1st) (Event bootstrap) | Δ mean place (Hoeffding) |
|---|---|---|---|---|---|---|
| anchor | plain | 0.961 [0.934, 0.988] | 1.12 [1.03, 1.20] | — | — | — |
| anchor | static 40-60 | 0.979 [0.952, 1.000] | 1.06 [1.00, 1.15] | +0.018 [-0.037, +0.073] | +0.018 [+0.000, +0.053] | -0.05 [-0.22, +0.11] |
| anchor | re-tuned window | 0.961 [0.934, 0.988] | 1.12 [1.03, 1.20] | +0.000 [-0.055, +0.055] | +0.000 [+0.000, +0.000] | +0.00 [-0.16, +0.16] |
| anchor | DP | 0.941 [0.913, 0.968] | 1.18 [1.10, 1.26] | -0.021 [-0.075, +0.034] | -0.021 [-0.083, +0.042] | +0.06 [-0.10, +0.23] |
| lgbm ext | plain | 0.875 [0.847, 0.902] | 1.38 [1.29, 1.46] | — | — | — |
| lgbm ext | static 40-60 | 0.915 [0.888, 0.943] | 1.25 [1.17, 1.34] | +0.041 [-0.014, +0.096] | +0.041 [-0.019, +0.108] | -0.12 [-0.29, +0.04] |
| lgbm ext | re-tuned window | 0.922 [0.895, 0.950] | 1.23 [1.15, 1.31] | +0.048 [-0.007, +0.103] | +0.048 [-0.004, +0.104] | -0.14 [-0.31, +0.02] |
| lgbm ext | DP | 0.980 [0.952, 1.000] | 1.06 [1.00, 1.14] | +0.105 [+0.050, +0.160] | +0.105 [+0.039, +0.176] | -0.32 [-0.48, -0.15] |

DP flips per trial (all DP bots, in order): anchor 0.3, lgbm ext 0.8. Re-tuned windows chosen (upper bound of the favourite's probability, 0.50 = never flip; Events): anchor: 0.50×52, 0.60×5; lgbm ext: 0.50×41, 0.58×2, 0.60×1, 0.70×13.

### market follower, 12 members

| Probabilities | Policy | P(1st) | Mean place | Δ P(1st) vs plain (Hoeffding) | Δ P(1st) (Event bootstrap) | Δ mean place (Hoeffding) |
|---|---|---|---|---|---|---|
| anchor | plain | 0.961 [0.934, 0.988] | 1.43 [1.13, 1.73] | — | — | — |
| anchor | static 40-60 | 0.979 [0.952, 1.000] | 1.23 [1.00, 1.53] | +0.018 [-0.037, +0.073] | +0.018 [+0.000, +0.053] | -0.20 [-0.80, +0.40] |
| anchor | re-tuned window | 0.961 [0.934, 0.988] | 1.43 [1.13, 1.73] | +0.000 [-0.055, +0.055] | +0.000 [+0.000, +0.000] | +0.00 [-0.60, +0.60] |
| anchor | DP | 0.941 [0.913, 0.968] | 1.65 [1.35, 1.96] | -0.021 [-0.075, +0.034] | -0.021 [-0.083, +0.042] | +0.23 [-0.38, +0.83] |
| lgbm ext | plain | 0.875 [0.847, 0.902] | 2.38 [2.08, 2.68] | — | — | — |
| lgbm ext | static 40-60 | 0.915 [0.888, 0.943] | 1.93 [1.63, 2.23] | +0.041 [-0.014, +0.096] | +0.041 [-0.019, +0.108] | -0.45 [-1.05, +0.15] |
| lgbm ext | re-tuned window | 0.922 [0.895, 0.950] | 1.85 [1.55, 2.15] | +0.048 [-0.007, +0.103] | +0.048 [-0.004, +0.104] | -0.53 [-1.13, +0.07] |
| lgbm ext | DP | 0.980 [0.952, 1.000] | 1.23 [1.00, 1.53] | +0.105 [+0.050, +0.160] | +0.105 [+0.039, +0.176] | -1.16 [-1.76, -0.55] |

DP flips per trial (all DP bots, in order): anchor 0.3, lgbm ext 0.8. Re-tuned windows chosen (upper bound of the favourite's probability, 0.50 = never flip; Events): anchor: 0.50×52, 0.60×5; lgbm ext: 0.50×41, 0.58×2, 0.60×1, 0.70×13.

