# Signal Lab report

Archive SHA-256 `beabb356eda6798dc620e39797cf5ee1e47a9b7ebbae11565ef04eaa56de54e4` (8285 files), protocol `signal-lab/2`, seed 33, 2000 Event resamples, 90% intervals, floor 100 paired Matches.

Baseline: the anchor: HybridAnchored without LLM, 0.80 market and 0.20 Valve rating logit (`debate::base`). Each Candidate Signal is fitted, for each Event, only on the Matches of the Events that ended before it began, and every datum it reads is observed before the Match cutoff. Differences are variant − anchor on the Matches both predicted (log-loss and Brier: below zero is better). The interval resamples whole Events. Verdict: `promote` when the interval of the log-loss difference is below zero and the Matches with a market line are not worse; `harmful` when it is above zero; `no evidence` otherwise or under the floor; `not testable` when the data could not be had. "Detectable" is the smallest log-loss difference these Matches would show with 80% probability: an effect smaller than it can hide as `no evidence`.

| Candidate Signal | Verdict | Computed on | Paired | Events | Δ log-loss | 90% interval | Detectable | Δ Brier | Δ accuracy | Market subset (n, Δ log-loss) |
|---|---|---|---|---|---|---|---|---|---|---|
| `rating_calibration` | promote | 2193 | 2193 | 60 | -0.1766 | [-0.2407, -0.1227] | 0.0886 | -0.0252 | +0.0005 | 765, -0.0179 |
| `learned_weights` | promote | 2243 | 2243 | 61 | -0.1713 | [-0.2305, -0.1154] | 0.0872 | -0.0243 | +0.0004 | 766, -0.0144 |
| `market_source_pinnacle` | no evidence | 27 | 27 | 1 | +0.0000 | [+0.0000, +0.0000] | 0.0000 | +0.0000 | +0.0000 | 27, +0.0000 |
| `market_source_polymarket` | no evidence | 27 | 27 | 1 | +0.0238 | [+0.0238, +0.0238] | 0.0000 | +0.0071 | +0.0000 | 27, +0.0238 |
| `market_source_kalshi` | no evidence | 27 | 27 | 1 | +0.0157 | [+0.0157, +0.0157] | 0.0000 | +0.0046 | +0.0000 | 27, +0.0157 |
| `market_source_average` | no evidence | 27 | 27 | 1 | +0.0126 | [+0.0126, +0.0126] | 0.0000 | +0.0038 | +0.0000 | 27, +0.0126 |
| `line_movement` | no evidence | 733 | 733 | 20 | +0.0004 | [-0.0042, +0.0067] | 0.0084 | -0.0001 | +0.0000 | 733, +0.0004 |
| `favourite_longshot` | no evidence | 735 | 735 | 20 | +0.0005 | [-0.0006, +0.0018] | 0.0018 | +0.0001 | -0.0014 | 735, +0.0005 |
| `own_rating` | promote | 2199 | 2168 | 61 | -0.1687 | [-0.2338, -0.1086] | 0.0928 | -0.0219 | -0.0092 | 747, -0.0153 |
| `recent_form` | no evidence | 1980 | 1980 | 60 | -0.0008 | [-0.0053, +0.0037] | 0.0069 | +0.0015 | -0.0015 | 686, +0.0068 |
| `head_to_head` | no evidence | 1280 | 1280 | 57 | -0.0039 | [-0.0127, +0.0046] | 0.0130 | +0.0007 | -0.0078 | 550, +0.0059 |
| `llm_adjustment` | not testable | 0 | 0 | 0 | — | — | — | — | — | — |
| `lineup_strength` | not testable | 0 | 0 | 0 | — | — | — | — | — | — |
| `player_form` | not testable | 0 | 0 | 0 | — | — | — | — | — | — |
| `per_map_rating` | promote | 2117 | 2109 | 61 | -0.1662 | [-0.2345, -0.1044] | 0.0970 | -0.0188 | -0.0142 | 713, -0.0136 |
| `bo1_veto_sim` | not testable | 0 | 0 | 0 | — | — | — | — | — | — |
| `roster_stand_in` | no evidence | 828 | 791 | 23 | +0.0005 | [-0.0035, +0.0044] | 0.0060 | -0.0029 | +0.0051 | — |
| `roster_change_30d` | harmful | 585 | 585 | 22 | +0.0094 | [+0.0026, +0.0176] | 0.0116 | +0.0001 | +0.0000 | — |
| `roster_stability` | no evidence | 332 | 332 | 20 | +0.0153 | [-0.0030, +0.0354] | 0.0293 | +0.0045 | +0.0000 | — |
| `roster_coach_change` | no evidence | 756 | 728 | 23 | +0.0022 | [-0.0015, +0.0056] | 0.0053 | +0.0006 | +0.0000 | — |
| `rest_schedule_load` | no evidence | 2128 | 2128 | 60 | -0.0001 | [-0.0044, +0.0040] | 0.0062 | +0.0005 | +0.0009 | 747, -0.0018 |
| `stakes` | no evidence | 630 | 630 | 51 | -0.0403 | [-0.0681, -0.0150] | 0.0403 | -0.0059 | +0.0000 | 211, +0.0097 |
| `lan_online` | promote | 752 | 752 | 22 | -0.0066 | [-0.0098, -0.0038] | 0.0046 | -0.0007 | +0.0027 | 62, -0.0022 |
| `series_format` | no evidence | 1710 | 1710 | 55 | +0.0003 | [-0.0010, +0.0016] | 0.0020 | +0.0004 | -0.0006 | 634, +0.0016 |
| `region_travel` | not testable | 0 | 0 | 0 | — | — | — | — | — | — |

## `rating_calibration`: promote

Valve rating's map probability rescaled in logit space by a factor fitted per series format, then blended with the market as the anchor does.

Verdict: the interval of the log-loss improvement excludes zero and the Matches with a market line are not worse.

| On the 2193 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.8037 | 0.2432 | 65.3% |
| rating_calibration | 0.6270 | 0.2180 | 65.3% |

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
| IEM Rio 2026 | 1700 | 30 | 30 | +0.0512 | — |
| BLAST Rivals Spring 2026 | 1730 | 13 | 13 | -0.0780 | — |
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
| ESL Pro League Season 24 2026 | 2149 | 41 | 41 | -0.0439 | — |
| Thunderpick World Championship 2026 | 2190 | 14 | 14 | -0.0108 | — |
| PGL Masters Bucharest 2026 | 2204 | 41 | 41 | -0.0998 | — |
| IEM Beijing 2026 | 2245 | 30 | 30 | -0.0398 | — |
| BLAST Rivals Fall 2026 | 2275 | 13 | 13 | -0.0201 | — |

## `learned_weights`: promote

the anchor's market and Valve rating weights (logit space) learned by the combiner on the Events that ended before, one set per availability of the two inputs, regularised towards 0.80 / 0.20.

Verdict: the interval of the log-loss improvement excludes zero and the Matches with a market line are not worse.

| On the 2243 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.8055 | 0.2431 | 65.4% |
| learned_weights | 0.6342 | 0.2188 | 65.4% |

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
| IEM Rio 2026 | 1700 | 30 | 30 | +0.0511 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.792052,"matches":264,"rating":0.057811},"rating":{"matches":1395,"rating":0.233835}}` |
| BLAST Rivals Spring 2026 | 1730 | 13 | 13 | -0.0749 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.819712,"matches":286,"rating":0.063733},"rating":{"matches":1403,"rating":0.234993}}` |
| PGL Astana 2026 | 1743 | 41 | 41 | +0.0014 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.803665,"matches":294,"rating":0.063984},"rating":{"matches":1408,"rating":0.235654}}` |
| IEM Atlanta 2026 | 1743 | 30 | 30 | -0.0214 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.803665,"matches":294,"rating":0.063984},"rating":{"matches":1408,"rating":0.235654}}` |
| CS Asia Championships 2026 | 1814 | 30 | 30 | -0.1678 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.81586,"matches":365,"rating":0.063481},"rating":{"matches":1408,"rating":0.235654}}` |
| IEM Cologne Major 2026 | 1844 | 106 | 106 | +0.0081 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.81586,"matches":365,"rating":0.063481},"rating":{"matches":1438,"rating":0.237308}}` |
| XSE Pro League 2026 | 1950 | 37 | 37 | -0.0334 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.909947,"matches":469,"rating":0.053218},"rating":{"matches":1440,"rating":0.237671}}` |
| BLAST Bounty Summer 2026 | 1990 | 30 | 30 | -0.0577 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.909947,"matches":469,"rating":0.053218},"rating":{"matches":1477,"rating":0.238845}}` |
| Esports World Cup 2026 | 2021 | 56 | 56 | +0.0100 | `{"market":{"market":1.02348,"matches":1},"market+rating":{"market":0.868339,"matches":498,"rating":0.044823},"rating":{"matches":1477,"rating":0.238845}}` |
| BLAST Open Porto Fall 2026 | 2077 | 29 | 29 | +0.0114 | `{"market":{"market":1.02348,"matches":1},"market+rating":{"market":0.836474,"matches":554,"rating":0.063249},"rating":{"matches":1477,"rating":0.238845}}` |
| FISSURE PLAYGROUND Season 3 2026 | 2106 | 29 | 29 | -0.0299 | `{"market":{"market":1.02348,"matches":1},"market+rating":{"market":0.867252,"matches":583,"rating":0.061034},"rating":{"matches":1477,"rating":0.238845}}` |
| StarLadder StarSeries Fall 2026 | 2135 | 14 | 14 | -0.0376 | `{"market":{"market":1.02348,"matches":1},"market+rating":{"market":0.815606,"matches":612,"rating":0.066192},"rating":{"matches":1477,"rating":0.238845}}` |
| ESL Pro League Season 24 2026 | 2149 | 41 | 41 | -0.0429 | `{"market":{"market":1.02348,"matches":1},"market+rating":{"market":0.825108,"matches":626,"rating":0.059844},"rating":{"matches":1477,"rating":0.238845}}` |
| Thunderpick World Championship 2026 | 2190 | 14 | 14 | -0.0103 | `{"market":{"market":1.02348,"matches":1},"market+rating":{"market":0.89892,"matches":667,"rating":0.031052},"rating":{"matches":1477,"rating":0.238845}}` |
| PGL Masters Bucharest 2026 | 2204 | 41 | 41 | -0.0978 | `{"market":{"market":1.02348,"matches":1},"market+rating":{"market":0.898075,"matches":681,"rating":0.03157},"rating":{"matches":1477,"rating":0.238845}}` |
| IEM Beijing 2026 | 2245 | 30 | 30 | -0.0512 | `{"market":{"market":1.02348,"matches":1},"market+rating":{"market":0.920265,"matches":722,"rating":0.004828},"rating":{"matches":1477,"rating":0.238845}}` |
| BLAST Rivals Fall 2026 | 2275 | 13 | 13 | -0.0320 | `{"market":{"market":1.02348,"matches":1},"market+rating":{"market":0.965475,"matches":752,"rating":-0.011468},"rating":{"matches":1477,"rating":0.238845}}` |

## `market_source_pinnacle`: no evidence

the anchor with Pinnacle's last price before the cutoff as the market, on Matches every source priced.

Verdict: computed on 27 paired Matches, under the floor of 100.

| On the 27 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.8122 | 0.2894 | 48.1% |
| market_source_pinnacle | 0.8122 | 0.2894 | 48.1% |

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
| PGL Masters Bucharest 2026 | 2204 | 27 | 27 | +0.0000 | — |
| IEM Beijing 2026 | 2245 | 0 | 0 | — | — |
| BLAST Rivals Fall 2026 | 2275 | 0 | 0 | — | — |

## `market_source_polymarket`: no evidence

the anchor with Polymarket's last price before the cutoff as the market, on Matches every source priced.

Verdict: computed on 27 paired Matches, under the floor of 100.

| On the 27 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.8122 | 0.2894 | 48.1% |
| market_source_polymarket | 0.8361 | 0.2965 | 48.1% |

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
| PGL Masters Bucharest 2026 | 2204 | 27 | 27 | +0.0238 | — |
| IEM Beijing 2026 | 2245 | 0 | 0 | — | — |
| BLAST Rivals Fall 2026 | 2275 | 0 | 0 | — | — |

## `market_source_kalshi`: no evidence

the anchor with Kalshi's last price before the cutoff as the market, on Matches every source priced.

Verdict: computed on 27 paired Matches, under the floor of 100.

| On the 27 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.8122 | 0.2894 | 48.1% |
| market_source_kalshi | 0.8279 | 0.2940 | 48.1% |

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
| PGL Masters Bucharest 2026 | 2204 | 27 | 27 | +0.0157 | — |
| IEM Beijing 2026 | 2245 | 0 | 0 | — | — |
| BLAST Rivals Fall 2026 | 2275 | 0 | 0 | — | — |

## `market_source_average`: no evidence

the anchor with the mean of Pinnacle, Polymarket and Kalshi as the market, on Matches every source priced.

Verdict: computed on 27 paired Matches, under the floor of 100.

| On the 27 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.8122 | 0.2894 | 48.1% |
| market_source_average | 0.8248 | 0.2932 | 48.1% |

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
| PGL Masters Bucharest 2026 | 2204 | 27 | 27 | +0.0126 | — |
| IEM Beijing 2026 | 2245 | 0 | 0 | — | — |
| BLAST Rivals Fall 2026 | 2275 | 0 | 0 | — | — |

## `line_movement`: no evidence

the anchor's logit moved by how the line moved (closing against opening, and closing against six hours before the cutoff), coefficients fitted walk-forward.

Verdict: the interval of the log-loss difference includes zero.

| On the 733 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.6258 | 0.2152 | 66.4% |
| line_movement | 0.6262 | 0.2150 | 66.4% |

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
| IEM Rio 2026 | 1700 | 22 | 22 | -0.0233 | — |
| BLAST Rivals Spring 2026 | 1730 | 8 | 8 | +0.0269 | — |
| PGL Astana 2026 | 1743 | 41 | 41 | -0.0065 | — |
| IEM Atlanta 2026 | 1743 | 30 | 30 | -0.0171 | — |
| CS Asia Championships 2026 | 1814 | 0 | 0 | — | — |
| IEM Cologne Major 2026 | 1844 | 103 | 103 | -0.0061 | — |
| XSE Pro League 2026 | 1950 | 0 | 0 | — | — |
| BLAST Bounty Summer 2026 | 1990 | 30 | 30 | +0.0461 | — |
| Esports World Cup 2026 | 2021 | 56 | 56 | -0.0027 | — |
| BLAST Open Porto Fall 2026 | 2077 | 29 | 29 | +0.0177 | — |
| FISSURE PLAYGROUND Season 3 2026 | 2106 | 29 | 29 | -0.0089 | — |
| StarLadder StarSeries Fall 2026 | 2135 | 14 | 14 | +0.0184 | — |
| ESL Pro League Season 24 2026 | 2149 | 41 | 41 | -0.0073 | — |
| Thunderpick World Championship 2026 | 2190 | 14 | 14 | +0.0067 | — |
| PGL Masters Bucharest 2026 | 2204 | 41 | 41 | +0.0052 | — |
| IEM Beijing 2026 | 2245 | 30 | 30 | +0.0094 | — |
| BLAST Rivals Fall 2026 | 2275 | 13 | 13 | -0.0062 | — |

## `favourite_longshot`: no evidence

the market's price rescaled in logit space by a factor fitted walk-forward on earlier Events (favourite-longshot bias), then blended as the anchor does.

Verdict: the interval of the log-loss difference includes zero.

| On the 735 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.6258 | 0.2152 | 66.4% |
| favourite_longshot | 0.6263 | 0.2153 | 66.3% |

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
| IEM Rio 2026 | 1700 | 22 | 22 | +0.0020 | — |
| BLAST Rivals Spring 2026 | 1730 | 8 | 8 | -0.0027 | — |
| PGL Astana 2026 | 1743 | 41 | 41 | +0.0004 | — |
| IEM Atlanta 2026 | 1743 | 30 | 30 | -0.0031 | — |
| CS Asia Championships 2026 | 1814 | 0 | 0 | — | — |
| IEM Cologne Major 2026 | 1844 | 104 | 104 | +0.0006 | — |
| XSE Pro League 2026 | 1950 | 0 | 0 | — | — |
| BLAST Bounty Summer 2026 | 1990 | 30 | 30 | +0.0086 | — |
| Esports World Cup 2026 | 2021 | 56 | 56 | -0.0002 | — |
| BLAST Open Porto Fall 2026 | 2077 | 29 | 29 | +0.0000 | — |
| FISSURE PLAYGROUND Season 3 2026 | 2106 | 29 | 29 | +0.0034 | — |
| StarLadder StarSeries Fall 2026 | 2135 | 14 | 14 | -0.0012 | — |
| ESL Pro League Season 24 2026 | 2149 | 41 | 41 | -0.0004 | — |
| Thunderpick World Championship 2026 | 2190 | 14 | 14 | -0.0003 | — |
| PGL Masters Bucharest 2026 | 2204 | 41 | 41 | -0.0014 | — |
| IEM Beijing 2026 | 2245 | 30 | 30 | -0.0001 | — |
| BLAST Rivals Fall 2026 | 2275 | 13 | 13 | +0.0001 | — |

## `own_rating`: promote

Glicko-2 team rating updated after every Match finished before the cutoff (teams joined by team_key), with format-calibrated map scale blended with the market as the anchor does.

Verdict: the interval of the log-loss improvement excludes zero and the Matches with a market line are not worse.

| On the 2168 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.8035 | 0.2438 | 65.1% |
| own_rating | 0.6348 | 0.2219 | 64.2% |

### Direct comparison with Valve rating (2167 paired Matches, 61 Events)

Reference: Valve Regional Standings (unblended). Δ log-loss -0.2367 (90% interval [-0.2991, -0.1786], detectable 0.0926), Δ Brier -0.0321, Δ accuracy -0.0106.

| On the 2167 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| Valve rating | 0.8794 | 0.2575 | 64.7% |
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
| IEM Rio 2026 | 1700 | 30 | 30 | +0.0331 | — |
| BLAST Rivals Spring 2026 | 1730 | 13 | 13 | -0.0675 | — |
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
| ESL Pro League Season 24 2026 | 2149 | 39 | 39 | -0.0572 | — |
| Thunderpick World Championship 2026 | 2190 | 14 | 14 | -0.0122 | — |
| PGL Masters Bucharest 2026 | 2204 | 41 | 41 | -0.1074 | — |
| IEM Beijing 2026 | 2245 | 30 | 30 | -0.0533 | — |
| BLAST Rivals Fall 2026 | 2275 | 13 | 13 | -0.0267 | — |

## `recent_form`: no evidence

Recent results over the last 45 days (14-day half-life) weighted by opponent strength from the point-in-time Glicko-2 rating, added as a fitted logit offset to the anchor.

Verdict: the interval of the log-loss difference includes zero.

| On the 1980 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.7845 | 0.2408 | 65.4% |
| recent_form | 0.7837 | 0.2423 | 65.3% |

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
| IEM Rio 2026 | 1700 | 27 | 27 | -0.0016 | — |
| BLAST Rivals Spring 2026 | 1730 | 12 | 12 | -0.0458 | — |
| PGL Astana 2026 | 1743 | 34 | 34 | +0.0063 | — |
| IEM Atlanta 2026 | 1743 | 26 | 26 | -0.0129 | — |
| CS Asia Championships 2026 | 1814 | 27 | 27 | +0.0244 | — |
| IEM Cologne Major 2026 | 1844 | 101 | 101 | +0.0143 | — |
| XSE Pro League 2026 | 1950 | 34 | 34 | +0.0137 | — |
| BLAST Bounty Summer 2026 | 1990 | 19 | 19 | +0.0090 | — |
| Esports World Cup 2026 | 2021 | 52 | 52 | -0.0066 | — |
| BLAST Open Porto Fall 2026 | 2077 | 28 | 28 | +0.0205 | — |
| FISSURE PLAYGROUND Season 3 2026 | 2106 | 27 | 27 | +0.0009 | — |
| StarLadder StarSeries Fall 2026 | 2135 | 13 | 13 | -0.0027 | — |
| ESL Pro League Season 24 2026 | 2149 | 39 | 39 | -0.0059 | — |
| Thunderpick World Championship 2026 | 2190 | 13 | 13 | -0.0078 | — |
| PGL Masters Bucharest 2026 | 2204 | 36 | 36 | -0.0048 | — |
| IEM Beijing 2026 | 2245 | 28 | 28 | +0.0323 | — |
| BLAST Rivals Fall 2026 | 2275 | 13 | 13 | +0.0139 | — |

## `head_to_head`: no evidence

Recency-weighted head-to-head series record between the two teams (180-day half-life, shrunk toward even), added as a fitted logit offset to the anchor.

Verdict: the interval of the log-loss difference includes zero.

| On the 1280 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.7402 | 0.2352 | 65.9% |
| head_to_head | 0.7363 | 0.2358 | 65.2% |

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
| IEM Rio 2026 | 1700 | 24 | 24 | +0.0528 | — |
| BLAST Rivals Spring 2026 | 1730 | 12 | 12 | -0.0029 | — |
| PGL Astana 2026 | 1743 | 22 | 22 | +0.0401 | — |
| IEM Atlanta 2026 | 1743 | 25 | 25 | -0.0139 | — |
| CS Asia Championships 2026 | 1814 | 20 | 20 | -0.0440 | — |
| IEM Cologne Major 2026 | 1844 | 70 | 70 | +0.0021 | — |
| XSE Pro League 2026 | 1950 | 20 | 20 | +0.0036 | — |
| BLAST Bounty Summer 2026 | 1990 | 19 | 19 | -0.0132 | — |
| Esports World Cup 2026 | 2021 | 42 | 42 | +0.0425 | — |
| BLAST Open Porto Fall 2026 | 2077 | 24 | 24 | +0.0063 | — |
| FISSURE PLAYGROUND Season 3 2026 | 2106 | 16 | 16 | -0.0193 | — |
| StarLadder StarSeries Fall 2026 | 2135 | 11 | 11 | -0.0189 | — |
| ESL Pro League Season 24 2026 | 2149 | 30 | 30 | -0.0165 | — |
| Thunderpick World Championship 2026 | 2190 | 12 | 12 | -0.0128 | — |
| PGL Masters Bucharest 2026 | 2204 | 34 | 34 | -0.0290 | — |
| IEM Beijing 2026 | 2245 | 30 | 30 | -0.0179 | — |
| BLAST Rivals Fall 2026 | 2275 | 13 | 13 | -0.0607 | — |

## `llm_adjustment`: not testable

The three-seat debate's adjustment (HybridAnchored, debate mode `always`, the first seating) on the Matches with a roster flag, from recorded answers.

Verdict: no LLM answers in this run; produce them with `cargo run -p kallbet-ai -- lab --llm <LLM_FILE> --answers <ANSWERS_FILE> --budget claude=<RUNS> --budget codex=<RUNS> --budget antigravity=<RUNS>`.

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

## `lineup_strength`: not testable

Mean HLTV player rating of the 5 playing starters from the pre-cutoff Liquipedia roster (90-day half-life, shrunk toward prior 1.00 with weight 6 maps; debutants/unknown stand-ins take the 1.00 prior when at least 3 of 5 starters on each team have HLTV history, else uncovered), added as a fitted logit offset to the anchor.

Verdict: computed on no Match: it needs pre-cutoff Liquipedia rosters, prior HLTV player ratings for at least 3 of 5 starters on both teams, and at least 30 anchored training Matches.

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| BLAST Premier Groups Fall 2024 | 0 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| IEM Cologne 2024 | 26 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| BLAST Premier Showdown Fall 2024 | 75 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| BetBoom Dacha Belgrade season 2 2024 | 75 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| ESL Pro League Season 20 2024 | 115 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| BLAST Premier Final Fall 2024 | 194 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| IEM Rio 2024 | 207 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| Thunderpick World Championship 2024 | 236 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| BLAST Premier World Final 2024 | 236 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| Perfect World Major Shanghai: American RMR 2024 | 277 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| Perfect World Major Shanghai: European RMR A 2024 | 311 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| Perfect World Major Shanghai: European RMR B 2024 | 346 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| Perfect World Major Shanghai Major 2024 | 380 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| BLAST Premier Bounty Closed Qualifier 2025 | 453 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| BLAST Premier Bounty season 1 2025 | 477 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| IEM Katowice 2025 | 484 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| PGL Cluj-Napoca 2025 | 533 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| ESL Pro League Season 21 2025 | 574 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| BLAST Open Lisbon Spring 2025 | 614 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| PGL Bucharest 2025 | 643 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| BLAST.tv Major Austin: European Regional Qualifier 2025 | 684 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| IEM Melbourne 2025 | 719 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| BLAST Rivals Spring 2025 | 748 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| PGL Astana 2025 | 761 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| IEM Dallas 2025 | 802 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| BLAST.tv Major Austin 2025 | 831 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| FISSURE PLAYGROUND Season 1 2025 | 937 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| IEM Cologne 2025 | 964 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| BLAST Bounty Fall 2025 | 1013 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| Esports World Cup 2025 | 1044 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| BLAST Open London Fall 2025 | 1060 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| FISSURE PLAYGROUND Season 2 2025 | 1089 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| ESL Pro League Season 22 2025 | 1129 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| CS Asia Championships 2025 | 1202 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| Thunderpick World Championship 2025 | 1202 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| PGL Masters Bucharest 2025 | 1246 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| IEM Chengdu 2025 | 1287 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| BLAST Rivals Fall 2025 | 1317 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| StarLadder Major Budapest 2025 | 1330 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| BLAST Bounty Winter 2026 | 1436 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| IEM Krakow 2026 | 1467 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| PGL Cluj-Napoca 2026 | 1517 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| ESL Pro League Season 23 2026 | 1558 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| BLAST Open Rotterdam Spring 2026 | 1632 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| PGL Bucharest 2026 | 1661 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| IEM Rio 2026 | 1700 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| BLAST Rivals Spring 2026 | 1730 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| PGL Astana 2026 | 1743 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| IEM Atlanta 2026 | 1743 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| CS Asia Championships 2026 | 1814 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| IEM Cologne Major 2026 | 1844 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| XSE Pro League 2026 | 1950 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| BLAST Bounty Summer 2026 | 1990 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| Esports World Cup 2026 | 2021 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| BLAST Open Porto Fall 2026 | 2077 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| FISSURE PLAYGROUND Season 3 2026 | 2106 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| StarLadder StarSeries Fall 2026 | 2135 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| ESL Pro League Season 24 2026 | 2149 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| Thunderpick World Championship 2026 | 2190 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| PGL Masters Bucharest 2026 | 2204 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| IEM Beijing 2026 | 2245 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| BLAST Rivals Fall 2026 | 2275 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |

## `player_form`: not testable

Mean recent-minus-longer-run HLTV rating form of the 5 playing starters from the pre-cutoff Liquipedia roster (30d/14d half-life vs 180d/90d half-life, shrunk toward neutral 0.0 with weight 4 maps; debutants/players without 2+ baseline & recent Matches take 0.0 when at least 3 of 5 starters on each team have form, else uncovered), added as a fitted logit offset to the anchor.

Verdict: computed on no Match: it needs pre-cutoff Liquipedia rosters, recent and baseline HLTV player ratings for at least 3 of 5 starters on both teams, and at least 30 anchored training Matches.

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| BLAST Premier Groups Fall 2024 | 0 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| IEM Cologne 2024 | 26 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| BLAST Premier Showdown Fall 2024 | 75 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| BetBoom Dacha Belgrade season 2 2024 | 75 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| ESL Pro League Season 20 2024 | 115 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| BLAST Premier Final Fall 2024 | 194 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| IEM Rio 2024 | 207 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| Thunderpick World Championship 2024 | 236 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| BLAST Premier World Final 2024 | 236 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| Perfect World Major Shanghai: American RMR 2024 | 277 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| Perfect World Major Shanghai: European RMR A 2024 | 311 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| Perfect World Major Shanghai: European RMR B 2024 | 346 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| Perfect World Major Shanghai Major 2024 | 380 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| BLAST Premier Bounty Closed Qualifier 2025 | 453 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| BLAST Premier Bounty season 1 2025 | 477 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| IEM Katowice 2025 | 484 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| PGL Cluj-Napoca 2025 | 533 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| ESL Pro League Season 21 2025 | 574 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| BLAST Open Lisbon Spring 2025 | 614 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| PGL Bucharest 2025 | 643 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| BLAST.tv Major Austin: European Regional Qualifier 2025 | 684 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| IEM Melbourne 2025 | 719 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| BLAST Rivals Spring 2025 | 748 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| PGL Astana 2025 | 761 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| IEM Dallas 2025 | 802 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| BLAST.tv Major Austin 2025 | 831 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| FISSURE PLAYGROUND Season 1 2025 | 937 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| IEM Cologne 2025 | 964 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| BLAST Bounty Fall 2025 | 1013 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| Esports World Cup 2025 | 1044 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| BLAST Open London Fall 2025 | 1060 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| FISSURE PLAYGROUND Season 2 2025 | 1089 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| ESL Pro League Season 22 2025 | 1129 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| CS Asia Championships 2025 | 1202 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| Thunderpick World Championship 2025 | 1202 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| PGL Masters Bucharest 2025 | 1246 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| IEM Chengdu 2025 | 1287 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| BLAST Rivals Fall 2025 | 1317 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| StarLadder Major Budapest 2025 | 1330 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| BLAST Bounty Winter 2026 | 1436 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| IEM Krakow 2026 | 1467 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| PGL Cluj-Napoca 2026 | 1517 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| ESL Pro League Season 23 2026 | 1558 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| BLAST Open Rotterdam Spring 2026 | 1632 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| PGL Bucharest 2026 | 1661 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| IEM Rio 2026 | 1700 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| BLAST Rivals Spring 2026 | 1730 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| PGL Astana 2026 | 1743 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| IEM Atlanta 2026 | 1743 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| CS Asia Championships 2026 | 1814 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| IEM Cologne Major 2026 | 1844 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| XSE Pro League 2026 | 1950 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| BLAST Bounty Summer 2026 | 1990 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| Esports World Cup 2026 | 2021 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| BLAST Open Porto Fall 2026 | 2077 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| FISSURE PLAYGROUND Season 3 2026 | 2106 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| StarLadder StarSeries Fall 2026 | 2135 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| ESL Pro League Season 24 2026 | 2149 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| Thunderpick World Championship 2026 | 2190 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| PGL Masters Bucharest 2026 | 2204 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| IEM Beijing 2026 | 2245 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |
| BLAST Rivals Fall 2026 | 2275 | 0 | 0 | — | `{"beta":null,"training_matches":0}` |

## `per_map_rating`: promote

Per-map Elo from completed Liquipedia series, averaged over equally likely map sets and blended with the market.

Verdict: the interval of the log-loss improvement excludes zero and the Matches with a market line are not worse.

| On the 2109 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.8110 | 0.2456 | 64.8% |
| per_map_rating | 0.6448 | 0.2268 | 63.4% |

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
| IEM Rio 2026 | 1700 | 30 | 30 | +0.0455 | — |
| BLAST Rivals Spring 2026 | 1730 | 13 | 13 | -0.0637 | — |
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
| ESL Pro League Season 24 2026 | 2149 | 35 | 35 | -0.0046 | — |
| Thunderpick World Championship 2026 | 2190 | 14 | 14 | -0.0144 | — |
| PGL Masters Bucharest 2026 | 2204 | 41 | 41 | -0.1146 | — |
| IEM Beijing 2026 | 2245 | 30 | 30 | -0.0499 | — |
| BLAST Rivals Fall 2026 | 2275 | 13 | 13 | -0.0299 | — |

## `bo1_veto_sim`: not testable

Bo1 ban simulation from past HLTV vetoes and map results, blended with the market.

Verdict: No complete HLTV Bo1 vetoes linked to finished PandaScore Matches in the archive.

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

## `roster_stand_in`: no evidence

A stand-in in the active five, from the pre-cutoff Liquipedia revision..

Verdict: the interval of the log-loss difference includes zero.

| On the 791 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.8951 | 0.2531 | 65.9% |
| roster_stand_in | 0.8956 | 0.2502 | 66.4% |

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
| BLAST Open Lisbon Spring 2025 | 614 | 0 | 0 | — | — |
| PGL Bucharest 2025 | 643 | 35 | 35 | +0.0000 | — |
| BLAST.tv Major Austin: European Regional Qualifier 2025 | 684 | 0 | 0 | — | — |
| IEM Melbourne 2025 | 719 | 0 | 0 | — | — |
| BLAST Rivals Spring 2025 | 748 | 13 | 13 | +0.0000 | — |
| PGL Astana 2025 | 761 | 41 | 41 | +0.0000 | — |
| IEM Dallas 2025 | 802 | 0 | 0 | — | — |
| BLAST.tv Major Austin 2025 | 831 | 106 | 106 | +0.0131 | — |
| FISSURE PLAYGROUND Season 1 2025 | 937 | 0 | 0 | — | — |
| IEM Cologne 2025 | 964 | 0 | 0 | — | — |
| BLAST Bounty Fall 2025 | 1013 | 7 | 7 | +0.0000 | — |
| Esports World Cup 2025 | 1044 | 16 | 16 | +0.0000 | — |
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

## `roster_change_30d`: harmful

An active-five change bounded to the last 30 days by dated lineup observations..

Verdict: the interval of the log-loss difference is above zero: worse than the anchor.

| On the 585 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.9228 | 0.2580 | 65.3% |
| roster_change_30d | 0.9321 | 0.2581 | 65.3% |

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
| BLAST Open Lisbon Spring 2025 | 614 | 0 | 0 | — | — |
| PGL Bucharest 2025 | 643 | 25 | 25 | +0.0172 | — |
| BLAST.tv Major Austin: European Regional Qualifier 2025 | 684 | 0 | 0 | — | — |
| IEM Melbourne 2025 | 719 | 0 | 0 | — | — |
| BLAST Rivals Spring 2025 | 748 | 13 | 13 | -0.0328 | — |
| PGL Astana 2025 | 761 | 32 | 32 | +0.0121 | — |
| IEM Dallas 2025 | 802 | 0 | 0 | — | — |
| BLAST.tv Major Austin 2025 | 831 | 80 | 80 | +0.0053 | — |
| FISSURE PLAYGROUND Season 1 2025 | 937 | 0 | 0 | — | — |
| IEM Cologne 2025 | 964 | 0 | 0 | — | — |
| BLAST Bounty Fall 2025 | 1013 | 4 | 4 | -0.0287 | — |
| Esports World Cup 2025 | 1044 | 7 | 7 | -0.0114 | — |
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

## `roster_stability`: no evidence

Days since the current five was first observed after a different five..

Verdict: the interval of the log-loss difference includes zero.

| On the 332 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 1.0458 | 0.2826 | 61.1% |
| roster_stability | 1.0611 | 0.2870 | 61.1% |

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
| BLAST Open Lisbon Spring 2025 | 614 | 0 | 0 | — | — |
| PGL Bucharest 2025 | 643 | 29 | 29 | +0.0174 | — |
| BLAST.tv Major Austin: European Regional Qualifier 2025 | 684 | 0 | 0 | — | — |
| IEM Melbourne 2025 | 719 | 0 | 0 | — | — |
| BLAST Rivals Spring 2025 | 748 | 9 | 9 | +0.0175 | — |
| PGL Astana 2025 | 761 | 30 | 30 | -0.0013 | — |
| IEM Dallas 2025 | 802 | 0 | 0 | — | — |
| BLAST.tv Major Austin 2025 | 831 | 86 | 86 | +0.0085 | — |
| FISSURE PLAYGROUND Season 1 2025 | 937 | 0 | 0 | — | — |
| IEM Cologne 2025 | 964 | 0 | 0 | — | — |
| BLAST Bounty Fall 2025 | 1013 | 7 | 7 | +0.0728 | — |
| Esports World Cup 2025 | 1044 | 16 | 16 | +0.0294 | — |
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

## `roster_coach_change`: no evidence

An emergency coach substitute or a change from the previously observed active coach set..

Verdict: the interval of the log-loss difference includes zero.

| On the 728 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.8535 | 0.2478 | 66.1% |
| roster_coach_change | 0.8557 | 0.2484 | 66.1% |

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
| BLAST Open Lisbon Spring 2025 | 614 | 0 | 0 | — | — |
| PGL Bucharest 2025 | 643 | 33 | 33 | +0.0239 | — |
| BLAST.tv Major Austin: European Regional Qualifier 2025 | 684 | 0 | 0 | — | — |
| IEM Melbourne 2025 | 719 | 0 | 0 | — | — |
| BLAST Rivals Spring 2025 | 748 | 13 | 13 | +0.0000 | — |
| PGL Astana 2025 | 761 | 33 | 33 | -0.0163 | — |
| IEM Dallas 2025 | 802 | 0 | 0 | — | — |
| BLAST.tv Major Austin 2025 | 831 | 105 | 105 | +0.0091 | — |
| FISSURE PLAYGROUND Season 1 2025 | 937 | 0 | 0 | — | — |
| IEM Cologne 2025 | 964 | 0 | 0 | — | — |
| BLAST Bounty Fall 2025 | 1013 | 7 | 7 | +0.0000 | — |
| Esports World Cup 2025 | 1044 | 15 | 15 | -0.0261 | — |
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

## `rest_schedule_load`: no evidence

Team differences in rest, Matches and maps in the preceding 48 hours, and a second Match on the same UTC day.

Verdict: the interval of the log-loss difference includes zero.

| On the 2128 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.8019 | 0.2437 | 65.1% |
| rest_schedule_load | 0.8018 | 0.2442 | 65.2% |

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
| IEM Rio 2026 | 1700 | 30 | 30 | -0.0243 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.08697553882467349,0.1429197126404185,0.3152849797117254,0.2541533737078092]}` |
| BLAST Rivals Spring 2026 | 1730 | 13 | 13 | -0.0135 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.09211881627091122,0.1773685183398654,0.30716007838201537,0.2519570447467008]}` |
| PGL Astana 2026 | 1743 | 38 | 38 | +0.0236 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.10563313076902339,0.19603655202823445,0.2961534315892791,0.253568289480063]}` |
| IEM Atlanta 2026 | 1743 | 30 | 30 | -0.0293 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.10563313076902339,0.19603655202823445,0.2961534315892791,0.253568289480063]}` |
| CS Asia Championships 2026 | 1814 | 30 | 30 | +0.0135 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.0972998328203799,0.15168971625366096,0.33400350060237693,0.22741043708877642]}` |
| IEM Cologne Major 2026 | 1844 | 104 | 104 | -0.0072 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.10522076662426579,0.16010978562183284,0.3022035822763096,0.24654697937994483]}` |
| XSE Pro League 2026 | 1950 | 36 | 36 | +0.0033 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.11985717623001654,0.14538603763719696,0.327918388518445,0.24925362725741257]}` |
| BLAST Bounty Summer 2026 | 1990 | 28 | 28 | -0.0248 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.11684887584516852,0.13335813606036484,0.33369831500981373,0.24942325478632432]}` |
| Esports World Cup 2026 | 2021 | 55 | 55 | +0.0080 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.1278507931087138,0.1485650322177915,0.34075968800288103,0.2495864404445107]}` |
| BLAST Open Porto Fall 2026 | 2077 | 29 | 29 | +0.0046 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.1161871460603242,0.1407268998598543,0.3318600059437439,0.24861500310130663]}` |
| FISSURE PLAYGROUND Season 3 2026 | 2106 | 28 | 28 | +0.0052 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.13902219741228686,0.1390842873206674,0.3278133509458842,0.254208505313624]}` |
| StarLadder StarSeries Fall 2026 | 2135 | 14 | 14 | -0.0326 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.1499723857694526,0.12272554537457588,0.3325979369581476,0.258001272495936]}` |
| ESL Pro League Season 24 2026 | 2149 | 39 | 39 | +0.0183 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.16160562991187047,0.13474307645089648,0.3316201597880358,0.27180090008840246]}` |
| Thunderpick World Championship 2026 | 2190 | 14 | 14 | -0.0473 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.15217029884534072,0.1107631347012186,0.3274252495788335,0.27275411973009467]}` |
| PGL Masters Bucharest 2026 | 2204 | 41 | 41 | -0.0160 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.1578174808568625,0.12918469303819374,0.3281034116272674,0.271867062441063]}` |
| IEM Beijing 2026 | 2245 | 30 | 30 | +0.0286 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.1822192809487832,0.15913493195631315,0.30715021241329665,0.2760356236124064]}` |
| BLAST Rivals Fall 2026 | 2275 | 13 | 13 | +0.0102 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.17421637616107272,0.1646127981648909,0.27147588681908236,0.27666853766341204]}` |

## `stakes`: no evidence

Pre-round Swiss records, advancement and elimination Matches, and Playoffs rounds.

Verdict: better overall, but worse on the Matches with a market line (+0.0097).

| On the 630 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.7345 | 0.2398 | 65.2% |
| stakes | 0.6941 | 0.2339 | 65.2% |

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
| IEM Rio 2026 | 1700 | 5 | 5 | +0.0549 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.2994936079984076,-0.29803246117751203,0.3080874203006721,0.07812472282021703,0.0,-0.5798847072333368,-0.5348844728288247,-0.5225995820906056]}` |
| BLAST Rivals Spring 2026 | 1730 | 5 | 5 | -0.0381 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.2994936079984075,-0.2980324611775121,0.308087420300672,0.07812472282021707,0.0,-0.5764907040868776,-0.5275821172615284,-0.5092982173279181]}` |
| PGL Astana 2026 | 1743 | 7 | 7 | +0.0437 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.2994936079984076,-0.29803246117751203,0.3080874203006721,0.07812472282021703,0.0,-0.5909768528879434,-0.5062732501400639,-0.4968305628759865]}` |
| IEM Atlanta 2026 | 1743 | 5 | 5 | -0.0150 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.2994936079984076,-0.29803246117751203,0.3080874203006721,0.07812472282021703,0.0,-0.5909768528879434,-0.5062732501400639,-0.4968305628759865]}` |
| CS Asia Championships 2026 | 1814 | 5 | 5 | -0.3343 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.2994936079984076,-0.29803246117751203,0.3080874203006721,0.07812472282021703,0.0,-0.5982297629201211,-0.4812928857946602,-0.4785027577486413]}` |
| IEM Cologne Major 2026 | 1844 | 106 | 106 | +0.0280 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.2994936079984075,-0.2980324611775121,0.308087420300672,0.07812472282021708,0.0,-0.607671366214078,-0.46594223609139096,-0.5385796967229389]}` |
| XSE Pro League 2026 | 1950 | 7 | 7 | -0.0425 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.28657340459345465,-0.25765080030199145,0.3300598606711892,0.015904913268416695,0.0,-0.600004840307362,-0.4676834796998614,-0.531152103110606]}` |
| BLAST Bounty Summer 2026 | 1990 | 7 | 7 | +0.0359 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.28657340459345465,-0.25765080030199145,0.3300598606711892,0.015904913268416695,0.0,-0.6066300461599778,-0.45188234193605886,-0.5176559639943096]}` |
| Esports World Cup 2026 | 2021 | 7 | 7 | -0.0526 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.2865734045791964,-0.25765080030452975,0.3300598607448935,0.01590491330205931,0.0,-0.5906289484013874,-0.43431354719488874,-0.5425667150485816]}` |
| BLAST Open Porto Fall 2026 | 2077 | 5 | 5 | +0.0929 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.28657340459345465,-0.25765080030199145,0.3300598606711892,0.015904913268416695,0.0,-0.6000550479164563,-0.43229493109643063,-0.530760717358222]}` |
| FISSURE PLAYGROUND Season 3 2026 | 2106 | 5 | 5 | +0.0530 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.28657340459345465,-0.25765080030199145,0.3300598606711892,0.015904913268416695,0.0,-0.5937230850907909,-0.4315511364093259,-0.5238680372432079]}` |
| StarLadder StarSeries Fall 2026 | 2135 | 1 | 1 | +0.1949 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.2865734045791964,-0.25765080030452975,0.3300598607448935,0.01590491330205931,0.0,-0.5940798638644124,-0.41670431939928987,-0.5229941774211007]}` |
| ESL Pro League Season 24 2026 | 2149 | 7 | 7 | +0.1166 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.2865734045791964,-0.25765080030452975,0.3300598607448935,0.01590491330205931,0.0,-0.5940798638644124,-0.41670431939928987,-0.5120644621378405]}` |
| Thunderpick World Championship 2026 | 2190 | 3 | 3 | -0.0632 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.2865734045791964,-0.25765080030452975,0.3300598607448935,0.01590491330205931,0.0,-0.5776671925365907,-0.41311633075591675,-0.5083974585571093]}` |
| PGL Masters Bucharest 2026 | 2204 | 7 | 7 | -0.1783 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.28657340459345465,-0.25765080030199145,0.3300598606711892,0.015904913268416695,0.0,-0.5776671925365907,-0.42955604018022664,-0.4977636299402084]}` |
| IEM Beijing 2026 | 2245 | 5 | 5 | -0.0220 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.28657340457919633,-0.2576508003045298,0.33005986074489346,0.015904913302059422,0.0,-0.5791280834636199,-0.47436752823697176,-0.48512268197888214]}` |
| BLAST Rivals Fall 2026 | 2275 | 5 | 5 | -0.0348 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.28657340459345465,-0.25765080030199145,0.3300598606711892,0.015904913268416695,0.0,-0.5813741751078846,-0.4754733592896696,-0.4727331532495836]}` |

## `lan_online`: promote

Team win rates in the current environment, with separate weights for LAN and online.

Verdict: the interval of the log-loss improvement excludes zero and the Matches with a market line are not worse.

| On the 752 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.8800 | 0.2500 | 65.7% |
| lan_online | 0.8734 | 0.2493 | 66.0% |

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
| BLAST Open Lisbon Spring 2025 | 614 | 0 | 0 | — | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.5095586131209192,-0.04241380923139176]}` |
| PGL Bucharest 2025 | 643 | 40 | 40 | -0.0208 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.5095586131209192,-0.04241380923139176]}` |
| BLAST.tv Major Austin: European Regional Qualifier 2025 | 684 | 0 | 0 | — | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.6531213942208257,-0.04241380923139176]}` |
| IEM Melbourne 2025 | 719 | 0 | 0 | — | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.6531213942208257,-0.04241380923139176]}` |
| BLAST Rivals Spring 2025 | 748 | 13 | 13 | -0.0168 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.6531213942208257,-0.04241380923139176]}` |
| PGL Astana 2025 | 761 | 39 | 39 | -0.0151 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.6820976795923088,-0.04241380923139176]}` |
| IEM Dallas 2025 | 802 | 0 | 0 | — | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.7574700118977356,-0.04241380923139176]}` |
| BLAST.tv Major Austin 2025 | 831 | 104 | 104 | -0.0077 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.7574700118977356,-0.04241380923139176]}` |
| FISSURE PLAYGROUND Season 1 2025 | 937 | 0 | 0 | — | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.839555780230536,-0.04241380923139176]}` |
| IEM Cologne 2025 | 964 | 0 | 0 | — | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.839555780230536,-0.04241380923139176]}` |
| BLAST Bounty Fall 2025 | 1013 | 28 | 28 | -0.0082 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.839555780230536,-0.04241380923139176]}` |
| Esports World Cup 2025 | 1044 | 16 | 16 | -0.0602 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.8619619893725399,-0.04241380923139176]}` |
| BLAST Open London Fall 2025 | 1060 | 0 | 0 | — | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.9564736961067205,-0.04241380923139176]}` |
| FISSURE PLAYGROUND Season 2 2025 | 1089 | 0 | 0 | — | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.9564736961067205,-0.04241380923139176]}` |
| ESL Pro League Season 22 2025 | 1129 | 0 | 0 | — | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.9564736961067205,-0.04241380923139176]}` |
| CS Asia Championships 2025 | 1202 | 0 | 0 | — | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.9564736961067205,-0.04241380923139176]}` |
| Thunderpick World Championship 2025 | 1202 | 0 | 0 | — | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.9564736961067205,-0.04241380923139176]}` |
| PGL Masters Bucharest 2025 | 1246 | 0 | 0 | — | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.9564736961067205,-0.04241380923139176]}` |
| IEM Chengdu 2025 | 1287 | 0 | 0 | — | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.9564736961067205,-0.04241380923139176]}` |
| BLAST Rivals Fall 2025 | 1317 | 0 | 0 | — | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.9564736961067205,-0.04241380923139176]}` |
| StarLadder Major Budapest 2025 | 1330 | 0 | 0 | — | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.9564736961067205,-0.04241380923139176]}` |
| BLAST Bounty Winter 2026 | 1436 | 0 | 0 | — | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.9564736961067205,-0.04241380923139176]}` |
| IEM Krakow 2026 | 1467 | 0 | 0 | — | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.9564736961067205,-0.04241380923139176]}` |
| PGL Cluj-Napoca 2026 | 1517 | 0 | 0 | — | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.9564736961067205,-0.04241380923139176]}` |
| ESL Pro League Season 23 2026 | 1558 | 0 | 0 | — | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.9564736961067205,-0.04241380923139176]}` |
| BLAST Open Rotterdam Spring 2026 | 1632 | 0 | 0 | — | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.9564736961067205,-0.04241380923139176]}` |
| PGL Bucharest 2026 | 1661 | 0 | 0 | — | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.9564736961067205,-0.04241380923139176]}` |
| IEM Rio 2026 | 1700 | 0 | 0 | — | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.9564736961067205,-0.04241380923139176]}` |
| BLAST Rivals Spring 2026 | 1730 | 0 | 0 | — | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.9564736961067205,-0.04241380923139176]}` |
| PGL Astana 2026 | 1743 | 0 | 0 | — | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.9564736961067205,-0.04241380923139176]}` |
| IEM Atlanta 2026 | 1743 | 0 | 0 | — | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.9564736961067205,-0.04241380923139176]}` |
| CS Asia Championships 2026 | 1814 | 0 | 0 | — | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.9564736961067205,-0.04241380923139176]}` |
| IEM Cologne Major 2026 | 1844 | 64 | 64 | -0.0032 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.9564736961067205,-0.04241380923139176]}` |
| XSE Pro League 2026 | 1950 | 0 | 0 | — | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.9676729708804822,-0.04241380923139176]}` |
| BLAST Bounty Summer 2026 | 1990 | 0 | 0 | — | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.9676729708804822,-0.04241380923139176]}` |
| Esports World Cup 2026 | 2021 | 0 | 0 | — | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.9676729708804822,-0.04241380923139176]}` |
| BLAST Open Porto Fall 2026 | 2077 | 0 | 0 | — | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.9676729708804822,-0.04241380923139176]}` |
| FISSURE PLAYGROUND Season 3 2026 | 2106 | 0 | 0 | — | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.9676729708804822,-0.04241380923139176]}` |
| StarLadder StarSeries Fall 2026 | 2135 | 0 | 0 | — | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.9676729708804822,-0.04241380923139176]}` |
| ESL Pro League Season 24 2026 | 2149 | 0 | 0 | — | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.9676729708804822,-0.04241380923139176]}` |
| Thunderpick World Championship 2026 | 2190 | 0 | 0 | — | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.9676729708804822,-0.04241380923139176]}` |
| PGL Masters Bucharest 2026 | 2204 | 0 | 0 | — | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.9676729708804822,-0.04241380923139176]}` |
| IEM Beijing 2026 | 2245 | 0 | 0 | — | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.9676729708804822,-0.04241380923139176]}` |
| BLAST Rivals Fall 2026 | 2275 | 0 | 0 | — | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.9676729708804822,-0.04241380923139176]}` |

## `series_format`: no evidence

Team differences in Bo1 versus Bo3 performance, conditioned on the current format.

Verdict: the interval of the log-loss difference includes zero.

| On the 1710 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.7655 | 0.2420 | 64.9% |
| series_format | 0.7658 | 0.2424 | 64.8% |

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
| IEM Rio 2026 | 1700 | 24 | 24 | +0.0029 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.13018679354398616,-0.3061668862072434]}` |
| BLAST Rivals Spring 2026 | 1730 | 12 | 12 | +0.0097 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.13018679354398616,-0.29256208543455225]}` |
| PGL Astana 2026 | 1743 | 20 | 20 | -0.0153 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.13018679354398616,-0.27069516670390703]}` |
| IEM Atlanta 2026 | 1743 | 29 | 29 | +0.0091 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.13018679354398616,-0.27069516670390703]}` |
| CS Asia Championships 2026 | 1814 | 29 | 29 | -0.0007 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.13018679354482976,-0.27374948423270273]}` |
| IEM Cologne Major 2026 | 1844 | 95 | 95 | -0.0016 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.14339254890495895,-0.2729826104472706]}` |
| XSE Pro League 2026 | 1950 | 29 | 29 | +0.0014 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.1684610894062286,-0.28671194658388827]}` |
| BLAST Bounty Summer 2026 | 1990 | 19 | 19 | +0.0110 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.18405849330894258,-0.2731653185362491]}` |
| Esports World Cup 2026 | 2021 | 50 | 50 | -0.0016 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.18405849299353555,-0.23502361463238886]}` |
| BLAST Open Porto Fall 2026 | 2077 | 23 | 23 | +0.0021 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.19830831536510216,-0.24203008746141866]}` |
| FISSURE PLAYGROUND Season 3 2026 | 2106 | 24 | 24 | +0.0014 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.19830831580052344,-0.2309671990706471]}` |
| StarLadder StarSeries Fall 2026 | 2135 | 13 | 13 | -0.0005 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.19830831536510216,-0.223578414298777]}` |
| ESL Pro League Season 24 2026 | 2149 | 32 | 32 | +0.0078 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.19830831536510216,-0.2240050095071843]}` |
| Thunderpick World Championship 2026 | 2190 | 13 | 13 | -0.0066 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.19830831536510216,-0.17123894733787437]}` |
| PGL Masters Bucharest 2026 | 2204 | 36 | 36 | +0.0025 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.19830831558448014,-0.19253193838448615]}` |
| IEM Beijing 2026 | 2245 | 29 | 29 | -0.0009 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.19830831536510216,-0.17012121430751184]}` |
| BLAST Rivals Fall 2026 | 2275 | 12 | 12 | -0.0028 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.19830831536510216,-0.17547719335312614]}` |

## `region_travel`: not testable

Difference in playing a LAN outside the team's published VRS qualifying region.

Verdict: computed on no Match: it needs a dated LAN host region and dated VRS qualifying regions for both teams, plus 20 anchored training Matches.

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

## Forward selection

The Candidate Signals marked `promote` enter one combination, one at a time: at each step the one that lowers the walk-forward log-loss most (on the 2243 training-side Matches the anchor predicted) enters, and selection stops when none lowers it. The combination is the combiner (a regularised logistic model without intercept, penalty 10 towards the anchor's weights) over the logits of the market line and the rating plus each member's features, refitted for every Event on the Events that ended before it. "Added" is the log-loss after the step − before it (the anchor before the first).

Considered: `rating_calibration`, `learned_weights`, `own_rating`, `per_map_rating`, `lan_online`. Anchor log-loss 0.8055.

| Order | Entered | Log-loss after | Added | 90% interval | Tried |
|---|---|---|---|---|---|
| 1 | `own_rating` | 0.6329 | -0.1726 | [-0.2324, -0.1163] | `rating_calibration` 0.6344, `learned_weights` 0.6344, `own_rating` 0.6329, `per_map_rating` 0.6332, `lan_online` 0.6343 |

Frozen combination: `own_rating`.

| Event | Fitted on | Combiner weights |
|---|---|---|
| BLAST Premier Groups Fall 2024 | 0 | `{"market":{"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"matches":0,"own_rating.shift":0.0,"rating":1.0}}` |
| IEM Cologne 2024 | 26 | `{"market":{"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"matches":0,"own_rating.shift":0.0,"rating":1.0}}` |
| BLAST Premier Showdown Fall 2024 | 75 | `{"market":{"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"matches":49,"own_rating.shift":0.266127,"rating":0.505061}}` |
| BetBoom Dacha Belgrade season 2 2024 | 75 | `{"market":{"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"matches":49,"own_rating.shift":0.266127,"rating":0.505061}}` |
| ESL Pro League Season 20 2024 | 115 | `{"market":{"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"matches":89,"own_rating.shift":0.181456,"rating":0.467907}}` |
| BLAST Premier Final Fall 2024 | 194 | `{"market":{"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"matches":165,"own_rating.shift":0.11818,"rating":0.384963}}` |
| IEM Rio 2024 | 207 | `{"market":{"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"matches":178,"own_rating.shift":0.119876,"rating":0.398216}}` |
| Thunderpick World Championship 2024 | 236 | `{"market":{"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"matches":207,"own_rating.shift":0.096721,"rating":0.367107}}` |
| BLAST Premier World Final 2024 | 236 | `{"market":{"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"matches":207,"own_rating.shift":0.096721,"rating":0.367107}}` |
| Perfect World Major Shanghai: American RMR 2024 | 277 | `{"market":{"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"matches":248,"own_rating.shift":0.120958,"rating":0.364001}}` |
| Perfect World Major Shanghai: European RMR A 2024 | 311 | `{"market":{"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"matches":282,"own_rating.shift":0.122794,"rating":0.381983}}` |
| Perfect World Major Shanghai: European RMR B 2024 | 346 | `{"market":{"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"matches":312,"own_rating.shift":0.096913,"rating":0.376646}}` |
| Perfect World Major Shanghai Major 2024 | 380 | `{"market":{"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"matches":346,"own_rating.shift":0.119959,"rating":0.375629}}` |
| BLAST Premier Bounty Closed Qualifier 2025 | 453 | `{"market":{"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"matches":419,"own_rating.shift":0.103735,"rating":0.385261}}` |
| BLAST Premier Bounty season 1 2025 | 477 | `{"market":{"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"matches":442,"own_rating.shift":0.131603,"rating":0.394721}}` |
| IEM Katowice 2025 | 484 | `{"market":{"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"matches":449,"own_rating.shift":0.135861,"rating":0.392766}}` |
| PGL Cluj-Napoca 2025 | 533 | `{"market":{"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"matches":496,"own_rating.shift":0.156116,"rating":0.40568}}` |
| ESL Pro League Season 21 2025 | 574 | `{"market":{"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"matches":534,"own_rating.shift":0.140563,"rating":0.401881}}` |
| BLAST Open Lisbon Spring 2025 | 614 | `{"market":{"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"matches":574,"own_rating.shift":0.142408,"rating":0.416287}}` |
| PGL Bucharest 2025 | 643 | `{"market":{"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"matches":603,"own_rating.shift":0.155556,"rating":0.432943}}` |
| BLAST.tv Major Austin: European Regional Qualifier 2025 | 684 | `{"market":{"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"matches":644,"own_rating.shift":0.168349,"rating":0.408077}}` |
| IEM Melbourne 2025 | 719 | `{"market":{"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"matches":679,"own_rating.shift":0.185066,"rating":0.409381}}` |
| BLAST Rivals Spring 2025 | 748 | `{"market":{"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"matches":708,"own_rating.shift":0.181325,"rating":0.411781}}` |
| PGL Astana 2025 | 761 | `{"market":{"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"matches":721,"own_rating.shift":0.185911,"rating":0.41761}}` |
| IEM Dallas 2025 | 802 | `{"market":{"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"matches":762,"own_rating.shift":0.225517,"rating":0.427707}}` |
| BLAST.tv Major Austin 2025 | 831 | `{"market":{"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"matches":791,"own_rating.shift":0.238807,"rating":0.447472}}` |
| FISSURE PLAYGROUND Season 1 2025 | 937 | `{"market":{"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"matches":897,"own_rating.shift":0.228995,"rating":0.43558}}` |
| IEM Cologne 2025 | 964 | `{"market":{"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"matches":924,"own_rating.shift":0.23688,"rating":0.434593}}` |
| BLAST Bounty Fall 2025 | 1013 | `{"market":{"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"matches":973,"own_rating.shift":0.243043,"rating":0.448455}}` |
| Esports World Cup 2025 | 1044 | `{"market":{"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"matches":1003,"own_rating.shift":0.251175,"rating":0.463861}}` |
| BLAST Open London Fall 2025 | 1060 | `{"market":{"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"matches":1019,"own_rating.shift":0.242243,"rating":0.45338}}` |
| FISSURE PLAYGROUND Season 2 2025 | 1089 | `{"market":{"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"matches":1048,"own_rating.shift":0.240604,"rating":0.448398}}` |
| ESL Pro League Season 22 2025 | 1129 | `{"market":{"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"matches":1088,"own_rating.shift":0.252608,"rating":0.457641}}` |
| CS Asia Championships 2025 | 1202 | `{"market":{"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"matches":1161,"own_rating.shift":0.219267,"rating":0.408322}}` |
| Thunderpick World Championship 2025 | 1202 | `{"market":{"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"matches":1161,"own_rating.shift":0.219267,"rating":0.408322}}` |
| PGL Masters Bucharest 2025 | 1246 | `{"market":{"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"matches":1205,"own_rating.shift":0.226644,"rating":0.414807}}` |
| IEM Chengdu 2025 | 1287 | `{"market":{"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"matches":1246,"own_rating.shift":0.228655,"rating":0.414938}}` |
| BLAST Rivals Fall 2025 | 1317 | `{"market":{"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"matches":1276,"own_rating.shift":0.231593,"rating":0.42036}}` |
| StarLadder Major Budapest 2025 | 1330 | `{"market":{"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"matches":1289,"own_rating.shift":0.23092,"rating":0.420984}}` |
| BLAST Bounty Winter 2026 | 1436 | `{"market":{"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"market":0.8,"matches":0,"own_rating.shift":0.0,"rating":0.2},"rating":{"matches":1395,"own_rating.shift":0.233581,"rating":0.425218}}` |
| IEM Krakow 2026 | 1467 | `{"market":{"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"market":0.746963,"matches":31,"own_rating.shift":-0.052084,"rating":-0.009755},"rating":{"matches":1395,"own_rating.shift":0.233581,"rating":0.425218}}` |
| PGL Cluj-Napoca 2026 | 1517 | `{"market":{"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"market":0.813856,"matches":81,"own_rating.shift":-0.063648,"rating":-0.003445},"rating":{"matches":1395,"own_rating.shift":0.233581,"rating":0.425218}}` |
| ESL Pro League Season 23 2026 | 1558 | `{"market":{"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"market":0.834654,"matches":122,"own_rating.shift":-0.047697,"rating":0.063847},"rating":{"matches":1395,"own_rating.shift":0.233581,"rating":0.425218}}` |
| BLAST Open Rotterdam Spring 2026 | 1632 | `{"market":{"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"market":0.78819,"matches":196,"own_rating.shift":-0.016626,"rating":0.036128},"rating":{"matches":1395,"own_rating.shift":0.233581,"rating":0.425218}}` |
| PGL Bucharest 2026 | 1661 | `{"market":{"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"market":0.753735,"matches":225,"own_rating.shift":-0.011697,"rating":0.047801},"rating":{"matches":1395,"own_rating.shift":0.233581,"rating":0.425218}}` |
| IEM Rio 2026 | 1700 | `{"market":{"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"market":0.794818,"matches":264,"own_rating.shift":-0.062783,"rating":0.047671},"rating":{"matches":1395,"own_rating.shift":0.233581,"rating":0.425218}}` |
| BLAST Rivals Spring 2026 | 1730 | `{"market":{"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"market":0.822789,"matches":286,"own_rating.shift":-0.072597,"rating":0.051996},"rating":{"matches":1403,"own_rating.shift":0.239983,"rating":0.43167}}` |
| PGL Astana 2026 | 1743 | `{"market":{"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"market":0.806917,"matches":294,"own_rating.shift":-0.073844,"rating":0.052027},"rating":{"matches":1408,"own_rating.shift":0.240145,"rating":0.432381}}` |
| IEM Atlanta 2026 | 1743 | `{"market":{"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"market":0.806917,"matches":294,"own_rating.shift":-0.073844,"rating":0.052027},"rating":{"matches":1408,"own_rating.shift":0.240145,"rating":0.432381}}` |
| CS Asia Championships 2026 | 1814 | `{"market":{"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"market":0.818279,"matches":365,"own_rating.shift":-0.052472,"rating":0.05508},"rating":{"matches":1408,"own_rating.shift":0.240145,"rating":0.432381}}` |
| IEM Cologne Major 2026 | 1844 | `{"market":{"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"market":0.818279,"matches":365,"own_rating.shift":-0.052472,"rating":0.05508},"rating":{"matches":1438,"own_rating.shift":0.246761,"rating":0.439196}}` |
| XSE Pro League 2026 | 1950 | `{"market":{"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"market":0.913928,"matches":469,"own_rating.shift":-0.083067,"rating":0.039754},"rating":{"matches":1440,"own_rating.shift":0.244362,"rating":0.437587}}` |
| BLAST Bounty Summer 2026 | 1990 | `{"market":{"market":1.0,"matches":0,"own_rating.shift":0.0},"market+rating":{"market":0.913928,"matches":469,"own_rating.shift":-0.083067,"rating":0.039754},"rating":{"matches":1477,"own_rating.shift":0.255898,"rating":0.448401}}` |
| Esports World Cup 2026 | 2021 | `{"market":{"market":1.023412,"matches":1,"own_rating.shift":-0.011703},"market+rating":{"market":0.876137,"matches":498,"own_rating.shift":-0.155161,"rating":0.019721},"rating":{"matches":1477,"own_rating.shift":0.255898,"rating":0.448401}}` |
| BLAST Open Porto Fall 2026 | 2077 | `{"market":{"market":1.023412,"matches":1,"own_rating.shift":-0.011703},"market+rating":{"market":0.848186,"matches":554,"own_rating.shift":-0.213433,"rating":0.028576},"rating":{"matches":1477,"own_rating.shift":0.255898,"rating":0.448401}}` |
| FISSURE PLAYGROUND Season 3 2026 | 2106 | `{"market":{"market":1.023412,"matches":1,"own_rating.shift":-0.011703},"market+rating":{"market":0.879222,"matches":583,"own_rating.shift":-0.215541,"rating":0.025942},"rating":{"matches":1477,"own_rating.shift":0.255898,"rating":0.448401}}` |
| StarLadder StarSeries Fall 2026 | 2135 | `{"market":{"market":1.023412,"matches":1,"own_rating.shift":-0.011703},"market+rating":{"market":0.825499,"matches":612,"own_rating.shift":-0.177105,"rating":0.03731},"rating":{"matches":1477,"own_rating.shift":0.255898,"rating":0.448401}}` |
| ESL Pro League Season 24 2026 | 2149 | `{"market":{"market":1.023412,"matches":1,"own_rating.shift":-0.011703},"market+rating":{"market":0.835236,"matches":626,"own_rating.shift":-0.181607,"rating":0.030227},"rating":{"matches":1477,"own_rating.shift":0.255898,"rating":0.448401}}` |
| Thunderpick World Championship 2026 | 2190 | `{"market":{"market":1.023412,"matches":1,"own_rating.shift":-0.011703},"market+rating":{"market":0.90801,"matches":667,"own_rating.shift":-0.150269,"rating":0.006254},"rating":{"matches":1477,"own_rating.shift":0.255898,"rating":0.448401}}` |
| PGL Masters Bucharest 2026 | 2204 | `{"market":{"market":1.023412,"matches":1,"own_rating.shift":-0.011703},"market+rating":{"market":0.907195,"matches":681,"own_rating.shift":-0.147541,"rating":0.007161},"rating":{"matches":1477,"own_rating.shift":0.255898,"rating":0.448401}}` |
| IEM Beijing 2026 | 2245 | `{"market":{"market":1.023412,"matches":1,"own_rating.shift":-0.011703},"market+rating":{"market":0.928568,"matches":722,"own_rating.shift":-0.12463,"rating":-0.0161},"rating":{"matches":1477,"own_rating.shift":0.255898,"rating":0.448401}}` |
| BLAST Rivals Fall 2026 | 2275 | `{"market":{"market":1.023412,"matches":1,"own_rating.shift":-0.011703},"market+rating":{"market":0.972609,"matches":752,"own_rating.shift":-0.105221,"rating":-0.029226},"rating":{"matches":1477,"own_rating.shift":0.255898,"rating":0.448401}}` |

## Holdout: pending

The Holdout is every Stage that begins at or after 2026-11-25 23:46:46 UTC, or whose Event has a result at or after it (except PandaScore tournament 22378, kept on the training side). No fitting and no selection step reads it; the frozen combination is evaluated on it once, last.

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
