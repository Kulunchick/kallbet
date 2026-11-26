# Signal Lab report

Archive SHA-256 `cd61333be4709fc8cab6b1951f7b01cf253c15f616e80fa58423ac75239455a7` (13433 files), protocol `signal-lab/3`, seed 33, 2000 Event resamples, 90% intervals, floor 100 paired Matches.

Baseline: the anchor: HybridAnchored without LLM, 0.80 market and 0.20 Valve rating logit (`debate::base`). Each Candidate Signal is fitted, for each Event, only on the Matches of the Events that ended before it began, and every datum it reads is observed before the Match cutoff. Differences are variant − anchor on the Matches both predicted (log-loss and Brier: below zero is better). The interval resamples whole Events. Verdict: `promote` when the interval of the log-loss difference is below zero, the Matches with a market line are not worse and the interval of the accuracy difference (same Matches and resamples) is not wholly below zero, since KallBet scores only the picked winner (else `no evidence`: better probabilities but fewer winners picked); `harmful` when it is above zero; `no evidence` otherwise or under the floor; `not testable` when the data could not be had. "Detectable" is the smallest log-loss difference these Matches would show with 80% probability: an effect smaller than it can hide as `no evidence`.

| Candidate Signal | Verdict | Computed on | Paired | Events | Δ log-loss | 90% interval | Detectable | Δ Brier | Δ accuracy | Δ accuracy 90% interval | Market subset (n, Δ log-loss) |
|---|---|---|---|---|---|---|---|---|---|---|---|
| `rating_calibration` | promote | 2193 | 2193 | 60 | -0.1759 | [-0.2403, -0.1217] | 0.0887 | -0.0250 | -0.0009 | [-0.0037, +0.0022] | 776, -0.0158 |
| `learned_weights` | promote | 2243 | 2243 | 61 | -0.1704 | [-0.2297, -0.1148] | 0.0872 | -0.0241 | -0.0013 | [-0.0044, +0.0020] | 777, -0.0119 |
| `market_source_pinnacle` | no evidence | 28 | 28 | 1 | +0.0000 | [+0.0000, +0.0000] | 0.0000 | +0.0000 | +0.0000 | [+0.0000, +0.0000] | 28, +0.0000 |
| `market_source_polymarket` | no evidence | 28 | 28 | 1 | +0.0230 | [+0.0230, +0.0230] | 0.0000 | +0.0073 | +0.0000 | [+0.0000, +0.0000] | 28, +0.0230 |
| `market_source_kalshi` | no evidence | 28 | 28 | 1 | +0.0160 | [+0.0160, +0.0160] | 0.0000 | +0.0052 | +0.0000 | [+0.0000, +0.0000] | 28, +0.0160 |
| `market_source_average` | no evidence | 28 | 28 | 1 | +0.0124 | [+0.0124, +0.0124] | 0.0000 | +0.0041 | +0.0000 | [+0.0000, +0.0000] | 28, +0.0124 |
| `line_movement` | no evidence | 744 | 744 | 20 | +0.0008 | [-0.0035, +0.0066] | 0.0076 | +0.0000 | +0.0013 | [-0.0079, +0.0102] | 744, +0.0008 |
| `favourite_longshot` | no evidence | 746 | 746 | 20 | +0.0008 | [-0.0003, +0.0021] | 0.0019 | +0.0001 | -0.0027 | [-0.0063, +0.0000] | 746, +0.0008 |
| `own_rating` | promote | 2199 | 2168 | 61 | -0.1677 | [-0.2329, -0.1079] | 0.0929 | -0.0216 | -0.0120 | [-0.0260, +0.0005] | 758, -0.0127 |
| `recent_form` | no evidence | 1980 | 1980 | 60 | -0.0009 | [-0.0053, +0.0036] | 0.0068 | +0.0014 | -0.0030 | [-0.0098, +0.0040] | 697, +0.0058 |
| `head_to_head` | no evidence | 1280 | 1280 | 57 | -0.0038 | [-0.0127, +0.0047] | 0.0131 | +0.0008 | -0.0086 | [-0.0220, +0.0042] | 561, +0.0065 |
| `llm_adjustment` | not testable | 0 | 0 | 0 | — | — | — | — | — | — | — |
| `lineup_strength` | promote | 1950 | 1950 | 60 | -0.0026 | [-0.0045, -0.0008] | 0.0028 | +0.0003 | -0.0041 | [-0.0093, +0.0006] | 641, -0.0023 |
| `player_form` | no evidence | 1567 | 1567 | 59 | +0.0000 | [-0.0002, +0.0003] | 0.0004 | +0.0000 | -0.0006 | [-0.0026, +0.0012] | 483, +0.0001 |
| `per_map_rating` | no evidence | 2117 | 2109 | 61 | -0.1654 | [-0.2335, -0.1035] | 0.0970 | -0.0186 | -0.0166 | [-0.0347, -0.0004] | 724, -0.0111 |
| `bo1_veto_sim` | promote | 323 | 319 | 13 | -0.0877 | [-0.1315, -0.0342] | 0.0738 | -0.0173 | +0.0094 | [-0.0088, +0.0280] | 54, -0.0025 |
| `bo3_veto_sim` | no evidence (added after list fixed) | 1882 | 1850 | 61 | -0.1700 | [-0.2491, -0.1005] | 0.1115 | -0.0156 | -0.0292 | [-0.0458, -0.0141] | 702, -0.0182 |
| `roster_stand_in` | no evidence | 2095 | 2056 | 61 | -0.0002 | [-0.0019, +0.0015] | 0.0025 | -0.0015 | +0.0034 | [+0.0011, +0.0060] | 651, -0.0010 |
| `roster_change_30d` | harmful | 1563 | 1563 | 60 | +0.0044 | [+0.0015, +0.0080] | 0.0048 | +0.0006 | +0.0000 | [-0.0020, +0.0021] | 504, +0.0023 |
| `roster_stability` | no evidence | 1534 | 1534 | 58 | +0.0045 | [-0.0006, +0.0098] | 0.0078 | +0.0016 | +0.0013 | [-0.0039, +0.0062] | 621, +0.0019 |
| `roster_coach_change` | harmful | 1932 | 1903 | 60 | +0.0041 | [+0.0001, +0.0094] | 0.0072 | +0.0010 | -0.0037 | [-0.0070, -0.0005] | 615, -0.0002 |
| `rest_schedule_load` | no evidence | 2128 | 2128 | 60 | -0.0000 | [-0.0043, +0.0040] | 0.0062 | +0.0004 | +0.0005 | [-0.0059, +0.0067] | 758, -0.0020 |
| `stakes` | no evidence | 630 | 630 | 51 | -0.0379 | [-0.0650, -0.0121] | 0.0397 | -0.0051 | +0.0000 | [+0.0000, +0.0000] | 216, +0.0177 |
| `lan_online` | no evidence | 2078 | 2078 | 59 | -0.0051 | [-0.0077, -0.0024] | 0.0041 | -0.0003 | -0.0053 | [-0.0093, -0.0015] | 756, -0.0017 |
| `series_format` | no evidence | 1710 | 1710 | 55 | +0.0003 | [-0.0010, +0.0015] | 0.0020 | +0.0004 | -0.0023 | [-0.0063, +0.0013] | 644, +0.0015 |
| `region_travel` | no evidence | 107 | 107 | 2 | +0.0010 | [+0.0000, +0.0010] | 0.0010 | +0.0003 | -0.0093 | [-0.0094, +0.0000] | 105, +0.0017 |

## `rating_calibration`: promote

Valve rating's map probability rescaled in logit space by a factor fitted per series format, then blended with the market as the anchor does.

Verdict: the interval of the log-loss improvement excludes zero and the Matches with a market line are not worse.

| On the 2193 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.8025 | 0.2428 | 65.6% |
| rating_calibration | 0.6266 | 0.2178 | 65.5% |

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| BLAST Premier Groups Fall 2024 | 0 | 0 | 0 | — | — |
| IEM Cologne 2024 | 26 | 0 | 0 | — | — |
| BLAST Premier Showdown Fall 2024 | 75 | 14 | 14 | -0.2780 | — |
| BetBoom Dacha Belgrade season 2 2024 | 75 | 26 | 26 | +0.0090 | — |
| ESL Pro League Season 20 2024 | 115 | 76 | 76 | -0.2724 | — |
| BLAST Premier Final Fall 2024 | 194 | 13 | 13 | -0.0293 | — |
| IEM Rio 2024 | 207 | 29 | 29 | -0.2889 | — |
| Thunderpick World Championship 2024 | 236 | 28 | 28 | -0.3230 | — |
| BLAST Premier World Final 2024 | 236 | 13 | 13 | -0.4464 | — |
| Perfect World Major Shanghai: American RMR 2024 | 277 | 34 | 34 | -0.0392 | — |
| Perfect World Major Shanghai: European RMR A 2024 | 311 | 30 | 30 | -0.1016 | — |
| Perfect World Major Shanghai: European RMR B 2024 | 346 | 34 | 34 | -0.4038 | — |
| Perfect World Major Shanghai Major 2024 | 380 | 73 | 73 | -0.0399 | — |
| BLAST Premier Bounty Closed Qualifier 2025 | 453 | 23 | 23 | -0.5210 | — |
| BLAST Premier Bounty season 1 2025 | 477 | 7 | 7 | -0.7785 | — |
| IEM Katowice 2025 | 484 | 47 | 47 | -0.2248 | — |
| PGL Cluj-Napoca 2025 | 533 | 38 | 38 | -0.0623 | — |
| ESL Pro League Season 21 2025 | 574 | 40 | 40 | -0.0560 | — |
| BLAST Open Lisbon Spring 2025 | 614 | 29 | 29 | -0.0699 | — |
| PGL Bucharest 2025 | 643 | 41 | 41 | -0.8738 | — |
| BLAST.tv Major Austin: European Regional Qualifier 2025 | 684 | 35 | 35 | -0.3927 | — |
| IEM Melbourne 2025 | 719 | 29 | 29 | -0.1630 | — |
| BLAST Rivals Spring 2025 | 748 | 13 | 13 | -0.1862 | — |
| PGL Astana 2025 | 761 | 41 | 41 | -0.9910 | — |
| IEM Dallas 2025 | 802 | 29 | 29 | +0.0091 | — |
| BLAST.tv Major Austin 2025 | 831 | 106 | 106 | -0.2478 | — |
| FISSURE PLAYGROUND Season 1 2025 | 937 | 27 | 27 | -0.6627 | — |
| IEM Cologne 2025 | 964 | 49 | 49 | +0.0466 | — |
| BLAST Bounty Fall 2025 | 1013 | 30 | 30 | +0.0459 | — |
| Esports World Cup 2025 | 1044 | 16 | 16 | -0.4705 | — |
| BLAST Open London Fall 2025 | 1060 | 29 | 29 | -0.4877 | — |
| FISSURE PLAYGROUND Season 2 2025 | 1089 | 40 | 40 | -0.2473 | — |
| ESL Pro League Season 22 2025 | 1129 | 73 | 73 | -0.7885 | — |
| CS Asia Championships 2025 | 1202 | 30 | 30 | -0.2174 | — |
| Thunderpick World Championship 2025 | 1202 | 14 | 14 | -0.1563 | — |
| PGL Masters Bucharest 2025 | 1246 | 41 | 41 | -0.2631 | — |
| IEM Chengdu 2025 | 1287 | 30 | 30 | -0.0498 | — |
| BLAST Rivals Fall 2025 | 1317 | 13 | 13 | -0.0882 | — |
| StarLadder Major Budapest 2025 | 1330 | 106 | 106 | -0.1473 | — |
| BLAST Bounty Winter 2026 | 1436 | 31 | 31 | -0.0482 | — |
| IEM Krakow 2026 | 1467 | 50 | 50 | -0.0169 | — |
| PGL Cluj-Napoca 2026 | 1517 | 41 | 41 | +0.0325 | — |
| ESL Pro League Season 23 2026 | 1558 | 74 | 74 | -0.0289 | — |
| BLAST Open Rotterdam Spring 2026 | 1632 | 29 | 29 | -0.0202 | — |
| PGL Bucharest 2026 | 1661 | 39 | 39 | +0.0050 | — |
| IEM Rio 2026 | 1700 | 30 | 30 | +0.0382 | — |
| BLAST Rivals Spring 2026 | 1730 | 13 | 13 | -0.0237 | — |
| PGL Astana 2026 | 1743 | 41 | 41 | +0.0034 | — |
| IEM Atlanta 2026 | 1743 | 30 | 30 | -0.0221 | — |
| CS Asia Championships 2026 | 1814 | 30 | 30 | -0.1658 | — |
| IEM Cologne Major 2026 | 1844 | 106 | 106 | +0.0097 | — |
| XSE Pro League 2026 | 1950 | 37 | 37 | -0.0361 | — |
| BLAST Bounty Summer 2026 | 1990 | 29 | 29 | -0.0773 | — |
| Esports World Cup 2026 | 2021 | 56 | 56 | +0.0103 | — |
| BLAST Open Porto Fall 2026 | 2077 | 29 | 29 | +0.0161 | — |
| FISSURE PLAYGROUND Season 3 2026 | 2106 | 29 | 29 | -0.0411 | — |
| StarLadder StarSeries Fall 2026 | 2135 | 14 | 14 | -0.0426 | — |
| ESL Pro League Season 24 2026 | 2149 | 41 | 41 | -0.0365 | — |
| Thunderpick World Championship 2026 | 2190 | 14 | 14 | +0.0017 | — |
| PGL Masters Bucharest 2026 | 2204 | 41 | 41 | -0.0857 | — |
| IEM Beijing 2026 | 2245 | 30 | 30 | -0.0475 | — |
| BLAST Rivals Fall 2026 | 2275 | 13 | 13 | +0.0094 | — |

## `learned_weights`: promote

the anchor's market and Valve rating weights (logit space) learned by the combiner on the Events that ended before, one set per availability of the two inputs, regularised towards 0.80 / 0.20.

Verdict: the interval of the log-loss improvement excludes zero and the Matches with a market line are not worse.

| On the 2243 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.8044 | 0.2428 | 65.7% |
| learned_weights | 0.6339 | 0.2187 | 65.5% |

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| BLAST Premier Groups Fall 2024 | 0 | 0 | 0 | — | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":0,"rating":1.0}}` |
| IEM Cologne 2024 | 26 | 49 | 49 | +0.0000 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":0,"rating":1.0}}` |
| BLAST Premier Showdown Fall 2024 | 75 | 14 | 14 | -0.2810 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":49,"rating":0.33103}}` |
| BetBoom Dacha Belgrade season 2 2024 | 75 | 26 | 26 | -0.0060 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":49,"rating":0.33103}}` |
| ESL Pro League Season 20 2024 | 115 | 76 | 76 | -0.2666 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":89,"rating":0.358327}}` |
| BLAST Premier Final Fall 2024 | 194 | 13 | 13 | -0.0373 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":165,"rating":0.308033}}` |
| IEM Rio 2024 | 207 | 29 | 29 | -0.2863 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":178,"rating":0.319101}}` |
| Thunderpick World Championship 2024 | 236 | 28 | 28 | -0.3160 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":207,"rating":0.302971}}` |
| BLAST Premier World Final 2024 | 236 | 13 | 13 | -0.4335 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":207,"rating":0.302971}}` |
| Perfect World Major Shanghai: American RMR 2024 | 277 | 34 | 34 | -0.0392 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":248,"rating":0.279275}}` |
| Perfect World Major Shanghai: European RMR A 2024 | 311 | 30 | 30 | -0.0895 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":282,"rating":0.294515}}` |
| Perfect World Major Shanghai: European RMR B 2024 | 346 | 34 | 34 | -0.4327 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":312,"rating":0.307622}}` |
| Perfect World Major Shanghai Major 2024 | 380 | 73 | 73 | -0.0387 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":346,"rating":0.28924}}` |
| BLAST Premier Bounty Closed Qualifier 2025 | 453 | 23 | 23 | -0.5138 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":419,"rating":0.309168}}` |
| BLAST Premier Bounty season 1 2025 | 477 | 7 | 7 | -0.7613 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":442,"rating":0.296132}}` |
| IEM Katowice 2025 | 484 | 47 | 47 | -0.2240 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":449,"rating":0.290083}}` |
| PGL Cluj-Napoca 2025 | 533 | 38 | 38 | -0.0659 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":496,"rating":0.285889}}` |
| ESL Pro League Season 21 2025 | 574 | 40 | 40 | -0.0616 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":534,"rating":0.293231}}` |
| BLAST Open Lisbon Spring 2025 | 614 | 29 | 29 | -0.0750 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":574,"rating":0.304732}}` |
| PGL Bucharest 2025 | 643 | 41 | 41 | -0.8501 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":603,"rating":0.310496}}` |
| BLAST.tv Major Austin: European Regional Qualifier 2025 | 684 | 35 | 35 | -0.4057 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":644,"rating":0.27408}}` |
| IEM Melbourne 2025 | 719 | 29 | 29 | -0.1671 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":679,"rating":0.261866}}` |
| BLAST Rivals Spring 2025 | 748 | 13 | 13 | -0.1940 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":708,"rating":0.266731}}` |
| PGL Astana 2025 | 761 | 41 | 41 | -0.9741 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":721,"rating":0.268609}}` |
| IEM Dallas 2025 | 802 | 29 | 29 | +0.0010 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":762,"rating":0.24419}}` |
| BLAST.tv Major Austin 2025 | 831 | 106 | 106 | -0.2511 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":791,"rating":0.252839}}` |
| FISSURE PLAYGROUND Season 1 2025 | 937 | 27 | 27 | -0.6597 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":897,"rating":0.248566}}` |
| IEM Cologne 2025 | 964 | 49 | 49 | +0.0459 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":924,"rating":0.240369}}` |
| BLAST Bounty Fall 2025 | 1013 | 30 | 30 | +0.0413 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":973,"rating":0.249021}}` |
| Esports World Cup 2025 | 1044 | 16 | 16 | -0.4698 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":1003,"rating":0.257753}}` |
| BLAST Open London Fall 2025 | 1060 | 29 | 29 | -0.4876 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":1019,"rating":0.255076}}` |
| FISSURE PLAYGROUND Season 2 2025 | 1089 | 40 | 40 | -0.2469 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":1048,"rating":0.251792}}` |
| ESL Pro League Season 22 2025 | 1129 | 73 | 73 | -0.7796 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":1088,"rating":0.250758}}` |
| CS Asia Championships 2025 | 1202 | 30 | 30 | -0.2166 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":1161,"rating":0.228173}}` |
| Thunderpick World Championship 2025 | 1202 | 14 | 14 | -0.1632 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":1161,"rating":0.228173}}` |
| PGL Masters Bucharest 2025 | 1246 | 41 | 41 | -0.2636 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":1205,"rating":0.228893}}` |
| IEM Chengdu 2025 | 1287 | 30 | 30 | -0.0493 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":1246,"rating":0.227123}}` |
| BLAST Rivals Fall 2025 | 1317 | 13 | 13 | -0.0856 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":1276,"rating":0.230338}}` |
| StarLadder Major Budapest 2025 | 1330 | 106 | 106 | -0.1478 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":1289,"rating":0.231593}}` |
| BLAST Bounty Winter 2026 | 1436 | 31 | 31 | +0.0000 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":1395,"rating":0.233835}}` |
| IEM Krakow 2026 | 1467 | 50 | 50 | -0.0136 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.748665,"matches":31,"rating":-0.004371},"rating":{"matches":1395,"rating":0.233835}}` |
| PGL Cluj-Napoca 2026 | 1517 | 41 | 41 | +0.0428 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.812926,"matches":81,"rating":0.005601},"rating":{"matches":1395,"rating":0.233835}}` |
| ESL Pro League Season 23 2026 | 1558 | 74 | 74 | -0.0244 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.833562,"matches":122,"rating":0.070797},"rating":{"matches":1395,"rating":0.233835}}` |
| BLAST Open Rotterdam Spring 2026 | 1632 | 29 | 29 | -0.0207 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.78756,"matches":196,"rating":0.038728},"rating":{"matches":1395,"rating":0.233835}}` |
| PGL Bucharest 2026 | 1661 | 39 | 39 | +0.0079 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.753182,"matches":225,"rating":0.049683},"rating":{"matches":1395,"rating":0.233835}}` |
| IEM Rio 2026 | 1700 | 30 | 30 | +0.0372 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.792052,"matches":264,"rating":0.057811},"rating":{"matches":1395,"rating":0.233835}}` |
| BLAST Rivals Spring 2026 | 1730 | 13 | 13 | -0.0210 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.838199,"matches":292,"rating":0.062866},"rating":{"matches":1397,"rating":0.234121}}` |
| PGL Astana 2026 | 1743 | 41 | 41 | +0.0005 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.821135,"matches":305,"rating":0.064026},"rating":{"matches":1397,"rating":0.234121}}` |
| IEM Atlanta 2026 | 1743 | 30 | 30 | -0.0205 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.821135,"matches":305,"rating":0.064026},"rating":{"matches":1397,"rating":0.234121}}` |
| CS Asia Championships 2026 | 1814 | 30 | 30 | -0.1676 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.831188,"matches":376,"rating":0.063054},"rating":{"matches":1397,"rating":0.234121}}` |
| IEM Cologne Major 2026 | 1844 | 106 | 106 | +0.0074 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.831188,"matches":376,"rating":0.063054},"rating":{"matches":1427,"rating":0.235804}}` |
| XSE Pro League 2026 | 1950 | 37 | 37 | -0.0333 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.922463,"matches":480,"rating":0.052714},"rating":{"matches":1429,"rating":0.236168}}` |
| BLAST Bounty Summer 2026 | 1990 | 30 | 30 | -0.0559 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.922463,"matches":480,"rating":0.052714},"rating":{"matches":1466,"rating":0.237353}}` |
| Esports World Cup 2026 | 2021 | 56 | 56 | +0.0099 | `{"market":{"market":1.02348,"matches":1},"market+rating":{"market":0.881301,"matches":509,"rating":0.044231},"rating":{"matches":1466,"rating":0.237353}}` |
| BLAST Open Porto Fall 2026 | 2077 | 29 | 29 | +0.0105 | `{"market":{"market":1.02348,"matches":1},"market+rating":{"market":0.84838,"matches":565,"rating":0.062575},"rating":{"matches":1466,"rating":0.237353}}` |
| FISSURE PLAYGROUND Season 3 2026 | 2106 | 29 | 29 | -0.0286 | `{"market":{"market":1.02348,"matches":1},"market+rating":{"market":0.878233,"matches":594,"rating":0.060363},"rating":{"matches":1466,"rating":0.237353}}` |
| StarLadder StarSeries Fall 2026 | 2135 | 14 | 14 | -0.0374 | `{"market":{"market":1.02348,"matches":1},"market+rating":{"market":0.82676,"matches":623,"rating":0.065508},"rating":{"matches":1466,"rating":0.237353}}` |
| ESL Pro League Season 24 2026 | 2149 | 41 | 41 | -0.0364 | `{"market":{"market":1.02348,"matches":1},"market+rating":{"market":0.836126,"matches":637,"rating":0.059155},"rating":{"matches":1466,"rating":0.237353}}` |
| Thunderpick World Championship 2026 | 2190 | 14 | 14 | +0.0029 | `{"market":{"market":1.02348,"matches":1},"market+rating":{"market":0.897026,"matches":678,"rating":0.035897},"rating":{"matches":1466,"rating":0.237353}}` |
| PGL Masters Bucharest 2026 | 2204 | 41 | 41 | -0.0802 | `{"market":{"market":1.02348,"matches":1},"market+rating":{"market":0.889965,"matches":692,"rating":0.039133},"rating":{"matches":1466,"rating":0.237353}}` |
| IEM Beijing 2026 | 2245 | 30 | 30 | -0.0556 | `{"market":{"market":1.02348,"matches":1},"market+rating":{"market":0.892155,"matches":733,"rating":0.02075},"rating":{"matches":1466,"rating":0.237353}}` |
| BLAST Rivals Fall 2026 | 2275 | 13 | 13 | +0.0136 | `{"market":{"market":1.02348,"matches":1},"market+rating":{"market":0.942883,"matches":763,"rating":0.001783},"rating":{"matches":1466,"rating":0.237353}}` |

## `market_source_pinnacle`: no evidence

the anchor with Pinnacle's last price before the cutoff as the market, on Matches every source priced.

Verdict: computed on 28 paired Matches, under the floor of 100.

| On the 28 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.7655 | 0.2745 | 57.1% |
| market_source_pinnacle | 0.7655 | 0.2745 | 57.1% |

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| BLAST Premier Groups Fall 2024 | 0 | 0 | 0 | — | — |
| IEM Cologne 2024 | 26 | 0 | 0 | — | — |
| BLAST Premier Showdown Fall 2024 | 75 | 0 | 0 | — | — |
| BetBoom Dacha Belgrade season 2 2024 | 75 | 0 | 0 | — | — |
| ESL Pro League Season 20 2024 | 115 | 0 | 0 | — | — |
| BLAST Premier Final Fall 2024 | 194 | 0 | 0 | — | — |
| IEM Rio 2024 | 207 | 0 | 0 | — | — |
| Thunderpick World Championship 2024 | 236 | 0 | 0 | — | — |
| BLAST Premier World Final 2024 | 236 | 0 | 0 | — | — |
| Perfect World Major Shanghai: American RMR 2024 | 277 | 0 | 0 | — | — |
| Perfect World Major Shanghai: European RMR A 2024 | 311 | 0 | 0 | — | — |
| Perfect World Major Shanghai: European RMR B 2024 | 346 | 0 | 0 | — | — |
| Perfect World Major Shanghai Major 2024 | 380 | 0 | 0 | — | — |
| BLAST Premier Bounty Closed Qualifier 2025 | 453 | 0 | 0 | — | — |
| BLAST Premier Bounty season 1 2025 | 477 | 0 | 0 | — | — |
| IEM Katowice 2025 | 484 | 0 | 0 | — | — |
| PGL Cluj-Napoca 2025 | 533 | 0 | 0 | — | — |
| ESL Pro League Season 21 2025 | 574 | 0 | 0 | — | — |
| BLAST Open Lisbon Spring 2025 | 614 | 0 | 0 | — | — |
| PGL Bucharest 2025 | 643 | 0 | 0 | — | — |
| BLAST.tv Major Austin: European Regional Qualifier 2025 | 684 | 0 | 0 | — | — |
| IEM Melbourne 2025 | 719 | 0 | 0 | — | — |
| BLAST Rivals Spring 2025 | 748 | 0 | 0 | — | — |
| PGL Astana 2025 | 761 | 0 | 0 | — | — |
| IEM Dallas 2025 | 802 | 0 | 0 | — | — |
| BLAST.tv Major Austin 2025 | 831 | 0 | 0 | — | — |
| FISSURE PLAYGROUND Season 1 2025 | 937 | 0 | 0 | — | — |
| IEM Cologne 2025 | 964 | 0 | 0 | — | — |
| BLAST Bounty Fall 2025 | 1013 | 0 | 0 | — | — |
| Esports World Cup 2025 | 1044 | 0 | 0 | — | — |
| BLAST Open London Fall 2025 | 1060 | 0 | 0 | — | — |
| FISSURE PLAYGROUND Season 2 2025 | 1089 | 0 | 0 | — | — |
| ESL Pro League Season 22 2025 | 1129 | 0 | 0 | — | — |
| CS Asia Championships 2025 | 1202 | 0 | 0 | — | — |
| Thunderpick World Championship 2025 | 1202 | 0 | 0 | — | — |
| PGL Masters Bucharest 2025 | 1246 | 0 | 0 | — | — |
| IEM Chengdu 2025 | 1287 | 0 | 0 | — | — |
| BLAST Rivals Fall 2025 | 1317 | 0 | 0 | — | — |
| StarLadder Major Budapest 2025 | 1330 | 0 | 0 | — | — |
| BLAST Bounty Winter 2026 | 1436 | 0 | 0 | — | — |
| IEM Krakow 2026 | 1467 | 0 | 0 | — | — |
| PGL Cluj-Napoca 2026 | 1517 | 0 | 0 | — | — |
| ESL Pro League Season 23 2026 | 1558 | 0 | 0 | — | — |
| BLAST Open Rotterdam Spring 2026 | 1632 | 0 | 0 | — | — |
| PGL Bucharest 2026 | 1661 | 0 | 0 | — | — |
| IEM Rio 2026 | 1700 | 0 | 0 | — | — |
| BLAST Rivals Spring 2026 | 1730 | 0 | 0 | — | — |
| PGL Astana 2026 | 1743 | 0 | 0 | — | — |
| IEM Atlanta 2026 | 1743 | 0 | 0 | — | — |
| CS Asia Championships 2026 | 1814 | 0 | 0 | — | — |
| IEM Cologne Major 2026 | 1844 | 0 | 0 | — | — |
| XSE Pro League 2026 | 1950 | 0 | 0 | — | — |
| BLAST Bounty Summer 2026 | 1990 | 0 | 0 | — | — |
| Esports World Cup 2026 | 2021 | 0 | 0 | — | — |
| BLAST Open Porto Fall 2026 | 2077 | 0 | 0 | — | — |
| FISSURE PLAYGROUND Season 3 2026 | 2106 | 0 | 0 | — | — |
| StarLadder StarSeries Fall 2026 | 2135 | 0 | 0 | — | — |
| ESL Pro League Season 24 2026 | 2149 | 0 | 0 | — | — |
| Thunderpick World Championship 2026 | 2190 | 0 | 0 | — | — |
| PGL Masters Bucharest 2026 | 2204 | 28 | 28 | +0.0000 | — |
| IEM Beijing 2026 | 2245 | 0 | 0 | — | — |
| BLAST Rivals Fall 2026 | 2275 | 0 | 0 | — | — |

## `market_source_polymarket`: no evidence

the anchor with Polymarket's last price before the cutoff as the market, on Matches every source priced.

Verdict: computed on 28 paired Matches, under the floor of 100.

| On the 28 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.7655 | 0.2745 | 57.1% |
| market_source_polymarket | 0.7885 | 0.2819 | 57.1% |

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| BLAST Premier Groups Fall 2024 | 0 | 0 | 0 | — | — |
| IEM Cologne 2024 | 26 | 0 | 0 | — | — |
| BLAST Premier Showdown Fall 2024 | 75 | 0 | 0 | — | — |
| BetBoom Dacha Belgrade season 2 2024 | 75 | 0 | 0 | — | — |
| ESL Pro League Season 20 2024 | 115 | 0 | 0 | — | — |
| BLAST Premier Final Fall 2024 | 194 | 0 | 0 | — | — |
| IEM Rio 2024 | 207 | 0 | 0 | — | — |
| Thunderpick World Championship 2024 | 236 | 0 | 0 | — | — |
| BLAST Premier World Final 2024 | 236 | 0 | 0 | — | — |
| Perfect World Major Shanghai: American RMR 2024 | 277 | 0 | 0 | — | — |
| Perfect World Major Shanghai: European RMR A 2024 | 311 | 0 | 0 | — | — |
| Perfect World Major Shanghai: European RMR B 2024 | 346 | 0 | 0 | — | — |
| Perfect World Major Shanghai Major 2024 | 380 | 0 | 0 | — | — |
| BLAST Premier Bounty Closed Qualifier 2025 | 453 | 0 | 0 | — | — |
| BLAST Premier Bounty season 1 2025 | 477 | 0 | 0 | — | — |
| IEM Katowice 2025 | 484 | 0 | 0 | — | — |
| PGL Cluj-Napoca 2025 | 533 | 0 | 0 | — | — |
| ESL Pro League Season 21 2025 | 574 | 0 | 0 | — | — |
| BLAST Open Lisbon Spring 2025 | 614 | 0 | 0 | — | — |
| PGL Bucharest 2025 | 643 | 0 | 0 | — | — |
| BLAST.tv Major Austin: European Regional Qualifier 2025 | 684 | 0 | 0 | — | — |
| IEM Melbourne 2025 | 719 | 0 | 0 | — | — |
| BLAST Rivals Spring 2025 | 748 | 0 | 0 | — | — |
| PGL Astana 2025 | 761 | 0 | 0 | — | — |
| IEM Dallas 2025 | 802 | 0 | 0 | — | — |
| BLAST.tv Major Austin 2025 | 831 | 0 | 0 | — | — |
| FISSURE PLAYGROUND Season 1 2025 | 937 | 0 | 0 | — | — |
| IEM Cologne 2025 | 964 | 0 | 0 | — | — |
| BLAST Bounty Fall 2025 | 1013 | 0 | 0 | — | — |
| Esports World Cup 2025 | 1044 | 0 | 0 | — | — |
| BLAST Open London Fall 2025 | 1060 | 0 | 0 | — | — |
| FISSURE PLAYGROUND Season 2 2025 | 1089 | 0 | 0 | — | — |
| ESL Pro League Season 22 2025 | 1129 | 0 | 0 | — | — |
| CS Asia Championships 2025 | 1202 | 0 | 0 | — | — |
| Thunderpick World Championship 2025 | 1202 | 0 | 0 | — | — |
| PGL Masters Bucharest 2025 | 1246 | 0 | 0 | — | — |
| IEM Chengdu 2025 | 1287 | 0 | 0 | — | — |
| BLAST Rivals Fall 2025 | 1317 | 0 | 0 | — | — |
| StarLadder Major Budapest 2025 | 1330 | 0 | 0 | — | — |
| BLAST Bounty Winter 2026 | 1436 | 0 | 0 | — | — |
| IEM Krakow 2026 | 1467 | 0 | 0 | — | — |
| PGL Cluj-Napoca 2026 | 1517 | 0 | 0 | — | — |
| ESL Pro League Season 23 2026 | 1558 | 0 | 0 | — | — |
| BLAST Open Rotterdam Spring 2026 | 1632 | 0 | 0 | — | — |
| PGL Bucharest 2026 | 1661 | 0 | 0 | — | — |
| IEM Rio 2026 | 1700 | 0 | 0 | — | — |
| BLAST Rivals Spring 2026 | 1730 | 0 | 0 | — | — |
| PGL Astana 2026 | 1743 | 0 | 0 | — | — |
| IEM Atlanta 2026 | 1743 | 0 | 0 | — | — |
| CS Asia Championships 2026 | 1814 | 0 | 0 | — | — |
| IEM Cologne Major 2026 | 1844 | 0 | 0 | — | — |
| XSE Pro League 2026 | 1950 | 0 | 0 | — | — |
| BLAST Bounty Summer 2026 | 1990 | 0 | 0 | — | — |
| Esports World Cup 2026 | 2021 | 0 | 0 | — | — |
| BLAST Open Porto Fall 2026 | 2077 | 0 | 0 | — | — |
| FISSURE PLAYGROUND Season 3 2026 | 2106 | 0 | 0 | — | — |
| StarLadder StarSeries Fall 2026 | 2135 | 0 | 0 | — | — |
| ESL Pro League Season 24 2026 | 2149 | 0 | 0 | — | — |
| Thunderpick World Championship 2026 | 2190 | 0 | 0 | — | — |
| PGL Masters Bucharest 2026 | 2204 | 28 | 28 | +0.0230 | — |
| IEM Beijing 2026 | 2245 | 0 | 0 | — | — |
| BLAST Rivals Fall 2026 | 2275 | 0 | 0 | — | — |

## `market_source_kalshi`: no evidence

the anchor with Kalshi's last price before the cutoff as the market, on Matches every source priced.

Verdict: computed on 28 paired Matches, under the floor of 100.

| On the 28 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.7655 | 0.2745 | 57.1% |
| market_source_kalshi | 0.7815 | 0.2797 | 57.1% |

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| BLAST Premier Groups Fall 2024 | 0 | 0 | 0 | — | — |
| IEM Cologne 2024 | 26 | 0 | 0 | — | — |
| BLAST Premier Showdown Fall 2024 | 75 | 0 | 0 | — | — |
| BetBoom Dacha Belgrade season 2 2024 | 75 | 0 | 0 | — | — |
| ESL Pro League Season 20 2024 | 115 | 0 | 0 | — | — |
| BLAST Premier Final Fall 2024 | 194 | 0 | 0 | — | — |
| IEM Rio 2024 | 207 | 0 | 0 | — | — |
| Thunderpick World Championship 2024 | 236 | 0 | 0 | — | — |
| BLAST Premier World Final 2024 | 236 | 0 | 0 | — | — |
| Perfect World Major Shanghai: American RMR 2024 | 277 | 0 | 0 | — | — |
| Perfect World Major Shanghai: European RMR A 2024 | 311 | 0 | 0 | — | — |
| Perfect World Major Shanghai: European RMR B 2024 | 346 | 0 | 0 | — | — |
| Perfect World Major Shanghai Major 2024 | 380 | 0 | 0 | — | — |
| BLAST Premier Bounty Closed Qualifier 2025 | 453 | 0 | 0 | — | — |
| BLAST Premier Bounty season 1 2025 | 477 | 0 | 0 | — | — |
| IEM Katowice 2025 | 484 | 0 | 0 | — | — |
| PGL Cluj-Napoca 2025 | 533 | 0 | 0 | — | — |
| ESL Pro League Season 21 2025 | 574 | 0 | 0 | — | — |
| BLAST Open Lisbon Spring 2025 | 614 | 0 | 0 | — | — |
| PGL Bucharest 2025 | 643 | 0 | 0 | — | — |
| BLAST.tv Major Austin: European Regional Qualifier 2025 | 684 | 0 | 0 | — | — |
| IEM Melbourne 2025 | 719 | 0 | 0 | — | — |
| BLAST Rivals Spring 2025 | 748 | 0 | 0 | — | — |
| PGL Astana 2025 | 761 | 0 | 0 | — | — |
| IEM Dallas 2025 | 802 | 0 | 0 | — | — |
| BLAST.tv Major Austin 2025 | 831 | 0 | 0 | — | — |
| FISSURE PLAYGROUND Season 1 2025 | 937 | 0 | 0 | — | — |
| IEM Cologne 2025 | 964 | 0 | 0 | — | — |
| BLAST Bounty Fall 2025 | 1013 | 0 | 0 | — | — |
| Esports World Cup 2025 | 1044 | 0 | 0 | — | — |
| BLAST Open London Fall 2025 | 1060 | 0 | 0 | — | — |
| FISSURE PLAYGROUND Season 2 2025 | 1089 | 0 | 0 | — | — |
| ESL Pro League Season 22 2025 | 1129 | 0 | 0 | — | — |
| CS Asia Championships 2025 | 1202 | 0 | 0 | — | — |
| Thunderpick World Championship 2025 | 1202 | 0 | 0 | — | — |
| PGL Masters Bucharest 2025 | 1246 | 0 | 0 | — | — |
| IEM Chengdu 2025 | 1287 | 0 | 0 | — | — |
| BLAST Rivals Fall 2025 | 1317 | 0 | 0 | — | — |
| StarLadder Major Budapest 2025 | 1330 | 0 | 0 | — | — |
| BLAST Bounty Winter 2026 | 1436 | 0 | 0 | — | — |
| IEM Krakow 2026 | 1467 | 0 | 0 | — | — |
| PGL Cluj-Napoca 2026 | 1517 | 0 | 0 | — | — |
| ESL Pro League Season 23 2026 | 1558 | 0 | 0 | — | — |
| BLAST Open Rotterdam Spring 2026 | 1632 | 0 | 0 | — | — |
| PGL Bucharest 2026 | 1661 | 0 | 0 | — | — |
| IEM Rio 2026 | 1700 | 0 | 0 | — | — |
| BLAST Rivals Spring 2026 | 1730 | 0 | 0 | — | — |
| PGL Astana 2026 | 1743 | 0 | 0 | — | — |
| IEM Atlanta 2026 | 1743 | 0 | 0 | — | — |
| CS Asia Championships 2026 | 1814 | 0 | 0 | — | — |
| IEM Cologne Major 2026 | 1844 | 0 | 0 | — | — |
| XSE Pro League 2026 | 1950 | 0 | 0 | — | — |
| BLAST Bounty Summer 2026 | 1990 | 0 | 0 | — | — |
| Esports World Cup 2026 | 2021 | 0 | 0 | — | — |
| BLAST Open Porto Fall 2026 | 2077 | 0 | 0 | — | — |
| FISSURE PLAYGROUND Season 3 2026 | 2106 | 0 | 0 | — | — |
| StarLadder StarSeries Fall 2026 | 2135 | 0 | 0 | — | — |
| ESL Pro League Season 24 2026 | 2149 | 0 | 0 | — | — |
| Thunderpick World Championship 2026 | 2190 | 0 | 0 | — | — |
| PGL Masters Bucharest 2026 | 2204 | 28 | 28 | +0.0160 | — |
| IEM Beijing 2026 | 2245 | 0 | 0 | — | — |
| BLAST Rivals Fall 2026 | 2275 | 0 | 0 | — | — |

## `market_source_average`: no evidence

the anchor with the mean of Pinnacle, Polymarket and Kalshi as the market, on Matches every source priced.

Verdict: computed on 28 paired Matches, under the floor of 100.

| On the 28 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.7655 | 0.2745 | 57.1% |
| market_source_average | 0.7779 | 0.2786 | 57.1% |

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| BLAST Premier Groups Fall 2024 | 0 | 0 | 0 | — | — |
| IEM Cologne 2024 | 26 | 0 | 0 | — | — |
| BLAST Premier Showdown Fall 2024 | 75 | 0 | 0 | — | — |
| BetBoom Dacha Belgrade season 2 2024 | 75 | 0 | 0 | — | — |
| ESL Pro League Season 20 2024 | 115 | 0 | 0 | — | — |
| BLAST Premier Final Fall 2024 | 194 | 0 | 0 | — | — |
| IEM Rio 2024 | 207 | 0 | 0 | — | — |
| Thunderpick World Championship 2024 | 236 | 0 | 0 | — | — |
| BLAST Premier World Final 2024 | 236 | 0 | 0 | — | — |
| Perfect World Major Shanghai: American RMR 2024 | 277 | 0 | 0 | — | — |
| Perfect World Major Shanghai: European RMR A 2024 | 311 | 0 | 0 | — | — |
| Perfect World Major Shanghai: European RMR B 2024 | 346 | 0 | 0 | — | — |
| Perfect World Major Shanghai Major 2024 | 380 | 0 | 0 | — | — |
| BLAST Premier Bounty Closed Qualifier 2025 | 453 | 0 | 0 | — | — |
| BLAST Premier Bounty season 1 2025 | 477 | 0 | 0 | — | — |
| IEM Katowice 2025 | 484 | 0 | 0 | — | — |
| PGL Cluj-Napoca 2025 | 533 | 0 | 0 | — | — |
| ESL Pro League Season 21 2025 | 574 | 0 | 0 | — | — |
| BLAST Open Lisbon Spring 2025 | 614 | 0 | 0 | — | — |
| PGL Bucharest 2025 | 643 | 0 | 0 | — | — |
| BLAST.tv Major Austin: European Regional Qualifier 2025 | 684 | 0 | 0 | — | — |
| IEM Melbourne 2025 | 719 | 0 | 0 | — | — |
| BLAST Rivals Spring 2025 | 748 | 0 | 0 | — | — |
| PGL Astana 2025 | 761 | 0 | 0 | — | — |
| IEM Dallas 2025 | 802 | 0 | 0 | — | — |
| BLAST.tv Major Austin 2025 | 831 | 0 | 0 | — | — |
| FISSURE PLAYGROUND Season 1 2025 | 937 | 0 | 0 | — | — |
| IEM Cologne 2025 | 964 | 0 | 0 | — | — |
| BLAST Bounty Fall 2025 | 1013 | 0 | 0 | — | — |
| Esports World Cup 2025 | 1044 | 0 | 0 | — | — |
| BLAST Open London Fall 2025 | 1060 | 0 | 0 | — | — |
| FISSURE PLAYGROUND Season 2 2025 | 1089 | 0 | 0 | — | — |
| ESL Pro League Season 22 2025 | 1129 | 0 | 0 | — | — |
| CS Asia Championships 2025 | 1202 | 0 | 0 | — | — |
| Thunderpick World Championship 2025 | 1202 | 0 | 0 | — | — |
| PGL Masters Bucharest 2025 | 1246 | 0 | 0 | — | — |
| IEM Chengdu 2025 | 1287 | 0 | 0 | — | — |
| BLAST Rivals Fall 2025 | 1317 | 0 | 0 | — | — |
| StarLadder Major Budapest 2025 | 1330 | 0 | 0 | — | — |
| BLAST Bounty Winter 2026 | 1436 | 0 | 0 | — | — |
| IEM Krakow 2026 | 1467 | 0 | 0 | — | — |
| PGL Cluj-Napoca 2026 | 1517 | 0 | 0 | — | — |
| ESL Pro League Season 23 2026 | 1558 | 0 | 0 | — | — |
| BLAST Open Rotterdam Spring 2026 | 1632 | 0 | 0 | — | — |
| PGL Bucharest 2026 | 1661 | 0 | 0 | — | — |
| IEM Rio 2026 | 1700 | 0 | 0 | — | — |
| BLAST Rivals Spring 2026 | 1730 | 0 | 0 | — | — |
| PGL Astana 2026 | 1743 | 0 | 0 | — | — |
| IEM Atlanta 2026 | 1743 | 0 | 0 | — | — |
| CS Asia Championships 2026 | 1814 | 0 | 0 | — | — |
| IEM Cologne Major 2026 | 1844 | 0 | 0 | — | — |
| XSE Pro League 2026 | 1950 | 0 | 0 | — | — |
| BLAST Bounty Summer 2026 | 1990 | 0 | 0 | — | — |
| Esports World Cup 2026 | 2021 | 0 | 0 | — | — |
| BLAST Open Porto Fall 2026 | 2077 | 0 | 0 | — | — |
| FISSURE PLAYGROUND Season 3 2026 | 2106 | 0 | 0 | — | — |
| StarLadder StarSeries Fall 2026 | 2135 | 0 | 0 | — | — |
| ESL Pro League Season 24 2026 | 2149 | 0 | 0 | — | — |
| Thunderpick World Championship 2026 | 2190 | 0 | 0 | — | — |
| PGL Masters Bucharest 2026 | 2204 | 28 | 28 | +0.0124 | — |
| IEM Beijing 2026 | 2245 | 0 | 0 | — | — |
| BLAST Rivals Fall 2026 | 2275 | 0 | 0 | — | — |

## `line_movement`: no evidence

the anchor's logit moved by how the line moved (closing against opening, and closing against six hours before the cutoff), coefficients fitted walk-forward.

Verdict: the interval of the log-loss difference includes zero.

| On the 744 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.6216 | 0.2137 | 67.2% |
| line_movement | 0.6224 | 0.2137 | 67.3% |

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| BLAST Premier Groups Fall 2024 | 0 | 0 | 0 | — | — |
| IEM Cologne 2024 | 26 | 0 | 0 | — | — |
| BLAST Premier Showdown Fall 2024 | 75 | 0 | 0 | — | — |
| BetBoom Dacha Belgrade season 2 2024 | 75 | 0 | 0 | — | — |
| ESL Pro League Season 20 2024 | 115 | 0 | 0 | — | — |
| BLAST Premier Final Fall 2024 | 194 | 0 | 0 | — | — |
| IEM Rio 2024 | 207 | 0 | 0 | — | — |
| Thunderpick World Championship 2024 | 236 | 0 | 0 | — | — |
| BLAST Premier World Final 2024 | 236 | 0 | 0 | — | — |
| Perfect World Major Shanghai: American RMR 2024 | 277 | 0 | 0 | — | — |
| Perfect World Major Shanghai: European RMR A 2024 | 311 | 0 | 0 | — | — |
| Perfect World Major Shanghai: European RMR B 2024 | 346 | 0 | 0 | — | — |
| Perfect World Major Shanghai Major 2024 | 380 | 0 | 0 | — | — |
| BLAST Premier Bounty Closed Qualifier 2025 | 453 | 0 | 0 | — | — |
| BLAST Premier Bounty season 1 2025 | 477 | 0 | 0 | — | — |
| IEM Katowice 2025 | 484 | 0 | 0 | — | — |
| PGL Cluj-Napoca 2025 | 533 | 0 | 0 | — | — |
| ESL Pro League Season 21 2025 | 574 | 0 | 0 | — | — |
| BLAST Open Lisbon Spring 2025 | 614 | 0 | 0 | — | — |
| PGL Bucharest 2025 | 643 | 0 | 0 | — | — |
| BLAST.tv Major Austin: European Regional Qualifier 2025 | 684 | 0 | 0 | — | — |
| IEM Melbourne 2025 | 719 | 0 | 0 | — | — |
| BLAST Rivals Spring 2025 | 748 | 0 | 0 | — | — |
| PGL Astana 2025 | 761 | 0 | 0 | — | — |
| IEM Dallas 2025 | 802 | 0 | 0 | — | — |
| BLAST.tv Major Austin 2025 | 831 | 0 | 0 | — | — |
| FISSURE PLAYGROUND Season 1 2025 | 937 | 0 | 0 | — | — |
| IEM Cologne 2025 | 964 | 0 | 0 | — | — |
| BLAST Bounty Fall 2025 | 1013 | 0 | 0 | — | — |
| Esports World Cup 2025 | 1044 | 0 | 0 | — | — |
| BLAST Open London Fall 2025 | 1060 | 0 | 0 | — | — |
| FISSURE PLAYGROUND Season 2 2025 | 1089 | 0 | 0 | — | — |
| ESL Pro League Season 22 2025 | 1129 | 0 | 0 | — | — |
| CS Asia Championships 2025 | 1202 | 0 | 0 | — | — |
| Thunderpick World Championship 2025 | 1202 | 0 | 0 | — | — |
| PGL Masters Bucharest 2025 | 1246 | 0 | 0 | — | — |
| IEM Chengdu 2025 | 1287 | 0 | 0 | — | — |
| BLAST Rivals Fall 2025 | 1317 | 0 | 0 | — | — |
| StarLadder Major Budapest 2025 | 1330 | 0 | 0 | — | — |
| BLAST Bounty Winter 2026 | 1436 | 0 | 0 | — | — |
| IEM Krakow 2026 | 1467 | 49 | 49 | -0.0041 | — |
| PGL Cluj-Napoca 2026 | 1517 | 41 | 41 | +0.0278 | — |
| ESL Pro League Season 23 2026 | 1558 | 74 | 74 | -0.0057 | — |
| BLAST Open Rotterdam Spring 2026 | 1632 | 29 | 29 | -0.0114 | — |
| PGL Bucharest 2026 | 1661 | 39 | 39 | -0.0031 | — |
| IEM Rio 2026 | 1700 | 28 | 28 | -0.0095 | — |
| BLAST Rivals Spring 2026 | 1730 | 13 | 13 | +0.0237 | — |
| PGL Astana 2026 | 1743 | 41 | 41 | -0.0057 | — |
| IEM Atlanta 2026 | 1743 | 30 | 30 | -0.0148 | — |
| CS Asia Championships 2026 | 1814 | 0 | 0 | — | — |
| IEM Cologne Major 2026 | 1844 | 103 | 103 | -0.0056 | — |
| XSE Pro League 2026 | 1950 | 0 | 0 | — | — |
| BLAST Bounty Summer 2026 | 1990 | 30 | 30 | +0.0413 | — |
| Esports World Cup 2026 | 2021 | 56 | 56 | -0.0024 | — |
| BLAST Open Porto Fall 2026 | 2077 | 29 | 29 | +0.0157 | — |
| FISSURE PLAYGROUND Season 3 2026 | 2106 | 29 | 29 | -0.0081 | — |
| StarLadder StarSeries Fall 2026 | 2135 | 14 | 14 | +0.0153 | — |
| ESL Pro League Season 24 2026 | 2149 | 41 | 41 | -0.0064 | — |
| Thunderpick World Championship 2026 | 2190 | 14 | 14 | +0.0054 | — |
| PGL Masters Bucharest 2026 | 2204 | 41 | 41 | +0.0046 | — |
| IEM Beijing 2026 | 2245 | 30 | 30 | +0.0077 | — |
| BLAST Rivals Fall 2026 | 2275 | 13 | 13 | -0.0041 | — |

## `favourite_longshot`: no evidence

the market's price rescaled in logit space by a factor fitted walk-forward on earlier Events (favourite-longshot bias), then blended as the anchor does.

Verdict: the interval of the log-loss difference includes zero.

| On the 746 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.6217 | 0.2137 | 67.2% |
| favourite_longshot | 0.6225 | 0.2139 | 66.9% |

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| BLAST Premier Groups Fall 2024 | 0 | 0 | 0 | — | — |
| IEM Cologne 2024 | 26 | 0 | 0 | — | — |
| BLAST Premier Showdown Fall 2024 | 75 | 0 | 0 | — | — |
| BetBoom Dacha Belgrade season 2 2024 | 75 | 0 | 0 | — | — |
| ESL Pro League Season 20 2024 | 115 | 0 | 0 | — | — |
| BLAST Premier Final Fall 2024 | 194 | 0 | 0 | — | — |
| IEM Rio 2024 | 207 | 0 | 0 | — | — |
| Thunderpick World Championship 2024 | 236 | 0 | 0 | — | — |
| BLAST Premier World Final 2024 | 236 | 0 | 0 | — | — |
| Perfect World Major Shanghai: American RMR 2024 | 277 | 0 | 0 | — | — |
| Perfect World Major Shanghai: European RMR A 2024 | 311 | 0 | 0 | — | — |
| Perfect World Major Shanghai: European RMR B 2024 | 346 | 0 | 0 | — | — |
| Perfect World Major Shanghai Major 2024 | 380 | 0 | 0 | — | — |
| BLAST Premier Bounty Closed Qualifier 2025 | 453 | 0 | 0 | — | — |
| BLAST Premier Bounty season 1 2025 | 477 | 0 | 0 | — | — |
| IEM Katowice 2025 | 484 | 0 | 0 | — | — |
| PGL Cluj-Napoca 2025 | 533 | 0 | 0 | — | — |
| ESL Pro League Season 21 2025 | 574 | 0 | 0 | — | — |
| BLAST Open Lisbon Spring 2025 | 614 | 0 | 0 | — | — |
| PGL Bucharest 2025 | 643 | 0 | 0 | — | — |
| BLAST.tv Major Austin: European Regional Qualifier 2025 | 684 | 0 | 0 | — | — |
| IEM Melbourne 2025 | 719 | 0 | 0 | — | — |
| BLAST Rivals Spring 2025 | 748 | 0 | 0 | — | — |
| PGL Astana 2025 | 761 | 0 | 0 | — | — |
| IEM Dallas 2025 | 802 | 0 | 0 | — | — |
| BLAST.tv Major Austin 2025 | 831 | 0 | 0 | — | — |
| FISSURE PLAYGROUND Season 1 2025 | 937 | 0 | 0 | — | — |
| IEM Cologne 2025 | 964 | 0 | 0 | — | — |
| BLAST Bounty Fall 2025 | 1013 | 0 | 0 | — | — |
| Esports World Cup 2025 | 1044 | 0 | 0 | — | — |
| BLAST Open London Fall 2025 | 1060 | 0 | 0 | — | — |
| FISSURE PLAYGROUND Season 2 2025 | 1089 | 0 | 0 | — | — |
| ESL Pro League Season 22 2025 | 1129 | 0 | 0 | — | — |
| CS Asia Championships 2025 | 1202 | 0 | 0 | — | — |
| Thunderpick World Championship 2025 | 1202 | 0 | 0 | — | — |
| PGL Masters Bucharest 2025 | 1246 | 0 | 0 | — | — |
| IEM Chengdu 2025 | 1287 | 0 | 0 | — | — |
| BLAST Rivals Fall 2025 | 1317 | 0 | 0 | — | — |
| StarLadder Major Budapest 2025 | 1330 | 0 | 0 | — | — |
| BLAST Bounty Winter 2026 | 1436 | 0 | 0 | — | — |
| IEM Krakow 2026 | 1467 | 50 | 50 | -0.0010 | — |
| PGL Cluj-Napoca 2026 | 1517 | 41 | 41 | +0.0075 | — |
| ESL Pro League Season 23 2026 | 1558 | 74 | 74 | +0.0004 | — |
| BLAST Open Rotterdam Spring 2026 | 1632 | 29 | 29 | -0.0087 | — |
| PGL Bucharest 2026 | 1661 | 39 | 39 | +0.0023 | — |
| IEM Rio 2026 | 1700 | 28 | 28 | +0.0030 | — |
| BLAST Rivals Spring 2026 | 1730 | 13 | 13 | -0.0001 | — |
| PGL Astana 2026 | 1743 | 41 | 41 | +0.0002 | — |
| IEM Atlanta 2026 | 1743 | 30 | 30 | -0.0015 | — |
| CS Asia Championships 2026 | 1814 | 0 | 0 | — | — |
| IEM Cologne Major 2026 | 1844 | 104 | 104 | +0.0002 | — |
| XSE Pro League 2026 | 1950 | 0 | 0 | — | — |
| BLAST Bounty Summer 2026 | 1990 | 30 | 30 | +0.0107 | — |
| Esports World Cup 2026 | 2021 | 56 | 56 | +0.0001 | — |
| BLAST Open Porto Fall 2026 | 2077 | 29 | 29 | -0.0004 | — |
| FISSURE PLAYGROUND Season 3 2026 | 2106 | 29 | 29 | +0.0049 | — |
| StarLadder StarSeries Fall 2026 | 2135 | 14 | 14 | -0.0005 | — |
| ESL Pro League Season 24 2026 | 2149 | 41 | 41 | -0.0002 | — |
| Thunderpick World Championship 2026 | 2190 | 14 | 14 | -0.0000 | — |
| PGL Masters Bucharest 2026 | 2204 | 41 | 41 | -0.0002 | — |
| IEM Beijing 2026 | 2245 | 30 | 30 | -0.0001 | — |
| BLAST Rivals Fall 2026 | 2275 | 13 | 13 | -0.0003 | — |

## `own_rating`: promote

Glicko-2 team rating updated after every Match finished before the cutoff (teams joined by team_key), with format-calibrated map scale blended with the market as the anchor does.

Verdict: the interval of the log-loss improvement excludes zero and the Matches with a market line are not worse.

| On the 2168 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.8023 | 0.2434 | 65.4% |
| own_rating | 0.6347 | 0.2218 | 64.2% |

### Direct comparison with Valve rating (2167 paired Matches, 61 Events)

Reference: Valve Regional Standings (unblended). Δ log-loss -0.2331 (90% interval [-0.2952, -0.1757], detectable 0.0917), Δ Brier -0.0306, Δ accuracy -0.0134.

| On the 2167 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| Valve rating | 0.8758 | 0.2560 | 65.0% |
| own_rating | 0.6427 | 0.2254 | 63.7% |

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| BLAST Premier Groups Fall 2024 | 0 | 18 | 0 | — | — |
| IEM Cologne 2024 | 26 | 40 | 40 | -0.1495 | — |
| BLAST Premier Showdown Fall 2024 | 75 | 9 | 9 | -0.1555 | — |
| BetBoom Dacha Belgrade season 2 2024 | 75 | 23 | 23 | +0.1431 | — |
| ESL Pro League Season 20 2024 | 115 | 70 | 68 | -0.1875 | — |
| BLAST Premier Final Fall 2024 | 194 | 13 | 13 | -0.0386 | — |
| IEM Rio 2024 | 207 | 29 | 29 | -0.2208 | — |
| Thunderpick World Championship 2024 | 236 | 26 | 26 | -0.3280 | — |
| BLAST Premier World Final 2024 | 236 | 13 | 13 | -0.4695 | — |
| Perfect World Major Shanghai: American RMR 2024 | 277 | 30 | 30 | -0.0104 | — |
| Perfect World Major Shanghai: European RMR A 2024 | 311 | 30 | 26 | +0.0340 | — |
| Perfect World Major Shanghai: European RMR B 2024 | 346 | 29 | 29 | -0.5107 | — |
| Perfect World Major Shanghai Major 2024 | 380 | 73 | 73 | +0.0040 | — |
| BLAST Premier Bounty Closed Qualifier 2025 | 453 | 22 | 22 | -0.6179 | — |
| BLAST Premier Bounty season 1 2025 | 477 | 7 | 7 | -0.7647 | — |
| IEM Katowice 2025 | 484 | 49 | 47 | -0.2461 | — |
| PGL Cluj-Napoca 2025 | 533 | 41 | 38 | +0.0141 | — |
| ESL Pro League Season 21 2025 | 574 | 39 | 39 | +0.0070 | — |
| BLAST Open Lisbon Spring 2025 | 614 | 28 | 28 | -0.0585 | — |
| PGL Bucharest 2025 | 643 | 40 | 40 | -0.9344 | — |
| BLAST.tv Major Austin: European Regional Qualifier 2025 | 684 | 33 | 33 | -0.4941 | — |
| IEM Melbourne 2025 | 719 | 29 | 29 | -0.1153 | — |
| BLAST Rivals Spring 2025 | 748 | 13 | 13 | -0.1944 | — |
| PGL Astana 2025 | 761 | 39 | 39 | -1.1240 | — |
| IEM Dallas 2025 | 802 | 28 | 28 | +0.0148 | — |
| BLAST.tv Major Austin 2025 | 831 | 105 | 105 | -0.2308 | — |
| FISSURE PLAYGROUND Season 1 2025 | 937 | 27 | 27 | -0.7135 | — |
| IEM Cologne 2025 | 964 | 49 | 49 | +0.0619 | — |
| BLAST Bounty Fall 2025 | 1013 | 28 | 28 | +0.0702 | — |
| Esports World Cup 2025 | 1044 | 16 | 16 | -0.4346 | — |
| BLAST Open London Fall 2025 | 1060 | 29 | 29 | -0.4932 | — |
| FISSURE PLAYGROUND Season 2 2025 | 1089 | 40 | 40 | -0.2742 | — |
| ESL Pro League Season 22 2025 | 1129 | 71 | 71 | -0.7420 | — |
| CS Asia Championships 2025 | 1202 | 30 | 30 | -0.2300 | — |
| Thunderpick World Championship 2025 | 1202 | 13 | 13 | -0.1859 | — |
| PGL Masters Bucharest 2025 | 1246 | 41 | 41 | -0.2672 | — |
| IEM Chengdu 2025 | 1287 | 30 | 30 | -0.0374 | — |
| BLAST Rivals Fall 2025 | 1317 | 13 | 13 | -0.0580 | — |
| StarLadder Major Budapest 2025 | 1330 | 106 | 106 | -0.1424 | — |
| BLAST Bounty Winter 2026 | 1436 | 27 | 27 | -0.0161 | — |
| IEM Krakow 2026 | 1467 | 50 | 50 | -0.0153 | — |
| PGL Cluj-Napoca 2026 | 1517 | 41 | 41 | +0.0256 | — |
| ESL Pro League Season 23 2026 | 1558 | 72 | 72 | -0.0314 | — |
| BLAST Open Rotterdam Spring 2026 | 1632 | 29 | 29 | -0.0199 | — |
| PGL Bucharest 2026 | 1661 | 37 | 37 | +0.0251 | — |
| IEM Rio 2026 | 1700 | 30 | 30 | +0.0337 | — |
| BLAST Rivals Spring 2026 | 1730 | 13 | 13 | -0.0198 | — |
| PGL Astana 2026 | 1743 | 38 | 38 | -0.0053 | — |
| IEM Atlanta 2026 | 1743 | 30 | 30 | -0.0163 | — |
| CS Asia Championships 2026 | 1814 | 30 | 30 | -0.1837 | — |
| IEM Cologne Major 2026 | 1844 | 104 | 104 | +0.0167 | — |
| XSE Pro League 2026 | 1950 | 38 | 36 | -0.0625 | — |
| BLAST Bounty Summer 2026 | 1990 | 28 | 28 | -0.0372 | — |
| Esports World Cup 2026 | 2021 | 55 | 55 | +0.0256 | — |
| BLAST Open Porto Fall 2026 | 2077 | 29 | 29 | +0.0150 | — |
| FISSURE PLAYGROUND Season 3 2026 | 2106 | 28 | 28 | -0.0507 | — |
| StarLadder StarSeries Fall 2026 | 2135 | 14 | 14 | -0.0378 | — |
| ESL Pro League Season 24 2026 | 2149 | 39 | 39 | -0.0470 | — |
| Thunderpick World Championship 2026 | 2190 | 14 | 14 | +0.0035 | — |
| PGL Masters Bucharest 2026 | 2204 | 41 | 41 | -0.0892 | — |
| IEM Beijing 2026 | 2245 | 30 | 30 | -0.0618 | — |
| BLAST Rivals Fall 2026 | 2275 | 13 | 13 | +0.0177 | — |

## `recent_form`: no evidence

Recent results over the last 45 days (14-day half-life) weighted by opponent strength from the point-in-time Glicko-2 rating, added as a fitted logit offset to the anchor.

Verdict: the interval of the log-loss difference includes zero.

| On the 1980 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.7834 | 0.2404 | 65.7% |
| recent_form | 0.7825 | 0.2418 | 65.4% |

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| BLAST Premier Groups Fall 2024 | 0 | 0 | 0 | — | — |
| IEM Cologne 2024 | 26 | 0 | 0 | — | — |
| BLAST Premier Showdown Fall 2024 | 75 | 9 | 9 | +0.0014 | — |
| BetBoom Dacha Belgrade season 2 2024 | 75 | 23 | 23 | -0.0007 | — |
| ESL Pro League Season 20 2024 | 115 | 68 | 68 | +0.0083 | — |
| BLAST Premier Final Fall 2024 | 194 | 13 | 13 | -0.0094 | — |
| IEM Rio 2024 | 207 | 29 | 29 | -0.0439 | — |
| Thunderpick World Championship 2024 | 236 | 22 | 22 | +0.0186 | — |
| BLAST Premier World Final 2024 | 236 | 13 | 13 | -0.0871 | — |
| Perfect World Major Shanghai: American RMR 2024 | 277 | 28 | 28 | +0.0173 | — |
| Perfect World Major Shanghai: European RMR A 2024 | 311 | 25 | 25 | -0.0145 | — |
| Perfect World Major Shanghai: European RMR B 2024 | 346 | 28 | 28 | -0.0147 | — |
| Perfect World Major Shanghai Major 2024 | 380 | 71 | 71 | +0.0009 | — |
| BLAST Premier Bounty Closed Qualifier 2025 | 453 | 13 | 13 | +0.0441 | — |
| BLAST Premier Bounty season 1 2025 | 477 | 7 | 7 | +0.0002 | — |
| IEM Katowice 2025 | 484 | 46 | 46 | +0.0305 | — |
| PGL Cluj-Napoca 2025 | 533 | 38 | 38 | -0.0158 | — |
| ESL Pro League Season 21 2025 | 574 | 39 | 39 | -0.0091 | — |
| BLAST Open Lisbon Spring 2025 | 614 | 26 | 26 | +0.0152 | — |
| PGL Bucharest 2025 | 643 | 36 | 36 | -0.0183 | — |
| BLAST.tv Major Austin: European Regional Qualifier 2025 | 684 | 27 | 27 | -0.0203 | — |
| IEM Melbourne 2025 | 719 | 28 | 28 | +0.0006 | — |
| BLAST Rivals Spring 2025 | 748 | 12 | 12 | +0.1017 | — |
| PGL Astana 2025 | 761 | 37 | 37 | -0.0139 | — |
| IEM Dallas 2025 | 802 | 27 | 27 | +0.0135 | — |
| BLAST.tv Major Austin 2025 | 831 | 99 | 99 | -0.0165 | — |
| FISSURE PLAYGROUND Season 1 2025 | 937 | 22 | 22 | -0.0246 | — |
| IEM Cologne 2025 | 964 | 46 | 46 | +0.0028 | — |
| BLAST Bounty Fall 2025 | 1013 | 22 | 22 | -0.0418 | — |
| Esports World Cup 2025 | 1044 | 16 | 16 | -0.1129 | — |
| BLAST Open London Fall 2025 | 1060 | 27 | 27 | -0.0274 | — |
| FISSURE PLAYGROUND Season 2 2025 | 1089 | 39 | 39 | -0.0258 | — |
| ESL Pro League Season 22 2025 | 1129 | 65 | 65 | +0.0104 | — |
| CS Asia Championships 2025 | 1202 | 27 | 27 | -0.0314 | — |
| Thunderpick World Championship 2025 | 1202 | 10 | 10 | +0.0599 | — |
| PGL Masters Bucharest 2025 | 1246 | 37 | 37 | +0.0071 | — |
| IEM Chengdu 2025 | 1287 | 30 | 30 | -0.0154 | — |
| BLAST Rivals Fall 2025 | 1317 | 12 | 12 | +0.0629 | — |
| StarLadder Major Budapest 2025 | 1330 | 101 | 101 | -0.0089 | — |
| BLAST Bounty Winter 2026 | 1436 | 17 | 17 | -0.0007 | — |
| IEM Krakow 2026 | 1467 | 45 | 45 | +0.0262 | — |
| PGL Cluj-Napoca 2026 | 1517 | 41 | 41 | +0.0362 | — |
| ESL Pro League Season 23 2026 | 1558 | 71 | 71 | -0.0193 | — |
| BLAST Open Rotterdam Spring 2026 | 1632 | 27 | 27 | +0.0308 | — |
| PGL Bucharest 2026 | 1661 | 32 | 32 | +0.0127 | — |
| IEM Rio 2026 | 1700 | 27 | 27 | -0.0009 | — |
| BLAST Rivals Spring 2026 | 1730 | 12 | 12 | -0.0308 | — |
| PGL Astana 2026 | 1743 | 34 | 34 | +0.0061 | — |
| IEM Atlanta 2026 | 1743 | 26 | 26 | -0.0128 | — |
| CS Asia Championships 2026 | 1814 | 27 | 27 | +0.0239 | — |
| IEM Cologne Major 2026 | 1844 | 101 | 101 | +0.0140 | — |
| XSE Pro League 2026 | 1950 | 34 | 34 | +0.0134 | — |
| BLAST Bounty Summer 2026 | 1990 | 19 | 19 | +0.0088 | — |
| Esports World Cup 2026 | 2021 | 52 | 52 | -0.0065 | — |
| BLAST Open Porto Fall 2026 | 2077 | 28 | 28 | +0.0201 | — |
| FISSURE PLAYGROUND Season 3 2026 | 2106 | 27 | 27 | +0.0008 | — |
| StarLadder StarSeries Fall 2026 | 2135 | 13 | 13 | -0.0027 | — |
| ESL Pro League Season 24 2026 | 2149 | 39 | 39 | -0.0064 | — |
| Thunderpick World Championship 2026 | 2190 | 13 | 13 | -0.0093 | — |
| PGL Masters Bucharest 2026 | 2204 | 36 | 36 | -0.0051 | — |
| IEM Beijing 2026 | 2245 | 28 | 28 | +0.0292 | — |
| BLAST Rivals Fall 2026 | 2275 | 13 | 13 | -0.0011 | — |

## `head_to_head`: no evidence

Recency-weighted head-to-head series record between the two teams (180-day half-life, shrunk toward even), added as a fitted logit offset to the anchor.

Verdict: the interval of the log-loss difference includes zero.

| On the 1280 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.7396 | 0.2350 | 66.2% |
| head_to_head | 0.7357 | 0.2358 | 65.4% |

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| BLAST Premier Groups Fall 2024 | 0 | 0 | 0 | — | — |
| IEM Cologne 2024 | 26 | 0 | 0 | — | — |
| BLAST Premier Showdown Fall 2024 | 75 | 0 | 0 | — | — |
| BetBoom Dacha Belgrade season 2 2024 | 75 | 0 | 0 | — | — |
| ESL Pro League Season 20 2024 | 115 | 0 | 0 | — | — |
| BLAST Premier Final Fall 2024 | 194 | 8 | 8 | +0.0320 | — |
| IEM Rio 2024 | 207 | 11 | 11 | +0.0014 | — |
| Thunderpick World Championship 2024 | 236 | 8 | 8 | +0.0162 | — |
| BLAST Premier World Final 2024 | 236 | 10 | 10 | +0.0030 | — |
| Perfect World Major Shanghai: American RMR 2024 | 277 | 6 | 6 | +0.0004 | — |
| Perfect World Major Shanghai: European RMR A 2024 | 311 | 8 | 8 | +0.0020 | — |
| Perfect World Major Shanghai: European RMR B 2024 | 346 | 6 | 6 | +0.0031 | — |
| Perfect World Major Shanghai Major 2024 | 380 | 33 | 33 | -0.0069 | — |
| BLAST Premier Bounty Closed Qualifier 2025 | 453 | 9 | 9 | -0.0520 | — |
| BLAST Premier Bounty season 1 2025 | 477 | 6 | 6 | +0.0816 | — |
| IEM Katowice 2025 | 484 | 25 | 25 | +0.0006 | — |
| PGL Cluj-Napoca 2025 | 533 | 23 | 23 | -0.0265 | — |
| ESL Pro League Season 21 2025 | 574 | 31 | 31 | +0.0121 | — |
| BLAST Open Lisbon Spring 2025 | 614 | 22 | 22 | +0.0162 | — |
| PGL Bucharest 2025 | 643 | 20 | 20 | -0.0218 | — |
| BLAST.tv Major Austin: European Regional Qualifier 2025 | 684 | 7 | 7 | -0.0505 | — |
| IEM Melbourne 2025 | 719 | 19 | 19 | -0.0093 | — |
| BLAST Rivals Spring 2025 | 748 | 12 | 12 | +0.0101 | — |
| PGL Astana 2025 | 761 | 25 | 25 | -0.0251 | — |
| IEM Dallas 2025 | 802 | 18 | 18 | -0.0026 | — |
| BLAST.tv Major Austin 2025 | 831 | 40 | 40 | -0.0682 | — |
| FISSURE PLAYGROUND Season 1 2025 | 937 | 15 | 15 | +0.0429 | — |
| IEM Cologne 2025 | 964 | 45 | 45 | -0.0140 | — |
| BLAST Bounty Fall 2025 | 1013 | 18 | 18 | +0.0173 | — |
| Esports World Cup 2025 | 1044 | 14 | 14 | -0.1165 | — |
| BLAST Open London Fall 2025 | 1060 | 23 | 23 | -0.0370 | — |
| FISSURE PLAYGROUND Season 2 2025 | 1089 | 34 | 34 | -0.0567 | — |
| ESL Pro League Season 22 2025 | 1129 | 36 | 36 | -0.0288 | — |
| CS Asia Championships 2025 | 1202 | 16 | 16 | +0.0336 | — |
| Thunderpick World Championship 2025 | 1202 | 10 | 10 | +0.1070 | — |
| PGL Masters Bucharest 2025 | 1246 | 23 | 23 | -0.1151 | — |
| IEM Chengdu 2025 | 1287 | 27 | 27 | +0.0519 | — |
| BLAST Rivals Fall 2025 | 1317 | 10 | 10 | +0.0598 | — |
| StarLadder Major Budapest 2025 | 1330 | 58 | 58 | -0.0002 | — |
| BLAST Bounty Winter 2026 | 1436 | 14 | 14 | -0.0039 | — |
| IEM Krakow 2026 | 1467 | 37 | 37 | +0.0375 | — |
| PGL Cluj-Napoca 2026 | 1517 | 34 | 34 | +0.0434 | — |
| ESL Pro League Season 23 2026 | 1558 | 52 | 52 | -0.0154 | — |
| BLAST Open Rotterdam Spring 2026 | 1632 | 25 | 25 | +0.0437 | — |
| PGL Bucharest 2026 | 1661 | 18 | 18 | +0.0311 | — |
| IEM Rio 2026 | 1700 | 24 | 24 | +0.0555 | — |
| BLAST Rivals Spring 2026 | 1730 | 12 | 12 | +0.0004 | — |
| PGL Astana 2026 | 1743 | 22 | 22 | +0.0397 | — |
| IEM Atlanta 2026 | 1743 | 25 | 25 | -0.0138 | — |
| CS Asia Championships 2026 | 1814 | 20 | 20 | -0.0436 | — |
| IEM Cologne Major 2026 | 1844 | 70 | 70 | +0.0020 | — |
| XSE Pro League 2026 | 1950 | 20 | 20 | +0.0036 | — |
| BLAST Bounty Summer 2026 | 1990 | 19 | 19 | -0.0132 | — |
| Esports World Cup 2026 | 2021 | 42 | 42 | +0.0421 | — |
| BLAST Open Porto Fall 2026 | 2077 | 24 | 24 | +0.0062 | — |
| FISSURE PLAYGROUND Season 3 2026 | 2106 | 16 | 16 | -0.0191 | — |
| StarLadder StarSeries Fall 2026 | 2135 | 11 | 11 | -0.0188 | — |
| ESL Pro League Season 24 2026 | 2149 | 30 | 30 | -0.0165 | — |
| Thunderpick World Championship 2026 | 2190 | 12 | 12 | -0.0155 | — |
| PGL Masters Bucharest 2026 | 2204 | 34 | 34 | -0.0316 | — |
| IEM Beijing 2026 | 2245 | 30 | 30 | -0.0157 | — |
| BLAST Rivals Fall 2026 | 2275 | 13 | 13 | -0.0513 | — |

## `llm_adjustment`: not testable

The three-seat debate's adjustment (HybridAnchored, debate mode `always`, the first seating) on the Matches with a roster flag, from recorded answers.

Verdict: no LLM answers in this run; produce them with `cargo run -p kallbet-ai -- lab --archive <ARCHIVE> --llm <LLM_FILE> --answers <ANSWERS_FILE> --budget claude=<RUNS> --budget codex=<RUNS> --budget antigravity=<RUNS>`.

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| BLAST Premier Groups Fall 2024 | 0 | 0 | 0 | — | — |
| IEM Cologne 2024 | 26 | 0 | 0 | — | — |
| BLAST Premier Showdown Fall 2024 | 75 | 0 | 0 | — | — |
| BetBoom Dacha Belgrade season 2 2024 | 75 | 0 | 0 | — | — |
| ESL Pro League Season 20 2024 | 115 | 0 | 0 | — | — |
| BLAST Premier Final Fall 2024 | 194 | 0 | 0 | — | — |
| IEM Rio 2024 | 207 | 0 | 0 | — | — |
| Thunderpick World Championship 2024 | 236 | 0 | 0 | — | — |
| BLAST Premier World Final 2024 | 236 | 0 | 0 | — | — |
| Perfect World Major Shanghai: American RMR 2024 | 277 | 0 | 0 | — | — |
| Perfect World Major Shanghai: European RMR A 2024 | 311 | 0 | 0 | — | — |
| Perfect World Major Shanghai: European RMR B 2024 | 346 | 0 | 0 | — | — |
| Perfect World Major Shanghai Major 2024 | 380 | 0 | 0 | — | — |
| BLAST Premier Bounty Closed Qualifier 2025 | 453 | 0 | 0 | — | — |
| BLAST Premier Bounty season 1 2025 | 477 | 0 | 0 | — | — |
| IEM Katowice 2025 | 484 | 0 | 0 | — | — |
| PGL Cluj-Napoca 2025 | 533 | 0 | 0 | — | — |
| ESL Pro League Season 21 2025 | 574 | 0 | 0 | — | — |
| BLAST Open Lisbon Spring 2025 | 614 | 0 | 0 | — | — |
| PGL Bucharest 2025 | 643 | 0 | 0 | — | — |
| BLAST.tv Major Austin: European Regional Qualifier 2025 | 684 | 0 | 0 | — | — |
| IEM Melbourne 2025 | 719 | 0 | 0 | — | — |
| BLAST Rivals Spring 2025 | 748 | 0 | 0 | — | — |
| PGL Astana 2025 | 761 | 0 | 0 | — | — |
| IEM Dallas 2025 | 802 | 0 | 0 | — | — |
| BLAST.tv Major Austin 2025 | 831 | 0 | 0 | — | — |
| FISSURE PLAYGROUND Season 1 2025 | 937 | 0 | 0 | — | — |
| IEM Cologne 2025 | 964 | 0 | 0 | — | — |
| BLAST Bounty Fall 2025 | 1013 | 0 | 0 | — | — |
| Esports World Cup 2025 | 1044 | 0 | 0 | — | — |
| BLAST Open London Fall 2025 | 1060 | 0 | 0 | — | — |
| FISSURE PLAYGROUND Season 2 2025 | 1089 | 0 | 0 | — | — |
| ESL Pro League Season 22 2025 | 1129 | 0 | 0 | — | — |
| CS Asia Championships 2025 | 1202 | 0 | 0 | — | — |
| Thunderpick World Championship 2025 | 1202 | 0 | 0 | — | — |
| PGL Masters Bucharest 2025 | 1246 | 0 | 0 | — | — |
| IEM Chengdu 2025 | 1287 | 0 | 0 | — | — |
| BLAST Rivals Fall 2025 | 1317 | 0 | 0 | — | — |
| StarLadder Major Budapest 2025 | 1330 | 0 | 0 | — | — |
| BLAST Bounty Winter 2026 | 1436 | 0 | 0 | — | — |
| IEM Krakow 2026 | 1467 | 0 | 0 | — | — |
| PGL Cluj-Napoca 2026 | 1517 | 0 | 0 | — | — |
| ESL Pro League Season 23 2026 | 1558 | 0 | 0 | — | — |
| BLAST Open Rotterdam Spring 2026 | 1632 | 0 | 0 | — | — |
| PGL Bucharest 2026 | 1661 | 0 | 0 | — | — |
| IEM Rio 2026 | 1700 | 0 | 0 | — | — |
| BLAST Rivals Spring 2026 | 1730 | 0 | 0 | — | — |
| PGL Astana 2026 | 1743 | 0 | 0 | — | — |
| IEM Atlanta 2026 | 1743 | 0 | 0 | — | — |
| CS Asia Championships 2026 | 1814 | 0 | 0 | — | — |
| IEM Cologne Major 2026 | 1844 | 0 | 0 | — | — |
| XSE Pro League 2026 | 1950 | 0 | 0 | — | — |
| BLAST Bounty Summer 2026 | 1990 | 0 | 0 | — | — |
| Esports World Cup 2026 | 2021 | 0 | 0 | — | — |
| BLAST Open Porto Fall 2026 | 2077 | 0 | 0 | — | — |
| FISSURE PLAYGROUND Season 3 2026 | 2106 | 0 | 0 | — | — |
| StarLadder StarSeries Fall 2026 | 2135 | 0 | 0 | — | — |
| ESL Pro League Season 24 2026 | 2149 | 0 | 0 | — | — |
| Thunderpick World Championship 2026 | 2190 | 0 | 0 | — | — |
| PGL Masters Bucharest 2026 | 2204 | 0 | 0 | — | — |
| IEM Beijing 2026 | 2245 | 0 | 0 | — | — |
| BLAST Rivals Fall 2026 | 2275 | 0 | 0 | — | — |

## `lineup_strength`: promote

Mean HLTV player rating of the 5 playing starters from the pre-cutoff Liquipedia roster (90-day half-life, shrunk toward prior 1.00 with weight 6 maps; debutants/unknown stand-ins take the 1.00 prior when at least 3 of 5 starters on each team have HLTV history, else uncovered), added as a fitted logit offset to the anchor.

Verdict: the interval of the log-loss improvement excludes zero and the Matches with a market line are not worse.

| On the 1950 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.8007 | 0.2436 | 65.6% |
| lineup_strength | 0.7981 | 0.2439 | 65.2% |

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| BLAST Premier Groups Fall 2024 | 0 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| IEM Cologne 2024 | 26 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| BLAST Premier Showdown Fall 2024 | 75 | 9 | 9 | -0.0003 | `{"beta":-0.10665580727743276,"training_matches":39}` |
| BetBoom Dacha Belgrade season 2 2024 | 75 | 23 | 23 | -0.0001 | `{"beta":-0.10665580727743276,"training_matches":39}` |
| ESL Pro League Season 20 2024 | 115 | 68 | 68 | -0.0012 | `{"beta":-0.1590055651774291,"training_matches":71}` |
| BLAST Premier Final Fall 2024 | 194 | 13 | 13 | -0.0005 | `{"beta":-0.6629871175506814,"training_matches":139}` |
| IEM Rio 2024 | 207 | 29 | 29 | -0.0109 | `{"beta":-0.6723632786103819,"training_matches":152}` |
| Thunderpick World Championship 2024 | 236 | 26 | 26 | +0.0007 | `{"beta":-1.1135665573558784,"training_matches":181}` |
| BLAST Premier World Final 2024 | 236 | 13 | 13 | -0.0191 | `{"beta":-1.1135665573558784,"training_matches":181}` |
| Perfect World Major Shanghai: American RMR 2024 | 277 | 30 | 30 | -0.0006 | `{"beta":-1.3053571433574023,"training_matches":220}` |
| Perfect World Major Shanghai: European RMR A 2024 | 311 | 24 | 24 | +0.0135 | `{"beta":-1.3145251501561543,"training_matches":250}` |
| Perfect World Major Shanghai: European RMR B 2024 | 346 | 29 | 29 | -0.0099 | `{"beta":-1.0813252557438502,"training_matches":274}` |
| Perfect World Major Shanghai Major 2024 | 380 | 73 | 73 | -0.0009 | `{"beta":-1.3251245446016027,"training_matches":303}` |
| BLAST Premier Bounty Closed Qualifier 2025 | 453 | 22 | 22 | -0.0014 | `{"beta":-1.3583276049509734,"training_matches":376}` |
| BLAST Premier Bounty season 1 2025 | 477 | 7 | 7 | -0.0299 | `{"beta":-1.3752079871814362,"training_matches":398}` |
| IEM Katowice 2025 | 484 | 47 | 47 | +0.0012 | `{"beta":-1.5129507813359053,"training_matches":405}` |
| PGL Cluj-Napoca 2025 | 533 | 38 | 38 | -0.0028 | `{"beta":-1.4701998585318719,"training_matches":452}` |
| ESL Pro League Season 21 2025 | 574 | 39 | 39 | +0.0041 | `{"beta":-1.529508227733885,"training_matches":490}` |
| BLAST Open Lisbon Spring 2025 | 614 | 28 | 28 | +0.0047 | `{"beta":-1.4280629221778263,"training_matches":529}` |
| PGL Bucharest 2025 | 643 | 35 | 35 | -0.0120 | `{"beta":-1.34012832640789,"training_matches":557}` |
| BLAST.tv Major Austin: European Regional Qualifier 2025 | 684 | 33 | 33 | -0.0046 | `{"beta":-1.6090589790955918,"training_matches":592}` |
| IEM Melbourne 2025 | 719 | 28 | 28 | +0.0053 | `{"beta":-1.685247576653094,"training_matches":625}` |
| BLAST Rivals Spring 2025 | 748 | 13 | 13 | +0.0014 | `{"beta":-1.6066943418504542,"training_matches":653}` |
| PGL Astana 2025 | 761 | 39 | 39 | -0.0121 | `{"beta":-1.5948055268902301,"training_matches":666}` |
| IEM Dallas 2025 | 802 | 28 | 28 | +0.0014 | `{"beta":-1.841609651755284,"training_matches":705}` |
| BLAST.tv Major Austin 2025 | 831 | 104 | 104 | -0.0034 | `{"beta":-1.819506277383613,"training_matches":733}` |
| FISSURE PLAYGROUND Season 1 2025 | 937 | 27 | 27 | -0.0208 | `{"beta":-1.9498802741900656,"training_matches":837}` |
| IEM Cologne 2025 | 964 | 49 | 49 | +0.0050 | `{"beta":-2.1809704100032334,"training_matches":864}` |
| BLAST Bounty Fall 2025 | 1013 | 7 | 7 | -0.0130 | `{"beta":-2.0775851105729624,"training_matches":913}` |
| Esports World Cup 2025 | 1044 | 16 | 16 | -0.0330 | `{"beta":-2.110259694459497,"training_matches":920}` |
| BLAST Open London Fall 2025 | 1060 | 7 | 7 | +0.0108 | `{"beta":-2.3069701073709075,"training_matches":936}` |
| FISSURE PLAYGROUND Season 2 2025 | 1089 | 40 | 40 | +0.0063 | `{"beta":-2.279617747033795,"training_matches":943}` |
| ESL Pro League Season 22 2025 | 1129 | 63 | 63 | +0.0008 | `{"beta":-2.183084817702775,"training_matches":983}` |
| CS Asia Championships 2025 | 1202 | 29 | 29 | -0.0023 | `{"beta":-2.1442239763007427,"training_matches":1046}` |
| Thunderpick World Championship 2025 | 1202 | 13 | 13 | -0.0006 | `{"beta":-2.1442239763007427,"training_matches":1046}` |
| PGL Masters Bucharest 2025 | 1246 | 41 | 41 | -0.0075 | `{"beta":-2.1657791694054938,"training_matches":1088}` |
| IEM Chengdu 2025 | 1287 | 30 | 30 | -0.0150 | `{"beta":-2.2683024291737235,"training_matches":1129}` |
| BLAST Rivals Fall 2025 | 1317 | 13 | 13 | +0.0047 | `{"beta":-2.4047969164603646,"training_matches":1159}` |
| StarLadder Major Budapest 2025 | 1330 | 106 | 106 | -0.0035 | `{"beta":-2.383471167885327,"training_matches":1172}` |
| BLAST Bounty Winter 2026 | 1436 | 7 | 7 | -0.0601 | `{"beta":-2.4695991323470423,"training_matches":1278}` |
| IEM Krakow 2026 | 1467 | 50 | 50 | +0.0012 | `{"beta":-2.58947480821843,"training_matches":1285}` |
| PGL Cluj-Napoca 2026 | 1517 | 41 | 41 | +0.0072 | `{"beta":-2.547452717827067,"training_matches":1335}` |
| ESL Pro League Season 23 2026 | 1558 | 8 | 8 | -0.0166 | `{"beta":-2.4487193812391483,"training_matches":1376}` |
| BLAST Open Rotterdam Spring 2026 | 1632 | 24 | 24 | -0.0147 | `{"beta":-2.4849273443916955,"training_matches":1384}` |
| PGL Bucharest 2026 | 1661 | 37 | 37 | +0.0002 | `{"beta":-2.571077045080745,"training_matches":1408}` |
| IEM Rio 2026 | 1700 | 30 | 30 | +0.0122 | `{"beta":-2.5610076771806654,"training_matches":1445}` |
| BLAST Rivals Spring 2026 | 1730 | 13 | 13 | -0.0087 | `{"beta":-2.4527373494463927,"training_matches":1475}` |
| PGL Astana 2026 | 1743 | 38 | 38 | +0.0034 | `{"beta":-2.479872921673267,"training_matches":1488}` |
| IEM Atlanta 2026 | 1743 | 30 | 30 | -0.0206 | `{"beta":-2.479872921673267,"training_matches":1488}` |
| CS Asia Championships 2026 | 1814 | 30 | 30 | -0.0070 | `{"beta":-2.5796359013505343,"training_matches":1556}` |
| IEM Cologne Major 2026 | 1844 | 105 | 105 | +0.0057 | `{"beta":-2.62935084505325,"training_matches":1586}` |
| XSE Pro League 2026 | 1950 | 36 | 36 | +0.0053 | `{"beta":-2.438189955312052,"training_matches":1691}` |
| BLAST Bounty Summer 2026 | 1990 | 6 | 6 | -0.0013 | `{"beta":-2.3798209250324387,"training_matches":1727}` |
| Esports World Cup 2026 | 2021 | 55 | 55 | +0.0054 | `{"beta":-2.378526663372356,"training_matches":1733}` |
| BLAST Open Porto Fall 2026 | 2077 | 26 | 26 | +0.0146 | `{"beta":-2.2755735896814224,"training_matches":1788}` |
| FISSURE PLAYGROUND Season 3 2026 | 2106 | 28 | 28 | -0.0142 | `{"beta":-2.1599058536973885,"training_matches":1814}` |
| StarLadder StarSeries Fall 2026 | 2135 | 14 | 14 | -0.0037 | `{"beta":-2.269679410908365,"training_matches":1842}` |
| ESL Pro League Season 24 2026 | 2149 | 35 | 35 | -0.0008 | `{"beta":-2.279986587546663,"training_matches":1856}` |
| Thunderpick World Championship 2026 | 2190 | 14 | 14 | -0.0004 | `{"beta":-2.275476814870035,"training_matches":1891}` |
| PGL Masters Bucharest 2026 | 2204 | 41 | 41 | -0.0257 | `{"beta":-2.2742371031318998,"training_matches":1905}` |
| IEM Beijing 2026 | 2245 | 30 | 30 | -0.0090 | `{"beta":-2.527569463805296,"training_matches":1946}` |
| BLAST Rivals Fall 2026 | 2275 | 13 | 13 | -0.0102 | `{"beta":-2.5821508846855563,"training_matches":1976}` |

## `player_form`: no evidence

Mean recent-minus-longer-run HLTV rating form of the 5 playing starters from the pre-cutoff Liquipedia roster (30d/14d half-life vs 180d/90d half-life, shrunk toward neutral 0.0 with weight 4 maps; debutants/players without 2+ baseline & recent Matches take 0.0 when at least 3 of 5 starters on each team have form, else uncovered), added as a fitted logit offset to the anchor.

Verdict: the interval of the log-loss difference includes zero.

| On the 1567 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.7748 | 0.2376 | 66.6% |
| player_form | 0.7748 | 0.2376 | 66.6% |

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| BLAST Premier Groups Fall 2024 | 0 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| IEM Cologne 2024 | 26 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| BLAST Premier Showdown Fall 2024 | 75 | 8 | 8 | +0.0000 | `{"beta":0.010286274881236662,"training_matches":30}` |
| BetBoom Dacha Belgrade season 2 2024 | 75 | 20 | 20 | -0.0000 | `{"beta":0.010286274881236662,"training_matches":30}` |
| ESL Pro League Season 20 2024 | 115 | 59 | 59 | -0.0000 | `{"beta":0.006140325031839224,"training_matches":58}` |
| BLAST Premier Final Fall 2024 | 194 | 13 | 13 | +0.0002 | `{"beta":0.028904054991354344,"training_matches":117}` |
| IEM Rio 2024 | 207 | 27 | 27 | +0.0005 | `{"beta":-0.05649941599403631,"training_matches":130}` |
| Thunderpick World Championship 2024 | 236 | 18 | 18 | +0.0002 | `{"beta":0.1637775955094828,"training_matches":157}` |
| BLAST Premier World Final 2024 | 236 | 12 | 12 | -0.0003 | `{"beta":0.1637775955094828,"training_matches":157}` |
| Perfect World Major Shanghai: American RMR 2024 | 277 | 22 | 22 | +0.0004 | `{"beta":0.1604031760636983,"training_matches":187}` |
| Perfect World Major Shanghai: European RMR A 2024 | 311 | 19 | 19 | -0.0017 | `{"beta":0.11270392155399109,"training_matches":209}` |
| Perfect World Major Shanghai: European RMR B 2024 | 346 | 22 | 22 | -0.0003 | `{"beta":0.39452201718155555,"training_matches":228}` |
| Perfect World Major Shanghai Major 2024 | 380 | 63 | 63 | +0.0009 | `{"beta":0.4085979228711065,"training_matches":250}` |
| BLAST Premier Bounty Closed Qualifier 2025 | 453 | 0 | 0 | — | `{"beta":0.26938565649016477,"training_matches":313}` |
| BLAST Premier Bounty season 1 2025 | 477 | 3 | 3 | +0.0063 | `{"beta":0.26938565649016477,"training_matches":313}` |
| IEM Katowice 2025 | 484 | 35 | 35 | +0.0001 | `{"beta":0.2014265012087344,"training_matches":316}` |
| PGL Cluj-Napoca 2025 | 533 | 38 | 38 | +0.0003 | `{"beta":0.18255225781602974,"training_matches":351}` |
| ESL Pro League Season 21 2025 | 574 | 34 | 34 | -0.0000 | `{"beta":0.1312205902611442,"training_matches":389}` |
| BLAST Open Lisbon Spring 2025 | 614 | 25 | 25 | +0.0002 | `{"beta":0.1381434228860336,"training_matches":423}` |
| PGL Bucharest 2025 | 643 | 31 | 31 | -0.0002 | `{"beta":0.11183079919628935,"training_matches":448}` |
| BLAST.tv Major Austin: European Regional Qualifier 2025 | 684 | 24 | 24 | +0.0005 | `{"beta":0.17131651524925268,"training_matches":479}` |
| IEM Melbourne 2025 | 719 | 26 | 26 | -0.0001 | `{"beta":0.10323573249508991,"training_matches":503}` |
| BLAST Rivals Spring 2025 | 748 | 11 | 11 | +0.0006 | `{"beta":0.13100935809602637,"training_matches":529}` |
| PGL Astana 2025 | 761 | 34 | 34 | -0.0000 | `{"beta":0.08758550719143962,"training_matches":540}` |
| IEM Dallas 2025 | 802 | 22 | 22 | +0.0001 | `{"beta":0.09877041285617191,"training_matches":574}` |
| BLAST.tv Major Austin 2025 | 831 | 97 | 97 | -0.0000 | `{"beta":0.08508638437489224,"training_matches":596}` |
| FISSURE PLAYGROUND Season 1 2025 | 937 | 19 | 19 | +0.0010 | `{"beta":0.09201633062120051,"training_matches":693}` |
| IEM Cologne 2025 | 964 | 36 | 36 | +0.0001 | `{"beta":-0.0973147903924231,"training_matches":712}` |
| BLAST Bounty Fall 2025 | 1013 | 7 | 7 | +0.0001 | `{"beta":-0.060663104918965635,"training_matches":748}` |
| Esports World Cup 2025 | 1044 | 16 | 16 | +0.0001 | `{"beta":-0.05074775143683084,"training_matches":755}` |
| BLAST Open London Fall 2025 | 1060 | 4 | 4 | +0.0004 | `{"beta":-0.02280021105365121,"training_matches":771}` |
| FISSURE PLAYGROUND Season 2 2025 | 1089 | 37 | 37 | +0.0001 | `{"beta":0.03885376762771746,"training_matches":775}` |
| ESL Pro League Season 22 2025 | 1129 | 52 | 52 | +0.0000 | `{"beta":-0.05845215454636171,"training_matches":812}` |
| CS Asia Championships 2025 | 1202 | 23 | 23 | -0.0001 | `{"beta":-0.026611526005485102,"training_matches":864}` |
| Thunderpick World Championship 2025 | 1202 | 8 | 8 | +0.0002 | `{"beta":-0.026611526005485102,"training_matches":864}` |
| PGL Masters Bucharest 2025 | 1246 | 23 | 23 | +0.0001 | `{"beta":-0.0263356474043471,"training_matches":895}` |
| IEM Chengdu 2025 | 1287 | 30 | 30 | +0.0000 | `{"beta":0.016075767081751645,"training_matches":918}` |
| BLAST Rivals Fall 2025 | 1317 | 12 | 12 | +0.0000 | `{"beta":-0.02253398245364662,"training_matches":948}` |
| StarLadder Major Budapest 2025 | 1330 | 94 | 94 | +0.0000 | `{"beta":-0.0022493934345479798,"training_matches":960}` |
| BLAST Bounty Winter 2026 | 1436 | 3 | 3 | -0.0010 | `{"beta":0.1983578185938536,"training_matches":1054}` |
| IEM Krakow 2026 | 1467 | 4 | 4 | +0.0008 | `{"beta":0.21149181177491894,"training_matches":1057}` |
| PGL Cluj-Napoca 2026 | 1517 | 33 | 33 | -0.0000 | `{"beta":0.19877308735807736,"training_matches":1061}` |
| ESL Pro League Season 23 2026 | 1558 | 6 | 6 | +0.0000 | `{"beta":0.20480704667236865,"training_matches":1094}` |
| BLAST Open Rotterdam Spring 2026 | 1632 | 20 | 20 | -0.0008 | `{"beta":0.204336506681502,"training_matches":1100}` |
| PGL Bucharest 2026 | 1661 | 27 | 27 | -0.0007 | `{"beta":0.2692989604036946,"training_matches":1120}` |
| IEM Rio 2026 | 1700 | 24 | 24 | -0.0003 | `{"beta":0.3243969789906539,"training_matches":1147}` |
| BLAST Rivals Spring 2026 | 1730 | 12 | 12 | +0.0015 | `{"beta":0.34556160935170654,"training_matches":1171}` |
| PGL Astana 2026 | 1743 | 31 | 31 | -0.0004 | `{"beta":0.30026404823247854,"training_matches":1183}` |
| IEM Atlanta 2026 | 1743 | 22 | 22 | -0.0016 | `{"beta":0.30026404823247854,"training_matches":1183}` |
| CS Asia Championships 2026 | 1814 | 26 | 26 | -0.0022 | `{"beta":0.4310665779726034,"training_matches":1236}` |
| IEM Cologne Major 2026 | 1844 | 97 | 97 | -0.0004 | `{"beta":0.5409609091509864,"training_matches":1262}` |
| XSE Pro League 2026 | 1950 | 31 | 31 | -0.0024 | `{"beta":0.5999500106039162,"training_matches":1359}` |
| BLAST Bounty Summer 2026 | 1990 | 3 | 3 | -0.0036 | `{"beta":0.6982903786471855,"training_matches":1390}` |
| Esports World Cup 2026 | 2021 | 38 | 38 | +0.0052 | `{"beta":0.7107256301779441,"training_matches":1393}` |
| BLAST Open Porto Fall 2026 | 2077 | 25 | 25 | +0.0002 | `{"beta":0.484469745635412,"training_matches":1431}` |
| FISSURE PLAYGROUND Season 3 2026 | 2106 | 26 | 26 | +0.0002 | `{"beta":0.47431486382515653,"training_matches":1456}` |
| StarLadder StarSeries Fall 2026 | 2135 | 13 | 13 | +0.0007 | `{"beta":0.4640281717298027,"training_matches":1482}` |
| ESL Pro League Season 24 2026 | 2149 | 33 | 33 | -0.0009 | `{"beta":0.4489781085202941,"training_matches":1495}` |
| Thunderpick World Championship 2026 | 2190 | 13 | 13 | -0.0013 | `{"beta":0.49843655294918476,"training_matches":1528}` |
| PGL Masters Bucharest 2026 | 2204 | 19 | 19 | -0.0033 | `{"beta":0.5249210859207996,"training_matches":1541}` |
| IEM Beijing 2026 | 2245 | 26 | 26 | +0.0014 | `{"beta":0.6181914373786515,"training_matches":1560}` |
| BLAST Rivals Fall 2026 | 2275 | 11 | 11 | +0.0025 | `{"beta":0.5689349912261576,"training_matches":1586}` |

## `per_map_rating`: no evidence

Per-map Elo from completed Liquipedia series, averaged over equally likely map sets and blended with the market.

Verdict: better probabilities but fewer winners picked: the interval of the log-loss difference is below zero, that of the accuracy difference too.

| On the 2109 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.8099 | 0.2452 | 65.1% |
| per_map_rating | 0.6444 | 0.2267 | 63.4% |

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| BLAST Premier Groups Fall 2024 | 0 | 4 | 0 | — | — |
| IEM Cologne 2024 | 26 | 37 | 37 | -0.2345 | — |
| BLAST Premier Showdown Fall 2024 | 75 | 9 | 9 | -0.1950 | — |
| BetBoom Dacha Belgrade season 2 2024 | 75 | 22 | 22 | +0.1820 | — |
| ESL Pro League Season 20 2024 | 115 | 67 | 67 | -0.1958 | — |
| BLAST Premier Final Fall 2024 | 194 | 13 | 13 | +0.0864 | — |
| IEM Rio 2024 | 207 | 29 | 29 | -0.2727 | — |
| Thunderpick World Championship 2024 | 236 | 25 | 25 | -0.3505 | — |
| BLAST Premier World Final 2024 | 236 | 13 | 13 | -0.4677 | — |
| Perfect World Major Shanghai: American RMR 2024 | 277 | 27 | 27 | +0.0440 | — |
| Perfect World Major Shanghai: European RMR A 2024 | 311 | 24 | 24 | -0.0163 | — |
| Perfect World Major Shanghai: European RMR B 2024 | 346 | 29 | 29 | -0.5096 | — |
| Perfect World Major Shanghai Major 2024 | 380 | 73 | 73 | +0.0215 | — |
| BLAST Premier Bounty Closed Qualifier 2025 | 453 | 22 | 22 | -0.5740 | — |
| BLAST Premier Bounty season 1 2025 | 477 | 7 | 7 | -0.7584 | — |
| IEM Katowice 2025 | 484 | 48 | 47 | -0.2184 | — |
| PGL Cluj-Napoca 2025 | 533 | 41 | 38 | +0.0120 | — |
| ESL Pro League Season 21 2025 | 574 | 39 | 39 | +0.0393 | — |
| BLAST Open Lisbon Spring 2025 | 614 | 27 | 27 | -0.0273 | — |
| PGL Bucharest 2025 | 643 | 40 | 40 | -0.9821 | — |
| BLAST.tv Major Austin: European Regional Qualifier 2025 | 684 | 33 | 33 | -0.5009 | — |
| IEM Melbourne 2025 | 719 | 29 | 29 | -0.0846 | — |
| BLAST Rivals Spring 2025 | 748 | 13 | 13 | -0.1162 | — |
| PGL Astana 2025 | 761 | 39 | 39 | -1.1392 | — |
| IEM Dallas 2025 | 802 | 28 | 28 | +0.1160 | — |
| BLAST.tv Major Austin 2025 | 831 | 104 | 104 | -0.2555 | — |
| FISSURE PLAYGROUND Season 1 2025 | 937 | 27 | 27 | -0.7225 | — |
| IEM Cologne 2025 | 964 | 49 | 49 | +0.0817 | — |
| BLAST Bounty Fall 2025 | 1013 | 28 | 28 | +0.1501 | — |
| Esports World Cup 2025 | 1044 | 16 | 16 | -0.4718 | — |
| BLAST Open London Fall 2025 | 1060 | 27 | 27 | -0.5400 | — |
| FISSURE PLAYGROUND Season 2 2025 | 1089 | 40 | 40 | -0.2492 | — |
| ESL Pro League Season 22 2025 | 1129 | 71 | 71 | -0.7222 | — |
| CS Asia Championships 2025 | 1202 | 26 | 26 | -0.2482 | — |
| Thunderpick World Championship 2025 | 1202 | 12 | 12 | -0.0212 | — |
| PGL Masters Bucharest 2025 | 1246 | 41 | 41 | -0.2589 | — |
| IEM Chengdu 2025 | 1287 | 30 | 30 | -0.0062 | — |
| BLAST Rivals Fall 2025 | 1317 | 13 | 13 | -0.0197 | — |
| StarLadder Major Budapest 2025 | 1330 | 102 | 102 | -0.1623 | — |
| BLAST Bounty Winter 2026 | 1436 | 24 | 24 | -0.0268 | — |
| IEM Krakow 2026 | 1467 | 50 | 50 | -0.0213 | — |
| PGL Cluj-Napoca 2026 | 1517 | 41 | 41 | +0.0331 | — |
| ESL Pro League Season 23 2026 | 1558 | 60 | 60 | -0.0259 | — |
| BLAST Open Rotterdam Spring 2026 | 1632 | 29 | 29 | -0.0149 | — |
| PGL Bucharest 2026 | 1661 | 36 | 36 | +0.0333 | — |
| IEM Rio 2026 | 1700 | 30 | 30 | +0.0343 | — |
| BLAST Rivals Spring 2026 | 1730 | 13 | 13 | -0.0273 | — |
| PGL Astana 2026 | 1743 | 33 | 33 | -0.0204 | — |
| IEM Atlanta 2026 | 1743 | 30 | 30 | -0.0134 | — |
| CS Asia Championships 2026 | 1814 | 30 | 30 | -0.1406 | — |
| IEM Cologne Major 2026 | 1844 | 103 | 103 | +0.0194 | — |
| XSE Pro League 2026 | 1950 | 35 | 35 | -0.0172 | — |
| BLAST Bounty Summer 2026 | 1990 | 26 | 26 | -0.0611 | — |
| Esports World Cup 2026 | 2021 | 50 | 50 | +0.0234 | — |
| BLAST Open Porto Fall 2026 | 2077 | 28 | 28 | +0.0150 | — |
| FISSURE PLAYGROUND Season 3 2026 | 2106 | 28 | 28 | -0.0511 | — |
| StarLadder StarSeries Fall 2026 | 2135 | 14 | 14 | -0.0471 | — |
| ESL Pro League Season 24 2026 | 2149 | 35 | 35 | +0.0024 | — |
| Thunderpick World Championship 2026 | 2190 | 14 | 14 | +0.0013 | — |
| PGL Masters Bucharest 2026 | 2204 | 41 | 41 | -0.0964 | — |
| IEM Beijing 2026 | 2245 | 30 | 30 | -0.0584 | — |
| BLAST Rivals Fall 2026 | 2275 | 13 | 13 | +0.0145 | — |

## `bo1_veto_sim`: promote

Bo1 ban simulation from past HLTV vetoes and map results, blended with the market.

Verdict: the interval of the log-loss improvement excludes zero and the Matches with a market line are not worse.

| On the 319 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.7997 | 0.2690 | 61.1% |
| bo1_veto_sim | 0.7120 | 0.2516 | 62.1% |

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| BLAST Premier Groups Fall 2024 | 0 | 0 | 0 | — | — |
| IEM Cologne 2024 | 26 | 0 | 0 | — | — |
| BLAST Premier Showdown Fall 2024 | 75 | 0 | 0 | — | — |
| BetBoom Dacha Belgrade season 2 2024 | 75 | 0 | 0 | — | — |
| ESL Pro League Season 20 2024 | 115 | 0 | 0 | — | — |
| BLAST Premier Final Fall 2024 | 194 | 0 | 0 | — | — |
| IEM Rio 2024 | 207 | 3 | 3 | -0.1638 | — |
| Thunderpick World Championship 2024 | 236 | 0 | 0 | — | — |
| BLAST Premier World Final 2024 | 236 | 0 | 0 | — | — |
| Perfect World Major Shanghai: American RMR 2024 | 277 | 18 | 18 | +0.0463 | — |
| Perfect World Major Shanghai: European RMR A 2024 | 311 | 17 | 14 | -0.0260 | — |
| Perfect World Major Shanghai: European RMR B 2024 | 346 | 17 | 17 | -0.2798 | — |
| Perfect World Major Shanghai Major 2024 | 380 | 40 | 40 | -0.0189 | — |
| BLAST Premier Bounty Closed Qualifier 2025 | 453 | 0 | 0 | — | — |
| BLAST Premier Bounty season 1 2025 | 477 | 0 | 0 | — | — |
| IEM Katowice 2025 | 484 | 0 | 0 | — | — |
| PGL Cluj-Napoca 2025 | 533 | 0 | 0 | — | — |
| ESL Pro League Season 21 2025 | 574 | 0 | 0 | — | — |
| BLAST Open Lisbon Spring 2025 | 614 | 0 | 0 | — | — |
| PGL Bucharest 2025 | 643 | 0 | 0 | — | — |
| BLAST.tv Major Austin: European Regional Qualifier 2025 | 684 | 19 | 19 | -0.1668 | — |
| IEM Melbourne 2025 | 719 | 0 | 0 | — | — |
| BLAST Rivals Spring 2025 | 748 | 0 | 0 | — | — |
| PGL Astana 2025 | 761 | 0 | 0 | — | — |
| IEM Dallas 2025 | 802 | 0 | 0 | — | — |
| BLAST.tv Major Austin 2025 | 831 | 58 | 58 | -0.1900 | — |
| FISSURE PLAYGROUND Season 1 2025 | 937 | 0 | 0 | — | — |
| IEM Cologne 2025 | 964 | 0 | 0 | — | — |
| BLAST Bounty Fall 2025 | 1013 | 0 | 0 | — | — |
| Esports World Cup 2025 | 1044 | 0 | 0 | — | — |
| BLAST Open London Fall 2025 | 1060 | 0 | 0 | — | — |
| FISSURE PLAYGROUND Season 2 2025 | 1089 | 0 | 0 | — | — |
| ESL Pro League Season 22 2025 | 1129 | 0 | 0 | — | — |
| CS Asia Championships 2025 | 1202 | 8 | 8 | -0.0926 | — |
| Thunderpick World Championship 2025 | 1202 | 0 | 0 | — | — |
| PGL Masters Bucharest 2025 | 1246 | 0 | 0 | — | — |
| IEM Chengdu 2025 | 1287 | 0 | 0 | — | — |
| BLAST Rivals Fall 2025 | 1317 | 0 | 0 | — | — |
| StarLadder Major Budapest 2025 | 1330 | 60 | 60 | -0.1147 | — |
| BLAST Bounty Winter 2026 | 1436 | 0 | 0 | — | — |
| IEM Krakow 2026 | 1467 | 0 | 0 | — | — |
| PGL Cluj-Napoca 2026 | 1517 | 0 | 0 | — | — |
| ESL Pro League Season 23 2026 | 1558 | 0 | 0 | — | — |
| BLAST Open Rotterdam Spring 2026 | 1632 | 0 | 0 | — | — |
| PGL Bucharest 2026 | 1661 | 0 | 0 | — | — |
| IEM Rio 2026 | 1700 | 0 | 0 | — | — |
| BLAST Rivals Spring 2026 | 1730 | 0 | 0 | — | — |
| PGL Astana 2026 | 1743 | 0 | 0 | — | — |
| IEM Atlanta 2026 | 1743 | 0 | 0 | — | — |
| CS Asia Championships 2026 | 1814 | 8 | 8 | -0.0636 | — |
| IEM Cologne Major 2026 | 1844 | 40 | 40 | +0.0124 | — |
| XSE Pro League 2026 | 1950 | 19 | 18 | -0.0271 | — |
| BLAST Bounty Summer 2026 | 1990 | 0 | 0 | — | — |
| Esports World Cup 2026 | 2021 | 16 | 16 | -0.0079 | — |
| BLAST Open Porto Fall 2026 | 2077 | 0 | 0 | — | — |
| FISSURE PLAYGROUND Season 3 2026 | 2106 | 0 | 0 | — | — |
| StarLadder StarSeries Fall 2026 | 2135 | 0 | 0 | — | — |
| ESL Pro League Season 24 2026 | 2149 | 0 | 0 | — | — |
| Thunderpick World Championship 2026 | 2190 | 0 | 0 | — | — |
| PGL Masters Bucharest 2026 | 2204 | 0 | 0 | — | — |
| IEM Beijing 2026 | 2245 | 0 | 0 | — | — |
| BLAST Rivals Fall 2026 | 2275 | 0 | 0 | — | — |

## `bo3_veto_sim`: no evidence (added after list fixed)

Bo3 veto simulation from past HLTV vetoes and per-map Elo ratings, blended with the market.

Verdict: better probabilities but fewer winners picked: the interval of the log-loss difference is below zero, that of the accuracy difference too (added after spec #33's candidate list was fixed).

| On the 1850 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.8062 | 0.2386 | 66.2% |
| bo3_veto_sim | 0.6362 | 0.2230 | 63.2% |

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| BLAST Premier Groups Fall 2024 | 0 | 18 | 0 | — | — |
| IEM Cologne 2024 | 26 | 45 | 45 | -0.1465 | — |
| BLAST Premier Showdown Fall 2024 | 75 | 14 | 14 | -0.1479 | — |
| BetBoom Dacha Belgrade season 2 2024 | 75 | 23 | 23 | +0.0813 | — |
| ESL Pro League Season 20 2024 | 115 | 78 | 75 | -0.1999 | — |
| BLAST Premier Final Fall 2024 | 194 | 12 | 12 | +0.1288 | — |
| IEM Rio 2024 | 207 | 20 | 20 | -0.3025 | — |
| Thunderpick World Championship 2024 | 236 | 27 | 27 | -0.3045 | — |
| BLAST Premier World Final 2024 | 236 | 12 | 12 | -0.5275 | — |
| Perfect World Major Shanghai: American RMR 2024 | 277 | 14 | 14 | +0.0317 | — |
| Perfect World Major Shanghai: European RMR A 2024 | 311 | 15 | 13 | -0.1670 | — |
| Perfect World Major Shanghai: European RMR B 2024 | 346 | 14 | 14 | -0.4210 | — |
| Perfect World Major Shanghai Major 2024 | 380 | 33 | 33 | +0.0370 | — |
| BLAST Premier Bounty Closed Qualifier 2025 | 453 | 24 | 23 | -0.5315 | — |
| BLAST Premier Bounty season 1 2025 | 477 | 6 | 6 | -0.9750 | — |
| IEM Katowice 2025 | 484 | 48 | 46 | -0.2025 | — |
| PGL Cluj-Napoca 2025 | 533 | 40 | 37 | +0.0340 | — |
| ESL Pro League Season 21 2025 | 574 | 39 | 39 | +0.0342 | — |
| BLAST Open Lisbon Spring 2025 | 614 | 28 | 28 | +0.0035 | — |
| PGL Bucharest 2025 | 643 | 40 | 40 | -0.9827 | — |
| BLAST.tv Major Austin: European Regional Qualifier 2025 | 684 | 15 | 15 | -0.5999 | — |
| IEM Melbourne 2025 | 719 | 28 | 28 | -0.1081 | — |
| BLAST Rivals Spring 2025 | 748 | 12 | 12 | -0.1850 | — |
| PGL Astana 2025 | 761 | 40 | 40 | -1.1004 | — |
| IEM Dallas 2025 | 802 | 28 | 28 | +0.1028 | — |
| BLAST.tv Major Austin 2025 | 831 | 46 | 46 | -0.2166 | — |
| FISSURE PLAYGROUND Season 1 2025 | 937 | 26 | 26 | -0.7117 | — |
| IEM Cologne 2025 | 964 | 48 | 48 | +0.0868 | — |
| BLAST Bounty Fall 2025 | 1013 | 30 | 29 | +0.1431 | — |
| Esports World Cup 2025 | 1044 | 15 | 15 | -0.4907 | — |
| BLAST Open London Fall 2025 | 1060 | 28 | 28 | -0.3428 | — |
| FISSURE PLAYGROUND Season 2 2025 | 1089 | 39 | 39 | -0.1917 | — |
| ESL Pro League Season 22 2025 | 1129 | 72 | 72 | -0.8857 | — |
| CS Asia Championships 2025 | 1202 | 21 | 21 | -0.1766 | — |
| Thunderpick World Championship 2025 | 1202 | 13 | 13 | -0.1525 | — |
| PGL Masters Bucharest 2025 | 1246 | 40 | 40 | -0.2925 | — |
| IEM Chengdu 2025 | 1287 | 29 | 29 | -0.0417 | — |
| BLAST Rivals Fall 2025 | 1317 | 12 | 12 | -0.0667 | — |
| StarLadder Major Budapest 2025 | 1330 | 45 | 45 | -0.1029 | — |
| BLAST Bounty Winter 2026 | 1436 | 30 | 30 | -0.0379 | — |
| IEM Krakow 2026 | 1467 | 49 | 49 | -0.0229 | — |
| PGL Cluj-Napoca 2026 | 1517 | 40 | 40 | +0.0350 | — |
| ESL Pro League Season 23 2026 | 1558 | 73 | 73 | -0.0279 | — |
| BLAST Open Rotterdam Spring 2026 | 1632 | 28 | 28 | -0.0215 | — |
| PGL Bucharest 2026 | 1661 | 38 | 38 | +0.0162 | — |
| IEM Rio 2026 | 1700 | 29 | 29 | +0.0258 | — |
| BLAST Rivals Spring 2026 | 1730 | 12 | 12 | -0.0287 | — |
| PGL Astana 2026 | 1743 | 40 | 40 | +0.0031 | — |
| IEM Atlanta 2026 | 1743 | 29 | 29 | -0.0121 | — |
| CS Asia Championships 2026 | 1814 | 21 | 21 | -0.1280 | — |
| IEM Cologne Major 2026 | 1844 | 65 | 65 | +0.0091 | — |
| XSE Pro League 2026 | 1950 | 19 | 18 | -0.0091 | — |
| BLAST Bounty Summer 2026 | 1990 | 30 | 29 | -0.0728 | — |
| Esports World Cup 2026 | 2021 | 39 | 39 | +0.0178 | — |
| BLAST Open Porto Fall 2026 | 2077 | 28 | 28 | +0.0128 | — |
| FISSURE PLAYGROUND Season 3 2026 | 2106 | 28 | 28 | -0.0483 | — |
| StarLadder StarSeries Fall 2026 | 2135 | 13 | 13 | -0.0564 | — |
| ESL Pro League Season 24 2026 | 2149 | 40 | 40 | -0.0420 | — |
| Thunderpick World Championship 2026 | 2190 | 13 | 13 | +0.0022 | — |
| PGL Masters Bucharest 2026 | 2204 | 40 | 40 | -0.0985 | — |
| IEM Beijing 2026 | 2245 | 29 | 29 | -0.0664 | — |
| BLAST Rivals Fall 2026 | 2275 | 12 | 12 | +0.0134 | — |

## `roster_stand_in`: no evidence

A stand-in in the active five, from the pre-cutoff Liquipedia revision..

Verdict: the interval of the log-loss difference includes zero.

| On the 2056 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.8047 | 0.2429 | 65.9% |
| roster_stand_in | 0.8045 | 0.2414 | 66.2% |

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| BLAST Premier Groups Fall 2024 | 0 | 26 | 0 | — | — |
| IEM Cologne 2024 | 26 | 48 | 48 | +0.0000 | — |
| BLAST Premier Showdown Fall 2024 | 75 | 14 | 14 | +0.0000 | — |
| BetBoom Dacha Belgrade season 2 2024 | 75 | 26 | 26 | +0.0000 | — |
| ESL Pro League Season 20 2024 | 115 | 76 | 76 | +0.0000 | — |
| BLAST Premier Final Fall 2024 | 194 | 13 | 13 | +0.0000 | — |
| IEM Rio 2024 | 207 | 29 | 29 | +0.0000 | — |
| Thunderpick World Championship 2024 | 236 | 28 | 28 | +0.0000 | — |
| BLAST Premier World Final 2024 | 236 | 13 | 13 | +0.0000 | — |
| Perfect World Major Shanghai: American RMR 2024 | 277 | 34 | 34 | +0.0000 | — |
| Perfect World Major Shanghai: European RMR A 2024 | 311 | 35 | 30 | -0.0298 | — |
| Perfect World Major Shanghai: European RMR B 2024 | 346 | 34 | 34 | +0.0208 | — |
| Perfect World Major Shanghai Major 2024 | 380 | 73 | 73 | -0.0111 | — |
| BLAST Premier Bounty Closed Qualifier 2025 | 453 | 24 | 23 | +0.0000 | — |
| BLAST Premier Bounty season 1 2025 | 477 | 7 | 7 | +0.0000 | — |
| IEM Katowice 2025 | 484 | 49 | 47 | +0.0000 | — |
| PGL Cluj-Napoca 2025 | 533 | 41 | 38 | +0.0000 | — |
| ESL Pro League Season 21 2025 | 574 | 40 | 40 | +0.0000 | — |
| BLAST Open Lisbon Spring 2025 | 614 | 29 | 29 | +0.0000 | — |
| PGL Bucharest 2025 | 643 | 35 | 35 | +0.0000 | — |
| BLAST.tv Major Austin: European Regional Qualifier 2025 | 684 | 35 | 35 | +0.0000 | — |
| IEM Melbourne 2025 | 719 | 28 | 28 | +0.0000 | — |
| BLAST Rivals Spring 2025 | 748 | 13 | 13 | +0.0000 | — |
| PGL Astana 2025 | 761 | 41 | 41 | +0.0000 | — |
| IEM Dallas 2025 | 802 | 29 | 29 | +0.0000 | — |
| BLAST.tv Major Austin 2025 | 831 | 106 | 106 | +0.0131 | — |
| FISSURE PLAYGROUND Season 1 2025 | 937 | 27 | 27 | +0.0000 | — |
| IEM Cologne 2025 | 964 | 49 | 49 | +0.0000 | — |
| BLAST Bounty Fall 2025 | 1013 | 7 | 7 | +0.0000 | — |
| Esports World Cup 2025 | 1044 | 16 | 16 | +0.0000 | — |
| BLAST Open London Fall 2025 | 1060 | 7 | 7 | +0.0000 | — |
| FISSURE PLAYGROUND Season 2 2025 | 1089 | 40 | 40 | +0.0000 | — |
| ESL Pro League Season 22 2025 | 1129 | 65 | 65 | +0.0000 | — |
| CS Asia Championships 2025 | 1202 | 30 | 30 | +0.0000 | — |
| Thunderpick World Championship 2025 | 1202 | 14 | 14 | +0.0000 | — |
| PGL Masters Bucharest 2025 | 1246 | 41 | 41 | +0.0000 | — |
| IEM Chengdu 2025 | 1287 | 30 | 30 | +0.0000 | — |
| BLAST Rivals Fall 2025 | 1317 | 13 | 13 | +0.0000 | — |
| StarLadder Major Budapest 2025 | 1330 | 106 | 106 | -0.0020 | — |
| BLAST Bounty Winter 2026 | 1436 | 7 | 7 | +0.0000 | — |
| IEM Krakow 2026 | 1467 | 50 | 50 | +0.0000 | — |
| PGL Cluj-Napoca 2026 | 1517 | 41 | 41 | +0.0000 | — |
| ESL Pro League Season 23 2026 | 1558 | 8 | 8 | +0.0000 | — |
| BLAST Open Rotterdam Spring 2026 | 1632 | 24 | 24 | +0.0000 | — |
| PGL Bucharest 2026 | 1661 | 39 | 39 | +0.0000 | — |
| IEM Rio 2026 | 1700 | 30 | 30 | +0.0000 | — |
| BLAST Rivals Spring 2026 | 1730 | 13 | 13 | +0.0000 | — |
| PGL Astana 2026 | 1743 | 41 | 41 | +0.0000 | — |
| IEM Atlanta 2026 | 1743 | 30 | 30 | +0.0000 | — |
| CS Asia Championships 2026 | 1814 | 30 | 30 | +0.0000 | — |
| IEM Cologne Major 2026 | 1844 | 106 | 106 | -0.0058 | — |
| XSE Pro League 2026 | 1950 | 39 | 37 | +0.0000 | — |
| BLAST Bounty Summer 2026 | 1990 | 7 | 7 | +0.0000 | — |
| Esports World Cup 2026 | 2021 | 56 | 56 | +0.0000 | — |
| BLAST Open Porto Fall 2026 | 2077 | 26 | 26 | +0.0000 | — |
| FISSURE PLAYGROUND Season 3 2026 | 2106 | 29 | 29 | +0.0000 | — |
| StarLadder StarSeries Fall 2026 | 2135 | 14 | 14 | +0.0000 | — |
| ESL Pro League Season 24 2026 | 2149 | 36 | 36 | +0.0000 | — |
| Thunderpick World Championship 2026 | 2190 | 14 | 14 | +0.0000 | — |
| PGL Masters Bucharest 2026 | 2204 | 41 | 41 | +0.0000 | — |
| IEM Beijing 2026 | 2245 | 30 | 30 | +0.0000 | — |
| BLAST Rivals Fall 2026 | 2275 | 13 | 13 | +0.0000 | — |

## `roster_change_30d`: harmful

An active-five change bounded to the last 30 days by dated lineup observations..

Verdict: the interval of the log-loss difference is above zero: worse than the anchor.

| On the 1563 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.7889 | 0.2429 | 65.5% |
| roster_change_30d | 0.7934 | 0.2434 | 65.5% |

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| BLAST Premier Groups Fall 2024 | 0 | 0 | 0 | — | — |
| IEM Cologne 2024 | 26 | 2 | 2 | +0.0000 | — |
| BLAST Premier Showdown Fall 2024 | 75 | 2 | 2 | +0.0000 | — |
| BetBoom Dacha Belgrade season 2 2024 | 75 | 0 | 0 | — | — |
| ESL Pro League Season 20 2024 | 115 | 54 | 54 | +0.0000 | — |
| BLAST Premier Final Fall 2024 | 194 | 13 | 13 | -0.0017 | — |
| IEM Rio 2024 | 207 | 29 | 29 | +0.0584 | — |
| Thunderpick World Championship 2024 | 236 | 22 | 22 | +0.0805 | — |
| BLAST Premier World Final 2024 | 236 | 12 | 12 | +0.0680 | — |
| Perfect World Major Shanghai: American RMR 2024 | 277 | 33 | 33 | -0.0011 | — |
| Perfect World Major Shanghai: European RMR A 2024 | 311 | 27 | 27 | +0.0167 | — |
| Perfect World Major Shanghai: European RMR B 2024 | 346 | 34 | 34 | -0.0043 | — |
| Perfect World Major Shanghai Major 2024 | 380 | 73 | 73 | +0.0020 | — |
| BLAST Premier Bounty Closed Qualifier 2025 | 453 | 15 | 15 | +0.0363 | — |
| BLAST Premier Bounty season 1 2025 | 477 | 7 | 7 | +0.0004 | — |
| IEM Katowice 2025 | 484 | 45 | 45 | -0.0004 | — |
| PGL Cluj-Napoca 2025 | 533 | 28 | 28 | -0.0102 | — |
| ESL Pro League Season 21 2025 | 574 | 28 | 28 | -0.0021 | — |
| BLAST Open Lisbon Spring 2025 | 614 | 21 | 21 | +0.0254 | — |
| PGL Bucharest 2025 | 643 | 25 | 25 | +0.0100 | — |
| BLAST.tv Major Austin: European Regional Qualifier 2025 | 684 | 27 | 27 | +0.0085 | — |
| IEM Melbourne 2025 | 719 | 24 | 24 | +0.0077 | — |
| BLAST Rivals Spring 2025 | 748 | 13 | 13 | -0.0125 | — |
| PGL Astana 2025 | 761 | 32 | 32 | +0.0063 | — |
| IEM Dallas 2025 | 802 | 17 | 17 | +0.0021 | — |
| BLAST.tv Major Austin 2025 | 831 | 80 | 80 | +0.0022 | — |
| FISSURE PLAYGROUND Season 1 2025 | 937 | 25 | 25 | +0.0006 | — |
| IEM Cologne 2025 | 964 | 49 | 49 | -0.0007 | — |
| BLAST Bounty Fall 2025 | 1013 | 4 | 4 | -0.0145 | — |
| Esports World Cup 2025 | 1044 | 7 | 7 | -0.0064 | — |
| BLAST Open London Fall 2025 | 1060 | 7 | 7 | +0.0127 | — |
| FISSURE PLAYGROUND Season 2 2025 | 1089 | 36 | 36 | +0.0114 | — |
| ESL Pro League Season 22 2025 | 1129 | 41 | 41 | +0.0023 | — |
| CS Asia Championships 2025 | 1202 | 18 | 18 | -0.0000 | — |
| Thunderpick World Championship 2025 | 1202 | 10 | 10 | -0.0013 | — |
| PGL Masters Bucharest 2025 | 1246 | 32 | 32 | -0.0074 | — |
| IEM Chengdu 2025 | 1287 | 24 | 24 | +0.0014 | — |
| BLAST Rivals Fall 2025 | 1317 | 13 | 13 | -0.0198 | — |
| StarLadder Major Budapest 2025 | 1330 | 82 | 82 | -0.0009 | — |
| BLAST Bounty Winter 2026 | 1436 | 7 | 7 | +0.0482 | — |
| IEM Krakow 2026 | 1467 | 44 | 44 | +0.0048 | — |
| PGL Cluj-Napoca 2026 | 1517 | 23 | 23 | +0.0030 | — |
| ESL Pro League Season 23 2026 | 1558 | 7 | 7 | -0.0124 | — |
| BLAST Open Rotterdam Spring 2026 | 1632 | 16 | 16 | +0.0079 | — |
| PGL Bucharest 2026 | 1661 | 12 | 12 | -0.0019 | — |
| IEM Rio 2026 | 1700 | 21 | 21 | -0.0046 | — |
| BLAST Rivals Spring 2026 | 1730 | 10 | 10 | +0.0011 | — |
| PGL Astana 2026 | 1743 | 41 | 41 | -0.0027 | — |
| IEM Atlanta 2026 | 1743 | 30 | 30 | -0.0014 | — |
| CS Asia Championships 2026 | 1814 | 28 | 28 | -0.0163 | — |
| IEM Cologne Major 2026 | 1844 | 96 | 96 | -0.0007 | — |
| XSE Pro League 2026 | 1950 | 16 | 16 | +0.0037 | — |
| BLAST Bounty Summer 2026 | 1990 | 7 | 7 | +0.0163 | — |
| Esports World Cup 2026 | 2021 | 36 | 36 | +0.0156 | — |
| BLAST Open Porto Fall 2026 | 2077 | 16 | 16 | +0.0019 | — |
| FISSURE PLAYGROUND Season 3 2026 | 2106 | 11 | 11 | -0.0068 | — |
| StarLadder StarSeries Fall 2026 | 2135 | 14 | 14 | -0.0001 | — |
| ESL Pro League Season 24 2026 | 2149 | 25 | 25 | +0.0056 | — |
| Thunderpick World Championship 2026 | 2190 | 12 | 12 | -0.0062 | — |
| PGL Masters Bucharest 2026 | 2204 | 41 | 41 | -0.0010 | — |
| IEM Beijing 2026 | 2245 | 26 | 26 | +0.0113 | — |
| BLAST Rivals Fall 2026 | 2275 | 13 | 13 | -0.0123 | — |

## `roster_stability`: no evidence

Days since the current five was first observed after a different five..

Verdict: the interval of the log-loss difference includes zero.

| On the 1534 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.7857 | 0.2432 | 65.2% |
| roster_stability | 0.7902 | 0.2449 | 65.3% |

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| BLAST Premier Groups Fall 2024 | 0 | 0 | 0 | — | — |
| IEM Cologne 2024 | 26 | 2 | 2 | +0.0000 | — |
| BLAST Premier Showdown Fall 2024 | 75 | 2 | 2 | +0.0000 | — |
| BetBoom Dacha Belgrade season 2 2024 | 75 | 0 | 0 | — | — |
| ESL Pro League Season 20 2024 | 115 | 4 | 4 | -0.0000 | — |
| BLAST Premier Final Fall 2024 | 194 | 0 | 0 | — | — |
| IEM Rio 2024 | 207 | 1 | 1 | +0.0768 | — |
| Thunderpick World Championship 2024 | 236 | 9 | 9 | -0.0211 | — |
| BLAST Premier World Final 2024 | 236 | 0 | 0 | — | — |
| Perfect World Major Shanghai: American RMR 2024 | 277 | 1 | 1 | -0.1371 | — |
| Perfect World Major Shanghai: European RMR A 2024 | 311 | 13 | 13 | -0.0572 | — |
| Perfect World Major Shanghai: European RMR B 2024 | 346 | 6 | 6 | -0.2538 | — |
| Perfect World Major Shanghai Major 2024 | 380 | 8 | 8 | +0.1881 | — |
| BLAST Premier Bounty Closed Qualifier 2025 | 453 | 12 | 12 | +0.1925 | — |
| BLAST Premier Bounty season 1 2025 | 477 | 3 | 3 | -0.0163 | — |
| IEM Katowice 2025 | 484 | 30 | 30 | -0.0013 | — |
| PGL Cluj-Napoca 2025 | 533 | 38 | 38 | +0.0420 | — |
| ESL Pro League Season 21 2025 | 574 | 26 | 26 | -0.0028 | — |
| BLAST Open Lisbon Spring 2025 | 614 | 19 | 19 | +0.0080 | — |
| PGL Bucharest 2025 | 643 | 29 | 29 | +0.0078 | — |
| BLAST.tv Major Austin: European Regional Qualifier 2025 | 684 | 31 | 31 | -0.0211 | — |
| IEM Melbourne 2025 | 719 | 24 | 24 | +0.0254 | — |
| BLAST Rivals Spring 2025 | 748 | 9 | 9 | +0.0558 | — |
| PGL Astana 2025 | 761 | 30 | 30 | +0.0062 | — |
| IEM Dallas 2025 | 802 | 29 | 29 | -0.0104 | — |
| BLAST.tv Major Austin 2025 | 831 | 86 | 86 | -0.0141 | — |
| FISSURE PLAYGROUND Season 1 2025 | 937 | 27 | 27 | -0.0129 | — |
| IEM Cologne 2025 | 964 | 49 | 49 | +0.0419 | — |
| BLAST Bounty Fall 2025 | 1013 | 7 | 7 | +0.0684 | — |
| Esports World Cup 2025 | 1044 | 16 | 16 | +0.0391 | — |
| BLAST Open London Fall 2025 | 1060 | 7 | 7 | +0.0199 | — |
| FISSURE PLAYGROUND Season 2 2025 | 1089 | 40 | 40 | -0.0039 | — |
| ESL Pro League Season 22 2025 | 1129 | 57 | 57 | +0.0094 | — |
| CS Asia Championships 2025 | 1202 | 30 | 30 | -0.0006 | — |
| Thunderpick World Championship 2025 | 1202 | 11 | 11 | -0.0111 | — |
| PGL Masters Bucharest 2025 | 1246 | 41 | 41 | -0.0163 | — |
| IEM Chengdu 2025 | 1287 | 27 | 27 | +0.0060 | — |
| BLAST Rivals Fall 2025 | 1317 | 13 | 13 | -0.0147 | — |
| StarLadder Major Budapest 2025 | 1330 | 106 | 106 | +0.0112 | — |
| BLAST Bounty Winter 2026 | 1436 | 7 | 7 | +0.0462 | — |
| IEM Krakow 2026 | 1467 | 49 | 49 | -0.0008 | — |
| PGL Cluj-Napoca 2026 | 1517 | 41 | 41 | -0.0011 | — |
| ESL Pro League Season 23 2026 | 1558 | 8 | 8 | -0.0077 | — |
| BLAST Open Rotterdam Spring 2026 | 1632 | 24 | 24 | +0.0091 | — |
| PGL Bucharest 2026 | 1661 | 33 | 33 | -0.0040 | — |
| IEM Rio 2026 | 1700 | 30 | 30 | -0.0048 | — |
| BLAST Rivals Spring 2026 | 1730 | 13 | 13 | +0.0107 | — |
| PGL Astana 2026 | 1743 | 41 | 41 | +0.0001 | — |
| IEM Atlanta 2026 | 1743 | 30 | 30 | +0.0098 | — |
| CS Asia Championships 2026 | 1814 | 30 | 30 | -0.0070 | — |
| IEM Cologne Major 2026 | 1844 | 99 | 99 | +0.0033 | — |
| XSE Pro League 2026 | 1950 | 37 | 37 | +0.0028 | — |
| BLAST Bounty Summer 2026 | 1990 | 7 | 7 | +0.0032 | — |
| Esports World Cup 2026 | 2021 | 56 | 56 | +0.0008 | — |
| BLAST Open Porto Fall 2026 | 2077 | 24 | 24 | +0.0018 | — |
| FISSURE PLAYGROUND Season 3 2026 | 2106 | 23 | 23 | +0.0006 | — |
| StarLadder StarSeries Fall 2026 | 2135 | 14 | 14 | -0.0004 | — |
| ESL Pro League Season 24 2026 | 2149 | 34 | 34 | -0.0017 | — |
| Thunderpick World Championship 2026 | 2190 | 14 | 14 | +0.0063 | — |
| PGL Masters Bucharest 2026 | 2204 | 37 | 37 | +0.0001 | — |
| IEM Beijing 2026 | 2245 | 27 | 27 | -0.0020 | — |
| BLAST Rivals Fall 2026 | 2275 | 13 | 13 | +0.0111 | — |

## `roster_coach_change`: harmful

An emergency coach substitute or a change from the previously observed active coach set..

Verdict: the interval of the log-loss difference is above zero: worse than the anchor.

| On the 1903 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.7855 | 0.2399 | 65.9% |
| roster_coach_change | 0.7896 | 0.2410 | 65.5% |

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| BLAST Premier Groups Fall 2024 | 0 | 21 | 0 | — | — |
| IEM Cologne 2024 | 26 | 40 | 40 | +0.0000 | — |
| BLAST Premier Showdown Fall 2024 | 75 | 11 | 11 | +0.0000 | — |
| BetBoom Dacha Belgrade season 2 2024 | 75 | 23 | 23 | +0.0000 | — |
| ESL Pro League Season 20 2024 | 115 | 64 | 64 | +0.0000 | — |
| BLAST Premier Final Fall 2024 | 194 | 13 | 13 | +0.0000 | — |
| IEM Rio 2024 | 207 | 29 | 29 | +0.0000 | — |
| Thunderpick World Championship 2024 | 236 | 28 | 28 | +0.0000 | — |
| BLAST Premier World Final 2024 | 236 | 13 | 13 | +0.0000 | — |
| Perfect World Major Shanghai: American RMR 2024 | 277 | 28 | 28 | +0.0000 | — |
| Perfect World Major Shanghai: European RMR A 2024 | 311 | 24 | 21 | +0.0096 | — |
| Perfect World Major Shanghai: European RMR B 2024 | 346 | 25 | 25 | -0.0100 | — |
| Perfect World Major Shanghai Major 2024 | 380 | 73 | 73 | +0.0000 | — |
| BLAST Premier Bounty Closed Qualifier 2025 | 453 | 24 | 23 | +0.0352 | — |
| BLAST Premier Bounty season 1 2025 | 477 | 6 | 6 | -0.0000 | — |
| IEM Katowice 2025 | 484 | 47 | 47 | +0.0000 | — |
| PGL Cluj-Napoca 2025 | 533 | 41 | 38 | +0.0000 | — |
| ESL Pro League Season 21 2025 | 574 | 40 | 40 | +0.0000 | — |
| BLAST Open Lisbon Spring 2025 | 614 | 28 | 28 | +0.0000 | — |
| PGL Bucharest 2025 | 643 | 33 | 33 | +0.0239 | — |
| BLAST.tv Major Austin: European Regional Qualifier 2025 | 684 | 0 | 0 | — | — |
| IEM Melbourne 2025 | 719 | 26 | 26 | +0.0002 | — |
| BLAST Rivals Spring 2025 | 748 | 13 | 13 | +0.0000 | — |
| PGL Astana 2025 | 761 | 33 | 33 | +0.0163 | — |
| IEM Dallas 2025 | 802 | 28 | 28 | +0.0000 | — |
| BLAST.tv Major Austin 2025 | 831 | 105 | 105 | +0.0030 | — |
| FISSURE PLAYGROUND Season 1 2025 | 937 | 27 | 27 | +0.0003 | — |
| IEM Cologne 2025 | 964 | 49 | 49 | -0.0013 | — |
| BLAST Bounty Fall 2025 | 1013 | 7 | 7 | +0.0000 | — |
| Esports World Cup 2025 | 1044 | 15 | 15 | +0.0000 | — |
| BLAST Open London Fall 2025 | 1060 | 7 | 7 | +0.0000 | — |
| FISSURE PLAYGROUND Season 2 2025 | 1089 | 40 | 40 | +0.0000 | — |
| ESL Pro League Season 22 2025 | 1129 | 61 | 61 | -0.0040 | — |
| CS Asia Championships 2025 | 1202 | 29 | 29 | -0.0033 | — |
| Thunderpick World Championship 2025 | 1202 | 13 | 13 | +0.0000 | — |
| PGL Masters Bucharest 2025 | 1246 | 39 | 39 | -0.0017 | — |
| IEM Chengdu 2025 | 1287 | 30 | 30 | -0.0002 | — |
| BLAST Rivals Fall 2025 | 1317 | 13 | 13 | -0.0370 | — |
| StarLadder Major Budapest 2025 | 1330 | 100 | 100 | +0.0590 | — |
| BLAST Bounty Winter 2026 | 1436 | 7 | 7 | +0.0000 | — |
| IEM Krakow 2026 | 1467 | 50 | 50 | +0.0000 | — |
| PGL Cluj-Napoca 2026 | 1517 | 41 | 41 | +0.0000 | — |
| ESL Pro League Season 23 2026 | 1558 | 7 | 7 | +0.0000 | — |
| BLAST Open Rotterdam Spring 2026 | 1632 | 19 | 19 | -0.0055 | — |
| PGL Bucharest 2026 | 1661 | 37 | 37 | -0.0044 | — |
| IEM Rio 2026 | 1700 | 30 | 30 | +0.0018 | — |
| BLAST Rivals Spring 2026 | 1730 | 12 | 12 | -0.0159 | — |
| PGL Astana 2026 | 1743 | 36 | 36 | +0.0055 | — |
| IEM Atlanta 2026 | 1743 | 30 | 30 | -0.0043 | — |
| CS Asia Championships 2026 | 1814 | 30 | 30 | +0.0171 | — |
| IEM Cologne Major 2026 | 1844 | 96 | 96 | +0.0058 | — |
| XSE Pro League 2026 | 1950 | 37 | 36 | +0.0000 | — |
| BLAST Bounty Summer 2026 | 1990 | 6 | 6 | +0.0000 | — |
| Esports World Cup 2026 | 2021 | 50 | 50 | -0.0018 | — |
| BLAST Open Porto Fall 2026 | 2077 | 25 | 25 | +0.0000 | — |
| FISSURE PLAYGROUND Season 3 2026 | 2106 | 29 | 29 | +0.0000 | — |
| StarLadder StarSeries Fall 2026 | 2135 | 12 | 12 | +0.0000 | — |
| ESL Pro League Season 24 2026 | 2149 | 35 | 35 | +0.0000 | — |
| Thunderpick World Championship 2026 | 2190 | 14 | 14 | +0.0000 | — |
| PGL Masters Bucharest 2026 | 2204 | 41 | 41 | -0.0043 | — |
| IEM Beijing 2026 | 2245 | 30 | 30 | +0.0028 | — |
| BLAST Rivals Fall 2026 | 2275 | 12 | 12 | -0.0023 | — |

## `rest_schedule_load`: no evidence

Team differences in rest, Matches and maps in the preceding 48 hours, and a second Match on the same UTC day.

Verdict: the interval of the log-loss difference includes zero.

| On the 2128 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.8007 | 0.2433 | 65.4% |
| rest_schedule_load | 0.8006 | 0.2437 | 65.4% |

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| BLAST Premier Groups Fall 2024 | 0 | 0 | 0 | — | — |
| IEM Cologne 2024 | 26 | 0 | 0 | — | — |
| BLAST Premier Showdown Fall 2024 | 75 | 9 | 9 | -0.0243 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[-0.21226499728382026,0.07060646348559473,0.0534345831532911,0.0]}` |
| BetBoom Dacha Belgrade season 2 2024 | 75 | 23 | 23 | -0.0159 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[-0.21226499728382026,0.07060646348559473,0.0534345831532911,0.0]}` |
| ESL Pro League Season 20 2024 | 115 | 68 | 68 | +0.0121 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[-0.2247738450662309,0.07156202653475736,0.04148971019399283,0.17079399799685382]}` |
| BLAST Premier Final Fall 2024 | 194 | 13 | 13 | -0.0087 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.02915057689642207,0.17540708572815095,0.14328184833923102,0.20220004286217197]}` |
| IEM Rio 2024 | 207 | 29 | 29 | -0.0052 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.041380541512377,0.204537058339896,0.1316633363614767,0.20230572174996703]}` |
| Thunderpick World Championship 2024 | 236 | 26 | 26 | +0.0102 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.0724610715122746,0.20782157259118902,0.17008160819452914,0.20449811745097687]}` |
| BLAST Premier World Final 2024 | 236 | 13 | 13 | +0.0115 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.0724610715122746,0.20782157259118902,0.17008160819452914,0.20449811745097687]}` |
| Perfect World Major Shanghai: American RMR 2024 | 277 | 30 | 30 | +0.0042 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.18297284313576986,0.17574366590256318,0.15972842900155496,0.23109145919710572]}` |
| Perfect World Major Shanghai: European RMR A 2024 | 311 | 26 | 26 | -0.0059 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.17217871654943498,0.15027749499903367,0.15830051441045787,0.232127292860017]}` |
| Perfect World Major Shanghai: European RMR B 2024 | 346 | 29 | 29 | -0.0021 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.15565147888333872,0.14944048876482785,0.20192513531013576,0.22498870645587812]}` |
| Perfect World Major Shanghai Major 2024 | 380 | 73 | 73 | -0.0005 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.15438166617795981,0.12221355371561839,0.2369773339394402,0.22460963581360727]}` |
| BLAST Premier Bounty Closed Qualifier 2025 | 453 | 22 | 22 | +0.0056 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.1360411346742145,0.14053057023847587,0.2203606860737583,0.22047422176914905]}` |
| BLAST Premier Bounty season 1 2025 | 477 | 7 | 7 | +0.0148 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.10505169344853303,0.13164377140661263,0.21273054981198064,0.21615256970019667]}` |
| IEM Katowice 2025 | 484 | 47 | 47 | +0.0056 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.08646359864577749,0.12278658812208296,0.20325602247496255,0.2144242964957965]}` |
| PGL Cluj-Napoca 2025 | 533 | 38 | 38 | -0.0163 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[-0.04250118141579205,0.08348621955763118,0.17601774394206787,0.19526438358441275]}` |
| ESL Pro League Season 21 2025 | 574 | 39 | 39 | -0.0055 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[-0.020189955304426865,0.04967038772675234,0.30579849087690997,0.19104803001055806]}` |
| BLAST Open Lisbon Spring 2025 | 614 | 28 | 28 | -0.0117 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[-0.026339622276877816,0.08540386161673427,0.28597760023895935,0.18723974208401567]}` |
| PGL Bucharest 2025 | 643 | 40 | 40 | +0.0041 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[-0.02133592943804901,0.10066704625679919,0.300421861905017,0.18489594527329492]}` |
| BLAST.tv Major Austin: European Regional Qualifier 2025 | 684 | 33 | 33 | -0.0466 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.19014879788888425,0.11492140352598043,0.3895014181834625,0.21782888171984166]}` |
| IEM Melbourne 2025 | 719 | 29 | 29 | +0.0051 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.30651655284628865,0.11210906056932347,0.47356161299361915,0.3225777007828287]}` |
| BLAST Rivals Spring 2025 | 748 | 13 | 13 | +0.0677 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.27892295696474256,0.13095282048409732,0.4569333533567594,0.3166990569473396]}` |
| PGL Astana 2025 | 761 | 39 | 39 | -0.0064 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.2556567960725577,0.10082035919303056,0.4188260185741566,0.3192981034053421]}` |
| IEM Dallas 2025 | 802 | 28 | 28 | -0.0032 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.2622611753884471,0.08103214659041445,0.4456330752824428,0.3203531873334777]}` |
| BLAST.tv Major Austin 2025 | 831 | 105 | 105 | +0.0275 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.30220843329506064,0.04294027376819113,0.4388561859517575,0.39840043877777387]}` |
| FISSURE PLAYGROUND Season 1 2025 | 937 | 27 | 27 | -0.0195 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.06683624153676539,0.11201263738314587,0.3598669470533139,0.24944361844672447]}` |
| IEM Cologne 2025 | 964 | 49 | 49 | -0.0062 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.09834931336866366,0.11884574634507605,0.3942155708841271,0.25184289677984384]}` |
| BLAST Bounty Fall 2025 | 1013 | 28 | 28 | +0.0001 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.1903770226935919,0.10204099315033285,0.44027776481522934,0.26776360632417107]}` |
| Esports World Cup 2025 | 1044 | 16 | 16 | -0.0408 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.19223405486432324,0.10218078669882337,0.4370248311670812,0.26840157353755334]}` |
| BLAST Open London Fall 2025 | 1060 | 29 | 29 | -0.0196 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.23073785038428318,0.13842672824699948,0.4387260518077711,0.2715857249883209]}` |
| FISSURE PLAYGROUND Season 2 2025 | 1089 | 40 | 40 | +0.0282 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.23248671618095634,0.15249399244964257,0.45853293398163525,0.26858940648657365]}` |
| ESL Pro League Season 22 2025 | 1129 | 71 | 71 | -0.0166 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.17586248357332915,0.19182198865326233,0.36401650189147,0.26178515956958953]}` |
| CS Asia Championships 2025 | 1202 | 30 | 30 | -0.0056 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.19967554410134913,0.17682512507787593,0.4225018975819397,0.2627978819884555]}` |
| Thunderpick World Championship 2025 | 1202 | 13 | 13 | +0.0263 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.19967554410134913,0.17682512507787593,0.4225018975819397,0.2627978819884555]}` |
| PGL Masters Bucharest 2025 | 1246 | 41 | 41 | -0.0194 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.22592871503219336,0.18639811983654664,0.38862931894074887,0.2698589242137701]}` |
| IEM Chengdu 2025 | 1287 | 30 | 30 | +0.0128 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.21133363279046638,0.17110912284106147,0.429105493095376,0.26510468259143277]}` |
| BLAST Rivals Fall 2025 | 1317 | 13 | 13 | +0.0035 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.19477896345497966,0.16561673248561382,0.4137341898874542,0.26390007661834175]}` |
| StarLadder Major Budapest 2025 | 1330 | 106 | 106 | +0.0144 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.2258422123323989,0.17259258649367823,0.39387593142461325,0.27089867160830083]}` |
| BLAST Bounty Winter 2026 | 1436 | 27 | 27 | -0.0161 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.15226459531483166,0.14809717784384963,0.3567212164827152,0.2627673364472445]}` |
| IEM Krakow 2026 | 1467 | 50 | 50 | -0.0015 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.1564565904990619,0.16075197249111675,0.3619219866225617,0.26167737963324555]}` |
| PGL Cluj-Napoca 2026 | 1517 | 41 | 41 | -0.0114 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.13209410015794218,0.17424685113040625,0.34159044551182094,0.25717269948666366]}` |
| ESL Pro League Season 23 2026 | 1558 | 72 | 72 | -0.0043 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.12485607532972132,0.15429770137078533,0.3696050798918275,0.2554069401986601]}` |
| BLAST Open Rotterdam Spring 2026 | 1632 | 29 | 29 | +0.0126 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.1229948599615626,0.18802205900436955,0.33134899529360634,0.25471365038522475]}` |
| PGL Bucharest 2026 | 1661 | 37 | 37 | +0.0239 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.12128154445994709,0.19622785217592306,0.3071220196340077,0.2556814667303325]}` |
| IEM Rio 2026 | 1700 | 30 | 30 | -0.0227 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.08697553882467349,0.1429197126404185,0.3152849797117254,0.2541533737078092]}` |
| BLAST Rivals Spring 2026 | 1730 | 13 | 13 | -0.0132 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.09047022338145025,0.17725593698279743,0.30497041369885575,0.2518402155687599]}` |
| PGL Astana 2026 | 1743 | 38 | 38 | +0.0235 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.10341528344144357,0.1955074916028867,0.2941022962481935,0.25337214668604036]}` |
| IEM Atlanta 2026 | 1743 | 30 | 30 | -0.0291 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.10341528344144357,0.1955074916028867,0.2941022962481935,0.25337214668604036]}` |
| CS Asia Championships 2026 | 1814 | 30 | 30 | +0.0135 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.09519046283736583,0.15124844697940268,0.33199418873846104,0.22720603620175853]}` |
| IEM Cologne Major 2026 | 1844 | 104 | 104 | -0.0071 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.10317836690416991,0.1597174154694954,0.30028379236481156,0.24629769770813173]}` |
| XSE Pro League 2026 | 1950 | 36 | 36 | +0.0033 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.11806957137822562,0.14509591156416385,0.32608845684169463,0.24904358471378057]}` |
| BLAST Bounty Summer 2026 | 1990 | 28 | 28 | -0.0247 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.1150755076461795,0.13308668489749936,0.3318711065076947,0.24921428593210707]}` |
| Esports World Cup 2026 | 2021 | 55 | 55 | +0.0079 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.12612564254588027,0.14831164911624273,0.3389448310036263,0.24938499854794766]}` |
| BLAST Open Porto Fall 2026 | 2077 | 29 | 29 | +0.0047 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.1144959978541206,0.14048940696948456,0.33005742440653396,0.24841808352014133]}` |
| FISSURE PLAYGROUND Season 3 2026 | 2106 | 28 | 28 | +0.0053 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.13734942408624573,0.1388747789705507,0.32602973391910406,0.25401329331184125]}` |
| StarLadder StarSeries Fall 2026 | 2135 | 14 | 14 | -0.0324 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.14830945113853514,0.12254128317872061,0.3308449704791143,0.2578046918444199]}` |
| ESL Pro League Season 24 2026 | 2149 | 39 | 39 | +0.0175 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.15995306264758327,0.13456056588376344,0.3298839504044131,0.271583051650117]}` |
| Thunderpick World Championship 2026 | 2190 | 14 | 14 | -0.0421 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.15101163992298425,0.11253204201034343,0.32466878018281864,0.2725159080219755]}` |
| PGL Masters Bucharest 2026 | 2204 | 41 | 41 | -0.0142 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.15526616133822727,0.1276926381539875,0.32591027898446556,0.27163011745345333]}` |
| IEM Beijing 2026 | 2245 | 30 | 30 | +0.0278 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.1786871181005339,0.1512317229110827,0.3089567107067007,0.27596163724198697]}` |
| BLAST Rivals Fall 2026 | 2275 | 13 | 13 | +0.0078 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.1721350047877657,0.15766860463039845,0.2722094830808479,0.27692307187543963]}` |

## `stakes`: no evidence

Pre-round Swiss records, advancement and elimination Matches, and Playoffs rounds.

Verdict: better overall, but worse on the Matches with a market line (+0.0177).

| On the 630 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.7313 | 0.2387 | 65.4% |
| stakes | 0.6934 | 0.2335 | 65.4% |

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| BLAST Premier Groups Fall 2024 | 0 | 0 | 0 | — | — |
| IEM Cologne 2024 | 26 | 0 | 0 | — | — |
| BLAST Premier Showdown Fall 2024 | 75 | 0 | 0 | — | — |
| BetBoom Dacha Belgrade season 2 2024 | 75 | 0 | 0 | — | — |
| ESL Pro League Season 20 2024 | 115 | 0 | 0 | — | — |
| BLAST Premier Final Fall 2024 | 194 | 0 | 0 | — | — |
| IEM Rio 2024 | 207 | 5 | 5 | -0.0142 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[0.0,0.0,0.0,0.0,0.0,-0.09803962769262599,-0.02369221819416532,-0.09231927314597348]}` |
| Thunderpick World Championship 2024 | 236 | 7 | 7 | -0.0745 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[0.0,0.0,0.0,0.0,0.0,-0.1695654315756198,0.0033589852585543434,-0.07586913459739368]}` |
| BLAST Premier World Final 2024 | 236 | 5 | 5 | +0.0136 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[0.0,0.0,0.0,0.0,0.0,-0.1695654315756198,0.0033589852585543434,-0.07586913459739368]}` |
| Perfect World Major Shanghai: American RMR 2024 | 277 | 0 | 0 | — | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[0.0,0.0,0.0,0.0,0.0,-0.32532432508709347,0.04758431025595944,-0.04563306851353747]}` |
| Perfect World Major Shanghai: European RMR A 2024 | 311 | 0 | 0 | — | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[0.0,0.0,0.0,0.0,0.0,-0.32532432508709347,0.04758431025595944,-0.04563306851353747]}` |
| Perfect World Major Shanghai: European RMR B 2024 | 346 | 0 | 0 | — | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[0.0,0.0,0.0,0.0,0.0,-0.32532432508709347,0.04758431025595944,-0.04563306851353747]}` |
| Perfect World Major Shanghai Major 2024 | 380 | 73 | 73 | -0.0032 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[0.0,0.0,0.0,0.0,0.0,-0.32532432508709347,0.04758431025595944,-0.04563306851353747]}` |
| BLAST Premier Bounty Closed Qualifier 2025 | 453 | 0 | 0 | — | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[0.03503880224436398,-0.2103220480939539,0.09623032087440926,-0.16453732433673943,0.0,-0.3654857929619832,-0.1433603392902976,-0.02220885477866251]}` |
| BLAST Premier Bounty season 1 2025 | 477 | 7 | 7 | -0.2874 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[0.03503880224436398,-0.2103220480939539,0.09623032087440926,-0.16453732433673943,0.0,-0.3654857929619832,-0.1433603392902976,-0.02220885477866251]}` |
| IEM Katowice 2025 | 484 | 5 | 5 | +0.0540 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[0.03503880224436398,-0.2103220480939539,0.09623032087440926,-0.16453732433673943,0.0,-0.48251226006529785,-0.433873920398032,-0.021812517287953102]}` |
| PGL Cluj-Napoca 2025 | 533 | 1 | 1 | +0.0044 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[0.03503880224435155,-0.21032204809400631,0.0962303208744153,-0.1645373243367784,0.0,-0.4523143207242036,-0.4249888999833491,-0.12454918954785009]}` |
| ESL Pro League Season 21 2025 | 574 | 7 | 7 | +0.0583 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[0.03503880224436398,-0.2103220480939539,0.09623032087440926,-0.16453732433673943,0.0,-0.4523143207242029,-0.42498889998334904,-0.12180304722142438]}` |
| BLAST Open Lisbon Spring 2025 | 614 | 5 | 5 | +0.0645 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[0.03503880224436398,-0.2103220480939539,0.09623032087440926,-0.16453732433673943,0.0,-0.4035649296736144,-0.43621096235158097,-0.10314768055312602]}` |
| PGL Bucharest 2025 | 643 | 1 | 1 | -0.0159 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[0.03503880224436398,-0.2103220480939539,0.09623032087440926,-0.16453732433673943,0.0,-0.37765106145746735,-0.4308084598874604,-0.08468543717727874]}` |
| BLAST.tv Major Austin: European Regional Qualifier 2025 | 684 | 0 | 0 | — | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[0.03503880224435155,-0.21032204809400631,0.0962303208744153,-0.1645373243367784,0.0,-0.3776510614574674,-0.4308084598874605,-0.0990126762136022]}` |
| IEM Melbourne 2025 | 719 | 5 | 5 | -0.0673 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[0.03503880224435155,-0.21032204809400631,0.0962303208744153,-0.1645373243367784,0.0,-0.3776510614574674,-0.4308084598874605,-0.0990126762136022]}` |
| BLAST Rivals Spring 2025 | 748 | 5 | 5 | -0.1016 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[0.03503880224436398,-0.2103220480939539,0.09623032087440926,-0.16453732433673943,0.0,-0.34831620841129163,-0.47314815436529084,-0.0933568807719791]}` |
| PGL Astana 2025 | 761 | 7 | 7 | -0.2699 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[0.03503880224435155,-0.21032204809400631,0.0962303208744153,-0.1645373243367784,0.0,-0.33266700644749697,-0.5110316653749952,-0.08795953455509377]}` |
| IEM Dallas 2025 | 802 | 1 | 1 | +0.0215 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[0.0350388022443392,-0.2103220480940583,0.09623032087442131,-0.16453732433681706,0.0,-0.38306372820220014,-0.572769365827638,-0.08507740197927216]}` |
| BLAST.tv Major Austin 2025 | 831 | 106 | 106 | -0.0648 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[0.03503880224433921,-0.21032204809405827,0.09623032087442131,-0.16453732433681703,0.0,-0.38306372820220014,-0.5727693658275643,-0.06599145788109638]}` |
| FISSURE PLAYGROUND Season 1 2025 | 937 | 7 | 7 | -0.3301 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.3059621315173986,-0.25823932516125714,0.2124915795189116,0.009474477058342189,0.0,-0.4020776088603537,-0.5458122691021188,-0.05407541877453726]}` |
| IEM Cologne 2025 | 964 | 5 | 5 | -0.1698 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.30596213151739854,-0.2582393251612572,0.21249157951891165,0.009474477058342323,0.0,-0.5346166080446845,-0.5155403349947727,-0.14353011488578898]}` |
| BLAST Bounty Fall 2025 | 1013 | 1 | 1 | +0.0400 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.3059621315173986,-0.258239325161257,0.21249157951891157,0.009474477058342059,0.0,-0.5399665951831496,-0.5314337412454749,-0.1526479026812769]}` |
| Esports World Cup 2025 | 1044 | 7 | 7 | -0.1999 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.30596213151739854,-0.2582393251612572,0.21249157951891165,0.009474477058342323,0.0,-0.5399665951831115,-0.5314337412453976,-0.13443425847082055]}` |
| BLAST Open London Fall 2025 | 1060 | 1 | 1 | -0.6556 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.30596213151739854,-0.2582393251612572,0.21249157951891165,0.009474477058342323,0.0,-0.5372795182248301,-0.5726144112208376,-0.11607075380874125]}` |
| FISSURE PLAYGROUND Season 2 2025 | 1089 | 1 | 1 | -1.2769 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.30596213151739854,-0.2582393251612572,0.21249157951891165,0.009474477058342323,0.0,-0.5372795182248301,-0.5726144112208376,-0.40366793881098423]}` |
| ESL Pro League Season 22 2025 | 1129 | 7 | 7 | -0.0307 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.30596213151739854,-0.2582393251612572,0.21249157951891165,0.009474477058342323,0.0,-0.5372795182248301,-0.5726144112208376,-0.5148920927044875]}` |
| CS Asia Championships 2025 | 1202 | 5 | 5 | -0.1543 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.3059621315173986,-0.258239325161257,0.21249157951891157,0.009474477058342064,0.0,-0.549907841565903,-0.559438780934567,-0.5060362357310508]}` |
| Thunderpick World Championship 2025 | 1202 | 3 | 3 | -0.1411 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.3059621315173986,-0.258239325161257,0.21249157951891157,0.009474477058342064,0.0,-0.549907841565903,-0.559438780934567,-0.5060362357310508]}` |
| PGL Masters Bucharest 2025 | 1246 | 7 | 7 | -0.1028 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.30596213151739854,-0.2582393251612572,0.21249157951891165,0.009474477058342323,0.0,-0.5326295815294777,-0.5769012489525773,-0.5587307507576015]}` |
| IEM Chengdu 2025 | 1287 | 5 | 5 | +0.1552 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.30596213151739854,-0.2582393251612572,0.21249157951891165,0.009474477058342323,0.0,-0.5522195995731833,-0.5653662080199213,-0.5447861595165595]}` |
| BLAST Rivals Fall 2025 | 1317 | 5 | 5 | +0.1220 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.30596213151739854,-0.2582393251612572,0.21249157951891165,0.009474477058342323,0.0,-0.5378649762799463,-0.5514328566284251,-0.5393735584302305]}` |
| StarLadder Major Budapest 2025 | 1330 | 106 | 106 | -0.0688 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.30596213151739854,-0.2582393251612572,0.21249157951891165,0.009474477058342323,0.0,-0.522710298362008,-0.542679457193563,-0.5296470521805143]}` |
| BLAST Bounty Winter 2026 | 1436 | 7 | 7 | -0.1680 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.2994936079984075,-0.2980324611775121,0.308087420300672,0.07812472282021708,0.0,-0.5877087552053268,-0.5487849807746119,-0.5148297863069498]}` |
| IEM Krakow 2026 | 1467 | 5 | 5 | +0.1267 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.2994936079984075,-0.2980324611775121,0.308087420300672,0.07812472282021707,0.0,-0.576929123666827,-0.5749411162647404,-0.5358738407062171]}` |
| PGL Cluj-Napoca 2026 | 1517 | 7 | 7 | -0.0683 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.2994936079984076,-0.29803246117751203,0.3080874203006721,0.07812472282021703,0.0,-0.5687687694529183,-0.5673204244948742,-0.5294671291740533]}` |
| ESL Pro League Season 23 2026 | 1558 | 7 | 7 | -0.0964 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.2994936079984076,-0.29803246117751203,0.3080874203006721,0.07812472282021703,0.0,-0.574161109943947,-0.5582978746624859,-0.5417212001454004]}` |
| BLAST Open Rotterdam Spring 2026 | 1632 | 5 | 5 | +0.0612 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.2994936079984075,-0.2980324611775121,0.30808742030067193,0.0781247228202171,0.0,-0.5907371584720259,-0.5518512903167458,-0.5380446068369009]}` |
| PGL Bucharest 2026 | 1661 | 7 | 7 | +0.0893 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.2994936079984076,-0.29803246117751203,0.3080874203006721,0.07812472282021703,0.0,-0.5924563250224286,-0.5418126896225831,-0.522765064504315]}` |
| IEM Rio 2026 | 1700 | 5 | 5 | +0.1086 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.2994936079984076,-0.29803246117751203,0.3080874203006721,0.07812472282021703,0.0,-0.5798847072333368,-0.5348844728288247,-0.5225995820906056]}` |
| BLAST Rivals Spring 2026 | 1730 | 5 | 5 | -0.0369 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.2994936079984075,-0.2980324611775121,0.30808742030067193,0.0781247228202171,0.0,-0.573118884768756,-0.5297749408502058,-0.5085767626626335]}` |
| PGL Astana 2026 | 1743 | 7 | 7 | +0.0439 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.2994936079984076,-0.29803246117751203,0.3080874203006721,0.07812472282021703,0.0,-0.5877430681547502,-0.5081853798076358,-0.4957278813743653]}` |
| IEM Atlanta 2026 | 1743 | 5 | 5 | -0.0146 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.2994936079984076,-0.29803246117751203,0.3080874203006721,0.07812472282021703,0.0,-0.5877430681547502,-0.5081853798076358,-0.4957278813743653]}` |
| CS Asia Championships 2026 | 1814 | 5 | 5 | -0.3327 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.2994936079984075,-0.2980324611775121,0.30808742030067193,0.0781247228202171,0.0,-0.5950846399526829,-0.48289890097289,-0.4768678555910857]}` |
| IEM Cologne Major 2026 | 1844 | 106 | 106 | +0.0280 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.2994936079984075,-0.2980324611775121,0.30808742030067193,0.0781247228202171,0.0,-0.6046495067984308,-0.4673766470597996,-0.5387801807213511]}` |
| XSE Pro League 2026 | 1950 | 7 | 7 | -0.0421 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.28657340459345465,-0.25765080030199145,0.3300598606711892,0.015904913268416695,0.0,-0.5969563991109239,-0.4691342737203149,-0.5311298647674707]}` |
| BLAST Bounty Summer 2026 | 1990 | 7 | 7 | +0.0355 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.28657340457919633,-0.25765080030452997,0.33005986074489346,0.015904913302059648,0.0,-0.6037236069760604,-0.4531378601375986,-0.5172414371294245]}` |
| Esports World Cup 2026 | 2021 | 7 | 7 | -0.0522 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.28657340457919633,-0.2576508003045298,0.33005986074489346,0.015904913302059422,0.0,-0.5876753093570302,-0.43536418284390255,-0.5428747774535698]}` |
| BLAST Open Porto Fall 2026 | 2077 | 5 | 5 | +0.0925 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.28657340459345465,-0.25765080030199145,0.3300598606711892,0.015904913268416695,0.0,-0.5971820327677083,-0.4333170211312557,-0.5307284471281036]}` |
| FISSURE PLAYGROUND Season 3 2026 | 2106 | 5 | 5 | +0.0532 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.28657340459345465,-0.25765080030199145,0.3300598606711892,0.015904913268416695,0.0,-0.5908132428712329,-0.4325640235067476,-0.5236410598616889]}` |
| StarLadder StarSeries Fall 2026 | 2135 | 1 | 1 | +0.1948 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.28657340459345465,-0.25765080030199145,0.3300598606711892,0.015904913268416695,0.0,-0.5911734273681546,-0.4175567582870821,-0.5227425604084948]}` |
| ESL Pro League Season 24 2026 | 2149 | 7 | 7 | +0.1153 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.28657340457919633,-0.25765080030452997,0.33005986074489346,0.015904913302059648,0.0,-0.5911734273681546,-0.4175567582870821,-0.5115116442321234]}` |
| Thunderpick World Championship 2026 | 2190 | 3 | 3 | -0.0394 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.28657340459345465,-0.25765080030199145,0.3300598606711892,0.015904913268416695,0.0,-0.5743581462737067,-0.41569961876424144,-0.5056365468596244]}` |
| PGL Masters Bucharest 2026 | 2204 | 7 | 7 | -0.0648 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.28657340457919633,-0.25765080030452997,0.33005986074489346,0.015904913302059648,0.0,-0.5743581462737067,-0.4291362919672306,-0.4949606910348009]}` |
| IEM Beijing 2026 | 2245 | 5 | 5 | +0.0024 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.28657340459345465,-0.25765080030199145,0.3300598606711892,0.015904913268416695,0.0,-0.5686044435197143,-0.46057404925277,-0.48419557660361395]}` |
| BLAST Rivals Fall 2026 | 2275 | 5 | 5 | +0.0144 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.2865734045934826,-0.2576508003019865,0.33005986067104465,0.015904913268350727,0.0,-0.5697765306912641,-0.4615320187283631,-0.47144547563181527]}` |

## `lan_online`: no evidence

Team win rates in the current environment, with separate weights for LAN and online.

Verdict: better probabilities but fewer winners picked: the interval of the log-loss difference is below zero, that of the accuracy difference too.

| On the 2078 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.7955 | 0.2430 | 65.3% |
| lan_online | 0.7904 | 0.2427 | 64.8% |

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| BLAST Premier Groups Fall 2024 | 0 | 0 | 0 | — | — |
| IEM Cologne 2024 | 26 | 0 | 0 | — | — |
| BLAST Premier Showdown Fall 2024 | 75 | 6 | 6 | -0.0000 | `{"features":["lan_win_rate","online_win_rate"],"weights":[0.043596636021595206,0.0]}` |
| BetBoom Dacha Belgrade season 2 2024 | 75 | 23 | 23 | -0.0006 | `{"features":["lan_win_rate","online_win_rate"],"weights":[0.043596636021595206,0.0]}` |
| ESL Pro League Season 20 2024 | 115 | 67 | 67 | +0.0013 | `{"features":["lan_win_rate","online_win_rate"],"weights":[0.07494173802115313,0.0]}` |
| BLAST Premier Final Fall 2024 | 194 | 13 | 13 | -0.0005 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.03764847538417787,0.0]}` |
| IEM Rio 2024 | 207 | 29 | 29 | -0.0029 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.05413049523666517,0.0]}` |
| Thunderpick World Championship 2024 | 236 | 0 | 0 | — | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.19868908895951762,0.0]}` |
| BLAST Premier World Final 2024 | 236 | 13 | 13 | -0.0110 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.19868908895951762,0.0]}` |
| Perfect World Major Shanghai: American RMR 2024 | 277 | 28 | 28 | -0.0011 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.2659815496040439,0.0]}` |
| Perfect World Major Shanghai: European RMR A 2024 | 311 | 26 | 26 | -0.0083 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.27627735602285597,0.0]}` |
| Perfect World Major Shanghai: European RMR B 2024 | 346 | 29 | 29 | -0.0074 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.34823508748385357,0.0]}` |
| Perfect World Major Shanghai Major 2024 | 380 | 72 | 72 | -0.0034 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.40501014861864587,0.0]}` |
| BLAST Premier Bounty Closed Qualifier 2025 | 453 | 11 | 11 | +0.0000 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.4578933042165715,0.0]}` |
| BLAST Premier Bounty season 1 2025 | 477 | 7 | 7 | -0.0245 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.4578933039721464,-0.04241380923139176]}` |
| IEM Katowice 2025 | 484 | 47 | 47 | -0.0029 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.49198476085804077,-0.04241380923139176]}` |
| PGL Cluj-Napoca 2025 | 533 | 38 | 38 | -0.0031 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.5150330764358811,-0.04241380923139176]}` |
| ESL Pro League Season 21 2025 | 574 | 39 | 39 | +0.0034 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.5333311700685105,-0.04241380923139176]}` |
| BLAST Open Lisbon Spring 2025 | 614 | 28 | 28 | +0.0044 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.5095586131209192,-0.04241380923139176]}` |
| PGL Bucharest 2025 | 643 | 40 | 40 | -0.0184 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.48655391448525703,-0.04241380923139176]}` |
| BLAST.tv Major Austin: European Regional Qualifier 2025 | 684 | 32 | 32 | +0.0006 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.618772070822132,-0.04241380923139176]}` |
| IEM Melbourne 2025 | 719 | 29 | 29 | -0.0113 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.618772070822132,0.004592633892439753]}` |
| BLAST Rivals Spring 2025 | 748 | 13 | 13 | -0.0171 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.6643152412408185,0.004592633892439753]}` |
| PGL Astana 2025 | 761 | 39 | 39 | -0.0167 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.6929943627505365,0.004592633892439753]}` |
| IEM Dallas 2025 | 802 | 28 | 28 | +0.0062 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.774146702260692,0.004592633892439747]}` |
| BLAST.tv Major Austin 2025 | 831 | 104 | 104 | -0.0073 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.7521537709023639,0.004592633892439753]}` |
| FISSURE PLAYGROUND Season 1 2025 | 937 | 27 | 27 | -0.0222 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.8295207818948074,0.004592633892439753]}` |
| IEM Cologne 2025 | 964 | 49 | 49 | +0.0041 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.8903201072539171,0.0045926338924397415]}` |
| BLAST Bounty Fall 2025 | 1013 | 28 | 28 | -0.0087 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.8684095305945969,0.004592633892439753]}` |
| Esports World Cup 2025 | 1044 | 16 | 16 | -0.0629 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.8907878007063479,0.004592633892439753]}` |
| BLAST Open London Fall 2025 | 1060 | 28 | 28 | -0.0300 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.9840357174166515,0.004592633892439753]}` |
| FISSURE PLAYGROUND Season 2 2025 | 1089 | 40 | 40 | -0.0029 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-1.0532138419110502,0.004592633892439753]}` |
| ESL Pro League Season 22 2025 | 1129 | 71 | 71 | -0.0047 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-1.0603573116383305,0.004592633892439753]}` |
| CS Asia Championships 2025 | 1202 | 29 | 29 | -0.0014 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-1.0767852185174691,0.004592633892439753]}` |
| Thunderpick World Championship 2025 | 1202 | 13 | 13 | +0.0055 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-1.0767852185174691,0.004592633892439753]}` |
| PGL Masters Bucharest 2025 | 1246 | 41 | 41 | -0.0108 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-1.0711436222603363,0.004592633892439753]}` |
| IEM Chengdu 2025 | 1287 | 30 | 30 | -0.0225 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-1.1037631265519432,0.004592633892439747]}` |
| BLAST Rivals Fall 2025 | 1317 | 13 | 13 | -0.0245 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-1.1487131947984555,0.004592633892439753]}` |
| StarLadder Major Budapest 2025 | 1330 | 106 | 106 | -0.0131 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-1.168783057444734,0.004592633892439744]}` |
| BLAST Bounty Winter 2026 | 1436 | 27 | 27 | -0.0193 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-1.2488722370061913,0.004592633892439753]}` |
| IEM Krakow 2026 | 1467 | 50 | 50 | -0.0012 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-1.2774119992318227,0.004592633892439753]}` |
| PGL Cluj-Napoca 2026 | 1517 | 41 | 41 | +0.0221 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-1.2737547689828885,0.004592633892439753]}` |
| ESL Pro League Season 23 2026 | 1558 | 72 | 72 | -0.0066 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-1.2140128824879397,0.004592633892439753]}` |
| BLAST Open Rotterdam Spring 2026 | 1632 | 29 | 29 | -0.0128 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-1.2377022487224558,0.004592633892439753]}` |
| PGL Bucharest 2026 | 1661 | 37 | 37 | +0.0129 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-1.2531577998756818,0.004592633892439753]}` |
| IEM Rio 2026 | 1700 | 30 | 30 | +0.0107 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-1.220770492645383,0.004592633892439753]}` |
| BLAST Rivals Spring 2026 | 1730 | 13 | 13 | -0.0092 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-1.1966853269559605,0.004592633892439753]}` |
| PGL Astana 2026 | 1743 | 38 | 38 | +0.0078 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-1.2020777809138756,0.004592633892439753]}` |
| IEM Atlanta 2026 | 1743 | 30 | 30 | -0.0225 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-1.2020777809138756,0.004592633892439753]}` |
| CS Asia Championships 2026 | 1814 | 30 | 30 | -0.0166 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-1.2142596112921096,0.004592633892439753]}` |
| IEM Cologne Major 2026 | 1844 | 104 | 104 | +0.0015 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-1.2411943911391623,0.004592633892439753]}` |
| XSE Pro League 2026 | 1950 | 36 | 36 | +0.0140 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-1.2202228385123806,0.004592633892439753]}` |
| BLAST Bounty Summer 2026 | 1990 | 28 | 28 | -0.0252 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-1.1881816310674909,0.004592633892439753]}` |
| Esports World Cup 2026 | 2021 | 54 | 54 | +0.0192 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-1.2254600135696718,0.004592633892439753]}` |
| BLAST Open Porto Fall 2026 | 2077 | 28 | 28 | +0.0108 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-1.1568481662429069,0.004592633892439753]}` |
| FISSURE PLAYGROUND Season 3 2026 | 2106 | 28 | 28 | -0.0041 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-1.1332292618235336,0.004592633892439753]}` |
| StarLadder StarSeries Fall 2026 | 2135 | 14 | 14 | -0.0213 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-1.1375286119221861,0.004592633892439753]}` |
| ESL Pro League Season 24 2026 | 2149 | 39 | 39 | -0.0131 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-1.1517350852988149,0.004592633892439753]}` |
| Thunderpick World Championship 2026 | 2190 | 14 | 14 | +0.0083 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-1.1748815289289822,0.004592633892439753]}` |
| PGL Masters Bucharest 2026 | 2204 | 41 | 41 | -0.0254 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-1.1675795271870657,0.004592633892439753]}` |
| IEM Beijing 2026 | 2245 | 30 | 30 | +0.0041 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-1.2183477886745377,0.004592633892439753]}` |
| BLAST Rivals Fall 2026 | 2275 | 13 | 13 | -0.0076 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-1.210344575078885,0.004592633892439753]}` |

## `series_format`: no evidence

Team differences in Bo1 versus Bo3 performance, conditioned on the current format.

Verdict: the interval of the log-loss difference includes zero.

| On the 1710 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.7643 | 0.2417 | 65.2% |
| series_format | 0.7646 | 0.2421 | 65.0% |

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| BLAST Premier Groups Fall 2024 | 0 | 0 | 0 | — | — |
| IEM Cologne 2024 | 26 | 0 | 0 | — | — |
| BLAST Premier Showdown Fall 2024 | 75 | 0 | 0 | — | — |
| BetBoom Dacha Belgrade season 2 2024 | 75 | 0 | 0 | — | — |
| ESL Pro League Season 20 2024 | 115 | 0 | 0 | — | — |
| BLAST Premier Final Fall 2024 | 194 | 0 | 0 | — | — |
| IEM Rio 2024 | 207 | 0 | 0 | — | — |
| Thunderpick World Championship 2024 | 236 | 2 | 2 | +0.0001 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.0,-0.09669995076263015]}` |
| BLAST Premier World Final 2024 | 236 | 8 | 8 | -0.0050 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.0,-0.09669995076263015]}` |
| Perfect World Major Shanghai: American RMR 2024 | 277 | 18 | 18 | -0.0010 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.0,-0.13600704621152118]}` |
| Perfect World Major Shanghai: European RMR A 2024 | 311 | 16 | 16 | -0.0036 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.0005249151279679677,-0.14754788800164284]}` |
| Perfect World Major Shanghai: European RMR B 2024 | 346 | 16 | 16 | -0.0152 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.012853748068887796,-0.18349401123732967]}` |
| Perfect World Major Shanghai Major 2024 | 380 | 71 | 71 | +0.0019 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.03821992983659075,-0.3049172975886139]}` |
| BLAST Premier Bounty Closed Qualifier 2025 | 453 | 20 | 20 | +0.0017 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.055785655628152554,-0.2594781372350611]}` |
| BLAST Premier Bounty season 1 2025 | 477 | 6 | 6 | +0.0348 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.05578565562815246,-0.24703818061339522]}` |
| IEM Katowice 2025 | 484 | 46 | 46 | +0.0052 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.05578565562815246,-0.17060737335709675]}` |
| PGL Cluj-Napoca 2025 | 533 | 37 | 37 | +0.0008 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.055785655628152436,-0.046134181869375374]}` |
| ESL Pro League Season 21 2025 | 574 | 36 | 36 | +0.0003 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.055785655628152436,0.006795529341121193]}` |
| BLAST Open Lisbon Spring 2025 | 614 | 26 | 26 | +0.0009 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.05578565562815246,-0.1297274149668784]}` |
| PGL Bucharest 2025 | 643 | 35 | 35 | +0.0021 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.05578565562815246,-0.11473831855851292]}` |
| BLAST.tv Major Austin: European Regional Qualifier 2025 | 684 | 26 | 26 | +0.0002 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.055785655628153616,-0.061403542780430614]}` |
| IEM Melbourne 2025 | 719 | 28 | 28 | -0.0010 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.04096664531881723,-0.06613502284794365]}` |
| BLAST Rivals Spring 2025 | 748 | 12 | 12 | +0.0019 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.040966645318816684,-0.10017090685639152]}` |
| PGL Astana 2025 | 761 | 31 | 31 | -0.0035 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.04096664531881668,-0.0822930425086123]}` |
| IEM Dallas 2025 | 802 | 24 | 24 | -0.0049 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.04096664531881709,-0.18684649545704018]}` |
| BLAST.tv Major Austin 2025 | 831 | 100 | 100 | +0.0007 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.040966645318816684,-0.23542089278790815]}` |
| FISSURE PLAYGROUND Season 1 2025 | 937 | 26 | 26 | +0.0056 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[-0.04146966165973363,-0.21993102909250206]}` |
| IEM Cologne 2025 | 964 | 48 | 48 | -0.0019 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[-0.041469661659733516,-0.16905207572157346]}` |
| BLAST Bounty Fall 2025 | 1013 | 27 | 27 | -0.0024 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[-0.04146966165973363,-0.20588915762846272]}` |
| Esports World Cup 2025 | 1044 | 15 | 15 | +0.0012 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[-0.04146966165973363,-0.2284583999469365]}` |
| BLAST Open London Fall 2025 | 1060 | 26 | 26 | -0.0093 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[-0.04146966165973363,-0.22218693610388068]}` |
| FISSURE PLAYGROUND Season 2 2025 | 1089 | 39 | 39 | -0.0023 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[-0.04146966165973363,-0.2985932213395892]}` |
| ESL Pro League Season 22 2025 | 1129 | 48 | 48 | +0.0068 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[-0.04146966165973359,-0.31711719750589257]}` |
| CS Asia Championships 2025 | 1202 | 27 | 27 | -0.0088 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[-0.041469661659733155,-0.2435516308192078]}` |
| Thunderpick World Championship 2025 | 1202 | 10 | 10 | -0.0099 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[-0.041469661659733155,-0.2435516308192078]}` |
| PGL Masters Bucharest 2025 | 1246 | 35 | 35 | -0.0179 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[-0.01527962909662764,-0.34002301284433106]}` |
| IEM Chengdu 2025 | 1287 | 29 | 29 | +0.0061 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[-0.01527962909662764,-0.45817286750546804]}` |
| BLAST Rivals Fall 2025 | 1317 | 12 | 12 | +0.0062 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[-0.015279629096627672,-0.43040456170279356]}` |
| StarLadder Major Budapest 2025 | 1330 | 105 | 105 | -0.0008 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[-0.01527962909662764,-0.41788342235401127]}` |
| BLAST Bounty Winter 2026 | 1436 | 18 | 18 | +0.0301 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.13018679354398616,-0.4303655993391043]}` |
| IEM Krakow 2026 | 1467 | 49 | 49 | -0.0028 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.13018679354398616,-0.34867214560355353]}` |
| PGL Cluj-Napoca 2026 | 1517 | 40 | 40 | +0.0066 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.13018679354398616,-0.36503715272189896]}` |
| ESL Pro League Season 23 2026 | 1558 | 59 | 59 | +0.0026 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.13018679354398616,-0.3175928298401075]}` |
| BLAST Open Rotterdam Spring 2026 | 1632 | 28 | 28 | -0.0030 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.130186793546517,-0.28590585393134027]}` |
| PGL Bucharest 2026 | 1661 | 22 | 22 | -0.0027 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.13018679354398616,-0.2979136766299653]}` |
| IEM Rio 2026 | 1700 | 24 | 24 | +0.0032 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.13018679354398616,-0.3061668862072434]}` |
| BLAST Rivals Spring 2026 | 1730 | 12 | 12 | +0.0094 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.13018679354398616,-0.2914036242432681]}` |
| PGL Astana 2026 | 1743 | 20 | 20 | -0.0152 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.13018679354398616,-0.27009436458285585]}` |
| IEM Atlanta 2026 | 1743 | 29 | 29 | +0.0090 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.13018679354398616,-0.27009436458285585]}` |
| CS Asia Championships 2026 | 1814 | 29 | 29 | -0.0007 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.13018679354398616,-0.2731685490606202]}` |
| IEM Cologne Major 2026 | 1844 | 95 | 95 | -0.0016 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.14339254890495895,-0.27240586495089975]}` |
| XSE Pro League 2026 | 1950 | 29 | 29 | +0.0014 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.16846108943780472,-0.286145156092966]}` |
| BLAST Bounty Summer 2026 | 1990 | 19 | 19 | +0.0109 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.18405849332742408,-0.27261523573780816]}` |
| Esports World Cup 2026 | 2021 | 50 | 50 | -0.0016 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.18405849299353555,-0.2345147774530623]}` |
| BLAST Open Porto Fall 2026 | 2077 | 23 | 23 | +0.0021 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.19830831536510216,-0.2415239973608656]}` |
| FISSURE PLAYGROUND Season 3 2026 | 2106 | 24 | 24 | +0.0014 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.19830831569424925,-0.23047913245695903]}` |
| StarLadder StarSeries Fall 2026 | 2135 | 13 | 13 | -0.0005 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.19830831536510216,-0.22310062116827387]}` |
| ESL Pro League Season 24 2026 | 2149 | 32 | 32 | +0.0076 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.19830831536510216,-0.2235310920453502]}` |
| Thunderpick World Championship 2026 | 2190 | 13 | 13 | -0.0074 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.19830831536510216,-0.17222332247055172]}` |
| PGL Masters Bucharest 2026 | 2204 | 36 | 36 | +0.0011 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.1983083155702922,-0.1964779428093162]}` |
| IEM Beijing 2026 | 2245 | 29 | 29 | -0.0013 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.19830831552453043,-0.1865573984667958]}` |
| BLAST Rivals Fall 2026 | 2275 | 12 | 12 | -0.0026 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.19830831536510216,-0.1945981626805892]}` |

## `region_travel`: no evidence

Difference in playing a LAN outside the team's published VRS qualifying region.

Verdict: the interval of the log-loss difference includes zero.

| On the 107 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.5844 | 0.1994 | 68.2% |
| region_travel | 0.5854 | 0.1997 | 67.3% |

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| BLAST Premier Groups Fall 2024 | 0 | 0 | 0 | — | — |
| IEM Cologne 2024 | 26 | 0 | 0 | — | — |
| BLAST Premier Showdown Fall 2024 | 75 | 0 | 0 | — | — |
| BetBoom Dacha Belgrade season 2 2024 | 75 | 0 | 0 | — | — |
| ESL Pro League Season 20 2024 | 115 | 0 | 0 | — | — |
| BLAST Premier Final Fall 2024 | 194 | 0 | 0 | — | — |
| IEM Rio 2024 | 207 | 0 | 0 | — | — |
| Thunderpick World Championship 2024 | 236 | 0 | 0 | — | — |
| BLAST Premier World Final 2024 | 236 | 0 | 0 | — | — |
| Perfect World Major Shanghai: American RMR 2024 | 277 | 0 | 0 | — | — |
| Perfect World Major Shanghai: European RMR A 2024 | 311 | 0 | 0 | — | — |
| Perfect World Major Shanghai: European RMR B 2024 | 346 | 0 | 0 | — | — |
| Perfect World Major Shanghai Major 2024 | 380 | 0 | 0 | — | — |
| BLAST Premier Bounty Closed Qualifier 2025 | 453 | 0 | 0 | — | — |
| BLAST Premier Bounty season 1 2025 | 477 | 0 | 0 | — | — |
| IEM Katowice 2025 | 484 | 0 | 0 | — | — |
| PGL Cluj-Napoca 2025 | 533 | 0 | 0 | — | — |
| ESL Pro League Season 21 2025 | 574 | 0 | 0 | — | — |
| BLAST Open Lisbon Spring 2025 | 614 | 0 | 0 | — | — |
| PGL Bucharest 2025 | 643 | 0 | 0 | — | — |
| BLAST.tv Major Austin: European Regional Qualifier 2025 | 684 | 0 | 0 | — | — |
| IEM Melbourne 2025 | 719 | 0 | 0 | — | — |
| BLAST Rivals Spring 2025 | 748 | 0 | 0 | — | — |
| PGL Astana 2025 | 761 | 0 | 0 | — | — |
| IEM Dallas 2025 | 802 | 0 | 0 | — | — |
| BLAST.tv Major Austin 2025 | 831 | 0 | 0 | — | — |
| FISSURE PLAYGROUND Season 1 2025 | 937 | 0 | 0 | — | — |
| IEM Cologne 2025 | 964 | 0 | 0 | — | — |
| BLAST Bounty Fall 2025 | 1013 | 0 | 0 | — | — |
| Esports World Cup 2025 | 1044 | 0 | 0 | — | — |
| BLAST Open London Fall 2025 | 1060 | 0 | 0 | — | — |
| FISSURE PLAYGROUND Season 2 2025 | 1089 | 0 | 0 | — | — |
| ESL Pro League Season 22 2025 | 1129 | 0 | 0 | — | — |
| CS Asia Championships 2025 | 1202 | 0 | 0 | — | — |
| Thunderpick World Championship 2025 | 1202 | 0 | 0 | — | — |
| PGL Masters Bucharest 2025 | 1246 | 0 | 0 | — | — |
| IEM Chengdu 2025 | 1287 | 0 | 0 | — | — |
| BLAST Rivals Fall 2025 | 1317 | 0 | 0 | — | — |
| StarLadder Major Budapest 2025 | 1330 | 0 | 0 | — | — |
| BLAST Bounty Winter 2026 | 1436 | 0 | 0 | — | `{"features":["away_region"],"weights":[0.2231117416435859]}` |
| IEM Krakow 2026 | 1467 | 0 | 0 | — | `{"features":["away_region"],"weights":[0.2231117416435859]}` |
| PGL Cluj-Napoca 2026 | 1517 | 0 | 0 | — | `{"features":["away_region"],"weights":[0.2231117416435859]}` |
| ESL Pro League Season 23 2026 | 1558 | 0 | 0 | — | `{"features":["away_region"],"weights":[0.2231117416435859]}` |
| BLAST Open Rotterdam Spring 2026 | 1632 | 0 | 0 | — | `{"features":["away_region"],"weights":[0.2231117416435859]}` |
| PGL Bucharest 2026 | 1661 | 0 | 0 | — | `{"features":["away_region"],"weights":[0.2231117416435859]}` |
| IEM Rio 2026 | 1700 | 1 | 1 | +0.0000 | `{"features":["away_region"],"weights":[0.2231117416435859]}` |
| BLAST Rivals Spring 2026 | 1730 | 0 | 0 | — | `{"features":["away_region"],"weights":[0.2231117416435859]}` |
| PGL Astana 2026 | 1743 | 0 | 0 | — | `{"features":["away_region"],"weights":[0.2231117416435859]}` |
| IEM Atlanta 2026 | 1743 | 0 | 0 | — | `{"features":["away_region"],"weights":[0.2231117416435859]}` |
| CS Asia Championships 2026 | 1814 | 0 | 0 | — | `{"features":["away_region"],"weights":[0.2231117416435859]}` |
| IEM Cologne Major 2026 | 1844 | 106 | 106 | +0.0010 | `{"features":["away_region"],"weights":[0.2231117416435859]}` |
| XSE Pro League 2026 | 1950 | 0 | 0 | — | `{"features":["away_region"],"weights":[0.164552060424046]}` |
| BLAST Bounty Summer 2026 | 1990 | 0 | 0 | — | `{"features":["away_region"],"weights":[0.164552060424046]}` |
| Esports World Cup 2026 | 2021 | 0 | 0 | — | `{"features":["away_region"],"weights":[0.164552060424046]}` |
| BLAST Open Porto Fall 2026 | 2077 | 0 | 0 | — | `{"features":["away_region"],"weights":[0.164552060424046]}` |
| FISSURE PLAYGROUND Season 3 2026 | 2106 | 0 | 0 | — | `{"features":["away_region"],"weights":[0.164552060424046]}` |
| StarLadder StarSeries Fall 2026 | 2135 | 0 | 0 | — | `{"features":["away_region"],"weights":[0.164552060424046]}` |
| ESL Pro League Season 24 2026 | 2149 | 0 | 0 | — | `{"features":["away_region"],"weights":[0.164552060424046]}` |
| Thunderpick World Championship 2026 | 2190 | 0 | 0 | — | `{"features":["away_region"],"weights":[0.164552060424046]}` |
| PGL Masters Bucharest 2026 | 2204 | 0 | 0 | — | `{"features":["away_region"],"weights":[0.164552060424046]}` |
| IEM Beijing 2026 | 2245 | 0 | 0 | — | `{"features":["away_region"],"weights":[0.164552060424046]}` |
| BLAST Rivals Fall 2026 | 2275 | 0 | 0 | — | `{"features":["away_region"],"weights":[0.164552060424046]}` |

## Forward selection

The Candidate Signals marked `promote` enter one combination, one at a time: at each step the one that lowers the walk-forward log-loss most (on the 2243 training-side Matches the anchor predicted) enters, and selection stops when none lowers it. The combination is the combiner (a regularised logistic model without intercept, penalty 10 towards the anchor's weights) over the logits of the market line and the rating plus each member's features, refitted for every Event on the Events that ended before it. "Added" is the log-loss after the step − before it (the anchor before the first).

Considered: `rating_calibration`, `learned_weights`, `own_rating`, `lineup_strength`, `bo1_veto_sim`. Anchor log-loss 0.8044.

| Order | Entered | Log-loss after | Added | 90% interval | Tried |
|---|---|---|---|---|---|
| 1 | `own_rating` | 0.6327 | -0.1716 | [-0.2316, -0.1154] | `rating_calibration` 0.6341, `learned_weights` 0.6341, `own_rating` 0.6327, `lineup_strength` 0.6334, `bo1_veto_sim` 0.6342 |
| 2 | `lineup_strength` | 0.6323 | -0.0004 | [-0.0007, -0.0003] | `rating_calibration` 0.6329, `learned_weights` 0.6329, `lineup_strength` 0.6323, `bo1_veto_sim` 0.6330 |

Frozen combination: `own_rating` + `lineup_strength`.

| Event | Fitted on | Combiner weights |
|---|---|---|
| BLAST Premier Groups Fall 2024 | 0 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"lineup_strength.lineup_strength":0.0,"matches":0,"own_rating.shift":0.0,"rating":1.0}}` |
| IEM Cologne 2024 | 26 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"lineup_strength.lineup_strength":0.0,"matches":0,"own_rating.shift":0.0,"rating":1.0}}` |
| BLAST Premier Showdown Fall 2024 | 75 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"lineup_strength.lineup_strength":0.003024,"matches":49,"own_rating.shift":0.26611,"rating":0.505019}}` |
| BetBoom Dacha Belgrade season 2 2024 | 75 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"lineup_strength.lineup_strength":0.003024,"matches":49,"own_rating.shift":0.26611,"rating":0.505019}}` |
| ESL Pro League Season 20 2024 | 115 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"lineup_strength.lineup_strength":-0.000218,"matches":89,"own_rating.shift":0.181457,"rating":0.467909}}` |
| BLAST Premier Final Fall 2024 | 194 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"lineup_strength.lineup_strength":0.002167,"matches":165,"own_rating.shift":0.118175,"rating":0.384939}}` |
| IEM Rio 2024 | 207 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"lineup_strength.lineup_strength":0.011251,"matches":178,"own_rating.shift":0.11985,"rating":0.398084}}` |
| Thunderpick World Championship 2024 | 236 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"lineup_strength.lineup_strength":0.001857,"matches":207,"own_rating.shift":0.096718,"rating":0.367084}}` |
| BLAST Premier World Final 2024 | 236 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"lineup_strength.lineup_strength":0.001857,"matches":207,"own_rating.shift":0.096718,"rating":0.367084}}` |
| Perfect World Major Shanghai: American RMR 2024 | 277 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"lineup_strength.lineup_strength":0.000946,"matches":248,"own_rating.shift":0.120956,"rating":0.363989}}` |
| Perfect World Major Shanghai: European RMR A 2024 | 311 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"lineup_strength.lineup_strength":0.003335,"matches":282,"own_rating.shift":0.122784,"rating":0.38194}}` |
| Perfect World Major Shanghai: European RMR B 2024 | 346 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"lineup_strength.lineup_strength":0.037721,"matches":312,"own_rating.shift":0.096835,"rating":0.376179}}` |
| Perfect World Major Shanghai Major 2024 | 380 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"lineup_strength.lineup_strength":0.041947,"matches":346,"own_rating.shift":0.119883,"rating":0.375112}}` |
| BLAST Premier Bounty Closed Qualifier 2025 | 453 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"lineup_strength.lineup_strength":0.063245,"matches":419,"own_rating.shift":0.103587,"rating":0.384419}}` |
| BLAST Premier Bounty season 1 2025 | 477 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"lineup_strength.lineup_strength":0.083568,"matches":442,"own_rating.shift":0.131371,"rating":0.393559}}` |
| IEM Katowice 2025 | 484 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"lineup_strength.lineup_strength":0.081101,"matches":449,"own_rating.shift":0.135645,"rating":0.391635}}` |
| PGL Cluj-Napoca 2025 | 533 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"lineup_strength.lineup_strength":0.10096,"matches":496,"own_rating.shift":0.155776,"rating":0.404233}}` |
| ESL Pro League Season 21 2025 | 574 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"lineup_strength.lineup_strength":0.110438,"matches":534,"own_rating.shift":0.140244,"rating":0.400324}}` |
| BLAST Open Lisbon Spring 2025 | 614 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"lineup_strength.lineup_strength":0.131762,"matches":574,"own_rating.shift":0.141879,"rating":0.414294}}` |
| PGL Bucharest 2025 | 643 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"lineup_strength.lineup_strength":0.149915,"matches":603,"own_rating.shift":0.154773,"rating":0.430519}}` |
| BLAST.tv Major Austin: European Regional Qualifier 2025 | 684 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"lineup_strength.lineup_strength":0.168056,"matches":644,"own_rating.shift":0.167452,"rating":0.405465}}` |
| IEM Melbourne 2025 | 719 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"lineup_strength.lineup_strength":0.172013,"matches":679,"own_rating.shift":0.184106,"rating":0.406721}}` |
| BLAST Rivals Spring 2025 | 748 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"lineup_strength.lineup_strength":0.193723,"matches":708,"own_rating.shift":0.180198,"rating":0.408744}}` |
| PGL Astana 2025 | 761 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"lineup_strength.lineup_strength":0.202567,"matches":721,"own_rating.shift":0.184686,"rating":0.414371}}` |
| IEM Dallas 2025 | 802 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"lineup_strength.lineup_strength":0.226787,"matches":762,"own_rating.shift":0.224006,"rating":0.424036}}` |
| BLAST.tv Major Austin 2025 | 831 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"lineup_strength.lineup_strength":0.235237,"matches":791,"own_rating.shift":0.237074,"rating":0.443523}}` |
| FISSURE PLAYGROUND Season 1 2025 | 937 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"lineup_strength.lineup_strength":0.246868,"matches":897,"own_rating.shift":0.2267,"rating":0.431083}}` |
| IEM Cologne 2025 | 964 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"lineup_strength.lineup_strength":0.251285,"matches":924,"own_rating.shift":0.234552,"rating":0.429995}}` |
| BLAST Bounty Fall 2025 | 1013 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"lineup_strength.lineup_strength":0.268822,"matches":973,"own_rating.shift":0.240345,"rating":0.443335}}` |
| Esports World Cup 2025 | 1044 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"lineup_strength.lineup_strength":0.258178,"matches":1003,"own_rating.shift":0.248498,"rating":0.458942}}` |
| BLAST Open London Fall 2025 | 1060 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"lineup_strength.lineup_strength":0.251649,"matches":1019,"own_rating.shift":0.239665,"rating":0.448597}}` |
| FISSURE PLAYGROUND Season 2 2025 | 1089 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"lineup_strength.lineup_strength":0.264128,"matches":1048,"own_rating.shift":0.238,"rating":0.443556}}` |
| ESL Pro League Season 22 2025 | 1129 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"lineup_strength.lineup_strength":0.285959,"matches":1088,"own_rating.shift":0.249603,"rating":0.452255}}` |
| CS Asia Championships 2025 | 1202 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"lineup_strength.lineup_strength":0.36738,"matches":1161,"own_rating.shift":0.215375,"rating":0.40153}}` |
| Thunderpick World Championship 2025 | 1202 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"lineup_strength.lineup_strength":0.36738,"matches":1161,"own_rating.shift":0.215375,"rating":0.40153}}` |
| PGL Masters Bucharest 2025 | 1246 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"lineup_strength.lineup_strength":0.378619,"matches":1205,"own_rating.shift":0.222566,"rating":0.407758}}` |
| IEM Chengdu 2025 | 1287 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"lineup_strength.lineup_strength":0.38057,"matches":1246,"own_rating.shift":0.224536,"rating":0.407842}}` |
| BLAST Rivals Fall 2025 | 1317 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"lineup_strength.lineup_strength":0.388016,"matches":1276,"own_rating.shift":0.226984,"rating":0.412687}}` |
| StarLadder Major Budapest 2025 | 1330 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"lineup_strength.lineup_strength":0.396266,"matches":1289,"own_rating.shift":0.226127,"rating":0.413054}}` |
| BLAST Bounty Winter 2026 | 1436 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"lineup_strength.lineup_strength":0.431604,"matches":1395,"own_rating.shift":0.228188,"rating":0.416352}}` |
| IEM Krakow 2026 | 1467 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":-0.011763,"market":0.746982,"matches":31,"own_rating.shift":-0.052091,"rating":-0.009712},"rating":{"lineup_strength.lineup_strength":0.431604,"matches":1395,"own_rating.shift":0.228188,"rating":0.416352}}` |
| PGL Cluj-Napoca 2026 | 1517 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.002983,"market":0.813822,"matches":81,"own_rating.shift":-0.063646,"rating":-0.003465},"rating":{"lineup_strength.lineup_strength":0.431604,"matches":1395,"own_rating.shift":0.228188,"rating":0.416352}}` |
| ESL Pro League Season 23 2026 | 1558 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.005152,"market":0.834566,"matches":122,"own_rating.shift":-0.047701,"rating":0.063817},"rating":{"lineup_strength.lineup_strength":0.431604,"matches":1395,"own_rating.shift":0.228188,"rating":0.416352}}` |
| BLAST Open Rotterdam Spring 2026 | 1632 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.008588,"market":0.788058,"matches":196,"own_rating.shift":-0.016635,"rating":0.036103},"rating":{"lineup_strength.lineup_strength":0.431604,"matches":1395,"own_rating.shift":0.228188,"rating":0.416352}}` |
| PGL Bucharest 2026 | 1661 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.00079,"market":0.753719,"matches":225,"own_rating.shift":-0.011698,"rating":0.0478},"rating":{"lineup_strength.lineup_strength":0.431604,"matches":1395,"own_rating.shift":0.228188,"rating":0.416352}}` |
| IEM Rio 2026 | 1700 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":-0.001341,"market":0.794848,"matches":264,"own_rating.shift":-0.062781,"rating":0.047673},"rating":{"lineup_strength.lineup_strength":0.431604,"matches":1395,"own_rating.shift":0.228188,"rating":0.416352}}` |
| BLAST Rivals Spring 2026 | 1730 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.008686,"market":0.840838,"matches":292,"own_rating.shift":-0.067077,"rating":0.052008},"rating":{"lineup_strength.lineup_strength":0.433075,"matches":1397,"own_rating.shift":0.229621,"rating":0.417814}}` |
| PGL Astana 2026 | 1743 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.008959,"market":0.824,"matches":305,"own_rating.shift":-0.070123,"rating":0.052658},"rating":{"lineup_strength.lineup_strength":0.433075,"matches":1397,"own_rating.shift":0.229621,"rating":0.417814}}` |
| IEM Atlanta 2026 | 1743 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.008959,"market":0.824,"matches":305,"own_rating.shift":-0.070123,"rating":0.052658},"rating":{"lineup_strength.lineup_strength":0.433075,"matches":1397,"own_rating.shift":0.229621,"rating":0.417814}}` |
| CS Asia Championships 2026 | 1814 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":-0.00336,"market":0.833551,"matches":376,"own_rating.shift":-0.049271,"rating":0.055166},"rating":{"lineup_strength.lineup_strength":0.433075,"matches":1397,"own_rating.shift":0.229621,"rating":0.417814}}` |
| IEM Cologne Major 2026 | 1844 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":-0.00336,"market":0.833551,"matches":376,"own_rating.shift":-0.049271,"rating":0.055166},"rating":{"lineup_strength.lineup_strength":0.440706,"matches":1427,"own_rating.shift":0.23603,"rating":0.424399}}` |
| XSE Pro League 2026 | 1950 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.00133,"market":0.926241,"matches":480,"own_rating.shift":-0.079771,"rating":0.03978},"rating":{"lineup_strength.lineup_strength":0.440491,"matches":1429,"own_rating.shift":0.23367,"rating":0.422826}}` |
| BLAST Bounty Summer 2026 | 1990 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.00133,"market":0.926241,"matches":480,"own_rating.shift":-0.079771,"rating":0.03978},"rating":{"lineup_strength.lineup_strength":0.447731,"matches":1466,"own_rating.shift":0.244874,"rating":0.433304}}` |
| Esports World Cup 2026 | 2021 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.023412,"matches":1,"own_rating.shift":-0.011703},"market+rating":{"lineup_strength.lineup_strength":0.018261,"market":0.888321,"matches":509,"own_rating.shift":-0.152217,"rating":0.019605},"rating":{"lineup_strength.lineup_strength":0.447731,"matches":1466,"own_rating.shift":0.244874,"rating":0.433304}}` |
| BLAST Open Porto Fall 2026 | 2077 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.023412,"matches":1,"own_rating.shift":-0.011703},"market+rating":{"lineup_strength.lineup_strength":0.037858,"market":0.858581,"matches":565,"own_rating.shift":-0.210734,"rating":0.028329},"rating":{"lineup_strength.lineup_strength":0.447731,"matches":1466,"own_rating.shift":0.244874,"rating":0.433304}}` |
| FISSURE PLAYGROUND Season 3 2026 | 2106 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.023412,"matches":1,"own_rating.shift":-0.011703},"market+rating":{"lineup_strength.lineup_strength":0.049579,"market":0.888226,"matches":594,"own_rating.shift":-0.212839,"rating":0.025702},"rating":{"lineup_strength.lineup_strength":0.447731,"matches":1466,"own_rating.shift":0.244874,"rating":0.433304}}` |
| StarLadder StarSeries Fall 2026 | 2135 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.023412,"matches":1,"own_rating.shift":-0.011703},"market+rating":{"lineup_strength.lineup_strength":0.047462,"market":0.834754,"matches":623,"own_rating.shift":-0.174405,"rating":0.037063},"rating":{"lineup_strength.lineup_strength":0.447731,"matches":1466,"own_rating.shift":0.244874,"rating":0.433304}}` |
| ESL Pro League Season 24 2026 | 2149 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.023412,"matches":1,"own_rating.shift":-0.011703},"market+rating":{"lineup_strength.lineup_strength":0.049181,"market":0.844287,"matches":637,"own_rating.shift":-0.178921,"rating":0.029973},"rating":{"lineup_strength.lineup_strength":0.447731,"matches":1466,"own_rating.shift":0.244874,"rating":0.433304}}` |
| Thunderpick World Championship 2026 | 2190 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.023412,"matches":1,"own_rating.shift":-0.011703},"market+rating":{"lineup_strength.lineup_strength":0.053094,"market":0.904382,"matches":678,"own_rating.shift":-0.15292,"rating":0.010627},"rating":{"lineup_strength.lineup_strength":0.447731,"matches":1466,"own_rating.shift":0.244874,"rating":0.433304}}` |
| PGL Masters Bucharest 2026 | 2204 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.023412,"matches":1,"own_rating.shift":-0.011703},"market+rating":{"lineup_strength.lineup_strength":0.055159,"market":0.897485,"matches":692,"own_rating.shift":-0.152552,"rating":0.013835},"rating":{"lineup_strength.lineup_strength":0.447731,"matches":1466,"own_rating.shift":0.244874,"rating":0.433304}}` |
| IEM Beijing 2026 | 2245 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.023412,"matches":1,"own_rating.shift":-0.011703},"market+rating":{"lineup_strength.lineup_strength":0.040703,"market":0.900187,"matches":733,"own_rating.shift":-0.138288,"rating":-0.002627},"rating":{"lineup_strength.lineup_strength":0.447731,"matches":1466,"own_rating.shift":0.244874,"rating":0.433304}}` |
| BLAST Rivals Fall 2026 | 2275 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.023412,"matches":1,"own_rating.shift":-0.011703},"market+rating":{"lineup_strength.lineup_strength":0.03156,"market":0.950086,"matches":763,"own_rating.shift":-0.116993,"rating":-0.018158},"rating":{"lineup_strength.lineup_strength":0.447731,"matches":1466,"own_rating.shift":0.244874,"rating":0.433304}}` |

## Holdout: pending

The Holdout is every Stage that begins at or after 2026-11-25 23:46:46 UTC, or whose Event has a result at or after it or is not complete (except PandaScore tournament 22378, kept on the training side). No fitting and no selection step reads it; the frozen combination is evaluated on it once, last.

The Holdout is pending: no Holdout Match has been played in this archive, so the frozen combination was not evaluated.

## Events (walk-forward order)

| Event | Begins | Ends | Scored Matches |
|---|---|---|---|
| BLAST Premier Groups Fall 2024 | 2024-07-29 10:00 | 2024-08-04 16:31 | 26 |
| IEM Cologne 2024 | 2024-08-07 08:30 | 2024-08-18 20:30 | 49 |
| BLAST Premier Showdown Fall 2024 | 2024-08-21 14:30 | 2024-08-25 20:16 | 14 |
| BetBoom Dacha Belgrade season 2 2024 | 2024-08-24 08:30 | 2024-09-01 14:05 | 26 |
| ESL Pro League Season 20 2024 | 2024-09-03 11:30 | 2024-09-22 20:42 | 79 |
| BLAST Premier Final Fall 2024 | 2024-09-25 10:00 | 2024-09-29 18:19 | 13 |
| IEM Rio 2024 | 2024-10-07 14:00 | 2024-10-13 21:04 | 29 |
| Thunderpick World Championship 2024 | 2024-10-21 09:40 | 2024-11-03 20:23 | 28 |
| BLAST Premier World Final 2024 | 2024-10-30 06:05 | 2024-11-03 11:58 | 13 |
| Perfect World Major Shanghai: American RMR 2024 | 2024-11-12 04:00 | 2024-11-15 13:50 | 34 |
| Perfect World Major Shanghai: European RMR A 2024 | 2024-11-17 04:00 | 2024-11-20 16:25 | 35 |
| Perfect World Major Shanghai: European RMR B 2024 | 2024-11-21 04:00 | 2024-11-24 14:03 | 34 |
| Perfect World Major Shanghai Major 2024 | 2024-11-30 03:00 | 2024-12-15 12:23 | 73 |
| BLAST Premier Bounty Closed Qualifier 2025 | 2025-01-14 11:00 | 2025-01-19 22:35 | 24 |
| BLAST Premier Bounty season 1 2025 | 2025-01-23 15:00 | 2025-01-26 19:09 | 7 |
| IEM Katowice 2025 | 2025-01-29 09:30 | 2025-02-09 18:59 | 49 |
| PGL Cluj-Napoca 2025 | 2025-02-14 08:00 | 2025-02-23 21:54 | 41 |
| ESL Pro League Season 21 2025 | 2025-03-07 09:30 | 2025-03-16 18:41 | 40 |
| BLAST Open Lisbon Spring 2025 | 2025-03-19 11:00 | 2025-03-30 20:48 | 29 |
| PGL Bucharest 2025 | 2025-04-06 07:00 | 2025-04-13 17:41 | 41 |
| BLAST.tv Major Austin: European Regional Qualifier 2025 | 2025-04-14 08:00 | 2025-04-17 18:05 | 35 |
| IEM Melbourne 2025 | 2025-04-21 01:15 | 2025-04-27 12:57 | 29 |
| BLAST Rivals Spring 2025 | 2025-04-30 09:00 | 2025-05-04 19:29 | 13 |
| PGL Astana 2025 | 2025-05-10 05:00 | 2025-05-18 16:33 | 41 |
| IEM Dallas 2025 | 2025-05-19 16:30 | 2025-05-25 22:45 | 29 |
| BLAST.tv Major Austin 2025 | 2025-06-03 15:15 | 2025-06-22 22:15 | 106 |
| FISSURE PLAYGROUND Season 1 2025 | 2025-07-15 09:00 | 2025-07-20 17:27 | 27 |
| IEM Cologne 2025 | 2025-07-23 08:30 | 2025-08-03 18:04 | 49 |
| BLAST Bounty Fall 2025 | 2025-08-05 10:00 | 2025-08-17 17:16 | 31 |
| Esports World Cup 2025 | 2025-08-20 11:00 | 2025-08-24 13:27 | 16 |
| BLAST Open London Fall 2025 | 2025-08-27 10:00 | 2025-09-07 19:37 | 29 |
| FISSURE PLAYGROUND Season 2 2025 | 2025-09-12 08:00 | 2025-09-21 20:52 | 40 |
| ESL Pro League Season 22 2025 | 2025-09-28 08:30 | 2025-10-12 18:03 | 73 |
| CS Asia Championships 2025 | 2025-10-14 03:00 | 2025-10-19 11:40 | 30 |
| Thunderpick World Championship 2025 | 2025-10-15 08:30 | 2025-10-19 20:41 | 14 |
| PGL Masters Bucharest 2025 | 2025-10-26 08:19 | 2025-11-01 18:20 | 41 |
| IEM Chengdu 2025 | 2025-11-03 02:00 | 2025-11-09 11:38 | 30 |
| BLAST Rivals Fall 2025 | 2025-11-12 02:00 | 2025-11-16 13:15 | 13 |
| StarLadder Major Budapest 2025 | 2025-11-24 12:00 | 2025-12-14 21:14 | 106 |
| BLAST Bounty Winter 2026 | 2026-01-13 11:00 | 2026-01-25 18:23 | 31 |
| IEM Krakow 2026 | 2026-01-28 10:00 | 2026-02-08 20:15 | 50 |
| PGL Cluj-Napoca 2026 | 2026-02-14 08:30 | 2026-02-22 19:11 | 41 |
| ESL Pro League Season 23 2026 | 2026-03-01 10:30 | 2026-03-15 20:50 | 74 |
| BLAST Open Rotterdam Spring 2026 | 2026-03-18 10:00 | 2026-03-29 13:52 | 29 |
| PGL Bucharest 2026 | 2026-04-04 07:00 | 2026-04-11 17:55 | 39 |
| IEM Rio 2026 | 2026-04-13 13:00 | 2026-04-19 20:11 | 30 |
| BLAST Rivals Spring 2026 | 2026-04-29 15:05 | 2026-05-03 20:46 | 13 |
| PGL Astana 2026 | 2026-05-09 05:00 | 2026-05-17 14:18 | 41 |
| IEM Atlanta 2026 | 2026-05-11 15:30 | 2026-05-17 21:40 | 30 |
| CS Asia Championships 2026 | 2026-05-20 03:12 | 2026-05-24 10:19 | 30 |
| IEM Cologne Major 2026 | 2026-06-02 10:30 | 2026-06-21 18:06 | 106 |
| XSE Pro League 2026 | 2026-07-01 08:46 | 2026-07-12 11:29 | 40 |
| BLAST Bounty Summer 2026 | 2026-07-21 10:00 | 2026-08-02 14:41 | 31 |
| Esports World Cup 2026 | 2026-08-12 09:00 | 2026-08-23 16:50 | 56 |
| BLAST Open Porto Fall 2026 | 2026-08-26 09:00 | 2026-09-06 18:05 | 29 |
| FISSURE PLAYGROUND Season 3 2026 | 2026-09-08 03:08 | 2026-09-13 12:26 | 29 |
| StarLadder StarSeries Fall 2026 | 2026-09-17 10:00 | 2026-09-20 18:47 | 14 |
| ESL Pro League Season 24 2026 | 2026-10-03 09:00 | 2026-10-11 18:11 | 41 |
| Thunderpick World Championship 2026 | 2026-10-14 08:56 | 2026-10-18 21:31 | 14 |
| PGL Masters Bucharest 2026 | 2026-10-24 07:00 | 2026-10-31 15:40 | 41 |
| IEM Beijing 2026 | 2026-11-02 02:00 | 2026-11-08 10:08 | 30 |
| BLAST Rivals Fall 2026 | 2026-11-11 02:00 | 2026-11-15 11:00 | 13 |

[Catch-up mode in simulated Leagues](catch-up.md)

[LLM track: single agent, debate and seatings](llm.md)

[Pick'em variants in Pick'em Points](pickem.md)
