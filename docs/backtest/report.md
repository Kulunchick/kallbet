# Retrospective benchmark

Archive: 272 files, SHA-256 `25459d150e1fac87409e38d17a73ab2110444401703a056b1c4d4c846c4df782`. Regenerate with `cargo run -p kallbet-ai -- backtest` (see `docs/agents/codebase.md`, ticket #31).

Profiles:
- `vrs`: VrsOnly
- `market`: MarketOnly
- `anchor`: HybridAnchored without LLM (the anchor)
- `single`: Single LLM agent (HybridAnchored, debate mode `never`)
- `debate`: 3-agent AdversarialDebateLlm (HybridAnchored, debate mode `always`)

## Data per Stage

| Stage | Kind | Finished | Scored Matches | Not scored | VRS | Market | Rosters | Pick'em view |
|---|---|---|--:|--:|--:|--:|--:|---|
| PGL Major Copenhagen 2024 / Opening Stage | Swiss | yes | 33 | 1 | 0 | 0 | 0 | Liquipedia revision before the Stage |
| PGL Major Copenhagen 2024 / Elimination Stage | Swiss | yes | 33 | 0 | 0 | 0 | 0 | Liquipedia revision before the Stage |
| PGL Major Copenhagen 2024 / Playoffs | Playoffs | yes | 7 | 0 | 0 | 0 | 0 | Liquipedia revision before the Stage |
| Perfect World Major Shanghai Major 2024 / Opening Stage | Swiss | yes | 33 | 0 | 33 | 0 | 33 | Liquipedia revision before the Stage |
| Perfect World Major Shanghai Major 2024 / Elimination Stage | Swiss | yes | 33 | 0 | 33 | 0 | 33 | Liquipedia revision before the Stage |
| Perfect World Major Shanghai Major 2024 / Playoffs | Playoffs | yes | 7 | 0 | 7 | 0 | 7 | Liquipedia revision before the Stage |
| BLAST.tv Major Austin 2025 / Stage 1 | Swiss | yes | 33 | 3 | 33 | 0 | 0 | Liquipedia revision before the Stage |
| BLAST.tv Major Austin 2025 / Stage 2 | Swiss | yes | 33 | 0 | 33 | 0 | 0 | Liquipedia revision before the Stage |
| BLAST.tv Major Austin 2025 / Stage 3 | Swiss | yes | 33 | 0 | 33 | 0 | 0 | Liquipedia revision before the Stage |
| BLAST.tv Major Austin 2025 / Playoffs | Playoffs | yes | 7 | 0 | 7 | 0 | 0 | Liquipedia revision before the Stage |
| StarLadder Major Budapest 2025 / Stage 1 | Swiss | yes | 33 | 0 | 33 | 0 | 0 | Liquipedia revision before the Stage |
| StarLadder Major Budapest 2025 / Stage 2 | Swiss | yes | 33 | 0 | 33 | 0 | 0 | Liquipedia revision before the Stage |
| StarLadder Major Budapest 2025 / Stage 3 | Swiss | yes | 33 | 0 | 33 | 0 | 0 | Liquipedia revision before the Stage |
| StarLadder Major Budapest 2025 / Playoffs | Playoffs | yes | 7 | 0 | 7 | 0 | 0 | Liquipedia revision before the Stage |
| IEM Cologne Major 2026 / Stage 1 | Swiss | yes | 33 | 0 | 33 | 21 | 33 | Liquipedia revision before the Stage |
| IEM Cologne Major 2026 / Stage 2 | Swiss | yes | 33 | 0 | 33 | 27 | 33 | Liquipedia revision before the Stage |
| IEM Cologne Major 2026 / Stage 3 | Swiss | yes | 33 | 0 | 33 | 33 | 33 | Liquipedia revision before the Stage |
| IEM Cologne Major 2026 / Playoffs | Playoffs | yes | 7 | 0 | 7 | 7 | 7 | Liquipedia revision before the Stage |
| PGL Major Singapore 2026 / Stage 1 | Swiss | no | 24 | 9 | 24 | 24 | 24 | Liquipedia revision before the Stage |

## Non-LLM track: every completed Match

Each profile on every Match it predicted. VrsOnly needs both teams in a Valve snapshot before the cutoff, MarketOnly a pre-match market line (OddsPapi, else Polymarket, else Kalshi; without one it falls back to VRS, so its rows include those Matches), the anchor either.

| Profile | Format | Matches | Correct | Accuracy | Log-loss | Brier | Overrides | Override hits |
|---|---|--:|--:|--:|--:|--:|--:|--:|
| VrsOnly | all | 415 | 255 | 61.4% | 0.775 | 0.258 | 0 | – |
| VrsOnly | bo1 | 220 | 127 | 57.7% | 0.808 | 0.277 | 0 | – |
| VrsOnly | bo3 | 193 | 126 | 65.3% | 0.747 | 0.240 | 0 | – |
| VrsOnly | bo5 | 2 | 2 | 100.0% | 0.043 | 0.002 | 0 | – |
| MarketOnly | all | 415 | 258 | 62.2% | 0.748 | 0.248 | 0 | – |
| MarketOnly | bo1 | 220 | 128 | 58.2% | 0.787 | 0.268 | 0 | – |
| MarketOnly | bo3 | 193 | 128 | 66.3% | 0.710 | 0.227 | 0 | – |
| MarketOnly | bo5 | 2 | 2 | 100.0% | 0.287 | 0.091 | 0 | – |
| HybridAnchored without LLM (the anchor) | all | 415 | 258 | 62.2% | 0.748 | 0.249 | 6 | 3 (50.0%) |
| HybridAnchored without LLM (the anchor) | bo1 | 220 | 129 | 58.6% | 0.788 | 0.269 | 3 | 2 (66.7%) |
| HybridAnchored without LLM (the anchor) | bo3 | 193 | 127 | 65.8% | 0.709 | 0.227 | 3 | 1 (33.3%) |
| HybridAnchored without LLM (the anchor) | bo5 | 2 | 2 | 100.0% | 0.199 | 0.050 | 0 | – |

## Non-LLM track: Matches with both a market line and a rating

The same profiles, only on the Matches where every one of them predicted and the dossier held both a market line and a VRS rating.

| Profile | Format | Matches | Correct | Accuracy | Log-loss | Brier | Overrides | Override hits |
|---|---|--:|--:|--:|--:|--:|--:|--:|
| VrsOnly | all | 112 | 67 | 59.8% | 0.734 | 0.258 | 0 | – |
| VrsOnly | bo1 | 49 | 27 | 55.1% | 0.760 | 0.276 | 0 | – |
| VrsOnly | bo3 | 62 | 39 | 62.9% | 0.723 | 0.248 | 0 | – |
| VrsOnly | bo5 | 1 | 1 | 100.0% | 0.067 | 0.004 | 0 | – |
| MarketOnly | all | 112 | 70 | 62.5% | 0.633 | 0.221 | 0 | – |
| MarketOnly | bo1 | 49 | 28 | 57.1% | 0.666 | 0.238 | 0 | – |
| MarketOnly | bo3 | 62 | 41 | 66.1% | 0.609 | 0.208 | 0 | – |
| MarketOnly | bo5 | 1 | 1 | 100.0% | 0.555 | 0.181 | 0 | – |
| HybridAnchored without LLM (the anchor) | all | 112 | 70 | 62.5% | 0.634 | 0.222 | 6 | 3 (50.0%) |
| HybridAnchored without LLM (the anchor) | bo1 | 49 | 29 | 59.2% | 0.671 | 0.240 | 3 | 2 (66.7%) |
| HybridAnchored without LLM (the anchor) | bo3 | 62 | 40 | 64.5% | 0.608 | 0.209 | 3 | 1 (33.3%) |
| HybridAnchored without LLM (the anchor) | bo5 | 1 | 1 | 100.0% | 0.380 | 0.100 | 0 | – |

## Market sources on the post-cutoff Matches

The favourite of each source's last pre-cutoff price (Pinnacle de-vigged; Polymarket's last trade price; Kalshi's two bid/ask mids normalised), on the Matches every source priced. Not a profile: no policy, no fallback.

| Profile | Format | Matches | Correct | Accuracy | Log-loss | Brier | Overrides | Override hits |
|---|---|--:|--:|--:|--:|--:|--:|--:|
| OddsPapi (Pinnacle) | all | 19 | 7 | 36.8% | 0.764 | 0.285 | 0 | – |
| OddsPapi (Pinnacle) | bo1 | 16 | 7 | 43.8% | 0.769 | 0.288 | 0 | – |
| OddsPapi (Pinnacle) | bo3 | 3 | 0 | 0.0% | 0.733 | 0.270 | 0 | – |
| Polymarket | all | 19 | 8 | 42.1% | 0.763 | 0.285 | 0 | – |
| Polymarket | bo1 | 16 | 8 | 50.0% | 0.768 | 0.287 | 0 | – |
| Polymarket | bo3 | 3 | 0 | 0.0% | 0.738 | 0.272 | 0 | – |
| Kalshi | all | 19 | 9 | 47.4% | 0.758 | 0.282 | 0 | – |
| Kalshi | bo1 | 16 | 8 | 50.0% | 0.765 | 0.286 | 0 | – |
| Kalshi | bo3 | 3 | 1 | 33.3% | 0.717 | 0.262 | 0 | – |

## LLM track: post-cutoff Matches, real names

All five profiles, only on the Matches every one of them predicted.

| Profile | Format | Matches | Correct | Accuracy | Log-loss | Brier | Overrides | Override hits |
|---|---|--:|--:|--:|--:|--:|--:|--:|
| VrsOnly | all | 24 | 13 | 54.2% | 0.760 | 0.280 | 0 | – |
| VrsOnly | bo1 | 20 | 11 | 55.0% | 0.747 | 0.274 | 0 | – |
| VrsOnly | bo3 | 4 | 2 | 50.0% | 0.822 | 0.311 | 0 | – |
| MarketOnly | all | 24 | 11 | 45.8% | 0.749 | 0.275 | 0 | – |
| MarketOnly | bo1 | 20 | 11 | 55.0% | 0.702 | 0.256 | 0 | – |
| MarketOnly | bo3 | 4 | 0 | 0.0% | 0.986 | 0.373 | 0 | – |
| HybridAnchored without LLM (the anchor) | all | 24 | 12 | 50.0% | 0.747 | 0.275 | 1 | 1 (100.0%) |
| HybridAnchored without LLM (the anchor) | bo1 | 20 | 11 | 55.0% | 0.707 | 0.258 | 0 | – |
| HybridAnchored without LLM (the anchor) | bo3 | 4 | 1 | 25.0% | 0.946 | 0.361 | 1 | 1 (100.0%) |
| Single LLM agent (HybridAnchored, debate mode `never`) | all | 24 | 11 | 45.8% | 0.744 | 0.274 | 0 | – |
| Single LLM agent (HybridAnchored, debate mode `never`) | bo1 | 20 | 11 | 55.0% | 0.700 | 0.255 | 0 | – |
| Single LLM agent (HybridAnchored, debate mode `never`) | bo3 | 4 | 0 | 0.0% | 0.961 | 0.368 | 0 | – |
| 3-agent AdversarialDebateLlm (HybridAnchored, debate mode `always`) | all | 24 | 11 | 45.8% | 0.745 | 0.274 | 0 | – |
| 3-agent AdversarialDebateLlm (HybridAnchored, debate mode `always`) | bo1 | 20 | 11 | 55.0% | 0.703 | 0.256 | 0 | – |
| 3-agent AdversarialDebateLlm (HybridAnchored, debate mode `always`) | bo3 | 4 | 0 | 0.0% | 0.954 | 0.364 | 0 | – |

## LLM track: anonymized historical slice (BlindAnonymizer)

All five profiles, only on the Matches every one of them predicted. The LLM seats saw TEAM_ALPHA/TEAM_BETA, PLAYER_01..10 for the rosters of all 6 Matches, no news item, no Event title and no dates.

| Profile | Format | Matches | Correct | Accuracy | Log-loss | Brier | Overrides | Override hits |
|---|---|--:|--:|--:|--:|--:|--:|--:|
| VrsOnly | all | 6 | 2 | 33.3% | 0.989 | 0.366 | 0 | – |
| VrsOnly | bo1 | 6 | 2 | 33.3% | 0.989 | 0.366 | 0 | – |
| MarketOnly | all | 6 | 3 | 50.0% | 0.789 | 0.292 | 0 | – |
| MarketOnly | bo1 | 6 | 3 | 50.0% | 0.789 | 0.292 | 0 | – |
| HybridAnchored without LLM (the anchor) | all | 6 | 3 | 50.0% | 0.812 | 0.302 | 0 | – |
| HybridAnchored without LLM (the anchor) | bo1 | 6 | 3 | 50.0% | 0.812 | 0.302 | 0 | – |
| Single LLM agent (HybridAnchored, debate mode `never`) | all | 6 | 3 | 50.0% | 0.812 | 0.302 | 0 | – |
| Single LLM agent (HybridAnchored, debate mode `never`) | bo1 | 6 | 3 | 50.0% | 0.812 | 0.302 | 0 | – |
| 3-agent AdversarialDebateLlm (HybridAnchored, debate mode `always`) | all | 6 | 3 | 50.0% | 0.812 | 0.302 | 0 | – |
| 3-agent AdversarialDebateLlm (HybridAnchored, debate mode `always`) | bo1 | 6 | 3 | 50.0% | 0.812 | 0.302 | 0 | – |

## Where an LLM moved the anchor

Probabilities are for the first-named team. Factors are the ones the policy accepted; none means the tie-break of at most 2 points near 50/50.

| Profile | Match | Anchor | With the LLM | Pick | Winner | Factors |
|---|---|--:|--:|---|---|---|
| single | 9z vs Lynn Vision (bo1) | 66.8% | 63.3% | 9z | Lynn Vision | stand_in (https://liquipedia.net/counterstrike/index.php?title=PGL/2026/Singapore&oldid=3473596) |
| debate | 9z vs Lynn Vision (bo1) | 66.8% | 64.8% | 9z | Lynn Vision | stand_in (https://liquipedia.net/counterstrike/index.php?title=PGL/2026/Singapore&oldid=3473596) |
| single | 9z vs M80 (bo1) | 49.3% | 46.8% | M80 | M80 | stand_in (https://liquipedia.net/counterstrike/index.php?title=PGL/2026/Singapore&oldid=3473790) |
| debate | 9z vs M80 (bo1) | 49.3% | 47.8% | M80 | M80 | stand_in (https://liquipedia.net/counterstrike/index.php?title=PGL/2026/Singapore&oldid=3473790) |
| single | 9z vs DENDELE CS (bo3) | 50.7% | 47.7% | DENDELE CS | 9z | stand_in (https://liquipedia.net/counterstrike/index.php?title=PGL/2026/Singapore&oldid=3474443) |
| debate | 9z vs DENDELE CS (bo3) | 50.7% | 49.2% | DENDELE CS | 9z | stand_in (https://liquipedia.net/counterstrike/index.php?title=PGL/2026/Singapore&oldid=3474443) |

## Pick'em Points (PickemSolver)

Every profile submits the same Pick'em: the sidecar's Pick'em step is the Monte-Carlo solver over pre-Stage market and VRS ratings, with no LLM and no decision policy (ticket #28). Swiss: 10 picks, Playoffs: 7 slots.

| Stage | Kind | Status | Points | Decided picks | Rated pairs |
|---|---|---|--:|--:|--:|
| PGL Major Copenhagen 2024 / Opening Stage | Swiss | final | 2 | 10/10 | 0 |
| PGL Major Copenhagen 2024 / Elimination Stage | Swiss | final | 3 | 10/10 | 0 |
| PGL Major Copenhagen 2024 / Playoffs | Playoffs | final | 4 | 7/7 | 0 |
| Perfect World Major Shanghai Major 2024 / Opening Stage | Swiss | final | 3 | 10/10 | 120 |
| Perfect World Major Shanghai Major 2024 / Elimination Stage | Swiss | final | 5 | 10/10 | 120 |
| Perfect World Major Shanghai Major 2024 / Playoffs | Playoffs | final | 3 | 7/7 | 28 |
| BLAST.tv Major Austin 2025 / Stage 1 | Swiss | final | 6 | 10/10 | 120 |
| BLAST.tv Major Austin 2025 / Stage 2 | Swiss | final | 2 | 10/10 | 120 |
| BLAST.tv Major Austin 2025 / Stage 3 | Swiss | final | 4 | 10/10 | 120 |
| BLAST.tv Major Austin 2025 / Playoffs | Playoffs | final | 6 | 7/7 | 28 |
| StarLadder Major Budapest 2025 / Stage 1 | Swiss | final | 6 | 10/10 | 120 |
| StarLadder Major Budapest 2025 / Stage 2 | Swiss | final | 5 | 10/10 | 120 |
| StarLadder Major Budapest 2025 / Stage 3 | Swiss | final | 4 | 10/10 | 120 |
| StarLadder Major Budapest 2025 / Playoffs | Playoffs | final | 1 | 7/7 | 28 |
| IEM Cologne Major 2026 / Stage 1 | Swiss | final | 4 | 10/10 | 120 |
| IEM Cologne Major 2026 / Stage 2 | Swiss | final | 5 | 10/10 | 120 |
| IEM Cologne Major 2026 / Stage 3 | Swiss | final | 4 | 10/10 | 120 |
| IEM Cologne Major 2026 / Playoffs | Playoffs | final | 4 | 7/7 | 28 |
| PGL Major Singapore 2026 / Stage 1 | Swiss | pending: Stage not finished (1 Points on 1 decided picks so far) | – | 1/10 | 120 |

## PGL Major Singapore 2026: every Match

Each profile's pick and the probability it gave its pick; ✓ correct, ✗ wrong, – no Prediction.

| Stage | Match | Format | Scheduled Start (UTC) | Winner | vrs | market | anchor | single | debate |
|---|---|---|---|---|---|---|---|---|---|
| Stage 1 | 9z vs Lynn Vision | BO1 | 2026-11-24 04:00 | Lynn Vision | ✗ 9z 78% | ✗ 9z 64% | ✗ 9z 67% | ✗ 9z 63% | ✗ 9z 65% |
| Stage 1 | Heroic vs Imperial | BO1 | 2026-11-24 04:00 | Imperial | ✗ Heroic 69% | ✗ Heroic 67% | ✗ Heroic 67% | ✗ Heroic 67% | ✗ Heroic 67% |
| Stage 1 | NIP vs M80 | BO1 | 2026-11-24 05:00 | NIP | ✓ NIP 60% | ✓ NIP 56% | ✓ NIP 57% | ✓ NIP 57% | ✓ NIP 57% |
| Stage 1 | 100 Thieves vs Chinggis Warriors | BO1 | 2026-11-24 05:25 | 100 Thieves | ✓ 100 Thieves 79% | ✓ 100 Thieves 80% | ✓ 100 Thieves 80% | ✓ 100 Thieves 80% | ✓ 100 Thieves 80% |
| Stage 1 | Liquid vs GOSH | BO1 | 2026-11-24 06:10 | Liquid | ✓ Liquid 74% | ✓ Liquid 56% | ✓ Liquid 60% | ✓ Liquid 60% | ✓ Liquid 60% |
| Stage 1 | PVISION vs DENDELE CS | BO1 | 2026-11-24 06:25 | PVISION | ✓ PVISION 63% | ✓ PVISION 61% | ✓ PVISION 61% | ✓ PVISION 61% | ✓ PVISION 61% |
| Stage 1 | FaZe vs Wildcard | BO1 | 2026-11-24 07:00 | Wildcard | ✗ FaZe 70% | ✗ FaZe 68% | ✗ FaZe 68% | ✗ FaZe 68% | ✗ FaZe 68% |
| Stage 1 | Eternal Fire vs FlyQuest | BO1 | 2026-11-24 07:15 | FlyQuest | ✗ Eternal Fire 75% | ✗ Eternal Fire 58% | ✗ Eternal Fire 62% | ✗ Eternal Fire 62% | ✗ Eternal Fire 62% |
| Stage 1 | NIP vs Lynn Vision | BO1 | 2026-11-24 08:45 | NIP | ✓ NIP 84% | ✓ NIP 71% | ✓ NIP 74% | ✓ NIP 74% | ✓ NIP 74% |
| Stage 1 | Heroic vs Chinggis Warriors | BO1 | 2026-11-24 08:45 | Heroic | ✓ Heroic 82% | ✓ Heroic 79% | ✓ Heroic 80% | ✓ Heroic 80% | ✓ Heroic 80% |
| Stage 1 | 9z vs M80 | BO1 | 2026-11-24 09:35 | M80 | ✗ 9z 50% | ✓ M80 51% | ✓ M80 51% | ✓ M80 53% | ✓ M80 52% |
| Stage 1 | 100 Thieves vs Imperial | BO1 | 2026-11-24 10:00 | Imperial | ✗ 100 Thieves 64% | ✗ 100 Thieves 63% | ✗ 100 Thieves 63% | ✗ 100 Thieves 63% | ✗ 100 Thieves 63% |
| Stage 1 | Eternal Fire vs DENDELE CS | BO1 | 2026-11-24 10:50 | Eternal Fire | ✓ Eternal Fire 57% | ✓ Eternal Fire 53% | ✓ Eternal Fire 54% | ✓ Eternal Fire 54% | ✓ Eternal Fire 54% |
| Stage 1 | PVISION vs FlyQuest | BO1 | 2026-11-24 11:00 | FlyQuest | ✗ PVISION 79% | ✗ PVISION 67% | ✗ PVISION 70% | ✗ PVISION 70% | ✗ PVISION 70% |
| Stage 1 | Liquid vs Wildcard | BO1 | 2026-11-24 12:00 | Liquid | ✓ Liquid 68% | ✓ Liquid 69% | ✓ Liquid 69% | ✓ Liquid 69% | ✓ Liquid 69% |
| Stage 1 | FaZe vs GOSH | BO1 | 2026-11-24 12:10 | FaZe | ✓ FaZe 76% | ✓ FaZe 56% | ✓ FaZe 61% | ✓ FaZe 61% | ✓ FaZe 61% |
| Stage 1 | Eternal Fire vs 100 Thieves | BO1 | 2026-11-25 06:00 | Eternal Fire | ✓ Eternal Fire 50% | ✗ 100 Thieves 59% | ✗ 100 Thieves 57% | ✗ 100 Thieves 57% | ✗ 100 Thieves 57% |
| Stage 1 | Heroic vs M80 | BO1 | 2026-11-25 06:00 | M80 | ✗ Heroic 58% | ✗ Heroic 61% | ✗ Heroic 60% | ✗ Heroic 60% | ✗ Heroic 60% |
| Stage 1 | Wildcard vs Lynn Vision | BO1 | 2026-11-25 07:00 | Lynn Vision | ✗ Wildcard 67% | ✗ Wildcard 50% | ✗ Wildcard 53% | ✗ Wildcard 53% | ✗ Wildcard 53% |
| Stage 1 | PVISION vs FaZe | BO1 | 2026-11-25 07:15 | PVISION | ✓ PVISION 53% | ✓ PVISION 55% | ✓ PVISION 55% | ✓ PVISION 55% | ✓ PVISION 55% |
| Stage 1 | 9z vs DENDELE CS | BO3 | 2026-11-25 08:10 | 9z | ✓ 9z 55% | ✗ DENDELE CS 50% | ✓ 9z 51% | ✗ DENDELE CS 52% | ✗ DENDELE CS 51% |
| Stage 1 | Imperial vs FlyQuest | BO3 | 2026-11-25 08:30 | FlyQuest | ✗ Imperial 68% | ✗ Imperial 53% | ✗ Imperial 56% | ✗ Imperial 56% | ✗ Imperial 56% |
| Stage 1 | GOSH vs Chinggis Warriors | BO3 | 2026-11-25 10:10 | Chinggis Warriors | ✗ GOSH 63% | ✗ GOSH 82% | ✗ GOSH 79% | ✗ GOSH 79% | ✗ GOSH 79% |
| Stage 1 | NIP vs Liquid | BO3 | 2026-11-25 11:00 | NIP | ✓ NIP 58% | ✗ Liquid 52% | ✗ Liquid 50% | ✗ Liquid 50% | ✗ Liquid 50% |

## PGL Major Singapore 2026 / Stage 1: the solver's Pick'em

pending: the Stage is not finished (1 of the 10 picked teams have finished it); 1 Points so far.

| Pick | Team | Record | State |
|---|---|---|---|
| three_nil | NIP | 3-0 | correct |
| three_nil | Heroic | 1-2 | wrong |
| zero_three | Chinggis Warriors | 1-2 | wrong |
| zero_three | Lynn Vision | 2-1 | wrong |
| advance | PVISION | 2-1 | undecided |
| advance | Liquid | 2-1 | undecided |
| advance | FaZe | 1-2 | undecided |
| advance | 100 Thieves | 1-2 | undecided |
| advance | Eternal Fire | 2-1 | undecided |
| advance | M80 | 2-1 | undecided |

## LLM track

Providers configured in this run: none.
- `debate`: 30 of 30 planned Matches answered; answers used: advocate by claude x30, arbiter by claude x30, devil by antigravity x30; provider calls in this run: 0.
- `single`: 30 of 30 planned Matches answered; answers used: predict by claude x30; provider calls in this run: 0.

## Notes

- Each Match is evaluated 1 s before its cutoff min(Scheduled Start − 15 min, actual start); every odds line, VRS snapshot and Liquipedia revision in its dossier was observed strictly before that instant. Each Stage's Pick'em is made 1 s before its first Match starts, from the Liquipedia Stage revision saved from before that instant (teams, Round 1 pairings, seeds).
- Scheduled Start and actual start come from PandaScore's final records (`scheduled_at`, `begin_at`).
- Market lines: OddsPapi historical Pinnacle series-winner prices, which exist for settled fixtures since January 2026 only (IEM Cologne Major 2026 and PGL Major Singapore 2026), then Polymarket and Kalshi price histories, saved for PGL Major Singapore 2026 Stage 1 only.
- Valve's global standings start in August 2024, so PGL Major Copenhagen 2024 has no rating: its Matches are not predicted and its Pick'em is drawn from even odds.
- Rosters: Liquipedia Event page revisions saved for some Matches only (see the Rosters column). No dated news archive exists, so no dossier holds news.
