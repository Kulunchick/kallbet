# How well do the veto simulators predict the actual map vetoes? (W6)

## Primary configuration (declared in a commit before any veto prediction was scored)

Written before any veto step was predicted or scored. Seen so far: only the shape of the data (veto
step patterns per format, record counts, map names) and the code of `features_signals.VetoEB` and the
Rust lab's `bo1_veto_sim`/`bo3_veto_sim`/`bo5_veto_sim`.

- **Data:** the extended archive with W5's recovered HLTV links (`ml-cache-w5/research-archive-ext`,
  9,305 Matches with a 7-step veto; 1,944 of them in scope). A veto is released to the predictor only
  when its Match's known finish is strictly before the evaluated Match's cutoff, so the evaluated
  Match's own veto is never seen. Scored set: the 2,087 in-scope anchor Matches that have a full
  7-step veto whose six acting teams resolve. Event bootstrap: 2000 resamples, seed 33, 90%.
- **What is predicted:** each veto step, given the acting team (known when the veto starts) and the
  realised earlier steps ("next-step" mode: the candidates are the detected active pool minus the maps
  already removed or picked). A second mode, "pre-veto", integrates over the unknown earlier steps
  (exact integration over the 5,040 ordered choices, in torch); it is reported but is not primary.
  The detected pool is the one `ActivePool` finds from released 7-step vetoes (top seven maps of the
  last 14 days, refreshed at most every 7 days). An actual choice outside the detected pool gets
  probability 1e-3 under every method.
- **Contender `eb`** (the empirical-Bayes simulator's choice model, as in `VetoEB`): a team removes
  (picks) map m with weight (number of the team's last 25 released vetoes, any format, in which it
  removed (picked) m + 0.3), normalised over the candidates.
- **Baseline `freq`:** the team's most frequent choice at that step type (format, ban/pick, the team's
  1st/2nd action of that kind) over its last N vetoes of the same format, with additive smoothing α for
  the log-loss. (N, α) is chosen per evaluated Event, walk-forward, from N ∈ {3, 5, 8, 12, 20, 40} ×
  α ∈ {0.25, 1}: the pair with the lowest next-step log-loss over the same-format vetoes of Events that
  ended before the evaluated Event began (all tiers in the extended archive). Other baselines: `uniform`
  (remaining active maps) and `global` (the most frequent choice at that step over the last G vetoes of
  the format, G walk-forward from {100, 300, 1000}).
- **Top-1 tie-break** (all methods): higher global frequency of that map at that step type, then map name.
- **Primary quantity:** top-1 hit rate on **team picks in Bo3** (veto steps 3 and 4), next-step mode,
  detected pool: `eb` minus `freq`, paired per pick step, Event-bootstrap 90% interval of the pooled
  difference.
- **Rule:** `eb` beats the frequency rule only if the interval lies wholly above 0. Otherwise: not shown.
  Everything else (other steps, pre-veto mode, sets, orders, calibration, breakdowns, the lab ports,
  oracle pool, extra variants) is exploratory and labelled so.
- **Expectation, written down so it is not found afterwards:** `eb` only counts a team's own picks over
  its last 25 vetoes (+0.3 smoothing), so at a Bo3 pick it is the frequency rule with N = 25 plus Bo5
  picks. A difference near zero is the expected outcome.

## Conclusion (written after the run; the tables follow the marker below)

**Primary: not shown.** On Bo3 team picks (3,216 pick steps, 1,608 Matches, 55 Events) the empirical-Bayes
simulator's top-1 hit rate is 43.3% against 45.2% for the team-frequency rule: EB − freq = **−1.8 pp
[−3.2, −0.3]**. The interval is wholly *below* zero, so the rule (lower bound above 0) fails and the
frequency rule is, if anything, better. Uniform guessing gets 26.4%, the global most frequent pick 27.5%.
This was the expected outcome: at a pick the simulator only counts the team's own picks over its last 25
vetoes, while the baseline's walk-forward N is 8 (12 for 29% of the Matches): recency matters more than sample size.

**What is predictable (Bo3, 1,608 scored Matches, each step given the acting team and the realised earlier
steps, top-1 hit rate of the best simple rule, chance in brackets):**

| Step | Team freq | EB sim | Lab sim | Uniform |
|---|---|---|---|---|
| First two bans | **83.3%** | 77.8% | 67.8% | 16.0% |
| Team picks | 45.2% | 43.3% | 40.2% | 26.4% |
| Closing bans (5-6) | 53.8% | 55.6% | 49.8% | 37.8% |
| Decider, seen before the veto (pre-veto mode) | 27.5% | 21.1% | 13.4% | 18.8% |

- **Opening bans are close to permabans:** four in five are the map the team banned at that slot
  recently, and the frequency rule's probabilities are within 0.06 of the observed rate in every bin. This is the one
  step a frequency rule predicts well; log-loss 0.65 nats against 1.89 for uniform.
- **Picks are half predictable** (45% top-1, log-loss 1.33 against 1.52): a team has a few favourite maps
  and the choice among them depends on what is left. **Closing bans** (54-58%) and the **decider** (about
  27% top-1 before the veto) are the weak part. The three maps played as a set are hit exactly in 13-18%
  of Bo3 Matches by the three simulators (chance 2.9%) with an expected overlap of 1.55-1.68 of 3 maps
  (chance 1.29); the exact order of the three maps is right 4-8% of the time (chance 0.5%).
- **Tier, team sample size and pool changes change little.** Tier S and A are alike (picks 43-44% EB,
  45-46% freq). 85% of steps are by teams with 40 or more earlier Bo3 vetoes (the extended archive
  gives every top team a long history), so the "few vetoes" rows are too small to read (32-87 pick
  steps, no usable interval). In the 28 days after a true pool change the picks are no worse (EB 43.0%,
  freq 42.7%). The pool matters only when the detector is wrong: with a wrong detected pool, picks drop
  from 45.6% to 35.2% (freq).

**Does our simulator beat the simple frequency rule?** Not for Bo3, which is the format it was built for.
It is *worse* at the opening bans (−5.5 pp [−7.1, −4.0], log-loss +0.43) because it pools a team's four ban
slots into one weight vector instead of keeping the slot ("first ban" vs "second ban"), roughly equal at
picks (log-loss +0.001 [−0.023, +0.022]), and slightly better at closing bans (+1.8 pp [+0.1, +3.5]). The
Rust lab's Bo3 port is clearly worse than both (−15.5 pp at opening bans, −4.9 pp at picks): it counts every
earlier veto with no recency. **In Bo1 (255 Matches) and Bo5 (52 Matches) the EB simulator beats the
frequency rule** (Bo1 bans +5.3 pp [+0.3, +9.2], +7.8 [+2.6, +12.9], +12.9 [+8.9, +18.3] by step group; Bo5
picks +9.6 [+3.8, +14.9]), but a plain "removal frequency over the last N vetoes of any format" rule, which
is what EB's weights are, gets the same gain (exploratory, +6.9 to +12.4 pp in Bo1). So the Bo1/Bo5 edge
comes from pooling vetoes across formats (a team has few Bo1 vetoes), not from simulation. The simulator
is a map-choice model without map strength, so it adds nothing to team frequency at the choice step; its
value, if any, is in the later conversion of the sampled maps into a series probability, which this page
does not test. Exploratory: a team-frequency rule shrunk to the global step distribution has lower log-loss than
the plain frequency rule at picks (−0.034) and closing bans (−0.083) but higher at the opening bans (+0.048).

**Calibration.** In next-step mode the frequency rule is nearly calibrated (expected calibration error
0.016) and the EB simulator is not (0.047): it under-states maps it ranks first (a 0.45 forecast comes
true 0.60 of the time) and over-states the very confident ones. Before the veto (pre-veto mode) the
simulator's high-probability maps are badly over-confident (forecast 0.84, observed 0.14 on 22
candidates; lab sim 0.92 → 0.00 on 7). Anyone using
the simulator's step probabilities should treat anything above 0.6 as unreliable.

**Pool detector (facts from our own data).** The detected pool equals the seven maps of the Match's own
veto for 95.9% of the 1,915 scored Matches. All 79 exceptions are the first Event after three real pool
changes (first scored Match 2025-07-23 Overpass in / Anubis out, 2026-01-14 Anubis in / Train out,
2026-07-01 Cache in / Overpass out): the detector sees them 6.2, 5.9 and 8.2 days late. The 2025-01 change
(Train in, Vertigo out) was seen 6 days *before* the first scored Match because lower-tier Events had
already used it. The lab's rule (the pool of the latest earlier veto of that format) is right 99.3% of
the time and is the better detector. Replacing the detected pool by the actual seven maps (oracle) moves
hit rates by 0.4-0.8 pp and leaves the primary difference unchanged (−1.8 pp [−3.3, −0.3]), so pool
errors do not explain the primary result.

**Caveats.** (1) Next-step mode gives each method the acting team and the realised earlier steps; pre-veto
mode (reported separately) is harder (picks 33-41% top-1). (2) Scored Matches are those with a linked HLTV
page and a full veto: 1,915 of the 2,087 anchor Matches (1,608 Bo3, 255 Bo1, 52 Bo5). (3) Bo1/Bo5 and
the small subsets have few Events, so their intervals are wide or degenerate (a subset inside one Event
has a zero-width interval). (4) Everything except the primary is exploratory; with this many tables some
intervals will exclude zero by chance.

<!-- tables -->

# Tables (generated by `uv run python -m kbml.veto_report`)

Scored vetoes: 1915 of the 2,087 anchor Matches (Bo1 255, Bo3 1608, Bo5 52); the rest have no linked HLTV page or no full veto. Intervals: 90%, Event bootstrap, 2000 resamples, seed 33. Pool = detected pool unless stated.

## Primary result

Bo3 team picks (veto steps 3 and 4), next-step mode, detected pool; 1608 scored Bo3 Matches with a full veto (55 Events), 3216 pick steps. Top-1 hit rate, 90% Event-bootstrap interval (2000 resamples, seed 33).

| Method | Top-1 hit rate (%) |
|---|---|
| uniform | 26.4 [24.9, 27.8] |
| global | 27.5 [26.1, 28.9] |
| team freq | 45.2 [43.1, 47.3] |
| EB sim | 43.3 [41.1, 45.7] |

**EB − team freq: -1.8 pp [-3.2, -0.3]**; rule (lower bound > 0): interval not wholly above 0: **not shown** (wholly below 0: the frequency rule is better).

## Step by step (exploratory)

**Bo3: Top-1 hit rate (%), next-step mode** (1608 Matches)

| Step | uniform | global | team freq | EB sim | lab sim | freq any format (expl.) | shrunk (expl.) |
|---|---|---|---|---|---|---|---|
| first bans (steps 1-2) | 16.0 [14.3, 17.7] | 21.2 [19.3, 23.0] | 83.3 [81.7, 84.9] | 77.8 [75.6, 80.0] | 67.8 [65.3, 70.2] | 81.7 [79.9, 83.5] | 83.8 [82.3, 85.4] |
| team picks (steps 3-4) | 26.4 [24.9, 27.8] | 27.5 [26.1, 28.9] | 45.2 [43.1, 47.3] | 43.3 [41.1, 45.7] | 40.2 [38.5, 42.0] | 45.5 [43.5, 47.4] | 45.6 [43.5, 47.5] |
| second bans (steps 5-6) | 37.8 [35.7, 39.8] | 45.9 [43.2, 48.3] | 53.8 [52.0, 55.5] | 55.6 [53.3, 57.8] | 49.8 [48.1, 51.6] | 57.8 [55.5, 59.7] | 55.6 [53.7, 57.3] |
| ban 1 | 15.8 [13.9, 17.8] | 19.4 [17.4, 21.5] | 81.0 [78.8, 83.2] | 75.6 [73.1, 78.0] | 65.2 [62.1, 68.3] | 79.2 [76.7, 81.8] | 81.7 [79.5, 83.8] |
| ban 2 | 16.1 [13.6, 18.8] | 23.0 [19.9, 26.2] | 85.6 [83.5, 87.5] | 80.0 [77.5, 82.7] | 70.3 [67.3, 73.1] | 84.1 [82.0, 86.2] | 86.0 [84.0, 87.9] |
| pick 1 | 25.1 [23.1, 27.2] | 25.7 [23.5, 27.9] | 44.2 [41.5, 47.0] | 42.3 [39.5, 45.3] | 39.9 [37.5, 42.4] | 44.7 [42.1, 47.3] | 44.0 [41.4, 46.7] |
| pick 2 | 27.6 [25.6, 29.6] | 29.2 [27.7, 30.8] | 46.1 [43.6, 48.9] | 44.4 [41.9, 47.2] | 40.5 [38.8, 42.4] | 46.3 [43.9, 48.7] | 47.1 [44.5, 49.7] |
| ban 3 | 33.3 [31.1, 35.5] | 37.4 [34.8, 39.8] | 47.5 [45.6, 49.3] | 48.4 [46.2, 50.7] | 44.2 [42.0, 46.3] | 50.7 [48.8, 52.6] | 48.8 [46.6, 51.0] |
| ban 4 | 42.4 [39.8, 45.1] | 54.4 [50.7, 58.0] | 60.1 [57.6, 62.5] | 62.7 [59.5, 65.8] | 55.5 [53.1, 57.8] | 64.8 [61.6, 67.7] | 62.4 [59.8, 64.8] |

**Bo3: Log-loss (nats), next-step mode** (1608 Matches)

| Step | uniform | global | team freq | EB sim | lab sim | freq any format (expl.) | shrunk (expl.) |
|---|---|---|---|---|---|---|---|
| first bans (steps 1-2) | 1.891 [1.869, 1.923] | 1.880 [1.851, 1.916] | 0.645 [0.597, 0.702] | 1.071 [1.025, 1.121] | 1.214 [1.179, 1.248] | 1.040 [1.000, 1.087] | 0.694 [0.647, 0.749] |
| team picks (steps 3-4) | 1.517 [1.499, 1.540] | 1.506 [1.487, 1.528] | 1.329 [1.288, 1.370] | 1.329 [1.287, 1.373] | 1.367 [1.340, 1.394] | 1.312 [1.274, 1.350] | 1.295 [1.260, 1.331] |
| second bans (steps 5-6) | 0.963 [0.901, 1.042] | 0.958 [0.890, 1.044] | 0.997 [0.928, 1.083] | 0.935 [0.861, 1.023] | 0.948 [0.916, 0.983] | 0.921 [0.848, 1.012] | 0.914 [0.847, 1.000] |
| ban 1 | 1.968 [1.946, 2.007] | 1.970 [1.936, 2.011] | 0.741 [0.677, 0.807] | 1.151 [1.102, 1.206] | 1.287 [1.245, 1.325] | 1.121 [1.078, 1.170] | 0.778 [0.716, 0.843] |
| ban 2 | 1.815 [1.792, 1.843] | 1.790 [1.758, 1.825] | 0.549 [0.497, 0.610] | 0.990 [0.940, 1.047] | 1.142 [1.092, 1.191] | 0.958 [0.911, 1.011] | 0.609 [0.559, 0.667] |
| pick 1 | 1.628 [1.609, 1.651] | 1.617 [1.593, 1.642] | 1.404 [1.354, 1.456] | 1.404 [1.347, 1.458] | 1.455 [1.415, 1.494] | 1.385 [1.338, 1.431] | 1.376 [1.331, 1.423] |
| pick 2 | 1.406 [1.390, 1.429] | 1.396 [1.377, 1.417] | 1.253 [1.202, 1.303] | 1.255 [1.208, 1.298] | 1.279 [1.247, 1.307] | 1.239 [1.190, 1.285] | 1.213 [1.171, 1.254] |
| ban 3 | 1.153 [1.106, 1.215] | 1.155 [1.098, 1.224] | 1.201 [1.139, 1.274] | 1.116 [1.050, 1.191] | 1.134 [1.099, 1.171] | 1.116 [1.051, 1.192] | 1.107 [1.050, 1.176] |
| ban 4 | 0.772 [0.695, 0.871] | 0.762 [0.679, 0.868] | 0.793 [0.708, 0.901] | 0.754 [0.662, 0.863] | 0.761 [0.724, 0.802] | 0.726 [0.636, 0.837] | 0.722 [0.639, 0.828] |

**Bo3: Brier (multiclass), next-step mode** (1608 Matches)

| Step | uniform | global | team freq | EB sim | lab sim | freq any format (expl.) | shrunk (expl.) |
|---|---|---|---|---|---|---|---|
| first bans (steps 1-2) | 0.851 [0.845, 0.859] | 0.846 [0.838, 0.856] | 0.269 [0.248, 0.294] | 0.501 [0.485, 0.519] | 0.563 [0.550, 0.576] | 0.495 [0.481, 0.512] | 0.286 [0.266, 0.308] |
| team picks (steps 3-4) | 0.780 [0.775, 0.786] | 0.775 [0.769, 0.781] | 0.686 [0.667, 0.705] | 0.690 [0.672, 0.708] | 0.710 [0.698, 0.721] | 0.681 [0.663, 0.698] | 0.673 [0.657, 0.690] |
| second bans (steps 5-6) | 0.604 [0.585, 0.629] | 0.600 [0.576, 0.630] | 0.600 [0.572, 0.633] | 0.568 [0.536, 0.603] | 0.602 [0.584, 0.622] | 0.553 [0.522, 0.590] | 0.562 [0.536, 0.594] |
| ban 1 | 0.863 [0.857, 0.873] | 0.862 [0.853, 0.873] | 0.309 [0.280, 0.336] | 0.537 [0.518, 0.556] | 0.593 [0.576, 0.609] | 0.532 [0.516, 0.549] | 0.320 [0.294, 0.347] |
| ban 2 | 0.839 [0.833, 0.847] | 0.830 [0.820, 0.841] | 0.230 [0.206, 0.259] | 0.466 [0.446, 0.487] | 0.533 [0.515, 0.550] | 0.459 [0.441, 0.479] | 0.251 [0.229, 0.276] |
| pick 1 | 0.805 [0.800, 0.811] | 0.799 [0.791, 0.807] | 0.698 [0.676, 0.719] | 0.702 [0.679, 0.724] | 0.725 [0.707, 0.742] | 0.693 [0.672, 0.712] | 0.689 [0.670, 0.708] |
| pick 2 | 0.755 [0.751, 0.761] | 0.751 [0.745, 0.757] | 0.673 [0.646, 0.699] | 0.679 [0.656, 0.699] | 0.694 [0.681, 0.706] | 0.668 [0.644, 0.691] | 0.657 [0.635, 0.678] |
| ban 3 | 0.682 [0.669, 0.700] | 0.684 [0.664, 0.707] | 0.684 [0.659, 0.712] | 0.646 [0.619, 0.675] | 0.668 [0.651, 0.686] | 0.640 [0.613, 0.670] | 0.646 [0.623, 0.672] |
| ban 4 | 0.526 [0.501, 0.559] | 0.516 [0.485, 0.555] | 0.516 [0.479, 0.559] | 0.490 [0.447, 0.536] | 0.535 [0.510, 0.561] | 0.466 [0.425, 0.514] | 0.478 [0.444, 0.518] |

**Bo3: difference to the team-frequency rule, next-step mode** (paired per step; negative log-loss/Brier = better)

| Step group | Δ top-1 (pp), EB | Δ top-1 (pp), lab | Δ top-1 (pp), freq any format (expl.) | Δ log-loss, EB | Δ log-loss, lab | Δ log-loss, freq any format (expl.) | Δ log-loss, shrunk (expl.) |
|---|---|---|---|---|---|---|---|
| first bans (steps 1-2) | -5.5 [-7.1, -4.0] | -15.5 [-17.8, -13.5] | -1.6 [-2.6, -0.7] | +0.425 [+0.400, +0.452] | +0.569 [+0.519, +0.614] | +0.394 [+0.373, +0.417] | +0.048 [+0.038, +0.058] |
| team picks (steps 3-4) | -1.8 [-3.2, -0.3] | -4.9 [-6.8, -3.0] | +0.3 [-0.7, +1.3] | +0.001 [-0.023, +0.022] | +0.039 [+0.006, +0.070] | -0.017 [-0.029, -0.005] | -0.034 [-0.045, -0.023] |
| second bans (steps 5-6) | +1.8 [+0.1, +3.5] | -4.0 [-5.7, -2.3] | +4.0 [+2.6, +5.4] | -0.062 [-0.086, -0.037] | -0.049 [-0.126, +0.017] | -0.076 [-0.092, -0.059] | -0.083 [-0.097, -0.068] |

**Bo1: Top-1 hit rate (%), next-step mode** (255 Matches)

| Step | uniform | global | team freq | EB sim | lab sim | freq any format (expl.) | shrunk (expl.) |
|---|---|---|---|---|---|---|---|
| opening bans (steps 1-2) | 16.9 [12.0, 22.6] | 20.6 [18.0, 23.5] | 46.1 [42.3, 52.5] | 51.4 [46.7, 56.5] | 40.2 [35.7, 46.5] | 52.9 [47.9, 57.6] | 48.0 [44.6, 53.3] |
| middle bans (steps 3-4) | 25.9 [22.5, 29.9] | 27.6 [24.3, 30.8] | 47.6 [42.7, 52.7] | 55.5 [51.9, 58.3] | 41.2 [36.4, 45.9] | 55.3 [51.9, 58.0] | 47.3 [42.6, 51.3] |
| closing bans (steps 5-6) | 38.0 [32.9, 41.2] | 42.0 [35.5, 46.6] | 43.1 [36.6, 47.4] | 56.1 [49.6, 61.3] | 50.4 [45.6, 54.1] | 55.5 [50.0, 58.7] | 44.3 [37.6, 48.7] |
| ban 1 | 16.9 [9.5, 23.8] | 20.0 [14.3, 24.8] | 61.2 [56.8, 66.8] | 70.2 [61.7, 76.9] | 47.5 [40.7, 56.8] | 73.3 [65.0, 79.5] | 63.1 [57.4, 68.9] |
| ban 2 | 16.9 [12.6, 24.2] | 21.2 [18.8, 26.5] | 31.0 [26.3, 39.7] | 32.5 [27.6, 41.0] | 32.9 [28.7, 38.1] | 32.5 [26.2, 40.0] | 32.9 [28.2, 41.9] |
| ban 3 | 23.9 [18.8, 28.6] | 22.4 [17.6, 26.3] | 58.4 [50.4, 64.7] | 72.2 [66.1, 77.0] | 50.6 [44.1, 58.6] | 71.8 [64.6, 76.5] | 60.0 [52.7, 65.4] |
| ban 4 | 27.8 [24.1, 32.9] | 32.9 [29.0, 36.5] | 36.9 [32.3, 42.7] | 38.8 [35.2, 43.0] | 31.8 [25.9, 37.3] | 38.8 [35.8, 42.9] | 34.5 [30.6, 39.2] |
| ban 5 | 35.7 [26.8, 42.5] | 36.1 [28.6, 41.9] | 41.6 [35.6, 45.6] | 54.1 [48.8, 59.8] | 45.9 [41.4, 49.9] | 51.8 [48.2, 54.2] | 40.8 [34.4, 45.0] |
| ban 6 | 40.4 [36.4, 44.3] | 47.8 [41.6, 52.7] | 44.7 [36.0, 50.8] | 58.0 [49.4, 65.1] | 54.9 [47.3, 59.8] | 59.2 [49.4, 65.2] | 47.8 [39.1, 54.1] |

**Bo1: Log-loss (nats), next-step mode** (255 Matches)

| Step | uniform | global | team freq | EB sim | lab sim | freq any format (expl.) | shrunk (expl.) |
|---|---|---|---|---|---|---|---|
| opening bans (steps 1-2) | 1.879 [1.869, 1.902] | 1.926 [1.885, 1.976] | 1.493 [1.406, 1.556] | 1.431 [1.376, 1.488] | 1.650 [1.588, 1.693] | 1.408 [1.354, 1.472] | 1.553 [1.434, 1.654] |
| middle bans (steps 3-4) | 1.520 [1.498, 1.572] | 1.547 [1.501, 1.606] | 1.308 [1.247, 1.388] | 1.216 [1.160, 1.306] | 1.372 [1.319, 1.444] | 1.233 [1.170, 1.327] | 1.341 [1.268, 1.436] |
| closing bans (steps 5-6) | 1.083 [0.896, 1.515] | 1.111 [0.914, 1.536] | 1.123 [0.916, 1.571] | 1.041 [0.820, 1.484] | 0.936 [0.875, 1.034] | 1.061 [0.856, 1.482] | 1.152 [0.943, 1.577] |
| ban 1 | 1.946 [1.946, 1.946] | 1.964 [1.916, 2.038] | 1.299 [1.195, 1.387] | 1.239 [1.151, 1.327] | 1.613 [1.517, 1.679] | 1.218 [1.129, 1.307] | 1.295 [1.154, 1.437] |
| ban 2 | 1.812 [1.792, 1.858] | 1.887 [1.831, 1.946] | 1.688 [1.586, 1.752] | 1.622 [1.553, 1.709] | 1.687 [1.628, 1.740] | 1.598 [1.521, 1.710] | 1.811 [1.702, 1.893] |
| ban 3 | 1.610 [1.609, 1.612] | 1.637 [1.595, 1.665] | 1.181 [1.073, 1.327] | 1.042 [0.974, 1.134] | 1.331 [1.245, 1.405] | 1.054 [0.966, 1.163] | 1.155 [1.043, 1.265] |
| ban 4 | 1.430 [1.386, 1.533] | 1.457 [1.376, 1.593] | 1.435 [1.368, 1.530] | 1.390 [1.308, 1.524] | 1.413 [1.360, 1.519] | 1.413 [1.317, 1.539] | 1.527 [1.425, 1.684] |
| ban 5 | 1.216 [1.099, 1.487] | 1.235 [1.089, 1.517] | 1.217 [1.072, 1.520] | 1.119 [0.968, 1.391] | 1.076 [1.007, 1.159] | 1.169 [1.018, 1.434] | 1.232 [1.089, 1.514] |
| ban 6 | 0.950 [0.693, 1.542] | 0.988 [0.728, 1.563] | 1.028 [0.746, 1.625] | 0.962 [0.662, 1.571] | 0.795 [0.727, 0.922] | 0.953 [0.652, 1.543] | 1.071 [0.786, 1.658] |

**Bo1: Brier (multiclass), next-step mode** (255 Matches)

| Step | uniform | global | team freq | EB sim | lab sim | freq any format (expl.) | shrunk (expl.) |
|---|---|---|---|---|---|---|---|
| opening bans (steps 1-2) | 0.848 [0.845, 0.854] | 0.856 [0.846, 0.865] | 0.686 [0.643, 0.715] | 0.678 [0.649, 0.704] | 0.763 [0.740, 0.778] | 0.676 [0.648, 0.706] | 0.677 [0.625, 0.712] |
| middle bans (steps 3-4) | 0.781 [0.775, 0.795] | 0.789 [0.776, 0.807] | 0.672 [0.642, 0.710] | 0.629 [0.604, 0.669] | 0.713 [0.692, 0.739] | 0.631 [0.603, 0.670] | 0.670 [0.642, 0.704] |
| closing bans (steps 5-6) | 0.642 [0.583, 0.778] | 0.662 [0.598, 0.797] | 0.676 [0.602, 0.824] | 0.603 [0.515, 0.753] | 0.595 [0.565, 0.644] | 0.604 [0.526, 0.748] | 0.687 [0.614, 0.824] |
| ban 1 | 0.857 [0.857, 0.857] | 0.858 [0.845, 0.873] | 0.582 [0.531, 0.622] | 0.579 [0.543, 0.617] | 0.739 [0.709, 0.757] | 0.581 [0.544, 0.619] | 0.551 [0.488, 0.605] |
| ban 2 | 0.839 [0.833, 0.851] | 0.854 [0.839, 0.866] | 0.791 [0.742, 0.820] | 0.778 [0.732, 0.812] | 0.788 [0.765, 0.805] | 0.772 [0.731, 0.818] | 0.804 [0.750, 0.834] |
| ban 3 | 0.800 [0.800, 0.800] | 0.811 [0.798, 0.820] | 0.581 [0.521, 0.652] | 0.519 [0.484, 0.569] | 0.674 [0.640, 0.704] | 0.527 [0.489, 0.582] | 0.560 [0.503, 0.622] |
| ban 4 | 0.762 [0.750, 0.790] | 0.768 [0.745, 0.805] | 0.763 [0.744, 0.787] | 0.739 [0.704, 0.783] | 0.753 [0.731, 0.790] | 0.735 [0.699, 0.774] | 0.781 [0.758, 0.813] |
| ban 5 | 0.700 [0.667, 0.778] | 0.707 [0.658, 0.799] | 0.701 [0.648, 0.800] | 0.623 [0.562, 0.712] | 0.634 [0.603, 0.671] | 0.642 [0.590, 0.727] | 0.699 [0.647, 0.795] |
| ban 6 | 0.584 [0.500, 0.777] | 0.616 [0.528, 0.800] | 0.651 [0.542, 0.846] | 0.582 [0.459, 0.797] | 0.557 [0.513, 0.635] | 0.565 [0.446, 0.779] | 0.675 [0.573, 0.857] |

**Bo1: difference to the team-frequency rule, next-step mode** (paired per step; negative log-loss/Brier = better)

| Step group | Δ top-1 (pp), EB | Δ top-1 (pp), lab | Δ top-1 (pp), freq any format (expl.) | Δ log-loss, EB | Δ log-loss, lab | Δ log-loss, freq any format (expl.) | Δ log-loss, shrunk (expl.) |
|---|---|---|---|---|---|---|---|
| opening bans (steps 1-2) | +5.3 [+0.3, +9.2] | -5.9 [-10.2, -1.7] | +6.9 [+0.4, +10.9] | -0.063 [-0.110, +0.001] | +0.157 [+0.096, +0.224] | -0.085 [-0.126, -0.019] | +0.060 [+0.005, +0.128] |
| middle bans (steps 3-4) | +7.8 [+2.6, +12.9] | -6.5 [-9.0, -4.0] | +7.6 [+3.5, +11.7] | -0.092 [-0.146, -0.031] | +0.064 [-0.018, +0.139] | -0.075 [-0.140, -0.006] | +0.033 [-0.008, +0.081] |
| closing bans (steps 5-6) | +12.9 [+8.9, +18.3] | +7.3 [+2.7, +11.8] | +12.4 [+9.9, +14.9] | -0.082 [-0.137, -0.033] | -0.187 [-0.543, -0.007] | -0.061 [-0.105, -0.022] | +0.029 [-0.017, +0.064] |

**Bo5: Top-1 hit rate (%), next-step mode** (52 Matches)

| Step | uniform | global | team freq | EB sim | lab sim |
|---|---|---|---|---|---|
| bans (steps 1-2) | 28.8 [22.1, 35.6] | 28.8 [22.1, 35.6] | 70.2 [62.5, 76.9] | 93.3 [88.5, 97.1] | 76.9 [70.2, 83.7] |
| team picks (steps 3-6) | 37.0 [32.2, 41.8] | 37.0 [32.2, 41.8] | 42.3 [37.0, 47.6] | 51.9 [47.1, 56.7] | 52.4 [46.6, 58.7] |
| ban 1 | 21.2 [11.5, 30.8] | 21.2 [11.5, 30.8] | 71.2 [61.5, 80.8] | 92.3 [86.5, 98.1] | 82.7 [73.1, 90.4] |
| ban 2 | 36.5 [25.0, 46.2] | 36.5 [25.0, 46.2] | 69.2 [57.7, 78.8] | 94.2 [88.5, 98.1] | 71.2 [61.5, 80.8] |
| pick 1 | 21.2 [11.5, 30.8] | 21.2 [11.5, 30.8] | 28.8 [19.2, 40.4] | 40.4 [28.8, 51.9] | 34.6 [25.0, 46.2] |
| pick 2 | 26.9 [17.3, 36.5] | 26.9 [17.3, 36.5] | 38.5 [26.9, 50.0] | 38.5 [26.9, 50.0] | 42.3 [30.8, 53.8] |
| ban 3 | 42.3 [30.8, 53.8] | 42.3 [30.8, 53.8] | 50.0 [38.5, 61.5] | 53.8 [42.3, 65.4] | 63.5 [51.9, 73.1] |
| ban 4 | 57.7 [46.2, 69.2] | 57.7 [46.2, 69.2] | 51.9 [40.4, 63.5] | 75.0 [63.5, 84.6] | 69.2 [57.7, 78.8] |

**Bo5: Log-loss (nats), next-step mode** (52 Matches)

| Step | uniform | global | team freq | EB sim | lab sim |
|---|---|---|---|---|---|
| bans (steps 1-2) | 1.869 [1.869, 1.869] | 1.707 [1.608, 1.808] | 0.962 [0.836, 1.102] | 0.804 [0.747, 0.871] | 0.977 [0.897, 1.066] |
| team picks (steps 3-6) | 1.197 [1.197, 1.197] | 1.242 [1.163, 1.324] | 1.236 [1.147, 1.323] | 1.047 [0.963, 1.129] | 1.057 [0.985, 1.130] |
| ban 1 | 1.946 [1.946, 1.946] | 1.934 [1.797, 2.092] | 0.940 [0.745, 1.149] | 0.853 [0.776, 0.942] | 0.972 [0.867, 1.091] |
| ban 2 | 1.792 [1.792, 1.792] | 1.480 [1.333, 1.635] | 0.985 [0.803, 1.166] | 0.755 [0.692, 0.825] | 0.982 [0.871, 1.102] |
| pick 1 | 1.609 [1.609, 1.609] | 1.695 [1.552, 1.850] | 1.721 [1.523, 1.917] | 1.349 [1.147, 1.561] | 1.419 [1.261, 1.584] |
| pick 2 | 1.386 [1.386, 1.386] | 1.483 [1.322, 1.658] | 1.433 [1.228, 1.651] | 1.243 [1.100, 1.401] | 1.242 [1.111, 1.393] |
| ban 3 | 1.099 [1.099, 1.099] | 1.140 [0.984, 1.288] | 1.100 [0.927, 1.283] | 1.088 [0.884, 1.300] | 1.007 [0.840, 1.194] |
| ban 4 | 0.693 [0.693, 0.693] | 0.650 [0.566, 0.746] | 0.688 [0.555, 0.835] | 0.508 [0.390, 0.653] | 0.561 [0.427, 0.717] |

**Bo5: Brier (multiclass), next-step mode** (52 Matches)

| Step | uniform | global | team freq | EB sim | lab sim |
|---|---|---|---|---|---|
| bans (steps 1-2) | 0.845 [0.845, 0.845] | 0.788 [0.756, 0.821] | 0.427 [0.366, 0.494] | 0.377 [0.348, 0.412] | 0.462 [0.423, 0.505] |
| team picks (steps 3-6) | 0.679 [0.679, 0.679] | 0.689 [0.653, 0.726] | 0.681 [0.633, 0.729] | 0.587 [0.547, 0.629] | 0.592 [0.550, 0.633] |
| ban 1 | 0.857 [0.857, 0.857] | 0.850 [0.817, 0.885] | 0.403 [0.310, 0.504] | 0.400 [0.360, 0.444] | 0.447 [0.394, 0.507] |
| ban 2 | 0.833 [0.833, 0.833] | 0.726 [0.672, 0.784] | 0.451 [0.359, 0.540] | 0.355 [0.319, 0.394] | 0.478 [0.419, 0.538] |
| pick 1 | 0.800 [0.800, 0.800] | 0.822 [0.781, 0.864] | 0.861 [0.768, 0.951] | 0.663 [0.585, 0.735] | 0.709 [0.642, 0.772] |
| pick 2 | 0.750 [0.750, 0.750] | 0.788 [0.726, 0.852] | 0.738 [0.635, 0.837] | 0.720 [0.642, 0.798] | 0.700 [0.634, 0.770] |
| ban 3 | 0.667 [0.667, 0.667] | 0.685 [0.594, 0.771] | 0.646 [0.542, 0.756] | 0.624 [0.518, 0.735] | 0.578 [0.493, 0.669] |
| ban 4 | 0.500 [0.500, 0.500] | 0.462 [0.387, 0.546] | 0.480 [0.381, 0.586] | 0.342 [0.249, 0.454] | 0.380 [0.280, 0.490] |

**Bo5: difference to the team-frequency rule, next-step mode** (paired per step; negative log-loss/Brier = better)

| Step group | Δ top-1 (pp), EB | Δ top-1 (pp), lab | Δ top-1 (pp), freq any format (expl.) | Δ log-loss, EB | Δ log-loss, lab | Δ log-loss, freq any format (expl.) | Δ log-loss, shrunk (expl.) |
|---|---|---|---|---|---|---|---|
| bans (steps 1-2) | +23.1 [+17.3, +28.8] | +6.7 [+1.0, +12.5] | +24.0 [+17.3, +30.8] | -0.159 [-0.263, -0.058] | +0.015 [-0.066, +0.097] | -0.129 [-0.243, -0.024] | +0.028 [-0.058, +0.113] |
| team picks (steps 3-6) | +9.6 [+3.8, +14.9] | +10.1 [+3.8, +16.3] | +17.8 [+12.0, +23.6] | -0.189 [-0.288, -0.087] | -0.178 [-0.275, -0.078] | -0.264 [-0.357, -0.169] | +0.023 [-0.055, +0.104] |

Walk-forward choice for `freq` on the scored Bo3 Matches: N=5, α=0.25: 48; N=8, α=0.25: 1096; N=12, α=0.25: 464.
Walk-forward choice for `global` on the scored Bo3 Matches: last 100: 48; last 300: 1560.
Walk-forward choice for `shrunk` on the scored Bo3 Matches: N=8, κ=1.0: 231; N=8, κ=3.0: 1294; N=20, κ=3.0: 83.

## Before the veto: what the simulators are used for (exploratory)

**Bo3: Top-1 hit rate (%), pre-veto mode** (1608 Matches; the earlier steps are not known)

| Step | uniform | global | team freq | team freq, integrated | EB sim | lab sim |
|---|---|---|---|---|---|---|
| ban 1 | 16.0 [14.0, 18.1] | 19.4 [17.4, 21.5] | 81.0 [78.8, 83.2] | 78.9 [76.6, 81.4] | 76.2 [73.7, 78.6] | 65.5 [62.3, 68.6] |
| ban 2 | 13.4 [11.2, 15.8] | 18.6 [15.8, 21.5] | 84.1 [82.0, 86.1] | 82.2 [80.1, 84.1] | 78.8 [76.3, 81.4] | 69.9 [67.0, 72.7] |
| pick 1 | 16.4 [14.4, 18.3] | 19.6 [17.4, 22.0] | 38.0 [35.6, 40.3] | 39.3 [36.9, 41.7] | 36.7 [34.4, 39.2] | 35.9 [33.6, 38.4] |
| pick 2 | 17.3 [15.5, 19.2] | 15.8 [14.3, 17.2] | 33.4 [30.9, 35.9] | 41.2 [38.8, 43.6] | 38.9 [36.6, 41.5] | 33.9 [32.0, 35.9] |
| ban 3 | 13.9 [12.1, 15.8] | 17.6 [15.4, 19.8] | 26.6 [24.9, 28.3] | 22.8 [21.1, 24.6] | 20.4 [18.8, 22.0] | 15.7 [14.1, 17.3] |
| ban 4 | 13.6 [11.9, 15.2] | 17.9 [16.0, 19.9] | 23.9 [22.1, 25.5] | 25.4 [24.1, 26.5] | 24.1 [22.2, 26.2] | 19.8 [18.1, 21.5] |
| decider | 18.8 [16.5, 21.0] | 16.2 [14.1, 18.3] | 27.5 [25.6, 29.2] | 24.8 [23.0, 26.7] | 21.1 [19.0, 23.2] | 13.4 [11.5, 15.3] |

**Bo3: Log-loss (nats), pre-veto mode** (1608 Matches; the earlier steps are not known)

| Step | uniform | global | team freq | team freq, integrated | EB sim | lab sim |
|---|---|---|---|---|---|---|
| ban 1 | 1.968 [1.946, 2.007] | 1.970 [1.936, 2.011] | 0.741 [0.677, 0.807] | 1.095 [1.053, 1.142] | 1.151 [1.102, 1.206] | 1.287 [1.245, 1.325] |
| ban 2 | 1.968 [1.946, 1.994] | 1.954 [1.924, 1.987] | 0.632 [0.579, 0.694] | 1.048 [1.007, 1.095] | 1.098 [1.050, 1.151] | 1.259 [1.210, 1.306] |
| pick 1 | 1.961 [1.946, 1.981] | 1.938 [1.912, 1.964] | 1.646 [1.598, 1.695] | 1.609 [1.560, 1.659] | 1.606 [1.553, 1.657] | 1.660 [1.621, 1.697] |
| pick 2 | 1.961 [1.949, 1.979] | 1.949 [1.933, 1.966] | 1.685 [1.635, 1.733] | 1.547 [1.501, 1.590] | 1.557 [1.511, 1.599] | 1.626 [1.593, 1.656] |
| ban 3 | 1.989 [1.952, 2.038] | 1.984 [1.937, 2.041] | 1.963 [1.903, 2.030] | 2.019 [1.960, 2.086] | 1.978 [1.921, 2.041] | 1.969 [1.943, 1.996] |
| ban 4 | 2.001 [1.946, 2.072] | 2.007 [1.947, 2.083] | 2.020 [1.949, 2.108] | 1.921 [1.851, 2.002] | 1.912 [1.846, 1.988] | 1.890 [1.865, 1.915] |
| decider | 1.961 [1.946, 1.981] | 1.936 [1.912, 1.964] | 1.870 [1.836, 1.905] | 1.822 [1.785, 1.861] | 1.883 [1.829, 1.939] | 2.135 [2.056, 2.221] |

**Bo1: Top-1 hit rate (%), pre-veto mode** (255 Matches; the earlier steps are not known)

| Step | uniform | global | team freq | team freq, integrated | EB sim | lab sim |
|---|---|---|---|---|---|---|
| ban 1 | 16.9 [9.5, 23.8] | 20.0 [14.3, 24.8] | 61.2 [56.8, 66.8] | 51.8 [46.8, 58.2] | 70.6 [62.8, 76.9] | 47.8 [41.8, 55.7] |
| ban 2 | 14.9 [11.6, 19.0] | 18.0 [15.2, 22.8] | 28.2 [23.0, 37.0] | 17.3 [12.8, 24.9] | 16.5 [9.8, 28.2] | 20.8 [15.3, 30.3] |
| ban 3 | 17.6 [12.5, 22.6] | 17.6 [13.3, 22.1] | 53.7 [45.2, 61.5] | 38.0 [30.5, 44.1] | 63.9 [55.0, 70.1] | 37.3 [30.0, 43.6] |
| ban 4 | 12.9 [8.3, 16.8] | 18.8 [14.5, 22.5] | 24.3 [20.8, 29.0] | 23.1 [20.0, 25.8] | 29.4 [22.6, 36.7] | 22.7 [19.2, 24.9] |
| ban 5 | 15.7 [6.7, 21.2] | 20.4 [16.2, 26.7] | 19.2 [15.2, 21.9] | 22.4 [18.3, 26.3] | 24.3 [21.9, 26.5] | 21.6 [18.3, 26.1] |
| ban 6 | 12.9 [10.4, 16.2] | 16.9 [13.2, 19.5] | 14.9 [11.2, 18.4] | 17.3 [11.6, 21.6] | 19.2 [16.3, 22.6] | 18.0 [13.2, 23.1] |
| leftover (map played) | 16.1 [14.0, 18.3] | 18.0 [13.4, 23.5] | 25.9 [22.7, 28.9] | 22.4 [19.6, 25.9] | 30.2 [25.6, 33.2] | 18.0 [14.9, 21.4] |

**Bo1: Log-loss (nats), pre-veto mode** (255 Matches; the earlier steps are not known)

| Step | uniform | global | team freq | team freq, integrated | EB sim | lab sim |
|---|---|---|---|---|---|---|
| ban 1 | 1.946 [1.946, 1.946] | 1.964 [1.916, 2.038] | 1.299 [1.195, 1.387] | 1.579 [1.515, 1.623] | 1.239 [1.151, 1.327] | 1.613 [1.517, 1.679] |
| ban 2 | 1.965 [1.946, 2.010] | 2.047 [1.989, 2.104] | 1.841 [1.732, 1.906] | 1.908 [1.853, 1.964] | 1.871 [1.786, 1.953] | 1.903 [1.845, 1.951] |
| ban 3 | 1.946 [1.946, 1.946] | 1.975 [1.935, 2.001] | 1.439 [1.304, 1.626] | 1.632 [1.560, 1.741] | 1.281 [1.203, 1.410] | 1.610 [1.536, 1.687] |
| ban 4 | 1.985 [1.946, 2.075] | 2.035 [1.960, 2.156] | 1.965 [1.903, 2.049] | 1.879 [1.822, 1.982] | 1.788 [1.720, 1.912] | 1.876 [1.832, 1.974] |
| ban 5 | 2.043 [1.946, 2.268] | 2.055 [1.923, 2.297] | 2.056 [1.914, 2.319] | 1.982 [1.876, 2.200] | 1.918 [1.804, 2.140] | 1.918 [1.871, 1.967] |
| ban 6 | 2.140 [1.946, 2.590] | 2.176 [1.972, 2.612] | 2.238 [2.014, 2.692] | 2.135 [1.898, 2.609] | 2.161 [1.920, 2.584] | 1.957 [1.894, 2.034] |
| leftover (map played) | 1.946 [1.946, 1.946] | 1.985 [1.936, 2.063] | 1.874 [1.840, 1.940] | 1.922 [1.856, 1.989] | 1.785 [1.683, 1.885] | 2.031 [1.963, 2.118] |

**Bo3 set of the three maps played and the pick order** (pre-veto: the acting teams are known, the earlier steps not; exact integration over the 5,040 ordered choices, torch on the GPU). Actual set = the two picked maps and the decider; 'order' = (map 1, map 2, map 3). Ties count in expectation (uniform: 1/35 for the set, 1/210 for the order). 'Expected overlap' = Σ over the three played maps of P(that map is played), maximum 3; 'top set overlap' = maps shared by the most likely set and the actual one.

| Forecast | Matches | Set: top-1 hit (%) | Set: top-3 hit (%) | Set log-loss | Expected overlap (of 3) | Top-set overlap (of 3) | Order: top-1 hit (%) | Order log-loss |
|---|---|---|---|---|---|---|---|---|
| uniform (every set equal) | 1608 | 2.8 [2.8, 2.9] | 8.5 [8.4, 8.6] | 3.587 [3.561, 3.622] | 1.28 [1.28, 1.28] | 1.28 [1.28, 1.28] | 0.5 [0.5, 0.5] | 5.362 [5.350, 5.378] |
| team freq, integrated | 1608 | 17.8 [16.3, 19.4] | 42.2 [39.9, 44.6] | 2.891 [2.822, 2.964] | 1.68 [1.67, 1.70] | 1.94 [1.91, 1.96] | 7.8 [6.8, 8.9] | 4.296 [4.216, 4.379] |
| EB sim | 1608 | 16.5 [14.7, 18.3] | 38.1 [35.6, 40.8] | 2.933 [2.853, 3.012] | 1.65 [1.63, 1.67] | 1.90 [1.86, 1.93] | 7.4 [6.2, 8.7] | 4.352 [4.263, 4.439] |
| lab sim | 1608 | 12.7 [11.1, 14.3] | 30.7 [28.2, 33.2] | 3.083 [3.024, 3.143] | 1.55 [1.54, 1.56] | 1.80 [1.77, 1.84] | 4.0 [3.0, 5.0] | 4.721 [4.630, 4.809] |
| **EB sim − team freq, integrated** (pp for hit rates) | 1608 | -1.4 [-3.2, +0.5] | -4.1 [-6.0, -2.1] | +0.041 [-0.001, +0.084] | -0.039 [-0.051, -0.027] | -0.041 [-0.071, -0.010] | -0.4 [-1.5, +0.7] | +0.055 [+0.001, +0.106] |
| **lab sim − team freq, integrated** (pp for hit rates) | 1608 | -5.2 [-7.3, -2.9] | -11.6 [-14.5, -8.6] | +0.192 [+0.120, +0.263] | -0.135 [-0.150, -0.118] | -0.135 [-0.175, -0.095] | -3.8 [-5.2, -2.4] | +0.425 [+0.334, +0.516] |

## Calibration (exploratory)

**Reliability (Bo3, next-step mode)**: probability given to each candidate map vs how often it was the choice (1608 Matches, steps 1, 2, 3, 4, 5; each cell: mean forecast → observed rate (n candidates)).

| Forecast bin | uniform | team freq | EB sim |
|---|---|---|---|
| 0.00-0.05 | – | 0.027 → 0.031 (20068) | 0.024 → 0.033 (8878) |
| 0.05-0.10 | – | 0.072 → 0.127 (2708) | 0.075 → 0.052 (6755) |
| 0.10-0.20 | 0.154 → 0.153 (20995) | 0.144 → 0.152 (4304) | 0.145 → 0.097 (9605) |
| 0.20-0.30 | 0.222 → 0.222 (14517) | 0.244 → 0.239 (3059) | 0.243 → 0.211 (5045) |
| 0.30-0.40 | 0.333 → 0.330 (4752) | 0.349 → 0.332 (2463) | 0.348 → 0.392 (3343) |
| 0.40-0.50 | – | 0.451 → 0.400 (1744) | 0.449 → 0.605 (3163) |
| 0.50-0.60 | – | 0.548 → 0.485 (1236) | 0.543 → 0.641 (2018) |
| 0.60-0.70 | – | 0.645 → 0.587 (1289) | 0.646 → 0.560 (814) |
| 0.70-0.80 | – | 0.747 → 0.716 (1158) | 0.745 → 0.622 (407) |
| 0.80-0.90 | – | 0.854 → 0.869 (1955) | 0.839 → 0.617 (183) |
| 0.90-1.00 | – | 0.910 → 0.896 (280) | 0.926 → 0.660 (53) |
| expected calibration error | 0.0009 | 0.0158 | 0.0467 |

**Reliability (Bo1, next-step mode)**: probability given to each candidate map vs how often it was the choice (255 Matches, steps 1, 2, 3, 4, 5; each cell: mean forecast → observed rate (n candidates)).

| Forecast bin | uniform | team freq | EB sim |
|---|---|---|---|
| 0.00-0.05 | – | 0.040 → 0.081 (62) | 0.023 → 0.037 (1088) |
| 0.05-0.10 | – | 0.069 → 0.056 (2033) | 0.075 → 0.052 (1077) |
| 0.10-0.20 | 0.154 → 0.154 (3321) | 0.139 → 0.149 (1841) | 0.148 → 0.136 (1785) |
| 0.20-0.30 | 0.222 → 0.221 (2303) | 0.238 → 0.241 (1072) | 0.245 → 0.201 (926) |
| 0.30-0.40 | 0.333 → 0.327 (756) | 0.338 → 0.328 (537) | 0.347 → 0.334 (595) |
| 0.40-0.50 | – | 0.427 → 0.439 (296) | 0.446 → 0.588 (486) |
| 0.50-0.60 | – | 0.526 → 0.506 (322) | 0.545 → 0.582 (268) |
| 0.60-0.70 | – | 0.641 → 0.677 (167) | 0.645 → 0.635 (96) |
| 0.70-0.80 | – | 0.739 → 0.708 (48) | 0.747 → 0.705 (44) |
| 0.80-0.90 | – | 0.818 → 0.500 (2) | 0.847 → 0.615 (13) |
| 0.90-1.00 | – | – | 0.926 → 1.000 (2) |
| expected calibration error | 0.0013 | 0.0116 | 0.0306 |

**Reliability (Bo3, pre-veto mode)**: probability given to each candidate map vs how often it was the choice (1608 Matches, steps 1, 2, 3, 4, 5, 6, 7; each cell: mean forecast → observed rate (n candidates)).

| Forecast bin | uniform | EB sim | lab sim |
|---|---|---|---|
| 0.00-0.05 | – | 0.022 → 0.028 (20442) | 0.027 → 0.037 (15428) |
| 0.05-0.10 | – | 0.074 → 0.073 (15069) | 0.075 → 0.079 (17386) |
| 0.10-0.20 | 0.143 → 0.142 (78792) | 0.146 → 0.126 (22908) | 0.145 → 0.133 (27988) |
| 0.20-0.30 | – | 0.243 → 0.201 (12006) | 0.240 → 0.218 (11606) |
| 0.30-0.40 | – | 0.342 → 0.373 (4542) | 0.344 → 0.367 (3425) |
| 0.40-0.50 | – | 0.446 → 0.676 (2659) | 0.442 → 0.616 (1888) |
| 0.50-0.60 | – | 0.535 → 0.702 (895) | 0.540 → 0.710 (775) |
| 0.60-0.70 | – | 0.638 → 0.428 (187) | 0.645 → 0.209 (148) |
| 0.70-0.80 | – | 0.741 → 0.387 (62) | 0.743 → 0.111 (90) |
| 0.80-0.90 | – | 0.843 → 0.136 (22) | 0.843 → 0.098 (51) |
| 0.90-1.00 | – | – | 0.922 → 0.000 (7) |
| expected calibration error | 0.0008 | 0.0264 | 0.0192 |

**Top-1 hit rate by the confidence of the forecast** (Bo3, next-step mode; confidence = probability of the top choice; steps 1-2 and 5-6 are bans, 3-4 team picks)

| Confidence of the top choice | Bans: n | Bans: EB hit (%) | Bans: freq hit (%) | Picks: n | Picks: EB hit (%) | Picks: freq hit (%) |
|---|---|---|---|---|---|---|
| 0.0-0.3 | 223 / 50 | 31.4 | 18.0 | 86 / 65 | 24.4 | 20.0 |
| 0.3-0.5 | 2817 / 737 | 69.0 | 38.8 | 1456 / 1280 | 33.9 | 35.7 |
| 0.5-0.7 | 2300 / 1852 | 66.7 | 55.5 | 1323 / 1337 | 49.4 | 48.8 |
| 0.7-0.9 | 862 / 3267 | 65.7 | 81.0 | 316 / 510 | 63.3 | 62.0 |
| 0.9-1.0 | 230 / 526 | 75.7 | 83.8 | 35 / 24 | 74.3 | 62.5 |

(n is shown as EB / freq in the first column of each group)

## Breakdowns (exploratory)

**What Bo3 vetoes look like** (1608 scored Bo3 Matches): for each map, how often it was the choice at each step when it was in the Match's pool (%).

| Map | In the pool (Matches) | Banned at steps 1-2 | Picked (steps 3-4) | Banned at steps 5-6 | Decider |
|---|---|---|---|---|---|
| Ancient | 1608 | 34.0 | 29.1 | 22.0 | 14.9 |
| Anubis | 1268 | 41.6 | 20.0 | 30.0 | 8.5 |
| Cache | 245 | 15.1 | 23.7 | 53.1 | 8.2 |
| Dust2 | 1608 | 20.5 | 35.6 | 27.5 | 16.4 |
| Inferno | 1608 | 26.6 | 32.2 | 25.4 | 15.9 |
| Mirage | 1608 | 22.1 | 35.6 | 23.6 | 18.7 |
| Nuke | 1608 | 23.9 | 28.2 | 29.5 | 18.3 |
| Overpass | 730 | 28.2 | 20.8 | 42.7 | 8.2 |
| Train | 711 | 38.1 | 17.2 | 36.8 | 7.9 |
| Vertigo | 262 | 49.6 | 17.6 | 28.6 | 4.2 |

**Bo3 team picks (steps 3-4)** by subset (next-step mode; Matches 1608)

| Subset | Steps | uniform hit (%) | team freq hit (%) | EB hit (%) | EB − freq, hit (pp) | EB − freq, log-loss |
|---|---|---|---|---|---|---|
| tier S | 1342 | 26.0 [23.9, 28.3] | 44.6 [42.2, 47.0] | 42.8 [40.1, 45.5] | -1.9 [-3.9, +0.4] | +0.030 [-0.002, +0.057] |
| tier A | 1874 | 26.6 [24.8, 28.6] | 45.6 [42.5, 48.7] | 43.8 [40.5, 46.9] | -1.8 [-3.7, +0.2] | -0.020 [-0.051, +0.010] |
| acting team: < 5 earlier vetoes of this format | 32 | 21.9 [0.0, 38.1] | 59.4 [40.0, 75.0] | 59.4 [40.0, 75.0] | +0.0 [+0.0, +0.0] | +0.013 [-0.002, +0.028] |
| 5-14 | 87 | 32.2 [25.0, 42.9] | 56.3 [43.9, 66.1] | 56.3 [48.7, 62.6] | +0.0 [-6.7, +7.5] | +0.004 [-0.053, +0.049] |
| 15-39 | 354 | 26.3 [20.7, 31.8] | 50.6 [45.0, 56.8] | 48.9 [42.9, 54.7] | -1.7 [-6.5, +2.1] | +0.009 [-0.030, +0.058] |
| ≥ 40 | 2743 | 26.2 [24.8, 27.7] | 44.0 [41.9, 46.0] | 42.0 [40.0, 44.2] | -1.9 [-3.4, -0.3] | -0.001 [-0.028, +0.022] |
| within 28 days after a true pool change | 412 | 26.2 [21.6, 30.7] | 42.7 [35.7, 50.2] | 43.0 [36.1, 52.2] | +0.2 [-1.9, +3.1] | +0.005 [-0.062, +0.048] |
| other times | 2804 | 26.4 [24.9, 28.0] | 45.5 [43.5, 47.6] | 43.4 [41.3, 45.4] | -2.1 [-3.8, -0.5] | -0.000 [-0.025, +0.024] |
| detected pool = actual seven maps | 3094 | 26.6 [25.0, 28.1] | 45.6 [43.5, 47.7] | 43.7 [41.5, 46.1] | -1.8 [-3.3, -0.3] | -0.001 [-0.024, +0.021] |
| detected pool ≠ actual | 122 | 21.3 [19.0, 22.2] | 35.2 [30.9, 36.9] | 33.6 [32.4, 34.8] | -1.6 [-3.2, +3.4] | +0.039 [-0.054, +0.068] |

**Bo3 bans (steps 1, 2, 5, 6)** by subset (next-step mode; Matches 1608)

| Subset | Steps | uniform hit (%) | team freq hit (%) | EB hit (%) | EB − freq, hit (pp) | EB − freq, log-loss |
|---|---|---|---|---|---|---|
| tier S | 2684 | 27.5 [24.8, 30.2] | 68.0 [65.5, 70.3] | 67.8 [64.4, 71.1] | -0.1 [-2.0, +1.6] | +0.152 [+0.123, +0.183] |
| tier A | 3748 | 26.4 [25.0, 27.9] | 69.0 [67.3, 70.6] | 65.9 [63.7, 68.0] | -3.1 [-4.9, -1.4] | +0.203 [+0.180, +0.225] |
| acting team: < 5 earlier vetoes of this format | 64 | 26.6 [11.7, 40.6] | 62.5 [52.1, 70.0] | 57.8 [45.0, 67.5] | -4.7 [-15.6, +5.7] | +0.080 [-0.017, +0.161] |
| 5-14 | 174 | 28.7 [23.1, 35.4] | 70.7 [66.5, 74.2] | 70.1 [60.3, 75.5] | -0.6 [-9.0, +4.2] | +0.163 [+0.096, +0.285] |
| 15-39 | 708 | 33.2 [29.4, 37.7] | 69.6 [66.4, 72.3] | 69.5 [66.5, 72.3] | -0.1 [-2.0, +1.9] | +0.198 [+0.160, +0.240] |
| ≥ 40 | 5486 | 26.0 [24.6, 27.5] | 68.4 [66.8, 70.0] | 66.3 [64.3, 68.4] | -2.1 [-3.6, -0.6] | +0.181 [+0.161, +0.203] |
| within 28 days after a true pool change | 824 | 21.4 [16.7, 26.1] | 59.6 [53.8, 64.4] | 56.3 [52.9, 59.5] | -3.3 [-6.5, +1.7] | +0.228 [+0.171, +0.286] |
| other times | 5608 | 27.7 [26.5, 29.0] | 69.9 [68.6, 71.1] | 68.2 [66.4, 70.0] | -1.7 [-3.0, -0.4] | +0.175 [+0.155, +0.194] |
| detected pool = actual seven maps | 6188 | 27.5 [26.3, 28.7] | 69.3 [68.0, 70.5] | 67.4 [65.7, 69.2] | -1.8 [-3.2, -0.5] | +0.183 [+0.162, +0.204] |
| detected pool ≠ actual | 244 | 11.5 [8.0, 22.4] | 50.4 [48.2, 52.9] | 47.5 [44.6, 52.2] | -2.9 [-3.7, -0.7] | +0.157 [+0.137, +0.209] |

**Bo1 bans (steps 1-6)** by subset (next-step mode; Matches 255)

| Subset | Steps | uniform hit (%) | team freq hit (%) | EB hit (%) | EB − freq, hit (pp) | EB − freq, log-loss |
|---|---|---|---|---|---|---|
| tier S | 1314 | 27.3 [25.1, 29.3] | 45.5 [43.0, 48.5] | 54.6 [52.9, 56.5] | +9.1 [+7.2, +10.8] | -0.083 [-0.108, -0.049] |
| tier A | 216 | 24.5 [19.9, 33.3] | 46.3 [41.7, 56.4] | 52.8 [47.8, 63.0] | +6.5 [+2.4, +12.5] | -0.055 [-0.126, +0.009] |
| acting team: < 5 earlier vetoes of this format | 111 | 27.9 [22.2, 35.6] | 41.4 [34.3, 49.5] | 44.1 [37.6, 52.7] | +2.7 [-2.0, +8.3] | +0.012 [-0.042, +0.083] |
| 5-14 | 519 | 27.2 [21.2, 31.7] | 46.4 [41.0, 48.8] | 54.3 [51.9, 58.5] | +7.9 [+4.7, +15.4] | -0.065 [-0.108, -0.022] |
| 15-39 | 852 | 26.3 [24.7, 28.1] | 45.2 [41.5, 49.9] | 55.2 [51.4, 58.5] | +10.0 [+6.3, +12.5] | -0.091 [-0.112, -0.059] |
| ≥ 40 | 48 | 33.3 [30.3, 41.7] | 54.2 [46.2, 66.7] | 62.5 [50.0, 80.0] | +8.3 [-13.3, +22.2] | -0.223 [-0.395, -0.178] |
| within 28 days after a true pool change | 108 | 18.5 [18.5, 18.5] | 44.4 [44.4, 44.4] | 47.2 [47.2, 47.2] | +2.8 [+2.8, +2.8] | -0.041 [-0.041, -0.041] |
| other times | 1422 | 27.6 [25.6, 29.6] | 45.7 [43.2, 48.6] | 54.9 [53.1, 56.7] | +9.1 [+7.4, +10.9] | -0.082 [-0.107, -0.048] |
| detected pool = actual seven maps | 1422 | 27.6 [25.6, 29.6] | 45.7 [43.2, 48.6] | 54.9 [53.1, 56.7] | +9.1 [+7.4, +10.9] | -0.082 [-0.107, -0.048] |
| detected pool ≠ actual | 108 | 18.5 [18.5, 18.5] | 44.4 [44.4, 44.4] | 47.2 [47.2, 47.2] | +2.8 [+2.8, +2.8] | -0.041 [-0.041, -0.041] |

## Pool and sensitivity (exploratory)

**Active-pool detector** (`ActivePool`: the seven maps most frequent in the 7-step vetoes of the last 14 days, refreshed at most every 7 days)

The detected pool equals the seven maps of the Match's own veto in 95.9% of the 1915 scored Matches (79 do not match); within 28 days after a true pool change (below) 65.8% of 231, otherwise 100.0%. The lab's pool (the seven maps of the latest earlier veto of that format) equals it in 99.3%.

True pool changes, read from the scored Matches' own vetoes (the Event whose most common seven-map pool differs from the previous Event's), against the detector:

| First Match of the new pool | Event | Map in | Map out | Detector saw it on | Lag (days) | Scored Matches played with a wrong detected pool | Matches since the change until the detector caught up |
|---|---|---|---|---|---|---|---|
| 2025-01-23 | 8943 | Train | Vertigo | 2025-01-17 | -6.2 | 0 | 0 |
| 2025-07-23 | 9495 | Overpass | Anubis | 2025-07-29 | 6.2 | 40 | 40 |
| 2026-01-14 | 10034 | Anubis | Train | 2026-01-20 | 5.9 | 8 | 8 |
| 2026-07-01 | 10685 | Cache | Overpass | 2026-07-09 | 8.2 | 31 | 31 |

Detector changes (all Events of the archive, not only the scored ones), with the share of 7-step vetoes containing the map in the 28 days before and after:

| Detected on | Map in | Map out | Share with the map in, before → after | Share with the map out, before → after |
|---|---|---|---|---|
| 2024-06-05 | Dust2 | Overpass | 79% → 100% | 21% → 0% |
| 2025-01-17 | Train | Vertigo | 100% → 100% | 0% → 0% |
| 2025-07-29 | Overpass | Anubis | 41% → 100% | 59% → 0% |
| 2026-01-20 | Anubis | Train | 88% → 92% | 12% → 8% |
| 2026-07-09 | Cache | Overpass | 36% → 100% | 64% → 0% |

**Sensitivity: the detected pool vs the actual seven maps** (Bo3, scored; top-1 hit (%) and log-loss, next-step mode). With the oracle pool the candidates are the seven maps of the Match's own veto (not a real forecast: it reads the Match's pool).

| Method | Pool | First bans: hit | Picks: hit | Second bans: hit | First bans: log-loss | Picks: log-loss | Second bans: log-loss |
|---|---|---|---|---|---|---|---|
| uniform | detected | 16.0 [14.3, 17.7] | 26.4 [24.9, 27.8] | 37.8 [35.7, 39.8] | 1.891 [1.869, 1.923] | 1.517 [1.499, 1.540] | 0.963 [0.901, 1.042] |
| uniform | actual | 16.0 [14.3, 17.7] | 26.4 [24.9, 27.8] | 38.8 [37.4, 40.3] | 1.869 [1.869, 1.869] | 1.498 [1.498, 1.498] | 0.896 [0.896, 0.896] |
| team freq | detected | 83.3 [81.7, 84.9] | 45.2 [43.1, 47.3] | 53.8 [52.0, 55.5] | 0.645 [0.597, 0.702] | 1.329 [1.288, 1.370] | 0.997 [0.928, 1.083] |
| team freq | actual | 83.8 [82.3, 85.3] | 45.6 [43.5, 47.6] | 54.5 [53.0, 56.0] | 0.620 [0.586, 0.652] | 1.310 [1.274, 1.347] | 0.935 [0.909, 0.962] |
| EB sim | detected | 77.8 [75.6, 80.0] | 43.3 [41.1, 45.7] | 55.6 [53.3, 57.8] | 1.071 [1.025, 1.121] | 1.329 [1.287, 1.373] | 0.935 [0.861, 1.023] |
| EB sim | actual | 78.0 [75.9, 80.1] | 43.7 [41.6, 46.0] | 56.4 [54.4, 58.4] | 1.052 [1.018, 1.086] | 1.313 [1.275, 1.351] | 0.875 [0.843, 0.906] |
| lab sim | detected | 67.8 [65.3, 70.2] | 40.2 [38.5, 42.0] | 49.8 [48.1, 51.6] | 1.214 [1.179, 1.248] | 1.367 [1.340, 1.394] | 0.948 [0.916, 0.983] |
| lab sim | actual | 67.9 [65.4, 70.3] | 40.2 [38.5, 42.0] | 49.9 [48.1, 51.6] | 1.203 [1.170, 1.234] | 1.365 [1.338, 1.391] | 0.941 [0.910, 0.975] |

With the oracle pool, the primary difference (EB − freq, Bo3 picks, top-1) is -1.8 pp [-3.3, -0.3] (exploratory).

**Extended archive, not scored (exploratory)**: the same Bo3 comparison on Matches outside the scored set (tier B/C, older S/A, qualifiers; all before the cutoff of 2026-11-25, none in the Holdout). Their vetoes also feed the histories of the scored set.

| Block | Matches | uniform picks hit (%) | team freq picks hit (%) | EB picks hit (%) | EB − freq, picks (pp) | team freq first-ban hit (%) | EB first-ban hit (%) |
|---|---|---|---|---|---|---|---|
| sa_old | 410 | 21.5 [18.1, 25.8] | 43.4 [39.8, 46.9] | 43.5 [40.4, 46.4] | +0.1 [-1.5, +1.4] | 79.5 [76.7, 81.5] | 77.3 [74.5, 79.3] |
| b | 206 | 24.8 [19.8, 29.4] | 43.7 [40.3, 48.0] | 43.2 [38.9, 48.3] | -0.5 [-3.6, +2.7] | 81.8 [77.6, 85.8] | 77.9 [74.1, 82.1] |
| c | 5223 | 25.8 [25.0, 26.6] | 46.8 [45.8, 47.8] | 46.1 [45.0, 47.2] | -0.7 [-1.4, -0.0] | 73.5 [72.1, 74.9] | 66.8 [65.2, 68.2] |
| qualifier | 761 | 26.1 [23.7, 28.4] | 43.6 [40.8, 46.5] | 42.0 [39.1, 45.2] | -1.5 [-3.4, +0.4] | 72.5 [69.1, 76.2] | 66.3 [62.7, 70.1] |

