# Catch-up mode

Archive SHA-256 `beabb356eda6798dc620e39797cf5ee1e47a9b7ebbae11565ef04eaa56de54e4`. Seed 33. Each of 2000 paired trials draws one of 62 archived Events uniformly and replays all its scored Matches. Of 2288 Matches, 2243 have an anchor and 766 have a market line. An unavailable input means a missed pick. Favourite followers independently reverse the anchor favourite with the stated probability. Market followers always take the market favourite. Exact ties pick team A.

Both bots face identical opponents. The variant reverses a favourite of over 50% through 60% whenever it trails. Only results published strictly before the cutoff enter standings. Missing end times enter final scoring only. Default Stage weights award one Match Point per correct pick. Equal points share place, including first.

Brackets are pointwise 90% Hoeffding intervals for the simulated means conditional on these Events and opponent models. They measure Monte Carlo uncertainty only.

## Favourite follower (10% noise)

| Members | Policy | First place probability | Expected place |
|---:|---|---:|---:|
| 4 | plain | 0.571 [0.543, 0.598] | 1.536 [1.453, 1.618] |
| 4 | catch-up | 0.659 [0.632, 0.687] | 1.534 [1.451, 1.616] |
| 12 | plain | 0.180 [0.153, 0.208] | 2.903 [2.602, 3.204] |
| 12 | catch-up | 0.397 [0.370, 0.424] | 3.296 [2.994, 3.597] |

### Verdict

Conditions at the first changed pick. Matches left includes the current cutoff. A higher chance of first and a lower expected place favour catch-up. Trials without a changed pick have identical finishes.

| Members | points behind | Matches left | Trials | Difference in first-place probability | Difference in expected place | Chance of first |
|---:|---:|---:|---:|---:|---:|---|
| 4 | 1 | 1-10 | 178 | 0.169 [-0.015, 0.352] | 0.062 [-0.489, 0.612] | inconclusive |
| 4 | 1 | 11+ | 775 | 0.164 [0.076, 0.252] | -0.022 [-0.286, 0.242] | higher |
| 4 | 2+ | 1-10 | 18 | 0.000 [-0.577, 0.577] | 0.556 [-1.175, 2.286] | inconclusive |
| 4 | 2+ | 11+ | 111 | 0.189 [-0.043, 0.422] | -0.072 [-0.769, 0.625] | inconclusive |
| 12 | 1 | 1-10 | 206 | 0.257 [0.087, 0.428] | 0.636 [-1.240, 2.512] | higher |
| 12 | 1 | 11+ | 1099 | 0.298 [0.225, 0.372] | 0.495 [-0.317, 1.307] | higher |
| 12 | 2+ | 1-10 | 27 | 0.037 [-0.434, 0.508] | 1.481 [-3.700, 6.663] | inconclusive |
| 12 | 2+ | 11+ | 241 | 0.212 [0.054, 0.369] | 0.290 [-1.444, 2.025] | higher |


## Favourite follower (25% noise)

| Members | Policy | First place probability | Expected place |
|---:|---|---:|---:|
| 4 | plain | 0.684 [0.657, 0.711] | 1.387 [1.305, 1.470] |
| 4 | catch-up | 0.704 [0.677, 0.731] | 1.407 [1.325, 1.489] |
| 12 | plain | 0.386 [0.358, 0.413] | 2.344 [2.042, 2.645] |
| 12 | catch-up | 0.451 [0.423, 0.478] | 2.554 [2.253, 2.856] |

### Verdict

Conditions at the first changed pick. Matches left includes the current cutoff. A higher chance of first and a lower expected place favour catch-up. Trials without a changed pick have identical finishes.

| Members | points behind | Matches left | Trials | Difference in first-place probability | Difference in expected place | Chance of first |
|---:|---:|---:|---:|---:|---:|---|
| 4 | 1 | 1-10 | 139 | -0.022 [-0.229, 0.186] | 0.108 [-0.515, 0.731] | inconclusive |
| 4 | 1 | 11+ | 681 | 0.037 [-0.057, 0.131] | 0.046 [-0.236, 0.327] | inconclusive |
| 4 | 2+ | 1-10 | 25 | -0.120 [-0.610, 0.370] | 0.120 [-1.349, 1.589] | inconclusive |
| 4 | 2+ | 11+ | 214 | 0.098 [-0.069, 0.265] | -0.047 [-0.549, 0.455] | inconclusive |
| 12 | 1 | 1-10 | 187 | 0.096 [-0.083, 0.275] | 0.230 [-1.739, 2.199] | inconclusive |
| 12 | 1 | 11+ | 847 | 0.090 [0.006, 0.174] | 0.242 [-0.683, 1.167] | higher |
| 12 | 2+ | 1-10 | 32 | -0.094 [-0.526, 0.339] | 0.844 [-3.916, 5.604] | inconclusive |
| 12 | 2+ | 11+ | 434 | 0.090 [-0.028, 0.207] | 0.339 [-0.954, 1.631] | inconclusive |


## Market only

| Members | Policy | First place probability | Expected place |
|---:|---|---:|---:|
| 4 | plain | 0.931 [0.904, 0.959] | 1.206 [1.123, 1.288] |
| 4 | catch-up | 0.965 [0.937, 0.992] | 1.107 [1.024, 1.189] |
| 12 | plain | 0.931 [0.904, 0.959] | 1.754 [1.452, 2.055] |
| 12 | catch-up | 0.965 [0.937, 0.992] | 1.391 [1.089, 1.692] |

### Verdict

Conditions at the first changed pick. Matches left includes the current cutoff. A higher chance of first and a lower expected place favour catch-up. Trials without a changed pick have identical finishes.

| Members | points behind | Matches left | Trials | Difference in first-place probability | Difference in expected place | Chance of first |
|---:|---:|---:|---:|---:|---:|---|
| 4 | 1 | 1-10 | 33 | 0.000 [-0.426, 0.426] | 0.000 [-1.278, 1.278] | inconclusive |
| 4 | 1 | 11+ | 181 | 0.365 [0.183, 0.547] | -1.094 [-1.640, -0.548] | higher |
| 12 | 1 | 1-10 | 33 | 0.000 [-0.426, 0.426] | 0.000 [-4.687, 4.687] | inconclusive |
| 12 | 1 | 11+ | 181 | 0.365 [0.183, 0.547] | -4.011 [-6.012, -2.010] | higher |

