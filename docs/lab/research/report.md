# Signal Lab report

Archive SHA-256 `a556cc446e2f5e75d4494b0fee8b4240e58e8147b3954e23cd959d5cdc0c7c8a` (15405 files), protocol `signal-lab/4`, seed 33, 2000 Event resamples, 90% intervals, floor 100 paired Matches.

Baseline: the anchor: HybridAnchored without LLM, 0.80 market and 0.20 Valve rating logit (`debate::base`). Each Candidate Signal is fitted, for each Event, only on the Matches of the Events that ended before it began, and every datum it reads is observed before the Match cutoff. Differences are variant − anchor on the Matches both predicted (log-loss and Brier: below zero is better). The interval resamples whole Events. Verdict: `promote` when the interval of the log-loss difference is below zero, the Matches with a market line are not worse and the interval of the accuracy difference (same Matches and resamples) is not wholly below zero, since KallBet scores only the picked winner (else `no evidence`: better probabilities but fewer winners picked); `harmful` when it is above zero; `no evidence` otherwise or under the floor; `not testable` when the data could not be had. "Detectable" is the smallest log-loss difference these Matches would show with 80% probability: an effect smaller than it can hide as `no evidence`.

| Candidate Signal | Verdict | Computed on | Paired | Events | Δ log-loss | 90% interval | Detectable | Δ Brier | Δ accuracy | Δ accuracy 90% interval | Market subset (n, Δ log-loss) |
|---|---|---|---|---|---|---|---|---|---|---|---|
| `rating_calibration` | promote | 2037 | 2037 | 55 | -0.1670 | [-0.2333, -0.1083] | 0.0944 | -0.0229 | -0.0010 | [-0.0042, +0.0023] | 776, -0.0158 |
| `learned_weights` | promote | 2087 | 2087 | 56 | -0.1612 | [-0.2224, -0.1049] | 0.0897 | -0.0220 | -0.0014 | [-0.0047, +0.0020] | 777, -0.0119 |
| `market_source_pinnacle` | no evidence | 28 | 28 | 1 | +0.0000 | [+0.0000, +0.0000] | 0.0000 | +0.0000 | +0.0000 | [+0.0000, +0.0000] | 28, +0.0000 |
| `market_source_polymarket` | no evidence | 28 | 28 | 1 | +0.0230 | [+0.0230, +0.0230] | 0.0000 | +0.0073 | +0.0000 | [+0.0000, +0.0000] | 28, +0.0230 |
| `market_source_kalshi` | no evidence | 28 | 28 | 1 | +0.0160 | [+0.0160, +0.0160] | 0.0000 | +0.0052 | +0.0000 | [+0.0000, +0.0000] | 28, +0.0160 |
| `market_source_average` | no evidence | 28 | 28 | 1 | +0.0124 | [+0.0124, +0.0124] | 0.0000 | +0.0041 | +0.0000 | [+0.0000, +0.0000] | 28, +0.0124 |
| `line_movement` | no evidence | 744 | 744 | 20 | +0.0008 | [-0.0035, +0.0066] | 0.0076 | +0.0000 | +0.0013 | [-0.0079, +0.0102] | 744, +0.0008 |
| `favourite_longshot` | no evidence | 746 | 746 | 20 | +0.0008 | [-0.0003, +0.0021] | 0.0019 | +0.0001 | -0.0027 | [-0.0063, +0.0000] | 746, +0.0008 |
| `own_rating` | promote | 2055 | 2028 | 56 | -0.1580 | [-0.2230, -0.0997] | 0.0948 | -0.0193 | -0.0094 | [-0.0239, +0.0030] | 758, -0.0126 |
| `recent_form` | no evidence | 1859 | 1859 | 55 | -0.0008 | [-0.0056, +0.0042] | 0.0074 | +0.0015 | -0.0032 | [-0.0105, +0.0034] | 697, +0.0058 |
| `head_to_head` | no evidence | 1244 | 1244 | 52 | -0.0027 | [-0.0108, +0.0050] | 0.0117 | +0.0009 | -0.0064 | [-0.0176, +0.0052] | 561, +0.0053 |
| `llm_adjustment` | not testable | 0 | 0 | 0 | — | — | — | — | — | — | — |
| `lineup_strength` | promote | 1812 | 1812 | 55 | -0.0026 | [-0.0047, -0.0007] | 0.0030 | +0.0003 | -0.0028 | [-0.0077, +0.0022] | 641, -0.0023 |
| `player_form` | no evidence | 1480 | 1480 | 55 | +0.0001 | [-0.0002, +0.0003] | 0.0003 | +0.0000 | -0.0007 | [-0.0026, +0.0013] | 483, +0.0001 |
| `per_map_rating` | promote | 1982 | 1974 | 56 | -0.1549 | [-0.2235, -0.0930] | 0.0998 | -0.0160 | -0.0132 | [-0.0304, +0.0026] | 724, -0.0111 |
| `bo1_pool_depth` | no evidence (added after list fixed) | 251 | 251 | 10 | -0.0051 | [-0.0110, +0.0028] | 0.0104 | -0.0013 | +0.0040 | [-0.0093, +0.0132] | 49, +0.0109 |
| `bo1_veto_sim` | promote | 252 | 251 | 9 | -0.0818 | [-0.1330, -0.0213] | 0.0816 | -0.0169 | +0.0199 | [+0.0000, +0.0439] | 54, -0.0025 |
| `bo3_veto_sim` | no evidence (added after list fixed) | 1800 | 1771 | 56 | -0.1613 | [-0.2401, -0.0882] | 0.1146 | -0.0139 | -0.0282 | [-0.0448, -0.0136] | 702, -0.0182 |
| `bo5_veto_sim` | no evidence (added after list fixed) | 52 | 52 | 52 | -0.1163 | [-0.3215, +0.0626] | 0.2945 | -0.0080 | -0.0577 | [-0.1346, +0.0000] | 21, +0.0173 |
| `veto_sim` | no evidence (added after list fixed) | 2104 | 2074 | 56 | -0.1505 | [-0.2214, -0.0875] | 0.1018 | -0.0141 | -0.0231 | [-0.0382, -0.0097] | 777, -0.0162 |
| `rank_gap` | promote (added after list fixed) | 2086 | 2086 | 56 | -0.1632 | [-0.2260, -0.1057] | 0.0920 | -0.0226 | -0.0019 | [-0.0073, +0.0029] | 776, -0.0091 |
| `market_rank_rule` | no evidence (added after list fixed) | 2087 | 2087 | 56 | +0.0033 | [-0.0014, +0.0089] | 0.0079 | +0.0014 | -0.0053 | [-0.0137, +0.0030] | 777, +0.0088 |
| `round_diff_rating` | promote (added after list fixed) | 2114 | 2084 | 56 | -0.1538 | [-0.2228, -0.0905] | 0.1012 | -0.0154 | -0.0158 | [-0.0339, +0.0010] | 777, -0.0158 |
| `stacker` | promote (added after list fixed) | 2087 | 2087 | 56 | -0.1616 | [-0.2256, -0.1040] | 0.0931 | -0.0218 | -0.0005 | [-0.0066, +0.0058] | 777, -0.0059 |
| `stacker_no_map_elo` | promote (added after list fixed) | 2087 | 2087 | 56 | -0.1626 | [-0.2265, -0.1051] | 0.0932 | -0.0220 | -0.0019 | [-0.0067, +0.0027] | 777, -0.0085 |
| `roster_stand_in` | no evidence | 1933 | 1900 | 56 | +0.0001 | [-0.0007, +0.0010] | 0.0014 | -0.0002 | +0.0011 | [+0.0000, +0.0022] | 651, -0.0008 |
| `roster_change_30d` | harmful | 1427 | 1427 | 55 | +0.0045 | [+0.0011, +0.0085] | 0.0056 | +0.0003 | +0.0000 | [-0.0029, +0.0025] | 504, +0.0017 |
| `roster_stability` | harmful | 1471 | 1471 | 53 | +0.0041 | [+0.0019, +0.0064] | 0.0034 | +0.0012 | -0.0027 | [-0.0075, +0.0017] | 621, +0.0009 |
| `roster_coach_change` | no evidence | 1831 | 1806 | 56 | +0.0038 | [-0.0004, +0.0079] | 0.0063 | +0.0007 | -0.0022 | [-0.0057, +0.0006] | 615, +0.0020 |
| `rest_schedule_load` | no evidence | 1988 | 1988 | 55 | +0.0006 | [-0.0029, +0.0038] | 0.0051 | +0.0005 | +0.0005 | [-0.0068, +0.0071] | 758, -0.0021 |
| `stakes` | no evidence | 630 | 630 | 51 | -0.0379 | [-0.0650, -0.0121] | 0.0397 | -0.0051 | +0.0000 | [+0.0000, +0.0000] | 216, +0.0177 |
| `lan_online` | promote | 1952 | 1952 | 54 | -0.0045 | [-0.0070, -0.0020] | 0.0038 | -0.0002 | -0.0031 | [-0.0071, +0.0010] | 756, -0.0017 |
| `series_format` | no evidence | 1614 | 1614 | 50 | +0.0004 | [-0.0005, +0.0014] | 0.0014 | +0.0003 | -0.0031 | [-0.0070, +0.0000] | 644, +0.0010 |
| `region_travel` | no evidence | 107 | 107 | 2 | +0.0010 | [+0.0000, +0.0010] | 0.0010 | +0.0003 | -0.0093 | [-0.0094, +0.0000] | 105, +0.0017 |

## Qualifiers and RMRs

A qualifier is an Event whose name contains `RMR` or `Qualifier` (case-insensitive): many unknown teams and near-random outcomes. Its Matches are neither scored nor used to fit a parameter, but they stay in the results, player and map histories, so ratings still update after them.

| Excluded Event | Scored Matches |
|---|---|
| Perfect World Major Shanghai: European RMR A 2024 | 35 |
| Perfect World Major Shanghai: European RMR B 2024 | 34 |
| Perfect World Major Shanghai: American RMR 2024 | 34 |
| BLAST Premier Bounty Closed Qualifier 2025 | 24 |
| BLAST.tv Major Austin: European Regional Qualifier 2025 | 35 |
| **5 Events** | **162** |

### With qualifiers (secondary view)

The same Candidate Signals with the qualifiers scored and fitted on too, so an effect that lives only on lesser-known teams stays visible. The primary verdicts are the ones above: qualifiers never appear in KallBet Leagues.

| Candidate Signal | Verdict (primary) | Paired | Verdict (with qualifiers) | Paired | Δ log-loss | 90% interval |
|---|---|---|---|---|---|---|
| `rating_calibration` | promote | 2037 | promote | 2193 | -0.1759 | [-0.2403, -0.1217] |
| `learned_weights` | promote | 2087 | promote | 2243 | -0.1704 | [-0.2297, -0.1148] |
| `market_source_pinnacle` | no evidence | 28 | no evidence | 28 | +0.0000 | [+0.0000, +0.0000] |
| `market_source_polymarket` | no evidence | 28 | no evidence | 28 | +0.0230 | [+0.0230, +0.0230] |
| `market_source_kalshi` | no evidence | 28 | no evidence | 28 | +0.0160 | [+0.0160, +0.0160] |
| `market_source_average` | no evidence | 28 | no evidence | 28 | +0.0124 | [+0.0124, +0.0124] |
| `line_movement` | no evidence | 744 | no evidence | 744 | +0.0008 | [-0.0035, +0.0066] |
| `favourite_longshot` | no evidence | 746 | no evidence | 746 | +0.0008 | [-0.0003, +0.0021] |
| `own_rating` | promote | 2028 | promote | 2168 | -0.1677 | [-0.2329, -0.1079] |
| `recent_form` | no evidence | 1859 | no evidence | 1980 | -0.0009 | [-0.0053, +0.0036] |
| `head_to_head` | no evidence | 1244 | no evidence | 1280 | -0.0038 | [-0.0127, +0.0047] |
| `llm_adjustment` | not testable | 0 | not testable | 0 | — | — |
| `lineup_strength` | promote | 1812 | promote | 1950 | -0.0026 | [-0.0045, -0.0008] |
| `player_form` | no evidence | 1480 | no evidence | 1567 | +0.0000 | [-0.0002, +0.0003] |
| `per_map_rating` | promote | 1974 | no evidence | 2109 | -0.1654 | [-0.2335, -0.1035] |
| `bo1_pool_depth` | no evidence | 251 | no evidence | 312 | -0.0240 | [-0.0413, -0.0077] |
| `bo1_veto_sim` | promote | 251 | promote | 319 | -0.0877 | [-0.1315, -0.0342] |
| `bo3_veto_sim` | no evidence | 1771 | no evidence | 1850 | -0.1700 | [-0.2491, -0.1005] |
| `bo5_veto_sim` | no evidence | 52 | no evidence | 52 | -0.1163 | [-0.3215, +0.0626] |
| `veto_sim` | no evidence | 2074 | no evidence | 2221 | -0.1569 | [-0.2266, -0.0971] |
| `rank_gap` | promote | 2086 | promote | 2242 | -0.1734 | [-0.2351, -0.1162] |
| `market_rank_rule` | no evidence | 2087 | no evidence | 2243 | +0.0030 | [-0.0014, +0.0081] |
| `round_diff_rating` | promote | 2084 | no evidence | 2240 | -0.1617 | [-0.2321, -0.1016] |
| `stacker` | promote | 2087 | promote | 2243 | -0.1714 | [-0.2341, -0.1138] |
| `stacker_no_map_elo` | promote | 2087 | promote | 2243 | -0.1724 | [-0.2360, -0.1146] |
| `roster_stand_in` | no evidence | 1900 | no evidence | 2056 | -0.0002 | [-0.0019, +0.0015] |
| `roster_change_30d` | harmful | 1427 | harmful | 1563 | +0.0044 | [+0.0015, +0.0080] |
| `roster_stability` | harmful | 1471 | no evidence | 1534 | +0.0045 | [-0.0006, +0.0098] |
| `roster_coach_change` | no evidence | 1806 | harmful | 1903 | +0.0041 | [+0.0001, +0.0094] |
| `rest_schedule_load` | no evidence | 1988 | no evidence | 2128 | -0.0000 | [-0.0043, +0.0040] |
| `stakes` | no evidence | 630 | no evidence | 630 | -0.0379 | [-0.0650, -0.0121] |
| `lan_online` | promote | 1952 | no evidence | 2078 | -0.0051 | [-0.0077, -0.0024] |
| `series_format` | no evidence | 1614 | no evidence | 1710 | +0.0003 | [-0.0010, +0.0015] |
| `region_travel` | no evidence | 107 | no evidence | 107 | +0.0010 | [+0.0000, +0.0010] |

## Tier slice `s` (secondary view)

The slice was chosen after results were seen, so it is a secondary view: the verdict that counts for promotion is the full-archive one above, and the Holdout is unchanged. Main columns evaluate tiers `all`; fits may use Events of tiers `all`; the slice evaluates `s`, fitted walk-forward as before. Scored Matches: tier a: 1155, tier s: 971.

| Candidate Signal | Verdict (full) | Paired (full) | Verdict (`s`) | Paired | Events | Δ log-loss | 90% interval |
|---|---|---|---|---|---|---|---|
| `rating_calibration` | promote | 2037 | promote | 894 | 22 | -0.0942 | [-0.1433, -0.0467] |
| `learned_weights` | promote | 2087 | promote | 943 | 23 | -0.0911 | [-0.1353, -0.0439] |
| `market_source_pinnacle` | no evidence | 28 | not testable | 0 | 0 | — | — |
| `market_source_polymarket` | no evidence | 28 | not testable | 0 | 0 | — | — |
| `market_source_kalshi` | no evidence | 28 | not testable | 0 | 0 | — | — |
| `market_source_average` | no evidence | 28 | not testable | 0 | 0 | — | — |
| `line_movement` | no evidence | 744 | no evidence | 322 | 8 | -0.0035 | [-0.0066, +0.0009] |
| `favourite_longshot` | no evidence | 746 | no evidence | 324 | 8 | +0.0000 | [-0.0005, +0.0007] |
| `own_rating` | promote | 2028 | promote | 929 | 23 | -0.0870 | [-0.1330, -0.0403] |
| `recent_form` | no evidence | 1859 | no evidence | 847 | 22 | -0.0011 | [-0.0100, +0.0073] |
| `head_to_head` | no evidence | 1244 | no evidence | 583 | 22 | +0.0012 | [-0.0098, +0.0123] |
| `llm_adjustment` | not testable | 0 | not testable | 0 | 0 | — | — |
| `lineup_strength` | promote | 1812 | no evidence | 888 | 22 | -0.0018 | [-0.0047, +0.0008] |
| `player_form` | no evidence | 1480 | harmful | 728 | 22 | +0.0003 | [+0.0000, +0.0008] |
| `per_map_rating` | promote | 1974 | promote | 915 | 23 | -0.0844 | [-0.1354, -0.0302] |
| `bo1_pool_depth` | no evidence | 251 | no evidence | 217 | 6 | -0.0034 | [-0.0098, +0.0065] |
| `bo1_veto_sim` | promote | 251 | promote | 217 | 6 | -0.0866 | [-0.1429, -0.0114] |
| `bo3_veto_sim` | no evidence | 1771 | no evidence | 695 | 23 | -0.0708 | [-0.1146, -0.0289] |
| `bo5_veto_sim` | no evidence | 52 | no evidence | 20 | 20 | +0.0819 | [-0.0625, +0.2018] |
| `veto_sim` | no evidence | 2074 | promote | 932 | 23 | -0.0712 | [-0.1129, -0.0288] |
| `rank_gap` | promote | 2086 | promote | 943 | 23 | -0.0926 | [-0.1363, -0.0464] |
| `market_rank_rule` | no evidence | 2087 | no evidence | 943 | 23 | +0.0005 | [-0.0049, +0.0065] |
| `round_diff_rating` | promote | 2084 | no evidence | 940 | 23 | -0.0797 | [-0.1267, -0.0308] |
| `stacker` | promote | 2087 | no evidence | 943 | 23 | -0.0889 | [-0.1310, -0.0445] |
| `stacker_no_map_elo` | promote | 2087 | no evidence | 943 | 23 | -0.0886 | [-0.1308, -0.0437] |
| `roster_stand_in` | no evidence | 1900 | no evidence | 941 | 23 | +0.0002 | [-0.0014, +0.0021] |
| `roster_change_30d` | harmful | 1427 | no evidence | 759 | 23 | +0.0029 | [-0.0012, +0.0081] |
| `roster_stability` | harmful | 1471 | harmful | 717 | 21 | +0.0052 | [+0.0023, +0.0085] |
| `roster_coach_change` | no evidence | 1806 | no evidence | 904 | 23 | +0.0039 | [-0.0025, +0.0100] |
| `rest_schedule_load` | no evidence | 1988 | no evidence | 889 | 22 | +0.0037 | [-0.0017, +0.0087] |
| `stakes` | no evidence | 630 | no evidence | 476 | 21 | -0.0257 | [-0.0532, +0.0098] |
| `lan_online` | promote | 1952 | no evidence | 886 | 22 | -0.0042 | [-0.0081, -0.0004] |
| `series_format` | no evidence | 1614 | no evidence | 798 | 20 | +0.0002 | [-0.0004, +0.0010] |
| `region_travel` | no evidence | 107 | no evidence | 107 | 2 | +0.0010 | [+0.0000, +0.0010] |

## `rating_calibration`: promote

Valve rating's map probability rescaled in logit space by a factor fitted per series format, then blended with the market as the anchor does.

Verdict: the interval of the log-loss improvement excludes zero and the Matches with a market line are not worse.

| On the 2037 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.7921 | 0.2398 | 65.7% |
| rating_calibration | 0.6251 | 0.2169 | 65.6% |

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
| Perfect World Major Shanghai Major 2024 | 277 | 73 | 73 | -0.0368 | — |
| BLAST Premier Bounty season 1 2025 | 350 | 7 | 7 | -0.7720 | — |
| IEM Katowice 2025 | 357 | 47 | 47 | -0.2245 | — |
| PGL Cluj-Napoca 2025 | 406 | 38 | 38 | -0.0638 | — |
| ESL Pro League Season 21 2025 | 447 | 40 | 40 | -0.0589 | — |
| BLAST Open Lisbon Spring 2025 | 487 | 29 | 29 | -0.0724 | — |
| PGL Bucharest 2025 | 516 | 41 | 41 | -0.8629 | — |
| IEM Melbourne 2025 | 557 | 29 | 29 | -0.1656 | — |
| BLAST Rivals Spring 2025 | 586 | 13 | 13 | -0.1887 | — |
| PGL Astana 2025 | 599 | 41 | 41 | -0.9820 | — |
| IEM Dallas 2025 | 640 | 29 | 29 | +0.0060 | — |
| BLAST.tv Major Austin 2025 | 669 | 106 | 106 | -0.2391 | — |
| FISSURE PLAYGROUND Season 1 2025 | 775 | 27 | 27 | -0.6592 | — |
| IEM Cologne 2025 | 802 | 49 | 49 | +0.0449 | — |
| BLAST Bounty Fall 2025 | 851 | 30 | 30 | +0.0424 | — |
| Esports World Cup 2025 | 882 | 16 | 16 | -0.4682 | — |
| BLAST Open London Fall 2025 | 898 | 29 | 29 | -0.4865 | — |
| FISSURE PLAYGROUND Season 2 2025 | 927 | 40 | 40 | -0.2470 | — |
| ESL Pro League Season 22 2025 | 967 | 73 | 73 | -0.7842 | — |
| CS Asia Championships 2025 | 1040 | 30 | 30 | -0.2173 | — |
| Thunderpick World Championship 2025 | 1040 | 14 | 14 | -0.1572 | — |
| PGL Masters Bucharest 2025 | 1084 | 41 | 41 | -0.2627 | — |
| IEM Chengdu 2025 | 1125 | 30 | 30 | -0.0506 | — |
| BLAST Rivals Fall 2025 | 1155 | 13 | 13 | -0.0890 | — |
| StarLadder Major Budapest 2025 | 1168 | 106 | 106 | -0.1478 | — |
| BLAST Bounty Winter 2026 | 1274 | 31 | 31 | -0.0481 | — |
| IEM Krakow 2026 | 1305 | 50 | 50 | -0.0169 | — |
| PGL Cluj-Napoca 2026 | 1355 | 41 | 41 | +0.0323 | — |
| ESL Pro League Season 23 2026 | 1396 | 74 | 74 | -0.0288 | — |
| BLAST Open Rotterdam Spring 2026 | 1470 | 29 | 29 | -0.0202 | — |
| PGL Bucharest 2026 | 1499 | 39 | 39 | +0.0049 | — |
| IEM Rio 2026 | 1538 | 30 | 30 | +0.0380 | — |
| BLAST Rivals Spring 2026 | 1568 | 13 | 13 | -0.0236 | — |
| PGL Astana 2026 | 1581 | 41 | 41 | +0.0033 | — |
| IEM Atlanta 2026 | 1581 | 30 | 30 | -0.0221 | — |
| CS Asia Championships 2026 | 1652 | 30 | 30 | -0.1654 | — |
| IEM Cologne Major 2026 | 1682 | 106 | 106 | +0.0099 | — |
| XSE Pro League 2026 | 1788 | 37 | 37 | -0.0364 | — |
| BLAST Bounty Summer 2026 | 1828 | 29 | 29 | -0.0770 | — |
| Esports World Cup 2026 | 1859 | 56 | 56 | +0.0102 | — |
| BLAST Open Porto Fall 2026 | 1915 | 29 | 29 | +0.0159 | — |
| FISSURE PLAYGROUND Season 3 2026 | 1944 | 29 | 29 | -0.0409 | — |
| StarLadder StarSeries Fall 2026 | 1973 | 14 | 14 | -0.0424 | — |
| ESL Pro League Season 24 2026 | 1987 | 41 | 41 | -0.0364 | — |
| Thunderpick World Championship 2026 | 2028 | 14 | 14 | +0.0016 | — |
| PGL Masters Bucharest 2026 | 2042 | 41 | 41 | -0.0854 | — |
| IEM Beijing 2026 | 2083 | 30 | 30 | -0.0473 | — |
| BLAST Rivals Fall 2026 | 2113 | 13 | 13 | +0.0093 | — |

## `learned_weights`: promote

the anchor's market and Valve rating weights (logit space) learned by the combiner on the Events that ended before, one set per availability of the two inputs, regularised towards 0.80 / 0.20.

Verdict: the interval of the log-loss improvement excludes zero and the Matches with a market line are not worse.

| On the 2087 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.7943 | 0.2398 | 65.8% |
| learned_weights | 0.6331 | 0.2179 | 65.7% |

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
| Perfect World Major Shanghai Major 2024 | 277 | 73 | 73 | -0.0371 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":248,"rating":0.279275}}` |
| BLAST Premier Bounty season 1 2025 | 350 | 7 | 7 | -0.7554 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":321,"rating":0.306278}}` |
| IEM Katowice 2025 | 357 | 47 | 47 | -0.2235 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":328,"rating":0.296999}}` |
| PGL Cluj-Napoca 2025 | 406 | 38 | 38 | -0.0666 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":375,"rating":0.290462}}` |
| ESL Pro League Season 21 2025 | 447 | 40 | 40 | -0.0631 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":413,"rating":0.300148}}` |
| BLAST Open Lisbon Spring 2025 | 487 | 29 | 29 | -0.0766 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":453,"rating":0.314739}}` |
| PGL Bucharest 2025 | 516 | 41 | 41 | -0.8403 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":482,"rating":0.321666}}` |
| IEM Melbourne 2025 | 557 | 29 | 29 | -0.1693 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":523,"rating":0.274126}}` |
| BLAST Rivals Spring 2025 | 586 | 13 | 13 | -0.1959 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":552,"rating":0.279572}}` |
| PGL Astana 2025 | 599 | 41 | 41 | -0.9634 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":565,"rating":0.281504}}` |
| IEM Dallas 2025 | 640 | 29 | 29 | -0.0013 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":606,"rating":0.249378}}` |
| BLAST.tv Major Austin 2025 | 669 | 106 | 106 | -0.2507 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":635,"rating":0.259621}}` |
| FISSURE PLAYGROUND Season 1 2025 | 775 | 27 | 27 | -0.6570 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":741,"rating":0.253836}}` |
| IEM Cologne 2025 | 802 | 49 | 49 | +0.0448 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":768,"rating":0.243914}}` |
| BLAST Bounty Fall 2025 | 851 | 30 | 30 | +0.0388 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":817,"rating":0.254044}}` |
| Esports World Cup 2025 | 882 | 16 | 16 | -0.4679 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":847,"rating":0.264102}}` |
| BLAST Open London Fall 2025 | 898 | 29 | 29 | -0.4864 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":863,"rating":0.26083}}` |
| FISSURE PLAYGROUND Season 2 2025 | 927 | 40 | 40 | -0.2467 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":892,"rating":0.256726}}` |
| ESL Pro League Season 22 2025 | 967 | 73 | 73 | -0.7765 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":932,"rating":0.255388}}` |
| CS Asia Championships 2025 | 1040 | 30 | 30 | -0.2165 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":1005,"rating":0.229193}}` |
| Thunderpick World Championship 2025 | 1040 | 14 | 14 | -0.1634 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":1005,"rating":0.229193}}` |
| PGL Masters Bucharest 2025 | 1084 | 41 | 41 | -0.2635 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":1049,"rating":0.229974}}` |
| IEM Chengdu 2025 | 1125 | 30 | 30 | -0.0495 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":1090,"rating":0.227962}}` |
| BLAST Rivals Fall 2025 | 1155 | 13 | 13 | -0.0859 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":1120,"rating":0.231563}}` |
| StarLadder Major Budapest 2025 | 1168 | 106 | 106 | -0.1479 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":1133,"rating":0.232962}}` |
| BLAST Bounty Winter 2026 | 1274 | 31 | 31 | +0.0000 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.8,"matches":0,"rating":0.2},"rating":{"matches":1239,"rating":0.235417}}` |
| IEM Krakow 2026 | 1305 | 50 | 50 | -0.0136 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.748665,"matches":31,"rating":-0.004371},"rating":{"matches":1239,"rating":0.235417}}` |
| PGL Cluj-Napoca 2026 | 1355 | 41 | 41 | +0.0428 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.812926,"matches":81,"rating":0.005601},"rating":{"matches":1239,"rating":0.235417}}` |
| ESL Pro League Season 23 2026 | 1396 | 74 | 74 | -0.0244 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.833562,"matches":122,"rating":0.070797},"rating":{"matches":1239,"rating":0.235417}}` |
| BLAST Open Rotterdam Spring 2026 | 1470 | 29 | 29 | -0.0207 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.78756,"matches":196,"rating":0.038728},"rating":{"matches":1239,"rating":0.235417}}` |
| PGL Bucharest 2026 | 1499 | 39 | 39 | +0.0079 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.753182,"matches":225,"rating":0.049683},"rating":{"matches":1239,"rating":0.235417}}` |
| IEM Rio 2026 | 1538 | 30 | 30 | +0.0371 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.792052,"matches":264,"rating":0.057811},"rating":{"matches":1239,"rating":0.235417}}` |
| BLAST Rivals Spring 2026 | 1568 | 13 | 13 | -0.0210 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.838199,"matches":292,"rating":0.062866},"rating":{"matches":1241,"rating":0.235738}}` |
| PGL Astana 2026 | 1581 | 41 | 41 | +0.0005 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.821135,"matches":305,"rating":0.064026},"rating":{"matches":1241,"rating":0.235738}}` |
| IEM Atlanta 2026 | 1581 | 30 | 30 | -0.0205 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.821135,"matches":305,"rating":0.064026},"rating":{"matches":1241,"rating":0.235738}}` |
| CS Asia Championships 2026 | 1652 | 30 | 30 | -0.1678 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.831188,"matches":376,"rating":0.063054},"rating":{"matches":1241,"rating":0.235738}}` |
| IEM Cologne Major 2026 | 1682 | 106 | 106 | +0.0073 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.831188,"matches":376,"rating":0.063054},"rating":{"matches":1271,"rating":0.237591}}` |
| XSE Pro League 2026 | 1788 | 37 | 37 | -0.0334 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.922463,"matches":480,"rating":0.052714},"rating":{"matches":1273,"rating":0.237998}}` |
| BLAST Bounty Summer 2026 | 1828 | 30 | 30 | -0.0559 | `{"market":{"market":1.0,"matches":0},"market+rating":{"market":0.922463,"matches":480,"rating":0.052714},"rating":{"matches":1310,"rating":0.239312}}` |
| Esports World Cup 2026 | 1859 | 56 | 56 | +0.0099 | `{"market":{"market":1.02348,"matches":1},"market+rating":{"market":0.881301,"matches":509,"rating":0.044231},"rating":{"matches":1310,"rating":0.239312}}` |
| BLAST Open Porto Fall 2026 | 1915 | 29 | 29 | +0.0105 | `{"market":{"market":1.02348,"matches":1},"market+rating":{"market":0.84838,"matches":565,"rating":0.062575},"rating":{"matches":1310,"rating":0.239312}}` |
| FISSURE PLAYGROUND Season 3 2026 | 1944 | 29 | 29 | -0.0286 | `{"market":{"market":1.02348,"matches":1},"market+rating":{"market":0.878233,"matches":594,"rating":0.060363},"rating":{"matches":1310,"rating":0.239312}}` |
| StarLadder StarSeries Fall 2026 | 1973 | 14 | 14 | -0.0374 | `{"market":{"market":1.02348,"matches":1},"market+rating":{"market":0.82676,"matches":623,"rating":0.065508},"rating":{"matches":1310,"rating":0.239312}}` |
| ESL Pro League Season 24 2026 | 1987 | 41 | 41 | -0.0364 | `{"market":{"market":1.02348,"matches":1},"market+rating":{"market":0.836126,"matches":637,"rating":0.059155},"rating":{"matches":1310,"rating":0.239312}}` |
| Thunderpick World Championship 2026 | 2028 | 14 | 14 | +0.0029 | `{"market":{"market":1.02348,"matches":1},"market+rating":{"market":0.897026,"matches":678,"rating":0.035897},"rating":{"matches":1310,"rating":0.239312}}` |
| PGL Masters Bucharest 2026 | 2042 | 41 | 41 | -0.0802 | `{"market":{"market":1.02348,"matches":1},"market+rating":{"market":0.889965,"matches":692,"rating":0.039133},"rating":{"matches":1310,"rating":0.239312}}` |
| IEM Beijing 2026 | 2083 | 30 | 30 | -0.0556 | `{"market":{"market":1.02348,"matches":1},"market+rating":{"market":0.892155,"matches":733,"rating":0.02075},"rating":{"matches":1310,"rating":0.239312}}` |
| BLAST Rivals Fall 2026 | 2113 | 13 | 13 | +0.0136 | `{"market":{"market":1.02348,"matches":1},"market+rating":{"market":0.942883,"matches":763,"rating":0.001783},"rating":{"matches":1310,"rating":0.239312}}` |

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
| Perfect World Major Shanghai Major 2024 | 277 | 0 | 0 | — | — |
| BLAST Premier Bounty season 1 2025 | 350 | 0 | 0 | — | — |
| IEM Katowice 2025 | 357 | 0 | 0 | — | — |
| PGL Cluj-Napoca 2025 | 406 | 0 | 0 | — | — |
| ESL Pro League Season 21 2025 | 447 | 0 | 0 | — | — |
| BLAST Open Lisbon Spring 2025 | 487 | 0 | 0 | — | — |
| PGL Bucharest 2025 | 516 | 0 | 0 | — | — |
| IEM Melbourne 2025 | 557 | 0 | 0 | — | — |
| BLAST Rivals Spring 2025 | 586 | 0 | 0 | — | — |
| PGL Astana 2025 | 599 | 0 | 0 | — | — |
| IEM Dallas 2025 | 640 | 0 | 0 | — | — |
| BLAST.tv Major Austin 2025 | 669 | 0 | 0 | — | — |
| FISSURE PLAYGROUND Season 1 2025 | 775 | 0 | 0 | — | — |
| IEM Cologne 2025 | 802 | 0 | 0 | — | — |
| BLAST Bounty Fall 2025 | 851 | 0 | 0 | — | — |
| Esports World Cup 2025 | 882 | 0 | 0 | — | — |
| BLAST Open London Fall 2025 | 898 | 0 | 0 | — | — |
| FISSURE PLAYGROUND Season 2 2025 | 927 | 0 | 0 | — | — |
| ESL Pro League Season 22 2025 | 967 | 0 | 0 | — | — |
| CS Asia Championships 2025 | 1040 | 0 | 0 | — | — |
| Thunderpick World Championship 2025 | 1040 | 0 | 0 | — | — |
| PGL Masters Bucharest 2025 | 1084 | 0 | 0 | — | — |
| IEM Chengdu 2025 | 1125 | 0 | 0 | — | — |
| BLAST Rivals Fall 2025 | 1155 | 0 | 0 | — | — |
| StarLadder Major Budapest 2025 | 1168 | 0 | 0 | — | — |
| BLAST Bounty Winter 2026 | 1274 | 0 | 0 | — | — |
| IEM Krakow 2026 | 1305 | 0 | 0 | — | — |
| PGL Cluj-Napoca 2026 | 1355 | 0 | 0 | — | — |
| ESL Pro League Season 23 2026 | 1396 | 0 | 0 | — | — |
| BLAST Open Rotterdam Spring 2026 | 1470 | 0 | 0 | — | — |
| PGL Bucharest 2026 | 1499 | 0 | 0 | — | — |
| IEM Rio 2026 | 1538 | 0 | 0 | — | — |
| BLAST Rivals Spring 2026 | 1568 | 0 | 0 | — | — |
| PGL Astana 2026 | 1581 | 0 | 0 | — | — |
| IEM Atlanta 2026 | 1581 | 0 | 0 | — | — |
| CS Asia Championships 2026 | 1652 | 0 | 0 | — | — |
| IEM Cologne Major 2026 | 1682 | 0 | 0 | — | — |
| XSE Pro League 2026 | 1788 | 0 | 0 | — | — |
| BLAST Bounty Summer 2026 | 1828 | 0 | 0 | — | — |
| Esports World Cup 2026 | 1859 | 0 | 0 | — | — |
| BLAST Open Porto Fall 2026 | 1915 | 0 | 0 | — | — |
| FISSURE PLAYGROUND Season 3 2026 | 1944 | 0 | 0 | — | — |
| StarLadder StarSeries Fall 2026 | 1973 | 0 | 0 | — | — |
| ESL Pro League Season 24 2026 | 1987 | 0 | 0 | — | — |
| Thunderpick World Championship 2026 | 2028 | 0 | 0 | — | — |
| PGL Masters Bucharest 2026 | 2042 | 28 | 28 | +0.0000 | — |
| IEM Beijing 2026 | 2083 | 0 | 0 | — | — |
| BLAST Rivals Fall 2026 | 2113 | 0 | 0 | — | — |

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
| Perfect World Major Shanghai Major 2024 | 277 | 0 | 0 | — | — |
| BLAST Premier Bounty season 1 2025 | 350 | 0 | 0 | — | — |
| IEM Katowice 2025 | 357 | 0 | 0 | — | — |
| PGL Cluj-Napoca 2025 | 406 | 0 | 0 | — | — |
| ESL Pro League Season 21 2025 | 447 | 0 | 0 | — | — |
| BLAST Open Lisbon Spring 2025 | 487 | 0 | 0 | — | — |
| PGL Bucharest 2025 | 516 | 0 | 0 | — | — |
| IEM Melbourne 2025 | 557 | 0 | 0 | — | — |
| BLAST Rivals Spring 2025 | 586 | 0 | 0 | — | — |
| PGL Astana 2025 | 599 | 0 | 0 | — | — |
| IEM Dallas 2025 | 640 | 0 | 0 | — | — |
| BLAST.tv Major Austin 2025 | 669 | 0 | 0 | — | — |
| FISSURE PLAYGROUND Season 1 2025 | 775 | 0 | 0 | — | — |
| IEM Cologne 2025 | 802 | 0 | 0 | — | — |
| BLAST Bounty Fall 2025 | 851 | 0 | 0 | — | — |
| Esports World Cup 2025 | 882 | 0 | 0 | — | — |
| BLAST Open London Fall 2025 | 898 | 0 | 0 | — | — |
| FISSURE PLAYGROUND Season 2 2025 | 927 | 0 | 0 | — | — |
| ESL Pro League Season 22 2025 | 967 | 0 | 0 | — | — |
| CS Asia Championships 2025 | 1040 | 0 | 0 | — | — |
| Thunderpick World Championship 2025 | 1040 | 0 | 0 | — | — |
| PGL Masters Bucharest 2025 | 1084 | 0 | 0 | — | — |
| IEM Chengdu 2025 | 1125 | 0 | 0 | — | — |
| BLAST Rivals Fall 2025 | 1155 | 0 | 0 | — | — |
| StarLadder Major Budapest 2025 | 1168 | 0 | 0 | — | — |
| BLAST Bounty Winter 2026 | 1274 | 0 | 0 | — | — |
| IEM Krakow 2026 | 1305 | 0 | 0 | — | — |
| PGL Cluj-Napoca 2026 | 1355 | 0 | 0 | — | — |
| ESL Pro League Season 23 2026 | 1396 | 0 | 0 | — | — |
| BLAST Open Rotterdam Spring 2026 | 1470 | 0 | 0 | — | — |
| PGL Bucharest 2026 | 1499 | 0 | 0 | — | — |
| IEM Rio 2026 | 1538 | 0 | 0 | — | — |
| BLAST Rivals Spring 2026 | 1568 | 0 | 0 | — | — |
| PGL Astana 2026 | 1581 | 0 | 0 | — | — |
| IEM Atlanta 2026 | 1581 | 0 | 0 | — | — |
| CS Asia Championships 2026 | 1652 | 0 | 0 | — | — |
| IEM Cologne Major 2026 | 1682 | 0 | 0 | — | — |
| XSE Pro League 2026 | 1788 | 0 | 0 | — | — |
| BLAST Bounty Summer 2026 | 1828 | 0 | 0 | — | — |
| Esports World Cup 2026 | 1859 | 0 | 0 | — | — |
| BLAST Open Porto Fall 2026 | 1915 | 0 | 0 | — | — |
| FISSURE PLAYGROUND Season 3 2026 | 1944 | 0 | 0 | — | — |
| StarLadder StarSeries Fall 2026 | 1973 | 0 | 0 | — | — |
| ESL Pro League Season 24 2026 | 1987 | 0 | 0 | — | — |
| Thunderpick World Championship 2026 | 2028 | 0 | 0 | — | — |
| PGL Masters Bucharest 2026 | 2042 | 28 | 28 | +0.0230 | — |
| IEM Beijing 2026 | 2083 | 0 | 0 | — | — |
| BLAST Rivals Fall 2026 | 2113 | 0 | 0 | — | — |

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
| Perfect World Major Shanghai Major 2024 | 277 | 0 | 0 | — | — |
| BLAST Premier Bounty season 1 2025 | 350 | 0 | 0 | — | — |
| IEM Katowice 2025 | 357 | 0 | 0 | — | — |
| PGL Cluj-Napoca 2025 | 406 | 0 | 0 | — | — |
| ESL Pro League Season 21 2025 | 447 | 0 | 0 | — | — |
| BLAST Open Lisbon Spring 2025 | 487 | 0 | 0 | — | — |
| PGL Bucharest 2025 | 516 | 0 | 0 | — | — |
| IEM Melbourne 2025 | 557 | 0 | 0 | — | — |
| BLAST Rivals Spring 2025 | 586 | 0 | 0 | — | — |
| PGL Astana 2025 | 599 | 0 | 0 | — | — |
| IEM Dallas 2025 | 640 | 0 | 0 | — | — |
| BLAST.tv Major Austin 2025 | 669 | 0 | 0 | — | — |
| FISSURE PLAYGROUND Season 1 2025 | 775 | 0 | 0 | — | — |
| IEM Cologne 2025 | 802 | 0 | 0 | — | — |
| BLAST Bounty Fall 2025 | 851 | 0 | 0 | — | — |
| Esports World Cup 2025 | 882 | 0 | 0 | — | — |
| BLAST Open London Fall 2025 | 898 | 0 | 0 | — | — |
| FISSURE PLAYGROUND Season 2 2025 | 927 | 0 | 0 | — | — |
| ESL Pro League Season 22 2025 | 967 | 0 | 0 | — | — |
| CS Asia Championships 2025 | 1040 | 0 | 0 | — | — |
| Thunderpick World Championship 2025 | 1040 | 0 | 0 | — | — |
| PGL Masters Bucharest 2025 | 1084 | 0 | 0 | — | — |
| IEM Chengdu 2025 | 1125 | 0 | 0 | — | — |
| BLAST Rivals Fall 2025 | 1155 | 0 | 0 | — | — |
| StarLadder Major Budapest 2025 | 1168 | 0 | 0 | — | — |
| BLAST Bounty Winter 2026 | 1274 | 0 | 0 | — | — |
| IEM Krakow 2026 | 1305 | 0 | 0 | — | — |
| PGL Cluj-Napoca 2026 | 1355 | 0 | 0 | — | — |
| ESL Pro League Season 23 2026 | 1396 | 0 | 0 | — | — |
| BLAST Open Rotterdam Spring 2026 | 1470 | 0 | 0 | — | — |
| PGL Bucharest 2026 | 1499 | 0 | 0 | — | — |
| IEM Rio 2026 | 1538 | 0 | 0 | — | — |
| BLAST Rivals Spring 2026 | 1568 | 0 | 0 | — | — |
| PGL Astana 2026 | 1581 | 0 | 0 | — | — |
| IEM Atlanta 2026 | 1581 | 0 | 0 | — | — |
| CS Asia Championships 2026 | 1652 | 0 | 0 | — | — |
| IEM Cologne Major 2026 | 1682 | 0 | 0 | — | — |
| XSE Pro League 2026 | 1788 | 0 | 0 | — | — |
| BLAST Bounty Summer 2026 | 1828 | 0 | 0 | — | — |
| Esports World Cup 2026 | 1859 | 0 | 0 | — | — |
| BLAST Open Porto Fall 2026 | 1915 | 0 | 0 | — | — |
| FISSURE PLAYGROUND Season 3 2026 | 1944 | 0 | 0 | — | — |
| StarLadder StarSeries Fall 2026 | 1973 | 0 | 0 | — | — |
| ESL Pro League Season 24 2026 | 1987 | 0 | 0 | — | — |
| Thunderpick World Championship 2026 | 2028 | 0 | 0 | — | — |
| PGL Masters Bucharest 2026 | 2042 | 28 | 28 | +0.0160 | — |
| IEM Beijing 2026 | 2083 | 0 | 0 | — | — |
| BLAST Rivals Fall 2026 | 2113 | 0 | 0 | — | — |

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
| Perfect World Major Shanghai Major 2024 | 277 | 0 | 0 | — | — |
| BLAST Premier Bounty season 1 2025 | 350 | 0 | 0 | — | — |
| IEM Katowice 2025 | 357 | 0 | 0 | — | — |
| PGL Cluj-Napoca 2025 | 406 | 0 | 0 | — | — |
| ESL Pro League Season 21 2025 | 447 | 0 | 0 | — | — |
| BLAST Open Lisbon Spring 2025 | 487 | 0 | 0 | — | — |
| PGL Bucharest 2025 | 516 | 0 | 0 | — | — |
| IEM Melbourne 2025 | 557 | 0 | 0 | — | — |
| BLAST Rivals Spring 2025 | 586 | 0 | 0 | — | — |
| PGL Astana 2025 | 599 | 0 | 0 | — | — |
| IEM Dallas 2025 | 640 | 0 | 0 | — | — |
| BLAST.tv Major Austin 2025 | 669 | 0 | 0 | — | — |
| FISSURE PLAYGROUND Season 1 2025 | 775 | 0 | 0 | — | — |
| IEM Cologne 2025 | 802 | 0 | 0 | — | — |
| BLAST Bounty Fall 2025 | 851 | 0 | 0 | — | — |
| Esports World Cup 2025 | 882 | 0 | 0 | — | — |
| BLAST Open London Fall 2025 | 898 | 0 | 0 | — | — |
| FISSURE PLAYGROUND Season 2 2025 | 927 | 0 | 0 | — | — |
| ESL Pro League Season 22 2025 | 967 | 0 | 0 | — | — |
| CS Asia Championships 2025 | 1040 | 0 | 0 | — | — |
| Thunderpick World Championship 2025 | 1040 | 0 | 0 | — | — |
| PGL Masters Bucharest 2025 | 1084 | 0 | 0 | — | — |
| IEM Chengdu 2025 | 1125 | 0 | 0 | — | — |
| BLAST Rivals Fall 2025 | 1155 | 0 | 0 | — | — |
| StarLadder Major Budapest 2025 | 1168 | 0 | 0 | — | — |
| BLAST Bounty Winter 2026 | 1274 | 0 | 0 | — | — |
| IEM Krakow 2026 | 1305 | 0 | 0 | — | — |
| PGL Cluj-Napoca 2026 | 1355 | 0 | 0 | — | — |
| ESL Pro League Season 23 2026 | 1396 | 0 | 0 | — | — |
| BLAST Open Rotterdam Spring 2026 | 1470 | 0 | 0 | — | — |
| PGL Bucharest 2026 | 1499 | 0 | 0 | — | — |
| IEM Rio 2026 | 1538 | 0 | 0 | — | — |
| BLAST Rivals Spring 2026 | 1568 | 0 | 0 | — | — |
| PGL Astana 2026 | 1581 | 0 | 0 | — | — |
| IEM Atlanta 2026 | 1581 | 0 | 0 | — | — |
| CS Asia Championships 2026 | 1652 | 0 | 0 | — | — |
| IEM Cologne Major 2026 | 1682 | 0 | 0 | — | — |
| XSE Pro League 2026 | 1788 | 0 | 0 | — | — |
| BLAST Bounty Summer 2026 | 1828 | 0 | 0 | — | — |
| Esports World Cup 2026 | 1859 | 0 | 0 | — | — |
| BLAST Open Porto Fall 2026 | 1915 | 0 | 0 | — | — |
| FISSURE PLAYGROUND Season 3 2026 | 1944 | 0 | 0 | — | — |
| StarLadder StarSeries Fall 2026 | 1973 | 0 | 0 | — | — |
| ESL Pro League Season 24 2026 | 1987 | 0 | 0 | — | — |
| Thunderpick World Championship 2026 | 2028 | 0 | 0 | — | — |
| PGL Masters Bucharest 2026 | 2042 | 28 | 28 | +0.0124 | — |
| IEM Beijing 2026 | 2083 | 0 | 0 | — | — |
| BLAST Rivals Fall 2026 | 2113 | 0 | 0 | — | — |

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
| Perfect World Major Shanghai Major 2024 | 277 | 0 | 0 | — | — |
| BLAST Premier Bounty season 1 2025 | 350 | 0 | 0 | — | — |
| IEM Katowice 2025 | 357 | 0 | 0 | — | — |
| PGL Cluj-Napoca 2025 | 406 | 0 | 0 | — | — |
| ESL Pro League Season 21 2025 | 447 | 0 | 0 | — | — |
| BLAST Open Lisbon Spring 2025 | 487 | 0 | 0 | — | — |
| PGL Bucharest 2025 | 516 | 0 | 0 | — | — |
| IEM Melbourne 2025 | 557 | 0 | 0 | — | — |
| BLAST Rivals Spring 2025 | 586 | 0 | 0 | — | — |
| PGL Astana 2025 | 599 | 0 | 0 | — | — |
| IEM Dallas 2025 | 640 | 0 | 0 | — | — |
| BLAST.tv Major Austin 2025 | 669 | 0 | 0 | — | — |
| FISSURE PLAYGROUND Season 1 2025 | 775 | 0 | 0 | — | — |
| IEM Cologne 2025 | 802 | 0 | 0 | — | — |
| BLAST Bounty Fall 2025 | 851 | 0 | 0 | — | — |
| Esports World Cup 2025 | 882 | 0 | 0 | — | — |
| BLAST Open London Fall 2025 | 898 | 0 | 0 | — | — |
| FISSURE PLAYGROUND Season 2 2025 | 927 | 0 | 0 | — | — |
| ESL Pro League Season 22 2025 | 967 | 0 | 0 | — | — |
| CS Asia Championships 2025 | 1040 | 0 | 0 | — | — |
| Thunderpick World Championship 2025 | 1040 | 0 | 0 | — | — |
| PGL Masters Bucharest 2025 | 1084 | 0 | 0 | — | — |
| IEM Chengdu 2025 | 1125 | 0 | 0 | — | — |
| BLAST Rivals Fall 2025 | 1155 | 0 | 0 | — | — |
| StarLadder Major Budapest 2025 | 1168 | 0 | 0 | — | — |
| BLAST Bounty Winter 2026 | 1274 | 0 | 0 | — | — |
| IEM Krakow 2026 | 1305 | 49 | 49 | -0.0041 | — |
| PGL Cluj-Napoca 2026 | 1355 | 41 | 41 | +0.0278 | — |
| ESL Pro League Season 23 2026 | 1396 | 74 | 74 | -0.0057 | — |
| BLAST Open Rotterdam Spring 2026 | 1470 | 29 | 29 | -0.0114 | — |
| PGL Bucharest 2026 | 1499 | 39 | 39 | -0.0031 | — |
| IEM Rio 2026 | 1538 | 28 | 28 | -0.0095 | — |
| BLAST Rivals Spring 2026 | 1568 | 13 | 13 | +0.0237 | — |
| PGL Astana 2026 | 1581 | 41 | 41 | -0.0057 | — |
| IEM Atlanta 2026 | 1581 | 30 | 30 | -0.0148 | — |
| CS Asia Championships 2026 | 1652 | 0 | 0 | — | — |
| IEM Cologne Major 2026 | 1682 | 103 | 103 | -0.0056 | — |
| XSE Pro League 2026 | 1788 | 0 | 0 | — | — |
| BLAST Bounty Summer 2026 | 1828 | 30 | 30 | +0.0413 | — |
| Esports World Cup 2026 | 1859 | 56 | 56 | -0.0024 | — |
| BLAST Open Porto Fall 2026 | 1915 | 29 | 29 | +0.0157 | — |
| FISSURE PLAYGROUND Season 3 2026 | 1944 | 29 | 29 | -0.0081 | — |
| StarLadder StarSeries Fall 2026 | 1973 | 14 | 14 | +0.0153 | — |
| ESL Pro League Season 24 2026 | 1987 | 41 | 41 | -0.0064 | — |
| Thunderpick World Championship 2026 | 2028 | 14 | 14 | +0.0054 | — |
| PGL Masters Bucharest 2026 | 2042 | 41 | 41 | +0.0046 | — |
| IEM Beijing 2026 | 2083 | 30 | 30 | +0.0077 | — |
| BLAST Rivals Fall 2026 | 2113 | 13 | 13 | -0.0041 | — |

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
| Perfect World Major Shanghai Major 2024 | 277 | 0 | 0 | — | — |
| BLAST Premier Bounty season 1 2025 | 350 | 0 | 0 | — | — |
| IEM Katowice 2025 | 357 | 0 | 0 | — | — |
| PGL Cluj-Napoca 2025 | 406 | 0 | 0 | — | — |
| ESL Pro League Season 21 2025 | 447 | 0 | 0 | — | — |
| BLAST Open Lisbon Spring 2025 | 487 | 0 | 0 | — | — |
| PGL Bucharest 2025 | 516 | 0 | 0 | — | — |
| IEM Melbourne 2025 | 557 | 0 | 0 | — | — |
| BLAST Rivals Spring 2025 | 586 | 0 | 0 | — | — |
| PGL Astana 2025 | 599 | 0 | 0 | — | — |
| IEM Dallas 2025 | 640 | 0 | 0 | — | — |
| BLAST.tv Major Austin 2025 | 669 | 0 | 0 | — | — |
| FISSURE PLAYGROUND Season 1 2025 | 775 | 0 | 0 | — | — |
| IEM Cologne 2025 | 802 | 0 | 0 | — | — |
| BLAST Bounty Fall 2025 | 851 | 0 | 0 | — | — |
| Esports World Cup 2025 | 882 | 0 | 0 | — | — |
| BLAST Open London Fall 2025 | 898 | 0 | 0 | — | — |
| FISSURE PLAYGROUND Season 2 2025 | 927 | 0 | 0 | — | — |
| ESL Pro League Season 22 2025 | 967 | 0 | 0 | — | — |
| CS Asia Championships 2025 | 1040 | 0 | 0 | — | — |
| Thunderpick World Championship 2025 | 1040 | 0 | 0 | — | — |
| PGL Masters Bucharest 2025 | 1084 | 0 | 0 | — | — |
| IEM Chengdu 2025 | 1125 | 0 | 0 | — | — |
| BLAST Rivals Fall 2025 | 1155 | 0 | 0 | — | — |
| StarLadder Major Budapest 2025 | 1168 | 0 | 0 | — | — |
| BLAST Bounty Winter 2026 | 1274 | 0 | 0 | — | — |
| IEM Krakow 2026 | 1305 | 50 | 50 | -0.0010 | — |
| PGL Cluj-Napoca 2026 | 1355 | 41 | 41 | +0.0075 | — |
| ESL Pro League Season 23 2026 | 1396 | 74 | 74 | +0.0004 | — |
| BLAST Open Rotterdam Spring 2026 | 1470 | 29 | 29 | -0.0087 | — |
| PGL Bucharest 2026 | 1499 | 39 | 39 | +0.0023 | — |
| IEM Rio 2026 | 1538 | 28 | 28 | +0.0030 | — |
| BLAST Rivals Spring 2026 | 1568 | 13 | 13 | -0.0001 | — |
| PGL Astana 2026 | 1581 | 41 | 41 | +0.0002 | — |
| IEM Atlanta 2026 | 1581 | 30 | 30 | -0.0015 | — |
| CS Asia Championships 2026 | 1652 | 0 | 0 | — | — |
| IEM Cologne Major 2026 | 1682 | 104 | 104 | +0.0002 | — |
| XSE Pro League 2026 | 1788 | 0 | 0 | — | — |
| BLAST Bounty Summer 2026 | 1828 | 30 | 30 | +0.0107 | — |
| Esports World Cup 2026 | 1859 | 56 | 56 | +0.0001 | — |
| BLAST Open Porto Fall 2026 | 1915 | 29 | 29 | -0.0004 | — |
| FISSURE PLAYGROUND Season 3 2026 | 1944 | 29 | 29 | +0.0049 | — |
| StarLadder StarSeries Fall 2026 | 1973 | 14 | 14 | -0.0005 | — |
| ESL Pro League Season 24 2026 | 1987 | 41 | 41 | -0.0002 | — |
| Thunderpick World Championship 2026 | 2028 | 14 | 14 | -0.0000 | — |
| PGL Masters Bucharest 2026 | 2042 | 41 | 41 | -0.0002 | — |
| IEM Beijing 2026 | 2083 | 30 | 30 | -0.0001 | — |
| BLAST Rivals Fall 2026 | 2113 | 13 | 13 | -0.0003 | — |

## `own_rating`: promote

Glicko-2 team rating updated after every Match finished before the cutoff (teams joined by team_key), with format-calibrated map scale blended with the market as the anchor does.

Verdict: the interval of the log-loss improvement excludes zero and the Matches with a market line are not worse.

| On the 2028 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.7904 | 0.2401 | 65.6% |
| own_rating | 0.6324 | 0.2207 | 64.7% |

### Direct comparison with Valve rating (2027 paired Matches, 56 Events)

Reference: Valve Regional Standings (unblended). Δ log-loss -0.2279 (90% interval [-0.2955, -0.1680], detectable 0.0965), Δ Brier -0.0290, Δ accuracy -0.0109.

| On the 2027 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| Valve rating | 0.8689 | 0.2535 | 65.2% |
| own_rating | 0.6410 | 0.2245 | 64.1% |

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
| Perfect World Major Shanghai Major 2024 | 277 | 73 | 73 | +0.0048 | — |
| BLAST Premier Bounty season 1 2025 | 350 | 7 | 7 | -0.7639 | — |
| IEM Katowice 2025 | 357 | 49 | 47 | -0.2475 | — |
| PGL Cluj-Napoca 2025 | 406 | 41 | 38 | +0.0151 | — |
| ESL Pro League Season 21 2025 | 447 | 39 | 39 | +0.0035 | — |
| BLAST Open Lisbon Spring 2025 | 487 | 28 | 28 | -0.0644 | — |
| PGL Bucharest 2025 | 516 | 40 | 40 | -0.9264 | — |
| IEM Melbourne 2025 | 557 | 29 | 29 | -0.1200 | — |
| BLAST Rivals Spring 2025 | 586 | 13 | 13 | -0.2047 | — |
| PGL Astana 2025 | 599 | 39 | 39 | -1.1208 | — |
| IEM Dallas 2025 | 640 | 28 | 28 | +0.0055 | — |
| BLAST.tv Major Austin 2025 | 669 | 105 | 105 | -0.2354 | — |
| FISSURE PLAYGROUND Season 1 2025 | 775 | 27 | 27 | -0.7116 | — |
| IEM Cologne 2025 | 802 | 49 | 49 | +0.0575 | — |
| BLAST Bounty Fall 2025 | 851 | 28 | 28 | +0.0610 | — |
| Esports World Cup 2025 | 882 | 16 | 16 | -0.4244 | — |
| BLAST Open London Fall 2025 | 898 | 29 | 29 | -0.4916 | — |
| FISSURE PLAYGROUND Season 2 2025 | 927 | 40 | 40 | -0.2766 | — |
| ESL Pro League Season 22 2025 | 967 | 71 | 71 | -0.7425 | — |
| CS Asia Championships 2025 | 1040 | 30 | 30 | -0.2320 | — |
| Thunderpick World Championship 2025 | 1040 | 13 | 13 | -0.1899 | — |
| PGL Masters Bucharest 2025 | 1084 | 41 | 41 | -0.2665 | — |
| IEM Chengdu 2025 | 1125 | 30 | 30 | -0.0391 | — |
| BLAST Rivals Fall 2025 | 1155 | 13 | 13 | -0.0592 | — |
| StarLadder Major Budapest 2025 | 1168 | 106 | 106 | -0.1409 | — |
| BLAST Bounty Winter 2026 | 1274 | 27 | 27 | -0.0162 | — |
| IEM Krakow 2026 | 1305 | 50 | 50 | -0.0151 | — |
| PGL Cluj-Napoca 2026 | 1355 | 41 | 41 | +0.0249 | — |
| ESL Pro League Season 23 2026 | 1396 | 72 | 72 | -0.0311 | — |
| BLAST Open Rotterdam Spring 2026 | 1470 | 29 | 29 | -0.0197 | — |
| PGL Bucharest 2026 | 1499 | 37 | 37 | +0.0247 | — |
| IEM Rio 2026 | 1538 | 30 | 30 | +0.0328 | — |
| BLAST Rivals Spring 2026 | 1568 | 13 | 13 | -0.0192 | — |
| PGL Astana 2026 | 1581 | 38 | 38 | -0.0054 | — |
| IEM Atlanta 2026 | 1581 | 30 | 30 | -0.0158 | — |
| CS Asia Championships 2026 | 1652 | 30 | 30 | -0.1842 | — |
| IEM Cologne Major 2026 | 1682 | 104 | 104 | +0.0163 | — |
| XSE Pro League 2026 | 1788 | 38 | 36 | -0.0584 | — |
| BLAST Bounty Summer 2026 | 1828 | 28 | 28 | -0.0367 | — |
| Esports World Cup 2026 | 1859 | 55 | 55 | +0.0265 | — |
| BLAST Open Porto Fall 2026 | 1915 | 29 | 29 | +0.0147 | — |
| FISSURE PLAYGROUND Season 3 2026 | 1944 | 28 | 28 | -0.0505 | — |
| StarLadder StarSeries Fall 2026 | 1973 | 14 | 14 | -0.0371 | — |
| ESL Pro League Season 24 2026 | 1987 | 39 | 39 | -0.0469 | — |
| Thunderpick World Championship 2026 | 2028 | 14 | 14 | +0.0035 | — |
| PGL Masters Bucharest 2026 | 2042 | 41 | 41 | -0.0885 | — |
| IEM Beijing 2026 | 2083 | 30 | 30 | -0.0619 | — |
| BLAST Rivals Fall 2026 | 2113 | 13 | 13 | +0.0176 | — |

## `recent_form`: no evidence

Recent results over the last 45 days (14-day half-life) weighted by opponent strength from the point-in-time Glicko-2 rating, added as a fitted logit offset to the anchor.

Verdict: the interval of the log-loss difference includes zero.

| On the 1859 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.7738 | 0.2380 | 65.8% |
| recent_form | 0.7731 | 0.2395 | 65.5% |

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
| Perfect World Major Shanghai Major 2024 | 277 | 71 | 71 | +0.0009 | — |
| BLAST Premier Bounty season 1 2025 | 350 | 7 | 7 | +0.0002 | — |
| IEM Katowice 2025 | 357 | 46 | 46 | +0.0333 | — |
| PGL Cluj-Napoca 2025 | 406 | 38 | 38 | -0.0168 | — |
| ESL Pro League Season 21 2025 | 447 | 39 | 39 | -0.0096 | — |
| BLAST Open Lisbon Spring 2025 | 487 | 26 | 26 | +0.0165 | — |
| PGL Bucharest 2025 | 516 | 36 | 36 | -0.0194 | — |
| IEM Melbourne 2025 | 557 | 28 | 28 | +0.0006 | — |
| BLAST Rivals Spring 2025 | 586 | 12 | 12 | +0.1006 | — |
| PGL Astana 2025 | 599 | 37 | 37 | -0.0135 | — |
| IEM Dallas 2025 | 640 | 27 | 27 | +0.0131 | — |
| BLAST.tv Major Austin 2025 | 669 | 99 | 99 | -0.0161 | — |
| FISSURE PLAYGROUND Season 1 2025 | 775 | 22 | 22 | -0.0245 | — |
| IEM Cologne 2025 | 802 | 46 | 46 | +0.0028 | — |
| BLAST Bounty Fall 2025 | 851 | 22 | 22 | -0.0416 | — |
| Esports World Cup 2025 | 882 | 16 | 16 | -0.1132 | — |
| BLAST Open London Fall 2025 | 898 | 27 | 27 | -0.0277 | — |
| FISSURE PLAYGROUND Season 2 2025 | 927 | 39 | 39 | -0.0260 | — |
| ESL Pro League Season 22 2025 | 967 | 65 | 65 | +0.0106 | — |
| CS Asia Championships 2025 | 1040 | 27 | 27 | -0.0316 | — |
| Thunderpick World Championship 2025 | 1040 | 10 | 10 | +0.0606 | — |
| PGL Masters Bucharest 2025 | 1084 | 37 | 37 | +0.0072 | — |
| IEM Chengdu 2025 | 1125 | 30 | 30 | -0.0154 | — |
| BLAST Rivals Fall 2025 | 1155 | 12 | 12 | +0.0636 | — |
| StarLadder Major Budapest 2025 | 1168 | 101 | 101 | -0.0089 | — |
| BLAST Bounty Winter 2026 | 1274 | 17 | 17 | -0.0007 | — |
| IEM Krakow 2026 | 1305 | 45 | 45 | +0.0265 | — |
| PGL Cluj-Napoca 2026 | 1355 | 41 | 41 | +0.0362 | — |
| ESL Pro League Season 23 2026 | 1396 | 71 | 71 | -0.0192 | — |
| BLAST Open Rotterdam Spring 2026 | 1470 | 27 | 27 | +0.0307 | — |
| PGL Bucharest 2026 | 1499 | 32 | 32 | +0.0126 | — |
| IEM Rio 2026 | 1538 | 27 | 27 | -0.0009 | — |
| BLAST Rivals Spring 2026 | 1568 | 12 | 12 | -0.0306 | — |
| PGL Astana 2026 | 1581 | 34 | 34 | +0.0061 | — |
| IEM Atlanta 2026 | 1581 | 26 | 26 | -0.0128 | — |
| CS Asia Championships 2026 | 1652 | 27 | 27 | +0.0238 | — |
| IEM Cologne Major 2026 | 1682 | 101 | 101 | +0.0138 | — |
| XSE Pro League 2026 | 1788 | 34 | 34 | +0.0132 | — |
| BLAST Bounty Summer 2026 | 1828 | 19 | 19 | +0.0087 | — |
| Esports World Cup 2026 | 1859 | 52 | 52 | -0.0065 | — |
| BLAST Open Porto Fall 2026 | 1915 | 28 | 28 | +0.0197 | — |
| FISSURE PLAYGROUND Season 3 2026 | 1944 | 27 | 27 | +0.0008 | — |
| StarLadder StarSeries Fall 2026 | 1973 | 13 | 13 | -0.0026 | — |
| ESL Pro League Season 24 2026 | 1987 | 39 | 39 | -0.0063 | — |
| Thunderpick World Championship 2026 | 2028 | 13 | 13 | -0.0093 | — |
| PGL Masters Bucharest 2026 | 2042 | 36 | 36 | -0.0051 | — |
| IEM Beijing 2026 | 2083 | 28 | 28 | +0.0286 | — |
| BLAST Rivals Fall 2026 | 2113 | 13 | 13 | -0.0013 | — |

## `head_to_head`: no evidence

Recency-weighted head-to-head series record between the two teams (180-day half-life, shrunk toward even), added as a fitted logit offset to the anchor.

Verdict: the interval of the log-loss difference includes zero.

| On the 1244 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.7335 | 0.2338 | 66.3% |
| head_to_head | 0.7308 | 0.2347 | 65.7% |

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
| Perfect World Major Shanghai Major 2024 | 277 | 33 | 33 | +0.0025 | — |
| BLAST Premier Bounty season 1 2025 | 350 | 6 | 6 | +0.0524 | — |
| IEM Katowice 2025 | 357 | 25 | 25 | -0.0001 | — |
| PGL Cluj-Napoca 2025 | 406 | 23 | 23 | -0.0144 | — |
| ESL Pro League Season 21 2025 | 447 | 31 | 31 | +0.0077 | — |
| BLAST Open Lisbon Spring 2025 | 487 | 22 | 22 | +0.0096 | — |
| PGL Bucharest 2025 | 516 | 20 | 20 | -0.0132 | — |
| IEM Melbourne 2025 | 557 | 19 | 19 | -0.0065 | — |
| BLAST Rivals Spring 2025 | 586 | 12 | 12 | +0.0060 | — |
| PGL Astana 2025 | 599 | 25 | 25 | -0.0168 | — |
| IEM Dallas 2025 | 640 | 18 | 18 | -0.0034 | — |
| BLAST.tv Major Austin 2025 | 669 | 40 | 40 | -0.0514 | — |
| FISSURE PLAYGROUND Season 1 2025 | 775 | 15 | 15 | +0.0368 | — |
| IEM Cologne 2025 | 802 | 45 | 45 | -0.0131 | — |
| BLAST Bounty Fall 2025 | 851 | 18 | 18 | +0.0146 | — |
| Esports World Cup 2025 | 882 | 14 | 14 | -0.1024 | — |
| BLAST Open London Fall 2025 | 898 | 23 | 23 | -0.0341 | — |
| FISSURE PLAYGROUND Season 2 2025 | 927 | 34 | 34 | -0.0522 | — |
| ESL Pro League Season 22 2025 | 967 | 36 | 36 | -0.0274 | — |
| CS Asia Championships 2025 | 1040 | 16 | 16 | +0.0308 | — |
| Thunderpick World Championship 2025 | 1040 | 10 | 10 | +0.0994 | — |
| PGL Masters Bucharest 2025 | 1084 | 23 | 23 | -0.1073 | — |
| IEM Chengdu 2025 | 1125 | 27 | 27 | +0.0481 | — |
| BLAST Rivals Fall 2025 | 1155 | 10 | 10 | +0.0545 | — |
| StarLadder Major Budapest 2025 | 1168 | 58 | 58 | -0.0011 | — |
| BLAST Bounty Winter 2026 | 1274 | 14 | 14 | -0.0043 | — |
| IEM Krakow 2026 | 1305 | 37 | 37 | +0.0340 | — |
| PGL Cluj-Napoca 2026 | 1355 | 34 | 34 | +0.0387 | — |
| ESL Pro League Season 23 2026 | 1396 | 52 | 52 | -0.0152 | — |
| BLAST Open Rotterdam Spring 2026 | 1470 | 25 | 25 | +0.0387 | — |
| PGL Bucharest 2026 | 1499 | 18 | 18 | +0.0277 | — |
| IEM Rio 2026 | 1538 | 24 | 24 | +0.0496 | — |
| BLAST Rivals Spring 2026 | 1568 | 12 | 12 | -0.0001 | — |
| PGL Astana 2026 | 1581 | 22 | 22 | +0.0350 | — |
| IEM Atlanta 2026 | 1581 | 25 | 25 | -0.0129 | — |
| CS Asia Championships 2026 | 1652 | 20 | 20 | -0.0393 | — |
| IEM Cologne Major 2026 | 1682 | 70 | 70 | +0.0013 | — |
| XSE Pro League 2026 | 1788 | 20 | 20 | +0.0030 | — |
| BLAST Bounty Summer 2026 | 1828 | 19 | 19 | -0.0123 | — |
| Esports World Cup 2026 | 1859 | 42 | 42 | +0.0377 | — |
| BLAST Open Porto Fall 2026 | 1915 | 24 | 24 | +0.0051 | — |
| FISSURE PLAYGROUND Season 3 2026 | 1944 | 16 | 16 | -0.0174 | — |
| StarLadder StarSeries Fall 2026 | 1973 | 11 | 11 | -0.0175 | — |
| ESL Pro League Season 24 2026 | 1987 | 30 | 30 | -0.0152 | — |
| Thunderpick World Championship 2026 | 2028 | 12 | 12 | -0.0145 | — |
| PGL Masters Bucharest 2026 | 2042 | 34 | 34 | -0.0291 | — |
| IEM Beijing 2026 | 2083 | 30 | 30 | -0.0149 | — |
| BLAST Rivals Fall 2026 | 2113 | 13 | 13 | -0.0479 | — |

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
| Perfect World Major Shanghai Major 2024 | 277 | 0 | 0 | — | — |
| BLAST Premier Bounty season 1 2025 | 350 | 0 | 0 | — | — |
| IEM Katowice 2025 | 357 | 0 | 0 | — | — |
| PGL Cluj-Napoca 2025 | 406 | 0 | 0 | — | — |
| ESL Pro League Season 21 2025 | 447 | 0 | 0 | — | — |
| BLAST Open Lisbon Spring 2025 | 487 | 0 | 0 | — | — |
| PGL Bucharest 2025 | 516 | 0 | 0 | — | — |
| IEM Melbourne 2025 | 557 | 0 | 0 | — | — |
| BLAST Rivals Spring 2025 | 586 | 0 | 0 | — | — |
| PGL Astana 2025 | 599 | 0 | 0 | — | — |
| IEM Dallas 2025 | 640 | 0 | 0 | — | — |
| BLAST.tv Major Austin 2025 | 669 | 0 | 0 | — | — |
| FISSURE PLAYGROUND Season 1 2025 | 775 | 0 | 0 | — | — |
| IEM Cologne 2025 | 802 | 0 | 0 | — | — |
| BLAST Bounty Fall 2025 | 851 | 0 | 0 | — | — |
| Esports World Cup 2025 | 882 | 0 | 0 | — | — |
| BLAST Open London Fall 2025 | 898 | 0 | 0 | — | — |
| FISSURE PLAYGROUND Season 2 2025 | 927 | 0 | 0 | — | — |
| ESL Pro League Season 22 2025 | 967 | 0 | 0 | — | — |
| CS Asia Championships 2025 | 1040 | 0 | 0 | — | — |
| Thunderpick World Championship 2025 | 1040 | 0 | 0 | — | — |
| PGL Masters Bucharest 2025 | 1084 | 0 | 0 | — | — |
| IEM Chengdu 2025 | 1125 | 0 | 0 | — | — |
| BLAST Rivals Fall 2025 | 1155 | 0 | 0 | — | — |
| StarLadder Major Budapest 2025 | 1168 | 0 | 0 | — | — |
| BLAST Bounty Winter 2026 | 1274 | 0 | 0 | — | — |
| IEM Krakow 2026 | 1305 | 0 | 0 | — | — |
| PGL Cluj-Napoca 2026 | 1355 | 0 | 0 | — | — |
| ESL Pro League Season 23 2026 | 1396 | 0 | 0 | — | — |
| BLAST Open Rotterdam Spring 2026 | 1470 | 0 | 0 | — | — |
| PGL Bucharest 2026 | 1499 | 0 | 0 | — | — |
| IEM Rio 2026 | 1538 | 0 | 0 | — | — |
| BLAST Rivals Spring 2026 | 1568 | 0 | 0 | — | — |
| PGL Astana 2026 | 1581 | 0 | 0 | — | — |
| IEM Atlanta 2026 | 1581 | 0 | 0 | — | — |
| CS Asia Championships 2026 | 1652 | 0 | 0 | — | — |
| IEM Cologne Major 2026 | 1682 | 0 | 0 | — | — |
| XSE Pro League 2026 | 1788 | 0 | 0 | — | — |
| BLAST Bounty Summer 2026 | 1828 | 0 | 0 | — | — |
| Esports World Cup 2026 | 1859 | 0 | 0 | — | — |
| BLAST Open Porto Fall 2026 | 1915 | 0 | 0 | — | — |
| FISSURE PLAYGROUND Season 3 2026 | 1944 | 0 | 0 | — | — |
| StarLadder StarSeries Fall 2026 | 1973 | 0 | 0 | — | — |
| ESL Pro League Season 24 2026 | 1987 | 0 | 0 | — | — |
| Thunderpick World Championship 2026 | 2028 | 0 | 0 | — | — |
| PGL Masters Bucharest 2026 | 2042 | 0 | 0 | — | — |
| IEM Beijing 2026 | 2083 | 0 | 0 | — | — |
| BLAST Rivals Fall 2026 | 2113 | 0 | 0 | — | — |

## `lineup_strength`: promote

Mean HLTV player rating of the 5 playing starters from the pre-cutoff Liquipedia roster (90-day half-life, shrunk toward prior 1.00 with weight 6 maps; debutants/unknown stand-ins take the 1.00 prior when at least 3 of 5 starters on each team have HLTV history, else uncovered), added as a fitted logit offset to the anchor.

Verdict: the interval of the log-loss improvement excludes zero and the Matches with a market line are not worse.

| On the 1812 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.7865 | 0.2396 | 65.9% |
| lineup_strength | 0.7838 | 0.2399 | 65.6% |

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
| Perfect World Major Shanghai Major 2024 | 277 | 73 | 73 | -0.0009 | `{"beta":-1.3053571433574023,"training_matches":220}` |
| BLAST Premier Bounty season 1 2025 | 350 | 7 | 7 | -0.0292 | `{"beta":-1.3393564082836327,"training_matches":293}` |
| IEM Katowice 2025 | 357 | 47 | 47 | +0.0012 | `{"beta":-1.4796810294981833,"training_matches":300}` |
| PGL Cluj-Napoca 2025 | 406 | 38 | 38 | -0.0028 | `{"beta":-1.436530610416193,"training_matches":347}` |
| ESL Pro League Season 21 2025 | 447 | 39 | 39 | +0.0040 | `{"beta":-1.497199328456189,"training_matches":385}` |
| BLAST Open Lisbon Spring 2025 | 487 | 28 | 28 | +0.0046 | `{"beta":-1.3943502832672707,"training_matches":424}` |
| PGL Bucharest 2025 | 516 | 35 | 35 | -0.0117 | `{"beta":-1.3052253963918408,"training_matches":452}` |
| IEM Melbourne 2025 | 557 | 28 | 28 | +0.0050 | `{"beta":-1.5790647020844983,"training_matches":487}` |
| BLAST Rivals Spring 2025 | 586 | 13 | 13 | +0.0013 | `{"beta":-1.4989958246003021,"training_matches":515}` |
| PGL Astana 2025 | 599 | 39 | 39 | -0.0113 | `{"beta":-1.4870881274672478,"training_matches":528}` |
| IEM Dallas 2025 | 640 | 28 | 28 | +0.0013 | `{"beta":-1.7406337341565514,"training_matches":567}` |
| BLAST.tv Major Austin 2025 | 669 | 104 | 104 | -0.0032 | `{"beta":-1.7184530094695862,"training_matches":595}` |
| FISSURE PLAYGROUND Season 1 2025 | 775 | 27 | 27 | -0.0198 | `{"beta":-1.8550582211212308,"training_matches":699}` |
| IEM Cologne 2025 | 802 | 49 | 49 | +0.0048 | `{"beta":-2.092030029177139,"training_matches":726}` |
| BLAST Bounty Fall 2025 | 851 | 7 | 7 | -0.0125 | `{"beta":-1.9872956119629621,"training_matches":775}` |
| Esports World Cup 2025 | 882 | 16 | 16 | -0.0316 | `{"beta":-2.0209779345470817,"training_matches":782}` |
| BLAST Open London Fall 2025 | 898 | 7 | 7 | +0.0104 | `{"beta":-2.2226207023447913,"training_matches":798}` |
| FISSURE PLAYGROUND Season 2 2025 | 927 | 40 | 40 | +0.0060 | `{"beta":-2.1947162099537527,"training_matches":805}` |
| ESL Pro League Season 22 2025 | 967 | 63 | 63 | +0.0007 | `{"beta":-2.096665272256871,"training_matches":845}` |
| CS Asia Championships 2025 | 1040 | 29 | 29 | -0.0022 | `{"beta":-2.0580360589366427,"training_matches":908}` |
| Thunderpick World Championship 2025 | 1040 | 13 | 13 | -0.0006 | `{"beta":-2.0580360589366427,"training_matches":908}` |
| PGL Masters Bucharest 2025 | 1084 | 41 | 41 | -0.0072 | `{"beta":-2.080486968397631,"training_matches":950}` |
| IEM Chengdu 2025 | 1125 | 30 | 30 | -0.0145 | `{"beta":-2.1858842811702788,"training_matches":991}` |
| BLAST Rivals Fall 2025 | 1155 | 13 | 13 | +0.0045 | `{"beta":-2.3266484648453893,"training_matches":1021}` |
| StarLadder Major Budapest 2025 | 1168 | 106 | 106 | -0.0035 | `{"beta":-2.3049887586392144,"training_matches":1034}` |
| BLAST Bounty Winter 2026 | 1274 | 7 | 7 | -0.0584 | `{"beta":-2.3950959925761657,"training_matches":1140}` |
| IEM Krakow 2026 | 1305 | 50 | 50 | +0.0012 | `{"beta":-2.5179546874190732,"training_matches":1147}` |
| PGL Cluj-Napoca 2026 | 1355 | 41 | 41 | +0.0069 | `{"beta":-2.4764587524013635,"training_matches":1197}` |
| ESL Pro League Season 23 2026 | 1396 | 8 | 8 | -0.0162 | `{"beta":-2.376597299816927,"training_matches":1238}` |
| BLAST Open Rotterdam Spring 2026 | 1470 | 24 | 24 | -0.0143 | `{"beta":-2.413719771426453,"training_matches":1246}` |
| PGL Bucharest 2026 | 1499 | 37 | 37 | +0.0001 | `{"beta":-2.5025138957187796,"training_matches":1270}` |
| IEM Rio 2026 | 1538 | 30 | 30 | +0.0119 | `{"beta":-2.492667642429825,"training_matches":1307}` |
| BLAST Rivals Spring 2026 | 1568 | 13 | 13 | -0.0085 | `{"beta":-2.3826760495249273,"training_matches":1337}` |
| PGL Astana 2026 | 1581 | 38 | 38 | +0.0033 | `{"beta":-2.4106559226545032,"training_matches":1350}` |
| IEM Atlanta 2026 | 1581 | 30 | 30 | -0.0200 | `{"beta":-2.4106559226545032,"training_matches":1350}` |
| CS Asia Championships 2026 | 1652 | 30 | 30 | -0.0069 | `{"beta":-2.5143394715647256,"training_matches":1418}` |
| IEM Cologne Major 2026 | 1682 | 105 | 105 | +0.0055 | `{"beta":-2.5653089628217556,"training_matches":1448}` |
| XSE Pro League 2026 | 1788 | 36 | 36 | +0.0051 | `{"beta":-2.372682288522176,"training_matches":1553}` |
| BLAST Bounty Summer 2026 | 1828 | 6 | 6 | -0.0013 | `{"beta":-2.313645644651154,"training_matches":1589}` |
| Esports World Cup 2026 | 1859 | 55 | 55 | +0.0052 | `{"beta":-2.3125329023474004,"training_matches":1595}` |
| BLAST Open Porto Fall 2026 | 1915 | 26 | 26 | +0.0141 | `{"beta":-2.209136732872399,"training_matches":1650}` |
| FISSURE PLAYGROUND Season 3 2026 | 1944 | 28 | 28 | -0.0137 | `{"beta":-2.0920739579740806,"training_matches":1676}` |
| StarLadder StarSeries Fall 2026 | 1973 | 14 | 14 | -0.0036 | `{"beta":-2.2041151226970843,"training_matches":1704}` |
| ESL Pro League Season 24 2026 | 1987 | 35 | 35 | -0.0008 | `{"beta":-2.2148280456416467,"training_matches":1718}` |
| Thunderpick World Championship 2026 | 2028 | 14 | 14 | -0.0005 | `{"beta":-2.2109217832378807,"training_matches":1753}` |
| PGL Masters Bucharest 2026 | 2042 | 41 | 41 | -0.0250 | `{"beta":-2.2098530547578106,"training_matches":1767}` |
| IEM Beijing 2026 | 2083 | 30 | 30 | -0.0088 | `{"beta":-2.469072784566582,"training_matches":1808}` |
| BLAST Rivals Fall 2026 | 2113 | 13 | 13 | -0.0100 | `{"beta":-2.5250318133183445,"training_matches":1838}` |

## `player_form`: no evidence

Mean recent-minus-longer-run HLTV rating form of the 5 playing starters from the pre-cutoff Liquipedia roster (30d/14d half-life vs 180d/90d half-life, shrunk toward neutral 0.0 with weight 4 maps; debutants/players without 2+ baseline & recent Matches take 0.0 when at least 3 of 5 starters on each team have form, else uncovered), added as a fitted logit offset to the anchor.

Verdict: the interval of the log-loss difference includes zero.

| On the 1480 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.7677 | 0.2358 | 66.8% |
| player_form | 0.7678 | 0.2358 | 66.7% |

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
| Perfect World Major Shanghai Major 2024 | 277 | 63 | 63 | +0.0004 | `{"beta":0.1604031760636983,"training_matches":187}` |
| BLAST Premier Bounty season 1 2025 | 350 | 3 | 3 | +0.0005 | `{"beta":0.022624794854665564,"training_matches":250}` |
| IEM Katowice 2025 | 357 | 35 | 35 | -0.0000 | `{"beta":-0.0458403836953426,"training_matches":253}` |
| PGL Cluj-Napoca 2025 | 406 | 38 | 38 | -0.0001 | `{"beta":-0.06313355684765065,"training_matches":288}` |
| ESL Pro League Season 21 2025 | 447 | 34 | 34 | +0.0000 | `{"beta":-0.11432072479206407,"training_matches":326}` |
| BLAST Open Lisbon Spring 2025 | 487 | 25 | 25 | -0.0001 | `{"beta":-0.10611632353868009,"training_matches":360}` |
| PGL Bucharest 2025 | 516 | 31 | 31 | +0.0003 | `{"beta":-0.13171871608272623,"training_matches":385}` |
| IEM Melbourne 2025 | 557 | 26 | 26 | +0.0001 | `{"beta":-0.07064855788576574,"training_matches":416}` |
| BLAST Rivals Spring 2025 | 586 | 11 | 11 | -0.0002 | `{"beta":-0.04213807244320307,"training_matches":442}` |
| PGL Astana 2025 | 599 | 34 | 34 | +0.0000 | `{"beta":-0.086080239545547,"training_matches":453}` |
| IEM Dallas 2025 | 640 | 22 | 22 | -0.0000 | `{"beta":-0.07456707322269543,"training_matches":487}` |
| BLAST.tv Major Austin 2025 | 669 | 97 | 97 | +0.0000 | `{"beta":-0.08797496969141508,"training_matches":509}` |
| FISSURE PLAYGROUND Season 1 2025 | 775 | 19 | 19 | -0.0008 | `{"beta":-0.07665064921256012,"training_matches":606}` |
| IEM Cologne 2025 | 802 | 36 | 36 | +0.0003 | `{"beta":-0.26792490500044175,"training_matches":625}` |
| BLAST Bounty Fall 2025 | 851 | 7 | 7 | +0.0004 | `{"beta":-0.22973878368688877,"training_matches":661}` |
| Esports World Cup 2025 | 882 | 16 | 16 | +0.0004 | `{"beta":-0.21962094966380752,"training_matches":668}` |
| BLAST Open London Fall 2025 | 898 | 4 | 4 | +0.0033 | `{"beta":-0.1910702921728188,"training_matches":684}` |
| FISSURE PLAYGROUND Season 2 2025 | 927 | 37 | 37 | -0.0004 | `{"beta":-0.12849064412968242,"training_matches":688}` |
| ESL Pro League Season 22 2025 | 967 | 52 | 52 | +0.0001 | `{"beta":-0.22547783702651936,"training_matches":725}` |
| CS Asia Championships 2025 | 1040 | 23 | 23 | -0.0004 | `{"beta":-0.19248576691518743,"training_matches":777}` |
| Thunderpick World Championship 2025 | 1040 | 8 | 8 | +0.0012 | `{"beta":-0.19248576691518743,"training_matches":777}` |
| PGL Masters Bucharest 2025 | 1084 | 23 | 23 | +0.0004 | `{"beta":-0.19179931765603303,"training_matches":808}` |
| IEM Chengdu 2025 | 1125 | 30 | 30 | -0.0002 | `{"beta":-0.14856126994296032,"training_matches":831}` |
| BLAST Rivals Fall 2025 | 1155 | 12 | 12 | +0.0004 | `{"beta":-0.18712637809790367,"training_matches":861}` |
| StarLadder Major Budapest 2025 | 1168 | 94 | 94 | +0.0004 | `{"beta":-0.16626852575609669,"training_matches":873}` |
| BLAST Bounty Winter 2026 | 1274 | 3 | 3 | -0.0002 | `{"beta":0.03978849775089088,"training_matches":967}` |
| IEM Krakow 2026 | 1305 | 4 | 4 | +0.0002 | `{"beta":0.053134752663857615,"training_matches":970}` |
| PGL Cluj-Napoca 2026 | 1355 | 33 | 33 | -0.0000 | `{"beta":0.040268030599972277,"training_matches":974}` |
| ESL Pro League Season 23 2026 | 1396 | 6 | 6 | +0.0000 | `{"beta":0.04764253677354709,"training_matches":1007}` |
| BLAST Open Rotterdam Spring 2026 | 1470 | 20 | 20 | -0.0002 | `{"beta":0.04724979471670519,"training_matches":1013}` |
| PGL Bucharest 2026 | 1499 | 27 | 27 | -0.0003 | `{"beta":0.11358025200788704,"training_matches":1033}` |
| IEM Rio 2026 | 1538 | 24 | 24 | -0.0002 | `{"beta":0.16965819346646976,"training_matches":1060}` |
| BLAST Rivals Spring 2026 | 1568 | 12 | 12 | +0.0008 | `{"beta":0.1916065237909808,"training_matches":1084}` |
| PGL Astana 2026 | 1581 | 31 | 31 | -0.0002 | `{"beta":0.14604346375526767,"training_matches":1096}` |
| IEM Atlanta 2026 | 1581 | 22 | 22 | -0.0008 | `{"beta":0.14604346375526767,"training_matches":1096}` |
| CS Asia Championships 2026 | 1652 | 26 | 26 | -0.0014 | `{"beta":0.2801386727172468,"training_matches":1149}` |
| IEM Cologne Major 2026 | 1682 | 97 | 97 | -0.0003 | `{"beta":0.3920898829228583,"training_matches":1175}` |
| XSE Pro League 2026 | 1788 | 31 | 31 | -0.0018 | `{"beta":0.4546229923296877,"training_matches":1272}` |
| BLAST Bounty Summer 2026 | 1828 | 3 | 3 | -0.0028 | `{"beta":0.5546816907565864,"training_matches":1303}` |
| Esports World Cup 2026 | 1859 | 38 | 38 | +0.0041 | `{"beta":0.5672712725239841,"training_matches":1306}` |
| BLAST Open Porto Fall 2026 | 1915 | 25 | 25 | +0.0002 | `{"beta":0.34014844438705394,"training_matches":1344}` |
| FISSURE PLAYGROUND Season 3 2026 | 1944 | 26 | 26 | +0.0001 | `{"beta":0.3301885851622456,"training_matches":1369}` |
| StarLadder StarSeries Fall 2026 | 1973 | 13 | 13 | +0.0005 | `{"beta":0.3203310906760739,"training_matches":1395}` |
| ESL Pro League Season 24 2026 | 1987 | 33 | 33 | -0.0006 | `{"beta":0.30529595780193475,"training_matches":1408}` |
| Thunderpick World Championship 2026 | 2028 | 13 | 13 | -0.0009 | `{"beta":0.3563078065491191,"training_matches":1441}` |
| PGL Masters Bucharest 2026 | 2042 | 19 | 19 | -0.0024 | `{"beta":0.3832718680667665,"training_matches":1454}` |
| IEM Beijing 2026 | 2083 | 26 | 26 | +0.0011 | `{"beta":0.47792905250836293,"training_matches":1473}` |
| BLAST Rivals Fall 2026 | 2113 | 11 | 11 | +0.0019 | `{"beta":0.42927105873242033,"training_matches":1499}` |

## `per_map_rating`: promote

Per-map Elo from completed Liquipedia series, averaged over equally likely map sets and blended with the market.

Verdict: the interval of the log-loss improvement excludes zero and the Matches with a market line are not worse.

| On the 1974 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.7973 | 0.2417 | 65.3% |
| per_map_rating | 0.6424 | 0.2257 | 64.0% |

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
| Perfect World Major Shanghai Major 2024 | 277 | 73 | 73 | +0.0215 | — |
| BLAST Premier Bounty season 1 2025 | 350 | 7 | 7 | -0.7584 | — |
| IEM Katowice 2025 | 357 | 48 | 47 | -0.2184 | — |
| PGL Cluj-Napoca 2025 | 406 | 41 | 38 | +0.0120 | — |
| ESL Pro League Season 21 2025 | 447 | 39 | 39 | +0.0393 | — |
| BLAST Open Lisbon Spring 2025 | 487 | 27 | 27 | -0.0273 | — |
| PGL Bucharest 2025 | 516 | 40 | 40 | -0.9821 | — |
| IEM Melbourne 2025 | 557 | 29 | 29 | -0.0846 | — |
| BLAST Rivals Spring 2025 | 586 | 13 | 13 | -0.1162 | — |
| PGL Astana 2025 | 599 | 39 | 39 | -1.1392 | — |
| IEM Dallas 2025 | 640 | 28 | 28 | +0.1160 | — |
| BLAST.tv Major Austin 2025 | 669 | 104 | 104 | -0.2555 | — |
| FISSURE PLAYGROUND Season 1 2025 | 775 | 27 | 27 | -0.7225 | — |
| IEM Cologne 2025 | 802 | 49 | 49 | +0.0817 | — |
| BLAST Bounty Fall 2025 | 851 | 28 | 28 | +0.1501 | — |
| Esports World Cup 2025 | 882 | 16 | 16 | -0.4718 | — |
| BLAST Open London Fall 2025 | 898 | 27 | 27 | -0.5400 | — |
| FISSURE PLAYGROUND Season 2 2025 | 927 | 40 | 40 | -0.2492 | — |
| ESL Pro League Season 22 2025 | 967 | 71 | 71 | -0.7222 | — |
| CS Asia Championships 2025 | 1040 | 26 | 26 | -0.2482 | — |
| Thunderpick World Championship 2025 | 1040 | 12 | 12 | -0.0212 | — |
| PGL Masters Bucharest 2025 | 1084 | 41 | 41 | -0.2589 | — |
| IEM Chengdu 2025 | 1125 | 30 | 30 | -0.0062 | — |
| BLAST Rivals Fall 2025 | 1155 | 13 | 13 | -0.0197 | — |
| StarLadder Major Budapest 2025 | 1168 | 102 | 102 | -0.1623 | — |
| BLAST Bounty Winter 2026 | 1274 | 24 | 24 | -0.0268 | — |
| IEM Krakow 2026 | 1305 | 50 | 50 | -0.0213 | — |
| PGL Cluj-Napoca 2026 | 1355 | 41 | 41 | +0.0331 | — |
| ESL Pro League Season 23 2026 | 1396 | 60 | 60 | -0.0259 | — |
| BLAST Open Rotterdam Spring 2026 | 1470 | 29 | 29 | -0.0149 | — |
| PGL Bucharest 2026 | 1499 | 36 | 36 | +0.0333 | — |
| IEM Rio 2026 | 1538 | 30 | 30 | +0.0343 | — |
| BLAST Rivals Spring 2026 | 1568 | 13 | 13 | -0.0273 | — |
| PGL Astana 2026 | 1581 | 33 | 33 | -0.0204 | — |
| IEM Atlanta 2026 | 1581 | 30 | 30 | -0.0134 | — |
| CS Asia Championships 2026 | 1652 | 30 | 30 | -0.1406 | — |
| IEM Cologne Major 2026 | 1682 | 103 | 103 | +0.0194 | — |
| XSE Pro League 2026 | 1788 | 35 | 35 | -0.0172 | — |
| BLAST Bounty Summer 2026 | 1828 | 26 | 26 | -0.0611 | — |
| Esports World Cup 2026 | 1859 | 50 | 50 | +0.0234 | — |
| BLAST Open Porto Fall 2026 | 1915 | 28 | 28 | +0.0150 | — |
| FISSURE PLAYGROUND Season 3 2026 | 1944 | 28 | 28 | -0.0511 | — |
| StarLadder StarSeries Fall 2026 | 1973 | 14 | 14 | -0.0471 | — |
| ESL Pro League Season 24 2026 | 1987 | 35 | 35 | +0.0024 | — |
| Thunderpick World Championship 2026 | 2028 | 14 | 14 | +0.0013 | — |
| PGL Masters Bucharest 2026 | 2042 | 41 | 41 | -0.0964 | — |
| IEM Beijing 2026 | 2083 | 30 | 30 | -0.0584 | — |
| BLAST Rivals Fall 2026 | 2113 | 13 | 13 | +0.0145 | — |

## `bo1_pool_depth`: no evidence (added after list fixed)

Bo1 map-pool depth, first-ban stability, pool overlap (interacted with anchor) and first-ban disruption from all prior complete HLTV vetoes, fitted as a walk-forward logit offset to the anchor.

Verdict: the interval of the log-loss difference includes zero (added after spec #33's candidate list was fixed).

| On the 251 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.7772 | 0.2640 | 60.2% |
| bo1_pool_depth | 0.7722 | 0.2628 | 60.6% |

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| BLAST Premier Groups Fall 2024 | 0 | 0 | 0 | — | `{"features":["pool_depth_diff","first_ban_stability_diff","pool_overlap_x_anchor","first_ban_disruption_diff"],"training_bo1_matches":0,"weights":[0.0,0.0,0.0,0.0]}` |
| IEM Cologne 2024 | 26 | 0 | 0 | — | `{"features":["pool_depth_diff","first_ban_stability_diff","pool_overlap_x_anchor","first_ban_disruption_diff"],"training_bo1_matches":0,"weights":[0.0,0.0,0.0,0.0]}` |
| BLAST Premier Showdown Fall 2024 | 75 | 0 | 0 | — | `{"features":["pool_depth_diff","first_ban_stability_diff","pool_overlap_x_anchor","first_ban_disruption_diff"],"training_bo1_matches":0,"weights":[0.0,0.0,0.0,0.0]}` |
| BetBoom Dacha Belgrade season 2 2024 | 75 | 2 | 2 | +0.0000 | `{"features":["pool_depth_diff","first_ban_stability_diff","pool_overlap_x_anchor","first_ban_disruption_diff"],"training_bo1_matches":0,"weights":[0.0,0.0,0.0,0.0]}` |
| ESL Pro League Season 20 2024 | 115 | 0 | 0 | — | `{"features":["pool_depth_diff","first_ban_stability_diff","pool_overlap_x_anchor","first_ban_disruption_diff"],"training_bo1_matches":2,"weights":[-0.006885,-0.007367,-0.000358,-0.006686]}` |
| BLAST Premier Final Fall 2024 | 194 | 0 | 0 | — | `{"features":["pool_depth_diff","first_ban_stability_diff","pool_overlap_x_anchor","first_ban_disruption_diff"],"training_bo1_matches":2,"weights":[-0.006885,-0.007367,-0.000358,-0.006686]}` |
| IEM Rio 2024 | 207 | 8 | 8 | +0.0003 | `{"features":["pool_depth_diff","first_ban_stability_diff","pool_overlap_x_anchor","first_ban_disruption_diff"],"training_bo1_matches":2,"weights":[-0.006885,-0.007367,-0.000358,-0.006686]}` |
| Thunderpick World Championship 2024 | 236 | 0 | 0 | — | `{"features":["pool_depth_diff","first_ban_stability_diff","pool_overlap_x_anchor","first_ban_disruption_diff"],"training_bo1_matches":10,"weights":[-0.03393,0.005855,0.015843,0.043698]}` |
| BLAST Premier World Final 2024 | 236 | 0 | 0 | — | `{"features":["pool_depth_diff","first_ban_stability_diff","pool_overlap_x_anchor","first_ban_disruption_diff"],"training_bo1_matches":10,"weights":[-0.03393,0.005855,0.015843,0.043698]}` |
| Perfect World Major Shanghai Major 2024 | 277 | 40 | 40 | -0.0007 | `{"features":["pool_depth_diff","first_ban_stability_diff","pool_overlap_x_anchor","first_ban_disruption_diff"],"training_bo1_matches":10,"weights":[-0.03393,0.005855,0.015843,0.043698]}` |
| BLAST Premier Bounty season 1 2025 | 350 | 0 | 0 | — | `{"features":["pool_depth_diff","first_ban_stability_diff","pool_overlap_x_anchor","first_ban_disruption_diff"],"training_bo1_matches":50,"weights":[-0.078229,-0.07756,0.107825,0.044335]}` |
| IEM Katowice 2025 | 357 | 0 | 0 | — | `{"features":["pool_depth_diff","first_ban_stability_diff","pool_overlap_x_anchor","first_ban_disruption_diff"],"training_bo1_matches":50,"weights":[-0.078229,-0.07756,0.107825,0.044335]}` |
| PGL Cluj-Napoca 2025 | 406 | 0 | 0 | — | `{"features":["pool_depth_diff","first_ban_stability_diff","pool_overlap_x_anchor","first_ban_disruption_diff"],"training_bo1_matches":50,"weights":[-0.078229,-0.07756,0.107825,0.044335]}` |
| ESL Pro League Season 21 2025 | 447 | 0 | 0 | — | `{"features":["pool_depth_diff","first_ban_stability_diff","pool_overlap_x_anchor","first_ban_disruption_diff"],"training_bo1_matches":50,"weights":[-0.078229,-0.07756,0.107825,0.044335]}` |
| BLAST Open Lisbon Spring 2025 | 487 | 0 | 0 | — | `{"features":["pool_depth_diff","first_ban_stability_diff","pool_overlap_x_anchor","first_ban_disruption_diff"],"training_bo1_matches":50,"weights":[-0.078229,-0.07756,0.107825,0.044335]}` |
| PGL Bucharest 2025 | 516 | 0 | 0 | — | `{"features":["pool_depth_diff","first_ban_stability_diff","pool_overlap_x_anchor","first_ban_disruption_diff"],"training_bo1_matches":50,"weights":[-0.078229,-0.07756,0.107825,0.044335]}` |
| IEM Melbourne 2025 | 557 | 0 | 0 | — | `{"features":["pool_depth_diff","first_ban_stability_diff","pool_overlap_x_anchor","first_ban_disruption_diff"],"training_bo1_matches":50,"weights":[-0.078229,-0.07756,0.107825,0.044335]}` |
| BLAST Rivals Spring 2025 | 586 | 0 | 0 | — | `{"features":["pool_depth_diff","first_ban_stability_diff","pool_overlap_x_anchor","first_ban_disruption_diff"],"training_bo1_matches":50,"weights":[-0.078229,-0.07756,0.107825,0.044335]}` |
| PGL Astana 2025 | 599 | 0 | 0 | — | `{"features":["pool_depth_diff","first_ban_stability_diff","pool_overlap_x_anchor","first_ban_disruption_diff"],"training_bo1_matches":50,"weights":[-0.078229,-0.07756,0.107825,0.044335]}` |
| IEM Dallas 2025 | 640 | 0 | 0 | — | `{"features":["pool_depth_diff","first_ban_stability_diff","pool_overlap_x_anchor","first_ban_disruption_diff"],"training_bo1_matches":50,"weights":[-0.078229,-0.07756,0.107825,0.044335]}` |
| BLAST.tv Major Austin 2025 | 669 | 58 | 58 | -0.0067 | `{"features":["pool_depth_diff","first_ban_stability_diff","pool_overlap_x_anchor","first_ban_disruption_diff"],"training_bo1_matches":50,"weights":[-0.078229,-0.07756,0.107825,0.044335]}` |
| FISSURE PLAYGROUND Season 1 2025 | 775 | 0 | 0 | — | `{"features":["pool_depth_diff","first_ban_stability_diff","pool_overlap_x_anchor","first_ban_disruption_diff"],"training_bo1_matches":108,"weights":[-0.132874,-0.215273,0.219762,0.169062]}` |
| IEM Cologne 2025 | 802 | 0 | 0 | — | `{"features":["pool_depth_diff","first_ban_stability_diff","pool_overlap_x_anchor","first_ban_disruption_diff"],"training_bo1_matches":108,"weights":[-0.132874,-0.215273,0.219762,0.169062]}` |
| BLAST Bounty Fall 2025 | 851 | 0 | 0 | — | `{"features":["pool_depth_diff","first_ban_stability_diff","pool_overlap_x_anchor","first_ban_disruption_diff"],"training_bo1_matches":108,"weights":[-0.132874,-0.215273,0.219762,0.169062]}` |
| Esports World Cup 2025 | 882 | 0 | 0 | — | `{"features":["pool_depth_diff","first_ban_stability_diff","pool_overlap_x_anchor","first_ban_disruption_diff"],"training_bo1_matches":108,"weights":[-0.132874,-0.215273,0.219762,0.169062]}` |
| BLAST Open London Fall 2025 | 898 | 0 | 0 | — | `{"features":["pool_depth_diff","first_ban_stability_diff","pool_overlap_x_anchor","first_ban_disruption_diff"],"training_bo1_matches":108,"weights":[-0.132874,-0.215273,0.219762,0.169062]}` |
| FISSURE PLAYGROUND Season 2 2025 | 927 | 0 | 0 | — | `{"features":["pool_depth_diff","first_ban_stability_diff","pool_overlap_x_anchor","first_ban_disruption_diff"],"training_bo1_matches":108,"weights":[-0.132874,-0.215273,0.219762,0.169062]}` |
| ESL Pro League Season 22 2025 | 967 | 0 | 0 | — | `{"features":["pool_depth_diff","first_ban_stability_diff","pool_overlap_x_anchor","first_ban_disruption_diff"],"training_bo1_matches":108,"weights":[-0.132874,-0.215273,0.219762,0.169062]}` |
| CS Asia Championships 2025 | 1040 | 7 | 7 | -0.0038 | `{"features":["pool_depth_diff","first_ban_stability_diff","pool_overlap_x_anchor","first_ban_disruption_diff"],"training_bo1_matches":108,"weights":[-0.132874,-0.215273,0.219762,0.169062]}` |
| Thunderpick World Championship 2025 | 1040 | 0 | 0 | — | `{"features":["pool_depth_diff","first_ban_stability_diff","pool_overlap_x_anchor","first_ban_disruption_diff"],"training_bo1_matches":108,"weights":[-0.132874,-0.215273,0.219762,0.169062]}` |
| PGL Masters Bucharest 2025 | 1084 | 0 | 0 | — | `{"features":["pool_depth_diff","first_ban_stability_diff","pool_overlap_x_anchor","first_ban_disruption_diff"],"training_bo1_matches":115,"weights":[-0.137887,-0.222089,0.210596,0.173883]}` |
| IEM Chengdu 2025 | 1125 | 0 | 0 | — | `{"features":["pool_depth_diff","first_ban_stability_diff","pool_overlap_x_anchor","first_ban_disruption_diff"],"training_bo1_matches":115,"weights":[-0.137887,-0.222089,0.210596,0.173883]}` |
| BLAST Rivals Fall 2025 | 1155 | 0 | 0 | — | `{"features":["pool_depth_diff","first_ban_stability_diff","pool_overlap_x_anchor","first_ban_disruption_diff"],"training_bo1_matches":115,"weights":[-0.137887,-0.222089,0.210596,0.173883]}` |
| StarLadder Major Budapest 2025 | 1168 | 60 | 60 | -0.0159 | `{"features":["pool_depth_diff","first_ban_stability_diff","pool_overlap_x_anchor","first_ban_disruption_diff"],"training_bo1_matches":115,"weights":[-0.137887,-0.222089,0.210596,0.173883]}` |
| BLAST Bounty Winter 2026 | 1274 | 0 | 0 | — | `{"features":["pool_depth_diff","first_ban_stability_diff","pool_overlap_x_anchor","first_ban_disruption_diff"],"training_bo1_matches":175,"weights":[-0.217379,-0.254555,0.345401,0.303267]}` |
| IEM Krakow 2026 | 1305 | 0 | 0 | — | `{"features":["pool_depth_diff","first_ban_stability_diff","pool_overlap_x_anchor","first_ban_disruption_diff"],"training_bo1_matches":175,"weights":[-0.217379,-0.254555,0.345401,0.303267]}` |
| PGL Cluj-Napoca 2026 | 1355 | 0 | 0 | — | `{"features":["pool_depth_diff","first_ban_stability_diff","pool_overlap_x_anchor","first_ban_disruption_diff"],"training_bo1_matches":175,"weights":[-0.217379,-0.254555,0.345401,0.303267]}` |
| ESL Pro League Season 23 2026 | 1396 | 0 | 0 | — | `{"features":["pool_depth_diff","first_ban_stability_diff","pool_overlap_x_anchor","first_ban_disruption_diff"],"training_bo1_matches":175,"weights":[-0.217379,-0.254555,0.345401,0.303267]}` |
| BLAST Open Rotterdam Spring 2026 | 1470 | 0 | 0 | — | `{"features":["pool_depth_diff","first_ban_stability_diff","pool_overlap_x_anchor","first_ban_disruption_diff"],"training_bo1_matches":175,"weights":[-0.217379,-0.254555,0.345401,0.303267]}` |
| PGL Bucharest 2026 | 1499 | 0 | 0 | — | `{"features":["pool_depth_diff","first_ban_stability_diff","pool_overlap_x_anchor","first_ban_disruption_diff"],"training_bo1_matches":175,"weights":[-0.217379,-0.254555,0.345401,0.303267]}` |
| IEM Rio 2026 | 1538 | 0 | 0 | — | `{"features":["pool_depth_diff","first_ban_stability_diff","pool_overlap_x_anchor","first_ban_disruption_diff"],"training_bo1_matches":175,"weights":[-0.217379,-0.254555,0.345401,0.303267]}` |
| BLAST Rivals Spring 2026 | 1568 | 0 | 0 | — | `{"features":["pool_depth_diff","first_ban_stability_diff","pool_overlap_x_anchor","first_ban_disruption_diff"],"training_bo1_matches":175,"weights":[-0.217379,-0.254555,0.345401,0.303267]}` |
| PGL Astana 2026 | 1581 | 0 | 0 | — | `{"features":["pool_depth_diff","first_ban_stability_diff","pool_overlap_x_anchor","first_ban_disruption_diff"],"training_bo1_matches":175,"weights":[-0.217379,-0.254555,0.345401,0.303267]}` |
| IEM Atlanta 2026 | 1581 | 0 | 0 | — | `{"features":["pool_depth_diff","first_ban_stability_diff","pool_overlap_x_anchor","first_ban_disruption_diff"],"training_bo1_matches":175,"weights":[-0.217379,-0.254555,0.345401,0.303267]}` |
| CS Asia Championships 2026 | 1652 | 8 | 8 | -0.0072 | `{"features":["pool_depth_diff","first_ban_stability_diff","pool_overlap_x_anchor","first_ban_disruption_diff"],"training_bo1_matches":175,"weights":[-0.217379,-0.254555,0.345401,0.303267]}` |
| IEM Cologne Major 2026 | 1682 | 37 | 37 | +0.0050 | `{"features":["pool_depth_diff","first_ban_stability_diff","pool_overlap_x_anchor","first_ban_disruption_diff"],"training_bo1_matches":183,"weights":[-0.227975,-0.230529,0.338806,0.332706]}` |
| XSE Pro League 2026 | 1788 | 17 | 17 | -0.0257 | `{"features":["pool_depth_diff","first_ban_stability_diff","pool_overlap_x_anchor","first_ban_disruption_diff"],"training_bo1_matches":220,"weights":[-0.22194,-0.154526,0.295793,0.380705]}` |
| BLAST Bounty Summer 2026 | 1828 | 0 | 0 | — | `{"features":["pool_depth_diff","first_ban_stability_diff","pool_overlap_x_anchor","first_ban_disruption_diff"],"training_bo1_matches":237,"weights":[-0.231094,-0.150056,0.332018,0.420502]}` |
| Esports World Cup 2026 | 1859 | 14 | 14 | +0.0310 | `{"features":["pool_depth_diff","first_ban_stability_diff","pool_overlap_x_anchor","first_ban_disruption_diff"],"training_bo1_matches":237,"weights":[-0.231094,-0.150056,0.332018,0.420502]}` |
| BLAST Open Porto Fall 2026 | 1915 | 0 | 0 | — | `{"features":["pool_depth_diff","first_ban_stability_diff","pool_overlap_x_anchor","first_ban_disruption_diff"],"training_bo1_matches":251,"weights":[-0.182888,-0.210471,0.310959,0.352606]}` |
| FISSURE PLAYGROUND Season 3 2026 | 1944 | 0 | 0 | — | `{"features":["pool_depth_diff","first_ban_stability_diff","pool_overlap_x_anchor","first_ban_disruption_diff"],"training_bo1_matches":251,"weights":[-0.182888,-0.210471,0.310959,0.352606]}` |
| StarLadder StarSeries Fall 2026 | 1973 | 0 | 0 | — | `{"features":["pool_depth_diff","first_ban_stability_diff","pool_overlap_x_anchor","first_ban_disruption_diff"],"training_bo1_matches":251,"weights":[-0.182888,-0.210471,0.310959,0.352606]}` |
| ESL Pro League Season 24 2026 | 1987 | 0 | 0 | — | `{"features":["pool_depth_diff","first_ban_stability_diff","pool_overlap_x_anchor","first_ban_disruption_diff"],"training_bo1_matches":251,"weights":[-0.182888,-0.210471,0.310959,0.352606]}` |
| Thunderpick World Championship 2026 | 2028 | 0 | 0 | — | `{"features":["pool_depth_diff","first_ban_stability_diff","pool_overlap_x_anchor","first_ban_disruption_diff"],"training_bo1_matches":251,"weights":[-0.182888,-0.210471,0.310959,0.352606]}` |
| PGL Masters Bucharest 2026 | 2042 | 0 | 0 | — | `{"features":["pool_depth_diff","first_ban_stability_diff","pool_overlap_x_anchor","first_ban_disruption_diff"],"training_bo1_matches":251,"weights":[-0.182888,-0.210471,0.310959,0.352606]}` |
| IEM Beijing 2026 | 2083 | 0 | 0 | — | `{"features":["pool_depth_diff","first_ban_stability_diff","pool_overlap_x_anchor","first_ban_disruption_diff"],"training_bo1_matches":251,"weights":[-0.182888,-0.210471,0.310959,0.352606]}` |
| BLAST Rivals Fall 2026 | 2113 | 0 | 0 | — | `{"features":["pool_depth_diff","first_ban_stability_diff","pool_overlap_x_anchor","first_ban_disruption_diff"],"training_bo1_matches":251,"weights":[-0.182888,-0.210471,0.310959,0.352606]}` |

## `bo1_veto_sim`: promote

Bo1 ban simulation from past HLTV vetoes and map results, blended with the market.

Verdict: the interval of the log-loss improvement excludes zero and the Matches with a market line are not worse.

| On the 251 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.7817 | 0.2660 | 59.4% |
| bo1_veto_sim | 0.6999 | 0.2491 | 61.4% |

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
| Perfect World Major Shanghai Major 2024 | 277 | 40 | 40 | -0.0189 | — |
| BLAST Premier Bounty season 1 2025 | 350 | 0 | 0 | — | — |
| IEM Katowice 2025 | 357 | 0 | 0 | — | — |
| PGL Cluj-Napoca 2025 | 406 | 0 | 0 | — | — |
| ESL Pro League Season 21 2025 | 447 | 0 | 0 | — | — |
| BLAST Open Lisbon Spring 2025 | 487 | 0 | 0 | — | — |
| PGL Bucharest 2025 | 516 | 0 | 0 | — | — |
| IEM Melbourne 2025 | 557 | 0 | 0 | — | — |
| BLAST Rivals Spring 2025 | 586 | 0 | 0 | — | — |
| PGL Astana 2025 | 599 | 0 | 0 | — | — |
| IEM Dallas 2025 | 640 | 0 | 0 | — | — |
| BLAST.tv Major Austin 2025 | 669 | 58 | 58 | -0.1900 | — |
| FISSURE PLAYGROUND Season 1 2025 | 775 | 0 | 0 | — | — |
| IEM Cologne 2025 | 802 | 0 | 0 | — | — |
| BLAST Bounty Fall 2025 | 851 | 0 | 0 | — | — |
| Esports World Cup 2025 | 882 | 0 | 0 | — | — |
| BLAST Open London Fall 2025 | 898 | 0 | 0 | — | — |
| FISSURE PLAYGROUND Season 2 2025 | 927 | 0 | 0 | — | — |
| ESL Pro League Season 22 2025 | 967 | 0 | 0 | — | — |
| CS Asia Championships 2025 | 1040 | 8 | 8 | -0.0926 | — |
| Thunderpick World Championship 2025 | 1040 | 0 | 0 | — | — |
| PGL Masters Bucharest 2025 | 1084 | 0 | 0 | — | — |
| IEM Chengdu 2025 | 1125 | 0 | 0 | — | — |
| BLAST Rivals Fall 2025 | 1155 | 0 | 0 | — | — |
| StarLadder Major Budapest 2025 | 1168 | 60 | 60 | -0.1147 | — |
| BLAST Bounty Winter 2026 | 1274 | 0 | 0 | — | — |
| IEM Krakow 2026 | 1305 | 0 | 0 | — | — |
| PGL Cluj-Napoca 2026 | 1355 | 0 | 0 | — | — |
| ESL Pro League Season 23 2026 | 1396 | 0 | 0 | — | — |
| BLAST Open Rotterdam Spring 2026 | 1470 | 0 | 0 | — | — |
| PGL Bucharest 2026 | 1499 | 0 | 0 | — | — |
| IEM Rio 2026 | 1538 | 0 | 0 | — | — |
| BLAST Rivals Spring 2026 | 1568 | 0 | 0 | — | — |
| PGL Astana 2026 | 1581 | 0 | 0 | — | — |
| IEM Atlanta 2026 | 1581 | 0 | 0 | — | — |
| CS Asia Championships 2026 | 1652 | 8 | 8 | -0.0636 | — |
| IEM Cologne Major 2026 | 1682 | 40 | 40 | +0.0124 | — |
| XSE Pro League 2026 | 1788 | 19 | 18 | -0.0271 | — |
| BLAST Bounty Summer 2026 | 1828 | 0 | 0 | — | — |
| Esports World Cup 2026 | 1859 | 16 | 16 | -0.0079 | — |
| BLAST Open Porto Fall 2026 | 1915 | 0 | 0 | — | — |
| FISSURE PLAYGROUND Season 3 2026 | 1944 | 0 | 0 | — | — |
| StarLadder StarSeries Fall 2026 | 1973 | 0 | 0 | — | — |
| ESL Pro League Season 24 2026 | 1987 | 0 | 0 | — | — |
| Thunderpick World Championship 2026 | 2028 | 0 | 0 | — | — |
| PGL Masters Bucharest 2026 | 2042 | 0 | 0 | — | — |
| IEM Beijing 2026 | 2083 | 0 | 0 | — | — |
| BLAST Rivals Fall 2026 | 2113 | 0 | 0 | — | — |

## `bo3_veto_sim`: no evidence (added after list fixed)

Bo3 veto simulation from past HLTV vetoes and per-map Elo ratings, blended with the market.

Verdict: better probabilities but fewer winners picked: the interval of the log-loss difference is below zero, that of the accuracy difference too (added after spec #33's candidate list was fixed).

| On the 1771 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.7962 | 0.2363 | 66.5% |
| bo3_veto_sim | 0.6349 | 0.2224 | 63.6% |

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
| Perfect World Major Shanghai Major 2024 | 277 | 33 | 33 | +0.0370 | — |
| BLAST Premier Bounty season 1 2025 | 350 | 6 | 6 | -0.9750 | — |
| IEM Katowice 2025 | 357 | 48 | 46 | -0.2025 | — |
| PGL Cluj-Napoca 2025 | 406 | 40 | 37 | +0.0340 | — |
| ESL Pro League Season 21 2025 | 447 | 39 | 39 | +0.0342 | — |
| BLAST Open Lisbon Spring 2025 | 487 | 28 | 28 | +0.0035 | — |
| PGL Bucharest 2025 | 516 | 40 | 40 | -0.9827 | — |
| IEM Melbourne 2025 | 557 | 28 | 28 | -0.1081 | — |
| BLAST Rivals Spring 2025 | 586 | 12 | 12 | -0.1850 | — |
| PGL Astana 2025 | 599 | 40 | 40 | -1.1004 | — |
| IEM Dallas 2025 | 640 | 28 | 28 | +0.1028 | — |
| BLAST.tv Major Austin 2025 | 669 | 46 | 46 | -0.2166 | — |
| FISSURE PLAYGROUND Season 1 2025 | 775 | 26 | 26 | -0.7117 | — |
| IEM Cologne 2025 | 802 | 48 | 48 | +0.0868 | — |
| BLAST Bounty Fall 2025 | 851 | 30 | 29 | +0.1431 | — |
| Esports World Cup 2025 | 882 | 15 | 15 | -0.4907 | — |
| BLAST Open London Fall 2025 | 898 | 28 | 28 | -0.3428 | — |
| FISSURE PLAYGROUND Season 2 2025 | 927 | 39 | 39 | -0.1917 | — |
| ESL Pro League Season 22 2025 | 967 | 72 | 72 | -0.8857 | — |
| CS Asia Championships 2025 | 1040 | 21 | 21 | -0.1766 | — |
| Thunderpick World Championship 2025 | 1040 | 13 | 13 | -0.1525 | — |
| PGL Masters Bucharest 2025 | 1084 | 40 | 40 | -0.2925 | — |
| IEM Chengdu 2025 | 1125 | 29 | 29 | -0.0417 | — |
| BLAST Rivals Fall 2025 | 1155 | 12 | 12 | -0.0667 | — |
| StarLadder Major Budapest 2025 | 1168 | 45 | 45 | -0.1029 | — |
| BLAST Bounty Winter 2026 | 1274 | 30 | 30 | -0.0379 | — |
| IEM Krakow 2026 | 1305 | 49 | 49 | -0.0229 | — |
| PGL Cluj-Napoca 2026 | 1355 | 40 | 40 | +0.0350 | — |
| ESL Pro League Season 23 2026 | 1396 | 73 | 73 | -0.0279 | — |
| BLAST Open Rotterdam Spring 2026 | 1470 | 28 | 28 | -0.0215 | — |
| PGL Bucharest 2026 | 1499 | 38 | 38 | +0.0162 | — |
| IEM Rio 2026 | 1538 | 29 | 29 | +0.0258 | — |
| BLAST Rivals Spring 2026 | 1568 | 12 | 12 | -0.0287 | — |
| PGL Astana 2026 | 1581 | 40 | 40 | +0.0031 | — |
| IEM Atlanta 2026 | 1581 | 29 | 29 | -0.0121 | — |
| CS Asia Championships 2026 | 1652 | 21 | 21 | -0.1280 | — |
| IEM Cologne Major 2026 | 1682 | 65 | 65 | +0.0091 | — |
| XSE Pro League 2026 | 1788 | 19 | 18 | -0.0091 | — |
| BLAST Bounty Summer 2026 | 1828 | 30 | 29 | -0.0728 | — |
| Esports World Cup 2026 | 1859 | 39 | 39 | +0.0178 | — |
| BLAST Open Porto Fall 2026 | 1915 | 28 | 28 | +0.0128 | — |
| FISSURE PLAYGROUND Season 3 2026 | 1944 | 28 | 28 | -0.0483 | — |
| StarLadder StarSeries Fall 2026 | 1973 | 13 | 13 | -0.0564 | — |
| ESL Pro League Season 24 2026 | 1987 | 40 | 40 | -0.0420 | — |
| Thunderpick World Championship 2026 | 2028 | 13 | 13 | +0.0022 | — |
| PGL Masters Bucharest 2026 | 2042 | 40 | 40 | -0.0985 | — |
| IEM Beijing 2026 | 2083 | 29 | 29 | -0.0664 | — |
| BLAST Rivals Fall 2026 | 2113 | 12 | 12 | +0.0134 | — |

## `bo5_veto_sim`: no evidence (added after list fixed)

Bo5 veto simulation from past HLTV vetoes and per-map Elo ratings, blended with the market.

Verdict: computed on 52 paired Matches, under the floor of 100 (added after spec #33's candidate list was fixed).

| On the 52 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.7221 | 0.2165 | 75.0% |
| bo5_veto_sim | 0.6058 | 0.2086 | 69.2% |

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| BLAST Premier Groups Fall 2024 | 0 | 0 | 0 | — | — |
| IEM Cologne 2024 | 26 | 0 | 0 | — | — |
| BLAST Premier Showdown Fall 2024 | 75 | 0 | 0 | — | — |
| BetBoom Dacha Belgrade season 2 2024 | 75 | 1 | 1 | +0.5766 | — |
| ESL Pro League Season 20 2024 | 115 | 1 | 1 | +0.4204 | — |
| BLAST Premier Final Fall 2024 | 194 | 1 | 1 | -0.7448 | — |
| IEM Rio 2024 | 207 | 1 | 1 | +0.4600 | — |
| Thunderpick World Championship 2024 | 236 | 1 | 1 | +0.2040 | — |
| BLAST Premier World Final 2024 | 236 | 1 | 1 | +0.1676 | — |
| Perfect World Major Shanghai Major 2024 | 277 | 0 | 0 | — | — |
| BLAST Premier Bounty season 1 2025 | 350 | 1 | 1 | +0.5445 | — |
| IEM Katowice 2025 | 357 | 1 | 1 | -1.0242 | — |
| PGL Cluj-Napoca 2025 | 406 | 1 | 1 | -0.1517 | — |
| ESL Pro League Season 21 2025 | 447 | 1 | 1 | +0.0605 | — |
| BLAST Open Lisbon Spring 2025 | 487 | 1 | 1 | +0.0257 | — |
| PGL Bucharest 2025 | 516 | 1 | 1 | +0.3370 | — |
| IEM Melbourne 2025 | 557 | 1 | 1 | +0.2834 | — |
| BLAST Rivals Spring 2025 | 586 | 1 | 1 | +0.3160 | — |
| PGL Astana 2025 | 599 | 1 | 1 | +0.3063 | — |
| IEM Dallas 2025 | 640 | 1 | 1 | +0.3008 | — |
| BLAST.tv Major Austin 2025 | 669 | 0 | 0 | — | — |
| FISSURE PLAYGROUND Season 1 2025 | 775 | 1 | 1 | -1.1273 | — |
| IEM Cologne 2025 | 802 | 1 | 1 | -0.2279 | — |
| BLAST Bounty Fall 2025 | 851 | 1 | 1 | +0.3856 | — |
| Esports World Cup 2025 | 882 | 1 | 1 | +0.2553 | — |
| BLAST Open London Fall 2025 | 898 | 1 | 1 | -4.4050 | — |
| FISSURE PLAYGROUND Season 2 2025 | 927 | 1 | 1 | -2.5077 | — |
| ESL Pro League Season 22 2025 | 967 | 1 | 1 | -0.0757 | — |
| CS Asia Championships 2025 | 1040 | 1 | 1 | -2.0057 | — |
| Thunderpick World Championship 2025 | 1040 | 1 | 1 | +0.4714 | — |
| PGL Masters Bucharest 2025 | 1084 | 1 | 1 | +0.7283 | — |
| IEM Chengdu 2025 | 1125 | 1 | 1 | +0.5160 | — |
| BLAST Rivals Fall 2025 | 1155 | 1 | 1 | +0.2249 | — |
| StarLadder Major Budapest 2025 | 1168 | 1 | 1 | +0.2762 | — |
| BLAST Bounty Winter 2026 | 1274 | 1 | 1 | -0.4219 | — |
| IEM Krakow 2026 | 1305 | 1 | 1 | -0.0432 | — |
| PGL Cluj-Napoca 2026 | 1355 | 1 | 1 | -0.1637 | — |
| ESL Pro League Season 23 2026 | 1396 | 1 | 1 | -0.0523 | — |
| BLAST Open Rotterdam Spring 2026 | 1470 | 1 | 1 | +0.0862 | — |
| PGL Bucharest 2026 | 1499 | 1 | 1 | -0.0788 | — |
| IEM Rio 2026 | 1538 | 1 | 1 | +0.1367 | — |
| BLAST Rivals Spring 2026 | 1568 | 1 | 1 | +0.0208 | — |
| PGL Astana 2026 | 1581 | 1 | 1 | -0.0285 | — |
| IEM Atlanta 2026 | 1581 | 1 | 1 | +0.0977 | — |
| CS Asia Championships 2026 | 1652 | 1 | 1 | -1.7279 | — |
| IEM Cologne Major 2026 | 1682 | 1 | 1 | +0.1923 | — |
| XSE Pro League 2026 | 1788 | 1 | 1 | +0.7238 | — |
| BLAST Bounty Summer 2026 | 1828 | 1 | 1 | -0.2960 | — |
| Esports World Cup 2026 | 1859 | 1 | 1 | +0.1350 | — |
| BLAST Open Porto Fall 2026 | 1915 | 1 | 1 | +0.0584 | — |
| FISSURE PLAYGROUND Season 3 2026 | 1944 | 1 | 1 | +0.0705 | — |
| StarLadder StarSeries Fall 2026 | 1973 | 1 | 1 | +0.1116 | — |
| ESL Pro League Season 24 2026 | 1987 | 1 | 1 | -0.0054 | — |
| Thunderpick World Championship 2026 | 2028 | 1 | 1 | +0.1170 | — |
| PGL Masters Bucharest 2026 | 2042 | 1 | 1 | +0.1325 | — |
| IEM Beijing 2026 | 2083 | 1 | 1 | +0.1418 | — |
| BLAST Rivals Fall 2026 | 2113 | 1 | 1 | +0.1529 | — |

## `veto_sim`: no evidence (added after list fixed)

Format-agnostic veto simulation routing each Match to Bo1, Bo3 or Bo5 simulation.

Verdict: better probabilities but fewer winners picked: the interval of the log-loss difference is below zero, that of the accuracy difference too (added after spec #33's candidate list was fixed).

| On the 2074 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.7926 | 0.2394 | 65.8% |
| veto_sim | 0.6421 | 0.2253 | 63.5% |

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| BLAST Premier Groups Fall 2024 | 0 | 18 | 0 | — | — |
| IEM Cologne 2024 | 26 | 45 | 45 | -0.1465 | — |
| BLAST Premier Showdown Fall 2024 | 75 | 14 | 14 | -0.1479 | — |
| BetBoom Dacha Belgrade season 2 2024 | 75 | 24 | 24 | +0.1019 | — |
| ESL Pro League Season 20 2024 | 115 | 79 | 76 | -0.1917 | — |
| BLAST Premier Final Fall 2024 | 194 | 13 | 13 | +0.0616 | — |
| IEM Rio 2024 | 207 | 24 | 24 | -0.2534 | — |
| Thunderpick World Championship 2024 | 236 | 28 | 28 | -0.2864 | — |
| BLAST Premier World Final 2024 | 236 | 13 | 13 | -0.4740 | — |
| Perfect World Major Shanghai Major 2024 | 277 | 73 | 73 | +0.0064 | — |
| BLAST Premier Bounty season 1 2025 | 350 | 7 | 7 | -0.7579 | — |
| IEM Katowice 2025 | 357 | 49 | 47 | -0.2200 | — |
| PGL Cluj-Napoca 2025 | 406 | 41 | 38 | +0.0291 | — |
| ESL Pro League Season 21 2025 | 447 | 40 | 40 | +0.0349 | — |
| BLAST Open Lisbon Spring 2025 | 487 | 29 | 29 | +0.0043 | — |
| PGL Bucharest 2025 | 516 | 41 | 41 | -0.9505 | — |
| IEM Melbourne 2025 | 557 | 29 | 29 | -0.0946 | — |
| BLAST Rivals Spring 2025 | 586 | 13 | 13 | -0.1464 | — |
| PGL Astana 2025 | 599 | 41 | 41 | -1.0661 | — |
| IEM Dallas 2025 | 640 | 29 | 29 | +0.1097 | — |
| BLAST.tv Major Austin 2025 | 669 | 104 | 104 | -0.2018 | — |
| FISSURE PLAYGROUND Season 1 2025 | 775 | 27 | 27 | -0.7271 | — |
| IEM Cologne 2025 | 802 | 49 | 49 | +0.0804 | — |
| BLAST Bounty Fall 2025 | 851 | 31 | 30 | +0.1512 | — |
| Esports World Cup 2025 | 882 | 16 | 16 | -0.4441 | — |
| BLAST Open London Fall 2025 | 898 | 29 | 29 | -0.4828 | — |
| FISSURE PLAYGROUND Season 2 2025 | 927 | 40 | 40 | -0.2496 | — |
| ESL Pro League Season 22 2025 | 967 | 73 | 73 | -0.8746 | — |
| CS Asia Championships 2025 | 1040 | 30 | 30 | -0.2151 | — |
| Thunderpick World Championship 2025 | 1040 | 14 | 14 | -0.1079 | — |
| PGL Masters Bucharest 2025 | 1084 | 41 | 41 | -0.2676 | — |
| IEM Chengdu 2025 | 1125 | 30 | 30 | -0.0231 | — |
| BLAST Rivals Fall 2025 | 1155 | 13 | 13 | -0.0443 | — |
| StarLadder Major Budapest 2025 | 1168 | 106 | 106 | -0.1060 | — |
| BLAST Bounty Winter 2026 | 1274 | 31 | 31 | -0.0503 | — |
| IEM Krakow 2026 | 1305 | 50 | 50 | -0.0233 | — |
| PGL Cluj-Napoca 2026 | 1355 | 41 | 41 | +0.0302 | — |
| ESL Pro League Season 23 2026 | 1396 | 74 | 74 | -0.0282 | — |
| BLAST Open Rotterdam Spring 2026 | 1470 | 29 | 29 | -0.0178 | — |
| PGL Bucharest 2026 | 1499 | 39 | 39 | +0.0137 | — |
| IEM Rio 2026 | 1538 | 30 | 30 | +0.0295 | — |
| BLAST Rivals Spring 2026 | 1568 | 13 | 13 | -0.0249 | — |
| PGL Astana 2026 | 1581 | 41 | 41 | +0.0023 | — |
| IEM Atlanta 2026 | 1581 | 30 | 30 | -0.0084 | — |
| CS Asia Championships 2026 | 1652 | 30 | 30 | -0.1642 | — |
| IEM Cologne Major 2026 | 1682 | 106 | 106 | +0.0120 | — |
| XSE Pro League 2026 | 1788 | 39 | 37 | +0.0020 | — |
| BLAST Bounty Summer 2026 | 1828 | 31 | 30 | -0.0803 | — |
| Esports World Cup 2026 | 1859 | 56 | 56 | +0.0126 | — |
| BLAST Open Porto Fall 2026 | 1915 | 29 | 29 | +0.0144 | — |
| FISSURE PLAYGROUND Season 3 2026 | 1944 | 29 | 29 | -0.0442 | — |
| StarLadder StarSeries Fall 2026 | 1973 | 14 | 14 | -0.0444 | — |
| ESL Pro League Season 24 2026 | 1987 | 41 | 41 | -0.0411 | — |
| Thunderpick World Championship 2026 | 2028 | 14 | 14 | +0.0104 | — |
| PGL Masters Bucharest 2026 | 2042 | 41 | 41 | -0.0929 | — |
| IEM Beijing 2026 | 2083 | 30 | 30 | -0.0594 | — |
| BLAST Rivals Fall 2026 | 2113 | 13 | 13 | +0.0241 | — |

## `rank_gap`: promote (added after list fixed)

the log rank gap of the two teams in Valve's dated standings (VRS) and in HLTV's dated weekly world ranking (HLTV match page badges), combined with the anchor's market and rating by the combiner fitted on the Events that ended before.

Verdict: the interval of the log-loss improvement excludes zero and the Matches with a market line are not worse (added after spec #33's candidate list was fixed).

| On the 2086 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.7945 | 0.2399 | 65.8% |
| rank_gap | 0.6313 | 0.2173 | 65.6% |

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| BLAST Premier Groups Fall 2024 | 0 | 0 | 0 | — | `{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.0,"matches":0,"rating":1.0,"vrs_rank_gap":0.0}}` |
| IEM Cologne 2024 | 26 | 49 | 49 | +0.0000 | `{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.0,"matches":0,"rating":1.0,"vrs_rank_gap":0.0}}` |
| BLAST Premier Showdown Fall 2024 | 75 | 14 | 14 | -0.2789 | `{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":-0.02176,"matches":49,"rating":0.409471,"vrs_rank_gap":-0.215441}}` |
| BetBoom Dacha Belgrade season 2 2024 | 75 | 26 | 26 | -0.0051 | `{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":-0.02176,"matches":49,"rating":0.409471,"vrs_rank_gap":-0.215441}}` |
| ESL Pro League Season 20 2024 | 115 | 76 | 76 | -0.2616 | `{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":-0.086946,"matches":89,"rating":0.442446,"vrs_rank_gap":-0.175325}}` |
| BLAST Premier Final Fall 2024 | 194 | 13 | 13 | -0.0360 | `{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.015577,"matches":165,"rating":0.335014,"vrs_rank_gap":-0.099077}}` |
| IEM Rio 2024 | 207 | 29 | 29 | -0.2879 | `{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.071755,"matches":178,"rating":0.325164,"vrs_rank_gap":-0.088663}}` |
| Thunderpick World Championship 2024 | 236 | 28 | 28 | -0.3151 | `{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.068988,"matches":207,"rating":0.32537,"vrs_rank_gap":-0.131194}}` |
| BLAST Premier World Final 2024 | 236 | 13 | 13 | -0.4360 | `{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.068988,"matches":207,"rating":0.32537,"vrs_rank_gap":-0.131194}}` |
| Perfect World Major Shanghai Major 2024 | 277 | 73 | 73 | -0.0324 | `{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.047765,"matches":248,"rating":0.312859,"vrs_rank_gap":-0.141674}}` |
| BLAST Premier Bounty season 1 2025 | 350 | 7 | 7 | -0.7522 | `{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":-0.037246,"matches":321,"rating":0.329973,"vrs_rank_gap":-0.026518}}` |
| IEM Katowice 2025 | 357 | 47 | 47 | -0.2219 | `{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":-0.013061,"matches":328,"rating":0.317961,"vrs_rank_gap":-0.042273}}` |
| PGL Cluj-Napoca 2025 | 406 | 38 | 38 | -0.0664 | `{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.066272,"matches":375,"rating":0.283406,"vrs_rank_gap":-0.04357}}` |
| ESL Pro League Season 21 2025 | 447 | 40 | 40 | -0.0600 | `{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.056733,"matches":413,"rating":0.311841,"vrs_rank_gap":-0.082452}}` |
| BLAST Open Lisbon Spring 2025 | 487 | 29 | 29 | -0.0805 | `{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.041095,"matches":453,"rating":0.302057,"vrs_rank_gap":-0.0041}}` |
| PGL Bucharest 2025 | 516 | 41 | 41 | -0.8647 | `{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.102953,"matches":482,"rating":0.292712,"vrs_rank_gap":-0.016506}}` |
| IEM Melbourne 2025 | 557 | 29 | 29 | -0.1763 | `{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.245056,"matches":523,"rating":0.202589,"vrs_rank_gap":-0.005085}}` |
| BLAST Rivals Spring 2025 | 586 | 13 | 13 | -0.1995 | `{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.252269,"matches":552,"rating":0.20264,"vrs_rank_gap":0.005889}}` |
| PGL Astana 2025 | 599 | 41 | 41 | -0.9902 | `{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.249039,"matches":565,"rating":0.199877,"vrs_rank_gap":0.021644}}` |
| IEM Dallas 2025 | 640 | 29 | 29 | -0.0112 | `{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.274676,"matches":606,"rating":0.13995,"vrs_rank_gap":0.095684}}` |
| BLAST.tv Major Austin 2025 | 669 | 106 | 106 | -0.2478 | `{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.270867,"matches":635,"rating":0.143415,"vrs_rank_gap":0.12409}}` |
| FISSURE PLAYGROUND Season 1 2025 | 775 | 27 | 27 | -0.6596 | `{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.249886,"matches":741,"rating":0.145173,"vrs_rank_gap":0.109418}}` |
| IEM Cologne 2025 | 802 | 49 | 49 | +0.0368 | `{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.241852,"matches":768,"rating":0.132758,"vrs_rank_gap":0.12577}}` |
| BLAST Bounty Fall 2025 | 851 | 30 | 30 | +0.0510 | `{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.238418,"matches":817,"rating":0.13049,"vrs_rank_gap":0.161306}}` |
| Esports World Cup 2025 | 882 | 16 | 16 | -0.4367 | `{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.213206,"matches":847,"rating":0.14447,"vrs_rank_gap":0.172079}}` |
| BLAST Open London Fall 2025 | 898 | 29 | 29 | -0.4788 | `{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.195363,"matches":863,"rating":0.150646,"vrs_rank_gap":0.155932}}` |
| FISSURE PLAYGROUND Season 2 2025 | 927 | 40 | 40 | -0.2670 | `{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.164614,"matches":892,"rating":0.147845,"vrs_rank_gap":0.179914}}` |
| ESL Pro League Season 22 2025 | 967 | 73 | 73 | -0.7966 | `{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.219793,"matches":932,"rating":0.140584,"vrs_rank_gap":0.15281}}` |
| CS Asia Championships 2025 | 1040 | 30 | 30 | -0.2197 | `{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.231234,"matches":1005,"rating":0.094034,"vrs_rank_gap":0.206985}}` |
| Thunderpick World Championship 2025 | 1040 | 14 | 14 | -0.1729 | `{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.231234,"matches":1005,"rating":0.094034,"vrs_rank_gap":0.206985}}` |
| PGL Masters Bucharest 2025 | 1084 | 41 | 41 | -0.2680 | `{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.231583,"matches":1049,"rating":0.092819,"vrs_rank_gap":0.208869}}` |
| IEM Chengdu 2025 | 1125 | 30 | 30 | -0.0923 | `{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.257279,"matches":1090,"rating":0.097478,"vrs_rank_gap":0.170345}}` |
| BLAST Rivals Fall 2025 | 1155 | 13 | 13 | -0.1267 | `{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.259844,"matches":1120,"rating":0.082106,"vrs_rank_gap":0.218873}}` |
| StarLadder Major Budapest 2025 | 1168 | 106 | 106 | -0.1482 | `{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.255201,"matches":1133,"rating":0.075241,"vrs_rank_gap":0.243941}}` |
| BLAST Bounty Winter 2026 | 1274 | 31 | 31 | +0.0000 | `{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.266242,"matches":1239,"rating":0.089019,"vrs_rank_gap":0.19339}}` |
| IEM Krakow 2026 | 1305 | 50 | 50 | -0.0114 | `{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":-0.163029,"market":0.753519,"matches":31,"rating":0.015975,"vrs_rank_gap":-0.0065},"rating":{"hltv_rank_gap":0.266242,"matches":1239,"rating":0.089019,"vrs_rank_gap":0.19339}}` |
| PGL Cluj-Napoca 2026 | 1355 | 41 | 41 | +0.0581 | `{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":-0.127812,"market":0.792953,"matches":81,"rating":-0.04757,"vrs_rank_gap":0.153693},"rating":{"hltv_rank_gap":0.266242,"matches":1239,"rating":0.089019,"vrs_rank_gap":0.19339}}` |
| ESL Pro League Season 23 2026 | 1396 | 74 | 74 | -0.0150 | `{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.191326,"market":0.774571,"matches":122,"rating":-0.013495,"vrs_rank_gap":0.170116},"rating":{"hltv_rank_gap":0.266242,"matches":1239,"rating":0.089019,"vrs_rank_gap":0.19339}}` |
| BLAST Open Rotterdam Spring 2026 | 1470 | 29 | 29 | -0.0219 | `{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.040842,"market":0.757961,"matches":196,"rating":-0.039181,"vrs_rank_gap":0.176915},"rating":{"hltv_rank_gap":0.266242,"matches":1239,"rating":0.089019,"vrs_rank_gap":0.19339}}` |
| PGL Bucharest 2026 | 1499 | 39 | 39 | +0.0117 | `{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.05722,"market":0.719614,"matches":225,"rating":-0.029221,"vrs_rank_gap":0.173162},"rating":{"hltv_rank_gap":0.266242,"matches":1239,"rating":0.089019,"vrs_rank_gap":0.19339}}` |
| IEM Rio 2026 | 1538 | 30 | 30 | +0.0316 | `{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.059383,"market":0.761601,"matches":264,"rating":-0.002791,"vrs_rank_gap":0.128147},"rating":{"hltv_rank_gap":0.266242,"matches":1239,"rating":0.089019,"vrs_rank_gap":0.19339}}` |
| BLAST Rivals Spring 2026 | 1568 | 13 | 13 | -0.0137 | `{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.071061,"market":0.805622,"matches":292,"rating":0.004227,"vrs_rank_gap":0.115278},"rating":{"hltv_rank_gap":0.268592,"matches":1241,"rating":0.08794,"vrs_rank_gap":0.195389}}` |
| PGL Astana 2026 | 1581 | 41 | 41 | +0.0006 | `{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.032812,"market":0.793927,"matches":305,"rating":0.008243,"vrs_rank_gap":0.121626},"rating":{"hltv_rank_gap":0.268592,"matches":1241,"rating":0.08794,"vrs_rank_gap":0.195389}}` |
| IEM Atlanta 2026 | 1581 | 30 | 30 | -0.0170 | `{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.032812,"market":0.793927,"matches":305,"rating":0.008243,"vrs_rank_gap":0.121626},"rating":{"hltv_rank_gap":0.268592,"matches":1241,"rating":0.08794,"vrs_rank_gap":0.195389}}` |
| CS Asia Championships 2026 | 1652 | 30 | 30 | -0.1867 | `{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":-0.004064,"market":0.815711,"matches":376,"rating":0.019436,"vrs_rank_gap":0.107042},"rating":{"hltv_rank_gap":0.268592,"matches":1241,"rating":0.08794,"vrs_rank_gap":0.195389}}` |
| IEM Cologne Major 2026 | 1682 | 106 | 106 | +0.0074 | `{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":-0.004064,"market":0.815711,"matches":376,"rating":0.019436,"vrs_rank_gap":0.107042},"rating":{"hltv_rank_gap":0.276671,"matches":1271,"rating":0.08485,"vrs_rank_gap":0.200175}}` |
| XSE Pro League 2026 | 1788 | 37 | 37 | -0.0335 | `{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":-0.013597,"market":0.916638,"matches":480,"rating":0.029264,"vrs_rank_gap":0.062122},"rating":{"hltv_rank_gap":0.276881,"matches":1273,"rating":0.083946,"vrs_rank_gap":0.203501}}` |
| BLAST Bounty Summer 2026 | 1828 | 29 | 29 | -0.0578 | `{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":-0.013597,"market":0.916638,"matches":480,"rating":0.029264,"vrs_rank_gap":0.062122},"rating":{"hltv_rank_gap":0.283054,"matches":1310,"rating":0.088491,"vrs_rank_gap":0.187928}}` |
| Esports World Cup 2026 | 1859 | 56 | 56 | +0.0101 | `{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.05803,"market":0.852128,"matches":509,"rating":0.006602,"vrs_rank_gap":0.064219},"rating":{"hltv_rank_gap":0.283054,"matches":1310,"rating":0.088491,"vrs_rank_gap":0.187928}}` |
| BLAST Open Porto Fall 2026 | 1915 | 29 | 29 | +0.0047 | `{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.069173,"market":0.817644,"matches":565,"rating":0.037638,"vrs_rank_gap":0.027633},"rating":{"hltv_rank_gap":0.283054,"matches":1310,"rating":0.088491,"vrs_rank_gap":0.187928}}` |
| FISSURE PLAYGROUND Season 3 2026 | 1944 | 29 | 29 | -0.0263 | `{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.084994,"market":0.831797,"matches":594,"rating":0.011451,"vrs_rank_gap":0.07595},"rating":{"hltv_rank_gap":0.283054,"matches":1310,"rating":0.088491,"vrs_rank_gap":0.187928}}` |
| StarLadder StarSeries Fall 2026 | 1973 | 14 | 14 | -0.0326 | `{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.044303,"market":0.787052,"matches":623,"rating":0.000218,"vrs_rank_gap":0.13313},"rating":{"hltv_rank_gap":0.283054,"matches":1310,"rating":0.088491,"vrs_rank_gap":0.187928}}` |
| ESL Pro League Season 24 2026 | 1987 | 41 | 41 | -0.0295 | `{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.045177,"market":0.799538,"matches":637,"rating":0.000903,"vrs_rank_gap":0.116094},"rating":{"hltv_rank_gap":0.283054,"matches":1310,"rating":0.088491,"vrs_rank_gap":0.187928}}` |
| Thunderpick World Championship 2026 | 2028 | 14 | 14 | +0.0039 | `{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.01613,"market":0.877906,"matches":678,"rating":-0.001159,"vrs_rank_gap":0.079126},"rating":{"hltv_rank_gap":0.283054,"matches":1310,"rating":0.088491,"vrs_rank_gap":0.187928}}` |
| PGL Masters Bucharest 2026 | 2042 | 41 | 41 | -0.0788 | `{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.024616,"market":0.870009,"matches":692,"rating":0.005666,"vrs_rank_gap":0.066144},"rating":{"hltv_rank_gap":0.283054,"matches":1310,"rating":0.088491,"vrs_rank_gap":0.187928}}` |
| IEM Beijing 2026 | 2083 | 30 | 30 | -0.0531 | `{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.077051,"market":0.861283,"matches":733,"rating":0.014491,"vrs_rank_gap":-0.018858},"rating":{"hltv_rank_gap":0.283054,"matches":1310,"rating":0.088491,"vrs_rank_gap":0.187928}}` |
| BLAST Rivals Fall 2026 | 2113 | 13 | 13 | +0.0083 | `{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.062198,"market":0.916154,"matches":763,"rating":-0.007427,"vrs_rank_gap":-0.005741},"rating":{"hltv_rank_gap":0.283054,"matches":1310,"rating":0.088491,"vrs_rank_gap":0.187928}}` |

## `market_rank_rule`: no evidence (added after list fixed)

where the market favourite and the rank favourite (VRS, else HLTV, else the Valve rating) disagree, picks the side that won more often in such disagreements on the Events that ended before, by size of the disagreement and series format; where they agree it follows the anchor.

Verdict: the interval of the log-loss difference includes zero (added after spec #33's candidate list was fixed).

127 disagreement Matches (market favourite and rank favourite differ): accuracy of the anchor 0.5354, of `market_rank_rule` 0.4488.

| On the 2087 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.7943 | 0.2398 | 65.8% |
| market_rank_rule | 0.7976 | 0.2412 | 65.3% |

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| BLAST Premier Groups Fall 2024 | 0 | 0 | 0 | — | `{"cells":[]}` |
| IEM Cologne 2024 | 26 | 49 | 49 | +0.0000 | `{"cells":[]}` |
| BLAST Premier Showdown Fall 2024 | 75 | 14 | 14 | +0.0000 | `{"cells":[]}` |
| BetBoom Dacha Belgrade season 2 2024 | 75 | 26 | 26 | +0.0000 | `{"cells":[]}` |
| ESL Pro League Season 20 2024 | 115 | 76 | 76 | +0.0000 | `{"cells":[]}` |
| BLAST Premier Final Fall 2024 | 194 | 13 | 13 | +0.0000 | `{"cells":[]}` |
| IEM Rio 2024 | 207 | 29 | 29 | +0.0000 | `{"cells":[]}` |
| Thunderpick World Championship 2024 | 236 | 28 | 28 | +0.0000 | `{"cells":[]}` |
| BLAST Premier World Final 2024 | 236 | 13 | 13 | +0.0000 | `{"cells":[]}` |
| Perfect World Major Shanghai Major 2024 | 277 | 73 | 73 | +0.0000 | `{"cells":[]}` |
| BLAST Premier Bounty season 1 2025 | 350 | 7 | 7 | +0.0000 | `{"cells":[]}` |
| IEM Katowice 2025 | 357 | 47 | 47 | +0.0000 | `{"cells":[]}` |
| PGL Cluj-Napoca 2025 | 406 | 38 | 38 | +0.0000 | `{"cells":[]}` |
| ESL Pro League Season 21 2025 | 447 | 40 | 40 | +0.0000 | `{"cells":[]}` |
| BLAST Open Lisbon Spring 2025 | 487 | 29 | 29 | +0.0000 | `{"cells":[]}` |
| PGL Bucharest 2025 | 516 | 41 | 41 | +0.0000 | `{"cells":[]}` |
| IEM Melbourne 2025 | 557 | 29 | 29 | +0.0000 | `{"cells":[]}` |
| BLAST Rivals Spring 2025 | 586 | 13 | 13 | +0.0000 | `{"cells":[]}` |
| PGL Astana 2025 | 599 | 41 | 41 | +0.0000 | `{"cells":[]}` |
| IEM Dallas 2025 | 640 | 29 | 29 | +0.0000 | `{"cells":[]}` |
| BLAST.tv Major Austin 2025 | 669 | 106 | 106 | +0.0000 | `{"cells":[]}` |
| FISSURE PLAYGROUND Season 1 2025 | 775 | 27 | 27 | +0.0000 | `{"cells":[]}` |
| IEM Cologne 2025 | 802 | 49 | 49 | +0.0000 | `{"cells":[]}` |
| BLAST Bounty Fall 2025 | 851 | 30 | 30 | +0.0000 | `{"cells":[]}` |
| Esports World Cup 2025 | 882 | 16 | 16 | +0.0000 | `{"cells":[]}` |
| BLAST Open London Fall 2025 | 898 | 29 | 29 | +0.0000 | `{"cells":[]}` |
| FISSURE PLAYGROUND Season 2 2025 | 927 | 40 | 40 | +0.0000 | `{"cells":[]}` |
| ESL Pro League Season 22 2025 | 967 | 73 | 73 | +0.0000 | `{"cells":[]}` |
| CS Asia Championships 2025 | 1040 | 30 | 30 | +0.0000 | `{"cells":[]}` |
| Thunderpick World Championship 2025 | 1040 | 14 | 14 | +0.0000 | `{"cells":[]}` |
| PGL Masters Bucharest 2025 | 1084 | 41 | 41 | +0.0000 | `{"cells":[]}` |
| IEM Chengdu 2025 | 1125 | 30 | 30 | +0.0000 | `{"cells":[]}` |
| BLAST Rivals Fall 2025 | 1155 | 13 | 13 | +0.0000 | `{"cells":[]}` |
| StarLadder Major Budapest 2025 | 1168 | 106 | 106 | +0.0000 | `{"cells":[]}` |
| BLAST Bounty Winter 2026 | 1274 | 31 | 31 | +0.0000 | `{"cells":[]}` |
| IEM Krakow 2026 | 1305 | 50 | 50 | -0.0434 | `{"cells":[{"format":3,"rank_wins":4,"rows":5,"size_bucket":0}]}` |
| PGL Cluj-Napoca 2026 | 1355 | 41 | 41 | +0.1308 | `{"cells":[{"format":3,"rank_wins":10,"rows":12,"size_bucket":0},{"format":3,"rank_wins":0,"rows":3,"size_bucket":1},{"format":5,"rank_wins":0,"rows":1,"size_bucket":1},{"format":3,"rank_wins":2,"rows":2,"size_bucket":2}]}` |
| ESL Pro League Season 23 2026 | 1396 | 74 | 74 | +0.0058 | `{"cells":[{"format":3,"rank_wins":11,"rows":17,"size_bucket":0},{"format":5,"rank_wins":1,"rows":1,"size_bucket":0},{"format":3,"rank_wins":3,"rows":7,"size_bucket":1},{"format":5,"rank_wins":0,"rows":1,"size_bucket":1},{"format":3,"rank_wins":3,"rows":4,"size_bucket":2}]}` |
| BLAST Open Rotterdam Spring 2026 | 1470 | 29 | 29 | -0.0371 | `{"cells":[{"format":3,"rank_wins":17,"rows":28,"size_bucket":0},{"format":5,"rank_wins":1,"rows":2,"size_bucket":0},{"format":3,"rank_wins":4,"rows":8,"size_bucket":1},{"format":5,"rank_wins":0,"rows":1,"size_bucket":1},{"format":3,"rank_wins":3,"rows":4,"size_bucket":2}]}` |
| PGL Bucharest 2026 | 1499 | 39 | 39 | -0.0213 | `{"cells":[{"format":3,"rank_wins":22,"rows":34,"size_bucket":0},{"format":5,"rank_wins":1,"rows":2,"size_bucket":0},{"format":3,"rank_wins":4,"rows":10,"size_bucket":1},{"format":5,"rank_wins":0,"rows":1,"size_bucket":1},{"format":3,"rank_wins":3,"rows":4,"size_bucket":2}]}` |
| IEM Rio 2026 | 1538 | 30 | 30 | +0.0602 | `{"cells":[{"format":3,"rank_wins":25,"rows":38,"size_bucket":0},{"format":5,"rank_wins":1,"rows":3,"size_bucket":0},{"format":3,"rank_wins":4,"rows":10,"size_bucket":1},{"format":5,"rank_wins":0,"rows":1,"size_bucket":1},{"format":3,"rank_wins":3,"rows":4,"size_bucket":2}]}` |
| BLAST Rivals Spring 2026 | 1568 | 13 | 13 | +0.0242 | `{"cells":[{"format":3,"rank_wins":25,"rows":41,"size_bucket":0},{"format":5,"rank_wins":1,"rows":3,"size_bucket":0},{"format":3,"rank_wins":6,"rows":12,"size_bucket":1},{"format":5,"rank_wins":0,"rows":1,"size_bucket":1},{"format":3,"rank_wins":3,"rows":4,"size_bucket":2}]}` |
| PGL Astana 2026 | 1581 | 41 | 41 | +0.0340 | `{"cells":[{"format":3,"rank_wins":25,"rows":42,"size_bucket":0},{"format":5,"rank_wins":1,"rows":3,"size_bucket":0},{"format":3,"rank_wins":6,"rows":12,"size_bucket":1},{"format":5,"rank_wins":0,"rows":1,"size_bucket":1},{"format":3,"rank_wins":3,"rows":4,"size_bucket":2}]}` |
| IEM Atlanta 2026 | 1581 | 30 | 30 | +0.0011 | `{"cells":[{"format":3,"rank_wins":25,"rows":42,"size_bucket":0},{"format":5,"rank_wins":1,"rows":3,"size_bucket":0},{"format":3,"rank_wins":6,"rows":12,"size_bucket":1},{"format":5,"rank_wins":0,"rows":1,"size_bucket":1},{"format":3,"rank_wins":3,"rows":4,"size_bucket":2}]}` |
| CS Asia Championships 2026 | 1652 | 30 | 30 | +0.0000 | `{"cells":[{"format":3,"rank_wins":28,"rows":53,"size_bucket":0},{"format":5,"rank_wins":1,"rows":3,"size_bucket":0},{"format":3,"rank_wins":7,"rows":13,"size_bucket":1},{"format":5,"rank_wins":0,"rows":2,"size_bucket":1},{"format":3,"rank_wins":3,"rows":4,"size_bucket":2}]}` |
| IEM Cologne Major 2026 | 1682 | 106 | 106 | +0.0078 | `{"cells":[{"format":3,"rank_wins":28,"rows":53,"size_bucket":0},{"format":5,"rank_wins":1,"rows":3,"size_bucket":0},{"format":3,"rank_wins":7,"rows":13,"size_bucket":1},{"format":5,"rank_wins":0,"rows":2,"size_bucket":1},{"format":3,"rank_wins":3,"rows":4,"size_bucket":2}]}` |
| XSE Pro League 2026 | 1788 | 37 | 37 | +0.0000 | `{"cells":[{"format":1,"rank_wins":3,"rows":7,"size_bucket":0},{"format":3,"rank_wins":30,"rows":59,"size_bucket":0},{"format":5,"rank_wins":1,"rows":3,"size_bucket":0},{"format":1,"rank_wins":0,"rows":1,"size_bucket":1},{"format":3,"rank_wins":7,"rows":16,"size_bucket":1},{"format":5,"rank_wins":0,"rows":2,"size_bucket":1},{"format":3,"rank_wins":3,"rows":4,"size_bucket":2}]}` |
| BLAST Bounty Summer 2026 | 1828 | 30 | 30 | -0.0100 | `{"cells":[{"format":1,"rank_wins":3,"rows":7,"size_bucket":0},{"format":3,"rank_wins":30,"rows":59,"size_bucket":0},{"format":5,"rank_wins":1,"rows":3,"size_bucket":0},{"format":1,"rank_wins":0,"rows":1,"size_bucket":1},{"format":3,"rank_wins":7,"rows":16,"size_bucket":1},{"format":5,"rank_wins":0,"rows":2,"size_bucket":1},{"format":3,"rank_wins":3,"rows":4,"size_bucket":2}]}` |
| Esports World Cup 2026 | 1859 | 56 | 56 | -0.0138 | `{"cells":[{"format":1,"rank_wins":3,"rows":7,"size_bucket":0},{"format":3,"rank_wins":31,"rows":61,"size_bucket":0},{"format":5,"rank_wins":1,"rows":3,"size_bucket":0},{"format":1,"rank_wins":0,"rows":1,"size_bucket":1},{"format":3,"rank_wins":7,"rows":17,"size_bucket":1},{"format":5,"rank_wins":0,"rows":2,"size_bucket":1},{"format":3,"rank_wins":4,"rows":5,"size_bucket":2}]}` |
| BLAST Open Porto Fall 2026 | 1915 | 29 | 29 | +0.0328 | `{"cells":[{"format":1,"rank_wins":4,"rows":10,"size_bucket":0},{"format":3,"rank_wins":32,"rows":62,"size_bucket":0},{"format":5,"rank_wins":1,"rows":3,"size_bucket":0},{"format":1,"rank_wins":0,"rows":1,"size_bucket":1},{"format":3,"rank_wins":7,"rows":19,"size_bucket":1},{"format":5,"rank_wins":0,"rows":2,"size_bucket":1},{"format":3,"rank_wins":5,"rows":6,"size_bucket":2}]}` |
| FISSURE PLAYGROUND Season 3 2026 | 1944 | 29 | 29 | -0.0117 | `{"cells":[{"format":1,"rank_wins":4,"rows":10,"size_bucket":0},{"format":3,"rank_wins":32,"rows":65,"size_bucket":0},{"format":5,"rank_wins":1,"rows":3,"size_bucket":0},{"format":1,"rank_wins":0,"rows":1,"size_bucket":1},{"format":3,"rank_wins":9,"rows":21,"size_bucket":1},{"format":5,"rank_wins":0,"rows":2,"size_bucket":1},{"format":3,"rank_wins":5,"rows":6,"size_bucket":2}]}` |
| StarLadder StarSeries Fall 2026 | 1973 | 14 | 14 | +0.0000 | `{"cells":[{"format":1,"rank_wins":4,"rows":10,"size_bucket":0},{"format":3,"rank_wins":35,"rows":68,"size_bucket":0},{"format":5,"rank_wins":2,"rows":4,"size_bucket":0},{"format":1,"rank_wins":0,"rows":1,"size_bucket":1},{"format":3,"rank_wins":9,"rows":21,"size_bucket":1},{"format":5,"rank_wins":0,"rows":2,"size_bucket":1},{"format":3,"rank_wins":5,"rows":6,"size_bucket":2}]}` |
| ESL Pro League Season 24 2026 | 1987 | 41 | 41 | +0.0249 | `{"cells":[{"format":1,"rank_wins":4,"rows":10,"size_bucket":0},{"format":3,"rank_wins":35,"rows":68,"size_bucket":0},{"format":5,"rank_wins":2,"rows":4,"size_bucket":0},{"format":1,"rank_wins":0,"rows":1,"size_bucket":1},{"format":3,"rank_wins":9,"rows":21,"size_bucket":1},{"format":5,"rank_wins":0,"rows":2,"size_bucket":1},{"format":3,"rank_wins":5,"rows":6,"size_bucket":2}]}` |
| Thunderpick World Championship 2026 | 2028 | 14 | 14 | +0.0191 | `{"cells":[{"format":1,"rank_wins":4,"rows":10,"size_bucket":0},{"format":3,"rank_wins":35,"rows":71,"size_bucket":0},{"format":5,"rank_wins":2,"rows":4,"size_bucket":0},{"format":1,"rank_wins":0,"rows":1,"size_bucket":1},{"format":3,"rank_wins":9,"rows":22,"size_bucket":1},{"format":5,"rank_wins":0,"rows":2,"size_bucket":1},{"format":3,"rank_wins":5,"rows":7,"size_bucket":2}]}` |
| PGL Masters Bucharest 2026 | 2042 | 41 | 41 | -0.0134 | `{"cells":[{"format":1,"rank_wins":4,"rows":10,"size_bucket":0},{"format":3,"rank_wins":35,"rows":71,"size_bucket":0},{"format":5,"rank_wins":2,"rows":4,"size_bucket":0},{"format":1,"rank_wins":0,"rows":1,"size_bucket":1},{"format":3,"rank_wins":9,"rows":22,"size_bucket":1},{"format":5,"rank_wins":0,"rows":2,"size_bucket":1},{"format":3,"rank_wins":6,"rows":9,"size_bucket":2}]}` |
| IEM Beijing 2026 | 2083 | 30 | 30 | -0.0009 | `{"cells":[{"format":1,"rank_wins":4,"rows":10,"size_bucket":0},{"format":3,"rank_wins":35,"rows":71,"size_bucket":0},{"format":5,"rank_wins":2,"rows":4,"size_bucket":0},{"format":1,"rank_wins":0,"rows":1,"size_bucket":1},{"format":3,"rank_wins":9,"rows":23,"size_bucket":1},{"format":5,"rank_wins":0,"rows":2,"size_bucket":1},{"format":3,"rank_wins":8,"rows":12,"size_bucket":2}]}` |
| BLAST Rivals Fall 2026 | 2113 | 13 | 13 | +0.0365 | `{"cells":[{"format":1,"rank_wins":4,"rows":10,"size_bucket":0},{"format":3,"rank_wins":35,"rows":72,"size_bucket":0},{"format":5,"rank_wins":2,"rows":4,"size_bucket":0},{"format":1,"rank_wins":0,"rows":1,"size_bucket":1},{"format":3,"rank_wins":9,"rows":24,"size_bucket":1},{"format":5,"rank_wins":0,"rows":2,"size_bucket":1},{"format":3,"rank_wins":8,"rows":12,"size_bucket":2}]}` |

## `round_diff_rating`: promote (added after list fixed)

Opponent-adjusted Elo updated by mean normalized HLTV map round differential, blended with the market.

Verdict: the interval of the log-loss improvement excludes zero and the Matches with a market line are not worse (added after spec #33's candidate list was fixed).

| On the 2084 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.7929 | 0.2396 | 65.8% |
| round_diff_rating | 0.6391 | 0.2242 | 64.3% |

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| BLAST Premier Groups Fall 2024 | 0 | 18 | 0 | — | — |
| IEM Cologne 2024 | 26 | 46 | 46 | -0.1534 | — |
| BLAST Premier Showdown Fall 2024 | 75 | 14 | 14 | -0.1382 | — |
| BetBoom Dacha Belgrade season 2 2024 | 75 | 26 | 26 | +0.1002 | — |
| ESL Pro League Season 20 2024 | 115 | 79 | 76 | -0.1855 | — |
| BLAST Premier Final Fall 2024 | 194 | 13 | 13 | +0.0553 | — |
| IEM Rio 2024 | 207 | 29 | 29 | -0.2753 | — |
| Thunderpick World Championship 2024 | 236 | 28 | 28 | -0.2826 | — |
| BLAST Premier World Final 2024 | 236 | 13 | 13 | -0.4694 | — |
| Perfect World Major Shanghai Major 2024 | 277 | 73 | 73 | +0.0151 | — |
| BLAST Premier Bounty season 1 2025 | 350 | 7 | 7 | -0.7449 | — |
| IEM Katowice 2025 | 357 | 49 | 47 | -0.2235 | — |
| PGL Cluj-Napoca 2025 | 406 | 41 | 38 | +0.0084 | — |
| ESL Pro League Season 21 2025 | 447 | 40 | 40 | +0.0392 | — |
| BLAST Open Lisbon Spring 2025 | 487 | 29 | 29 | +0.0076 | — |
| PGL Bucharest 2025 | 516 | 41 | 41 | -0.9475 | — |
| IEM Melbourne 2025 | 557 | 29 | 29 | -0.0975 | — |
| BLAST Rivals Spring 2025 | 586 | 13 | 13 | -0.1314 | — |
| PGL Astana 2025 | 599 | 41 | 41 | -1.0564 | — |
| IEM Dallas 2025 | 640 | 29 | 29 | +0.1300 | — |
| BLAST.tv Major Austin 2025 | 669 | 106 | 106 | -0.2323 | — |
| FISSURE PLAYGROUND Season 1 2025 | 775 | 27 | 27 | -0.7063 | — |
| IEM Cologne 2025 | 802 | 49 | 49 | +0.0783 | — |
| BLAST Bounty Fall 2025 | 851 | 31 | 30 | +0.1544 | — |
| Esports World Cup 2025 | 882 | 16 | 16 | -0.4711 | — |
| BLAST Open London Fall 2025 | 898 | 29 | 29 | -0.4769 | — |
| FISSURE PLAYGROUND Season 2 2025 | 927 | 40 | 40 | -0.2537 | — |
| ESL Pro League Season 22 2025 | 967 | 73 | 73 | -0.8757 | — |
| CS Asia Championships 2025 | 1040 | 30 | 30 | -0.2321 | — |
| Thunderpick World Championship 2025 | 1040 | 14 | 14 | -0.0922 | — |
| PGL Masters Bucharest 2025 | 1084 | 41 | 41 | -0.2760 | — |
| IEM Chengdu 2025 | 1125 | 30 | 30 | -0.0250 | — |
| BLAST Rivals Fall 2025 | 1155 | 13 | 13 | -0.0618 | — |
| StarLadder Major Budapest 2025 | 1168 | 106 | 106 | -0.1402 | — |
| BLAST Bounty Winter 2026 | 1274 | 31 | 31 | -0.0502 | — |
| IEM Krakow 2026 | 1305 | 50 | 50 | -0.0213 | — |
| PGL Cluj-Napoca 2026 | 1355 | 41 | 41 | +0.0359 | — |
| ESL Pro League Season 23 2026 | 1396 | 74 | 74 | -0.0326 | — |
| BLAST Open Rotterdam Spring 2026 | 1470 | 29 | 29 | -0.0168 | — |
| PGL Bucharest 2026 | 1499 | 39 | 39 | +0.0146 | — |
| IEM Rio 2026 | 1538 | 30 | 30 | +0.0420 | — |
| BLAST Rivals Spring 2026 | 1568 | 13 | 13 | -0.0223 | — |
| PGL Astana 2026 | 1581 | 41 | 41 | +0.0061 | — |
| IEM Atlanta 2026 | 1581 | 30 | 30 | -0.0175 | — |
| CS Asia Championships 2026 | 1652 | 30 | 30 | -0.1448 | — |
| IEM Cologne Major 2026 | 1682 | 106 | 106 | +0.0150 | — |
| XSE Pro League 2026 | 1788 | 39 | 37 | -0.0358 | — |
| BLAST Bounty Summer 2026 | 1828 | 31 | 30 | -0.0802 | — |
| Esports World Cup 2026 | 1859 | 56 | 56 | +0.0159 | — |
| BLAST Open Porto Fall 2026 | 1915 | 29 | 29 | +0.0185 | — |
| FISSURE PLAYGROUND Season 3 2026 | 1944 | 29 | 29 | -0.0444 | — |
| StarLadder StarSeries Fall 2026 | 1973 | 14 | 14 | -0.0501 | — |
| ESL Pro League Season 24 2026 | 1987 | 41 | 41 | -0.0394 | — |
| Thunderpick World Championship 2026 | 2028 | 14 | 14 | +0.0049 | — |
| PGL Masters Bucharest 2026 | 2042 | 41 | 41 | -0.0947 | — |
| IEM Beijing 2026 | 2083 | 30 | 30 | -0.0561 | — |
| BLAST Rivals Fall 2026 | 2113 | 13 | 13 | +0.0180 | — |

## `stacker`: promote (added after list fixed)

a combiner fitted to pick winners: the market line and the calibrated Valve rating (one logit scale per format) as base inputs, own_rating, per_map_rating and the VRS and HLTV rank gaps as extras (a missing one reads as no shift), fitted on the Events that ended before. Objective: the weights minimise the log-loss penalised towards the anchor; the penalty, which sets how far the fit moves from the anchor, is chosen from 1/3/10/30/100/1000 by accuracy (share of winners picked at probability above one half) on the latest third of the training Events, refitted on the training Events before them (ties: lower log-loss, then the larger penalty); so the side it picks is that of the setting that picked most winners on Matches it was not fitted on, and its probability stays a log-loss fit.

Verdict: the interval of the log-loss improvement excludes zero and the Matches with a market line are not worse (added after spec #33's candidate list was fixed).

| On the 2087 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.7943 | 0.2398 | 65.8% |
| stacker | 0.6327 | 0.2181 | 65.8% |

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| BLAST Premier Groups Fall 2024 | 0 | 0 | 0 | — | `{"calibration_scales":{"bo1":null,"bo3":null,"bo5":null},"chosen_by":"default","fitted_through":null,"lambda":10.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":0,"training_matches":0,"validation":[],"validation_events":0,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.0,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"rating":1.0,"vrs_rank_gap":0.0}}}` |
| IEM Cologne 2024 | 26 | 49 | 49 | +0.0000 | `{"calibration_scales":{"bo1":null,"bo3":null,"bo5":null},"chosen_by":"default","fitted_through":"2024-08-04T14:20:00Z","lambda":10.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":1,"training_matches":26,"validation":[],"validation_events":0,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.0,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"rating":1.0,"vrs_rank_gap":0.0}}}` |
| BLAST Premier Showdown Fall 2024 | 75 | 14 | 14 | -0.2578 | `{"calibration_scales":{"bo1":0.253898,"bo3":0.259052,"bo5":0.253898},"chosen_by":"default","fitted_through":"2024-08-18T14:45:00Z","lambda":10.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":2,"training_matches":75,"validation":[],"validation_events":0,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.10612,"matches":49,"own_rating":0.210438,"per_map_rating":-0.019324,"rating":0.979842,"vrs_rank_gap":-0.080742}}}` |
| BetBoom Dacha Belgrade season 2 2024 | 75 | 26 | 26 | -0.0178 | `{"calibration_scales":{"bo1":0.253898,"bo3":0.259052,"bo5":0.253898},"chosen_by":"default","fitted_through":"2024-08-18T14:45:00Z","lambda":10.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":2,"training_matches":75,"validation":[],"validation_events":0,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.10612,"matches":49,"own_rating":0.210438,"per_map_rating":-0.019324,"rating":0.979842,"vrs_rank_gap":-0.080742}}}` |
| ESL Pro League Season 20 2024 | 115 | 76 | 76 | -0.2444 | `{"calibration_scales":{"bo1":0.324383,"bo3":0.320437,"bo5":0.324383},"chosen_by":"default","fitted_through":"2024-09-01T10:45:00Z","lambda":10.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":4,"training_matches":115,"validation":[{"accuracy":0.730769,"lambda":1.0,"log_loss":0.594395,"matches":26},{"accuracy":0.730769,"lambda":3.0,"log_loss":0.58518,"matches":26},{"accuracy":0.769231,"lambda":10.0,"log_loss":0.581925,"matches":26},{"accuracy":0.730769,"lambda":30.0,"log_loss":0.588332,"matches":26},{"accuracy":0.769231,"lambda":100.0,"log_loss":0.598776,"matches":26},{"accuracy":0.769231,"lambda":1000.0,"log_loss":0.607513,"matches":26}],"validation_events":0,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.034411,"matches":89,"own_rating":0.210186,"per_map_rating":0.003629,"rating":0.994416,"vrs_rank_gap":-0.062976}}}` |
| BLAST Premier Final Fall 2024 | 194 | 13 | 13 | -0.0468 | `{"calibration_scales":{"bo1":0.307282,"bo3":0.303027,"bo5":0.307282},"chosen_by":"validation_accuracy","fitted_through":"2024-09-22T14:45:00Z","lambda":30.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":5,"training_matches":194,"validation":[{"accuracy":0.644737,"lambda":1.0,"log_loss":0.632612,"matches":76},{"accuracy":0.644737,"lambda":3.0,"log_loss":0.627217,"matches":76},{"accuracy":0.644737,"lambda":10.0,"log_loss":0.618902,"matches":76},{"accuracy":0.657895,"lambda":30.0,"log_loss":0.608213,"matches":76},{"accuracy":0.644737,"lambda":100.0,"log_loss":0.598034,"matches":76},{"accuracy":0.644737,"lambda":1000.0,"log_loss":0.591686,"matches":76}],"validation_events":1,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.032638,"matches":165,"own_rating":0.044103,"per_map_rating":-0.004511,"rating":0.97772,"vrs_rank_gap":-0.005074}}}` |
| IEM Rio 2024 | 207 | 29 | 29 | -0.2661 | `{"calibration_scales":{"bo1":0.319241,"bo3":0.318395,"bo5":0.319241},"chosen_by":"validation_accuracy","fitted_through":"2024-09-29T13:45:00Z","lambda":30.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":6,"training_matches":207,"validation":[{"accuracy":0.651685,"lambda":1.0,"log_loss":0.611534,"matches":89},{"accuracy":0.651685,"lambda":3.0,"log_loss":0.608027,"matches":89},{"accuracy":0.651685,"lambda":10.0,"log_loss":0.602442,"matches":89},{"accuracy":0.662921,"lambda":30.0,"log_loss":0.595083,"matches":89},{"accuracy":0.651685,"lambda":100.0,"log_loss":0.588723,"matches":89},{"accuracy":0.651685,"lambda":1000.0,"log_loss":0.585877,"matches":89}],"validation_events":2,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.04733,"matches":178,"own_rating":0.067505,"per_map_rating":-0.011816,"rating":0.96065,"vrs_rank_gap":-0.020145}}}` |
| Thunderpick World Championship 2024 | 236 | 28 | 28 | -0.3136 | `{"calibration_scales":{"bo1":0.304606,"bo3":0.305638,"bo5":0.304606},"chosen_by":"validation_accuracy","fitted_through":"2024-10-13T16:45:00Z","lambda":1.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":7,"training_matches":236,"validation":[{"accuracy":0.642857,"lambda":1.0,"log_loss":0.642643,"matches":42},{"accuracy":0.619048,"lambda":3.0,"log_loss":0.643447,"matches":42},{"accuracy":0.619048,"lambda":10.0,"log_loss":0.64378,"matches":42},{"accuracy":0.619048,"lambda":30.0,"log_loss":0.643177,"matches":42},{"accuracy":0.619048,"lambda":100.0,"log_loss":0.641342,"matches":42},{"accuracy":0.619048,"lambda":1000.0,"log_loss":0.638693,"matches":42}],"validation_events":2,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.248086,"matches":207,"own_rating":0.034034,"per_map_rating":-0.589311,"rating":0.806867,"vrs_rank_gap":-0.089343}}}` |
| BLAST Premier World Final 2024 | 236 | 13 | 13 | -0.4565 | `{"calibration_scales":{"bo1":0.304606,"bo3":0.305638,"bo5":0.304606},"chosen_by":"validation_accuracy","fitted_through":"2024-10-13T16:45:00Z","lambda":1.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":7,"training_matches":236,"validation":[{"accuracy":0.642857,"lambda":1.0,"log_loss":0.642643,"matches":42},{"accuracy":0.619048,"lambda":3.0,"log_loss":0.643447,"matches":42},{"accuracy":0.619048,"lambda":10.0,"log_loss":0.64378,"matches":42},{"accuracy":0.619048,"lambda":30.0,"log_loss":0.643177,"matches":42},{"accuracy":0.619048,"lambda":100.0,"log_loss":0.641342,"matches":42},{"accuracy":0.619048,"lambda":1000.0,"log_loss":0.638693,"matches":42}],"validation_events":2,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.248086,"matches":207,"own_rating":0.034034,"per_map_rating":-0.589311,"rating":0.806867,"vrs_rank_gap":-0.089343}}}` |
| Perfect World Major Shanghai Major 2024 | 277 | 73 | 73 | -0.0291 | `{"calibration_scales":{"bo1":0.285003,"bo3":0.283926,"bo5":0.285003},"chosen_by":"validation_accuracy","fitted_through":"2024-11-03T16:15:00Z","lambda":3.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":9,"training_matches":277,"validation":[{"accuracy":0.628571,"lambda":1.0,"log_loss":0.69233,"matches":70},{"accuracy":0.657143,"lambda":3.0,"log_loss":0.691347,"matches":70},{"accuracy":0.642857,"lambda":10.0,"log_loss":0.689231,"matches":70},{"accuracy":0.642857,"lambda":30.0,"log_loss":0.686754,"matches":70},{"accuracy":0.642857,"lambda":100.0,"log_loss":0.682879,"matches":70},{"accuracy":0.642857,"lambda":1000.0,"log_loss":0.676572,"matches":70}],"validation_events":3,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.146183,"matches":248,"own_rating":0.028055,"per_map_rating":-0.252426,"rating":0.955958,"vrs_rank_gap":-0.113024}}}` |
| BLAST Premier Bounty season 1 2025 | 350 | 7 | 7 | -0.7711 | `{"calibration_scales":{"bo1":0.395913,"bo3":0.302271,"bo5":0.313307},"chosen_by":"validation_accuracy","fitted_through":"2024-12-15T09:05:00Z","lambda":1000.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":10,"training_matches":350,"validation":[{"accuracy":0.649123,"lambda":1.0,"log_loss":0.642941,"matches":114},{"accuracy":0.631579,"lambda":3.0,"log_loss":0.63911,"matches":114},{"accuracy":0.666667,"lambda":10.0,"log_loss":0.635563,"matches":114},{"accuracy":0.666667,"lambda":30.0,"log_loss":0.633505,"matches":114},{"accuracy":0.666667,"lambda":100.0,"log_loss":0.632486,"matches":114},{"accuracy":0.666667,"lambda":1000.0,"log_loss":0.632095,"matches":114}],"validation_events":3,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.000087,"matches":321,"own_rating":0.003603,"per_map_rating":-0.000281,"rating":1.00038,"vrs_rank_gap":0.000414}}}` |
| IEM Katowice 2025 | 357 | 47 | 47 | -0.2250 | `{"calibration_scales":{"bo1":0.395913,"bo3":0.292364,"bo5":0.305588},"chosen_by":"validation_accuracy","fitted_through":"2025-01-26T14:45:00Z","lambda":1000.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":11,"training_matches":357,"validation":[{"accuracy":0.648148,"lambda":1.0,"log_loss":0.629545,"matches":108},{"accuracy":0.62963,"lambda":3.0,"log_loss":0.625029,"matches":108},{"accuracy":0.666667,"lambda":10.0,"log_loss":0.621612,"matches":108},{"accuracy":0.666667,"lambda":30.0,"log_loss":0.619853,"matches":108},{"accuracy":0.666667,"lambda":100.0,"log_loss":0.619055,"matches":108},{"accuracy":0.666667,"lambda":1000.0,"log_loss":0.618888,"matches":108}],"validation_events":3,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.000923,"matches":328,"own_rating":0.003551,"per_map_rating":-0.000363,"rating":1.000559,"vrs_rank_gap":0.000447}}}` |
| PGL Cluj-Napoca 2025 | 406 | 38 | 38 | -0.0608 | `{"calibration_scales":{"bo1":0.395913,"bo3":0.289783,"bo5":0.299907},"chosen_by":"validation_accuracy","fitted_through":"2025-02-09T15:45:00Z","lambda":10.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":12,"training_matches":406,"validation":[{"accuracy":0.645161,"lambda":1.0,"log_loss":0.635022,"matches":155},{"accuracy":0.632258,"lambda":3.0,"log_loss":0.632134,"matches":155},{"accuracy":0.645161,"lambda":10.0,"log_loss":0.630803,"matches":155},{"accuracy":0.63871,"lambda":30.0,"log_loss":0.630448,"matches":155},{"accuracy":0.63871,"lambda":100.0,"log_loss":0.630358,"matches":155},{"accuracy":0.63871,"lambda":1000.0,"log_loss":0.630521,"matches":155}],"validation_events":4,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.069213,"matches":375,"own_rating":0.061894,"per_map_rating":-0.025971,"rating":0.951089,"vrs_rank_gap":-0.064604}}}` |
| ESL Pro League Season 21 2025 | 447 | 40 | 40 | -0.0601 | `{"calibration_scales":{"bo1":0.395913,"bo3":0.301953,"bo5":0.310583},"chosen_by":"validation_accuracy","fitted_through":"2025-02-23T17:45:00Z","lambda":10.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":13,"training_matches":447,"validation":[{"accuracy":0.630303,"lambda":1.0,"log_loss":0.626479,"matches":165},{"accuracy":0.636364,"lambda":3.0,"log_loss":0.623912,"matches":165},{"accuracy":0.642424,"lambda":10.0,"log_loss":0.622503,"matches":165},{"accuracy":0.636364,"lambda":30.0,"log_loss":0.622049,"matches":165},{"accuracy":0.636364,"lambda":100.0,"log_loss":0.622053,"matches":165},{"accuracy":0.636364,"lambda":1000.0,"log_loss":0.622523,"matches":165}],"validation_events":4,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.063556,"matches":413,"own_rating":0.047375,"per_map_rating":-0.022687,"rating":0.989237,"vrs_rank_gap":-0.080062}}}` |
| BLAST Open Lisbon Spring 2025 | 487 | 29 | 29 | -0.0840 | `{"calibration_scales":{"bo1":0.395913,"bo3":0.320069,"bo5":0.327154},"chosen_by":"validation_accuracy","fitted_through":"2025-03-16T15:45:00Z","lambda":30.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":14,"training_matches":487,"validation":[{"accuracy":0.674242,"lambda":1.0,"log_loss":0.596693,"matches":132},{"accuracy":0.666667,"lambda":3.0,"log_loss":0.595233,"matches":132},{"accuracy":0.666667,"lambda":10.0,"log_loss":0.594501,"matches":132},{"accuracy":0.674242,"lambda":30.0,"log_loss":0.594341,"matches":132},{"accuracy":0.674242,"lambda":100.0,"log_loss":0.594535,"matches":132},{"accuracy":0.674242,"lambda":1000.0,"log_loss":0.595314,"matches":132}],"validation_events":4,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.014323,"matches":453,"own_rating":0.052794,"per_map_rating":0.0181,"rating":0.969391,"vrs_rank_gap":-0.01527}}}` |
| PGL Bucharest 2025 | 516 | 41 | 41 | -0.8532 | `{"calibration_scales":{"bo1":0.395913,"bo3":0.328512,"bo5":0.335118},"chosen_by":"validation_accuracy","fitted_through":"2025-03-30T15:15:00Z","lambda":30.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":15,"training_matches":516,"validation":[{"accuracy":0.689441,"lambda":1.0,"log_loss":0.585184,"matches":161},{"accuracy":0.68323,"lambda":3.0,"log_loss":0.583096,"matches":161},{"accuracy":0.68323,"lambda":10.0,"log_loss":0.582035,"matches":161},{"accuracy":0.689441,"lambda":30.0,"log_loss":0.581785,"matches":161},{"accuracy":0.689441,"lambda":100.0,"log_loss":0.582027,"matches":161},{"accuracy":0.689441,"lambda":1000.0,"log_loss":0.583061,"matches":161}],"validation_events":5,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.04217,"matches":482,"own_rating":0.068976,"per_map_rating":0.038891,"rating":0.948169,"vrs_rank_gap":-0.031679}}}` |
| IEM Melbourne 2025 | 557 | 29 | 29 | -0.1677 | `{"calibration_scales":{"bo1":0.395913,"bo3":0.282383,"bo5":0.28989},"chosen_by":"validation_accuracy","fitted_through":"2025-04-13T14:45:00Z","lambda":1000.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":16,"training_matches":557,"validation":[{"accuracy":0.666667,"lambda":1.0,"log_loss":0.62005,"matches":195},{"accuracy":0.661538,"lambda":3.0,"log_loss":0.618629,"matches":195},{"accuracy":0.65641,"lambda":10.0,"log_loss":0.618056,"matches":195},{"accuracy":0.661538,"lambda":30.0,"log_loss":0.618073,"matches":195},{"accuracy":0.661538,"lambda":100.0,"log_loss":0.618215,"matches":195},{"accuracy":0.666667,"lambda":1000.0,"log_loss":0.618544,"matches":195}],"validation_events":5,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.012465,"matches":523,"own_rating":0.008804,"per_map_rating":0.001505,"rating":0.999111,"vrs_rank_gap":0.002348}}}` |
| BLAST Rivals Spring 2025 | 586 | 13 | 13 | -0.1970 | `{"calibration_scales":{"bo1":0.395913,"bo3":0.288126,"bo5":0.296077},"chosen_by":"validation_accuracy","fitted_through":"2025-04-27T06:15:00Z","lambda":10.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":17,"training_matches":586,"validation":[{"accuracy":0.689266,"lambda":1.0,"log_loss":0.590591,"matches":177},{"accuracy":0.689266,"lambda":3.0,"log_loss":0.590566,"matches":177},{"accuracy":0.700565,"lambda":10.0,"log_loss":0.591568,"matches":177},{"accuracy":0.700565,"lambda":30.0,"log_loss":0.592838,"matches":177},{"accuracy":0.700565,"lambda":100.0,"log_loss":0.593575,"matches":177},{"accuracy":0.700565,"lambda":1000.0,"log_loss":0.5941,"matches":177}],"validation_events":5,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.233661,"matches":552,"own_rating":0.019154,"per_map_rating":0.077705,"rating":0.815234,"vrs_rank_gap":-0.051723}}}` |
| PGL Astana 2025 | 599 | 41 | 41 | -0.9942 | `{"calibration_scales":{"bo1":0.395913,"bo3":0.289727,"bo5":0.298474},"chosen_by":"validation_accuracy","fitted_through":"2025-05-04T14:00:00Z","lambda":10.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":18,"training_matches":599,"validation":[{"accuracy":0.694737,"lambda":1.0,"log_loss":0.583556,"matches":190},{"accuracy":0.694737,"lambda":3.0,"log_loss":0.583426,"matches":190},{"accuracy":0.705263,"lambda":10.0,"log_loss":0.584287,"matches":190},{"accuracy":0.705263,"lambda":30.0,"log_loss":0.585471,"matches":190},{"accuracy":0.705263,"lambda":100.0,"log_loss":0.586261,"matches":190},{"accuracy":0.705263,"lambda":1000.0,"log_loss":0.587081,"matches":190}],"validation_events":6,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.226837,"matches":565,"own_rating":0.026676,"per_map_rating":0.086,"rating":0.799566,"vrs_rank_gap":-0.039915}}}` |
| IEM Dallas 2025 | 640 | 29 | 29 | -0.0174 | `{"calibration_scales":{"bo1":0.395913,"bo3":0.258219,"bo5":0.267802},"chosen_by":"validation_accuracy","fitted_through":"2025-05-18T11:55:00Z","lambda":10.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":19,"training_matches":640,"validation":[{"accuracy":0.689119,"lambda":1.0,"log_loss":0.614507,"matches":193},{"accuracy":0.689119,"lambda":3.0,"log_loss":0.613494,"matches":193},{"accuracy":0.699482,"lambda":10.0,"log_loss":0.613549,"matches":193},{"accuracy":0.699482,"lambda":30.0,"log_loss":0.614776,"matches":193},{"accuracy":0.699482,"lambda":100.0,"log_loss":0.615959,"matches":193},{"accuracy":0.699482,"lambda":1000.0,"log_loss":0.616513,"matches":193}],"validation_events":6,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.241928,"matches":606,"own_rating":0.029855,"per_map_rating":0.118086,"rating":0.721131,"vrs_rank_gap":-0.00577}}}` |
| BLAST.tv Major Austin 2025 | 669 | 106 | 106 | -0.2365 | `{"calibration_scales":{"bo1":0.395913,"bo3":0.269482,"bo5":0.278819},"chosen_by":"validation_accuracy","fitted_through":"2025-05-25T19:25:00Z","lambda":1000.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":20,"training_matches":669,"validation":[{"accuracy":0.686813,"lambda":1.0,"log_loss":0.608319,"matches":182},{"accuracy":0.686813,"lambda":3.0,"log_loss":0.610024,"matches":182},{"accuracy":0.686813,"lambda":10.0,"log_loss":0.612703,"matches":182},{"accuracy":0.686813,"lambda":30.0,"log_loss":0.615169,"matches":182},{"accuracy":0.686813,"lambda":100.0,"log_loss":0.616959,"matches":182},{"accuracy":0.692308,"lambda":1000.0,"log_loss":0.61775,"matches":182}],"validation_events":6,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.016454,"matches":635,"own_rating":0.018821,"per_map_rating":0.00373,"rating":0.997896,"vrs_rank_gap":0.006092}}}` |
| FISSURE PLAYGROUND Season 1 2025 | 775 | 27 | 27 | -0.6598 | `{"calibration_scales":{"bo1":0.247395,"bo3":0.268806,"bo5":0.27203},"chosen_by":"validation_accuracy","fitted_through":"2025-06-22T19:15:00Z","lambda":1.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":21,"training_matches":775,"validation":[{"accuracy":0.659722,"lambda":1.0,"log_loss":0.634402,"matches":288},{"accuracy":0.659722,"lambda":3.0,"log_loss":0.63557,"matches":288},{"accuracy":0.659722,"lambda":10.0,"log_loss":0.637434,"matches":288},{"accuracy":0.65625,"lambda":30.0,"log_loss":0.639115,"matches":288},{"accuracy":0.659722,"lambda":100.0,"log_loss":0.640222,"matches":288},{"accuracy":0.659722,"lambda":1000.0,"log_loss":0.640143,"matches":288}],"validation_events":7,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.252055,"matches":741,"own_rating":0.005664,"per_map_rating":0.552347,"rating":0.473367,"vrs_rank_gap":0.115859}}}` |
| IEM Cologne 2025 | 802 | 49 | 49 | +0.0191 | `{"calibration_scales":{"bo1":0.247395,"bo3":0.258638,"bo5":0.261852},"chosen_by":"validation_accuracy","fitted_through":"2025-07-20T12:45:00Z","lambda":1.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":22,"training_matches":802,"validation":[{"accuracy":0.63986,"lambda":1.0,"log_loss":0.661991,"matches":286},{"accuracy":0.63986,"lambda":3.0,"log_loss":0.662594,"matches":286},{"accuracy":0.632867,"lambda":10.0,"log_loss":0.664965,"matches":286},{"accuracy":0.632867,"lambda":30.0,"log_loss":0.667735,"matches":286},{"accuracy":0.632867,"lambda":100.0,"log_loss":0.669633,"matches":286},{"accuracy":0.632867,"lambda":1000.0,"log_loss":0.66869,"matches":286}],"validation_events":7,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.240847,"matches":768,"own_rating":0.012661,"per_map_rating":0.509716,"rating":0.427477,"vrs_rank_gap":0.140911}}}` |
| BLAST Bounty Fall 2025 | 851 | 30 | 30 | +0.0341 | `{"calibration_scales":{"bo1":0.247395,"bo3":0.270232,"bo5":0.272275},"chosen_by":"validation_accuracy","fitted_through":"2025-08-03T14:50:00Z","lambda":1.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":23,"training_matches":851,"validation":[{"accuracy":0.64966,"lambda":1.0,"log_loss":0.62435,"matches":294},{"accuracy":0.64966,"lambda":3.0,"log_loss":0.625949,"matches":294},{"accuracy":0.646259,"lambda":10.0,"log_loss":0.628089,"matches":294},{"accuracy":0.64966,"lambda":30.0,"log_loss":0.629935,"matches":294},{"accuracy":0.64966,"lambda":100.0,"log_loss":0.63136,"matches":294},{"accuracy":0.64966,"lambda":1000.0,"log_loss":0.632154,"matches":294}],"validation_events":7,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.222141,"matches":817,"own_rating":-0.001716,"per_map_rating":0.804655,"rating":0.401436,"vrs_rank_gap":0.166062}}}` |
| Esports World Cup 2025 | 882 | 16 | 16 | -0.3747 | `{"calibration_scales":{"bo1":0.247395,"bo3":0.281385,"bo5":0.282961},"chosen_by":"validation_accuracy","fitted_through":"2025-08-17T13:45:00Z","lambda":1.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":24,"training_matches":882,"validation":[{"accuracy":0.669753,"lambda":1.0,"log_loss":0.608837,"matches":324},{"accuracy":0.669753,"lambda":3.0,"log_loss":0.610291,"matches":324},{"accuracy":0.666667,"lambda":10.0,"log_loss":0.611905,"matches":324},{"accuracy":0.669753,"lambda":30.0,"log_loss":0.613088,"matches":324},{"accuracy":0.669753,"lambda":100.0,"log_loss":0.614078,"matches":324},{"accuracy":0.669753,"lambda":1000.0,"log_loss":0.615148,"matches":324}],"validation_events":8,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.196632,"matches":847,"own_rating":0.000466,"per_map_rating":0.893472,"rating":0.439214,"vrs_rank_gap":0.158346}}}` |
| BLAST Open London Fall 2025 | 898 | 29 | 29 | -0.4712 | `{"calibration_scales":{"bo1":0.247395,"bo3":0.277258,"bo5":0.279645},"chosen_by":"validation_accuracy","fitted_through":"2025-08-24T10:10:00Z","lambda":1.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":25,"training_matches":898,"validation":[{"accuracy":0.665595,"lambda":1.0,"log_loss":0.623658,"matches":311},{"accuracy":0.659164,"lambda":3.0,"log_loss":0.624889,"matches":311},{"accuracy":0.655949,"lambda":10.0,"log_loss":0.626302,"matches":311},{"accuracy":0.659164,"lambda":30.0,"log_loss":0.627264,"matches":311},{"accuracy":0.655949,"lambda":100.0,"log_loss":0.627917,"matches":311},{"accuracy":0.655949,"lambda":1000.0,"log_loss":0.628033,"matches":311}],"validation_events":8,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.179458,"matches":863,"own_rating":0.004887,"per_map_rating":0.698554,"rating":0.481896,"vrs_rank_gap":0.148642}}}` |
| FISSURE PLAYGROUND Season 2 2025 | 927 | 40 | 40 | -0.2687 | `{"calibration_scales":{"bo1":0.247395,"bo3":0.276219,"bo5":0.275806},"chosen_by":"validation_accuracy","fitted_through":"2025-09-07T14:20:00Z","lambda":1.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":26,"training_matches":927,"validation":[{"accuracy":0.651376,"lambda":1.0,"log_loss":0.632222,"matches":327},{"accuracy":0.651376,"lambda":3.0,"log_loss":0.633263,"matches":327},{"accuracy":0.64526,"lambda":10.0,"log_loss":0.634505,"matches":327},{"accuracy":0.64526,"lambda":30.0,"log_loss":0.635323,"matches":327},{"accuracy":0.648318,"lambda":100.0,"log_loss":0.635838,"matches":327},{"accuracy":0.64526,"lambda":1000.0,"log_loss":0.635398,"matches":327}],"validation_events":8,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.146741,"matches":892,"own_rating":0.007089,"per_map_rating":0.622662,"rating":0.463366,"vrs_rank_gap":0.183982}}}` |
| ESL Pro League Season 22 2025 | 967 | 73 | 73 | -0.8195 | `{"calibration_scales":{"bo1":0.247395,"bo3":0.276262,"bo5":0.274371},"chosen_by":"validation_accuracy","fitted_through":"2025-09-21T14:45:00Z","lambda":1.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":27,"training_matches":967,"validation":[{"accuracy":0.651226,"lambda":1.0,"log_loss":0.632486,"matches":367},{"accuracy":0.651226,"lambda":3.0,"log_loss":0.633525,"matches":367},{"accuracy":0.645777,"lambda":10.0,"log_loss":0.635152,"matches":367},{"accuracy":0.645777,"lambda":30.0,"log_loss":0.636784,"matches":367},{"accuracy":0.648501,"lambda":100.0,"log_loss":0.638215,"matches":367},{"accuracy":0.643052,"lambda":1000.0,"log_loss":0.638664,"matches":367}],"validation_events":9,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.208821,"matches":932,"own_rating":0.019809,"per_map_rating":0.561522,"rating":0.435787,"vrs_rank_gap":0.143471}}}` |
| CS Asia Championships 2025 | 1040 | 30 | 30 | -0.2239 | `{"calibration_scales":{"bo1":0.247395,"bo3":0.24735,"bo5":0.24801},"chosen_by":"validation_accuracy","fitted_through":"2025-10-12T14:45:00Z","lambda":1.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":28,"training_matches":1040,"validation":[{"accuracy":0.649123,"lambda":1.0,"log_loss":0.639083,"matches":399},{"accuracy":0.64411,"lambda":3.0,"log_loss":0.640117,"matches":399},{"accuracy":0.641604,"lambda":10.0,"log_loss":0.642003,"matches":399},{"accuracy":0.641604,"lambda":30.0,"log_loss":0.643883,"matches":399},{"accuracy":0.641604,"lambda":100.0,"log_loss":0.645594,"matches":399},{"accuracy":0.639098,"lambda":1000.0,"log_loss":0.646849,"matches":399}],"validation_events":9,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.18994,"matches":1005,"own_rating":0.075435,"per_map_rating":0.507907,"rating":0.28436,"vrs_rank_gap":0.177279}}}` |
| Thunderpick World Championship 2025 | 1040 | 14 | 14 | -0.1820 | `{"calibration_scales":{"bo1":0.247395,"bo3":0.24735,"bo5":0.24801},"chosen_by":"validation_accuracy","fitted_through":"2025-10-12T14:45:00Z","lambda":1.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":28,"training_matches":1040,"validation":[{"accuracy":0.649123,"lambda":1.0,"log_loss":0.639083,"matches":399},{"accuracy":0.64411,"lambda":3.0,"log_loss":0.640117,"matches":399},{"accuracy":0.641604,"lambda":10.0,"log_loss":0.642003,"matches":399},{"accuracy":0.641604,"lambda":30.0,"log_loss":0.643883,"matches":399},{"accuracy":0.641604,"lambda":100.0,"log_loss":0.645594,"matches":399},{"accuracy":0.639098,"lambda":1000.0,"log_loss":0.646849,"matches":399}],"validation_events":9,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.18994,"matches":1005,"own_rating":0.075435,"per_map_rating":0.507907,"rating":0.28436,"vrs_rank_gap":0.177279}}}` |
| PGL Masters Bucharest 2025 | 1084 | 41 | 41 | -0.2637 | `{"calibration_scales":{"bo1":0.246468,"bo3":0.248916,"bo5":0.248788},"chosen_by":"validation_accuracy","fitted_through":"2025-10-19T15:45:00Z","lambda":1.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":30,"training_matches":1084,"validation":[{"accuracy":0.635266,"lambda":1.0,"log_loss":0.652163,"matches":414},{"accuracy":0.63285,"lambda":3.0,"log_loss":0.653018,"matches":414},{"accuracy":0.625604,"lambda":10.0,"log_loss":0.654977,"matches":414},{"accuracy":0.625604,"lambda":30.0,"log_loss":0.657204,"matches":414},{"accuracy":0.630435,"lambda":100.0,"log_loss":0.659286,"matches":414},{"accuracy":0.628019,"lambda":1000.0,"log_loss":0.660108,"matches":414}],"validation_events":10,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.188461,"matches":1049,"own_rating":0.07072,"per_map_rating":0.570267,"rating":0.279164,"vrs_rank_gap":0.175234}}}` |
| IEM Chengdu 2025 | 1125 | 30 | 30 | -0.0810 | `{"calibration_scales":{"bo1":0.246468,"bo3":0.24586,"bo5":0.246563},"chosen_by":"validation_accuracy","fitted_through":"2025-11-01T12:10:00Z","lambda":1.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":31,"training_matches":1125,"validation":[{"accuracy":0.636103,"lambda":1.0,"log_loss":0.648895,"matches":349},{"accuracy":0.627507,"lambda":3.0,"log_loss":0.649442,"matches":349},{"accuracy":0.618911,"lambda":10.0,"log_loss":0.651226,"matches":349},{"accuracy":0.616046,"lambda":30.0,"log_loss":0.653744,"matches":349},{"accuracy":0.621777,"lambda":100.0,"log_loss":0.656359,"matches":349},{"accuracy":0.616046,"lambda":1000.0,"log_loss":0.658155,"matches":349}],"validation_events":10,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.224524,"matches":1090,"own_rating":0.076926,"per_map_rating":0.506545,"rating":0.309008,"vrs_rank_gap":0.126291}}}` |
| BLAST Rivals Fall 2025 | 1155 | 13 | 13 | -0.1119 | `{"calibration_scales":{"bo1":0.246468,"bo3":0.249795,"bo5":0.250304},"chosen_by":"validation_accuracy","fitted_through":"2025-11-09T07:45:00Z","lambda":1.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":32,"training_matches":1155,"validation":[{"accuracy":0.653409,"lambda":1.0,"log_loss":0.631528,"matches":352},{"accuracy":0.647727,"lambda":3.0,"log_loss":0.63212,"matches":352},{"accuracy":0.642045,"lambda":10.0,"log_loss":0.634105,"matches":352},{"accuracy":0.636364,"lambda":30.0,"log_loss":0.63682,"matches":352},{"accuracy":0.642045,"lambda":100.0,"log_loss":0.639653,"matches":352},{"accuracy":0.639205,"lambda":1000.0,"log_loss":0.642775,"matches":352}],"validation_events":10,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.220278,"matches":1120,"own_rating":0.08224,"per_map_rating":0.373012,"rating":0.213618,"vrs_rank_gap":0.207194}}}` |
| StarLadder Major Budapest 2025 | 1168 | 106 | 106 | -0.1528 | `{"calibration_scales":{"bo1":0.246468,"bo3":0.251216,"bo5":0.251813},"chosen_by":"validation_accuracy","fitted_through":"2025-11-16T07:55:00Z","lambda":1.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":33,"training_matches":1168,"validation":[{"accuracy":0.654795,"lambda":1.0,"log_loss":0.629468,"matches":365},{"accuracy":0.649315,"lambda":3.0,"log_loss":0.629998,"matches":365},{"accuracy":0.643836,"lambda":10.0,"log_loss":0.632006,"matches":365},{"accuracy":0.638356,"lambda":30.0,"log_loss":0.634754,"matches":365},{"accuracy":0.643836,"lambda":100.0,"log_loss":0.637559,"matches":365},{"accuracy":0.643836,"lambda":1000.0,"log_loss":0.640733,"matches":365}],"validation_events":11,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.214332,"matches":1133,"own_rating":0.085276,"per_map_rating":0.301874,"rating":0.175546,"vrs_rank_gap":0.245012}}}` |
| BLAST Bounty Winter 2026 | 1274 | 31 | 31 | -0.0481 | `{"calibration_scales":{"bo1":0.237421,"bo3":0.253688,"bo5":0.321513},"chosen_by":"validation_accuracy","fitted_through":"2025-12-14T16:45:00Z","lambda":3.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":34,"training_matches":1274,"validation":[{"accuracy":0.63981,"lambda":1.0,"log_loss":0.642315,"matches":422},{"accuracy":0.63981,"lambda":3.0,"log_loss":0.642149,"matches":422},{"accuracy":0.627962,"lambda":10.0,"log_loss":0.643545,"matches":422},{"accuracy":0.627962,"lambda":30.0,"log_loss":0.646215,"matches":422},{"accuracy":0.630332,"lambda":100.0,"log_loss":0.649192,"matches":422},{"accuracy":0.630332,"lambda":1000.0,"log_loss":0.651798,"matches":422}],"validation_events":11,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.21654,"matches":1239,"own_rating":0.063649,"per_map_rating":0.498178,"rating":0.343264,"vrs_rank_gap":0.125772}}}` |
| IEM Krakow 2026 | 1305 | 50 | 50 | +0.0200 | `{"calibration_scales":{"bo1":0.237421,"bo3":0.255109,"bo5":0.257303},"chosen_by":"validation_accuracy","fitted_through":"2026-01-25T14:45:00Z","lambda":1.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":35,"training_matches":1305,"validation":[{"accuracy":0.624113,"lambda":1.0,"log_loss":0.65562,"matches":423},{"accuracy":0.617021,"lambda":3.0,"log_loss":0.65515,"matches":423},{"accuracy":0.607565,"lambda":10.0,"log_loss":0.656398,"matches":423},{"accuracy":0.607565,"lambda":30.0,"log_loss":0.659253,"matches":423},{"accuracy":0.609929,"lambda":100.0,"log_loss":0.662497,"matches":423},{"accuracy":0.609929,"lambda":1000.0,"log_loss":0.664748,"matches":423}],"validation_events":11,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":-0.694381,"market":0.610747,"matches":31,"own_rating":0.356235,"per_map_rating":-0.230134,"rating":-0.161342,"vrs_rank_gap":0.133747},"rating":{"hltv_rank_gap":0.218926,"matches":1239,"own_rating":0.051276,"per_map_rating":0.644554,"rating":0.277864,"vrs_rank_gap":0.153074}}}` |
| PGL Cluj-Napoca 2026 | 1355 | 41 | 41 | +0.0924 | `{"calibration_scales":{"bo1":0.237421,"bo3":0.254301,"bo5":0.255931},"chosen_by":"validation_accuracy","fitted_through":"2026-02-08T15:45:00Z","lambda":1.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":36,"training_matches":1355,"validation":[{"accuracy":0.623679,"lambda":1.0,"log_loss":0.651938,"matches":473},{"accuracy":0.617336,"lambda":3.0,"log_loss":0.651518,"matches":473},{"accuracy":0.608879,"lambda":10.0,"log_loss":0.652634,"matches":473},{"accuracy":0.608879,"lambda":30.0,"log_loss":0.655187,"matches":473},{"accuracy":0.610994,"lambda":100.0,"log_loss":0.658088,"matches":473},{"accuracy":0.610994,"lambda":1000.0,"log_loss":0.660101,"matches":473}],"validation_events":12,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":-0.436737,"market":0.642214,"matches":81,"own_rating":-0.018807,"per_map_rating":0.450226,"rating":-0.593979,"vrs_rank_gap":0.435232},"rating":{"hltv_rank_gap":0.218917,"matches":1239,"own_rating":0.051281,"per_map_rating":0.644645,"rating":0.278592,"vrs_rank_gap":0.15312}}}` |
| ESL Pro League Season 23 2026 | 1396 | 74 | 74 | -0.0012 | `{"calibration_scales":{"bo1":0.237421,"bo3":0.260187,"bo5":0.256053},"chosen_by":"validation_accuracy","fitted_through":"2026-02-22T15:45:00Z","lambda":1.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":37,"training_matches":1396,"validation":[{"accuracy":0.638554,"lambda":1.0,"log_loss":0.639923,"matches":498},{"accuracy":0.628514,"lambda":3.0,"log_loss":0.640456,"matches":498},{"accuracy":0.620482,"lambda":10.0,"log_loss":0.642332,"matches":498},{"accuracy":0.624498,"lambda":30.0,"log_loss":0.645108,"matches":498},{"accuracy":0.626506,"lambda":100.0,"log_loss":0.648003,"matches":498},{"accuracy":0.626506,"lambda":1000.0,"log_loss":0.65033,"matches":498}],"validation_events":12,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.205462,"market":0.5403,"matches":122,"own_rating":0.06782,"per_map_rating":0.530585,"rating":-0.335779,"vrs_rank_gap":0.289577},"rating":{"hltv_rank_gap":0.219064,"matches":1239,"own_rating":0.051196,"per_map_rating":0.644719,"rating":0.27386,"vrs_rank_gap":0.152496}}}` |
| BLAST Open Rotterdam Spring 2026 | 1470 | 29 | 29 | -0.0215 | `{"calibration_scales":{"bo1":0.237421,"bo3":0.257877,"bo5":0.253405},"chosen_by":"validation_accuracy","fitted_through":"2026-03-15T16:10:00Z","lambda":1.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":38,"training_matches":1470,"validation":[{"accuracy":0.629834,"lambda":1.0,"log_loss":0.642118,"matches":543},{"accuracy":0.626151,"lambda":3.0,"log_loss":0.642758,"matches":543},{"accuracy":0.620626,"lambda":10.0,"log_loss":0.644539,"matches":543},{"accuracy":0.620626,"lambda":30.0,"log_loss":0.64708,"matches":543},{"accuracy":0.626151,"lambda":100.0,"log_loss":0.649757,"matches":543},{"accuracy":0.620626,"lambda":1000.0,"log_loss":0.652164,"matches":543}],"validation_events":12,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":-0.072968,"market":0.582933,"matches":196,"own_rating":0.111122,"per_map_rating":0.160993,"rating":-0.444371,"vrs_rank_gap":0.335286},"rating":{"hltv_rank_gap":0.219029,"matches":1239,"own_rating":0.051216,"per_map_rating":0.64489,"rating":0.27587,"vrs_rank_gap":0.152639}}}` |
| PGL Bucharest 2026 | 1499 | 39 | 39 | +0.0144 | `{"calibration_scales":{"bo1":0.237421,"bo3":0.25847,"bo5":0.276963},"chosen_by":"validation_accuracy","fitted_through":"2026-03-29T10:15:00Z","lambda":1.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":39,"training_matches":1499,"validation":[{"accuracy":0.629371,"lambda":1.0,"log_loss":0.639696,"matches":572},{"accuracy":0.625874,"lambda":3.0,"log_loss":0.640303,"matches":572},{"accuracy":0.620629,"lambda":10.0,"log_loss":0.641994,"matches":572},{"accuracy":0.620629,"lambda":30.0,"log_loss":0.644406,"matches":572},{"accuracy":0.625874,"lambda":100.0,"log_loss":0.646947,"matches":572},{"accuracy":0.620629,"lambda":1000.0,"log_loss":0.649232,"matches":572}],"validation_events":13,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":-0.020038,"market":0.534718,"matches":225,"own_rating":0.150972,"per_map_rating":-0.101572,"rating":-0.420669,"vrs_rank_gap":0.347854},"rating":{"hltv_rank_gap":0.218845,"matches":1239,"own_rating":0.051326,"per_map_rating":0.643105,"rating":0.273865,"vrs_rank_gap":0.153472}}}` |
| IEM Rio 2026 | 1538 | 30 | 30 | +0.0369 | `{"calibration_scales":{"bo1":0.237421,"bo3":0.261177,"bo5":0.267862},"chosen_by":"validation_accuracy","fitted_through":"2026-04-11T13:45:00Z","lambda":1.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":40,"training_matches":1538,"validation":[{"accuracy":0.633975,"lambda":1.0,"log_loss":0.633219,"matches":571},{"accuracy":0.630473,"lambda":3.0,"log_loss":0.63386,"matches":571},{"accuracy":0.625219,"lambda":10.0,"log_loss":0.635454,"matches":571},{"accuracy":0.625219,"lambda":30.0,"log_loss":0.637626,"matches":571},{"accuracy":0.62697,"lambda":100.0,"log_loss":0.639936,"matches":571},{"accuracy":0.62697,"lambda":1000.0,"log_loss":0.642028,"matches":571}],"validation_events":13,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.042234,"market":0.623153,"matches":264,"own_rating":0.19924,"per_map_rating":-0.491263,"rating":-0.228063,"vrs_rank_gap":0.252349},"rating":{"hltv_rank_gap":0.218988,"matches":1239,"own_rating":0.051239,"per_map_rating":0.643847,"rating":0.272375,"vrs_rank_gap":0.152812}}}` |
| BLAST Rivals Spring 2026 | 1568 | 13 | 13 | -0.0096 | `{"calibration_scales":{"bo1":0.237421,"bo3":0.266287,"bo5":0.289673},"chosen_by":"validation_accuracy","fitted_through":"2026-04-19T16:45:00Z","lambda":10.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":41,"training_matches":1568,"validation":[{"accuracy":0.642045,"lambda":1.0,"log_loss":0.614745,"matches":528},{"accuracy":0.645833,"lambda":3.0,"log_loss":0.615142,"matches":528},{"accuracy":0.649621,"lambda":10.0,"log_loss":0.616079,"matches":528},{"accuracy":0.643939,"lambda":30.0,"log_loss":0.617274,"matches":528},{"accuracy":0.638258,"lambda":100.0,"log_loss":0.618389,"matches":528},{"accuracy":0.640152,"lambda":1000.0,"log_loss":0.619665,"matches":528}],"validation_events":13,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.031295,"market":0.724457,"matches":292,"own_rating":0.115443,"per_map_rating":-0.053927,"rating":0.117186,"vrs_rank_gap":0.048666},"rating":{"hltv_rank_gap":0.205313,"matches":1241,"own_rating":0.079064,"per_map_rating":0.30357,"rating":0.484371,"vrs_rank_gap":0.057046}}}` |
| PGL Astana 2026 | 1581 | 41 | 41 | +0.0059 | `{"calibration_scales":{"bo1":0.237421,"bo3":0.266822,"bo5":0.30361},"chosen_by":"validation_accuracy","fitted_through":"2026-05-03T17:15:00Z","lambda":10.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":42,"training_matches":1581,"validation":[{"accuracy":0.643253,"lambda":1.0,"log_loss":0.614195,"matches":541},{"accuracy":0.64695,"lambda":3.0,"log_loss":0.614583,"matches":541},{"accuracy":0.650647,"lambda":10.0,"log_loss":0.615497,"matches":541},{"accuracy":0.645102,"lambda":30.0,"log_loss":0.616663,"matches":541},{"accuracy":0.639556,"lambda":100.0,"log_loss":0.617751,"matches":541},{"accuracy":0.641405,"lambda":1000.0,"log_loss":0.618996,"matches":541}],"validation_events":14,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":-0.006444,"market":0.716374,"matches":305,"own_rating":0.106428,"per_map_rating":-0.031926,"rating":0.118701,"vrs_rank_gap":0.062611},"rating":{"hltv_rank_gap":0.205156,"matches":1241,"own_rating":0.079051,"per_map_rating":0.302867,"rating":0.482686,"vrs_rank_gap":0.057445}}}` |
| IEM Atlanta 2026 | 1581 | 30 | 30 | -0.0133 | `{"calibration_scales":{"bo1":0.237421,"bo3":0.266822,"bo5":0.30361},"chosen_by":"validation_accuracy","fitted_through":"2026-05-03T17:15:00Z","lambda":10.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":42,"training_matches":1581,"validation":[{"accuracy":0.643253,"lambda":1.0,"log_loss":0.614195,"matches":541},{"accuracy":0.64695,"lambda":3.0,"log_loss":0.614583,"matches":541},{"accuracy":0.650647,"lambda":10.0,"log_loss":0.615497,"matches":541},{"accuracy":0.645102,"lambda":30.0,"log_loss":0.616663,"matches":541},{"accuracy":0.639556,"lambda":100.0,"log_loss":0.617751,"matches":541},{"accuracy":0.641405,"lambda":1000.0,"log_loss":0.618996,"matches":541}],"validation_events":14,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":-0.006444,"market":0.716374,"matches":305,"own_rating":0.106428,"per_map_rating":-0.031926,"rating":0.118701,"vrs_rank_gap":0.062611},"rating":{"hltv_rank_gap":0.205156,"matches":1241,"own_rating":0.079051,"per_map_rating":0.302867,"rating":0.482686,"vrs_rank_gap":0.057445}}}` |
| CS Asia Championships 2026 | 1652 | 30 | 30 | -0.1868 | `{"calibration_scales":{"bo1":0.237421,"bo3":0.269944,"bo5":0.321068},"chosen_by":"validation_accuracy","fitted_through":"2026-05-17T18:10:00Z","lambda":10.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":44,"training_matches":1652,"validation":[{"accuracy":0.649648,"lambda":1.0,"log_loss":0.609056,"matches":568},{"accuracy":0.651408,"lambda":3.0,"log_loss":0.609322,"matches":568},{"accuracy":0.65493,"lambda":10.0,"log_loss":0.610006,"matches":568},{"accuracy":0.65493,"lambda":30.0,"log_loss":0.61095,"matches":568},{"accuracy":0.651408,"lambda":100.0,"log_loss":0.611874,"matches":568},{"accuracy":0.651408,"lambda":1000.0,"log_loss":0.61289,"matches":568}],"validation_events":14,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":-0.022699,"market":0.777956,"matches":376,"own_rating":0.063639,"per_map_rating":-0.078831,"rating":0.124,"vrs_rank_gap":0.07465},"rating":{"hltv_rank_gap":0.205112,"matches":1241,"own_rating":0.079031,"per_map_rating":0.301941,"rating":0.475944,"vrs_rank_gap":0.058248}}}` |
| IEM Cologne Major 2026 | 1682 | 106 | 106 | +0.0075 | `{"calibration_scales":{"bo1":0.248029,"bo3":0.271524,"bo5":0.281227},"chosen_by":"validation_accuracy","fitted_through":"2026-05-24T06:00:00Z","lambda":10.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":45,"training_matches":1682,"validation":[{"accuracy":0.650502,"lambda":1.0,"log_loss":0.608575,"matches":598},{"accuracy":0.652174,"lambda":3.0,"log_loss":0.608777,"matches":598},{"accuracy":0.655518,"lambda":10.0,"log_loss":0.609439,"matches":598},{"accuracy":0.655518,"lambda":30.0,"log_loss":0.610453,"matches":598},{"accuracy":0.652174,"lambda":100.0,"log_loss":0.611514,"matches":598},{"accuracy":0.652174,"lambda":1000.0,"log_loss":0.61287,"matches":598}],"validation_events":15,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":-0.022523,"market":0.777738,"matches":376,"own_rating":0.063658,"per_map_rating":-0.078647,"rating":0.124881,"vrs_rank_gap":0.074085},"rating":{"hltv_rank_gap":0.210787,"matches":1271,"own_rating":0.085771,"per_map_rating":0.278675,"rating":0.458485,"vrs_rank_gap":0.061836}}}` |
| XSE Pro League 2026 | 1788 | 37 | 37 | -0.0464 | `{"calibration_scales":{"bo1":0.272686,"bo3":0.274544,"bo5":0.297334},"chosen_by":"validation_accuracy","fitted_through":"2026-06-21T14:45:00Z","lambda":10.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":46,"training_matches":1788,"validation":[{"accuracy":0.668175,"lambda":1.0,"log_loss":0.602177,"matches":663},{"accuracy":0.668175,"lambda":3.0,"log_loss":0.602496,"matches":663},{"accuracy":0.671192,"lambda":10.0,"log_loss":0.60324,"matches":663},{"accuracy":0.669683,"lambda":30.0,"log_loss":0.60411,"matches":663},{"accuracy":0.669683,"lambda":100.0,"log_loss":0.604828,"matches":663},{"accuracy":0.669683,"lambda":1000.0,"log_loss":0.605812,"matches":663}],"validation_events":15,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":-0.030428,"market":0.879508,"matches":480,"own_rating":0.081103,"per_map_rating":-0.135181,"rating":0.136356,"vrs_rank_gap":0.0357},"rating":{"hltv_rank_gap":0.211604,"matches":1273,"own_rating":0.083988,"per_map_rating":0.276529,"rating":0.446297,"vrs_rank_gap":0.06614}}}` |
| BLAST Bounty Summer 2026 | 1828 | 30 | 30 | -0.0515 | `{"calibration_scales":{"bo1":0.270938,"bo3":0.275425,"bo5":0.311595},"chosen_by":"validation_accuracy","fitted_through":"2026-07-12T08:05:00Z","lambda":3.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":47,"training_matches":1828,"validation":[{"accuracy":0.664179,"lambda":1.0,"log_loss":0.606875,"matches":670},{"accuracy":0.665672,"lambda":3.0,"log_loss":0.606981,"matches":670},{"accuracy":0.665672,"lambda":10.0,"log_loss":0.60725,"matches":670},{"accuracy":0.665672,"lambda":30.0,"log_loss":0.607706,"matches":670},{"accuracy":0.664179,"lambda":100.0,"log_loss":0.608307,"matches":670},{"accuracy":0.664179,"lambda":1000.0,"log_loss":0.609381,"matches":670}],"validation_events":15,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"per_map_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":-0.036163,"market":0.919862,"matches":480,"own_rating":0.107358,"per_map_rating":-0.311825,"rating":0.026339,"vrs_rank_gap":0.082299},"rating":{"hltv_rank_gap":0.230857,"matches":1310,"own_rating":0.074897,"per_map_rating":0.485101,"rating":0.300334,"vrs_rank_gap":0.119328}}}` |
| Esports World Cup 2026 | 1859 | 56 | 56 | +0.0055 | `{"calibration_scales":{"bo1":0.270938,"bo3":0.272057,"bo5":0.280486},"chosen_by":"validation_accuracy","fitted_through":"2026-08-02T10:15:00Z","lambda":3.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":48,"training_matches":1859,"validation":[{"accuracy":0.662857,"lambda":1.0,"log_loss":0.610373,"matches":700},{"accuracy":0.664286,"lambda":3.0,"log_loss":0.610474,"matches":700},{"accuracy":0.664286,"lambda":10.0,"log_loss":0.610731,"matches":700},{"accuracy":0.664286,"lambda":30.0,"log_loss":0.611168,"matches":700},{"accuracy":0.662857,"lambda":100.0,"log_loss":0.611743,"matches":700},{"accuracy":0.662857,"lambda":1000.0,"log_loss":0.612771,"matches":700}],"validation_events":16,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.053109,"matches":1,"own_rating":-0.192072,"per_map_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.054392,"market":0.834422,"matches":509,"own_rating":0.09607,"per_map_rating":-0.29201,"rating":-0.016561,"vrs_rank_gap":0.068422},"rating":{"hltv_rank_gap":0.230951,"matches":1310,"own_rating":0.074765,"per_map_rating":0.487335,"rating":0.307116,"vrs_rank_gap":0.117386}}}` |
| BLAST Open Porto Fall 2026 | 1915 | 29 | 29 | +0.0091 | `{"calibration_scales":{"bo1":0.282125,"bo3":0.275584,"bo5":0.297826},"chosen_by":"validation_accuracy","fitted_through":"2026-08-23T12:00:00Z","lambda":3.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":49,"training_matches":1915,"validation":[{"accuracy":0.66891,"lambda":1.0,"log_loss":0.610095,"matches":743},{"accuracy":0.66891,"lambda":3.0,"log_loss":0.610083,"matches":743},{"accuracy":0.66891,"lambda":10.0,"log_loss":0.610113,"matches":743},{"accuracy":0.66891,"lambda":30.0,"log_loss":0.610335,"matches":743},{"accuracy":0.667564,"lambda":100.0,"log_loss":0.610794,"matches":743},{"accuracy":0.666218,"lambda":1000.0,"log_loss":0.611719,"matches":743}],"validation_events":16,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.053109,"matches":1,"own_rating":-0.192072,"per_map_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.062903,"market":0.786292,"matches":565,"own_rating":0.122794,"per_map_rating":-0.334457,"rating":0.087416,"vrs_rank_gap":0.030832},"rating":{"hltv_rank_gap":0.230813,"matches":1310,"own_rating":0.075025,"per_map_rating":0.486579,"rating":0.29769,"vrs_rank_gap":0.119913}}}` |
| FISSURE PLAYGROUND Season 3 2026 | 1944 | 29 | 29 | -0.0374 | `{"calibration_scales":{"bo1":0.282125,"bo3":0.278828,"bo5":0.307948},"chosen_by":"validation_accuracy","fitted_through":"2026-09-06T13:45:00Z","lambda":1000.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":50,"training_matches":1944,"validation":[{"accuracy":0.677177,"lambda":1.0,"log_loss":0.59936,"matches":666},{"accuracy":0.678679,"lambda":3.0,"log_loss":0.599357,"matches":666},{"accuracy":0.678679,"lambda":10.0,"log_loss":0.599428,"matches":666},{"accuracy":0.678679,"lambda":30.0,"log_loss":0.599627,"matches":666},{"accuracy":0.678679,"lambda":100.0,"log_loss":0.599963,"matches":666},{"accuracy":0.68018,"lambda":1000.0,"log_loss":0.600687,"matches":666}],"validation_events":16,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.000237,"matches":1,"own_rating":-0.000858,"per_map_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.009309,"market":0.805027,"matches":594,"own_rating":0.014523,"per_map_rating":0.000652,"rating":0.202579,"vrs_rank_gap":0.00813},"rating":{"hltv_rank_gap":0.017121,"matches":1310,"own_rating":0.031519,"per_map_rating":0.008323,"rating":0.982316,"vrs_rank_gap":-0.003047}}}` |
| StarLadder StarSeries Fall 2026 | 1973 | 14 | 14 | -0.0387 | `{"calibration_scales":{"bo1":0.282125,"bo3":0.277275,"bo5":0.313009},"chosen_by":"validation_accuracy","fitted_through":"2026-09-13T08:55:00Z","lambda":1000.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":51,"training_matches":1973,"validation":[{"accuracy":0.671942,"lambda":1.0,"log_loss":0.604272,"matches":695},{"accuracy":0.673381,"lambda":3.0,"log_loss":0.604269,"matches":695},{"accuracy":0.673381,"lambda":10.0,"log_loss":0.604337,"matches":695},{"accuracy":0.673381,"lambda":30.0,"log_loss":0.604528,"matches":695},{"accuracy":0.673381,"lambda":100.0,"log_loss":0.60485,"matches":695},{"accuracy":0.67482,"lambda":1000.0,"log_loss":0.605544,"matches":695}],"validation_events":17,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.000237,"matches":1,"own_rating":-0.000858,"per_map_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.005922,"market":0.802174,"matches":623,"own_rating":0.014138,"per_map_rating":0.00029,"rating":0.201654,"vrs_rank_gap":0.00716},"rating":{"hltv_rank_gap":0.01743,"matches":1310,"own_rating":0.031843,"per_map_rating":0.008366,"rating":0.982759,"vrs_rank_gap":-0.00262}}}` |
| ESL Pro League Season 24 2026 | 1987 | 41 | 41 | -0.0355 | `{"calibration_scales":{"bo1":0.282125,"bo3":0.276451,"bo5":0.328123},"chosen_by":"validation_accuracy","fitted_through":"2026-09-20T14:45:00Z","lambda":1000.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":52,"training_matches":1987,"validation":[{"accuracy":0.625369,"lambda":1.0,"log_loss":0.650355,"matches":678},{"accuracy":0.671091,"lambda":3.0,"log_loss":0.628418,"matches":678},{"accuracy":0.672566,"lambda":10.0,"log_loss":0.614258,"matches":678},{"accuracy":0.674041,"lambda":30.0,"log_loss":0.608777,"matches":678},{"accuracy":0.676991,"lambda":100.0,"log_loss":0.606256,"matches":678},{"accuracy":0.678466,"lambda":1000.0,"log_loss":0.605549,"matches":678}],"validation_events":17,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.000237,"matches":1,"own_rating":-0.000858,"per_map_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.005574,"market":0.802192,"matches":637,"own_rating":0.012422,"per_map_rating":0.00023,"rating":0.201382,"vrs_rank_gap":0.0061},"rating":{"hltv_rank_gap":0.0175,"matches":1310,"own_rating":0.031898,"per_map_rating":0.008352,"rating":0.982894,"vrs_rank_gap":-0.002475}}}` |
| Thunderpick World Championship 2026 | 2028 | 14 | 14 | +0.0008 | `{"calibration_scales":{"bo1":0.282125,"bo3":0.274277,"bo5":0.332202},"chosen_by":"validation_accuracy","fitted_through":"2026-10-11T14:45:00Z","lambda":1000.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":53,"training_matches":2028,"validation":[{"accuracy":0.659193,"lambda":1.0,"log_loss":0.630822,"matches":669},{"accuracy":0.678625,"lambda":3.0,"log_loss":0.615201,"matches":669},{"accuracy":0.681614,"lambda":10.0,"log_loss":0.606259,"matches":669},{"accuracy":0.681614,"lambda":30.0,"log_loss":0.603347,"matches":669},{"accuracy":0.681614,"lambda":100.0,"log_loss":0.602448,"matches":669},{"accuracy":0.683109,"lambda":1000.0,"log_loss":0.602511,"matches":669}],"validation_events":17,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.000237,"matches":1,"own_rating":-0.000858,"per_map_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.003319,"market":0.803753,"matches":678,"own_rating":0.012787,"per_map_rating":0.001378,"rating":0.200453,"vrs_rank_gap":0.003463},"rating":{"hltv_rank_gap":0.017962,"matches":1310,"own_rating":0.032386,"per_map_rating":0.008423,"rating":0.983521,"vrs_rank_gap":-0.00185}}}` |
| PGL Masters Bucharest 2026 | 2042 | 41 | 41 | -0.0811 | `{"calibration_scales":{"bo1":0.282125,"bo3":0.274449,"bo5":0.345565},"chosen_by":"validation_accuracy","fitted_through":"2026-10-18T15:45:00Z","lambda":1000.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":54,"training_matches":2042,"validation":[{"accuracy":0.657394,"lambda":1.0,"log_loss":0.631721,"matches":683},{"accuracy":0.677892,"lambda":3.0,"log_loss":0.615914,"matches":683},{"accuracy":0.682284,"lambda":10.0,"log_loss":0.606842,"matches":683},{"accuracy":0.682284,"lambda":30.0,"log_loss":0.603865,"matches":683},{"accuracy":0.68082,"lambda":100.0,"log_loss":0.602931,"matches":683},{"accuracy":0.682284,"lambda":1000.0,"log_loss":0.602963,"matches":683}],"validation_events":18,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.000237,"matches":1,"own_rating":-0.000858,"per_map_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.003808,"market":0.803667,"matches":692,"own_rating":0.013071,"per_map_rating":0.001714,"rating":0.200687,"vrs_rank_gap":0.003472},"rating":{"hltv_rank_gap":0.017826,"matches":1310,"own_rating":0.032222,"per_map_rating":0.008379,"rating":0.983352,"vrs_rank_gap":-0.001985}}}` |
| IEM Beijing 2026 | 2083 | 30 | 30 | -0.0517 | `{"calibration_scales":{"bo1":0.282125,"bo3":0.267663,"bo5":0.359388},"chosen_by":"validation_accuracy","fitted_through":"2026-10-31T10:45:00Z","lambda":100.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":55,"training_matches":2083,"validation":[{"accuracy":0.672035,"lambda":1.0,"log_loss":0.623411,"matches":683},{"accuracy":0.670571,"lambda":3.0,"log_loss":0.620853,"matches":683},{"accuracy":0.673499,"lambda":10.0,"log_loss":0.617023,"matches":683},{"accuracy":0.673499,"lambda":30.0,"log_loss":0.613988,"matches":683},{"accuracy":0.674963,"lambda":100.0,"log_loss":0.611747,"matches":683},{"accuracy":0.673499,"lambda":1000.0,"log_loss":0.610892,"matches":683}],"validation_events":18,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.002337,"matches":1,"own_rating":-0.008452,"per_map_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.021497,"market":0.80175,"matches":733,"own_rating":0.036474,"per_map_rating":-0.001764,"rating":0.182431,"vrs_rank_gap":-0.031795},"rating":{"hltv_rank_gap":0.092387,"matches":1310,"own_rating":0.085686,"per_map_rating":0.053781,"rating":0.852283,"vrs_rank_gap":-0.031173}}}` |
| BLAST Rivals Fall 2026 | 2113 | 13 | 13 | +0.0091 | `{"calibration_scales":{"bo1":0.282125,"bo3":0.264677,"bo5":0.372648},"chosen_by":"validation_accuracy","fitted_through":"2026-11-08T07:15:00Z","lambda":1000.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":56,"training_matches":2113,"validation":[{"accuracy":0.666667,"lambda":1.0,"log_loss":0.608527,"matches":639},{"accuracy":0.676056,"lambda":3.0,"log_loss":0.60613,"matches":639},{"accuracy":0.676056,"lambda":10.0,"log_loss":0.604586,"matches":639},{"accuracy":0.677621,"lambda":30.0,"log_loss":0.604131,"matches":639},{"accuracy":0.677621,"lambda":100.0,"log_loss":0.604333,"matches":639},{"accuracy":0.679186,"lambda":1000.0,"log_loss":0.605158,"matches":639}],"validation_events":18,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.000237,"matches":1,"own_rating":-0.000858,"per_map_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.002825,"market":0.802511,"matches":763,"own_rating":0.010482,"per_map_rating":0.001032,"rating":0.197032,"vrs_rank_gap":-0.005297},"rating":{"hltv_rank_gap":0.019884,"matches":1310,"own_rating":0.034382,"per_map_rating":0.008681,"rating":0.985963,"vrs_rank_gap":0.000826}}}` |

## `stacker_no_map_elo`: promote (added after list fixed)

ablation of `stacker` with identical fitting and inputs minus `per_map_rating`: `stacker` vs `stacker_no_map_elo` isolates the map-level Elo input on the same Matches.

Verdict: the interval of the log-loss improvement excludes zero and the Matches with a market line are not worse (added after spec #33's candidate list was fixed).

| On the 2087 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.7943 | 0.2398 | 65.8% |
| stacker_no_map_elo | 0.6317 | 0.2178 | 65.6% |

| Event | Fitted on | Computed on | Paired | Δ log-loss | Fitted parameters |
|---|---|---|---|---|---|
| BLAST Premier Groups Fall 2024 | 0 | 0 | 0 | — | `{"calibration_scales":{"bo1":null,"bo3":null,"bo5":null},"chosen_by":"default","fitted_through":null,"lambda":10.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":0,"training_matches":0,"validation":[],"validation_events":0,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.0,"matches":0,"own_rating":0.0,"rating":1.0,"vrs_rank_gap":0.0}}}` |
| IEM Cologne 2024 | 26 | 49 | 49 | +0.0000 | `{"calibration_scales":{"bo1":null,"bo3":null,"bo5":null},"chosen_by":"default","fitted_through":"2024-08-04T14:20:00Z","lambda":10.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":1,"training_matches":26,"validation":[],"validation_events":0,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.0,"matches":0,"own_rating":0.0,"rating":1.0,"vrs_rank_gap":0.0}}}` |
| BLAST Premier Showdown Fall 2024 | 75 | 14 | 14 | -0.2580 | `{"calibration_scales":{"bo1":0.253898,"bo3":0.259052,"bo5":0.253898},"chosen_by":"default","fitted_through":"2024-08-18T14:45:00Z","lambda":10.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":2,"training_matches":75,"validation":[],"validation_events":0,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.106012,"matches":49,"own_rating":0.210049,"rating":0.979824,"vrs_rank_gap":-0.080771}}}` |
| BetBoom Dacha Belgrade season 2 2024 | 75 | 26 | 26 | -0.0179 | `{"calibration_scales":{"bo1":0.253898,"bo3":0.259052,"bo5":0.253898},"chosen_by":"default","fitted_through":"2024-08-18T14:45:00Z","lambda":10.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":2,"training_matches":75,"validation":[],"validation_events":0,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.106012,"matches":49,"own_rating":0.210049,"rating":0.979824,"vrs_rank_gap":-0.080771}}}` |
| ESL Pro League Season 20 2024 | 115 | 76 | 76 | -0.2444 | `{"calibration_scales":{"bo1":0.324383,"bo3":0.320437,"bo5":0.324383},"chosen_by":"default","fitted_through":"2024-09-01T10:45:00Z","lambda":10.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":4,"training_matches":115,"validation":[{"accuracy":0.730769,"lambda":1.0,"log_loss":0.593215,"matches":26},{"accuracy":0.730769,"lambda":3.0,"log_loss":0.584885,"matches":26},{"accuracy":0.769231,"lambda":10.0,"log_loss":0.581854,"matches":26},{"accuracy":0.730769,"lambda":30.0,"log_loss":0.588309,"matches":26},{"accuracy":0.769231,"lambda":100.0,"log_loss":0.598769,"matches":26},{"accuracy":0.769231,"lambda":1000.0,"log_loss":0.607513,"matches":26}],"validation_events":0,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.034401,"matches":89,"own_rating":0.210271,"rating":0.994419,"vrs_rank_gap":-0.062974}}}` |
| BLAST Premier Final Fall 2024 | 194 | 13 | 13 | -0.0468 | `{"calibration_scales":{"bo1":0.307282,"bo3":0.303027,"bo5":0.307282},"chosen_by":"validation_accuracy","fitted_through":"2024-09-22T14:45:00Z","lambda":30.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":5,"training_matches":194,"validation":[{"accuracy":0.644737,"lambda":1.0,"log_loss":0.632683,"matches":76},{"accuracy":0.644737,"lambda":3.0,"log_loss":0.627221,"matches":76},{"accuracy":0.644737,"lambda":10.0,"log_loss":0.618883,"matches":76},{"accuracy":0.657895,"lambda":30.0,"log_loss":0.608194,"matches":76},{"accuracy":0.644737,"lambda":100.0,"log_loss":0.598024,"matches":76},{"accuracy":0.644737,"lambda":1000.0,"log_loss":0.591685,"matches":76}],"validation_events":1,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.032618,"matches":165,"own_rating":0.043973,"rating":0.977716,"vrs_rank_gap":-0.00509}}}` |
| IEM Rio 2024 | 207 | 29 | 29 | -0.2659 | `{"calibration_scales":{"bo1":0.319241,"bo3":0.318395,"bo5":0.319241},"chosen_by":"validation_accuracy","fitted_through":"2024-09-29T13:45:00Z","lambda":30.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":6,"training_matches":207,"validation":[{"accuracy":0.651685,"lambda":1.0,"log_loss":0.611638,"matches":89},{"accuracy":0.651685,"lambda":3.0,"log_loss":0.608033,"matches":89},{"accuracy":0.651685,"lambda":10.0,"log_loss":0.602417,"matches":89},{"accuracy":0.662921,"lambda":30.0,"log_loss":0.595059,"matches":89},{"accuracy":0.651685,"lambda":100.0,"log_loss":0.588712,"matches":89},{"accuracy":0.651685,"lambda":1000.0,"log_loss":0.585875,"matches":89}],"validation_events":2,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.047254,"matches":178,"own_rating":0.067143,"rating":0.960654,"vrs_rank_gap":-0.020229}}}` |
| Thunderpick World Championship 2024 | 236 | 28 | 28 | -0.3229 | `{"calibration_scales":{"bo1":0.304606,"bo3":0.305638,"bo5":0.304606},"chosen_by":"validation_accuracy","fitted_through":"2024-10-13T16:45:00Z","lambda":1000.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":7,"training_matches":236,"validation":[{"accuracy":0.619048,"lambda":1.0,"log_loss":0.645317,"matches":42},{"accuracy":0.619048,"lambda":3.0,"log_loss":0.644383,"matches":42},{"accuracy":0.619048,"lambda":10.0,"log_loss":0.644039,"matches":42},{"accuracy":0.619048,"lambda":30.0,"log_loss":0.643248,"matches":42},{"accuracy":0.619048,"lambda":100.0,"log_loss":0.641355,"matches":42},{"accuracy":0.619048,"lambda":1000.0,"log_loss":0.638693,"matches":42}],"validation_events":2,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.002915,"matches":207,"own_rating":0.00079,"rating":0.999813,"vrs_rank_gap":0.000247}}}` |
| BLAST Premier World Final 2024 | 236 | 13 | 13 | -0.4457 | `{"calibration_scales":{"bo1":0.304606,"bo3":0.305638,"bo5":0.304606},"chosen_by":"validation_accuracy","fitted_through":"2024-10-13T16:45:00Z","lambda":1000.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":7,"training_matches":236,"validation":[{"accuracy":0.619048,"lambda":1.0,"log_loss":0.645317,"matches":42},{"accuracy":0.619048,"lambda":3.0,"log_loss":0.644383,"matches":42},{"accuracy":0.619048,"lambda":10.0,"log_loss":0.644039,"matches":42},{"accuracy":0.619048,"lambda":30.0,"log_loss":0.643248,"matches":42},{"accuracy":0.619048,"lambda":100.0,"log_loss":0.641355,"matches":42},{"accuracy":0.619048,"lambda":1000.0,"log_loss":0.638693,"matches":42}],"validation_events":2,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.002915,"matches":207,"own_rating":0.00079,"rating":0.999813,"vrs_rank_gap":0.000247}}}` |
| Perfect World Major Shanghai Major 2024 | 277 | 73 | 73 | -0.0308 | `{"calibration_scales":{"bo1":0.285003,"bo3":0.283926,"bo5":0.285003},"chosen_by":"validation_accuracy","fitted_through":"2024-11-03T16:15:00Z","lambda":3.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":9,"training_matches":277,"validation":[{"accuracy":0.642857,"lambda":1.0,"log_loss":0.694871,"matches":70},{"accuracy":0.657143,"lambda":3.0,"log_loss":0.692391,"matches":70},{"accuracy":0.642857,"lambda":10.0,"log_loss":0.689568,"matches":70},{"accuracy":0.642857,"lambda":30.0,"log_loss":0.686865,"matches":70},{"accuracy":0.642857,"lambda":100.0,"log_loss":0.682907,"matches":70},{"accuracy":0.642857,"lambda":1000.0,"log_loss":0.676573,"matches":70}],"validation_events":3,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.144608,"matches":248,"own_rating":0.017851,"rating":0.964278,"vrs_rank_gap":-0.123576}}}` |
| BLAST Premier Bounty season 1 2025 | 350 | 7 | 7 | -0.7711 | `{"calibration_scales":{"bo1":0.395913,"bo3":0.302271,"bo5":0.313307},"chosen_by":"validation_accuracy","fitted_through":"2024-12-15T09:05:00Z","lambda":1000.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":10,"training_matches":350,"validation":[{"accuracy":0.631579,"lambda":1.0,"log_loss":0.640281,"matches":114},{"accuracy":0.640351,"lambda":3.0,"log_loss":0.638214,"matches":114},{"accuracy":0.666667,"lambda":10.0,"log_loss":0.635316,"matches":114},{"accuracy":0.666667,"lambda":30.0,"log_loss":0.633425,"matches":114},{"accuracy":0.666667,"lambda":100.0,"log_loss":0.63246,"matches":114},{"accuracy":0.666667,"lambda":1000.0,"log_loss":0.632092,"matches":114}],"validation_events":3,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.000086,"matches":321,"own_rating":0.003601,"rating":1.000379,"vrs_rank_gap":0.000413}}}` |
| IEM Katowice 2025 | 357 | 47 | 47 | -0.2250 | `{"calibration_scales":{"bo1":0.395913,"bo3":0.292364,"bo5":0.305588},"chosen_by":"validation_accuracy","fitted_through":"2025-01-26T14:45:00Z","lambda":1000.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":11,"training_matches":357,"validation":[{"accuracy":0.62963,"lambda":1.0,"log_loss":0.625092,"matches":108},{"accuracy":0.638889,"lambda":3.0,"log_loss":0.62342,"matches":108},{"accuracy":0.666667,"lambda":10.0,"log_loss":0.621132,"matches":108},{"accuracy":0.666667,"lambda":30.0,"log_loss":0.619691,"matches":108},{"accuracy":0.666667,"lambda":100.0,"log_loss":0.619003,"matches":108},{"accuracy":0.666667,"lambda":1000.0,"log_loss":0.618882,"matches":108}],"validation_events":3,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.000922,"matches":328,"own_rating":0.003548,"rating":1.000558,"vrs_rank_gap":0.000445}}}` |
| PGL Cluj-Napoca 2025 | 406 | 38 | 38 | -0.0619 | `{"calibration_scales":{"bo1":0.395913,"bo3":0.289783,"bo5":0.299907},"chosen_by":"validation_accuracy","fitted_through":"2025-02-09T15:45:00Z","lambda":30.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":12,"training_matches":406,"validation":[{"accuracy":0.63871,"lambda":1.0,"log_loss":0.631075,"matches":155},{"accuracy":0.63871,"lambda":3.0,"log_loss":0.630684,"matches":155},{"accuracy":0.63871,"lambda":10.0,"log_loss":0.630353,"matches":155},{"accuracy":0.63871,"lambda":30.0,"log_loss":0.63029,"matches":155},{"accuracy":0.63871,"lambda":100.0,"log_loss":0.630306,"matches":155},{"accuracy":0.63871,"lambda":1000.0,"log_loss":0.630515,"matches":155}],"validation_events":4,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.033723,"matches":375,"own_rating":0.051807,"rating":0.973161,"vrs_rank_gap":-0.035107}}}` |
| ESL Pro League Season 21 2025 | 447 | 40 | 40 | -0.0583 | `{"calibration_scales":{"bo1":0.395913,"bo3":0.301953,"bo5":0.310583},"chosen_by":"validation_accuracy","fitted_through":"2025-02-23T17:45:00Z","lambda":3.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":13,"training_matches":447,"validation":[{"accuracy":0.642424,"lambda":1.0,"log_loss":0.623089,"matches":165},{"accuracy":0.642424,"lambda":3.0,"log_loss":0.62261,"matches":165},{"accuracy":0.636364,"lambda":10.0,"log_loss":0.622087,"matches":165},{"accuracy":0.636364,"lambda":30.0,"log_loss":0.621903,"matches":165},{"accuracy":0.636364,"lambda":100.0,"log_loss":0.622005,"matches":165},{"accuracy":0.636364,"lambda":1000.0,"log_loss":0.622517,"matches":165}],"validation_events":4,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.095929,"matches":413,"own_rating":0.050036,"rating":1.011138,"vrs_rank_gap":-0.131887}}}` |
| BLAST Open Lisbon Spring 2025 | 487 | 29 | 29 | -0.0856 | `{"calibration_scales":{"bo1":0.395913,"bo3":0.320069,"bo5":0.327154},"chosen_by":"validation_accuracy","fitted_through":"2025-03-16T15:45:00Z","lambda":10.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":14,"training_matches":487,"validation":[{"accuracy":0.674242,"lambda":1.0,"log_loss":0.594198,"matches":132},{"accuracy":0.674242,"lambda":3.0,"log_loss":0.594162,"matches":132},{"accuracy":0.674242,"lambda":10.0,"log_loss":0.594141,"matches":132},{"accuracy":0.674242,"lambda":30.0,"log_loss":0.594217,"matches":132},{"accuracy":0.674242,"lambda":100.0,"log_loss":0.594498,"matches":132},{"accuracy":0.674242,"lambda":1000.0,"log_loss":0.59531,"matches":132}],"validation_events":4,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.029051,"matches":453,"own_rating":0.060907,"rating":0.943007,"vrs_rank_gap":-0.0183}}}` |
| PGL Bucharest 2025 | 516 | 41 | 41 | -0.8620 | `{"calibration_scales":{"bo1":0.395913,"bo3":0.328512,"bo5":0.335118},"chosen_by":"validation_accuracy","fitted_through":"2025-03-30T15:15:00Z","lambda":10.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":15,"training_matches":516,"validation":[{"accuracy":0.689441,"lambda":1.0,"log_loss":0.581595,"matches":161},{"accuracy":0.689441,"lambda":3.0,"log_loss":0.581553,"matches":161},{"accuracy":0.689441,"lambda":10.0,"log_loss":0.581519,"matches":161},{"accuracy":0.689441,"lambda":30.0,"log_loss":0.581608,"matches":161},{"accuracy":0.689441,"lambda":100.0,"log_loss":0.581974,"matches":161},{"accuracy":0.689441,"lambda":1000.0,"log_loss":0.583055,"matches":161}],"validation_events":5,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.077362,"matches":482,"own_rating":0.079813,"rating":0.907045,"vrs_rank_gap":-0.04419}}}` |
| IEM Melbourne 2025 | 557 | 29 | 29 | -0.1737 | `{"calibration_scales":{"bo1":0.395913,"bo3":0.282383,"bo5":0.28989},"chosen_by":"validation_accuracy","fitted_through":"2025-04-13T14:45:00Z","lambda":3.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":16,"training_matches":557,"validation":[{"accuracy":0.666667,"lambda":1.0,"log_loss":0.617718,"matches":195},{"accuracy":0.666667,"lambda":3.0,"log_loss":0.617598,"matches":195},{"accuracy":0.666667,"lambda":10.0,"log_loss":0.617702,"matches":195},{"accuracy":0.661538,"lambda":30.0,"log_loss":0.617951,"matches":195},{"accuracy":0.661538,"lambda":100.0,"log_loss":0.618179,"matches":195},{"accuracy":0.666667,"lambda":1000.0,"log_loss":0.618541,"matches":195}],"validation_events":5,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.279528,"matches":523,"own_rating":0.028471,"rating":0.745793,"vrs_rank_gap":-0.055414}}}` |
| BLAST Rivals Spring 2025 | 586 | 13 | 13 | -0.1964 | `{"calibration_scales":{"bo1":0.395913,"bo3":0.288126,"bo5":0.296077},"chosen_by":"validation_accuracy","fitted_through":"2025-04-27T06:15:00Z","lambda":10.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":17,"training_matches":586,"validation":[{"accuracy":0.694915,"lambda":1.0,"log_loss":0.589428,"matches":177},{"accuracy":0.694915,"lambda":3.0,"log_loss":0.590032,"matches":177},{"accuracy":0.700565,"lambda":10.0,"log_loss":0.591384,"matches":177},{"accuracy":0.700565,"lambda":30.0,"log_loss":0.592781,"matches":177},{"accuracy":0.700565,"lambda":100.0,"log_loss":0.593563,"matches":177},{"accuracy":0.700565,"lambda":1000.0,"log_loss":0.594102,"matches":177}],"validation_events":5,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.235489,"matches":552,"own_rating":0.024367,"rating":0.814628,"vrs_rank_gap":-0.049677}}}` |
| PGL Astana 2025 | 599 | 41 | 41 | -0.9934 | `{"calibration_scales":{"bo1":0.395913,"bo3":0.289727,"bo5":0.298474},"chosen_by":"validation_accuracy","fitted_through":"2025-05-04T14:00:00Z","lambda":10.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":18,"training_matches":599,"validation":[{"accuracy":0.7,"lambda":1.0,"log_loss":0.58231,"matches":190},{"accuracy":0.7,"lambda":3.0,"log_loss":0.582854,"matches":190},{"accuracy":0.705263,"lambda":10.0,"log_loss":0.58409,"matches":190},{"accuracy":0.705263,"lambda":30.0,"log_loss":0.585409,"matches":190},{"accuracy":0.705263,"lambda":100.0,"log_loss":0.586249,"matches":190},{"accuracy":0.705263,"lambda":1000.0,"log_loss":0.587082,"matches":190}],"validation_events":6,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.229024,"matches":565,"own_rating":0.032633,"rating":0.798878,"vrs_rank_gap":-0.037574}}}` |
| IEM Dallas 2025 | 640 | 29 | 29 | -0.0161 | `{"calibration_scales":{"bo1":0.395913,"bo3":0.258219,"bo5":0.267802},"chosen_by":"validation_accuracy","fitted_through":"2025-05-18T11:55:00Z","lambda":10.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":19,"training_matches":640,"validation":[{"accuracy":0.699482,"lambda":1.0,"log_loss":0.613617,"matches":193},{"accuracy":0.694301,"lambda":3.0,"log_loss":0.613014,"matches":193},{"accuracy":0.699482,"lambda":10.0,"log_loss":0.613359,"matches":193},{"accuracy":0.699482,"lambda":30.0,"log_loss":0.614712,"matches":193},{"accuracy":0.699482,"lambda":100.0,"log_loss":0.615945,"matches":193},{"accuracy":0.699482,"lambda":1000.0,"log_loss":0.616514,"matches":193}],"validation_events":6,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.244875,"matches":606,"own_rating":0.038822,"rating":0.720584,"vrs_rank_gap":-0.003163}}}` |
| BLAST.tv Major Austin 2025 | 669 | 106 | 106 | -0.2365 | `{"calibration_scales":{"bo1":0.395913,"bo3":0.269482,"bo5":0.278819},"chosen_by":"validation_accuracy","fitted_through":"2025-05-25T19:25:00Z","lambda":1000.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":20,"training_matches":669,"validation":[{"accuracy":0.686813,"lambda":1.0,"log_loss":0.609344,"matches":182},{"accuracy":0.686813,"lambda":3.0,"log_loss":0.610642,"matches":182},{"accuracy":0.686813,"lambda":10.0,"log_loss":0.612982,"matches":182},{"accuracy":0.686813,"lambda":30.0,"log_loss":0.61529,"matches":182},{"accuracy":0.686813,"lambda":100.0,"log_loss":0.617008,"matches":182},{"accuracy":0.692308,"lambda":1000.0,"log_loss":0.617758,"matches":182}],"validation_events":6,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.016495,"matches":635,"own_rating":0.018909,"rating":0.997922,"vrs_rank_gap":0.006134}}}` |
| FISSURE PLAYGROUND Season 1 2025 | 775 | 27 | 27 | -0.6625 | `{"calibration_scales":{"bo1":0.247395,"bo3":0.268806,"bo5":0.27203},"chosen_by":"validation_accuracy","fitted_through":"2025-06-22T19:15:00Z","lambda":1.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":21,"training_matches":775,"validation":[{"accuracy":0.663194,"lambda":1.0,"log_loss":0.635055,"matches":288},{"accuracy":0.659722,"lambda":3.0,"log_loss":0.635979,"matches":288},{"accuracy":0.659722,"lambda":10.0,"log_loss":0.637621,"matches":288},{"accuracy":0.65625,"lambda":30.0,"log_loss":0.639195,"matches":288},{"accuracy":0.659722,"lambda":100.0,"log_loss":0.640252,"matches":288},{"accuracy":0.659722,"lambda":1000.0,"log_loss":0.640147,"matches":288}],"validation_events":7,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.264906,"matches":741,"own_rating":0.050189,"rating":0.459368,"vrs_rank_gap":0.136996}}}` |
| IEM Cologne 2025 | 802 | 49 | 49 | +0.0325 | `{"calibration_scales":{"bo1":0.247395,"bo3":0.258638,"bo5":0.261852},"chosen_by":"validation_accuracy","fitted_through":"2025-07-20T12:45:00Z","lambda":10.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":22,"training_matches":802,"validation":[{"accuracy":0.629371,"lambda":1.0,"log_loss":0.661154,"matches":286},{"accuracy":0.629371,"lambda":3.0,"log_loss":0.66243,"matches":286},{"accuracy":0.632867,"lambda":10.0,"log_loss":0.664998,"matches":286},{"accuracy":0.632867,"lambda":30.0,"log_loss":0.667763,"matches":286},{"accuracy":0.632867,"lambda":100.0,"log_loss":0.669637,"matches":286},{"accuracy":0.632867,"lambda":1000.0,"log_loss":0.668684,"matches":286}],"validation_events":7,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.210144,"matches":768,"own_rating":0.051043,"rating":0.666147,"vrs_rank_gap":0.044301}}}` |
| BLAST Bounty Fall 2025 | 851 | 30 | 30 | +0.0440 | `{"calibration_scales":{"bo1":0.247395,"bo3":0.270232,"bo5":0.272275},"chosen_by":"validation_accuracy","fitted_through":"2025-08-03T14:50:00Z","lambda":3.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":23,"training_matches":851,"validation":[{"accuracy":0.646259,"lambda":1.0,"log_loss":0.626132,"matches":294},{"accuracy":0.64966,"lambda":3.0,"log_loss":0.62702,"matches":294},{"accuracy":0.646259,"lambda":10.0,"log_loss":0.628573,"matches":294},{"accuracy":0.646259,"lambda":30.0,"log_loss":0.630146,"matches":294},{"accuracy":0.64966,"lambda":100.0,"log_loss":0.631446,"matches":294},{"accuracy":0.64966,"lambda":1000.0,"log_loss":0.632171,"matches":294}],"validation_events":7,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.230677,"matches":817,"own_rating":0.065031,"rating":0.465567,"vrs_rank_gap":0.151358}}}` |
| Esports World Cup 2025 | 882 | 16 | 16 | -0.4186 | `{"calibration_scales":{"bo1":0.247395,"bo3":0.281385,"bo5":0.282961},"chosen_by":"validation_accuracy","fitted_through":"2025-08-17T13:45:00Z","lambda":3.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":24,"training_matches":882,"validation":[{"accuracy":0.666667,"lambda":1.0,"log_loss":0.611006,"matches":324},{"accuracy":0.669753,"lambda":3.0,"log_loss":0.611577,"matches":324},{"accuracy":0.666667,"lambda":10.0,"log_loss":0.612474,"matches":324},{"accuracy":0.666667,"lambda":30.0,"log_loss":0.613331,"matches":324},{"accuracy":0.669753,"lambda":100.0,"log_loss":0.614176,"matches":324},{"accuracy":0.669753,"lambda":1000.0,"log_loss":0.615169,"matches":324}],"validation_events":8,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.200638,"matches":847,"own_rating":0.077748,"rating":0.489812,"vrs_rank_gap":0.160355}}}` |
| BLAST Open London Fall 2025 | 898 | 29 | 29 | -0.4772 | `{"calibration_scales":{"bo1":0.247395,"bo3":0.277258,"bo5":0.279645},"chosen_by":"validation_accuracy","fitted_through":"2025-08-24T10:10:00Z","lambda":1.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":25,"training_matches":898,"validation":[{"accuracy":0.659164,"lambda":1.0,"log_loss":0.625667,"matches":311},{"accuracy":0.659164,"lambda":3.0,"log_loss":0.626143,"matches":311},{"accuracy":0.655949,"lambda":10.0,"log_loss":0.626871,"matches":311},{"accuracy":0.652733,"lambda":30.0,"log_loss":0.627504,"matches":311},{"accuracy":0.655949,"lambda":100.0,"log_loss":0.62801,"matches":311},{"accuracy":0.655949,"lambda":1000.0,"log_loss":0.628051,"matches":311}],"validation_events":8,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.188266,"matches":863,"own_rating":0.067454,"rating":0.455092,"vrs_rank_gap":0.19088}}}` |
| FISSURE PLAYGROUND Season 2 2025 | 927 | 40 | 40 | -0.2719 | `{"calibration_scales":{"bo1":0.247395,"bo3":0.276219,"bo5":0.275806},"chosen_by":"validation_accuracy","fitted_through":"2025-09-07T14:20:00Z","lambda":1.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":26,"training_matches":927,"validation":[{"accuracy":0.648318,"lambda":1.0,"log_loss":0.633885,"matches":327},{"accuracy":0.648318,"lambda":3.0,"log_loss":0.634344,"matches":327},{"accuracy":0.64526,"lambda":10.0,"log_loss":0.635009,"matches":327},{"accuracy":0.648318,"lambda":30.0,"log_loss":0.635536,"matches":327},{"accuracy":0.64526,"lambda":100.0,"log_loss":0.635917,"matches":327},{"accuracy":0.64526,"lambda":1000.0,"log_loss":0.635411,"matches":327}],"validation_events":8,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.153262,"matches":892,"own_rating":0.064558,"rating":0.437596,"vrs_rank_gap":0.225158}}}` |
| ESL Pro League Season 22 2025 | 967 | 73 | 73 | -0.8209 | `{"calibration_scales":{"bo1":0.247395,"bo3":0.276262,"bo5":0.274371},"chosen_by":"validation_accuracy","fitted_through":"2025-09-21T14:45:00Z","lambda":1.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":27,"training_matches":967,"validation":[{"accuracy":0.648501,"lambda":1.0,"log_loss":0.633735,"matches":367},{"accuracy":0.648501,"lambda":3.0,"log_loss":0.634361,"matches":367},{"accuracy":0.645777,"lambda":10.0,"log_loss":0.635553,"matches":367},{"accuracy":0.648501,"lambda":30.0,"log_loss":0.636957,"matches":367},{"accuracy":0.645777,"lambda":100.0,"log_loss":0.638282,"matches":367},{"accuracy":0.643052,"lambda":1000.0,"log_loss":0.638677,"matches":367}],"validation_events":9,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.216324,"matches":932,"own_rating":0.072526,"rating":0.413488,"vrs_rank_gap":0.179612}}}` |
| CS Asia Championships 2025 | 1040 | 30 | 30 | -0.2213 | `{"calibration_scales":{"bo1":0.247395,"bo3":0.24735,"bo5":0.24801},"chosen_by":"validation_accuracy","fitted_through":"2025-10-12T14:45:00Z","lambda":1.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":28,"training_matches":1040,"validation":[{"accuracy":0.646617,"lambda":1.0,"log_loss":0.640095,"matches":399},{"accuracy":0.64411,"lambda":3.0,"log_loss":0.640939,"matches":399},{"accuracy":0.641604,"lambda":10.0,"log_loss":0.642468,"matches":399},{"accuracy":0.641604,"lambda":30.0,"log_loss":0.644107,"matches":399},{"accuracy":0.639098,"lambda":100.0,"log_loss":0.645692,"matches":399},{"accuracy":0.639098,"lambda":1000.0,"log_loss":0.646873,"matches":399}],"validation_events":9,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.199765,"matches":1005,"own_rating":0.12193,"rating":0.254227,"vrs_rank_gap":0.215125}}}` |
| Thunderpick World Championship 2025 | 1040 | 14 | 14 | -0.1742 | `{"calibration_scales":{"bo1":0.247395,"bo3":0.24735,"bo5":0.24801},"chosen_by":"validation_accuracy","fitted_through":"2025-10-12T14:45:00Z","lambda":1.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":28,"training_matches":1040,"validation":[{"accuracy":0.646617,"lambda":1.0,"log_loss":0.640095,"matches":399},{"accuracy":0.64411,"lambda":3.0,"log_loss":0.640939,"matches":399},{"accuracy":0.641604,"lambda":10.0,"log_loss":0.642468,"matches":399},{"accuracy":0.641604,"lambda":30.0,"log_loss":0.644107,"matches":399},{"accuracy":0.639098,"lambda":100.0,"log_loss":0.645692,"matches":399},{"accuracy":0.639098,"lambda":1000.0,"log_loss":0.646873,"matches":399}],"validation_events":9,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.199765,"matches":1005,"own_rating":0.12193,"rating":0.254227,"vrs_rank_gap":0.215125}}}` |
| PGL Masters Bucharest 2025 | 1084 | 41 | 41 | -0.2658 | `{"calibration_scales":{"bo1":0.246468,"bo3":0.248916,"bo5":0.248788},"chosen_by":"validation_accuracy","fitted_through":"2025-10-19T15:45:00Z","lambda":100.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":30,"training_matches":1084,"validation":[{"accuracy":0.625604,"lambda":1.0,"log_loss":0.652857,"matches":414},{"accuracy":0.625604,"lambda":3.0,"log_loss":0.653702,"matches":414},{"accuracy":0.625604,"lambda":10.0,"log_loss":0.655417,"matches":414},{"accuracy":0.623188,"lambda":30.0,"log_loss":0.657426,"matches":414},{"accuracy":0.630435,"lambda":100.0,"log_loss":0.659381,"matches":414},{"accuracy":0.625604,"lambda":1000.0,"log_loss":0.660128,"matches":414}],"validation_events":10,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.070131,"matches":1049,"own_rating":0.090617,"rating":0.897623,"vrs_rank_gap":-0.016694}}}` |
| IEM Chengdu 2025 | 1125 | 30 | 30 | -0.0655 | `{"calibration_scales":{"bo1":0.246468,"bo3":0.24586,"bo5":0.246563},"chosen_by":"validation_accuracy","fitted_through":"2025-11-01T12:10:00Z","lambda":100.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":31,"training_matches":1125,"validation":[{"accuracy":0.616046,"lambda":1.0,"log_loss":0.649144,"matches":349},{"accuracy":0.616046,"lambda":3.0,"log_loss":0.649864,"matches":349},{"accuracy":0.613181,"lambda":10.0,"log_loss":0.651567,"matches":349},{"accuracy":0.613181,"lambda":30.0,"log_loss":0.653931,"matches":349},{"accuracy":0.618911,"lambda":100.0,"log_loss":0.656445,"matches":349},{"accuracy":0.616046,"lambda":1000.0,"log_loss":0.658177,"matches":349}],"validation_events":10,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.081969,"matches":1090,"own_rating":0.09106,"rating":0.898483,"vrs_rank_gap":-0.027331}}}` |
| BLAST Rivals Fall 2025 | 1155 | 13 | 13 | -0.1231 | `{"calibration_scales":{"bo1":0.246468,"bo3":0.249795,"bo5":0.250304},"chosen_by":"validation_accuracy","fitted_through":"2025-11-09T07:45:00Z","lambda":1.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":32,"training_matches":1155,"validation":[{"accuracy":0.639205,"lambda":1.0,"log_loss":0.631451,"matches":352},{"accuracy":0.636364,"lambda":3.0,"log_loss":0.632355,"matches":352},{"accuracy":0.636364,"lambda":10.0,"log_loss":0.634392,"matches":352},{"accuracy":0.633523,"lambda":30.0,"log_loss":0.637004,"matches":352},{"accuracy":0.639205,"lambda":100.0,"log_loss":0.639749,"matches":352},{"accuracy":0.639205,"lambda":1000.0,"log_loss":0.642807,"matches":352}],"validation_events":10,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.227707,"matches":1120,"own_rating":0.118345,"rating":0.192293,"vrs_rank_gap":0.233351}}}` |
| StarLadder Major Budapest 2025 | 1168 | 106 | 106 | -0.1494 | `{"calibration_scales":{"bo1":0.246468,"bo3":0.251216,"bo5":0.251813},"chosen_by":"validation_accuracy","fitted_through":"2025-11-16T07:55:00Z","lambda":1000.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":33,"training_matches":1168,"validation":[{"accuracy":0.641096,"lambda":1.0,"log_loss":0.628909,"matches":365},{"accuracy":0.638356,"lambda":3.0,"log_loss":0.629931,"matches":365},{"accuracy":0.638356,"lambda":10.0,"log_loss":0.632163,"matches":365},{"accuracy":0.635616,"lambda":30.0,"log_loss":0.634888,"matches":365},{"accuracy":0.641096,"lambda":100.0,"log_loss":0.637638,"matches":365},{"accuracy":0.643836,"lambda":1000.0,"log_loss":0.640763,"matches":365}],"validation_events":11,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.021437,"matches":1133,"own_rating":0.034845,"rating":0.991412,"vrs_rank_gap":0.007414}}}` |
| BLAST Bounty Winter 2026 | 1274 | 31 | 31 | -0.0481 | `{"calibration_scales":{"bo1":0.237421,"bo3":0.253688,"bo5":0.321513},"chosen_by":"validation_accuracy","fitted_through":"2025-12-14T16:45:00Z","lambda":100.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":34,"training_matches":1274,"validation":[{"accuracy":0.627962,"lambda":1.0,"log_loss":0.642013,"matches":422},{"accuracy":0.625592,"lambda":3.0,"log_loss":0.642531,"matches":422},{"accuracy":0.625592,"lambda":10.0,"log_loss":0.644073,"matches":422},{"accuracy":0.625592,"lambda":30.0,"log_loss":0.646547,"matches":422},{"accuracy":0.630332,"lambda":100.0,"log_loss":0.649346,"matches":422},{"accuracy":0.630332,"lambda":1000.0,"log_loss":0.651835,"matches":422}],"validation_events":11,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.0,"market":0.8,"matches":0,"own_rating":0.0,"rating":0.2,"vrs_rank_gap":0.0},"rating":{"hltv_rank_gap":0.093622,"matches":1239,"own_rating":0.083024,"rating":0.877195,"vrs_rank_gap":-0.013894}}}` |
| IEM Krakow 2026 | 1305 | 50 | 50 | +0.0155 | `{"calibration_scales":{"bo1":0.237421,"bo3":0.255109,"bo5":0.257303},"chosen_by":"validation_accuracy","fitted_through":"2026-01-25T14:45:00Z","lambda":1.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":35,"training_matches":1305,"validation":[{"accuracy":0.609929,"lambda":1.0,"log_loss":0.654175,"matches":423},{"accuracy":0.607565,"lambda":3.0,"log_loss":0.654808,"matches":423},{"accuracy":0.605201,"lambda":10.0,"log_loss":0.65662,"matches":423},{"accuracy":0.605201,"lambda":30.0,"log_loss":0.659469,"matches":423},{"accuracy":0.609929,"lambda":100.0,"log_loss":0.662606,"matches":423},{"accuracy":0.609929,"lambda":1000.0,"log_loss":0.664771,"matches":423}],"validation_events":11,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":-0.717992,"market":0.604336,"matches":31,"own_rating":0.329892,"rating":-0.163461,"vrs_rank_gap":0.127548},"rating":{"hltv_rank_gap":0.232182,"matches":1239,"own_rating":0.112833,"rating":0.239512,"vrs_rank_gap":0.198484}}}` |
| PGL Cluj-Napoca 2026 | 1355 | 41 | 41 | +0.0972 | `{"calibration_scales":{"bo1":0.237421,"bo3":0.254301,"bo5":0.255931},"chosen_by":"validation_accuracy","fitted_through":"2026-02-08T15:45:00Z","lambda":1.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":36,"training_matches":1355,"validation":[{"accuracy":0.610994,"lambda":1.0,"log_loss":0.650646,"matches":473},{"accuracy":0.608879,"lambda":3.0,"log_loss":0.651212,"matches":473},{"accuracy":0.606765,"lambda":10.0,"log_loss":0.652832,"matches":473},{"accuracy":0.606765,"lambda":30.0,"log_loss":0.65538,"matches":473},{"accuracy":0.610994,"lambda":100.0,"log_loss":0.658185,"matches":473},{"accuracy":0.610994,"lambda":1000.0,"log_loss":0.660122,"matches":473}],"validation_events":12,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":-0.402407,"market":0.68149,"matches":81,"own_rating":0.0305,"rating":-0.62658,"vrs_rank_gap":0.460068},"rating":{"hltv_rank_gap":0.232179,"matches":1239,"own_rating":0.112849,"rating":0.239979,"vrs_rank_gap":0.198614}}}` |
| ESL Pro League Season 23 2026 | 1396 | 74 | 74 | -0.0221 | `{"calibration_scales":{"bo1":0.237421,"bo3":0.260187,"bo5":0.256053},"chosen_by":"validation_accuracy","fitted_through":"2026-02-22T15:45:00Z","lambda":100.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":37,"training_matches":1396,"validation":[{"accuracy":0.620482,"lambda":1.0,"log_loss":0.640495,"matches":498},{"accuracy":0.620482,"lambda":3.0,"log_loss":0.641154,"matches":498},{"accuracy":0.620482,"lambda":10.0,"log_loss":0.642882,"matches":498},{"accuracy":0.62249,"lambda":30.0,"log_loss":0.645417,"matches":498},{"accuracy":0.628514,"lambda":100.0,"log_loss":0.648138,"matches":498},{"accuracy":0.624498,"lambda":1000.0,"log_loss":0.650358,"matches":498}],"validation_events":12,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.035904,"market":0.798602,"matches":122,"own_rating":0.045477,"rating":0.196183,"vrs_rank_gap":0.027976},"rating":{"hltv_rank_gap":0.093346,"matches":1239,"own_rating":0.082876,"rating":0.870397,"vrs_rank_gap":-0.016967}}}` |
| BLAST Open Rotterdam Spring 2026 | 1470 | 29 | 29 | -0.0207 | `{"calibration_scales":{"bo1":0.237421,"bo3":0.257877,"bo5":0.253405},"chosen_by":"validation_accuracy","fitted_through":"2026-03-15T16:10:00Z","lambda":100.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":38,"training_matches":1470,"validation":[{"accuracy":0.620626,"lambda":1.0,"log_loss":0.643068,"matches":543},{"accuracy":0.620626,"lambda":3.0,"log_loss":0.64362,"matches":543},{"accuracy":0.620626,"lambda":10.0,"log_loss":0.645121,"matches":543},{"accuracy":0.620626,"lambda":30.0,"log_loss":0.64739,"matches":543},{"accuracy":0.626151,"lambda":100.0,"log_loss":0.649891,"matches":543},{"accuracy":0.620626,"lambda":1000.0,"log_loss":0.652192,"matches":543}],"validation_events":12,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.007403,"market":0.78252,"matches":196,"own_rating":0.027555,"rating":0.185934,"vrs_rank_gap":0.008335},"rating":{"hltv_rank_gap":0.093853,"matches":1239,"own_rating":0.083249,"rating":0.87327,"vrs_rank_gap":-0.015771}}}` |
| PGL Bucharest 2026 | 1499 | 39 | 39 | +0.0031 | `{"calibration_scales":{"bo1":0.237421,"bo3":0.25847,"bo5":0.276963},"chosen_by":"validation_accuracy","fitted_through":"2026-03-29T10:15:00Z","lambda":100.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":39,"training_matches":1499,"validation":[{"accuracy":0.620629,"lambda":1.0,"log_loss":0.640598,"matches":572},{"accuracy":0.620629,"lambda":3.0,"log_loss":0.641122,"matches":572},{"accuracy":0.620629,"lambda":10.0,"log_loss":0.642546,"matches":572},{"accuracy":0.620629,"lambda":30.0,"log_loss":0.644701,"matches":572},{"accuracy":0.625874,"lambda":100.0,"log_loss":0.647075,"matches":572},{"accuracy":0.620629,"lambda":1000.0,"log_loss":0.649259,"matches":572}],"validation_events":13,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.009147,"market":0.774075,"matches":225,"own_rating":0.026661,"rating":0.18494,"vrs_rank_gap":0.008443},"rating":{"hltv_rank_gap":0.093308,"matches":1239,"own_rating":0.082841,"rating":0.872377,"vrs_rank_gap":-0.016258}}}` |
| IEM Rio 2026 | 1538 | 30 | 30 | +0.0321 | `{"calibration_scales":{"bo1":0.237421,"bo3":0.261177,"bo5":0.267862},"chosen_by":"validation_accuracy","fitted_through":"2026-04-11T13:45:00Z","lambda":1.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":40,"training_matches":1538,"validation":[{"accuracy":0.62697,"lambda":1.0,"log_loss":0.634265,"matches":571},{"accuracy":0.625219,"lambda":3.0,"log_loss":0.634747,"matches":571},{"accuracy":0.623468,"lambda":10.0,"log_loss":0.636024,"matches":571},{"accuracy":0.623468,"lambda":30.0,"log_loss":0.637924,"matches":571},{"accuracy":0.62697,"lambda":100.0,"log_loss":0.640064,"matches":571},{"accuracy":0.62697,"lambda":1000.0,"log_loss":0.642054,"matches":571}],"validation_events":13,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.001233,"market":0.5967,"matches":264,"own_rating":0.132346,"rating":-0.176239,"vrs_rank_gap":0.226474},"rating":{"hltv_rank_gap":0.232204,"matches":1239,"own_rating":0.112716,"rating":0.23593,"vrs_rank_gap":0.197596}}}` |
| BLAST Rivals Spring 2026 | 1568 | 13 | 13 | -0.0107 | `{"calibration_scales":{"bo1":0.237421,"bo3":0.266287,"bo5":0.289673},"chosen_by":"validation_accuracy","fitted_through":"2026-04-19T16:45:00Z","lambda":10.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":41,"training_matches":1568,"validation":[{"accuracy":0.642045,"lambda":1.0,"log_loss":0.616089,"matches":528},{"accuracy":0.642045,"lambda":3.0,"log_loss":0.616221,"matches":528},{"accuracy":0.643939,"lambda":10.0,"log_loss":0.616743,"matches":528},{"accuracy":0.643939,"lambda":30.0,"log_loss":0.617615,"matches":528},{"accuracy":0.640152,"lambda":100.0,"log_loss":0.618536,"matches":528},{"accuracy":0.640152,"lambda":1000.0,"log_loss":0.619701,"matches":528}],"validation_events":13,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.02783,"market":0.72218,"matches":292,"own_rating":0.108497,"rating":0.118024,"vrs_rank_gap":0.047351},"rating":{"hltv_rank_gap":0.212561,"matches":1241,"own_rating":0.107784,"rating":0.4759,"vrs_rank_gap":0.072798}}}` |
| PGL Astana 2026 | 1581 | 41 | 41 | +0.0057 | `{"calibration_scales":{"bo1":0.237421,"bo3":0.266822,"bo5":0.30361},"chosen_by":"validation_accuracy","fitted_through":"2026-05-03T17:15:00Z","lambda":10.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":42,"training_matches":1581,"validation":[{"accuracy":0.643253,"lambda":1.0,"log_loss":0.615506,"matches":541},{"accuracy":0.643253,"lambda":3.0,"log_loss":0.615635,"matches":541},{"accuracy":0.645102,"lambda":10.0,"log_loss":0.616145,"matches":541},{"accuracy":0.645102,"lambda":30.0,"log_loss":0.616996,"matches":541},{"accuracy":0.641405,"lambda":100.0,"log_loss":0.617895,"matches":541},{"accuracy":0.641405,"lambda":1000.0,"log_loss":0.619032,"matches":541}],"validation_events":14,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":-0.0085,"market":0.714956,"matches":305,"own_rating":0.102261,"rating":0.119226,"vrs_rank_gap":0.061897},"rating":{"hltv_rank_gap":0.212373,"matches":1241,"own_rating":0.107695,"rating":0.474445,"vrs_rank_gap":0.073067}}}` |
| IEM Atlanta 2026 | 1581 | 30 | 30 | -0.0124 | `{"calibration_scales":{"bo1":0.237421,"bo3":0.266822,"bo5":0.30361},"chosen_by":"validation_accuracy","fitted_through":"2026-05-03T17:15:00Z","lambda":10.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":42,"training_matches":1581,"validation":[{"accuracy":0.643253,"lambda":1.0,"log_loss":0.615506,"matches":541},{"accuracy":0.643253,"lambda":3.0,"log_loss":0.615635,"matches":541},{"accuracy":0.645102,"lambda":10.0,"log_loss":0.616145,"matches":541},{"accuracy":0.645102,"lambda":30.0,"log_loss":0.616996,"matches":541},{"accuracy":0.641405,"lambda":100.0,"log_loss":0.617895,"matches":541},{"accuracy":0.641405,"lambda":1000.0,"log_loss":0.619032,"matches":541}],"validation_events":14,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":-0.0085,"market":0.714956,"matches":305,"own_rating":0.102261,"rating":0.119226,"vrs_rank_gap":0.061897},"rating":{"hltv_rank_gap":0.212373,"matches":1241,"own_rating":0.107695,"rating":0.474445,"vrs_rank_gap":0.073067}}}` |
| CS Asia Championships 2026 | 1652 | 30 | 30 | -0.1905 | `{"calibration_scales":{"bo1":0.237421,"bo3":0.269944,"bo5":0.321068},"chosen_by":"validation_accuracy","fitted_through":"2026-05-17T18:10:00Z","lambda":10.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":44,"training_matches":1652,"validation":[{"accuracy":0.651408,"lambda":1.0,"log_loss":0.610055,"matches":568},{"accuracy":0.651408,"lambda":3.0,"log_loss":0.610145,"matches":568},{"accuracy":0.653169,"lambda":10.0,"log_loss":0.61053,"matches":568},{"accuracy":0.653169,"lambda":30.0,"log_loss":0.611225,"matches":568},{"accuracy":0.653169,"lambda":100.0,"log_loss":0.611993,"matches":568},{"accuracy":0.651408,"lambda":1000.0,"log_loss":0.612919,"matches":568}],"validation_events":14,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":-0.026801,"market":0.773626,"matches":376,"own_rating":0.053609,"rating":0.126125,"vrs_rank_gap":0.073025},"rating":{"hltv_rank_gap":0.212291,"matches":1241,"own_rating":0.107575,"rating":0.46805,"vrs_rank_gap":0.073705}}}` |
| IEM Cologne Major 2026 | 1682 | 106 | 106 | +0.0079 | `{"calibration_scales":{"bo1":0.248029,"bo3":0.271524,"bo5":0.281227},"chosen_by":"validation_accuracy","fitted_through":"2026-05-24T06:00:00Z","lambda":10.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":45,"training_matches":1682,"validation":[{"accuracy":0.652174,"lambda":1.0,"log_loss":0.609144,"matches":598},{"accuracy":0.652174,"lambda":3.0,"log_loss":0.609291,"matches":598},{"accuracy":0.653846,"lambda":10.0,"log_loss":0.609804,"matches":598},{"accuracy":0.653846,"lambda":30.0,"log_loss":0.610662,"matches":598},{"accuracy":0.653846,"lambda":100.0,"log_loss":0.611611,"matches":598},{"accuracy":0.652174,"lambda":1000.0,"log_loss":0.612899,"matches":598}],"validation_events":15,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":-0.026614,"market":0.773406,"matches":376,"own_rating":0.05365,"rating":0.127089,"vrs_rank_gap":0.072428},"rating":{"hltv_rank_gap":0.21765,"matches":1271,"own_rating":0.11282,"rating":0.450658,"vrs_rank_gap":0.076131}}}` |
| XSE Pro League 2026 | 1788 | 37 | 37 | -0.0430 | `{"calibration_scales":{"bo1":0.272686,"bo3":0.274544,"bo5":0.297334},"chosen_by":"validation_accuracy","fitted_through":"2026-06-21T14:45:00Z","lambda":3.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":46,"training_matches":1788,"validation":[{"accuracy":0.668175,"lambda":1.0,"log_loss":0.602846,"matches":663},{"accuracy":0.669683,"lambda":3.0,"log_loss":0.603068,"matches":663},{"accuracy":0.669683,"lambda":10.0,"log_loss":0.603625,"matches":663},{"accuracy":0.668175,"lambda":30.0,"log_loss":0.604323,"matches":663},{"accuracy":0.669683,"lambda":100.0,"log_loss":0.604927,"matches":663},{"accuracy":0.669683,"lambda":1000.0,"log_loss":0.605841,"matches":663}],"validation_events":15,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":-0.053102,"market":0.896407,"matches":480,"own_rating":0.064662,"rating":0.054705,"vrs_rank_gap":0.071737},"rating":{"hltv_rank_gap":0.23666,"matches":1273,"own_rating":0.115101,"rating":0.259615,"vrs_rank_gap":0.172598}}}` |
| BLAST Bounty Summer 2026 | 1828 | 30 | 30 | -0.0537 | `{"calibration_scales":{"bo1":0.270938,"bo3":0.275425,"bo5":0.311595},"chosen_by":"validation_accuracy","fitted_through":"2026-07-12T08:05:00Z","lambda":10.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":47,"training_matches":1828,"validation":[{"accuracy":0.665672,"lambda":1.0,"log_loss":0.607879,"matches":670},{"accuracy":0.665672,"lambda":3.0,"log_loss":0.607787,"matches":670},{"accuracy":0.665672,"lambda":10.0,"log_loss":0.607747,"matches":670},{"accuracy":0.664179,"lambda":30.0,"log_loss":0.607964,"matches":670},{"accuracy":0.665672,"lambda":100.0,"log_loss":0.608421,"matches":670},{"accuracy":0.664179,"lambda":1000.0,"log_loss":0.609411,"matches":670}],"validation_events":15,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.0,"matches":0,"own_rating":0.0,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":-0.037476,"market":0.871239,"matches":480,"own_rating":0.06334,"rating":0.141016,"vrs_rank_gap":0.032884},"rating":{"hltv_rank_gap":0.22225,"matches":1310,"own_rating":0.118097,"rating":0.448532,"vrs_rank_gap":0.064077}}}` |
| Esports World Cup 2026 | 1859 | 56 | 56 | +0.0065 | `{"calibration_scales":{"bo1":0.270938,"bo3":0.272057,"bo5":0.280486},"chosen_by":"validation_accuracy","fitted_through":"2026-08-02T10:15:00Z","lambda":10.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":48,"training_matches":1859,"validation":[{"accuracy":0.664286,"lambda":1.0,"log_loss":0.611334,"matches":700},{"accuracy":0.664286,"lambda":3.0,"log_loss":0.611245,"matches":700},{"accuracy":0.664286,"lambda":10.0,"log_loss":0.611208,"matches":700},{"accuracy":0.662857,"lambda":30.0,"log_loss":0.611415,"matches":700},{"accuracy":0.664286,"lambda":100.0,"log_loss":0.611853,"matches":700},{"accuracy":0.662857,"lambda":1000.0,"log_loss":0.6128,"matches":700}],"validation_events":16,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.020503,"matches":1,"own_rating":-0.07415,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.038646,"market":0.808019,"matches":509,"own_rating":0.052401,"rating":0.109521,"vrs_rank_gap":0.014408},"rating":{"hltv_rank_gap":0.222447,"matches":1310,"own_rating":0.118298,"rating":0.456057,"vrs_rank_gap":0.06287}}}` |
| BLAST Open Porto Fall 2026 | 1915 | 29 | 29 | +0.0074 | `{"calibration_scales":{"bo1":0.282125,"bo3":0.275584,"bo5":0.297826},"chosen_by":"validation_accuracy","fitted_through":"2026-08-23T12:00:00Z","lambda":10.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":49,"training_matches":1915,"validation":[{"accuracy":0.66891,"lambda":1.0,"log_loss":0.610996,"matches":743},{"accuracy":0.66891,"lambda":3.0,"log_loss":0.610812,"matches":743},{"accuracy":0.66891,"lambda":10.0,"log_loss":0.610569,"matches":743},{"accuracy":0.667564,"lambda":30.0,"log_loss":0.610574,"matches":743},{"accuracy":0.667564,"lambda":100.0,"log_loss":0.6109,"matches":743},{"accuracy":0.666218,"lambda":1000.0,"log_loss":0.611746,"matches":743}],"validation_events":16,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.020503,"matches":1,"own_rating":-0.07415,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.041317,"market":0.767294,"matches":565,"own_rating":0.073019,"rating":0.155646,"vrs_rank_gap":0.002871},"rating":{"hltv_rank_gap":0.222148,"matches":1310,"own_rating":0.118446,"rating":0.446869,"vrs_rank_gap":0.063723}}}` |
| FISSURE PLAYGROUND Season 3 2026 | 1944 | 29 | 29 | -0.0374 | `{"calibration_scales":{"bo1":0.282125,"bo3":0.278828,"bo5":0.307948},"chosen_by":"validation_accuracy","fitted_through":"2026-09-06T13:45:00Z","lambda":1000.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":50,"training_matches":1944,"validation":[{"accuracy":0.678679,"lambda":1.0,"log_loss":0.599302,"matches":666},{"accuracy":0.678679,"lambda":3.0,"log_loss":0.599331,"matches":666},{"accuracy":0.678679,"lambda":10.0,"log_loss":0.599431,"matches":666},{"accuracy":0.678679,"lambda":30.0,"log_loss":0.599638,"matches":666},{"accuracy":0.678679,"lambda":100.0,"log_loss":0.599975,"matches":666},{"accuracy":0.68018,"lambda":1000.0,"log_loss":0.600696,"matches":666}],"validation_events":16,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.000237,"matches":1,"own_rating":-0.000858,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.009322,"market":0.805038,"matches":594,"own_rating":0.014546,"rating":0.202585,"vrs_rank_gap":0.008144},"rating":{"hltv_rank_gap":0.017293,"matches":1310,"own_rating":0.031869,"rating":0.982418,"vrs_rank_gap":-0.002865}}}` |
| StarLadder StarSeries Fall 2026 | 1973 | 14 | 14 | -0.0387 | `{"calibration_scales":{"bo1":0.282125,"bo3":0.277275,"bo5":0.313009},"chosen_by":"validation_accuracy","fitted_through":"2026-09-13T08:55:00Z","lambda":1000.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":51,"training_matches":1973,"validation":[{"accuracy":0.673381,"lambda":1.0,"log_loss":0.604217,"matches":695},{"accuracy":0.673381,"lambda":3.0,"log_loss":0.604244,"matches":695},{"accuracy":0.673381,"lambda":10.0,"log_loss":0.60434,"matches":695},{"accuracy":0.673381,"lambda":30.0,"log_loss":0.604539,"matches":695},{"accuracy":0.673381,"lambda":100.0,"log_loss":0.604861,"matches":695},{"accuracy":0.67482,"lambda":1000.0,"log_loss":0.605552,"matches":695}],"validation_events":17,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.000237,"matches":1,"own_rating":-0.000858,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.005928,"market":0.802179,"matches":623,"own_rating":0.014148,"rating":0.201657,"vrs_rank_gap":0.007166},"rating":{"hltv_rank_gap":0.017603,"matches":1310,"own_rating":0.032195,"rating":0.982861,"vrs_rank_gap":-0.002437}}}` |
| ESL Pro League Season 24 2026 | 1987 | 41 | 41 | -0.0355 | `{"calibration_scales":{"bo1":0.282125,"bo3":0.276451,"bo5":0.328123},"chosen_by":"validation_accuracy","fitted_through":"2026-09-20T14:45:00Z","lambda":1000.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":52,"training_matches":1987,"validation":[{"accuracy":0.628319,"lambda":1.0,"log_loss":0.65226,"matches":678},{"accuracy":0.666667,"lambda":3.0,"log_loss":0.628857,"matches":678},{"accuracy":0.672566,"lambda":10.0,"log_loss":0.614262,"matches":678},{"accuracy":0.674041,"lambda":30.0,"log_loss":0.608735,"matches":678},{"accuracy":0.676991,"lambda":100.0,"log_loss":0.606238,"matches":678},{"accuracy":0.678466,"lambda":1000.0,"log_loss":0.605554,"matches":678}],"validation_events":17,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.000237,"matches":1,"own_rating":-0.000858,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.005579,"market":0.802196,"matches":637,"own_rating":0.01243,"rating":0.201385,"vrs_rank_gap":0.006105},"rating":{"hltv_rank_gap":0.017673,"matches":1310,"own_rating":0.032249,"rating":0.982996,"vrs_rank_gap":-0.002291}}}` |
| Thunderpick World Championship 2026 | 2028 | 14 | 14 | +0.0008 | `{"calibration_scales":{"bo1":0.282125,"bo3":0.274277,"bo5":0.332202},"chosen_by":"validation_accuracy","fitted_through":"2026-10-11T14:45:00Z","lambda":1000.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":53,"training_matches":2028,"validation":[{"accuracy":0.669656,"lambda":1.0,"log_loss":0.627073,"matches":669},{"accuracy":0.678625,"lambda":3.0,"log_loss":0.614345,"matches":669},{"accuracy":0.683109,"lambda":10.0,"log_loss":0.6062,"matches":669},{"accuracy":0.681614,"lambda":30.0,"log_loss":0.603375,"matches":669},{"accuracy":0.681614,"lambda":100.0,"log_loss":0.602473,"matches":669},{"accuracy":0.683109,"lambda":1000.0,"log_loss":0.602521,"matches":669}],"validation_events":17,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.000237,"matches":1,"own_rating":-0.000858,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.003348,"market":0.803779,"matches":678,"own_rating":0.012842,"rating":0.200468,"vrs_rank_gap":0.003496},"rating":{"hltv_rank_gap":0.018136,"matches":1310,"own_rating":0.03274,"rating":0.983624,"vrs_rank_gap":-0.001665}}}` |
| PGL Masters Bucharest 2026 | 2042 | 41 | 41 | -0.0759 | `{"calibration_scales":{"bo1":0.282125,"bo3":0.274449,"bo5":0.345565},"chosen_by":"validation_accuracy","fitted_through":"2026-10-18T15:45:00Z","lambda":10.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":54,"training_matches":2042,"validation":[{"accuracy":0.667643,"lambda":1.0,"log_loss":0.628339,"matches":683},{"accuracy":0.676428,"lambda":3.0,"log_loss":0.615193,"matches":683},{"accuracy":0.683748,"lambda":10.0,"log_loss":0.60682,"matches":683},{"accuracy":0.682284,"lambda":30.0,"log_loss":0.603904,"matches":683},{"accuracy":0.68082,"lambda":100.0,"log_loss":0.602959,"matches":683},{"accuracy":0.682284,"lambda":1000.0,"log_loss":0.602974,"matches":683}],"validation_events":18,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.020503,"matches":1,"own_rating":-0.07415,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":-0.003853,"market":0.805209,"matches":692,"own_rating":0.076365,"rating":0.122911,"vrs_rank_gap":0.004511},"rating":{"hltv_rank_gap":0.221587,"matches":1310,"own_rating":0.11826,"rating":0.444257,"vrs_rank_gap":0.065883}}}` |
| IEM Beijing 2026 | 2083 | 30 | 30 | -0.0517 | `{"calibration_scales":{"bo1":0.282125,"bo3":0.267663,"bo5":0.359388},"chosen_by":"validation_accuracy","fitted_through":"2026-10-31T10:45:00Z","lambda":100.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":55,"training_matches":2083,"validation":[{"accuracy":0.664714,"lambda":1.0,"log_loss":0.622416,"matches":683},{"accuracy":0.670571,"lambda":3.0,"log_loss":0.620386,"matches":683},{"accuracy":0.672035,"lambda":10.0,"log_loss":0.61672,"matches":683},{"accuracy":0.674963,"lambda":30.0,"log_loss":0.613789,"matches":683},{"accuracy":0.674963,"lambda":100.0,"log_loss":0.611653,"matches":683},{"accuracy":0.673499,"lambda":1000.0,"log_loss":0.610892,"matches":683}],"validation_events":18,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.002337,"matches":1,"own_rating":-0.008452,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.021419,"market":0.801662,"matches":733,"own_rating":0.03629,"rating":0.182412,"vrs_rank_gap":-0.031844},"rating":{"hltv_rank_gap":0.09388,"matches":1310,"own_rating":0.090179,"rating":0.852713,"vrs_rank_gap":-0.029395}}}` |
| BLAST Rivals Fall 2026 | 2113 | 13 | 13 | +0.0091 | `{"calibration_scales":{"bo1":0.282125,"bo3":0.264677,"bo5":0.372648},"chosen_by":"validation_accuracy","fitted_through":"2026-11-08T07:15:00Z","lambda":1000.0,"objective":"penalised log-loss; λ chosen by accuracy on the latest training Events","training_events":56,"training_matches":2113,"validation":[{"accuracy":0.674491,"lambda":1.0,"log_loss":0.607995,"matches":639},{"accuracy":0.677621,"lambda":3.0,"log_loss":0.605903,"matches":639},{"accuracy":0.672926,"lambda":10.0,"log_loss":0.604525,"matches":639},{"accuracy":0.677621,"lambda":30.0,"log_loss":0.60412,"matches":639},{"accuracy":0.677621,"lambda":100.0,"log_loss":0.604338,"matches":639},{"accuracy":0.679186,"lambda":1000.0,"log_loss":0.605168,"matches":639}],"validation_events":18,"weights":{"market":{"hltv_rank_gap":0.0,"market":1.000237,"matches":1,"own_rating":-0.000858,"vrs_rank_gap":0.0},"market+rating":{"hltv_rank_gap":0.00285,"market":0.802533,"matches":763,"own_rating":0.010527,"rating":0.197044,"vrs_rank_gap":-0.00527},"rating":{"hltv_rank_gap":0.020064,"matches":1310,"own_rating":0.034748,"rating":0.986066,"vrs_rank_gap":0.001018}}}` |

## `roster_stand_in`: no evidence

A stand-in in the active five, from the pre-cutoff Liquipedia revision..

Verdict: the interval of the log-loss difference includes zero.

| On the 1900 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.7937 | 0.2397 | 66.1% |
| roster_stand_in | 0.7938 | 0.2395 | 66.2% |

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
| Perfect World Major Shanghai Major 2024 | 277 | 73 | 73 | +0.0000 | — |
| BLAST Premier Bounty season 1 2025 | 350 | 7 | 7 | +0.0000 | — |
| IEM Katowice 2025 | 357 | 49 | 47 | +0.0000 | — |
| PGL Cluj-Napoca 2025 | 406 | 41 | 38 | +0.0000 | — |
| ESL Pro League Season 21 2025 | 447 | 40 | 40 | +0.0000 | — |
| BLAST Open Lisbon Spring 2025 | 487 | 29 | 29 | +0.0000 | — |
| PGL Bucharest 2025 | 516 | 35 | 35 | +0.0000 | — |
| IEM Melbourne 2025 | 557 | 28 | 28 | +0.0000 | — |
| BLAST Rivals Spring 2025 | 586 | 13 | 13 | +0.0000 | — |
| PGL Astana 2025 | 599 | 41 | 41 | +0.0000 | — |
| IEM Dallas 2025 | 640 | 29 | 29 | +0.0000 | — |
| BLAST.tv Major Austin 2025 | 669 | 106 | 106 | +0.0086 | — |
| FISSURE PLAYGROUND Season 1 2025 | 775 | 27 | 27 | +0.0000 | — |
| IEM Cologne 2025 | 802 | 49 | 49 | +0.0000 | — |
| BLAST Bounty Fall 2025 | 851 | 7 | 7 | +0.0000 | — |
| Esports World Cup 2025 | 882 | 16 | 16 | +0.0000 | — |
| BLAST Open London Fall 2025 | 898 | 7 | 7 | +0.0000 | — |
| FISSURE PLAYGROUND Season 2 2025 | 927 | 40 | 40 | +0.0000 | — |
| ESL Pro League Season 22 2025 | 967 | 65 | 65 | +0.0000 | — |
| CS Asia Championships 2025 | 1040 | 30 | 30 | +0.0000 | — |
| Thunderpick World Championship 2025 | 1040 | 14 | 14 | +0.0000 | — |
| PGL Masters Bucharest 2025 | 1084 | 41 | 41 | +0.0000 | — |
| IEM Chengdu 2025 | 1125 | 30 | 30 | +0.0000 | — |
| BLAST Rivals Fall 2025 | 1155 | 13 | 13 | +0.0000 | — |
| StarLadder Major Budapest 2025 | 1168 | 106 | 106 | -0.0021 | — |
| BLAST Bounty Winter 2026 | 1274 | 7 | 7 | +0.0000 | — |
| IEM Krakow 2026 | 1305 | 50 | 50 | +0.0000 | — |
| PGL Cluj-Napoca 2026 | 1355 | 41 | 41 | +0.0000 | — |
| ESL Pro League Season 23 2026 | 1396 | 8 | 8 | +0.0000 | — |
| BLAST Open Rotterdam Spring 2026 | 1470 | 24 | 24 | +0.0000 | — |
| PGL Bucharest 2026 | 1499 | 39 | 39 | +0.0000 | — |
| IEM Rio 2026 | 1538 | 30 | 30 | +0.0000 | — |
| BLAST Rivals Spring 2026 | 1568 | 13 | 13 | +0.0000 | — |
| PGL Astana 2026 | 1581 | 41 | 41 | +0.0000 | — |
| IEM Atlanta 2026 | 1581 | 30 | 30 | +0.0000 | — |
| CS Asia Championships 2026 | 1652 | 30 | 30 | +0.0000 | — |
| IEM Cologne Major 2026 | 1682 | 106 | 106 | -0.0047 | — |
| XSE Pro League 2026 | 1788 | 39 | 37 | +0.0000 | — |
| BLAST Bounty Summer 2026 | 1828 | 7 | 7 | +0.0000 | — |
| Esports World Cup 2026 | 1859 | 56 | 56 | +0.0000 | — |
| BLAST Open Porto Fall 2026 | 1915 | 26 | 26 | +0.0000 | — |
| FISSURE PLAYGROUND Season 3 2026 | 1944 | 29 | 29 | +0.0000 | — |
| StarLadder StarSeries Fall 2026 | 1973 | 14 | 14 | +0.0000 | — |
| ESL Pro League Season 24 2026 | 1987 | 36 | 36 | +0.0000 | — |
| Thunderpick World Championship 2026 | 2028 | 14 | 14 | +0.0000 | — |
| PGL Masters Bucharest 2026 | 2042 | 41 | 41 | +0.0000 | — |
| IEM Beijing 2026 | 2083 | 30 | 30 | +0.0000 | — |
| BLAST Rivals Fall 2026 | 2113 | 13 | 13 | +0.0000 | — |

## `roster_change_30d`: harmful

An active-five change bounded to the last 30 days by dated lineup observations..

Verdict: the interval of the log-loss difference is above zero: worse than the anchor.

| On the 1427 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.7773 | 0.2397 | 65.7% |
| roster_change_30d | 0.7818 | 0.2401 | 65.7% |

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
| Perfect World Major Shanghai Major 2024 | 277 | 73 | 73 | -0.0001 | — |
| BLAST Premier Bounty season 1 2025 | 350 | 7 | 7 | +0.0010 | — |
| IEM Katowice 2025 | 357 | 45 | 45 | -0.0010 | — |
| PGL Cluj-Napoca 2025 | 406 | 28 | 28 | -0.0149 | — |
| ESL Pro League Season 21 2025 | 447 | 28 | 28 | -0.0020 | — |
| BLAST Open Lisbon Spring 2025 | 487 | 21 | 21 | +0.0325 | — |
| PGL Bucharest 2025 | 516 | 25 | 25 | +0.0129 | — |
| IEM Melbourne 2025 | 557 | 24 | 24 | -0.0123 | — |
| BLAST Rivals Spring 2025 | 586 | 13 | 13 | -0.0460 | — |
| PGL Astana 2025 | 599 | 32 | 32 | +0.0159 | — |
| IEM Dallas 2025 | 640 | 17 | 17 | +0.0072 | — |
| BLAST.tv Major Austin 2025 | 669 | 80 | 80 | +0.0064 | — |
| FISSURE PLAYGROUND Season 1 2025 | 775 | 25 | 25 | +0.0037 | — |
| IEM Cologne 2025 | 802 | 49 | 49 | -0.0041 | — |
| BLAST Bounty Fall 2025 | 851 | 4 | 4 | -0.0387 | — |
| Esports World Cup 2025 | 882 | 7 | 7 | -0.0138 | — |
| BLAST Open London Fall 2025 | 898 | 7 | 7 | +0.0277 | — |
| FISSURE PLAYGROUND Season 2 2025 | 927 | 36 | 36 | +0.0284 | — |
| ESL Pro League Season 22 2025 | 967 | 41 | 41 | -0.0012 | — |
| CS Asia Championships 2025 | 1040 | 18 | 18 | +0.0005 | — |
| Thunderpick World Championship 2025 | 1040 | 10 | 10 | +0.0108 | — |
| PGL Masters Bucharest 2025 | 1084 | 32 | 32 | +0.0059 | — |
| IEM Chengdu 2025 | 1125 | 24 | 24 | +0.0008 | — |
| BLAST Rivals Fall 2025 | 1155 | 13 | 13 | -0.0132 | — |
| StarLadder Major Budapest 2025 | 1168 | 82 | 82 | -0.0008 | — |
| BLAST Bounty Winter 2026 | 1274 | 7 | 7 | +0.0365 | — |
| IEM Krakow 2026 | 1305 | 44 | 44 | +0.0031 | — |
| PGL Cluj-Napoca 2026 | 1355 | 23 | 23 | +0.0017 | — |
| ESL Pro League Season 23 2026 | 1396 | 7 | 7 | -0.0075 | — |
| BLAST Open Rotterdam Spring 2026 | 1470 | 16 | 16 | +0.0050 | — |
| PGL Bucharest 2026 | 1499 | 12 | 12 | -0.0012 | — |
| IEM Rio 2026 | 1538 | 21 | 21 | -0.0028 | — |
| BLAST Rivals Spring 2026 | 1568 | 10 | 10 | +0.0006 | — |
| PGL Astana 2026 | 1581 | 41 | 41 | -0.0020 | — |
| IEM Atlanta 2026 | 1581 | 30 | 30 | -0.0010 | — |
| CS Asia Championships 2026 | 1652 | 28 | 28 | -0.0119 | — |
| IEM Cologne Major 2026 | 1682 | 96 | 96 | -0.0007 | — |
| XSE Pro League 2026 | 1788 | 16 | 16 | +0.0029 | — |
| BLAST Bounty Summer 2026 | 1828 | 7 | 7 | +0.0133 | — |
| Esports World Cup 2026 | 1859 | 36 | 36 | +0.0124 | — |
| BLAST Open Porto Fall 2026 | 1915 | 16 | 16 | +0.0013 | — |
| FISSURE PLAYGROUND Season 3 2026 | 1944 | 11 | 11 | -0.0051 | — |
| StarLadder StarSeries Fall 2026 | 1973 | 14 | 14 | -0.0002 | — |
| ESL Pro League Season 24 2026 | 1987 | 25 | 25 | +0.0042 | — |
| Thunderpick World Championship 2026 | 2028 | 12 | 12 | -0.0045 | — |
| PGL Masters Bucharest 2026 | 2042 | 41 | 41 | -0.0008 | — |
| IEM Beijing 2026 | 2083 | 26 | 26 | +0.0084 | — |
| BLAST Rivals Fall 2026 | 2113 | 13 | 13 | -0.0083 | — |

## `roster_stability`: harmful

Days since the current five was first observed after a different five..

Verdict: the interval of the log-loss difference is above zero: worse than the anchor.

| On the 1471 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.7732 | 0.2394 | 65.7% |
| roster_stability | 0.7773 | 0.2406 | 65.4% |

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
| Perfect World Major Shanghai Major 2024 | 277 | 8 | 8 | +0.0451 | — |
| BLAST Premier Bounty season 1 2025 | 350 | 3 | 3 | -0.0095 | — |
| IEM Katowice 2025 | 357 | 30 | 30 | -0.0020 | — |
| PGL Cluj-Napoca 2025 | 406 | 38 | 38 | +0.0278 | — |
| ESL Pro League Season 21 2025 | 447 | 26 | 26 | -0.0077 | — |
| BLAST Open Lisbon Spring 2025 | 487 | 19 | 19 | +0.0185 | — |
| PGL Bucharest 2025 | 516 | 29 | 29 | +0.0241 | — |
| IEM Melbourne 2025 | 557 | 24 | 24 | +0.0034 | — |
| BLAST Rivals Spring 2025 | 586 | 9 | 9 | -0.0066 | — |
| PGL Astana 2025 | 599 | 30 | 30 | -0.0043 | — |
| IEM Dallas 2025 | 640 | 29 | 29 | +0.0187 | — |
| BLAST.tv Major Austin 2025 | 669 | 86 | 86 | +0.0029 | — |
| FISSURE PLAYGROUND Season 1 2025 | 775 | 27 | 27 | -0.0101 | — |
| IEM Cologne 2025 | 802 | 49 | 49 | +0.0254 | — |
| BLAST Bounty Fall 2025 | 851 | 7 | 7 | +0.0280 | — |
| Esports World Cup 2025 | 882 | 16 | 16 | +0.0076 | — |
| BLAST Open London Fall 2025 | 898 | 7 | 7 | -0.0174 | — |
| FISSURE PLAYGROUND Season 2 2025 | 927 | 40 | 40 | +0.0172 | — |
| ESL Pro League Season 22 2025 | 967 | 57 | 57 | +0.0017 | — |
| CS Asia Championships 2025 | 1040 | 30 | 30 | +0.0013 | — |
| Thunderpick World Championship 2025 | 1040 | 11 | 11 | +0.0105 | — |
| PGL Masters Bucharest 2025 | 1084 | 41 | 41 | -0.0013 | — |
| IEM Chengdu 2025 | 1125 | 27 | 27 | +0.0035 | — |
| BLAST Rivals Fall 2025 | 1155 | 13 | 13 | -0.0097 | — |
| StarLadder Major Budapest 2025 | 1168 | 106 | 106 | +0.0070 | — |
| BLAST Bounty Winter 2026 | 1274 | 7 | 7 | +0.0197 | — |
| IEM Krakow 2026 | 1305 | 49 | 49 | -0.0002 | — |
| PGL Cluj-Napoca 2026 | 1355 | 41 | 41 | -0.0005 | — |
| ESL Pro League Season 23 2026 | 1396 | 8 | 8 | -0.0030 | — |
| BLAST Open Rotterdam Spring 2026 | 1470 | 24 | 24 | +0.0039 | — |
| PGL Bucharest 2026 | 1499 | 33 | 33 | -0.0009 | — |
| IEM Rio 2026 | 1538 | 30 | 30 | -0.0022 | — |
| BLAST Rivals Spring 2026 | 1568 | 13 | 13 | +0.0058 | — |
| PGL Astana 2026 | 1581 | 41 | 41 | -0.0000 | — |
| IEM Atlanta 2026 | 1581 | 30 | 30 | +0.0047 | — |
| CS Asia Championships 2026 | 1652 | 30 | 30 | -0.0010 | — |
| IEM Cologne Major 2026 | 1682 | 99 | 99 | +0.0017 | — |
| XSE Pro League 2026 | 1788 | 37 | 37 | +0.0009 | — |
| BLAST Bounty Summer 2026 | 1828 | 7 | 7 | +0.0002 | — |
| Esports World Cup 2026 | 1859 | 56 | 56 | -0.0000 | — |
| BLAST Open Porto Fall 2026 | 1915 | 24 | 24 | -0.0009 | — |
| FISSURE PLAYGROUND Season 3 2026 | 1944 | 23 | 23 | -0.0030 | — |
| StarLadder StarSeries Fall 2026 | 1973 | 14 | 14 | -0.0011 | — |
| ESL Pro League Season 24 2026 | 1987 | 34 | 34 | -0.0044 | — |
| Thunderpick World Championship 2026 | 2028 | 14 | 14 | +0.0110 | — |
| PGL Masters Bucharest 2026 | 2042 | 37 | 37 | +0.0004 | — |
| IEM Beijing 2026 | 2083 | 27 | 27 | -0.0041 | — |
| BLAST Rivals Fall 2026 | 2113 | 13 | 13 | +0.0191 | — |

## `roster_coach_change`: no evidence

An emergency coach substitute or a change from the previously observed active coach set..

Verdict: the interval of the log-loss difference includes zero.

| On the 1806 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.7816 | 0.2392 | 65.8% |
| roster_coach_change | 0.7853 | 0.2399 | 65.6% |

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
| Perfect World Major Shanghai Major 2024 | 277 | 73 | 73 | +0.0000 | — |
| BLAST Premier Bounty season 1 2025 | 350 | 6 | 6 | +0.0000 | — |
| IEM Katowice 2025 | 357 | 47 | 47 | +0.0000 | — |
| PGL Cluj-Napoca 2025 | 406 | 41 | 38 | +0.0000 | — |
| ESL Pro League Season 21 2025 | 447 | 40 | 40 | +0.0000 | — |
| BLAST Open Lisbon Spring 2025 | 487 | 28 | 28 | +0.0000 | — |
| PGL Bucharest 2025 | 516 | 33 | 33 | +0.0000 | — |
| IEM Melbourne 2025 | 557 | 26 | 26 | -0.0002 | — |
| BLAST Rivals Spring 2025 | 586 | 13 | 13 | +0.0000 | — |
| PGL Astana 2025 | 599 | 33 | 33 | -0.0293 | — |
| IEM Dallas 2025 | 640 | 28 | 28 | +0.0000 | — |
| BLAST.tv Major Austin 2025 | 669 | 105 | 105 | -0.0149 | — |
| FISSURE PLAYGROUND Season 1 2025 | 775 | 27 | 27 | +0.0808 | — |
| IEM Cologne 2025 | 802 | 49 | 49 | +0.0252 | — |
| BLAST Bounty Fall 2025 | 851 | 7 | 7 | +0.0000 | — |
| Esports World Cup 2025 | 882 | 15 | 15 | +0.0000 | — |
| BLAST Open London Fall 2025 | 898 | 7 | 7 | +0.0000 | — |
| FISSURE PLAYGROUND Season 2 2025 | 927 | 40 | 40 | +0.0000 | — |
| ESL Pro League Season 22 2025 | 967 | 61 | 61 | +0.0165 | — |
| CS Asia Championships 2025 | 1040 | 29 | 29 | +0.0074 | — |
| Thunderpick World Championship 2025 | 1040 | 13 | 13 | -0.0000 | — |
| PGL Masters Bucharest 2025 | 1084 | 39 | 39 | +0.0018 | — |
| IEM Chengdu 2025 | 1125 | 30 | 30 | +0.0001 | — |
| BLAST Rivals Fall 2025 | 1155 | 13 | 13 | +0.0102 | — |
| StarLadder Major Budapest 2025 | 1168 | 100 | 100 | +0.0181 | — |
| BLAST Bounty Winter 2026 | 1274 | 7 | 7 | +0.0000 | — |
| IEM Krakow 2026 | 1305 | 50 | 50 | +0.0000 | — |
| PGL Cluj-Napoca 2026 | 1355 | 41 | 41 | +0.0000 | — |
| ESL Pro League Season 23 2026 | 1396 | 7 | 7 | +0.0000 | — |
| BLAST Open Rotterdam Spring 2026 | 1470 | 19 | 19 | -0.0111 | — |
| PGL Bucharest 2026 | 1499 | 37 | 37 | -0.0083 | — |
| IEM Rio 2026 | 1538 | 30 | 30 | +0.0049 | — |
| BLAST Rivals Spring 2026 | 1568 | 12 | 12 | -0.0297 | — |
| PGL Astana 2026 | 1581 | 36 | 36 | +0.0142 | — |
| IEM Atlanta 2026 | 1581 | 30 | 30 | -0.0075 | — |
| CS Asia Championships 2026 | 1652 | 30 | 30 | +0.0366 | — |
| IEM Cologne Major 2026 | 1682 | 96 | 96 | +0.0252 | — |
| XSE Pro League 2026 | 1788 | 37 | 36 | +0.0000 | — |
| BLAST Bounty Summer 2026 | 1828 | 6 | 6 | +0.0000 | — |
| Esports World Cup 2026 | 1859 | 50 | 50 | -0.0038 | — |
| BLAST Open Porto Fall 2026 | 1915 | 25 | 25 | +0.0000 | — |
| FISSURE PLAYGROUND Season 3 2026 | 1944 | 29 | 29 | +0.0000 | — |
| StarLadder StarSeries Fall 2026 | 1973 | 12 | 12 | +0.0000 | — |
| ESL Pro League Season 24 2026 | 1987 | 35 | 35 | +0.0000 | — |
| Thunderpick World Championship 2026 | 2028 | 14 | 14 | +0.0000 | — |
| PGL Masters Bucharest 2026 | 2042 | 41 | 41 | -0.0086 | — |
| IEM Beijing 2026 | 2083 | 30 | 30 | +0.0054 | — |
| BLAST Rivals Fall 2026 | 2113 | 12 | 12 | -0.0041 | — |

## `rest_schedule_load`: no evidence

Team differences in rest, Matches and maps in the preceding 48 hours, and a second Match on the same UTC day.

Verdict: the interval of the log-loss difference includes zero.

| On the 1988 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.7883 | 0.2399 | 65.6% |
| rest_schedule_load | 0.7889 | 0.2404 | 65.6% |

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
| Perfect World Major Shanghai Major 2024 | 277 | 73 | 73 | -0.0005 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.18297284313576986,0.17574366590256318,0.15972842900155496,0.23109145919710572]}` |
| BLAST Premier Bounty season 1 2025 | 350 | 7 | 7 | +0.0184 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.1589947240604247,0.19159339528180136,0.1453198296028801,0.2260569753893188]}` |
| IEM Katowice 2025 | 357 | 47 | 47 | +0.0108 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.1389341138844598,0.1822276005472611,0.13553979795581414,0.22421666050692904]}` |
| PGL Cluj-Napoca 2025 | 406 | 38 | 38 | -0.0127 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[-0.003660023907946744,0.13933502033473832,0.10979555235612203,0.20317422929847992]}` |
| ESL Pro League Season 21 2025 | 447 | 39 | 39 | -0.0059 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.01918018193997921,0.10385033161599537,0.24960452218076393,0.19830814650889456]}` |
| BLAST Open Lisbon Spring 2025 | 487 | 28 | 28 | -0.0115 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.012295465264876912,0.1397545257693562,0.22986239263924776,0.1943674375326869]}` |
| PGL Bucharest 2025 | 516 | 40 | 40 | -0.0019 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.015861120414012223,0.15458712854686185,0.2457567002374572,0.19164719135293773]}` |
| IEM Melbourne 2025 | 557 | 29 | 29 | +0.0015 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.2398012217348253,0.1723110393682649,0.336410262883247,0.22567069174387142]}` |
| BLAST Rivals Spring 2025 | 586 | 13 | 13 | +0.0581 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.2085546876560036,0.19000111210635107,0.3204506019241287,0.2191609587860511]}` |
| PGL Astana 2025 | 599 | 39 | 39 | -0.0042 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.18179549969779413,0.15848317169096285,0.279827353875231,0.22156279347467042]}` |
| IEM Dallas 2025 | 640 | 28 | 28 | +0.0032 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.19015670013957472,0.13769986318071942,0.3103136956722343,0.2227116028465313]}` |
| BLAST.tv Major Austin 2025 | 669 | 105 | 105 | +0.0184 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.23805240180614104,0.10059117540204758,0.30395188022756764,0.3058743023822474]}` |
| FISSURE PLAYGROUND Season 1 2025 | 775 | 27 | 27 | -0.0127 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[-0.009743684019240077,0.16022232862410068,0.2270953750872989,0.15814137609506515]}` |
| IEM Cologne 2025 | 802 | 49 | 49 | +0.0043 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.02521179695784476,0.16816010902050377,0.26508060407185075,0.16071353164359994]}` |
| BLAST Bounty Fall 2025 | 851 | 28 | 28 | +0.0001 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.13710598743929955,0.15629274349999925,0.31778644964034336,0.1792595657424986]}` |
| Esports World Cup 2025 | 882 | 16 | 16 | -0.0342 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.13980217574333575,0.15702465700241564,0.31474729508180027,0.17997121851785675]}` |
| BLAST Open London Fall 2025 | 898 | 29 | 29 | -0.0183 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.18322499434401707,0.19616004730237524,0.3176663599325233,0.18358945633813845]}` |
| FISSURE PLAYGROUND Season 2 2025 | 927 | 40 | 40 | +0.0191 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.18586158415537246,0.20982610402733573,0.3399850193718466,0.18056521633292164]}` |
| ESL Pro League Season 22 2025 | 967 | 71 | 71 | -0.0136 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.1271208754459218,0.24900180779595685,0.24370621381575583,0.17369262869147942]}` |
| CS Asia Championships 2025 | 1040 | 30 | 30 | -0.0046 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.15607054820449617,0.23404783196647796,0.30965238914848053,0.17493041089337424]}` |
| Thunderpick World Championship 2025 | 1040 | 13 | 13 | +0.0250 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.15607054820449617,0.23404783196647796,0.30965238914848053,0.17493041089337424]}` |
| PGL Masters Bucharest 2025 | 1084 | 41 | 41 | -0.0180 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.18744350328300025,0.2465435172478958,0.27585394814028014,0.1825857526341306]}` |
| IEM Chengdu 2025 | 1125 | 30 | 30 | +0.0104 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.17368582261597226,0.22957607476777628,0.32094659352577487,0.17778234253895872]}` |
| BLAST Rivals Fall 2025 | 1155 | 13 | 13 | +0.0071 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.15666090701357993,0.2230244592140937,0.30647731961170677,0.17657013261191587]}` |
| StarLadder Major Budapest 2025 | 1168 | 106 | 106 | +0.0118 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.19027244942848398,0.23191113130121663,0.2862662425051977,0.18379348439333842]}` |
| BLAST Bounty Winter 2026 | 1274 | 27 | 27 | -0.0153 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.11604538090089671,0.20301909341695146,0.25124105831760535,0.17604278835729054]}` |
| IEM Krakow 2026 | 1305 | 50 | 50 | -0.0039 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.12127647555516896,0.21613609249017993,0.2571291449331874,0.17503428039676946]}` |
| PGL Cluj-Napoca 2026 | 1355 | 41 | 41 | -0.0102 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.09773385308584066,0.2282782721375575,0.23844998842155737,0.170780396016687]}` |
| ESL Pro League Season 23 2026 | 1396 | 72 | 72 | -0.0056 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.09059609498085801,0.20619844816912222,0.27097182822593574,0.1688660223595149]}` |
| BLAST Open Rotterdam Spring 2026 | 1470 | 29 | 29 | +0.0106 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.09062160035900792,0.23929652173748567,0.23511384560292986,0.16842736914345802]}` |
| PGL Bucharest 2026 | 1499 | 37 | 37 | +0.0221 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.08941248608760377,0.24769400719614484,0.21049943434736937,0.16952220906037335]}` |
| IEM Rio 2026 | 1538 | 30 | 30 | -0.0234 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.05520276317537761,0.19204556924177923,0.22122406057370486,0.16811863764196527]}` |
| BLAST Rivals Spring 2026 | 1568 | 13 | 13 | -0.0121 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.05976856486780131,0.2265320669577508,0.21244019200478395,0.16587728222928635]}` |
| PGL Astana 2026 | 1581 | 38 | 38 | +0.0230 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.07363744055873771,0.24549513199222397,0.2017755405081723,0.16746048223538754]}` |
| IEM Atlanta 2026 | 1581 | 30 | 30 | -0.0267 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.07363744055873771,0.24549513199222397,0.2017755405081723,0.16746048223538754]}` |
| CS Asia Championships 2026 | 1652 | 30 | 30 | +0.0127 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.06650308291707879,0.19884335062190694,0.24423168547203714,0.14219882868842673]}` |
| IEM Cologne Major 2026 | 1682 | 104 | 104 | -0.0059 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.07570386448558851,0.2085758055058742,0.21231383958564973,0.1622322076519601]}` |
| XSE Pro League 2026 | 1788 | 36 | 36 | +0.0035 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.09480304021201771,0.1919848420635454,0.24337019113177988,0.16561146047699063]}` |
| BLAST Bounty Summer 2026 | 1828 | 28 | 28 | -0.0231 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.09184933726711587,0.17917013359404563,0.2501018660281465,0.16579486603827057]}` |
| Esports World Cup 2026 | 1859 | 55 | 55 | +0.0070 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.10391163352867147,0.19495337124886764,0.25766303436847576,0.16606779757253604]}` |
| BLAST Open Porto Fall 2026 | 1915 | 29 | 29 | +0.0070 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.09228700508491203,0.1865574966484533,0.2490808460916298,0.16516701431301098]}` |
| FISSURE PLAYGROUND Season 3 2026 | 1944 | 28 | 28 | +0.0071 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.11625372617948385,0.1854168846634676,0.24513701567231408,0.1708645885946145]}` |
| StarLadder StarSeries Fall 2026 | 1973 | 14 | 14 | -0.0295 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.12780855643738098,0.16847264418112115,0.25138217296501814,0.1746712135846579]}` |
| ESL Pro League Season 24 2026 | 1987 | 39 | 39 | +0.0168 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.1399205962671759,0.18076923129499403,0.2507029698506427,0.18896646049081897]}` |
| Thunderpick World Championship 2026 | 2028 | 14 | 14 | -0.0413 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.13076723226510453,0.15797069765424696,0.24627402450117394,0.1898689782304039]}` |
| PGL Masters Bucharest 2026 | 2042 | 41 | 41 | -0.0135 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.13548246381649315,0.17333490306434826,0.2483239561152517,0.18900080585992723]}` |
| IEM Beijing 2026 | 2083 | 30 | 30 | +0.0245 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.1602161281293137,0.19756650600756506,0.23146588698314247,0.1935079754029177]}` |
| BLAST Rivals Fall 2026 | 2113 | 13 | 13 | +0.0080 | `{"features":["log_rest_hours","matches_last_48h","maps_last_48h","second_match_today"],"weights":[0.15358788869484638,0.20419133356207209,0.19443926418342272,0.19447138325352667]}` |

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
| Perfect World Major Shanghai Major 2024 | 277 | 73 | 73 | -0.0032 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[0.0,0.0,0.0,0.0,0.0,-0.32532432508709347,0.04758431025595944,-0.04563306851353747]}` |
| BLAST Premier Bounty season 1 2025 | 350 | 7 | 7 | -0.2874 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[0.03503880224436398,-0.2103220480939539,0.09623032087440926,-0.16453732433673943,0.0,-0.3654857929619832,-0.1433603392902976,-0.02220885477866251]}` |
| IEM Katowice 2025 | 357 | 5 | 5 | +0.0540 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[0.03503880224436398,-0.2103220480939539,0.09623032087440926,-0.16453732433673943,0.0,-0.48251226006529785,-0.433873920398032,-0.021812517287953102]}` |
| PGL Cluj-Napoca 2025 | 406 | 1 | 1 | +0.0044 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[0.03503880224435155,-0.21032204809400631,0.0962303208744153,-0.1645373243367784,0.0,-0.4523143207242036,-0.4249888999833491,-0.12454918954785009]}` |
| ESL Pro League Season 21 2025 | 447 | 7 | 7 | +0.0583 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[0.03503880224436398,-0.2103220480939539,0.09623032087440926,-0.16453732433673943,0.0,-0.4523143207242029,-0.42498889998334904,-0.12180304722142438]}` |
| BLAST Open Lisbon Spring 2025 | 487 | 5 | 5 | +0.0645 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[0.03503880224436398,-0.2103220480939539,0.09623032087440926,-0.16453732433673943,0.0,-0.4035649296736144,-0.43621096235158097,-0.10314768055312602]}` |
| PGL Bucharest 2025 | 516 | 1 | 1 | -0.0159 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[0.03503880224436398,-0.2103220480939539,0.09623032087440926,-0.16453732433673943,0.0,-0.37765106145746735,-0.4308084598874604,-0.08468543717727874]}` |
| IEM Melbourne 2025 | 557 | 5 | 5 | -0.0673 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[0.03503880224435155,-0.21032204809400631,0.0962303208744153,-0.1645373243367784,0.0,-0.3776510614574674,-0.4308084598874605,-0.0990126762136022]}` |
| BLAST Rivals Spring 2025 | 586 | 5 | 5 | -0.1016 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[0.03503880224436398,-0.2103220480939539,0.09623032087440926,-0.16453732433673943,0.0,-0.34831620841129163,-0.47314815436529084,-0.0933568807719791]}` |
| PGL Astana 2025 | 599 | 7 | 7 | -0.2699 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[0.03503880224435155,-0.21032204809400631,0.0962303208744153,-0.1645373243367784,0.0,-0.33266700644749697,-0.5110316653749952,-0.08795953455509377]}` |
| IEM Dallas 2025 | 640 | 1 | 1 | +0.0215 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[0.0350388022443392,-0.2103220480940583,0.09623032087442131,-0.16453732433681706,0.0,-0.38306372820220014,-0.572769365827638,-0.08507740197927216]}` |
| BLAST.tv Major Austin 2025 | 669 | 106 | 106 | -0.0648 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[0.03503880224433921,-0.21032204809405827,0.09623032087442131,-0.16453732433681703,0.0,-0.38306372820220014,-0.5727693658275643,-0.06599145788109638]}` |
| FISSURE PLAYGROUND Season 1 2025 | 775 | 7 | 7 | -0.3301 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.3059621315173986,-0.25823932516125714,0.2124915795189116,0.009474477058342189,0.0,-0.4020776088603537,-0.5458122691021188,-0.05407541877453726]}` |
| IEM Cologne 2025 | 802 | 5 | 5 | -0.1698 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.30596213151739854,-0.2582393251612572,0.21249157951891165,0.009474477058342323,0.0,-0.5346166080446845,-0.5155403349947727,-0.14353011488578898]}` |
| BLAST Bounty Fall 2025 | 851 | 1 | 1 | +0.0400 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.3059621315173986,-0.258239325161257,0.21249157951891157,0.009474477058342059,0.0,-0.5399665951831496,-0.5314337412454749,-0.1526479026812769]}` |
| Esports World Cup 2025 | 882 | 7 | 7 | -0.1999 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.30596213151739854,-0.2582393251612572,0.21249157951891165,0.009474477058342323,0.0,-0.5399665951831115,-0.5314337412453976,-0.13443425847082055]}` |
| BLAST Open London Fall 2025 | 898 | 1 | 1 | -0.6556 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.30596213151739854,-0.2582393251612572,0.21249157951891165,0.009474477058342323,0.0,-0.5372795182248301,-0.5726144112208376,-0.11607075380874125]}` |
| FISSURE PLAYGROUND Season 2 2025 | 927 | 1 | 1 | -1.2769 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.30596213151739854,-0.2582393251612572,0.21249157951891165,0.009474477058342323,0.0,-0.5372795182248301,-0.5726144112208376,-0.40366793881098423]}` |
| ESL Pro League Season 22 2025 | 967 | 7 | 7 | -0.0307 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.30596213151739854,-0.2582393251612572,0.21249157951891165,0.009474477058342323,0.0,-0.5372795182248301,-0.5726144112208376,-0.5148920927044875]}` |
| CS Asia Championships 2025 | 1040 | 5 | 5 | -0.1543 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.3059621315173986,-0.258239325161257,0.21249157951891157,0.009474477058342064,0.0,-0.549907841565903,-0.559438780934567,-0.5060362357310508]}` |
| Thunderpick World Championship 2025 | 1040 | 3 | 3 | -0.1411 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.3059621315173986,-0.258239325161257,0.21249157951891157,0.009474477058342064,0.0,-0.549907841565903,-0.559438780934567,-0.5060362357310508]}` |
| PGL Masters Bucharest 2025 | 1084 | 7 | 7 | -0.1028 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.30596213151739854,-0.2582393251612572,0.21249157951891165,0.009474477058342323,0.0,-0.5326295815294777,-0.5769012489525773,-0.5587307507576015]}` |
| IEM Chengdu 2025 | 1125 | 5 | 5 | +0.1552 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.30596213151739854,-0.2582393251612572,0.21249157951891165,0.009474477058342323,0.0,-0.5522195995731833,-0.5653662080199213,-0.5447861595165595]}` |
| BLAST Rivals Fall 2025 | 1155 | 5 | 5 | +0.1220 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.30596213151739854,-0.2582393251612572,0.21249157951891165,0.009474477058342323,0.0,-0.5378649762799463,-0.5514328566284251,-0.5393735584302305]}` |
| StarLadder Major Budapest 2025 | 1168 | 106 | 106 | -0.0688 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.30596213151739854,-0.2582393251612572,0.21249157951891165,0.009474477058342323,0.0,-0.522710298362008,-0.542679457193563,-0.5296470521805143]}` |
| BLAST Bounty Winter 2026 | 1274 | 7 | 7 | -0.1680 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.2994936079984075,-0.2980324611775121,0.308087420300672,0.07812472282021708,0.0,-0.5877087552053268,-0.5487849807746119,-0.5148297863069498]}` |
| IEM Krakow 2026 | 1305 | 5 | 5 | +0.1267 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.2994936079984075,-0.2980324611775121,0.308087420300672,0.07812472282021707,0.0,-0.576929123666827,-0.5749411162647404,-0.5358738407062171]}` |
| PGL Cluj-Napoca 2026 | 1355 | 7 | 7 | -0.0683 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.2994936079984076,-0.29803246117751203,0.3080874203006721,0.07812472282021703,0.0,-0.5687687694529183,-0.5673204244948742,-0.5294671291740533]}` |
| ESL Pro League Season 23 2026 | 1396 | 7 | 7 | -0.0964 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.2994936079984076,-0.29803246117751203,0.3080874203006721,0.07812472282021703,0.0,-0.574161109943947,-0.5582978746624859,-0.5417212001454004]}` |
| BLAST Open Rotterdam Spring 2026 | 1470 | 5 | 5 | +0.0612 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.2994936079984075,-0.2980324611775121,0.30808742030067193,0.0781247228202171,0.0,-0.5907371584720259,-0.5518512903167458,-0.5380446068369009]}` |
| PGL Bucharest 2026 | 1499 | 7 | 7 | +0.0893 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.2994936079984076,-0.29803246117751203,0.3080874203006721,0.07812472282021703,0.0,-0.5924563250224286,-0.5418126896225831,-0.522765064504315]}` |
| IEM Rio 2026 | 1538 | 5 | 5 | +0.1086 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.2994936079984076,-0.29803246117751203,0.3080874203006721,0.07812472282021703,0.0,-0.5798847072333368,-0.5348844728288247,-0.5225995820906056]}` |
| BLAST Rivals Spring 2026 | 1568 | 5 | 5 | -0.0369 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.2994936079984075,-0.2980324611775121,0.30808742030067193,0.0781247228202171,0.0,-0.573118884768756,-0.5297749408502058,-0.5085767626626335]}` |
| PGL Astana 2026 | 1581 | 7 | 7 | +0.0439 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.2994936079984076,-0.29803246117751203,0.3080874203006721,0.07812472282021703,0.0,-0.5877430681547502,-0.5081853798076358,-0.4957278813743653]}` |
| IEM Atlanta 2026 | 1581 | 5 | 5 | -0.0146 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.2994936079984076,-0.29803246117751203,0.3080874203006721,0.07812472282021703,0.0,-0.5877430681547502,-0.5081853798076358,-0.4957278813743653]}` |
| CS Asia Championships 2026 | 1652 | 5 | 5 | -0.3327 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.2994936079984075,-0.2980324611775121,0.30808742030067193,0.0781247228202171,0.0,-0.5950846399526829,-0.48289890097289,-0.4768678555910857]}` |
| IEM Cologne Major 2026 | 1682 | 106 | 106 | +0.0280 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.2994936079984075,-0.2980324611775121,0.30808742030067193,0.0781247228202171,0.0,-0.6046495067984308,-0.4673766470597996,-0.5387801807213511]}` |
| XSE Pro League 2026 | 1788 | 7 | 7 | -0.0421 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.28657340459345465,-0.25765080030199145,0.3300598606711892,0.015904913268416695,0.0,-0.5969563991109239,-0.4691342737203149,-0.5311298647674707]}` |
| BLAST Bounty Summer 2026 | 1828 | 7 | 7 | +0.0355 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.28657340457919633,-0.25765080030452997,0.33005986074489346,0.015904913302059648,0.0,-0.6037236069760604,-0.4531378601375986,-0.5172414371294245]}` |
| Esports World Cup 2026 | 1859 | 7 | 7 | -0.0522 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.28657340457919633,-0.2576508003045298,0.33005986074489346,0.015904913302059422,0.0,-0.5876753093570302,-0.43536418284390255,-0.5428747774535698]}` |
| BLAST Open Porto Fall 2026 | 1915 | 5 | 5 | +0.0925 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.28657340459345465,-0.25765080030199145,0.3300598606711892,0.015904913268416695,0.0,-0.5971820327677083,-0.4333170211312557,-0.5307284471281036]}` |
| FISSURE PLAYGROUND Season 3 2026 | 1944 | 5 | 5 | +0.0532 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.28657340459345465,-0.25765080030199145,0.3300598606711892,0.015904913268416695,0.0,-0.5908132428712329,-0.4325640235067476,-0.5236410598616889]}` |
| StarLadder StarSeries Fall 2026 | 1973 | 1 | 1 | +0.1948 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.28657340459345465,-0.25765080030199145,0.3300598606711892,0.015904913268416695,0.0,-0.5911734273681546,-0.4175567582870821,-0.5227425604084948]}` |
| ESL Pro League Season 24 2026 | 1987 | 7 | 7 | +0.1153 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.28657340457919633,-0.25765080030452997,0.33005986074489346,0.015904913302059648,0.0,-0.5911734273681546,-0.4175567582870821,-0.5115116442321234]}` |
| Thunderpick World Championship 2026 | 2028 | 3 | 3 | -0.0394 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.28657340459345465,-0.25765080030199145,0.3300598606711892,0.015904913268416695,0.0,-0.5743581462737067,-0.41569961876424144,-0.5056365468596244]}` |
| PGL Masters Bucharest 2026 | 2042 | 7 | 7 | -0.0648 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.28657340457919633,-0.25765080030452997,0.33005986074489346,0.015904913302059648,0.0,-0.5743581462737067,-0.4291362919672306,-0.4949606910348009]}` |
| IEM Beijing 2026 | 2083 | 5 | 5 | +0.0024 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.28657340459345465,-0.25765080030199145,0.3300598606711892,0.015904913268416695,0.0,-0.5686044435197143,-0.46057404925277,-0.48419557660361395]}` |
| BLAST Rivals Fall 2026 | 2113 | 5 | 5 | +0.0144 | `{"features":["wins","losses","advancement","elimination","round_of_16","quarterfinal","semifinal","final"],"weights":[-0.2865734045934826,-0.2576508003019865,0.33005986067104465,0.015904913268350727,0.0,-0.5697765306912641,-0.4615320187283631,-0.47144547563181527]}` |

## `lan_online`: promote

Team win rates in the current environment, with separate weights for LAN and online.

Verdict: the interval of the log-loss improvement excludes zero and the Matches with a market line are not worse.

| On the 1952 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.7840 | 0.2397 | 65.5% |
| lan_online | 0.7795 | 0.2395 | 65.2% |

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
| Perfect World Major Shanghai Major 2024 | 277 | 72 | 72 | -0.0023 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.2659815496040439,0.0]}` |
| BLAST Premier Bounty season 1 2025 | 350 | 7 | 7 | -0.0172 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.32174208266835813,0.0]}` |
| IEM Katowice 2025 | 357 | 47 | 47 | -0.0021 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.3564238261153847,0.0]}` |
| PGL Cluj-Napoca 2025 | 406 | 38 | 38 | -0.0023 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.38097685467331127,0.0]}` |
| ESL Pro League Season 21 2025 | 447 | 39 | 39 | +0.0025 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.40061571096331344,0.0]}` |
| BLAST Open Lisbon Spring 2025 | 487 | 28 | 28 | +0.0032 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.3771961707148745,0.0]}` |
| PGL Bucharest 2025 | 516 | 40 | 40 | -0.0134 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.35455779254637815,0.0]}` |
| IEM Melbourne 2025 | 557 | 29 | 29 | -0.0090 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.4896755470748957,0.0]}` |
| BLAST Rivals Spring 2025 | 586 | 13 | 13 | -0.0139 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.536362679556172,0.0]}` |
| PGL Astana 2025 | 599 | 39 | 39 | -0.0136 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.5658016257560967,0.0]}` |
| IEM Dallas 2025 | 640 | 28 | 28 | +0.0051 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.6486212912809162,0.0]}` |
| BLAST.tv Major Austin 2025 | 669 | 104 | 104 | -0.0062 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.62709830159814,0.0]}` |
| FISSURE PLAYGROUND Season 1 2025 | 775 | 27 | 27 | -0.0190 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.7085432209274929,0.0]}` |
| IEM Cologne 2025 | 802 | 49 | 49 | +0.0034 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.7704474196407526,0.0]}` |
| BLAST Bounty Fall 2025 | 851 | 28 | 28 | -0.0075 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.7490410564227001,0.0]}` |
| Esports World Cup 2025 | 882 | 16 | 16 | -0.0546 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.7720919889506113,0.0]}` |
| BLAST Open London Fall 2025 | 898 | 28 | 28 | -0.0265 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.867176378027621,0.0]}` |
| FISSURE PLAYGROUND Season 2 2025 | 927 | 40 | 40 | -0.0026 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.9379379098465321,0.0]}` |
| ESL Pro League Season 22 2025 | 967 | 71 | 71 | -0.0043 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.9456146658466654,0.0]}` |
| CS Asia Championships 2025 | 1040 | 29 | 29 | -0.0013 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.9634943008345912,0.0]}` |
| Thunderpick World Championship 2025 | 1040 | 13 | 13 | +0.0048 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.9634943008345912,0.0]}` |
| PGL Masters Bucharest 2025 | 1084 | 41 | 41 | -0.0097 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.9584735346964439,0.0]}` |
| IEM Chengdu 2025 | 1125 | 30 | 30 | -0.0204 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-0.9918163640757359,0.0]}` |
| BLAST Rivals Fall 2025 | 1155 | 13 | 13 | -0.0223 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-1.0384011831392577,0.0]}` |
| StarLadder Major Budapest 2025 | 1168 | 106 | 106 | -0.0120 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-1.0592173303984855,0.0]}` |
| BLAST Bounty Winter 2026 | 1274 | 27 | 27 | -0.0179 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-1.1431121930501293,0.0]}` |
| IEM Krakow 2026 | 1305 | 50 | 50 | -0.0013 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-1.1728310249101623,0.0]}` |
| PGL Cluj-Napoca 2026 | 1355 | 41 | 41 | +0.0201 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-1.1703608572609006,0.0]}` |
| ESL Pro League Season 23 2026 | 1396 | 72 | 72 | -0.0061 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-1.1106189644324638,0.0]}` |
| BLAST Open Rotterdam Spring 2026 | 1470 | 29 | 29 | -0.0120 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-1.1357232482466288,0.0]}` |
| PGL Bucharest 2026 | 1499 | 37 | 37 | +0.0118 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-1.1526434865587338,0.0]}` |
| IEM Rio 2026 | 1538 | 30 | 30 | +0.0096 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-1.120426982897163,0.0]}` |
| BLAST Rivals Spring 2026 | 1568 | 13 | 13 | -0.0086 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-1.0968002757393005,0.0]}` |
| PGL Astana 2026 | 1581 | 38 | 38 | +0.0070 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-1.10261601879145,0.0]}` |
| IEM Atlanta 2026 | 1581 | 30 | 30 | -0.0208 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-1.10261601879145,0.0]}` |
| CS Asia Championships 2026 | 1652 | 30 | 30 | -0.0153 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-1.1168434688279838,0.0]}` |
| IEM Cologne Major 2026 | 1682 | 104 | 104 | +0.0012 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-1.1446009316363441,0.0]}` |
| XSE Pro League 2026 | 1788 | 36 | 36 | +0.0128 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-1.1253146113742158,0.0]}` |
| BLAST Bounty Summer 2026 | 1828 | 28 | 28 | -0.0234 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-1.0932863143285938,0.0]}` |
| Esports World Cup 2026 | 1859 | 54 | 54 | +0.0175 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-1.131800237912343,0.0]}` |
| BLAST Open Porto Fall 2026 | 1915 | 28 | 28 | +0.0097 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-1.0639763158104656,0.0]}` |
| FISSURE PLAYGROUND Season 3 2026 | 1944 | 28 | 28 | -0.0038 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-1.0410178439450808,0.0]}` |
| StarLadder StarSeries Fall 2026 | 1973 | 14 | 14 | -0.0199 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-1.0457824269395766,0.0]}` |
| ESL Pro League Season 24 2026 | 1987 | 39 | 39 | -0.0123 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-1.0607574356626823,0.0]}` |
| Thunderpick World Championship 2026 | 2028 | 14 | 14 | +0.0076 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-1.0852783092385365,0.0]}` |
| PGL Masters Bucharest 2026 | 2042 | 41 | 41 | -0.0237 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-1.0779980878148339,0.0]}` |
| IEM Beijing 2026 | 2083 | 30 | 30 | +0.0037 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-1.1306905357839692,0.0]}` |
| BLAST Rivals Fall 2026 | 2113 | 13 | 13 | -0.0072 | `{"features":["lan_win_rate","online_win_rate"],"weights":[-1.1227990026969936,0.0]}` |

## `series_format`: no evidence

Team differences in Bo1 versus Bo3 performance, conditioned on the current format.

Verdict: the interval of the log-loss difference includes zero.

| On the 1614 paired Matches | Log-loss | Brier | Accuracy |
|---|---|---|---|
| anchor | 0.7483 | 0.2382 | 65.4% |
| series_format | 0.7487 | 0.2384 | 65.1% |

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
| Perfect World Major Shanghai Major 2024 | 277 | 71 | 71 | +0.0008 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.0,-0.13600704621152118]}` |
| BLAST Premier Bounty season 1 2025 | 350 | 6 | 6 | +0.0132 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.01847243900677671,-0.09402769171478843]}` |
| IEM Katowice 2025 | 357 | 46 | 46 | +0.0004 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.018472439006776704,-0.015179346207447132]}` |
| PGL Cluj-Napoca 2025 | 406 | 37 | 37 | -0.0017 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.018472439006776704,0.10712500782743974]}` |
| ESL Pro League Season 21 2025 | 447 | 36 | 36 | +0.0072 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.01847243900677671,0.15798591834544928]}` |
| BLAST Open Lisbon Spring 2025 | 487 | 26 | 26 | -0.0001 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.018472439006776704,0.014882646247034201]}` |
| PGL Bucharest 2025 | 516 | 35 | 35 | -0.0005 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.018472439006776715,0.028116877335143214]}` |
| IEM Melbourne 2025 | 557 | 28 | 28 | +0.0013 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.018472439006776704,0.08228294973556002]}` |
| BLAST Rivals Spring 2025 | 586 | 12 | 12 | -0.0008 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.01847243900677671,0.04547506858261833]}` |
| PGL Astana 2025 | 599 | 31 | 31 | +0.0027 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.018472439006776704,0.0638563437635414]}` |
| IEM Dallas 2025 | 640 | 24 | 24 | -0.0012 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.01847243900677671,-0.04587621628997773]}` |
| BLAST.tv Major Austin 2025 | 669 | 100 | 100 | +0.0003 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.018472439006776715,-0.09818260080648519]}` |
| FISSURE PLAYGROUND Season 1 2025 | 775 | 26 | 26 | +0.0022 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[-0.06422436895025253,-0.08690186587963841]}` |
| IEM Cologne 2025 | 802 | 48 | 48 | -0.0004 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[-0.06422436895025253,-0.03451756614845573]}` |
| BLAST Bounty Fall 2025 | 851 | 27 | 27 | -0.0009 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[-0.06422436895025253,-0.07733278906914645]}` |
| Esports World Cup 2025 | 882 | 15 | 15 | +0.0005 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[-0.06422436895025253,-0.10171028782452819]}` |
| BLAST Open London Fall 2025 | 898 | 26 | 26 | -0.0041 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[-0.06422436895025256,-0.09603184966034185]}` |
| FISSURE PLAYGROUND Season 2 2025 | 927 | 39 | 39 | -0.0014 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[-0.06422436895025256,-0.1771796031050179]}` |
| ESL Pro League Season 22 2025 | 967 | 48 | 48 | +0.0042 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[-0.06422436895025253,-0.1984851406509926]}` |
| CS Asia Championships 2025 | 1040 | 27 | 27 | -0.0041 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[-0.06422436895025256,-0.12476838888586465]}` |
| Thunderpick World Championship 2025 | 1040 | 10 | 10 | -0.0051 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[-0.06422436895025256,-0.12476838888586465]}` |
| PGL Masters Bucharest 2025 | 1084 | 35 | 35 | -0.0120 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[-0.03752066568611742,-0.22580119524515524]}` |
| IEM Chengdu 2025 | 1125 | 29 | 29 | +0.0046 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[-0.03752066568611745,-0.35118319133249193]}` |
| BLAST Rivals Fall 2025 | 1155 | 12 | 12 | +0.0045 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[-0.037520665686117435,-0.3235479396894129]}` |
| StarLadder Major Budapest 2025 | 1168 | 105 | 105 | -0.0003 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[-0.037520665686117435,-0.311208385724264]}` |
| BLAST Bounty Winter 2026 | 1274 | 18 | 18 | +0.0227 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.11125012762234396,-0.32663016790854843]}` |
| IEM Krakow 2026 | 1305 | 49 | 49 | -0.0022 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.11125012762234396,-0.24316328658394853]}` |
| PGL Cluj-Napoca 2026 | 1355 | 40 | 40 | +0.0046 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.11125012762234396,-0.26489871432325807]}` |
| ESL Pro League Season 23 2026 | 1396 | 59 | 59 | +0.0017 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.11125012762393063,-0.21894246196464237]}` |
| BLAST Open Rotterdam Spring 2026 | 1470 | 28 | 28 | -0.0022 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.11125012762388208,-0.18893329969924355]}` |
| PGL Bucharest 2026 | 1499 | 22 | 22 | -0.0020 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.11125012762234396,-0.20429739397810573]}` |
| IEM Rio 2026 | 1538 | 24 | 24 | +0.0022 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.11125012762234396,-0.21439925941945384]}` |
| BLAST Rivals Spring 2026 | 1568 | 12 | 12 | +0.0064 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.11125012762234396,-0.19997352552717237]}` |
| PGL Astana 2026 | 1581 | 20 | 20 | -0.0102 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.11125012762234396,-0.1783801353322988]}` |
| IEM Atlanta 2026 | 1581 | 29 | 29 | +0.0059 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.11125012762234396,-0.1783801353322988]}` |
| CS Asia Championships 2026 | 1652 | 29 | 29 | -0.0006 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.11125012762272632,-0.1851532541603939]}` |
| IEM Cologne Major 2026 | 1682 | 95 | 95 | -0.0013 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.12479484581062765,-0.18489398815735067]}` |
| XSE Pro League 2026 | 1788 | 29 | 29 | +0.0008 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.15089125763760455,-0.2026944373523219]}` |
| BLAST Bounty Summer 2026 | 1828 | 19 | 19 | +0.0075 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.16682210958662727,-0.18929140622977153]}` |
| Esports World Cup 2026 | 1859 | 50 | 50 | -0.0012 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.16682210958662727,-0.15071366364325825]}` |
| BLAST Open Porto Fall 2026 | 1915 | 23 | 23 | +0.0013 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.18145195933014957,-0.1595241361937022]}` |
| FISSURE PLAYGROUND Season 3 2026 | 1944 | 24 | 24 | +0.0008 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.18145195933014963,-0.14942645191591117]}` |
| StarLadder StarSeries Fall 2026 | 1973 | 13 | 13 | -0.0004 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.18145195933014957,-0.14240993676116898]}` |
| ESL Pro League Season 24 2026 | 1987 | 32 | 32 | +0.0048 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.18145195933014957,-0.14358794711035744]}` |
| Thunderpick World Championship 2026 | 2028 | 13 | 13 | -0.0041 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.18145195933014957,-0.09233544213558911]}` |
| PGL Masters Bucharest 2026 | 2042 | 36 | 36 | +0.0006 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.18145195941823322,-0.11826347302137491]}` |
| IEM Beijing 2026 | 2083 | 29 | 29 | -0.0008 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.18145195956545576,-0.10913107999351336]}` |
| BLAST Rivals Fall 2026 | 2113 | 12 | 12 | -0.0017 | `{"features":["bo1_relative_form","bo3_relative_form"],"weights":[0.18145195956539828,-0.11836692696731352]}` |

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
| Perfect World Major Shanghai Major 2024 | 277 | 0 | 0 | — | — |
| BLAST Premier Bounty season 1 2025 | 350 | 0 | 0 | — | — |
| IEM Katowice 2025 | 357 | 0 | 0 | — | — |
| PGL Cluj-Napoca 2025 | 406 | 0 | 0 | — | — |
| ESL Pro League Season 21 2025 | 447 | 0 | 0 | — | — |
| BLAST Open Lisbon Spring 2025 | 487 | 0 | 0 | — | — |
| PGL Bucharest 2025 | 516 | 0 | 0 | — | — |
| IEM Melbourne 2025 | 557 | 0 | 0 | — | — |
| BLAST Rivals Spring 2025 | 586 | 0 | 0 | — | — |
| PGL Astana 2025 | 599 | 0 | 0 | — | — |
| IEM Dallas 2025 | 640 | 0 | 0 | — | — |
| BLAST.tv Major Austin 2025 | 669 | 0 | 0 | — | — |
| FISSURE PLAYGROUND Season 1 2025 | 775 | 0 | 0 | — | — |
| IEM Cologne 2025 | 802 | 0 | 0 | — | — |
| BLAST Bounty Fall 2025 | 851 | 0 | 0 | — | — |
| Esports World Cup 2025 | 882 | 0 | 0 | — | — |
| BLAST Open London Fall 2025 | 898 | 0 | 0 | — | — |
| FISSURE PLAYGROUND Season 2 2025 | 927 | 0 | 0 | — | — |
| ESL Pro League Season 22 2025 | 967 | 0 | 0 | — | — |
| CS Asia Championships 2025 | 1040 | 0 | 0 | — | — |
| Thunderpick World Championship 2025 | 1040 | 0 | 0 | — | — |
| PGL Masters Bucharest 2025 | 1084 | 0 | 0 | — | — |
| IEM Chengdu 2025 | 1125 | 0 | 0 | — | — |
| BLAST Rivals Fall 2025 | 1155 | 0 | 0 | — | — |
| StarLadder Major Budapest 2025 | 1168 | 0 | 0 | — | — |
| BLAST Bounty Winter 2026 | 1274 | 0 | 0 | — | `{"features":["away_region"],"weights":[0.2231117416435859]}` |
| IEM Krakow 2026 | 1305 | 0 | 0 | — | `{"features":["away_region"],"weights":[0.2231117416435859]}` |
| PGL Cluj-Napoca 2026 | 1355 | 0 | 0 | — | `{"features":["away_region"],"weights":[0.2231117416435859]}` |
| ESL Pro League Season 23 2026 | 1396 | 0 | 0 | — | `{"features":["away_region"],"weights":[0.2231117416435859]}` |
| BLAST Open Rotterdam Spring 2026 | 1470 | 0 | 0 | — | `{"features":["away_region"],"weights":[0.2231117416435859]}` |
| PGL Bucharest 2026 | 1499 | 0 | 0 | — | `{"features":["away_region"],"weights":[0.2231117416435859]}` |
| IEM Rio 2026 | 1538 | 1 | 1 | +0.0000 | `{"features":["away_region"],"weights":[0.2231117416435859]}` |
| BLAST Rivals Spring 2026 | 1568 | 0 | 0 | — | `{"features":["away_region"],"weights":[0.2231117416435859]}` |
| PGL Astana 2026 | 1581 | 0 | 0 | — | `{"features":["away_region"],"weights":[0.2231117416435859]}` |
| IEM Atlanta 2026 | 1581 | 0 | 0 | — | `{"features":["away_region"],"weights":[0.2231117416435859]}` |
| CS Asia Championships 2026 | 1652 | 0 | 0 | — | `{"features":["away_region"],"weights":[0.2231117416435859]}` |
| IEM Cologne Major 2026 | 1682 | 106 | 106 | +0.0010 | `{"features":["away_region"],"weights":[0.2231117416435859]}` |
| XSE Pro League 2026 | 1788 | 0 | 0 | — | `{"features":["away_region"],"weights":[0.164552060424046]}` |
| BLAST Bounty Summer 2026 | 1828 | 0 | 0 | — | `{"features":["away_region"],"weights":[0.164552060424046]}` |
| Esports World Cup 2026 | 1859 | 0 | 0 | — | `{"features":["away_region"],"weights":[0.164552060424046]}` |
| BLAST Open Porto Fall 2026 | 1915 | 0 | 0 | — | `{"features":["away_region"],"weights":[0.164552060424046]}` |
| FISSURE PLAYGROUND Season 3 2026 | 1944 | 0 | 0 | — | `{"features":["away_region"],"weights":[0.164552060424046]}` |
| StarLadder StarSeries Fall 2026 | 1973 | 0 | 0 | — | `{"features":["away_region"],"weights":[0.164552060424046]}` |
| ESL Pro League Season 24 2026 | 1987 | 0 | 0 | — | `{"features":["away_region"],"weights":[0.164552060424046]}` |
| Thunderpick World Championship 2026 | 2028 | 0 | 0 | — | `{"features":["away_region"],"weights":[0.164552060424046]}` |
| PGL Masters Bucharest 2026 | 2042 | 0 | 0 | — | `{"features":["away_region"],"weights":[0.164552060424046]}` |
| IEM Beijing 2026 | 2083 | 0 | 0 | — | `{"features":["away_region"],"weights":[0.164552060424046]}` |
| BLAST Rivals Fall 2026 | 2113 | 0 | 0 | — | `{"features":["away_region"],"weights":[0.164552060424046]}` |

## Forward selection

The Candidate Signals marked `promote` enter one combination, one at a time: at each step the one that lowers the walk-forward log-loss most (on the 2087 training-side Matches the anchor predicted) enters, and selection stops when none lowers it. The combination is the combiner (a regularised logistic model without intercept, penalty 10 towards the anchor's weights) over the logits of the market line and the rating plus each member's features, refitted for every Event on the Events that ended before it. "Added" is the log-loss after the step − before it (the anchor before the first).

Considered: `rating_calibration`, `learned_weights`, `own_rating`, `lineup_strength`, `per_map_rating`, `bo1_veto_sim`, `rank_gap`, `round_diff_rating`, `stacker`, `stacker_no_map_elo`, `lan_online`. Anchor log-loss 0.7943.

| Order | Entered | Log-loss after | Added | 90% interval | Tried |
|---|---|---|---|---|---|
| 1 | `round_diff_rating` | 0.6302 | -0.1641 | [-0.2261, -0.1073] | `rating_calibration` 0.6333, `learned_weights` 0.6333, `own_rating` 0.6322, `lineup_strength` 0.6327, `per_map_rating` 0.6327, `bo1_veto_sim` 0.6333, `rank_gap` 0.6312, `round_diff_rating` 0.6302, `stacker` 0.6322, `stacker_no_map_elo` 0.6326, `lan_online` 0.6326 |
| 2 | `rank_gap` | 0.6291 | -0.0011 | [-0.0034, +0.0010] | `rating_calibration` 0.6305, `learned_weights` 0.6305, `own_rating` 0.6302, `lineup_strength` 0.6299, `per_map_rating` 0.6303, `bo1_veto_sim` 0.6304, `rank_gap` 0.6291, `stacker` 0.6301, `stacker_no_map_elo` 0.6303, `lan_online` 0.6300 |
| 3 | `lineup_strength` | 0.6289 | -0.0002 | [-0.0003, -0.0001] | `rating_calibration` 0.6294, `learned_weights` 0.6294, `own_rating` 0.6292, `lineup_strength` 0.6289, `per_map_rating` 0.6292, `bo1_veto_sim` 0.6294, `stacker` 0.6292, `stacker_no_map_elo` 0.6293, `lan_online` 0.6291 |

Frozen combination: `round_diff_rating` + `rank_gap` + `lineup_strength`.

| Event | Fitted on | Combiner weights |
|---|---|---|
| BLAST Premier Groups Fall 2024 | 0 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"round_diff_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"rating":0.2,"round_diff_rating.shift":0.0},"rating":{"lineup_strength.lineup_strength":0.0,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"rating":1.0,"round_diff_rating.shift":0.0}}` |
| IEM Cologne 2024 | 26 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"round_diff_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"rating":0.2,"round_diff_rating.shift":0.0},"rating":{"lineup_strength.lineup_strength":0.0,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"rating":1.0,"round_diff_rating.shift":0.0}}` |
| BLAST Premier Showdown Fall 2024 | 75 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"round_diff_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"rating":0.2,"round_diff_rating.shift":0.0},"rating":{"lineup_strength.lineup_strength":0.013217,"matches":49,"rank_gap.hltv_rank_gap":0.009039,"rank_gap.vrs_rank_gap":-0.177711,"rating":0.506236,"round_diff_rating.shift":0.152933}}` |
| BetBoom Dacha Belgrade season 2 2024 | 75 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"round_diff_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"rating":0.2,"round_diff_rating.shift":0.0},"rating":{"lineup_strength.lineup_strength":0.013217,"matches":49,"rank_gap.hltv_rank_gap":0.009039,"rank_gap.vrs_rank_gap":-0.177711,"rating":0.506236,"round_diff_rating.shift":0.152933}}` |
| ESL Pro League Season 20 2024 | 115 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"round_diff_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"rating":0.2,"round_diff_rating.shift":0.0},"rating":{"lineup_strength.lineup_strength":0.007747,"matches":89,"rank_gap.hltv_rank_gap":-0.06585,"rank_gap.vrs_rank_gap":-0.148254,"rating":0.528228,"round_diff_rating.shift":0.11725}}` |
| BLAST Premier Final Fall 2024 | 194 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"round_diff_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"rating":0.2,"round_diff_rating.shift":0.0},"rating":{"lineup_strength.lineup_strength":0.003598,"matches":165,"rank_gap.hltv_rank_gap":0.033227,"rank_gap.vrs_rank_gap":-0.068391,"rating":0.470469,"round_diff_rating.shift":0.162474}}` |
| IEM Rio 2024 | 207 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"round_diff_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"rating":0.2,"round_diff_rating.shift":0.0},"rating":{"lineup_strength.lineup_strength":0.010563,"matches":178,"rank_gap.hltv_rank_gap":0.088578,"rank_gap.vrs_rank_gap":-0.059726,"rating":0.471208,"round_diff_rating.shift":0.1737}}` |
| Thunderpick World Championship 2024 | 236 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"round_diff_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"rating":0.2,"round_diff_rating.shift":0.0},"rating":{"lineup_strength.lineup_strength":0.001856,"matches":207,"rank_gap.hltv_rank_gap":0.079595,"rank_gap.vrs_rank_gap":-0.111986,"rating":0.447569,"round_diff_rating.shift":0.143141}}` |
| BLAST Premier World Final 2024 | 236 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"round_diff_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"rating":0.2,"round_diff_rating.shift":0.0},"rating":{"lineup_strength.lineup_strength":0.001856,"matches":207,"rank_gap.hltv_rank_gap":0.079595,"rank_gap.vrs_rank_gap":-0.111986,"rating":0.447569,"round_diff_rating.shift":0.143141}}` |
| Perfect World Major Shanghai Major 2024 | 277 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"round_diff_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"rating":0.2,"round_diff_rating.shift":0.0},"rating":{"lineup_strength.lineup_strength":0.00313,"matches":248,"rank_gap.hltv_rank_gap":0.056029,"rank_gap.vrs_rank_gap":-0.12499,"rating":0.436135,"round_diff_rating.shift":0.141092}}` |
| BLAST Premier Bounty season 1 2025 | 350 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"round_diff_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"rating":0.2,"round_diff_rating.shift":0.0},"rating":{"lineup_strength.lineup_strength":0.025019,"matches":321,"rank_gap.hltv_rank_gap":-0.032564,"rank_gap.vrs_rank_gap":-0.009968,"rating":0.475555,"round_diff_rating.shift":0.164595}}` |
| IEM Katowice 2025 | 357 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"round_diff_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"rating":0.2,"round_diff_rating.shift":0.0},"rating":{"lineup_strength.lineup_strength":0.0216,"matches":328,"rank_gap.hltv_rank_gap":-0.008589,"rank_gap.vrs_rank_gap":-0.02615,"rating":0.461628,"round_diff_rating.shift":0.162039}}` |
| PGL Cluj-Napoca 2025 | 406 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"round_diff_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"rating":0.2,"round_diff_rating.shift":0.0},"rating":{"lineup_strength.lineup_strength":0.035894,"matches":375,"rank_gap.hltv_rank_gap":0.067022,"rank_gap.vrs_rank_gap":-0.023494,"rating":0.467195,"round_diff_rating.shift":0.204892}}` |
| ESL Pro League Season 21 2025 | 447 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"round_diff_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"rating":0.2,"round_diff_rating.shift":0.0},"rating":{"lineup_strength.lineup_strength":0.047565,"matches":413,"rank_gap.hltv_rank_gap":0.056322,"rank_gap.vrs_rank_gap":-0.063746,"rating":0.483993,"round_diff_rating.shift":0.191729}}` |
| BLAST Open Lisbon Spring 2025 | 487 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"round_diff_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"rating":0.2,"round_diff_rating.shift":0.0},"rating":{"lineup_strength.lineup_strength":0.064711,"matches":453,"rank_gap.hltv_rank_gap":0.032798,"rank_gap.vrs_rank_gap":0.018637,"rating":0.525781,"round_diff_rating.shift":0.245214}}` |
| PGL Bucharest 2025 | 516 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"round_diff_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"rating":0.2,"round_diff_rating.shift":0.0},"rating":{"lineup_strength.lineup_strength":0.076536,"matches":482,"rank_gap.hltv_rank_gap":0.089563,"rank_gap.vrs_rank_gap":0.004394,"rating":0.551503,"round_diff_rating.shift":0.280256}}` |
| IEM Melbourne 2025 | 557 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"round_diff_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"rating":0.2,"round_diff_rating.shift":0.0},"rating":{"lineup_strength.lineup_strength":0.073474,"matches":523,"rank_gap.hltv_rank_gap":0.22988,"rank_gap.vrs_rank_gap":0.011472,"rating":0.461938,"round_diff_rating.shift":0.27656}}` |
| BLAST Rivals Spring 2025 | 586 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"round_diff_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"rating":0.2,"round_diff_rating.shift":0.0},"rating":{"lineup_strength.lineup_strength":0.091305,"matches":552,"rank_gap.hltv_rank_gap":0.233376,"rank_gap.vrs_rank_gap":0.022244,"rating":0.48412,"round_diff_rating.shift":0.29931}}` |
| PGL Astana 2025 | 599 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"round_diff_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"rating":0.2,"round_diff_rating.shift":0.0},"rating":{"lineup_strength.lineup_strength":0.098779,"matches":565,"rank_gap.hltv_rank_gap":0.228193,"rank_gap.vrs_rank_gap":0.038453,"rating":0.496463,"round_diff_rating.shift":0.315303}}` |
| IEM Dallas 2025 | 640 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"round_diff_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"rating":0.2,"round_diff_rating.shift":0.0},"rating":{"lineup_strength.lineup_strength":0.110609,"matches":606,"rank_gap.hltv_rank_gap":0.249684,"rank_gap.vrs_rank_gap":0.112212,"rating":0.485639,"round_diff_rating.shift":0.364544}}` |
| BLAST.tv Major Austin 2025 | 669 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"round_diff_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"rating":0.2,"round_diff_rating.shift":0.0},"rating":{"lineup_strength.lineup_strength":0.115157,"matches":635,"rank_gap.hltv_rank_gap":0.245034,"rank_gap.vrs_rank_gap":0.13744,"rating":0.515774,"round_diff_rating.shift":0.391458}}` |
| FISSURE PLAYGROUND Season 1 2025 | 775 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"round_diff_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"rating":0.2,"round_diff_rating.shift":0.0},"rating":{"lineup_strength.lineup_strength":0.133044,"matches":741,"rank_gap.hltv_rank_gap":0.225631,"rank_gap.vrs_rank_gap":0.120349,"rating":0.483263,"round_diff_rating.shift":0.355468}}` |
| IEM Cologne 2025 | 802 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"round_diff_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"rating":0.2,"round_diff_rating.shift":0.0},"rating":{"lineup_strength.lineup_strength":0.137694,"matches":768,"rank_gap.hltv_rank_gap":0.219025,"rank_gap.vrs_rank_gap":0.135708,"rating":0.44958,"round_diff_rating.shift":0.333267}}` |
| BLAST Bounty Fall 2025 | 851 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"round_diff_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"rating":0.2,"round_diff_rating.shift":0.0},"rating":{"lineup_strength.lineup_strength":0.150851,"matches":817,"rank_gap.hltv_rank_gap":0.21251,"rank_gap.vrs_rank_gap":0.171686,"rating":0.501644,"round_diff_rating.shift":0.390546}}` |
| Esports World Cup 2025 | 882 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"round_diff_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"rating":0.2,"round_diff_rating.shift":0.0},"rating":{"lineup_strength.lineup_strength":0.142764,"matches":847,"rank_gap.hltv_rank_gap":0.190361,"rank_gap.vrs_rank_gap":0.177818,"rating":0.536805,"round_diff_rating.shift":0.413403}}` |
| BLAST Open London Fall 2025 | 898 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"round_diff_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"rating":0.2,"round_diff_rating.shift":0.0},"rating":{"lineup_strength.lineup_strength":0.13945,"matches":863,"rank_gap.hltv_rank_gap":0.172972,"rank_gap.vrs_rank_gap":0.158437,"rating":0.515138,"round_diff_rating.shift":0.383261}}` |
| FISSURE PLAYGROUND Season 2 2025 | 927 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"round_diff_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"rating":0.2,"round_diff_rating.shift":0.0},"rating":{"lineup_strength.lineup_strength":0.154759,"matches":892,"rank_gap.hltv_rank_gap":0.145846,"rank_gap.vrs_rank_gap":0.178852,"rating":0.496207,"round_diff_rating.shift":0.366851}}` |
| ESL Pro League Season 22 2025 | 967 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"round_diff_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"rating":0.2,"round_diff_rating.shift":0.0},"rating":{"lineup_strength.lineup_strength":0.167631,"matches":932,"rank_gap.hltv_rank_gap":0.19886,"rank_gap.vrs_rank_gap":0.152485,"rating":0.499853,"round_diff_rating.shift":0.378417}}` |
| CS Asia Championships 2025 | 1040 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"round_diff_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"rating":0.2,"round_diff_rating.shift":0.0},"rating":{"lineup_strength.lineup_strength":0.224562,"matches":1005,"rank_gap.hltv_rank_gap":0.20016,"rank_gap.vrs_rank_gap":0.204096,"rating":0.536479,"round_diff_rating.shift":0.462439}}` |
| Thunderpick World Championship 2025 | 1040 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"round_diff_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"rating":0.2,"round_diff_rating.shift":0.0},"rating":{"lineup_strength.lineup_strength":0.224562,"matches":1005,"rank_gap.hltv_rank_gap":0.20016,"rank_gap.vrs_rank_gap":0.204096,"rating":0.536479,"round_diff_rating.shift":0.462439}}` |
| PGL Masters Bucharest 2025 | 1084 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"round_diff_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"rating":0.2,"round_diff_rating.shift":0.0},"rating":{"lineup_strength.lineup_strength":0.234124,"matches":1049,"rank_gap.hltv_rank_gap":0.198424,"rank_gap.vrs_rank_gap":0.206071,"rating":0.560732,"round_diff_rating.shift":0.488781}}` |
| IEM Chengdu 2025 | 1125 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"round_diff_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"rating":0.2,"round_diff_rating.shift":0.0},"rating":{"lineup_strength.lineup_strength":0.231865,"matches":1090,"rank_gap.hltv_rank_gap":0.224304,"rank_gap.vrs_rank_gap":0.167383,"rating":0.573829,"round_diff_rating.shift":0.497819}}` |
| BLAST Rivals Fall 2025 | 1155 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"round_diff_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"rating":0.2,"round_diff_rating.shift":0.0},"rating":{"lineup_strength.lineup_strength":0.228288,"matches":1120,"rank_gap.hltv_rank_gap":0.223291,"rank_gap.vrs_rank_gap":0.215342,"rating":0.560585,"round_diff_rating.shift":0.499342}}` |
| StarLadder Major Budapest 2025 | 1168 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"round_diff_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"rating":0.2,"round_diff_rating.shift":0.0},"rating":{"lineup_strength.lineup_strength":0.233248,"matches":1133,"rank_gap.hltv_rank_gap":0.217712,"rank_gap.vrs_rank_gap":0.239349,"rating":0.557204,"round_diff_rating.shift":0.502527}}` |
| BLAST Bounty Winter 2026 | 1274 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"round_diff_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.0,"market":0.8,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"rating":0.2,"round_diff_rating.shift":0.0},"rating":{"lineup_strength.lineup_strength":0.264101,"matches":1239,"rank_gap.hltv_rank_gap":0.225523,"rank_gap.vrs_rank_gap":0.184496,"rating":0.592514,"round_diff_rating.shift":0.523521}}` |
| IEM Krakow 2026 | 1305 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"round_diff_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":-0.009155,"market":0.754523,"matches":31,"rank_gap.hltv_rank_gap":-0.162964,"rank_gap.vrs_rank_gap":-0.005192,"rating":0.019046,"round_diff_rating.shift":0.025907},"rating":{"lineup_strength.lineup_strength":0.264101,"matches":1239,"rank_gap.hltv_rank_gap":0.225523,"rank_gap.vrs_rank_gap":0.184496,"rating":0.592514,"round_diff_rating.shift":0.523521}}` |
| PGL Cluj-Napoca 2026 | 1355 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"round_diff_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.002233,"market":0.793168,"matches":81,"rank_gap.hltv_rank_gap":-0.128302,"rank_gap.vrs_rank_gap":0.155549,"rating":-0.039075,"round_diff_rating.shift":0.055823},"rating":{"lineup_strength.lineup_strength":0.264101,"matches":1239,"rank_gap.hltv_rank_gap":0.225523,"rank_gap.vrs_rank_gap":0.184496,"rating":0.592514,"round_diff_rating.shift":0.523521}}` |
| ESL Pro League Season 23 2026 | 1396 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"round_diff_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":-0.001401,"market":0.774392,"matches":122,"rank_gap.hltv_rank_gap":0.190329,"rank_gap.vrs_rank_gap":0.172101,"rating":-0.004852,"round_diff_rating.shift":0.055808},"rating":{"lineup_strength.lineup_strength":0.264101,"matches":1239,"rank_gap.hltv_rank_gap":0.225523,"rank_gap.vrs_rank_gap":0.184496,"rating":0.592514,"round_diff_rating.shift":0.523521}}` |
| BLAST Open Rotterdam Spring 2026 | 1470 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"round_diff_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.004486,"market":0.757332,"matches":196,"rank_gap.hltv_rank_gap":0.039268,"rank_gap.vrs_rank_gap":0.179411,"rating":-0.026439,"round_diff_rating.shift":0.077727},"rating":{"lineup_strength.lineup_strength":0.264101,"matches":1239,"rank_gap.hltv_rank_gap":0.225523,"rank_gap.vrs_rank_gap":0.184496,"rating":0.592514,"round_diff_rating.shift":0.523521}}` |
| PGL Bucharest 2026 | 1499 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"round_diff_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":-0.004397,"market":0.718696,"matches":225,"rank_gap.hltv_rank_gap":0.056016,"rank_gap.vrs_rank_gap":0.175356,"rating":-0.017343,"round_diff_rating.shift":0.070629},"rating":{"lineup_strength.lineup_strength":0.264101,"matches":1239,"rank_gap.hltv_rank_gap":0.225523,"rank_gap.vrs_rank_gap":0.184496,"rating":0.592514,"round_diff_rating.shift":0.523521}}` |
| IEM Rio 2026 | 1538 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"round_diff_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":-0.00571,"market":0.760742,"matches":264,"rank_gap.hltv_rank_gap":0.058684,"rank_gap.vrs_rank_gap":0.129641,"rating":0.006375,"round_diff_rating.shift":0.052874},"rating":{"lineup_strength.lineup_strength":0.264101,"matches":1239,"rank_gap.hltv_rank_gap":0.225523,"rank_gap.vrs_rank_gap":0.184496,"rating":0.592514,"round_diff_rating.shift":0.523521}}` |
| BLAST Rivals Spring 2026 | 1568 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"round_diff_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.003895,"market":0.804404,"matches":292,"rank_gap.hltv_rank_gap":0.070199,"rank_gap.vrs_rank_gap":0.116978,"rating":0.014814,"round_diff_rating.shift":0.061082},"rating":{"lineup_strength.lineup_strength":0.264672,"matches":1241,"rank_gap.hltv_rank_gap":0.227701,"rank_gap.vrs_rank_gap":0.186409,"rating":0.593248,"round_diff_rating.shift":0.525375}}` |
| PGL Astana 2026 | 1581 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"round_diff_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.004817,"market":0.792686,"matches":305,"rank_gap.hltv_rank_gap":0.03191,"rank_gap.vrs_rank_gap":0.123252,"rating":0.018439,"round_diff_rating.shift":0.058686},"rating":{"lineup_strength.lineup_strength":0.264672,"matches":1241,"rank_gap.hltv_rank_gap":0.227701,"rank_gap.vrs_rank_gap":0.186409,"rating":0.593248,"round_diff_rating.shift":0.525375}}` |
| IEM Atlanta 2026 | 1581 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"round_diff_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":0.004817,"market":0.792686,"matches":305,"rank_gap.hltv_rank_gap":0.03191,"rank_gap.vrs_rank_gap":0.123252,"rating":0.018439,"round_diff_rating.shift":0.058686},"rating":{"lineup_strength.lineup_strength":0.264672,"matches":1241,"rank_gap.hltv_rank_gap":0.227701,"rank_gap.vrs_rank_gap":0.186409,"rating":0.593248,"round_diff_rating.shift":0.525375}}` |
| CS Asia Championships 2026 | 1652 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"round_diff_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":-0.005917,"market":0.814887,"matches":376,"rank_gap.hltv_rank_gap":-0.004657,"rank_gap.vrs_rank_gap":0.108245,"rating":0.02722,"round_diff_rating.shift":0.044414},"rating":{"lineup_strength.lineup_strength":0.264672,"matches":1241,"rank_gap.hltv_rank_gap":0.227701,"rank_gap.vrs_rank_gap":0.186409,"rating":0.593248,"round_diff_rating.shift":0.525375}}` |
| IEM Cologne Major 2026 | 1682 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"round_diff_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":-0.005917,"market":0.814887,"matches":376,"rank_gap.hltv_rank_gap":-0.004657,"rank_gap.vrs_rank_gap":0.108245,"rating":0.02722,"round_diff_rating.shift":0.044414},"rating":{"lineup_strength.lineup_strength":0.270209,"matches":1271,"rank_gap.hltv_rank_gap":0.234904,"rank_gap.vrs_rank_gap":0.190159,"rating":0.585996,"round_diff_rating.shift":0.520921}}` |
| XSE Pro League 2026 | 1788 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"round_diff_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":-0.000034,"market":0.91543,"matches":480,"rank_gap.hltv_rank_gap":-0.014185,"rank_gap.vrs_rank_gap":0.063265,"rating":0.037482,"round_diff_rating.shift":0.046286},"rating":{"lineup_strength.lineup_strength":0.269986,"matches":1273,"rank_gap.hltv_rank_gap":0.235524,"rank_gap.vrs_rank_gap":0.193819,"rating":0.580124,"round_diff_rating.shift":0.515853}}` |
| BLAST Bounty Summer 2026 | 1828 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.0,"matches":0,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"round_diff_rating.shift":0.0},"market+rating":{"lineup_strength.lineup_strength":-0.000034,"market":0.91543,"matches":480,"rank_gap.hltv_rank_gap":-0.014185,"rank_gap.vrs_rank_gap":0.063265,"rating":0.037482,"round_diff_rating.shift":0.046286},"rating":{"lineup_strength.lineup_strength":0.27558,"matches":1310,"rank_gap.hltv_rank_gap":0.239557,"rank_gap.vrs_rank_gap":0.177842,"rating":0.607089,"round_diff_rating.shift":0.538997}}` |
| Esports World Cup 2026 | 1859 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.023476,"matches":1,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"round_diff_rating.shift":-0.002665},"market+rating":{"lineup_strength.lineup_strength":0.012558,"market":0.850693,"matches":509,"rank_gap.hltv_rank_gap":0.057426,"rank_gap.vrs_rank_gap":0.065057,"rating":0.013924,"round_diff_rating.shift":0.040794},"rating":{"lineup_strength.lineup_strength":0.27558,"matches":1310,"rank_gap.hltv_rank_gap":0.239557,"rank_gap.vrs_rank_gap":0.177842,"rating":0.607089,"round_diff_rating.shift":0.538997}}` |
| BLAST Open Porto Fall 2026 | 1915 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.023476,"matches":1,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"round_diff_rating.shift":-0.002665},"market+rating":{"lineup_strength.lineup_strength":0.031391,"market":0.815802,"matches":565,"rank_gap.hltv_rank_gap":0.068398,"rank_gap.vrs_rank_gap":0.028069,"rating":0.043596,"round_diff_rating.shift":0.032326},"rating":{"lineup_strength.lineup_strength":0.27558,"matches":1310,"rank_gap.hltv_rank_gap":0.239557,"rank_gap.vrs_rank_gap":0.177842,"rating":0.607089,"round_diff_rating.shift":0.538997}}` |
| FISSURE PLAYGROUND Season 3 2026 | 1944 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.023476,"matches":1,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"round_diff_rating.shift":-0.002665},"market+rating":{"lineup_strength.lineup_strength":0.039943,"market":0.829462,"matches":594,"rank_gap.hltv_rank_gap":0.08403,"rank_gap.vrs_rank_gap":0.076476,"rating":0.018407,"round_diff_rating.shift":0.037624},"rating":{"lineup_strength.lineup_strength":0.27558,"matches":1310,"rank_gap.hltv_rank_gap":0.239557,"rank_gap.vrs_rank_gap":0.177842,"rating":0.607089,"round_diff_rating.shift":0.538997}}` |
| StarLadder StarSeries Fall 2026 | 1973 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.023476,"matches":1,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"round_diff_rating.shift":-0.002665},"market+rating":{"lineup_strength.lineup_strength":0.03871,"market":0.784532,"matches":623,"rank_gap.hltv_rank_gap":0.043259,"rank_gap.vrs_rank_gap":0.133743,"rating":0.008623,"round_diff_rating.shift":0.045432},"rating":{"lineup_strength.lineup_strength":0.27558,"matches":1310,"rank_gap.hltv_rank_gap":0.239557,"rank_gap.vrs_rank_gap":0.177842,"rating":0.607089,"round_diff_rating.shift":0.538997}}` |
| ESL Pro League Season 24 2026 | 1987 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.023476,"matches":1,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"round_diff_rating.shift":-0.002665},"market+rating":{"lineup_strength.lineup_strength":0.040955,"market":0.796793,"matches":637,"rank_gap.hltv_rank_gap":0.044052,"rank_gap.vrs_rank_gap":0.116784,"rating":0.010097,"round_diff_rating.shift":0.049751},"rating":{"lineup_strength.lineup_strength":0.27558,"matches":1310,"rank_gap.hltv_rank_gap":0.239557,"rank_gap.vrs_rank_gap":0.177842,"rating":0.607089,"round_diff_rating.shift":0.538997}}` |
| Thunderpick World Championship 2026 | 2028 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.023476,"matches":1,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"round_diff_rating.shift":-0.002665},"market+rating":{"lineup_strength.lineup_strength":0.048369,"market":0.875044,"matches":678,"rank_gap.hltv_rank_gap":0.01498,"rank_gap.vrs_rank_gap":0.079598,"rating":0.007383,"round_diff_rating.shift":0.04568},"rating":{"lineup_strength.lineup_strength":0.27558,"matches":1310,"rank_gap.hltv_rank_gap":0.239557,"rank_gap.vrs_rank_gap":0.177842,"rating":0.607089,"round_diff_rating.shift":0.538997}}` |
| PGL Masters Bucharest 2026 | 2042 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.023476,"matches":1,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"round_diff_rating.shift":-0.002665},"market+rating":{"lineup_strength.lineup_strength":0.050186,"market":0.867078,"matches":692,"rank_gap.hltv_rank_gap":0.023429,"rank_gap.vrs_rank_gap":0.066618,"rating":0.014244,"round_diff_rating.shift":0.045867},"rating":{"lineup_strength.lineup_strength":0.27558,"matches":1310,"rank_gap.hltv_rank_gap":0.239557,"rank_gap.vrs_rank_gap":0.177842,"rating":0.607089,"round_diff_rating.shift":0.538997}}` |
| IEM Beijing 2026 | 2083 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.023476,"matches":1,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"round_diff_rating.shift":-0.002665},"market+rating":{"lineup_strength.lineup_strength":0.035255,"market":0.859035,"matches":733,"rank_gap.hltv_rank_gap":0.076303,"rank_gap.vrs_rank_gap":-0.018554,"rating":0.020656,"round_diff_rating.shift":0.032952},"rating":{"lineup_strength.lineup_strength":0.27558,"matches":1310,"rank_gap.hltv_rank_gap":0.239557,"rank_gap.vrs_rank_gap":0.177842,"rating":0.607089,"round_diff_rating.shift":0.538997}}` |
| BLAST Rivals Fall 2026 | 2113 | `{"market":{"lineup_strength.lineup_strength":0.0,"market":1.023476,"matches":1,"rank_gap.hltv_rank_gap":0.0,"rank_gap.vrs_rank_gap":0.0,"round_diff_rating.shift":-0.002665},"market+rating":{"lineup_strength.lineup_strength":0.026789,"market":0.91401,"matches":763,"rank_gap.hltv_rank_gap":0.061508,"rank_gap.vrs_rank_gap":-0.005346,"rating":-0.000707,"round_diff_rating.shift":0.036034},"rating":{"lineup_strength.lineup_strength":0.27558,"matches":1310,"rank_gap.hltv_rank_gap":0.239557,"rank_gap.vrs_rank_gap":0.177842,"rating":0.607089,"round_diff_rating.shift":0.538997}}` |

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
| Perfect World Major Shanghai Major 2024 | 2024-11-30 03:00 | 2024-12-15 12:23 | 73 |
| BLAST Premier Bounty season 1 2025 | 2025-01-23 15:00 | 2025-01-26 19:09 | 7 |
| IEM Katowice 2025 | 2025-01-29 09:30 | 2025-02-09 18:59 | 49 |
| PGL Cluj-Napoca 2025 | 2025-02-14 08:00 | 2025-02-23 21:54 | 41 |
| ESL Pro League Season 21 2025 | 2025-03-07 09:30 | 2025-03-16 18:41 | 40 |
| BLAST Open Lisbon Spring 2025 | 2025-03-19 11:00 | 2025-03-30 20:48 | 29 |
| PGL Bucharest 2025 | 2025-04-06 07:00 | 2025-04-13 17:41 | 41 |
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
