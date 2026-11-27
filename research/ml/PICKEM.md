# Swiss Pick’em replay

This replay scores ten picks on **28 completed stages**. Each correct pick earns one point. An ordinary advance pick earns a point only for a 3–1 or 3–2 finish.

| Model | Strategy | Mean correct (95% CI) | Stages ≥5 | Share (95% CI) | Same slip at half count | Mean changed slots |
|---|---|---:|---:|---:|---:|---:|
| Anchor | A | 4.25 (3.64–4.86) | 13/28 | 46.4% (28.6%–64.3%) | 28/28 | 0.00 |
| Anchor | B | 4.25 (3.64–4.86) | 11/28 | 39.3% (21.4%–57.1%) | 28/28 | 0.00 |
| VRS | A | 4.11 (3.50–4.71) | 11/28 | 39.3% (21.4%–57.1%) | 28/28 | 0.00 |
| VRS | B | 4.21 (3.61–4.86) | 11/28 | 39.3% (21.4%–57.1%) | 27/28 | 0.04 |
| Anchor + veto | A | 4.18 (3.61–4.79) | 12/28 | 42.9% (25.0%–60.7%) | 25/28 | 0.18 |
| Anchor + veto | B | 4.43 (3.82–5.04) | 14/28 | 50.0% (32.1%–67.9%) | 26/28 | 0.11 |
| logreg baseline | A | 4.46 (3.89–5.04) | 14/28 | 50.0% (32.1%–67.9%) | 27/28 | 0.04 |
| logreg baseline | B | 4.43 (3.86–5.04) | 14/28 | 50.0% (32.1%–67.9%) | 25/28 | 0.21 |
| lgbm baseline | A | 4.39 (3.79–5.00) | 14/28 | 50.0% (32.1%–67.9%) | 27/28 | 0.04 |
| lgbm baseline | B | 4.36 (3.75–4.96) | 15/28 | 53.6% (35.7%–71.4%) | 27/28 | 0.04 |
| logreg all | A | 4.32 (3.71–4.93) | 12/28 | 42.9% (25.0%–60.7%) | 28/28 | 0.00 |
| logreg all | B | 4.39 (3.82–5.00) | 12/28 | 42.9% (25.0%–60.7%) | 27/28 | 0.04 |
| lgbm all | A | 4.39 (3.79–5.00) | 13/28 | 46.4% (28.6%–64.3%) | 25/28 | 0.21 |
| lgbm all | B | 4.57 (3.96–5.18) | 14/28 | 50.0% (32.1%–67.9%) | 25/28 | 0.18 |
| HLTV top six | — | 3.61 (3.18–4.07) | 8/28 | 28.6% (14.3%–46.4%) | — | — |
| Random slip | — | 2.57 (2.25–2.93) | 1/28 | 3.6% (0.0%–10.7%) | — | — |

On Singapore Stage 1, anchor A reached five. The earlier 20,000-run replay scored four. The larger simulation swapped Heroic into 3–0 and PVISION into ordinary advance. On the same 200,000 outcomes the slips were almost tied at 4.5453 and 4.5445 expected points.

A uniform random slip has an exact mean of **2.75** and a **7.95%** chance of at least five (801,836 of 10,090,080 legal slips for any fixed stage result). The random row shows one reproducible uniform draw per stage and includes the same stage-bootstrap intervals as the other rows.

The intervals resample 28 whole stages with replacement 20,000 times, seed 33, taking the 2.5th and 97.5th percentiles. These scores describe the archived contests. Stages from the same Major share teams and history, which can make stage-bootstrap intervals too narrow. This sample does not establish which model or strategy will perform best in future contests.

## Forecast and selection rules

- The research snapshot supplies the completed Swiss rounds. The Singapore fixture supplies its first 24 results and opening odds. The final nine results come from the PandaScore tournament 22378 matches endpoint. Canceled listings without winners are ignored. Officially awarded matches count for standings, including ESL Pro League S22 Stage 2 and PGL Bucharest 2026. They are simulated as matches before the stage.
- The opening time freezes ratings, lines, rank badges and match history for all 120 possible opponents in both formats. Each learned model fits non-qualifier results with a known finish strictly before that time, including earlier stages of the same event. Hyperparameters minimize inner walk-forward loss on earlier completed events. Baseline and all correspond to the prototype’s baseline and full sets. Future match pages, lineups and match-specific form panels are unavailable at stage start and use the feature groups’ missing-data behavior. Venue comes from opening-match venue metadata when consistent, otherwise from the tournament listing.
- The anchor blends de-vigged market and VRS logits 80/20 when both exist. The historical opening line applies only to that pairing and format. A missing source leaves the available source at full weight. Both missing gives 50%. The veto variant replaces BO1 with the prototype map_pool probability when both teams have earlier vetoes. It weights maps by ban/pick tendencies and historical map win shares. BO3 retains the anchor. This uses the Python map-pool approximation to the Rust six-ban simulation.
- Each model simulates 200,000 joint outcomes per stage with seed 33, using actual round-one pairings and observed formats by win/loss bucket. The sidecar Buchholz ranking and rematch priorities govern later rounds. Pre-stage Liquipedia seed tables cover 12 stages. The remaining 16 use the sidecar fallback of sorting teams by summed BO1 win probability, breaking ties by team ID. The anchor supplies these seeds for every model so the pairing rule stays fixed. For non-Major stages this common pairing procedure may differ from the organizer’s. Replaying recorded winners reproduced every pairing and final record on all 12 Major stages with published seeds.
- A solves the 2/2/6 disjoint assignment exactly for simulated expected points. B searches for the highest simulated chance of five points using best-improving swaps of two teams’ roles, including unpicked. It starts from A and two seeded random slips and also tries the half-count optimum on the full run. Search stops at a local optimum, so a better unsearched slip can exist. Simulated probabilities are in-sample estimates. Changed slots counts team/category selections replaced when increasing the simulation count from 100,000 to 200,000 on the same random stream. Reordering picks within a category counts as zero.
- The HLTV baseline puts ranks 1–6 among entrants into ordinary advance, 7–8 into 3–0 and the lowest two into 0–3. The archive supplies usable dated rank badges for all 16 teams in 20/28 stages (range 5–16). Missing ranks sort last, with published seed order or team name breaking ties. When coverage is partial, this measures a ranking rule with missing data. The Singapore baseline needed a fallback even for its sixth advance pick. With complete rank coverage alone, the HLTV rule averaged 3.65 and passed on 6/20 stages.

### Excluded RMR and qualifier Swiss stages

- BLAST.tv Major Austin: European Regional Qualifier 2025 / Group Stage
- Perfect World Major Shanghai: American RMR 2024 / Group Stage
- Perfect World Major Shanghai: European RMR A 2024 / Group Stage
- Perfect World Major Shanghai: European RMR B 2024 / Group Stage

The replay also excludes the BLAST Premier Bounty Closed Qualifier 2025 Playoffs and the Austin 2025 European Regional Qualifier Play-Ins. Both used other formats.

### Reproduce

From `research/ml/` using the snapshot and the prototype’s HLTV page cache or source files.

```powershell
uv sync
uv run python -m kbml.pickem_check --runs 200000 --jobs 3
```

## Stage results

### Perfect World Major Shanghai Major 2024 / Opening Stage

Stage 15477 opened 2024-11-30T03:00:00Z. The fit had 277 earlier matches, 0/8 opening lines and 14/16 archived HLTV ranks. The simulation used published initial seeds. Qualification and elimination matches were BO3. Earlier matches were BO1.

| Model | Strategy | 3–0 | 0–3 | Other advance | Correct | Expected | Simulated ≥5 | Changed slots |
|---|---|---|---|---|---:|---:|---:|---:|
| Anchor | A | FURIA, TheMongolz | Passion UA, Rare Atom | Virtus.pro, Liquid, Complexity, BIG, paiN, FlyQuest | 3 | 5.92 | 87.3% | 0 |
| Anchor | B | FURIA, TheMongolz | Passion UA, Rare Atom | Virtus.pro, Liquid, Complexity, BIG, paiN, FlyQuest | 3 | 5.92 | 87.3% | 0 |
| VRS | A | FURIA, TheMongolz | Passion UA, Rare Atom | Virtus.pro, Liquid, Complexity, BIG, paiN, FlyQuest | 3 | 5.92 | 87.3% | 0 |
| VRS | B | FURIA, TheMongolz | Passion UA, Rare Atom | Virtus.pro, Liquid, Complexity, BIG, paiN, FlyQuest | 3 | 5.92 | 87.3% | 0 |
| Anchor + veto | A | TheMongolz, MIBR | Cloud9, Rare Atom | FURIA, Virtus.pro, Liquid, Complexity, BIG, paiN | 4 | 4.09 | 37.4% | 2 |
| Anchor + veto | B | TheMongolz, MIBR | Rare Atom, Imperial | FURIA, Virtus.pro, Liquid, Complexity, BIG, paiN | 5 | 4.08 | 37.6% | 0 |
| logreg baseline | A | FURIA, TheMongolz | Passion UA, Rare Atom | Virtus.pro, Liquid, Complexity, BIG, paiN, MIBR | 4 | 3.69 | 28.4% | 0 |
| logreg baseline | B | FURIA, TheMongolz | Passion UA, Rare Atom | Virtus.pro, Liquid, Complexity, BIG, paiN, MIBR | 4 | 3.69 | 28.4% | 2 |
| lgbm baseline | A | FURIA, TheMongolz | Passion UA, Rare Atom | Virtus.pro, Liquid, Complexity, BIG, paiN, MIBR | 4 | 4.37 | 46.6% | 0 |
| lgbm baseline | B | FURIA, TheMongolz | Passion UA, Rare Atom | Virtus.pro, Liquid, Complexity, BIG, paiN, MIBR | 4 | 4.37 | 46.6% | 0 |
| logreg all | A | TheMongolz, paiN | Rare Atom, Imperial | FURIA, Virtus.pro, Liquid, Complexity, fnatic, Cloud9 | 3 | 3.55 | 24.8% | 0 |
| logreg all | B | TheMongolz, paiN | Rare Atom, Imperial | FURIA, Virtus.pro, Liquid, Complexity, fnatic, Cloud9 | 3 | 3.55 | 24.8% | 0 |
| lgbm all | A | FURIA, TheMongolz | Passion UA, Rare Atom | Virtus.pro, Liquid, Complexity, paiN, MIBR, Wildcard | 4 | 3.84 | 31.9% | 2 |
| lgbm all | B | FURIA, TheMongolz | Passion UA, Rare Atom | Virtus.pro, Liquid, Complexity, paiN, MIBR, Wildcard | 4 | 3.84 | 31.9% | 3 |
| HLTV top six | — | BIG, MIBR | FlyQuest, Rare Atom | FURIA, Virtus.pro, Liquid, Complexity, TheMongolz, paiN | 2 | — | — | — |
| Random slip | — | Liquid, Wildcard | Complexity, TheMongolz | FURIA, Virtus.pro, GamerLegion, Cloud9, FlyQuest, Passion UA | 3 | — | — | — |
| **Actual** | | Liquid, TheMongolz | fnatic, Imperial | FURIA, BIG, paiN, GamerLegion, MIBR, Wildcard | | | | |

### Perfect World Major Shanghai Major 2024 / Elimination Stage

Stage 15478 opened 2024-12-05T03:00:00Z. The fit had 310 earlier matches, 0/8 opening lines and 16/16 archived HLTV ranks. The simulation used published initial seeds. Qualification and elimination matches were BO3. Earlier matches were BO1.

| Model | Strategy | 3–0 | 0–3 | Other advance | Correct | Expected | Simulated ≥5 | Changed slots |
|---|---|---|---|---|---:|---:|---:|---:|
| Anchor | A | G2, Natus Vincere | Wildcard, MIBR | Vitality, Spirit, MOUZ, FaZe, Heroic, TheMongolz | 5 | 5.82 | 80.4% | 0 |
| Anchor | B | G2, Vitality | Wildcard, MIBR | Natus Vincere, Spirit, MOUZ, FaZe, Heroic, TheMongolz | 6 | 5.79 | 80.8% | 0 |
| VRS | A | G2, Natus Vincere | Wildcard, MIBR | Vitality, Spirit, MOUZ, FaZe, Heroic, TheMongolz | 5 | 5.82 | 80.4% | 0 |
| VRS | B | G2, Vitality | Wildcard, MIBR | Natus Vincere, Spirit, MOUZ, FaZe, Heroic, TheMongolz | 6 | 5.79 | 80.8% | 0 |
| Anchor + veto | A | FaZe, Liquid | GamerLegion, Wildcard | G2, Natus Vincere, Vitality, Spirit, MOUZ, TheMongolz | 4 | 4.18 | 37.4% | 0 |
| Anchor + veto | B | Natus Vincere, Vitality | GamerLegion, Wildcard | G2, Spirit, MOUZ, FaZe, Heroic, TheMongolz | 7 | 4.16 | 39.6% | 0 |
| logreg baseline | A | G2, Vitality | GamerLegion, Wildcard | Natus Vincere, Spirit, MOUZ, FaZe, Heroic, TheMongolz | 6 | 3.94 | 34.1% | 0 |
| logreg baseline | B | G2, Vitality | GamerLegion, Wildcard | Natus Vincere, Spirit, MOUZ, FaZe, Heroic, TheMongolz | 6 | 3.94 | 34.1% | 0 |
| lgbm baseline | A | Natus Vincere, Vitality | Wildcard, MIBR | G2, Spirit, MOUZ, FaZe, Heroic, TheMongolz | 7 | 4.56 | 51.0% | 0 |
| lgbm baseline | B | Natus Vincere, Vitality | Wildcard, MIBR | G2, Spirit, MOUZ, FaZe, Heroic, TheMongolz | 7 | 4.56 | 51.0% | 0 |
| logreg all | A | Natus Vincere, Vitality | GamerLegion, Wildcard | G2, Spirit, MOUZ, FaZe, Heroic, TheMongolz | 7 | 3.82 | 31.2% | 0 |
| logreg all | B | Natus Vincere, Vitality | GamerLegion, Wildcard | G2, Spirit, MOUZ, FaZe, Heroic, TheMongolz | 7 | 3.82 | 31.2% | 0 |
| lgbm all | A | Natus Vincere, Vitality | GamerLegion, Wildcard | G2, Spirit, MOUZ, FaZe, Heroic, TheMongolz | 7 | 4.50 | 50.2% | 0 |
| lgbm all | B | Natus Vincere, Vitality | GamerLegion, Wildcard | G2, Spirit, MOUZ, FaZe, Heroic, TheMongolz | 7 | 4.50 | 50.2% | 0 |
| HLTV top six | — | Heroic, FURIA | GamerLegion, Wildcard | G2, Natus Vincere, Vitality, Spirit, MOUZ, FaZe | 5 | — | — | — |
| Random slip | — | FaZe, BIG | MOUZ, MIBR | Natus Vincere, Vitality, Heroic, TheMongolz, Liquid, Wildcard | 2 | — | — | — |
| **Actual** | | Vitality, TheMongolz | Wildcard, BIG | G2, Spirit, MOUZ, FaZe, Heroic, Liquid | | | | |

### PGL Cluj-Napoca 2025 / Group Stage

Stage 15965 opened 2025-02-14T08:00:00Z. The fit had 406 earlier matches, 0/8 opening lines and 15/16 archived HLTV ranks. The simulation used anchor-derived initial seeds. Every match was BO3.

| Model | Strategy | 3–0 | 0–3 | Other advance | Correct | Expected | Simulated ≥5 | Changed slots |
|---|---|---|---|---|---:|---:|---:|---:|
| Anchor | A | FaZe, TheMongolz | FlyQuest, SAW | 3DMAX, Astralis, Eternal Fire, MOUZ, Team Falcons, Virtus.pro | 7 | 6.37 | 92.6% | 0 |
| Anchor | B | FaZe, TheMongolz | Complexity, SAW | 3DMAX, Astralis, Eternal Fire, MOUZ, Team Falcons, Virtus.pro | 6 | 6.27 | 93.9% | 0 |
| VRS | A | FaZe, TheMongolz | FlyQuest, SAW | 3DMAX, Astralis, Eternal Fire, MOUZ, Team Falcons, Virtus.pro | 7 | 6.37 | 92.6% | 0 |
| VRS | B | FaZe, TheMongolz | Complexity, SAW | 3DMAX, Astralis, Eternal Fire, MOUZ, Team Falcons, Virtus.pro | 6 | 6.27 | 93.9% | 0 |
| Anchor + veto | A | FaZe, TheMongolz | FlyQuest, SAW | 3DMAX, Astralis, Eternal Fire, MOUZ, Team Falcons, Virtus.pro | 7 | 6.37 | 92.6% | 0 |
| Anchor + veto | B | FaZe, TheMongolz | Complexity, SAW | 3DMAX, Astralis, Eternal Fire, MOUZ, Team Falcons, Virtus.pro | 6 | 6.27 | 93.9% | 0 |
| logreg baseline | A | FaZe, TheMongolz | Complexity, SAW | 3DMAX, Eternal Fire, MOUZ, Team Falcons, Virtus.pro, paiN | 6 | 4.20 | 42.8% | 1 |
| logreg baseline | B | FaZe, TheMongolz | Complexity, SAW | 3DMAX, Eternal Fire, MOUZ, Team Falcons, Virtus.pro, paiN | 6 | 4.20 | 42.8% | 0 |
| lgbm baseline | A | FaZe, TheMongolz | Complexity, SAW | 3DMAX, Eternal Fire, MOUZ, Team Falcons, Virtus.pro, paiN | 6 | 4.16 | 41.6% | 0 |
| lgbm baseline | B | FaZe, TheMongolz | Complexity, SAW | 3DMAX, Eternal Fire, MOUZ, Team Falcons, Virtus.pro, paiN | 6 | 4.16 | 41.6% | 0 |
| logreg all | A | Eternal Fire, TheMongolz | Complexity, Imperial Female | 3DMAX, FaZe, MIBR, MOUZ, Virtus.pro, paiN | 4 | 3.54 | 24.4% | 0 |
| logreg all | B | Eternal Fire, TheMongolz | Complexity, Imperial Female | 3DMAX, FaZe, MIBR, MOUZ, Virtus.pro, paiN | 4 | 3.54 | 24.4% | 0 |
| lgbm all | A | Eternal Fire, TheMongolz | Complexity, Imperial Female | 3DMAX, Astralis, FaZe, MIBR, MOUZ, Virtus.pro | 4 | 4.00 | 36.7% | 0 |
| lgbm all | B | Eternal Fire, TheMongolz | Complexity, Imperial Female | 3DMAX, Astralis, FaZe, MIBR, MOUZ, Virtus.pro | 4 | 4.00 | 36.7% | 0 |
| HLTV top six | — | Team Falcons, paiN | Imperial Female, SAW | Astralis, Eternal Fire, FaZe, MOUZ, TheMongolz, Virtus.pro | 4 | — | — | — |
| Random slip | — | SAW, Wildcard | 3DMAX, FlyQuest | Complexity, Eternal Fire, FaZe, Imperial Female, Virtus.pro, paiN | 3 | — | — | — |
| **Actual** | | FaZe, TheMongolz | FlyQuest, Imperial Female | Astralis, Eternal Fire, MOUZ, SAW, Team Falcons, paiN | | | | |

### ESL Pro League Season 21 2025 / Group Stage

Stage 16056 opened 2025-03-07T09:30:00Z. The fit had 447 earlier matches, 0/8 opening lines and 16/16 archived HLTV ranks. The simulation used anchor-derived initial seeds. Every match was BO3.

| Model | Strategy | 3–0 | 0–3 | Other advance | Correct | Expected | Simulated ≥5 | Changed slots |
|---|---|---|---|---|---:|---:|---:|---:|
| Anchor | A | Spirit, Vitality | SAW, TYLOO | Eternal Fire, G2, MOUZ, Natus Vincere, Team Falcons, TheMongolz | 8 | 7.10 | 96.8% | 0 |
| Anchor | B | Spirit, Vitality | MIBR, TYLOO | Eternal Fire, G2, MOUZ, Natus Vincere, Team Falcons, TheMongolz | 9 | 6.83 | 97.0% | 0 |
| VRS | A | Spirit, Vitality | SAW, TYLOO | Eternal Fire, G2, MOUZ, Natus Vincere, Team Falcons, TheMongolz | 8 | 7.10 | 96.8% | 0 |
| VRS | B | Spirit, Vitality | MIBR, TYLOO | Eternal Fire, G2, MOUZ, Natus Vincere, Team Falcons, TheMongolz | 9 | 6.83 | 97.0% | 0 |
| Anchor + veto | A | Spirit, Vitality | SAW, TYLOO | Eternal Fire, G2, MOUZ, Natus Vincere, Team Falcons, TheMongolz | 8 | 7.10 | 96.8% | 0 |
| Anchor + veto | B | Spirit, Vitality | MIBR, TYLOO | Eternal Fire, G2, MOUZ, Natus Vincere, Team Falcons, TheMongolz | 9 | 6.83 | 97.0% | 0 |
| logreg baseline | A | Spirit, Vitality | SAW, TYLOO | Eternal Fire, G2, MOUZ, Natus Vincere, Team Falcons, TheMongolz | 8 | 5.17 | 69.0% | 0 |
| logreg baseline | B | Spirit, Vitality | MIBR, TYLOO | Eternal Fire, G2, MOUZ, Natus Vincere, Team Falcons, TheMongolz | 9 | 5.17 | 69.9% | 0 |
| lgbm baseline | A | Spirit, Vitality | SAW, TYLOO | Eternal Fire, G2, MOUZ, Natus Vincere, Team Falcons, TheMongolz | 8 | 4.66 | 54.4% | 0 |
| lgbm baseline | B | Spirit, Vitality | SAW, TYLOO | Eternal Fire, G2, MOUZ, Natus Vincere, Team Falcons, TheMongolz | 8 | 4.66 | 54.4% | 0 |
| logreg all | A | Spirit, Vitality | MIBR, TYLOO | Eternal Fire, FURIA, G2, MOUZ, Natus Vincere, TheMongolz | 9 | 4.06 | 38.1% | 0 |
| logreg all | B | Spirit, Vitality | MIBR, TYLOO | Eternal Fire, FURIA, G2, MOUZ, Natus Vincere, TheMongolz | 9 | 4.06 | 38.1% | 0 |
| lgbm all | A | Spirit, Vitality | SAW, TYLOO | Eternal Fire, G2, MOUZ, Natus Vincere, Team Falcons, TheMongolz | 8 | 4.57 | 51.9% | 0 |
| lgbm all | B | Spirit, Vitality | MIBR, TYLOO | Eternal Fire, G2, MOUZ, Natus Vincere, Team Falcons, TheMongolz | 9 | 4.56 | 52.5% | 0 |
| HLTV top six | — | MOUZ, Team Falcons | SAW, TYLOO | Eternal Fire, G2, Natus Vincere, Spirit, TheMongolz, Vitality | 5 | — | — | — |
| Random slip | — | Eternal Fire, Natus Vincere | 3DMAX, paiN | GamerLegion, SAW, TYLOO, Team Falcons, TheMongolz, Vitality | 1 | — | — | — |
| **Actual** | | Spirit, Vitality | MIBR, TYLOO | Eternal Fire, G2, Liquid, MOUZ, Natus Vincere, TheMongolz | | | | |

### PGL Bucharest 2025 / Group Stage

Stage 16299 opened 2025-04-06T07:00:00Z. The fit had 516 earlier matches, 0/8 opening lines and 15/16 archived HLTV ranks. The simulation used anchor-derived initial seeds. Every match was BO3.

| Model | Strategy | 3–0 | 0–3 | Other advance | Correct | Expected | Simulated ≥5 | Changed slots |
|---|---|---|---|---|---:|---:|---:|---:|
| Anchor | A | Team Falcons, TheMongolz | Apogee Esports, Aurora Gaming | Astralis, FaZe, G2, GamerLegion, Virtus.pro, paiN | 3 | 6.62 | 93.4% | 0 |
| Anchor | B | Team Falcons, TheMongolz | Apogee Esports, Aurora Gaming | 3DMAX, Astralis, GamerLegion, Liquid, Virtus.pro, paiN | 2 | 6.36 | 96.9% | 0 |
| VRS | A | Team Falcons, TheMongolz | Apogee Esports, Aurora Gaming | Astralis, FaZe, G2, GamerLegion, Virtus.pro, paiN | 3 | 6.62 | 93.4% | 0 |
| VRS | B | Team Falcons, TheMongolz | Apogee Esports, Aurora Gaming | 3DMAX, Astralis, GamerLegion, Liquid, Virtus.pro, paiN | 2 | 6.36 | 96.9% | 0 |
| Anchor + veto | A | Team Falcons, TheMongolz | Apogee Esports, Aurora Gaming | Astralis, FaZe, G2, GamerLegion, Virtus.pro, paiN | 3 | 6.62 | 93.4% | 0 |
| Anchor + veto | B | Team Falcons, TheMongolz | Apogee Esports, Aurora Gaming | 3DMAX, Astralis, GamerLegion, Liquid, Virtus.pro, paiN | 2 | 6.36 | 96.9% | 0 |
| logreg baseline | A | FaZe, TheMongolz | Aurora Gaming, Rare Atom | Astralis, G2, GamerLegion, Team Falcons, Virtus.pro, paiN | 3 | 4.96 | 61.6% | 0 |
| logreg baseline | B | Team Falcons, TheMongolz | Aurora Gaming, Rare Atom | 3DMAX, Astralis, FaZe, G2, GamerLegion, Virtus.pro | 3 | 4.92 | 61.8% | 2 |
| lgbm baseline | A | FaZe, TheMongolz | Aurora Gaming, Rare Atom | Astralis, G2, GamerLegion, Team Falcons, Virtus.pro, paiN | 3 | 4.98 | 61.6% | 0 |
| lgbm baseline | B | FaZe, TheMongolz | Aurora Gaming, Rare Atom | 3DMAX, Astralis, G2, GamerLegion, Team Falcons, Virtus.pro | 3 | 4.97 | 62.0% | 1 |
| logreg all | A | Team Falcons, TheMongolz | Apogee Esports, Legacy | 3DMAX, Astralis, FURIA, FaZe, G2, GamerLegion | 2 | 4.41 | 46.9% | 0 |
| logreg all | B | Team Falcons, TheMongolz | Apogee Esports, Legacy | 3DMAX, Astralis, FaZe, G2, GamerLegion, Virtus.pro | 3 | 4.41 | 47.0% | 0 |
| lgbm all | A | G2, TheMongolz | Apogee Esports, Legacy | Astralis, FaZe, GamerLegion, Liquid, Team Falcons, Virtus.pro | 5 | 4.41 | 46.9% | 0 |
| lgbm all | B | G2, TheMongolz | Apogee Esports, Legacy | Astralis, FaZe, GamerLegion, Liquid, Team Falcons, Virtus.pro | 5 | 4.41 | 46.9% | 0 |
| HLTV top six | — | Astralis, Virtus.pro | Apogee Esports, Legacy | Aurora Gaming, FaZe, G2, Liquid, Team Falcons, TheMongolz | 3 | — | — | — |
| Random slip | — | Aurora Gaming, Team Falcons | FURIA, Rare Atom | Apogee Esports, Astralis, Complexity, Legacy, Virtus.pro, paiN | 2 | — | — | — |
| **Actual** | | 3DMAX, G2 | Liquid, paiN | Aurora Gaming, Complexity, FaZe, GamerLegion, Team Falcons, Virtus.pro | | | | |

### PGL Astana 2025 / Group Stage

Stage 16574 opened 2025-05-10T05:00:00Z. The fit had 599 earlier matches, 0/8 opening lines and 16/16 archived HLTV ranks. The simulation used anchor-derived initial seeds. Every match was BO3.

| Model | Strategy | 3–0 | 0–3 | Other advance | Correct | Expected | Simulated ≥5 | Changed slots |
|---|---|---|---|---|---:|---:|---:|---:|
| Anchor | A | G2, Spirit | HOTU, ODDIK | Astralis, Aurora Gaming, GamerLegion, Natus Vincere, TheMongolz, Virtus.pro | 4 | 7.34 | 96.5% | 0 |
| Anchor | B | G2, TheMongolz | HOTU, ODDIK | Astralis, GamerLegion, Natus Vincere, Spirit, Virtus.pro, paiN | 3 | 6.65 | 98.4% | 0 |
| VRS | A | G2, Spirit | HOTU, ODDIK | Astralis, Aurora Gaming, GamerLegion, Natus Vincere, TheMongolz, Virtus.pro | 4 | 7.34 | 96.5% | 0 |
| VRS | B | G2, TheMongolz | HOTU, ODDIK | Astralis, GamerLegion, Natus Vincere, Spirit, Virtus.pro, paiN | 3 | 6.65 | 98.4% | 0 |
| Anchor + veto | A | G2, Spirit | HOTU, ODDIK | Astralis, Aurora Gaming, GamerLegion, Natus Vincere, TheMongolz, Virtus.pro | 4 | 7.34 | 96.5% | 0 |
| Anchor + veto | B | G2, TheMongolz | HOTU, ODDIK | Astralis, GamerLegion, Natus Vincere, Spirit, Virtus.pro, paiN | 3 | 6.65 | 98.4% | 0 |
| logreg baseline | A | G2, Spirit | NIP, ODDIK | Astralis, Aurora Gaming, GamerLegion, Natus Vincere, TheMongolz, Virtus.pro | 4 | 5.15 | 65.8% | 0 |
| logreg baseline | B | G2, Spirit | MIBR, ODDIK | Astralis, Aurora Gaming, GamerLegion, Natus Vincere, TheMongolz, Virtus.pro | 4 | 5.15 | 66.2% | 0 |
| lgbm baseline | A | G2, Spirit | NIP, ODDIK | Astralis, Aurora Gaming, GamerLegion, Natus Vincere, TheMongolz, Virtus.pro | 4 | 4.90 | 59.0% | 0 |
| lgbm baseline | B | G2, Spirit | MIBR, ODDIK | Astralis, Aurora Gaming, GamerLegion, Natus Vincere, TheMongolz, Virtus.pro | 4 | 4.88 | 59.4% | 0 |
| logreg all | A | G2, Spirit | HOTU, ODDIK | Aurora Gaming, FURIA, GamerLegion, Natus Vincere, TheMongolz, Virtus.pro | 4 | 4.08 | 37.5% | 0 |
| logreg all | B | G2, Spirit | HOTU, ODDIK | Aurora Gaming, FURIA, GamerLegion, Natus Vincere, TheMongolz, Virtus.pro | 4 | 4.08 | 37.5% | 0 |
| lgbm all | A | G2, Spirit | NIP, ODDIK | Astralis, Aurora Gaming, GamerLegion, Natus Vincere, TheMongolz, Virtus.pro | 4 | 4.35 | 44.6% | 0 |
| lgbm all | B | G2, Spirit | NIP, ODDIK | Astralis, Aurora Gaming, GamerLegion, Natus Vincere, TheMongolz, Virtus.pro | 4 | 4.35 | 44.6% | 0 |
| HLTV top six | — | Astralis, Virtus.pro | HOTU, ODDIK | Aurora Gaming, G2, GamerLegion, Natus Vincere, Spirit, TheMongolz | 2 | — | — | — |
| Random slip | — | FURIA, paiN | BIG, Virtus.pro | G2, M80, MIBR, NIP, Natus Vincere, ODDIK | 3 | — | — | — |
| **Actual** | | Spirit, TheMongolz | GamerLegion, M80 | Astralis, Aurora Gaming, FURIA, MIBR, NIP, Natus Vincere | | | | |

### BLAST.tv Major Austin 2025 / Stage 1

Stage 16658 opened 2025-06-03T15:15:00Z. The fit had 669 earlier matches, 0/8 opening lines and 16/16 archived HLTV ranks. The simulation used published initial seeds. Qualification and elimination matches were BO3. Earlier matches were BO1.

| Model | Strategy | 3–0 | 0–3 | Other advance | Correct | Expected | Simulated ≥5 | Changed slots |
|---|---|---|---|---|---:|---:|---:|---:|
| Anchor | A | Complexity, Heroic | Metizport, Fluxo | B8, BetBoom Team, TYLOO, Lynn Vision, FlyQuest, NRG | 6 | 5.68 | 77.5% | 0 |
| Anchor | B | Complexity, Heroic | Metizport, Fluxo | B8, BetBoom Team, TYLOO, Lynn Vision, FlyQuest, NRG | 6 | 5.68 | 77.5% | 0 |
| VRS | A | Complexity, Heroic | Metizport, Fluxo | B8, BetBoom Team, TYLOO, Lynn Vision, FlyQuest, NRG | 6 | 5.68 | 77.5% | 0 |
| VRS | B | Complexity, Heroic | Metizport, Fluxo | B8, BetBoom Team, TYLOO, Lynn Vision, FlyQuest, NRG | 6 | 5.68 | 77.5% | 0 |
| Anchor + veto | A | Heroic, OG | Chinggis Warriors, Fluxo | Complexity, B8, TYLOO, Lynn Vision, FlyQuest, NRG | 4 | 4.70 | 56.4% | 0 |
| Anchor + veto | B | Heroic, OG | Chinggis Warriors, Fluxo | Complexity, B8, TYLOO, Lynn Vision, FlyQuest, NRG | 4 | 4.70 | 56.4% | 0 |
| logreg baseline | A | Heroic, B8 | NRG, Legacy | Complexity, BetBoom Team, TYLOO, OG, Chinggis Warriors, Nemiga | 6 | 3.56 | 24.3% | 0 |
| logreg baseline | B | Heroic, B8 | NRG, Legacy | Complexity, BetBoom Team, TYLOO, OG, Chinggis Warriors, Nemiga | 6 | 3.56 | 24.3% | 0 |
| lgbm baseline | A | Heroic, B8 | NRG, Legacy | Complexity, BetBoom Team, TYLOO, FlyQuest, OG, Chinggis Warriors | 5 | 3.77 | 29.5% | 0 |
| lgbm baseline | B | Heroic, B8 | Legacy, Fluxo | Complexity, BetBoom Team, TYLOO, FlyQuest, OG, Chinggis Warriors | 6 | 3.77 | 29.5% | 0 |
| logreg all | A | Heroic, BetBoom Team | Metizport, Fluxo | Complexity, B8, TYLOO, Wildcard, FlyQuest, NRG | 4 | 3.41 | 20.6% | 0 |
| logreg all | B | Heroic, Wildcard | Metizport, Fluxo | Complexity, B8, BetBoom Team, TYLOO, FlyQuest, NRG | 5 | 3.41 | 21.2% | 0 |
| lgbm all | A | Heroic, B8 | Metizport, Fluxo | Complexity, BetBoom Team, TYLOO, Lynn Vision, Wildcard, FlyQuest | 7 | 3.53 | 23.7% | 0 |
| lgbm all | B | Heroic, B8 | Metizport, Fluxo | Complexity, BetBoom Team, TYLOO, Lynn Vision, Wildcard, FlyQuest | 7 | 3.53 | 23.7% | 0 |
| HLTV top six | — | Lynn Vision, Legacy | Nemiga, Metizport | Complexity, Heroic, B8, TYLOO, Wildcard, FlyQuest | 2 | — | — | — |
| Random slip | — | B8, Lynn Vision | BetBoom Team, Metizport | FlyQuest, OG, Nemiga, NRG, Legacy, Fluxo | 5 | — | — | — |
| **Actual** | | Heroic, B8 | Metizport, Fluxo | BetBoom Team, TYLOO, Lynn Vision, OG, Nemiga, Legacy | | | | |

### BLAST.tv Major Austin 2025 / Stage 2

Stage 16659 opened 2025-06-07T15:00:00Z. The fit had 702 earlier matches, 0/8 opening lines and 16/16 archived HLTV ranks. The simulation used published initial seeds. Qualification and elimination matches were BO3. Earlier matches were BO1.

| Model | Strategy | 3–0 | 0–3 | Other advance | Correct | Expected | Simulated ≥5 | Changed slots |
|---|---|---|---|---|---:|---:|---:|---:|
| Anchor | A | Team Falcons, 3DMAX | Nemiga, Legacy | FaZe, Virtus.pro, FURIA, MIBR, Heroic, TYLOO | 2 | 6.26 | 92.0% | 0 |
| Anchor | B | Team Falcons, 3DMAX | Nemiga, Legacy | FaZe, Virtus.pro, FURIA, MIBR, Heroic, TYLOO | 2 | 6.26 | 92.0% | 0 |
| VRS | A | Team Falcons, 3DMAX | Nemiga, Legacy | FaZe, Virtus.pro, FURIA, MIBR, Heroic, TYLOO | 2 | 6.26 | 92.0% | 0 |
| VRS | B | Team Falcons, 3DMAX | Nemiga, Legacy | FaZe, Virtus.pro, FURIA, MIBR, Heroic, TYLOO | 2 | 6.26 | 92.0% | 0 |
| Anchor + veto | A | FURIA, B8 | paiN, Legacy | Team Falcons, FaZe, 3DMAX, Virtus.pro, Heroic, TYLOO | 2 | 4.33 | 42.9% | 0 |
| Anchor + veto | B | FURIA, B8 | paiN, Legacy | Team Falcons, FaZe, 3DMAX, Virtus.pro, Heroic, TYLOO | 2 | 4.33 | 42.9% | 0 |
| logreg baseline | A | Team Falcons, Heroic | paiN, M80 | FaZe, 3DMAX, Virtus.pro, FURIA, B8, OG | 3 | 4.09 | 38.0% | 0 |
| logreg baseline | B | Team Falcons, Heroic | paiN, M80 | FaZe, 3DMAX, Virtus.pro, FURIA, B8, OG | 3 | 4.09 | 38.0% | 0 |
| lgbm baseline | A | Team Falcons, 3DMAX | M80, Legacy | FaZe, Virtus.pro, FURIA, MIBR, B8, Heroic | 2 | 4.32 | 44.5% | 0 |
| lgbm baseline | B | Team Falcons, 3DMAX | M80, Legacy | FaZe, Virtus.pro, FURIA, MIBR, B8, Heroic | 2 | 4.32 | 44.5% | 0 |
| logreg all | A | Team Falcons, Virtus.pro | OG, Nemiga | FaZe, 3DMAX, paiN, FURIA, MIBR, Heroic | 6 | 3.80 | 30.9% | 0 |
| logreg all | B | Team Falcons, Virtus.pro | OG, Nemiga | FaZe, 3DMAX, paiN, FURIA, MIBR, Heroic | 6 | 3.80 | 30.9% | 0 |
| lgbm all | A | Team Falcons, 3DMAX | Lynn Vision, Legacy | FaZe, Virtus.pro, FURIA, MIBR, B8, Heroic | 2 | 3.76 | 29.1% | 2 |
| lgbm all | B | Team Falcons, FURIA | Lynn Vision, Legacy | FaZe, 3DMAX, Virtus.pro, paiN, MIBR, Heroic | 3 | 3.74 | 29.5% | 1 |
| HLTV top six | — | paiN, MIBR | OG, Nemiga | Team Falcons, FaZe, 3DMAX, Virtus.pro, FURIA, Heroic | 4 | — | — | — |
| Random slip | — | FaZe, MIBR | B8, Legacy | Team Falcons, 3DMAX, Virtus.pro, FURIA, M80, Nemiga | 3 | — | — | — |
| **Actual** | | Virtus.pro, Legacy | BetBoom Team, OG | FaZe, 3DMAX, paiN, FURIA, Nemiga, Lynn Vision | | | | |

### BLAST.tv Major Austin 2025 / Stage 3

Stage 16660 opened 2025-06-12T15:00:00Z. The fit had 735 earlier matches, 0/8 opening lines and 16/16 archived HLTV ranks. The simulation used published initial seeds. Qualification and elimination matches were BO3. Earlier matches were BO1.

| Model | Strategy | 3–0 | 0–3 | Other advance | Correct | Expected | Simulated ≥5 | Changed slots |
|---|---|---|---|---|---:|---:|---:|---:|
| Anchor | A | Vitality, MOUZ | Legacy, Nemiga | Spirit, G2, TheMongolz, Aurora Gaming, Natus Vincere, FaZe | 4 | 6.59 | 95.3% | 0 |
| Anchor | B | Vitality, MOUZ | Legacy, Nemiga | Spirit, G2, TheMongolz, Aurora Gaming, Natus Vincere, FaZe | 4 | 6.59 | 95.3% | 0 |
| VRS | A | Vitality, MOUZ | Legacy, Nemiga | Spirit, G2, TheMongolz, Aurora Gaming, Natus Vincere, FaZe | 4 | 6.59 | 95.3% | 0 |
| VRS | B | Vitality, MOUZ | Legacy, Nemiga | Spirit, G2, TheMongolz, Aurora Gaming, Natus Vincere, FaZe | 4 | 6.59 | 95.3% | 0 |
| Anchor + veto | A | Vitality, Spirit | Legacy, paiN | MOUZ, G2, TheMongolz, Aurora Gaming, Natus Vincere, FaZe | 5 | 4.97 | 62.1% | 0 |
| Anchor + veto | B | Vitality, Spirit | Legacy, paiN | MOUZ, G2, TheMongolz, Aurora Gaming, Natus Vincere, FaZe | 5 | 4.97 | 62.1% | 0 |
| logreg baseline | A | Vitality, Spirit | Legacy, paiN | MOUZ, G2, TheMongolz, Aurora Gaming, Natus Vincere, FaZe | 5 | 4.94 | 61.8% | 0 |
| logreg baseline | B | Vitality, Spirit | Legacy, paiN | MOUZ, G2, TheMongolz, Aurora Gaming, Natus Vincere, FaZe | 5 | 4.94 | 61.8% | 0 |
| lgbm baseline | A | Vitality, Spirit | Legacy, paiN | MOUZ, G2, TheMongolz, Aurora Gaming, Natus Vincere, FaZe | 5 | 4.88 | 59.9% | 0 |
| lgbm baseline | B | Vitality, Spirit | Legacy, paiN | MOUZ, G2, TheMongolz, Aurora Gaming, Natus Vincere, FaZe | 5 | 4.88 | 59.9% | 0 |
| logreg all | A | Vitality, Spirit | Nemiga, Lynn Vision | MOUZ, G2, TheMongolz, Aurora Gaming, Natus Vincere, Liquid | 5 | 3.74 | 28.6% | 0 |
| logreg all | B | Vitality, Spirit | Legacy, Lynn Vision | MOUZ, G2, TheMongolz, Aurora Gaming, Natus Vincere, Liquid | 4 | 3.73 | 28.9% | 0 |
| lgbm all | A | Vitality, Spirit | Legacy, Lynn Vision | MOUZ, G2, TheMongolz, Aurora Gaming, Natus Vincere, Liquid | 4 | 3.98 | 35.9% | 0 |
| lgbm all | B | Vitality, Spirit | Legacy, Lynn Vision | MOUZ, G2, TheMongolz, Aurora Gaming, Natus Vincere, Liquid | 4 | 3.98 | 35.9% | 0 |
| HLTV top six | — | G2, Liquid | Nemiga, Lynn Vision | Vitality, MOUZ, Spirit, TheMongolz, Aurora Gaming, Natus Vincere | 5 | — | — | — |
| Random slip | — | Vitality, Aurora Gaming | Legacy, FaZe | Spirit, Natus Vincere, Liquid, paiN, Nemiga, Lynn Vision | 2 | — | — | — |
| **Actual** | | Spirit, FURIA | Liquid, Nemiga | Vitality, MOUZ, TheMongolz, Natus Vincere, paiN, FaZe | | | | |

### FISSURE PLAYGROUND Season 2 2025 / Group Stage

Stage 17542 opened 2025-09-12T08:00:00Z. The fit had 927 earlier matches, 0/8 opening lines and 16/16 archived HLTV ranks. The simulation used anchor-derived initial seeds. Every match was BO3.

| Model | Strategy | 3–0 | 0–3 | Other advance | Correct | Expected | Simulated ≥5 | Changed slots |
|---|---|---|---|---|---:|---:|---:|---:|
| Anchor | A | Team Falcons, TheMongolz | Legacy, Liquid | 3DMAX, Astralis, Aurora Gaming, FURIA, FaZe, TYLOO | 3 | 6.68 | 93.4% | 0 |
| Anchor | B | Team Falcons, TheMongolz | Legacy, Liquid | 3DMAX, Astralis, Aurora Gaming, FURIA, FaZe, TYLOO | 3 | 6.68 | 93.4% | 0 |
| VRS | A | Team Falcons, TheMongolz | Legacy, Liquid | 3DMAX, Astralis, Aurora Gaming, FURIA, FaZe, TYLOO | 3 | 6.68 | 93.4% | 0 |
| VRS | B | Team Falcons, TheMongolz | Legacy, Liquid | 3DMAX, Astralis, Aurora Gaming, FURIA, FaZe, TYLOO | 3 | 6.68 | 93.4% | 0 |
| Anchor + veto | A | Team Falcons, TheMongolz | Legacy, Liquid | 3DMAX, Astralis, Aurora Gaming, FURIA, FaZe, TYLOO | 3 | 6.68 | 93.4% | 0 |
| Anchor + veto | B | Team Falcons, TheMongolz | Legacy, Liquid | 3DMAX, Astralis, Aurora Gaming, FURIA, FaZe, TYLOO | 3 | 6.68 | 93.4% | 0 |
| logreg baseline | A | Team Falcons, TheMongolz | Liquid, Virtus.pro | 3DMAX, Aurora Gaming, FURIA, FaZe, G2, TYLOO | 3 | 4.08 | 37.8% | 0 |
| logreg baseline | B | Team Falcons, TheMongolz | Liquid, Virtus.pro | 3DMAX, Aurora Gaming, FURIA, FaZe, G2, TYLOO | 3 | 4.08 | 37.8% | 0 |
| lgbm baseline | A | Team Falcons, TheMongolz | Legacy, Liquid | 3DMAX, Aurora Gaming, FURIA, FaZe, G2, TYLOO | 3 | 4.07 | 37.3% | 0 |
| lgbm baseline | B | Team Falcons, TheMongolz | Legacy, Liquid | 3DMAX, Aurora Gaming, FURIA, FaZe, G2, TYLOO | 3 | 4.07 | 37.3% | 0 |
| logreg all | A | Team Falcons, TheMongolz | Legacy, Liquid | 3DMAX, Astralis, Aurora Gaming, FaZe, G2, Heroic | 3 | 3.35 | 19.7% | 0 |
| logreg all | B | Team Falcons, TheMongolz | Legacy, Virtus.pro | 3DMAX, Astralis, Aurora Gaming, FaZe, G2, Heroic | 3 | 3.34 | 19.8% | 0 |
| lgbm all | A | Team Falcons, TheMongolz | Legacy, Liquid | 3DMAX, Astralis, Aurora Gaming, FURIA, FaZe, G2 | 4 | 3.69 | 27.6% | 0 |
| lgbm all | B | Team Falcons, TheMongolz | Legacy, Liquid | 3DMAX, Astralis, Aurora Gaming, FURIA, FaZe, G2 | 4 | 3.69 | 27.6% | 0 |
| HLTV top six | — | 3DMAX, paiN | Legacy, Virtus.pro | Aurora Gaming, FURIA, FaZe, G2, Team Falcons, TheMongolz | 3 | — | — | — |
| Random slip | — | Liquid, Lynn Vision | Heroic, Virtus.pro | FaZe, G2, Legacy, TYLOO, TheMongolz, paiN | 4 | — | — | — |
| **Actual** | | Aurora Gaming, Team Falcons | Heroic, Lynn Vision | Astralis, FURIA, G2, Liquid, TheMongolz, paiN | | | | |

### ESL Pro League Season 22 2025 / Stage 1

Stage 17555 opened 2025-09-28T08:30:00Z. The fit had 967 earlier matches, 0/8 opening lines and 16/16 archived HLTV ranks. The simulation used anchor-derived initial seeds. Every match was BO3.

| Model | Strategy | 3–0 | 0–3 | Other advance | Correct | Expected | Simulated ≥5 | Changed slots |
|---|---|---|---|---|---:|---:|---:|---:|
| Anchor | A | 3DMAX, Astralis | Inner Circle Esports, Rooster | B8, FURIA, G2, GamerLegion, Heroic, Legacy | 4 | 7.09 | 94.6% | 0 |
| Anchor | B | 3DMAX, FURIA | Inner Circle Esports, Rooster | B8, G2, GamerLegion, Gentle Mates, Heroic, Legacy | 4 | 6.99 | 98.1% | 0 |
| VRS | A | 3DMAX, Astralis | Inner Circle Esports, Rooster | B8, FURIA, G2, GamerLegion, Heroic, Legacy | 4 | 7.09 | 94.6% | 0 |
| VRS | B | 3DMAX, FURIA | Inner Circle Esports, Rooster | B8, G2, GamerLegion, Gentle Mates, Heroic, Legacy | 4 | 6.99 | 98.1% | 0 |
| Anchor + veto | A | 3DMAX, Astralis | Inner Circle Esports, Rooster | B8, FURIA, G2, GamerLegion, Heroic, Legacy | 4 | 7.09 | 94.6% | 0 |
| Anchor + veto | B | 3DMAX, FURIA | Inner Circle Esports, Rooster | B8, G2, GamerLegion, Gentle Mates, Heroic, Legacy | 4 | 6.99 | 98.1% | 0 |
| logreg baseline | A | 3DMAX, FURIA | Inner Circle Esports, Rooster | Astralis, B8, G2, GamerLegion, Gentle Mates, Heroic | 5 | 4.70 | 55.9% | 0 |
| logreg baseline | B | 3DMAX, FURIA | Inner Circle Esports, Rooster | Astralis, B8, G2, GamerLegion, Gentle Mates, Heroic | 5 | 4.70 | 55.9% | 0 |
| lgbm baseline | A | 3DMAX, FURIA | Inner Circle Esports, Rooster | Astralis, G2, GamerLegion, Gentle Mates, Heroic, Legacy | 5 | 4.60 | 52.9% | 0 |
| lgbm baseline | B | 3DMAX, FURIA | Inner Circle Esports, Rooster | Astralis, G2, GamerLegion, Gentle Mates, Heroic, Legacy | 5 | 4.60 | 52.9% | 0 |
| logreg all | A | 3DMAX, FURIA | Inner Circle Esports, Rooster | Astralis, B8, Fluxo, G2, GamerLegion, Heroic | 4 | 3.95 | 34.3% | 0 |
| logreg all | B | 3DMAX, FURIA | Inner Circle Esports, Rooster | Astralis, B8, Fluxo, G2, GamerLegion, Heroic | 4 | 3.95 | 34.3% | 0 |
| lgbm all | A | FURIA, G2 | HOTU, Rooster | 3DMAX, Astralis, B8, GamerLegion, Heroic, Legacy | 4 | 4.45 | 48.4% | 0 |
| lgbm all | B | FURIA, G2 | HOTU, Rooster | 3DMAX, Astralis, B8, GamerLegion, Heroic, Legacy | 4 | 4.45 | 48.4% | 0 |
| HLTV top six | — | Legacy, M80 | Fluxo, Rooster | 3DMAX, Astralis, FURIA, G2, GamerLegion, Heroic | 6 | — | — | — |
| Random slip | — | GamerLegion, Heroic | Gentle Mates, NRG | Astralis, B8, Fluxo, G2, Inner Circle Esports, Legacy | 3 | — | — | — |
| **Actual** | | HOTU, Inner Circle Esports | NRG, Rooster | 3DMAX, Astralis, FURIA, G2, GamerLegion, Gentle Mates | | | | |

### ESL Pro League Season 22 2025 / Stage 2

Stage 17633 opened 2025-10-04T08:30:00Z. The fit had 1,000 earlier matches, 0/8 opening lines and 16/16 archived HLTV ranks. The simulation used anchor-derived initial seeds. Every match was BO3.

| Model | Strategy | 3–0 | 0–3 | Other advance | Correct | Expected | Simulated ≥5 | Changed slots |
|---|---|---|---|---|---:|---:|---:|---:|
| Anchor | A | Spirit, TheMongolz | HOTU, Inner Circle Esports | 3DMAX, Aurora Gaming, MOUZ, Natus Vincere, Team Falcons, Vitality | 5 | 6.51 | 90.7% | 0 |
| Anchor | B | Spirit, Vitality | HOTU, Inner Circle Esports | 3DMAX, Astralis, Aurora Gaming, FURIA, Natus Vincere, Team Falcons | 4 | 6.13 | 95.6% | 0 |
| VRS | A | Spirit, TheMongolz | HOTU, Inner Circle Esports | 3DMAX, Aurora Gaming, MOUZ, Natus Vincere, Team Falcons, Vitality | 5 | 6.51 | 90.7% | 0 |
| VRS | B | Spirit, Vitality | HOTU, Inner Circle Esports | 3DMAX, Astralis, Aurora Gaming, FURIA, Natus Vincere, Team Falcons | 4 | 6.13 | 95.6% | 0 |
| Anchor + veto | A | Spirit, TheMongolz | HOTU, Inner Circle Esports | 3DMAX, Aurora Gaming, MOUZ, Natus Vincere, Team Falcons, Vitality | 5 | 6.51 | 90.7% | 0 |
| Anchor + veto | B | Spirit, Vitality | HOTU, Inner Circle Esports | 3DMAX, Astralis, Aurora Gaming, FURIA, Natus Vincere, Team Falcons | 4 | 6.13 | 95.6% | 0 |
| logreg baseline | A | Spirit, Vitality | HOTU, Inner Circle Esports | Aurora Gaming, FURIA, MOUZ, Natus Vincere, Team Falcons, TheMongolz | 4 | 5.16 | 68.0% | 0 |
| logreg baseline | B | Spirit, Vitality | HOTU, Inner Circle Esports | Aurora Gaming, FURIA, MOUZ, Natus Vincere, Team Falcons, TheMongolz | 4 | 5.16 | 68.0% | 0 |
| lgbm baseline | A | Spirit, Vitality | HOTU, Inner Circle Esports | Aurora Gaming, FURIA, MOUZ, Natus Vincere, Team Falcons, TheMongolz | 4 | 4.59 | 52.4% | 0 |
| lgbm baseline | B | Spirit, Vitality | HOTU, Inner Circle Esports | Aurora Gaming, FURIA, MOUZ, Natus Vincere, Team Falcons, TheMongolz | 4 | 4.59 | 52.4% | 0 |
| logreg all | A | Spirit, Vitality | HOTU, Inner Circle Esports | FURIA, G2, MOUZ, Natus Vincere, Team Falcons, TheMongolz | 4 | 4.25 | 42.9% | 0 |
| logreg all | B | Spirit, Vitality | HOTU, Inner Circle Esports | FURIA, G2, MOUZ, Natus Vincere, Team Falcons, TheMongolz | 4 | 4.25 | 42.9% | 0 |
| lgbm all | A | TheMongolz, Vitality | HOTU, Inner Circle Esports | Aurora Gaming, FURIA, MOUZ, Natus Vincere, Spirit, Team Falcons | 3 | 4.07 | 38.2% | 0 |
| lgbm all | B | Spirit, TheMongolz | HOTU, Inner Circle Esports | Aurora Gaming, FURIA, MOUZ, Natus Vincere, Team Falcons, Vitality | 5 | 4.06 | 38.5% | 0 |
| HLTV top six | — | Aurora Gaming, Team Falcons | HOTU, Inner Circle Esports | FURIA, G2, MOUZ, Spirit, TheMongolz, Vitality | 4 | — | — | — |
| Random slip | — | FURIA, TheMongolz | GamerLegion, HOTU | 3DMAX, Astralis, FaZe, G2, Gentle Mates, Team Falcons | 3 | — | — | — |
| **Actual** | | Spirit, Team Falcons | Astralis, GamerLegion | 3DMAX, FURIA, FaZe, MOUZ, Natus Vincere, Vitality | | | | |

### PGL Masters Bucharest 2025 / Group Stage

Stage 17846 opened 2025-10-26T08:19:00Z. The fit had 1,084 earlier matches, 0/8 opening lines and 12/16 archived HLTV ranks. The simulation used anchor-derived initial seeds. Every match was BO3.

| Model | Strategy | 3–0 | 0–3 | Other advance | Correct | Expected | Simulated ≥5 | Changed slots |
|---|---|---|---|---|---:|---:|---:|---:|
| Anchor | A | Astralis, Aurora Gaming | FlyQuest, Heroic | 3DMAX, GamerLegion, Liquid, MIBR, NIP, paiN | 2 | 5.89 | 83.5% | 0 |
| Anchor | B | Astralis, Aurora Gaming | FlyQuest, Heroic | 3DMAX, GamerLegion, Liquid, MIBR, NIP, paiN | 2 | 5.89 | 83.5% | 0 |
| VRS | A | Astralis, Aurora Gaming | FlyQuest, Heroic | 3DMAX, GamerLegion, Liquid, MIBR, NIP, paiN | 2 | 5.89 | 83.5% | 0 |
| VRS | B | Astralis, Aurora Gaming | FlyQuest, Heroic | 3DMAX, GamerLegion, Liquid, MIBR, NIP, paiN | 2 | 5.89 | 83.5% | 0 |
| Anchor + veto | A | Astralis, Aurora Gaming | FlyQuest, Heroic | 3DMAX, GamerLegion, Liquid, MIBR, NIP, paiN | 2 | 5.89 | 83.5% | 0 |
| Anchor + veto | B | Astralis, Aurora Gaming | FlyQuest, Heroic | 3DMAX, GamerLegion, Liquid, MIBR, NIP, paiN | 2 | 5.89 | 83.5% | 0 |
| logreg baseline | A | 3DMAX, Aurora Gaming | FlyQuest, MIBR | Astralis, GamerLegion, Gentle Mates, Legacy, Liquid, NIP | 4 | 3.64 | 26.6% | 0 |
| logreg baseline | B | 3DMAX, Aurora Gaming | FlyQuest, fnatic | Astralis, GamerLegion, Gentle Mates, Legacy, Liquid, NIP | 3 | 3.64 | 26.9% | 0 |
| lgbm baseline | A | Aurora Gaming, Liquid | FlyQuest, Heroic | 3DMAX, Astralis, GamerLegion, Gentle Mates, NIP, paiN | 1 | 3.71 | 28.4% | 0 |
| lgbm baseline | B | Aurora Gaming, Liquid | FlyQuest, Heroic | 3DMAX, Astralis, GamerLegion, Gentle Mates, NIP, paiN | 1 | 3.71 | 28.4% | 0 |
| logreg all | A | 3DMAX, GamerLegion | FlyQuest, SAW | Astralis, Aurora Gaming, B8, Gentle Mates, Liquid, paiN | 3 | 3.98 | 35.2% | 0 |
| logreg all | B | 3DMAX, GamerLegion | FlyQuest, SAW | Astralis, Aurora Gaming, B8, Gentle Mates, Liquid, paiN | 3 | 3.98 | 35.2% | 0 |
| lgbm all | A | 3DMAX, Aurora Gaming | BetBoom Team, FlyQuest | Astralis, B8, GamerLegion, Gentle Mates, Liquid, paiN | 3 | 4.10 | 38.3% | 0 |
| lgbm all | B | 3DMAX, Aurora Gaming | FlyQuest, SAW | Astralis, B8, GamerLegion, Gentle Mates, Liquid, paiN | 3 | 4.10 | 38.5% | 0 |
| HLTV top six | — | Gentle Mates, Heroic | NIP, SAW | 3DMAX, Astralis, Aurora Gaming, GamerLegion, Liquid, paiN | 5 | — | — | — |
| Random slip | — | B8, Liquid | 3DMAX, GamerLegion | Astralis, Aurora Gaming, BetBoom Team, FlyQuest, MIBR, paiN | 1 | — | — | — |
| **Actual** | | Heroic, paiN | MIBR, NIP | Aurora Gaming, B8, GamerLegion, Legacy, Liquid, SAW | | | | |

### StarLadder Major Budapest 2025 / Stage 1

Stage 17897 opened 2025-11-24T12:00:00Z. The fit had 1,168 earlier matches, 0/8 opening lines and 16/16 archived HLTV ranks. The simulation used published initial seeds. Qualification and elimination matches were BO3. Earlier matches were BO1.

| Model | Strategy | 3–0 | 0–3 | Other advance | Correct | Expected | Simulated ≥5 | Changed slots |
|---|---|---|---|---|---:|---:|---:|---:|
| Anchor | A | Legacy, FaZe | NRG, Rare Atom | B8, GamerLegion, fnatic, PARIVISION, NIP, Imperial | 6 | 6.22 | 87.5% | 0 |
| Anchor | B | Legacy, FaZe | NRG, Rare Atom | B8, GamerLegion, fnatic, PARIVISION, NIP, Imperial | 6 | 6.22 | 87.5% | 0 |
| VRS | A | Legacy, FaZe | NRG, Rare Atom | B8, GamerLegion, fnatic, PARIVISION, NIP, Imperial | 6 | 6.22 | 87.5% | 0 |
| VRS | B | Legacy, FaZe | NRG, Rare Atom | B8, GamerLegion, fnatic, PARIVISION, NIP, Imperial | 6 | 6.22 | 87.5% | 0 |
| Anchor + veto | A | NIP, Imperial | NRG, Rare Atom | Legacy, FaZe, B8, GamerLegion, fnatic, PARIVISION | 5 | 4.24 | 38.7% | 0 |
| Anchor + veto | B | Legacy, NIP | NRG, Rare Atom | FaZe, B8, GamerLegion, fnatic, PARIVISION, Imperial | 6 | 4.23 | 41.8% | 2 |
| logreg baseline | A | Legacy, FaZe | NRG, Rare Atom | B8, GamerLegion, fnatic, PARIVISION, NIP, Imperial | 6 | 3.91 | 33.3% | 0 |
| logreg baseline | B | Legacy, FaZe | NRG, Rare Atom | B8, GamerLegion, fnatic, PARIVISION, NIP, Imperial | 6 | 3.91 | 33.3% | 0 |
| lgbm baseline | A | Legacy, FaZe | Fluxo, Rare Atom | B8, GamerLegion, fnatic, PARIVISION, NIP, Imperial | 6 | 3.91 | 33.4% | 0 |
| lgbm baseline | B | Legacy, FaZe | Fluxo, Rare Atom | B8, GamerLegion, fnatic, PARIVISION, NIP, Imperial | 6 | 3.91 | 33.4% | 0 |
| logreg all | A | fnatic, PARIVISION | RED Canids, The Huns Esports | Legacy, FaZe, B8, GamerLegion, NIP, M80 | 3 | 3.30 | 18.1% | 0 |
| logreg all | B | Legacy, fnatic | RED Canids, The Huns Esports | FaZe, B8, GamerLegion, PARIVISION, NIP, M80 | 4 | 3.29 | 18.3% | 0 |
| lgbm all | A | Legacy, GamerLegion | RED Canids, The Huns Esports | FaZe, B8, fnatic, PARIVISION, NIP, Lynn Vision | 5 | 3.59 | 25.5% | 0 |
| lgbm all | B | Legacy, GamerLegion | RED Canids, The Huns Esports | FaZe, B8, fnatic, PARIVISION, NIP, Lynn Vision | 5 | 3.59 | 25.5% | 0 |
| HLTV top six | — | fnatic, NIP | RED Canids, The Huns Esports | Legacy, FaZe, B8, GamerLegion, Lynn Vision, M80 | 2 | — | — | — |
| Random slip | — | Legacy, Fluxo | RED Canids, Rare Atom | FaZe, NIP, Imperial, FlyQuest, Lynn Vision, M80 | 4 | — | — | — |
| **Actual** | | FlyQuest, M80 | Lynn Vision, Rare Atom | FaZe, B8, fnatic, PARIVISION, NIP, Imperial | | | | |

### StarLadder Major Budapest 2025 / Stage 2

Stage 17898 opened 2025-11-29T12:00:00Z. The fit had 1,201 earlier matches, 0/8 opening lines and 16/16 archived HLTV ranks. The simulation used published initial seeds. Qualification and elimination matches were BO3. Earlier matches were BO1.

| Model | Strategy | 3–0 | 0–3 | Other advance | Correct | Expected | Simulated ≥5 | Changed slots |
|---|---|---|---|---|---:|---:|---:|---:|
| Anchor | A | Aurora Gaming, Natus Vincere | M80, FlyQuest | Liquid, 3DMAX, Astralis, TYLOO, B8, FaZe | 5 | 5.89 | 82.7% | 0 |
| Anchor | B | Aurora Gaming, Natus Vincere | M80, FlyQuest | Liquid, 3DMAX, Astralis, TYLOO, B8, FaZe | 5 | 5.89 | 82.7% | 0 |
| VRS | A | Aurora Gaming, Natus Vincere | M80, FlyQuest | Liquid, 3DMAX, Astralis, TYLOO, B8, FaZe | 5 | 5.89 | 82.7% | 0 |
| VRS | B | Aurora Gaming, Natus Vincere | M80, FlyQuest | Liquid, 3DMAX, Astralis, TYLOO, B8, FaZe | 5 | 5.89 | 82.7% | 0 |
| Anchor + veto | A | B8, fnatic | MIBR, M80 | Aurora Gaming, Natus Vincere, Liquid, 3DMAX, Astralis, FaZe | 3 | 4.24 | 39.4% | 0 |
| Anchor + veto | B | Natus Vincere, fnatic | MIBR, M80 | Aurora Gaming, Liquid, 3DMAX, Astralis, B8, FaZe | 5 | 4.22 | 40.4% | 0 |
| logreg baseline | A | Aurora Gaming, Natus Vincere | MIBR, M80 | Liquid, 3DMAX, Astralis, B8, NIP, FaZe | 5 | 3.66 | 27.5% | 0 |
| logreg baseline | B | Aurora Gaming, Natus Vincere | MIBR, M80 | Liquid, 3DMAX, Astralis, B8, NIP, FaZe | 5 | 3.66 | 27.5% | 0 |
| lgbm baseline | A | Aurora Gaming, Natus Vincere | MIBR, M80 | Liquid, 3DMAX, Astralis, B8, fnatic, FaZe | 5 | 3.75 | 29.6% | 0 |
| lgbm baseline | B | Aurora Gaming, Natus Vincere | MIBR, M80 | Liquid, 3DMAX, Astralis, B8, fnatic, FaZe | 5 | 3.75 | 29.6% | 0 |
| logreg all | A | Aurora Gaming, TYLOO | MIBR, Passion UA | Natus Vincere, Liquid, 3DMAX, Astralis, B8, FaZe | 4 | 3.13 | 14.0% | 0 |
| logreg all | B | Aurora Gaming, Natus Vincere | MIBR, FlyQuest | Liquid, 3DMAX, Astralis, B8, NIP, FaZe | 6 | 3.11 | 15.1% | 0 |
| lgbm all | A | Aurora Gaming, Natus Vincere | MIBR, FlyQuest | Liquid, 3DMAX, Astralis, B8, fnatic, FaZe | 6 | 3.38 | 20.8% | 0 |
| lgbm all | B | Aurora Gaming, Natus Vincere | MIBR, FlyQuest | Liquid, 3DMAX, Astralis, B8, fnatic, FaZe | 6 | 3.38 | 20.8% | 1 |
| HLTV top six | — | Passion UA, B8 | PARIVISION, Imperial | Aurora Gaming, Natus Vincere, Liquid, 3DMAX, Astralis, FaZe | 2 | — | — | — |
| Random slip | — | M80, Imperial | Passion UA, B8 | Natus Vincere, Liquid, TYLOO, MIBR, FlyQuest, PARIVISION | 2 | — | — | — |
| **Actual** | | Natus Vincere, FaZe | MIBR, FlyQuest | Liquid, 3DMAX, Passion UA, B8, PARIVISION, Imperial | | | | |

### StarLadder Major Budapest 2025 / Stage 3

Stage 17899 opened 2025-12-04T12:00:00Z. The fit had 1,234 earlier matches, 0/8 opening lines and 16/16 archived HLTV ranks. The simulation used published initial seeds. Qualification and elimination matches were BO3. Earlier matches were BO1.

| Model | Strategy | 3–0 | 0–3 | Other advance | Correct | Expected | Simulated ≥5 | Changed slots |
|---|---|---|---|---|---:|---:|---:|---:|
| Anchor | A | FURIA, Team Falcons | Imperial, Passion UA | Vitality, TheMongolz, MOUZ, Spirit, G2, Liquid | 4 | 5.94 | 83.5% | 0 |
| Anchor | B | FURIA, Team Falcons | Imperial, Passion UA | TheMongolz, MOUZ, Spirit, G2, Natus Vincere, Liquid | 4 | 5.92 | 84.3% | 0 |
| VRS | A | FURIA, Team Falcons | Imperial, Passion UA | Vitality, TheMongolz, MOUZ, Spirit, G2, Liquid | 4 | 5.94 | 83.5% | 0 |
| VRS | B | FURIA, Team Falcons | Imperial, Passion UA | TheMongolz, MOUZ, Spirit, G2, Natus Vincere, Liquid | 4 | 5.92 | 84.3% | 0 |
| Anchor + veto | A | Vitality, Spirit | Imperial, Passion UA | FURIA, Team Falcons, TheMongolz, MOUZ, G2, Natus Vincere | 5 | 4.16 | 39.7% | 0 |
| Anchor + veto | B | Vitality, Spirit | Imperial, Passion UA | FURIA, Team Falcons, TheMongolz, MOUZ, G2, Natus Vincere | 5 | 4.16 | 39.7% | 0 |
| logreg baseline | A | FURIA, Team Falcons | Imperial, Passion UA | Vitality, TheMongolz, MOUZ, Spirit, Natus Vincere, Liquid | 5 | 3.89 | 32.6% | 0 |
| logreg baseline | B | FURIA, Team Falcons | Imperial, Passion UA | Vitality, TheMongolz, MOUZ, Spirit, Natus Vincere, Liquid | 5 | 3.89 | 32.6% | 0 |
| lgbm baseline | A | FURIA, Team Falcons | Imperial, Passion UA | Vitality, TheMongolz, MOUZ, Spirit, Natus Vincere, Liquid | 5 | 4.09 | 37.7% | 0 |
| lgbm baseline | B | FURIA, Team Falcons | Imperial, Passion UA | Vitality, TheMongolz, MOUZ, Spirit, Natus Vincere, Liquid | 5 | 4.09 | 37.7% | 0 |
| logreg all | A | FURIA, Team Falcons | PARIVISION, Passion UA | Vitality, TheMongolz, MOUZ, Spirit, G2, Natus Vincere | 6 | 3.46 | 21.8% | 0 |
| logreg all | B | Team Falcons, MOUZ | PARIVISION, Passion UA | FURIA, Vitality, TheMongolz, Spirit, G2, Natus Vincere | 4 | 3.45 | 22.1% | 1 |
| lgbm all | A | FURIA, Vitality | Imperial, PARIVISION | Team Falcons, TheMongolz, MOUZ, Spirit, G2, Natus Vincere | 6 | 3.78 | 29.4% | 0 |
| lgbm all | B | FURIA, Vitality | Imperial, PARIVISION | Team Falcons, TheMongolz, MOUZ, Spirit, G2, Natus Vincere | 6 | 3.78 | 29.4% | 0 |
| HLTV top six | — | G2, Natus Vincere | Imperial, PARIVISION | FURIA, Vitality, Team Falcons, TheMongolz, MOUZ, Spirit | 5 | — | — | — |
| Random slip | — | Natus Vincere, Imperial | MOUZ, FaZe | FURIA, TheMongolz, paiN, PARIVISION, Liquid, Passion UA | 1 | — | — | — |
| **Actual** | | FURIA, Spirit | PARIVISION, Liquid | Vitality, Team Falcons, TheMongolz, MOUZ, Natus Vincere, FaZe | | | | |

### PGL Cluj-Napoca 2026 / Group Stage

Stage 20058 opened 2026-02-14T08:30:00Z. The fit had 1,355 earlier matches, 8/8 opening lines and 16/16 archived HLTV ranks. The simulation used anchor-derived initial seeds. Every match was BO3.

| Model | Strategy | 3–0 | 0–3 | Other advance | Correct | Expected | Simulated ≥5 | Changed slots |
|---|---|---|---|---|---:|---:|---:|---:|
| Anchor | A | FURIA, Vitality | Heroic, paiN | Aurora Gaming, FaZe, MOUZ, Natus Vincere, PARIVISION, Team Falcons | 6 | 5.51 | 75.0% | 0 |
| Anchor | B | FURIA, Vitality | Heroic, paiN | Aurora Gaming, FaZe, MOUZ, Natus Vincere, PARIVISION, Team Falcons | 6 | 5.51 | 75.0% | 0 |
| VRS | A | FURIA, Vitality | Heroic, paiN | Aurora Gaming, FaZe, MOUZ, Natus Vincere, PARIVISION, Team Falcons | 6 | 5.67 | 78.5% | 0 |
| VRS | B | FURIA, Vitality | Heroic, paiN | Aurora Gaming, FaZe, MOUZ, Natus Vincere, PARIVISION, Team Falcons | 6 | 5.67 | 78.5% | 0 |
| Anchor + veto | A | FURIA, Vitality | Heroic, paiN | Aurora Gaming, FaZe, MOUZ, Natus Vincere, PARIVISION, Team Falcons | 6 | 5.51 | 75.0% | 0 |
| Anchor + veto | B | FURIA, Vitality | Heroic, paiN | Aurora Gaming, FaZe, MOUZ, Natus Vincere, PARIVISION, Team Falcons | 6 | 5.51 | 75.0% | 0 |
| logreg baseline | A | FURIA, Vitality | Heroic, paiN | FaZe, MOUZ, Natus Vincere, PARIVISION, Team Falcons, TheMongolz | 6 | 4.45 | 47.8% | 0 |
| logreg baseline | B | FURIA, Vitality | Heroic, paiN | FaZe, MOUZ, Natus Vincere, PARIVISION, Team Falcons, TheMongolz | 6 | 4.45 | 47.8% | 0 |
| lgbm baseline | A | FURIA, Vitality | Heroic, paiN | FaZe, MOUZ, Natus Vincere, PARIVISION, Team Falcons, TheMongolz | 6 | 4.74 | 55.2% | 0 |
| lgbm baseline | B | FURIA, Vitality | Heroic, paiN | FaZe, MOUZ, Natus Vincere, PARIVISION, Team Falcons, TheMongolz | 6 | 4.74 | 55.2% | 0 |
| logreg all | A | FURIA, Vitality | B8, paiN | Aurora Gaming, FaZe, MOUZ, Natus Vincere, Team Falcons, TheMongolz | 6 | 3.77 | 30.1% | 0 |
| logreg all | B | FURIA, Vitality | B8, paiN | Aurora Gaming, FaZe, MOUZ, Natus Vincere, Team Falcons, TheMongolz | 6 | 3.77 | 30.1% | 0 |
| lgbm all | A | FURIA, Vitality | Heroic, paiN | FaZe, MOUZ, Natus Vincere, PARIVISION, Team Falcons, TheMongolz | 6 | 4.30 | 43.8% | 0 |
| lgbm all | B | FURIA, Vitality | Heroic, paiN | FaZe, MOUZ, Natus Vincere, PARIVISION, Team Falcons, TheMongolz | 6 | 4.30 | 43.8% | 0 |
| HLTV top six | — | G2, Natus Vincere | Heroic, paiN | FURIA, FaZe, MOUZ, PARIVISION, Team Falcons, Vitality | 4 | — | — | — |
| Random slip | — | 3DMAX, TheMongolz | Aurora Gaming, FUT Esports | Astralis, FURIA, FaZe, Heroic, MOUZ, Vitality | 2 | — | — | — |
| **Actual** | | Team Falcons, Vitality | 3DMAX, paiN | Aurora Gaming, FURIA, MOUZ, Natus Vincere, PARIVISION, TheMongolz | | | | |

### ESL Pro League Season 23 2026 / Stage 1

Stage 20263 opened 2026-03-01T10:30:00Z. The fit had 1,396 earlier matches, 8/8 opening lines and 7/16 archived HLTV ranks. The simulation used anchor-derived initial seeds. Every match was BO3.

| Model | Strategy | 3–0 | 0–3 | Other advance | Correct | Expected | Simulated ≥5 | Changed slots |
|---|---|---|---|---|---:|---:|---:|---:|
| Anchor | A | FUT Esports, PARIVISION | Gaimin Gladiators, SemperFi Esports | 3DMAX, Astralis, G2, Legacy, Liquid, NRG | 5 | 6.00 | 92.0% | 0 |
| Anchor | B | FUT Esports, PARIVISION | Gaimin Gladiators, SemperFi Esports | 3DMAX, Astralis, G2, Legacy, Liquid, NRG | 5 | 6.00 | 92.0% | 0 |
| VRS | A | Astralis, PARIVISION | Gaimin Gladiators, SemperFi Esports | 3DMAX, G2, Legacy, Liquid, NRG, paiN | 4 | 6.07 | 91.9% | 0 |
| VRS | B | FUT Esports, PARIVISION | Gaimin Gladiators, SemperFi Esports | 3DMAX, Astralis, G2, Legacy, Liquid, NRG | 5 | 6.04 | 93.0% | 0 |
| Anchor + veto | A | FUT Esports, PARIVISION | Gaimin Gladiators, SemperFi Esports | 3DMAX, Astralis, G2, Legacy, Liquid, NRG | 5 | 6.00 | 92.0% | 0 |
| Anchor + veto | B | FUT Esports, PARIVISION | Gaimin Gladiators, SemperFi Esports | 3DMAX, Astralis, G2, Legacy, Liquid, NRG | 5 | 6.00 | 92.0% | 0 |
| logreg baseline | A | G2, PARIVISION | Gaimin Gladiators, SemperFi Esports | 3DMAX, Astralis, FUT Esports, Legacy, Liquid, Passion UA | 3 | 3.99 | 36.8% | 0 |
| logreg baseline | B | G2, PARIVISION | Gaimin Gladiators, SemperFi Esports | 3DMAX, Astralis, FUT Esports, Legacy, Liquid, Passion UA | 3 | 3.99 | 36.8% | 0 |
| lgbm baseline | A | G2, PARIVISION | Gaimin Gladiators, SemperFi Esports | 3DMAX, Astralis, FUT Esports, Legacy, Liquid, NRG | 3 | 3.83 | 32.6% | 0 |
| lgbm baseline | B | G2, PARIVISION | Gaimin Gladiators, SemperFi Esports | 3DMAX, Astralis, FUT Esports, Legacy, Liquid, NRG | 3 | 3.83 | 32.6% | 0 |
| logreg all | A | G2, PARIVISION | Gaimin Gladiators, SemperFi Esports | 3DMAX, Astralis, FUT Esports, Heroic, Liquid, paiN | 5 | 4.08 | 39.0% | 0 |
| logreg all | B | G2, PARIVISION | Gaimin Gladiators, SemperFi Esports | 3DMAX, Astralis, FUT Esports, Heroic, Liquid, paiN | 5 | 4.08 | 39.0% | 0 |
| lgbm all | A | G2, PARIVISION | Gaimin Gladiators, SemperFi Esports | 3DMAX, Astralis, FUT Esports, Heroic, Liquid, paiN | 5 | 3.78 | 31.4% | 0 |
| lgbm all | B | G2, PARIVISION | Gaimin Gladiators, SemperFi Esports | 3DMAX, Astralis, FUT Esports, Heroic, Liquid, paiN | 5 | 3.78 | 31.4% | 0 |
| HLTV top six | — | Gaimin Gladiators, Heroic | Passion UA, SemperFi Esports | 3DMAX, Astralis, FUT Esports, G2, PARIVISION, paiN | 5 | — | — | — |
| Random slip | — | Monte, paiN | Heroic, NRG | FUT Esports, G2, Gaimin Gladiators, Legacy, M80, Passion UA | 2 | — | — | — |
| **Actual** | | FUT Esports, Legacy | NRG, SemperFi Esports | 3DMAX, Astralis, G2, Heroic, Monte, paiN | | | | |

### ESL Pro League Season 23 2026 / Stage 2

Stage 20277 opened 2026-03-06T10:30:00Z. The fit had 1,429 earlier matches, 8/8 opening lines and 13/16 archived HLTV ranks. The simulation used anchor-derived initial seeds. Every match was BO3.

| Model | Strategy | 3–0 | 0–3 | Other advance | Correct | Expected | Simulated ≥5 | Changed slots |
|---|---|---|---|---|---:|---:|---:|---:|
| Anchor | A | FURIA, MOUZ | Legacy, paiN | Astralis, Aurora Gaming, G2, Natus Vincere, Spirit, TheMongolz | 5 | 6.15 | 85.8% | 0 |
| Anchor | B | FURIA, MOUZ | Legacy, paiN | Astralis, Aurora Gaming, G2, Natus Vincere, Spirit, TheMongolz | 5 | 6.15 | 85.8% | 0 |
| VRS | A | FURIA, MOUZ | Legacy, paiN | Astralis, Aurora Gaming, G2, Natus Vincere, Spirit, TheMongolz | 5 | 6.74 | 92.9% | 0 |
| VRS | B | FURIA, MOUZ | Legacy, paiN | Astralis, Aurora Gaming, G2, Natus Vincere, Spirit, TheMongolz | 5 | 6.74 | 92.9% | 0 |
| Anchor + veto | A | FURIA, MOUZ | Legacy, paiN | Astralis, Aurora Gaming, G2, Natus Vincere, Spirit, TheMongolz | 5 | 6.15 | 85.8% | 0 |
| Anchor + veto | B | FURIA, MOUZ | Legacy, paiN | Astralis, Aurora Gaming, G2, Natus Vincere, Spirit, TheMongolz | 5 | 6.15 | 85.8% | 0 |
| logreg baseline | A | FURIA, Spirit | 3DMAX, paiN | Aurora Gaming, FaZe, G2, MOUZ, Natus Vincere, TheMongolz | 4 | 4.29 | 44.4% | 0 |
| logreg baseline | B | FURIA, Spirit | 3DMAX, paiN | Aurora Gaming, FaZe, G2, MOUZ, Natus Vincere, TheMongolz | 4 | 4.29 | 44.4% | 0 |
| lgbm baseline | A | FURIA, MOUZ | B8, paiN | Aurora Gaming, FUT Esports, G2, Natus Vincere, Spirit, TheMongolz | 5 | 4.51 | 50.0% | 0 |
| lgbm baseline | B | FURIA, MOUZ | B8, paiN | Aurora Gaming, FUT Esports, G2, Natus Vincere, Spirit, TheMongolz | 5 | 4.51 | 50.0% | 0 |
| logreg all | A | FURIA, MOUZ | Legacy, Monte | Aurora Gaming, FaZe, G2, Natus Vincere, Spirit, TheMongolz | 5 | 4.03 | 36.7% | 0 |
| logreg all | B | FURIA, MOUZ | Legacy, Monte | Aurora Gaming, FaZe, G2, Natus Vincere, Spirit, TheMongolz | 5 | 4.03 | 36.7% | 0 |
| lgbm all | A | FURIA, MOUZ | Legacy, paiN | Aurora Gaming, FaZe, G2, Natus Vincere, Spirit, TheMongolz | 4 | 4.18 | 40.4% | 0 |
| lgbm all | B | FURIA, MOUZ | B8, Legacy | Aurora Gaming, FaZe, G2, Natus Vincere, Spirit, TheMongolz | 4 | 4.17 | 40.7% | 0 |
| HLTV top six | — | Astralis, Aurora Gaming | Monte, Spirit | FURIA, FaZe, G2, MOUZ, Natus Vincere, TheMongolz | 3 | — | — | — |
| Random slip | — | MOUZ, Spirit | FaZe, G2 | 3DMAX, B8, FURIA, Heroic, Legacy, Monte | 3 | — | — | — |
| **Actual** | | MOUZ, Spirit | Heroic, Monte | Astralis, Aurora Gaming, FUT Esports, Legacy, Natus Vincere, TheMongolz | | | | |

### PGL Bucharest 2026 / Group Stage

Stage 20635 opened 2026-04-04T07:00:00Z. The fit had 1,499 earlier matches, 7/8 opening lines and 16/16 archived HLTV ranks. The simulation used anchor-derived initial seeds. Every match was BO3.

| Model | Strategy | 3–0 | 0–3 | Other advance | Correct | Expected | Simulated ≥5 | Changed slots |
|---|---|---|---|---|---:|---:|---:|---:|
| Anchor | A | PARIVISION, TheMongolz | FOKUS, Wildcard | Astralis, B8, FUT Esports, FaZe, MIBR, NRG | 2 | 6.59 | 92.4% | 0 |
| Anchor | B | PARIVISION, TheMongolz | FOKUS, Wildcard | Astralis, B8, FUT Esports, FaZe, MIBR, NRG | 2 | 6.59 | 92.4% | 0 |
| VRS | A | PARIVISION, TheMongolz | FOKUS, Wildcard | Astralis, B8, FUT Esports, FaZe, MIBR, NRG | 2 | 7.24 | 98.4% | 0 |
| VRS | B | PARIVISION, TheMongolz | FOKUS, Wildcard | 3DMAX, B8, FUT Esports, FaZe, MIBR, NRG | 3 | 7.13 | 98.4% | 0 |
| Anchor + veto | A | PARIVISION, TheMongolz | FOKUS, Wildcard | Astralis, B8, FUT Esports, FaZe, MIBR, NRG | 2 | 6.59 | 92.4% | 0 |
| Anchor + veto | B | PARIVISION, TheMongolz | FOKUS, Wildcard | Astralis, B8, FUT Esports, FaZe, MIBR, NRG | 2 | 6.59 | 92.4% | 0 |
| logreg baseline | A | PARIVISION, TheMongolz | FOKUS, Wildcard | 3DMAX, Astralis, B8, FUT Esports, FaZe, Legacy | 2 | 4.30 | 43.5% | 0 |
| logreg baseline | B | PARIVISION, TheMongolz | FOKUS, Wildcard | 3DMAX, Astralis, B8, FUT Esports, FaZe, Legacy | 2 | 4.30 | 43.5% | 0 |
| lgbm baseline | A | PARIVISION, TheMongolz | FOKUS, Wildcard | 3DMAX, Astralis, B8, FUT Esports, FaZe, NRG | 2 | 4.33 | 44.5% | 0 |
| lgbm baseline | B | PARIVISION, TheMongolz | FOKUS, Wildcard | 3DMAX, Astralis, B8, FUT Esports, FaZe, NRG | 2 | 4.33 | 44.5% | 0 |
| logreg all | A | PARIVISION, TheMongolz | FOKUS, Wildcard | 3DMAX, Astralis, B8, FUT Esports, FaZe, Legacy | 2 | 3.69 | 27.4% | 0 |
| logreg all | B | PARIVISION, TheMongolz | FOKUS, Wildcard | 3DMAX, Astralis, B8, FUT Esports, FaZe, Legacy | 2 | 3.69 | 27.4% | 0 |
| lgbm all | A | PARIVISION, TheMongolz | FOKUS, Wildcard | 3DMAX, Astralis, B8, FUT Esports, FaZe, Legacy | 2 | 4.07 | 37.3% | 0 |
| lgbm all | B | PARIVISION, TheMongolz | FOKUS, Wildcard | 3DMAX, Astralis, B8, FUT Esports, FaZe, Legacy | 2 | 4.07 | 37.3% | 0 |
| HLTV top six | — | 3DMAX, B8 | Inner Circle Esports, Wildcard | Astralis, FUT Esports, FaZe, Legacy, PARIVISION, TheMongolz | 2 | — | — | — |
| Random slip | — | MIBR, TheMongolz | EYEBALLERS, Legacy | 3DMAX, Astralis, B8, BC.Game Esports, NRG, Wildcard | 2 | — | — | — |
| **Actual** | | Astralis, FUT Esports | FaZe, Voca | 3DMAX, B8, EYEBALLERS, MIBR, PARIVISION, TheMongolz | | | | |

### PGL Astana 2026 / Group Stage

Stage 20902 opened 2026-05-09T05:00:00Z. The fit had 1,581 earlier matches, 8/8 opening lines and 16/16 archived HLTV ranks. The simulation used anchor-derived initial seeds. Every match was BO3.

| Model | Strategy | 3–0 | 0–3 | Other advance | Correct | Expected | Simulated ≥5 | Changed slots |
|---|---|---|---|---|---:|---:|---:|---:|
| Anchor | A | Spirit, Team Falcons | Fisher College, The Huns Esports | Aurora Gaming, FURIA, G2, MOUZ, PARIVISION, TheMongolz | 6 | 6.64 | 93.4% | 0 |
| Anchor | B | Spirit, Team Falcons | Fisher College, The Huns Esports | Aurora Gaming, FURIA, G2, MOUZ, PARIVISION, TheMongolz | 6 | 6.64 | 93.4% | 0 |
| VRS | A | Spirit, Team Falcons | Fisher College, The Huns Esports | Aurora Gaming, FURIA, G2, MOUZ, PARIVISION, TheMongolz | 6 | 6.74 | 93.9% | 0 |
| VRS | B | Spirit, Team Falcons | Fisher College, The Huns Esports | Aurora Gaming, FURIA, G2, MOUZ, PARIVISION, TheMongolz | 6 | 6.74 | 93.9% | 0 |
| Anchor + veto | A | Spirit, Team Falcons | Fisher College, The Huns Esports | Aurora Gaming, FURIA, G2, MOUZ, PARIVISION, TheMongolz | 6 | 6.64 | 93.4% | 0 |
| Anchor + veto | B | Spirit, Team Falcons | Fisher College, The Huns Esports | Aurora Gaming, FURIA, G2, MOUZ, PARIVISION, TheMongolz | 6 | 6.64 | 93.4% | 0 |
| logreg baseline | A | Spirit, Team Falcons | Fisher College, The Huns Esports | Aurora Gaming, FURIA, G2, MOUZ, PARIVISION, TheMongolz | 6 | 4.90 | 60.3% | 0 |
| logreg baseline | B | Spirit, Team Falcons | Fisher College, The Huns Esports | Aurora Gaming, FURIA, G2, MOUZ, PARIVISION, TheMongolz | 6 | 4.90 | 60.3% | 0 |
| lgbm baseline | A | Spirit, Team Falcons | Fisher College, The Huns Esports | Aurora Gaming, FURIA, G2, MOUZ, PARIVISION, TheMongolz | 6 | 4.93 | 61.0% | 0 |
| lgbm baseline | B | Spirit, Team Falcons | Fisher College, The Huns Esports | Aurora Gaming, FURIA, G2, MOUZ, PARIVISION, TheMongolz | 6 | 4.93 | 61.0% | 0 |
| logreg all | A | Spirit, Team Falcons | Fisher College, The Huns Esports | Aurora Gaming, FURIA, G2, MOUZ, PARIVISION, TheMongolz | 6 | 4.21 | 41.3% | 0 |
| logreg all | B | Spirit, Team Falcons | Fisher College, The Huns Esports | Aurora Gaming, FURIA, G2, MOUZ, PARIVISION, TheMongolz | 6 | 4.21 | 41.3% | 0 |
| lgbm all | A | Spirit, Team Falcons | Fisher College, The Huns Esports | Aurora Gaming, FURIA, G2, MOUZ, PARIVISION, TheMongolz | 6 | 4.42 | 47.2% | 0 |
| lgbm all | B | Spirit, Team Falcons | Fisher College, The Huns Esports | Aurora Gaming, FURIA, G2, MOUZ, PARIVISION, TheMongolz | 6 | 4.42 | 47.2% | 0 |
| HLTV top six | — | G2, MOUZ | K27, The Huns Esports | Aurora Gaming, FURIA, PARIVISION, Spirit, Team Falcons, TheMongolz | 4 | — | — | — |
| Random slip | — | Fisher College, G2 | Aurora Gaming, magic | FURIA, Gentle Mates, Heroic, Monte, PARIVISION, Team Falcons | 2 | — | — | — |
| **Actual** | | 9z, Spirit | Fisher College, K27 | Aurora Gaming, FURIA, G2, MOUZ, Team Falcons, magic | | | | |

### IEM Cologne Major 2026 / Stage 1

Stage 20708 opened 2026-06-02T10:30:00Z. The fit had 1,682 earlier matches, 8/8 opening lines and 16/16 archived HLTV ranks. The simulation used published initial seeds. Qualification and elimination matches were BO3. Earlier matches were BO1.

| Model | Strategy | 3–0 | 0–3 | Other advance | Correct | Expected | Simulated ≥5 | Changed slots |
|---|---|---|---|---|---:|---:|---:|---:|
| Anchor | A | GamerLegion, B8 | THUNDER dOWNUNDER, FlyQuest | Heroic, BetBoom Team, BIG, M80, MIBR, Lynn Vision | 4 | 5.12 | 65.9% | 0 |
| Anchor | B | GamerLegion, BetBoom Team | THUNDER dOWNUNDER, FlyQuest | B8, Heroic, BIG, M80, MIBR, Lynn Vision | 4 | 5.12 | 66.9% | 0 |
| VRS | A | GamerLegion, B8 | THUNDER dOWNUNDER, FlyQuest | Heroic, BetBoom Team, BIG, MIBR, Sinners, Lynn Vision | 3 | 5.53 | 75.3% | 0 |
| VRS | B | GamerLegion, B8 | THUNDER dOWNUNDER, FlyQuest | Heroic, BetBoom Team, BIG, MIBR, TYLOO, Lynn Vision | 4 | 5.51 | 75.5% | 0 |
| Anchor + veto | A | Heroic, BetBoom Team | Gaimin Gladiators, THUNDER dOWNUNDER | GamerLegion, B8, BIG, MIBR, TYLOO, Lynn Vision | 6 | 4.46 | 47.8% | 0 |
| Anchor + veto | B | BetBoom Team, MIBR | Gaimin Gladiators, THUNDER dOWNUNDER | GamerLegion, B8, Heroic, BIG, TYLOO, Lynn Vision | 5 | 4.46 | 48.6% | 0 |
| logreg baseline | A | GamerLegion, BIG | NRG, FlyQuest | B8, Heroic, BetBoom Team, MIBR, Sinners, Sharks | 1 | 3.58 | 25.3% | 0 |
| logreg baseline | B | GamerLegion, B8 | NRG, FlyQuest | Heroic, BetBoom Team, MIBR, Sinners, Sharks, Liquid | 2 | 3.58 | 25.9% | 0 |
| lgbm baseline | A | GamerLegion, B8 | NRG, FlyQuest | Heroic, BetBoom Team, MIBR, Sinners, TYLOO, Sharks | 3 | 3.55 | 24.8% | 1 |
| lgbm baseline | B | GamerLegion, B8 | NRG, FlyQuest | Heroic, BetBoom Team, MIBR, Sinners, Sharks, Lynn Vision | 2 | 3.55 | 25.2% | 0 |
| logreg all | A | GamerLegion, BetBoom Team | Gaimin Gladiators, FlyQuest | B8, Heroic, M80, MIBR, Sinners, TYLOO | 5 | 3.43 | 21.8% | 0 |
| logreg all | B | GamerLegion, BetBoom Team | Gaimin Gladiators, THUNDER dOWNUNDER | B8, Heroic, M80, MIBR, Sinners, TYLOO | 5 | 3.43 | 22.0% | 0 |
| lgbm all | A | GamerLegion, BetBoom Team | Gaimin Gladiators, FlyQuest | B8, Heroic, M80, MIBR, Sinners, TYLOO | 5 | 3.52 | 24.1% | 0 |
| lgbm all | B | GamerLegion, BetBoom Team | Gaimin Gladiators, FlyQuest | B8, Heroic, M80, MIBR, Sinners, TYLOO | 5 | 3.52 | 24.1% | 0 |
| HLTV top six | — | M80, TYLOO | Gaimin Gladiators, FlyQuest | GamerLegion, B8, Heroic, BetBoom Team, MIBR, Liquid | 3 | — | — | — |
| Random slip | — | GamerLegion, BetBoom Team | B8, Lynn Vision | BIG, Sinners, NRG, TYLOO, Gaimin Gladiators, THUNDER dOWNUNDER | 3 | — | — | — |
| **Actual** | | B8, BetBoom Team | Sinners, Gaimin Gladiators | GamerLegion, BIG, M80, MIBR, TYLOO, FlyQuest | | | | |

### IEM Cologne Major 2026 / Stage 2

Stage 20709 opened 2026-06-06T10:30:00Z. The fit had 1,715 earlier matches, 8/8 opening lines and 16/16 archived HLTV ranks. The simulation used published initial seeds. Qualification and elimination matches were BO3. Earlier matches were BO1.

| Model | Strategy | 3–0 | 0–3 | Other advance | Correct | Expected | Simulated ≥5 | Changed slots |
|---|---|---|---|---|---:|---:|---:|---:|
| Anchor | A | Spirit, Legacy | M80, FlyQuest | FUT Esports, Astralis, G2, 9z, B8, GamerLegion | 5 | 5.59 | 77.5% | 0 |
| Anchor | B | Spirit, Legacy | M80, FlyQuest | FUT Esports, Astralis, G2, 9z, B8, GamerLegion | 5 | 5.59 | 77.5% | 0 |
| VRS | A | Spirit, Legacy | TYLOO, FlyQuest | FUT Esports, Astralis, G2, 9z, B8, GamerLegion | 5 | 6.18 | 88.9% | 0 |
| VRS | B | Spirit, Legacy | M80, FlyQuest | FUT Esports, Astralis, G2, 9z, B8, GamerLegion | 5 | 6.16 | 88.9% | 1 |
| Anchor + veto | A | Spirit, B8 | TYLOO, FlyQuest | FUT Esports, Astralis, G2, Legacy, 9z, GamerLegion | 5 | 4.08 | 37.7% | 1 |
| Anchor + veto | B | Spirit, BetBoom Team | TYLOO, FlyQuest | FUT Esports, Astralis, G2, Legacy, 9z, GamerLegion | 5 | 4.08 | 38.0% | 1 |
| logreg baseline | A | FUT Esports, Spirit | M80, FlyQuest | Astralis, G2, Legacy, B8, BetBoom Team, GamerLegion | 7 | 3.89 | 32.4% | 0 |
| logreg baseline | B | Spirit, Legacy | M80, FlyQuest | FUT Esports, Astralis, G2, 9z, B8, GamerLegion | 5 | 3.88 | 33.0% | 2 |
| lgbm baseline | A | FUT Esports, Spirit | M80, FlyQuest | Astralis, G2, Legacy, 9z, B8, GamerLegion | 7 | 4.06 | 37.4% | 0 |
| lgbm baseline | B | Spirit, 9z | M80, FlyQuest | FUT Esports, Astralis, G2, Legacy, B8, GamerLegion | 5 | 4.05 | 37.7% | 0 |
| logreg all | A | Spirit, 9z | M80, FlyQuest | FUT Esports, Astralis, G2, Legacy, B8, GamerLegion | 5 | 3.43 | 21.6% | 0 |
| logreg all | B | Spirit, G2 | M80, FlyQuest | FUT Esports, Astralis, Legacy, 9z, B8, GamerLegion | 5 | 3.42 | 21.7% | 0 |
| lgbm all | A | Spirit, 9z | BIG, FlyQuest | FUT Esports, Astralis, G2, Legacy, B8, GamerLegion | 5 | 3.81 | 31.3% | 0 |
| lgbm all | B | Spirit, 9z | BIG, FlyQuest | FUT Esports, Astralis, G2, Legacy, B8, GamerLegion | 5 | 3.81 | 31.3% | 0 |
| HLTV top six | — | paiN, B8 | BIG, FlyQuest | FUT Esports, Spirit, Astralis, G2, Legacy, GamerLegion | 3 | — | — | — |
| Random slip | — | BetBoom Team, FlyQuest | Spirit, B8 | Astralis, Monte, 9z, GamerLegion, MIBR, BIG | 2 | — | — | — |
| **Actual** | | FUT Esports, Spirit | GamerLegion, FlyQuest | G2, Legacy, Monte, 9z, B8, BetBoom Team | | | | |

### IEM Cologne Major 2026 / Stage 3

Stage 21115 opened 2026-06-11T09:00:00Z. The fit had 1,748 earlier matches, 8/8 opening lines and 16/16 archived HLTV ranks. The simulation used published initial seeds. Every match was BO3.

| Model | Strategy | 3–0 | 0–3 | Other advance | Correct | Expected | Simulated ≥5 | Changed slots |
|---|---|---|---|---|---:|---:|---:|---:|
| Anchor | A | Vitality, Team Falcons | Monte, B8 | Natus Vincere, TheMongolz, Aurora Gaming, MOUZ, Spirit, Legacy | 2 | 5.46 | 74.2% | 0 |
| Anchor | B | Vitality, Spirit | Monte, B8 | Natus Vincere, Team Falcons, TheMongolz, Aurora Gaming, MOUZ, Legacy | 4 | 5.45 | 74.7% | 0 |
| VRS | A | Vitality, Team Falcons | BetBoom Team, Monte | Natus Vincere, TheMongolz, Aurora Gaming, MOUZ, Spirit, Legacy | 1 | 5.90 | 82.8% | 0 |
| VRS | B | Vitality, Team Falcons | BetBoom Team, Monte | Natus Vincere, TheMongolz, Aurora Gaming, MOUZ, Spirit, Legacy | 1 | 5.90 | 82.8% | 0 |
| Anchor + veto | A | Vitality, Team Falcons | Monte, B8 | Natus Vincere, TheMongolz, Aurora Gaming, MOUZ, Spirit, Legacy | 2 | 5.46 | 74.2% | 0 |
| Anchor + veto | B | Vitality, Spirit | Monte, B8 | Natus Vincere, Team Falcons, TheMongolz, Aurora Gaming, MOUZ, Legacy | 4 | 5.45 | 74.7% | 0 |
| logreg baseline | A | Vitality, Spirit | Monte, B8 | Natus Vincere, Team Falcons, TheMongolz, FURIA, MOUZ, Legacy | 3 | 4.12 | 39.3% | 0 |
| logreg baseline | B | Vitality, Spirit | Monte, B8 | Natus Vincere, Team Falcons, TheMongolz, FURIA, MOUZ, Legacy | 3 | 4.12 | 39.3% | 0 |
| lgbm baseline | A | Vitality, Spirit | Monte, B8 | Natus Vincere, Team Falcons, TheMongolz, FURIA, MOUZ, Legacy | 3 | 4.36 | 45.8% | 0 |
| lgbm baseline | B | Vitality, Spirit | Monte, B8 | Natus Vincere, Team Falcons, TheMongolz, FURIA, MOUZ, Legacy | 3 | 4.36 | 45.8% | 0 |
| logreg all | A | Vitality, FURIA | Monte, B8 | Natus Vincere, Team Falcons, TheMongolz, Aurora Gaming, MOUZ, Spirit | 4 | 3.72 | 28.7% | 0 |
| logreg all | B | Vitality, FURIA | Monte, B8 | Natus Vincere, Team Falcons, TheMongolz, Aurora Gaming, MOUZ, Spirit | 4 | 3.72 | 28.7% | 0 |
| lgbm all | A | Vitality, Team Falcons | Monte, B8 | Natus Vincere, TheMongolz, FURIA, MOUZ, Spirit, Legacy | 1 | 4.07 | 37.8% | 0 |
| lgbm all | B | Vitality, Team Falcons | Monte, B8 | Natus Vincere, TheMongolz, FURIA, MOUZ, Spirit, Legacy | 1 | 4.07 | 37.8% | 0 |
| HLTV top six | — | MOUZ, Legacy | 9z, Monte | Vitality, Natus Vincere, Team Falcons, Aurora Gaming, FURIA, Spirit | 3 | — | — | — |
| Random slip | — | PARIVISION, 9z | TheMongolz, FUT Esports | Vitality, Natus Vincere, FURIA, G2, Monte, B8 | 2 | — | — | — |
| **Actual** | | FURIA, Spirit | PARIVISION, B8 | Vitality, Team Falcons, Aurora Gaming, G2, BetBoom Team, 9z | | | | |

### XSE Pro League 2026 / Group Stage

Stage 21197 opened 2026-07-01T08:46:51Z. The fit had 1,788 earlier matches, 0/8 opening lines and 16/16 archived HLTV ranks. The simulation used anchor-derived initial seeds. Qualification and elimination matches were BO3. Earlier matches were BO1.

| Model | Strategy | 3–0 | 0–3 | Other advance | Correct | Expected | Simulated ≥5 | Changed slots |
|---|---|---|---|---|---:|---:|---:|---:|
| Anchor | A | 9z, PARIVISION | EYEBALLERS, Sinners | B8, BetBoom Team, FaZe, Luminosity, MIBR, NIP | 2 | 4.57 | 51.9% | 0 |
| Anchor | B | 9z, PARIVISION | EYEBALLERS, Sinners | B8, BetBoom Team, FaZe, Luminosity, MIBR, NIP | 2 | 4.57 | 51.9% | 0 |
| VRS | A | 9z, PARIVISION | EYEBALLERS, Sinners | B8, BetBoom Team, FaZe, Luminosity, MIBR, NIP | 2 | 4.57 | 51.9% | 0 |
| VRS | B | 9z, PARIVISION | EYEBALLERS, Sinners | B8, BetBoom Team, FaZe, Luminosity, MIBR, NIP | 2 | 4.57 | 51.9% | 0 |
| Anchor + veto | A | Luminosity, PARIVISION | Alliance, Team Nemesis | 9z, B8, BetBoom Team, FaZe, MIBR, NIP | 2 | 3.98 | 35.6% | 0 |
| Anchor + veto | B | Luminosity, PARIVISION | Alliance, Team Nemesis | 9z, B8, BetBoom Team, FaZe, MIBR, NIP | 2 | 3.98 | 35.6% | 0 |
| logreg baseline | A | 9z, BetBoom Team | Lynn Vision, Sinners | Alliance, B8, FaZe, Luminosity, MIBR, PARIVISION | 5 | 3.25 | 18.1% | 0 |
| logreg baseline | B | 9z, BetBoom Team | EYEBALLERS, Sinners | Alliance, B8, FaZe, Luminosity, MIBR, PARIVISION | 5 | 3.25 | 18.5% | 0 |
| lgbm baseline | A | 9z, BIG | EYEBALLERS, Sinners | B8, BetBoom Team, FaZe, Luminosity, MIBR, PARIVISION | 3 | 3.36 | 19.8% | 0 |
| lgbm baseline | B | 9z, BetBoom Team | EYEBALLERS, Sinners | Alliance, B8, FaZe, Luminosity, MIBR, PARIVISION | 5 | 3.35 | 20.9% | 0 |
| logreg all | A | 9z, PARIVISION | Alliance, Team Nemesis | 3DMAX, BetBoom Team, FaZe, Luminosity, NIP, Sinners | 1 | 3.15 | 15.8% | 0 |
| logreg all | B | FaZe, PARIVISION | Alliance, TYLOO | 3DMAX, 9z, BetBoom Team, Luminosity, NIP, Sinners | 1 | 3.14 | 16.1% | 0 |
| lgbm all | A | 9z, Luminosity | EYEBALLERS, Team Nemesis | 3DMAX, B8, BetBoom Team, FaZe, MIBR, PARIVISION | 2 | 3.38 | 20.9% | 2 |
| lgbm all | B | 9z, BetBoom Team | EYEBALLERS, Team Nemesis | 3DMAX, B8, FaZe, Luminosity, MIBR, PARIVISION | 3 | 3.37 | 21.1% | 0 |
| HLTV top six | — | MIBR, TYLOO | Sinners, Team Nemesis | 9z, B8, BetBoom Team, FaZe, Luminosity, PARIVISION | 5 | — | — | — |
| Random slip | — | 9z, EYEBALLERS | Lynn Vision, Sinners | 3DMAX, Alliance, B8, BIG, MIBR, TYLOO | 3 | — | — | — |
| **Actual** | | BetBoom Team, TYLOO | Luminosity, Sinners | 9z, Alliance, BIG, FaZe, PARIVISION, Team Nemesis | | | | |

### ESL Pro League Season 24 2026 / Group Stage

Stage 22017 opened 2026-10-03T09:00:00Z. The fit had 1,987 earlier matches, 8/8 opening lines and 16/16 archived HLTV ranks. The simulation used anchor-derived initial seeds. Every match was BO3.

| Model | Strategy | 3–0 | 0–3 | Other advance | Correct | Expected | Simulated ≥5 | Changed slots |
|---|---|---|---|---|---:|---:|---:|---:|
| Anchor | A | Spirit, Team Falcons | 1WIN, ShindeN | 9z, FURIA, G2, Legacy, MOUZ, Vitality | 2 | 6.67 | 93.4% | 0 |
| Anchor | B | MOUZ, Spirit | M80, ShindeN | 9z, FURIA, G2, Legacy, Team Falcons, Vitality | 4 | 6.39 | 96.4% | 0 |
| VRS | A | Spirit, Team Falcons | 1WIN, ShindeN | 9z, FURIA, G2, Legacy, MOUZ, Vitality | 2 | 6.83 | 94.3% | 0 |
| VRS | B | MOUZ, Spirit | M80, ShindeN | 9z, FURIA, G2, Legacy, Team Falcons, Vitality | 4 | 6.54 | 97.9% | 0 |
| Anchor + veto | A | Spirit, Team Falcons | 1WIN, ShindeN | 9z, FURIA, G2, Legacy, MOUZ, Vitality | 2 | 6.67 | 93.4% | 0 |
| Anchor + veto | B | MOUZ, Spirit | M80, ShindeN | 9z, FURIA, G2, Legacy, Team Falcons, Vitality | 4 | 6.39 | 96.4% | 0 |
| logreg baseline | A | Spirit, Vitality | M80, ShindeN | Aurora Gaming, FURIA, G2, Legacy, MOUZ, Team Falcons | 5 | 5.05 | 65.1% | 0 |
| logreg baseline | B | MOUZ, Spirit | M80, ShindeN | Aurora Gaming, FURIA, G2, Legacy, Team Falcons, Vitality | 5 | 4.97 | 65.6% | 0 |
| lgbm baseline | A | Spirit, Vitality | M80, ShindeN | Aurora Gaming, FURIA, G2, Legacy, MOUZ, Team Falcons | 5 | 4.89 | 60.3% | 0 |
| lgbm baseline | B | MOUZ, Spirit | M80, ShindeN | Aurora Gaming, FURIA, G2, Legacy, Team Falcons, Vitality | 5 | 4.86 | 61.8% | 0 |
| logreg all | A | Spirit, Vitality | 1WIN, ShindeN | Aurora Gaming, FURIA, G2, Legacy, MOUZ, Team Falcons | 5 | 4.21 | 42.1% | 0 |
| logreg all | B | Spirit, Vitality | 1WIN, ShindeN | Aurora Gaming, FURIA, G2, Legacy, MOUZ, Team Falcons | 5 | 4.21 | 42.1% | 0 |
| lgbm all | A | Spirit, Vitality | M80, ShindeN | Aurora Gaming, FURIA, G2, Legacy, MOUZ, Team Falcons | 5 | 4.43 | 47.7% | 0 |
| lgbm all | B | MOUZ, Spirit | M80, ShindeN | Aurora Gaming, FURIA, G2, Legacy, Team Falcons, Vitality | 5 | 4.40 | 48.3% | 0 |
| HLTV top six | — | Aurora Gaming, G2 | 1WIN, ShindeN | FURIA, Legacy, MOUZ, Spirit, Team Falcons, Vitality | 4 | — | — | — |
| Random slip | — | 1WIN, 9z | PARIVISION, Team Falcons | Aurora Gaming, BetBoom Team, FURIA, MOUZ, Natus Vincere, Spirit | 3 | — | — | — |
| **Actual** | | MOUZ, Vitality | ShindeN, TYLOO | 1WIN, Aurora Gaming, FURIA, PARIVISION, Spirit, Team Falcons | | | | |

### PGL Masters Bucharest 2026 / Group Stage

Stage 22222 opened 2026-10-24T07:00:00Z. The fit had 2,042 earlier matches, 8/8 opening lines and 11/16 archived HLTV ranks. The simulation used anchor-derived initial seeds. Every match was BO3.

| Model | Strategy | 3–0 | 0–3 | Other advance | Correct | Expected | Simulated ≥5 | Changed slots |
|---|---|---|---|---|---:|---:|---:|---:|
| Anchor | A | Legacy, Spirit | TYLOO, TheMongolz | Aurora Gaming, FURIA, FUT Esports, G2, MOUZ, Vitality | 4 | 6.59 | 91.5% | 0 |
| Anchor | B | Legacy, Spirit | TYLOO, TheMongolz | Aurora Gaming, FURIA, FUT Esports, G2, MOUZ, Vitality | 4 | 6.59 | 91.5% | 0 |
| VRS | A | Legacy, Spirit | TYLOO, TheMongolz | Aurora Gaming, FURIA, FUT Esports, G2, MOUZ, Vitality | 4 | 7.03 | 95.5% | 0 |
| VRS | B | Legacy, Spirit | TYLOO, TheMongolz | Aurora Gaming, FURIA, FUT Esports, G2, MOUZ, Vitality | 4 | 7.03 | 95.5% | 0 |
| Anchor + veto | A | Legacy, Spirit | TYLOO, TheMongolz | Aurora Gaming, FURIA, FUT Esports, G2, MOUZ, Vitality | 4 | 6.59 | 91.5% | 0 |
| Anchor + veto | B | Legacy, Spirit | TYLOO, TheMongolz | Aurora Gaming, FURIA, FUT Esports, G2, MOUZ, Vitality | 4 | 6.59 | 91.5% | 0 |
| logreg baseline | A | Spirit, Vitality | DENDELE CS, TYLOO | Aurora Gaming, FURIA, FUT Esports, G2, Legacy, MOUZ | 3 | 4.81 | 57.5% | 0 |
| logreg baseline | B | Spirit, Vitality | DENDELE CS, TYLOO | Aurora Gaming, FURIA, FUT Esports, G2, Legacy, MOUZ | 3 | 4.81 | 57.5% | 0 |
| lgbm baseline | A | Spirit, Vitality | DENDELE CS, TYLOO | Aurora Gaming, FURIA, FUT Esports, G2, Legacy, MOUZ | 3 | 4.95 | 61.2% | 0 |
| lgbm baseline | B | Spirit, Vitality | DENDELE CS, TYLOO | Aurora Gaming, FURIA, FUT Esports, G2, Legacy, MOUZ | 3 | 4.95 | 61.2% | 0 |
| logreg all | A | Spirit, Vitality | DENDELE CS, TheMongolz | Aurora Gaming, BetBoom Team, FURIA, G2, Legacy, MOUZ | 3 | 4.20 | 40.4% | 0 |
| logreg all | B | Spirit, Vitality | DENDELE CS, TheMongolz | Aurora Gaming, BetBoom Team, FURIA, G2, Legacy, MOUZ | 3 | 4.20 | 40.4% | 0 |
| lgbm all | A | Spirit, Vitality | DENDELE CS, TheMongolz | Aurora Gaming, BetBoom Team, FURIA, G2, Legacy, MOUZ | 3 | 4.49 | 48.3% | 0 |
| lgbm all | B | Spirit, Vitality | DENDELE CS, TheMongolz | Aurora Gaming, FURIA, FUT Esports, G2, Legacy, MOUZ | 3 | 4.48 | 48.5% | 0 |
| HLTV top six | — | Legacy, Natus Vincere | Nemiga, TheMongolz | Aurora Gaming, FURIA, G2, MOUZ, Spirit, Vitality | 4 | — | — | — |
| Random slip | — | MOUZ, Spirit | Nemiga, TYLOO | 9z, Aurora Gaming, BetBoom Team, FUT Esports, TheMongolz, Vitality | 2 | — | — | — |
| **Actual** | | BetBoom Team, FUT Esports | 9z, G2 | Aurora Gaming, FURIA, MOUZ, Natus Vincere, Nemiga, Vitality | | | | |

### PGL Major Singapore 2026 / Stage 1

Stage 22378 opened 2026-11-24T04:00:00Z. The fit had 2,126 earlier matches, 8/8 opening lines and 5/16 archived HLTV ranks. The simulation used published initial seeds. Qualification and elimination matches were BO3. Earlier matches were BO1.

| Model | Strategy | 3–0 | 0–3 | Other advance | Correct | Expected | Simulated ≥5 | Changed slots |
|---|---|---|---|---|---:|---:|---:|---:|
| Anchor | A | NIP, Heroic | Chinggis Warriors, Lynn Vision | PVISION, FaZe, Liquid, Eternal Fire, 100 Thieves, M80 | 5 | 4.55 | 50.2% | 0 |
| Anchor | B | PVISION, Heroic | Chinggis Warriors, Lynn Vision | NIP, FaZe, Liquid, Eternal Fire, 100 Thieves, M80 | 3 | 4.54 | 50.3% | 0 |
| VRS | A | NIP, PVISION | Chinggis Warriors, Lynn Vision | Heroic, FaZe, Liquid, Eternal Fire, 100 Thieves, 9z | 4 | 4.61 | 51.5% | 0 |
| VRS | B | NIP, PVISION | Chinggis Warriors, Lynn Vision | Heroic, FaZe, Liquid, Eternal Fire, 100 Thieves, M80 | 4 | 4.61 | 51.7% | 0 |
| Anchor + veto | A | FaZe, Eternal Fire | GOSH, Lynn Vision | NIP, PVISION, Heroic, Liquid, 100 Thieves, M80 | 4 | 3.71 | 27.8% | 2 |
| Anchor + veto | B | FaZe, Eternal Fire | GOSH, Lynn Vision | NIP, PVISION, Heroic, Liquid, 100 Thieves, M80 | 4 | 3.71 | 27.8% | 0 |
| logreg baseline | A | FaZe, Eternal Fire | Chinggis Warriors, Lynn Vision | NIP, PVISION, Heroic, Liquid, 100 Thieves, 9z | 3 | 3.53 | 23.8% | 0 |
| logreg baseline | B | FaZe, Eternal Fire | Chinggis Warriors, Lynn Vision | NIP, PVISION, Heroic, Liquid, 100 Thieves, 9z | 3 | 3.53 | 23.8% | 0 |
| lgbm baseline | A | Heroic, FaZe | Chinggis Warriors, Lynn Vision | NIP, PVISION, Liquid, Eternal Fire, 100 Thieves, 9z | 4 | 3.65 | 26.0% | 0 |
| lgbm baseline | B | FaZe, Eternal Fire | FlyQuest, Lynn Vision | NIP, PVISION, Heroic, Liquid, 100 Thieves, 9z | 3 | 3.64 | 26.4% | 0 |
| logreg all | A | PVISION, FaZe | Chinggis Warriors, Lynn Vision | NIP, Heroic, Liquid, Eternal Fire, 100 Thieves, 9z | 3 | 3.39 | 20.5% | 0 |
| logreg all | B | PVISION, FaZe | Wildcard, Chinggis Warriors | NIP, Heroic, Liquid, Eternal Fire, 100 Thieves, 9z | 3 | 3.37 | 20.5% | 0 |
| lgbm all | A | PVISION, FaZe | Wildcard, Lynn Vision | NIP, Heroic, Liquid, Eternal Fire, 100 Thieves, 9z | 3 | 3.53 | 23.8% | 0 |
| lgbm all | B | PVISION, FaZe | Wildcard, Lynn Vision | NIP, Heroic, Liquid, Eternal Fire, 100 Thieves, 9z | 3 | 3.53 | 23.8% | 0 |
| HLTV top six | — | Eternal Fire, 100 Thieves | Chinggis Warriors, Lynn Vision | NIP, PVISION, Heroic, FaZe, Liquid, 9z | 2 | — | — | — |
| Random slip | — | Liquid, M80 | 9z, GOSH | NIP, PVISION, Eternal Fire, 100 Thieves, Wildcard, Chinggis Warriors | 4 | — | — | — |
| **Actual** | | NIP, FlyQuest | DENDELE CS, GOSH | PVISION, Liquid, Eternal Fire, 100 Thieves, Imperial, Lynn Vision | | | | |
