# Catch-up mode

Archive SHA-256 `cd61333be4709fc8cab6b1951f7b01cf253c15f616e80fa58423ac75239455a7`. Seed 33. Each of 2000 paired trials draws one of 62 archived training-side Events uniformly (the Holdout is not drawn) and replays all its scored Matches. Of 2288 Matches, 2243 have an anchor and 777 have a market line. An unavailable input means a missed pick. Favourite followers independently reverse the anchor favourite with the stated probability. Market followers always take the market favourite. Exact ties pick team A.

Both bots face identical opponents. The variant reverses a favourite of over 50% through 60% whenever it trails. Only results published strictly before the cutoff enter standings. Missing end times enter final scoring only. Default Stage weights award one Match Point per correct pick. Equal points share place, including first.

Brackets are pointwise 90% Hoeffding intervals for the simulated means conditional on these Events and opponent models. They measure Monte Carlo uncertainty only.

## Favourite follower (10% noise)

| Members | Policy | First place probability | Expected place |
|---:|---|---:|---:|
| 4 | plain | 0.577 [0.549, 0.604] | 1.527 [1.445, 1.609] |
| 4 | catch-up | 0.650 [0.623, 0.677] | 1.562 [1.480, 1.644] |
| 12 | plain | 0.192 [0.165, 0.219] | 2.867 [2.566, 3.168] |
| 12 | catch-up | 0.394 [0.367, 0.421] | 3.426 [3.125, 3.728] |

### Verdict

Conditions at the first changed pick. Matches left includes the current cutoff. A higher chance of first and a lower expected place favour catch-up. Trials without a changed pick have identical finishes.

| Members | points behind | Matches left | Trials | Difference in first-place probability | Difference in expected place | Chance of first |
|---:|---:|---:|---:|---:|---:|---|
| 4 | 1 | 1-10 | 193 | 0.073 [-0.104, 0.249] | 0.212 [-0.316, 0.741] | inconclusive |
| 4 | 1 | 11+ | 768 | 0.151 [0.063, 0.239] | 0.025 [-0.240, 0.290] | higher |
| 4 | 2+ | 1-10 | 19 | 0.000 [-0.562, 0.562] | 0.474 [-1.211, 2.158] | inconclusive |
| 4 | 2+ | 11+ | 111 | 0.153 [-0.079, 0.385] | 0.009 [-0.688, 0.706] | inconclusive |
| 12 | 1 | 1-10 | 217 | 0.194 [0.027, 0.360] | 1.373 [-0.455, 3.201] | higher |
| 12 | 1 | 11+ | 1097 | 0.287 [0.213, 0.361] | 0.605 [-0.208, 1.418] | higher |
| 12 | 2+ | 1-10 | 27 | 0.074 [-0.397, 0.545] | 1.444 [-3.737, 6.626] | inconclusive |
| 12 | 2+ | 11+ | 243 | 0.185 [0.028, 0.342] | 0.486 [-1.242, 2.213] | higher |


## Favourite follower (25% noise)

| Members | Policy | First place probability | Expected place |
|---:|---|---:|---:|
| 4 | plain | 0.691 [0.664, 0.718] | 1.372 [1.290, 1.454] |
| 4 | catch-up | 0.699 [0.671, 0.726] | 1.423 [1.340, 1.505] |
| 12 | plain | 0.395 [0.367, 0.422] | 2.305 [2.004, 2.606] |
| 12 | catch-up | 0.450 [0.422, 0.477] | 2.637 [2.336, 2.938] |

### Verdict

Conditions at the first changed pick. Matches left includes the current cutoff. A higher chance of first and a lower expected place favour catch-up. Trials without a changed pick have identical finishes.

| Members | points behind | Matches left | Trials | Difference in first-place probability | Difference in expected place | Chance of first |
|---:|---:|---:|---:|---:|---:|---|
| 4 | 1 | 1-10 | 149 | -0.087 [-0.288, 0.113] | 0.262 [-0.340, 0.863] | inconclusive |
| 4 | 1 | 11+ | 674 | 0.030 [-0.065, 0.124] | 0.056 [-0.226, 0.339] | inconclusive |
| 4 | 2+ | 1-10 | 26 | -0.154 [-0.634, 0.326] | 0.115 [-1.325, 1.556] | inconclusive |
| 4 | 2+ | 11+ | 212 | 0.057 [-0.112, 0.225] | 0.099 [-0.405, 0.603] | inconclusive |
| 12 | 1 | 1-10 | 208 | 0.038 [-0.131, 0.208] | 0.803 [-1.064, 2.670] | inconclusive |
| 12 | 1 | 11+ | 843 | 0.088 [0.003, 0.172] | 0.251 [-0.676, 1.179] | higher |
| 12 | 2+ | 1-10 | 32 | -0.062 [-0.495, 0.370] | 0.656 [-4.104, 5.416] | inconclusive |
| 12 | 2+ | 11+ | 438 | 0.068 [-0.048, 0.185] | 0.603 [-0.684, 1.889] | inconclusive |


## Market only

| Members | Policy | First place probability | Expected place |
|---:|---|---:|---:|
| 4 | plain | 0.963 [0.936, 0.990] | 1.111 [1.029, 1.193] |
| 4 | catch-up | 0.984 [0.956, 1.000] | 1.050 [1.000, 1.132] |
| 12 | plain | 0.963 [0.936, 0.990] | 1.407 [1.106, 1.708] |
| 12 | catch-up | 0.984 [0.956, 1.000] | 1.181 [1.000, 1.483] |

### Verdict

Conditions at the first changed pick. Matches left includes the current cutoff. A higher chance of first and a lower expected place favour catch-up. Trials without a changed pick have identical finishes.

| Members | points behind | Matches left | Trials | Difference in first-place probability | Difference in expected place | Chance of first |
|---:|---:|---:|---:|---:|---:|---|
| 4 | 1 | 1-10 | 33 | 0.000 [-0.426, 0.426] | 0.000 [-1.278, 1.278] | inconclusive |
| 4 | 1 | 11+ | 143 | 0.287 [0.082, 0.491] | -0.860 [-1.474, -0.246] | higher |
| 12 | 1 | 1-10 | 33 | 0.000 [-0.426, 0.426] | 0.000 [-4.687, 4.687] | inconclusive |
| 12 | 1 | 11+ | 143 | 0.287 [0.082, 0.491] | -3.154 [-5.405, -0.902] | higher |

