# Catch-up mode

Archive SHA-256 `25459d150e1fac87409e38d17a73ab2110444401703a056b1c4d4c846c4df782`. Seed 33. Each of 2000 paired trials draws one of 6 archived Events uniformly and replays all its scored Matches. Of 488 Matches, 415 have an anchor and 112 have a market line. An unavailable input means a missed pick. Favourite followers independently reverse the anchor favourite with the stated probability. Market followers always take the market favourite. Exact ties pick team A.

Both bots face identical opponents. The variant reverses a favourite of over 50% through 60% whenever it trails. Only results published strictly before the cutoff enter standings. Missing end times enter final scoring only. Default Stage weights award one Match Point per correct pick. Equal points share place, including first.

Brackets are pointwise 90% Hoeffding intervals for the simulated means conditional on these Events and opponent models. They measure Monte Carlo uncertainty only.

## Favourite follower (10% noise)

| Members | Policy | First place probability | Expected place |
|---:|---|---:|---:|
| 4 | plain | 0.612 [0.584, 0.639] | 1.495 [1.413, 1.577] |
| 4 | catch-up | 0.735 [0.707, 0.762] | 1.383 [1.301, 1.466] |
| 12 | plain | 0.304 [0.277, 0.332] | 2.808 [2.506, 3.109] |
| 12 | catch-up | 0.584 [0.557, 0.611] | 2.628 [2.327, 2.929] |

### Verdict

Conditions at the first changed pick. Matches left includes the current cutoff. A higher chance of first and a lower expected place favour catch-up. Trials without a changed pick have identical finishes.

| Members | points behind | Matches left | Trials | Difference in first-place probability | Difference in expected place | Chance of first |
|---:|---:|---:|---:|---:|---:|---|
| 4 | 1 | 1-10 | 67 | 0.567 [0.268, 0.866] | -0.746 [-1.643, 0.151] | higher |
| 4 | 1 | 11+ | 1206 | 0.135 [0.065, 0.206] | -0.095 [-0.306, 0.117] | higher |
| 4 | 2+ | 1-10 | 1 | 1.000 [-1.000, 1.000] | -1.000 [-3.000, 3.000] | inconclusive |
| 4 | 2+ | 11+ | 162 | 0.272 [0.079, 0.464] | -0.358 [-0.935, 0.219] | higher |
| 12 | 1 | 1-10 | 6 | 0.833 [-0.166, 1.000] | -2.000 [-11.000, 8.992] | inconclusive |
| 12 | 1 | 11+ | 1359 | 0.335 [0.268, 0.401] | -0.091 [-0.822, 0.639] | higher |
| 12 | 2+ | 11+ | 276 | 0.359 [0.211, 0.506] | -0.808 [-2.429, 0.813] | higher |


## Favourite follower (25% noise)

| Members | Policy | First place probability | Expected place |
|---:|---|---:|---:|
| 4 | plain | 0.752 [0.725, 0.779] | 1.326 [1.244, 1.408] |
| 4 | catch-up | 0.793 [0.766, 0.820] | 1.328 [1.246, 1.410] |
| 12 | plain | 0.510 [0.483, 0.537] | 2.216 [1.915, 2.518] |
| 12 | catch-up | 0.657 [0.630, 0.684] | 2.308 [2.007, 2.610] |

### Verdict

Conditions at the first changed pick. Matches left includes the current cutoff. A higher chance of first and a lower expected place favour catch-up. Trials without a changed pick have identical finishes.

| Members | points behind | Matches left | Trials | Difference in first-place probability | Difference in expected place | Chance of first |
|---:|---:|---:|---:|---:|---:|---|
| 4 | 1 | 1-10 | 19 | 0.263 [-0.298, 0.825] | -0.632 [-2.316, 1.053] | inconclusive |
| 4 | 1 | 11+ | 1134 | 0.036 [-0.037, 0.109] | 0.033 [-0.185, 0.251] | inconclusive |
| 4 | 2+ | 1-10 | 2 | 0.500 [-1.000, 1.000] | -0.500 [-3.000, 3.000] | inconclusive |
| 4 | 2+ | 11+ | 357 | 0.098 [-0.032, 0.228] | -0.056 [-0.445, 0.333] | inconclusive |
| 12 | 1 | 11+ | 877 | 0.186 [0.103, 0.269] | 0.084 [-0.825, 0.994] | higher |
| 12 | 2+ | 11+ | 767 | 0.171 [0.082, 0.259] | 0.143 [-0.829, 1.116] | higher |


## Market only

| Members | Policy | First place probability | Expected place |
|---:|---|---:|---:|
| 4 | plain | 1.000 [0.973, 1.000] | 1.000 [1.000, 1.082] |
| 4 | catch-up | 1.000 [0.973, 1.000] | 1.000 [1.000, 1.082] |
| 12 | plain | 1.000 [0.973, 1.000] | 1.000 [1.000, 1.301] |
| 12 | catch-up | 1.000 [0.973, 1.000] | 1.000 [1.000, 1.301] |

### Verdict

Conditions at the first changed pick. Matches left includes the current cutoff. A higher chance of first and a lower expected place favour catch-up. Trials without a changed pick have identical finishes.

| Members | points behind | Matches left | Trials | Difference in first-place probability | Difference in expected place | Chance of first |
|---:|---:|---:|---:|---:|---:|---|
| 4 | — | — | 2000 | 0 | 0 | identical in every trial (no changed pick) |
| 12 | — | — | 2000 | 0 | 0 | identical in every trial (no changed pick) |

