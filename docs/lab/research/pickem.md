# Pick'em variants in Pick'em Points

Protocol `signal-lab/2`, archive SHA-256 `cd61333be4709fc8cab6b1951f7b01cf253c15f616e80fa58423ac75239455a7`, seed 33, 2000 Event resamples, 90% intervals. Each Stage's Pick'em is made 1 s before its first Match from the Liquipedia opening view saved from before it and the ratings the replay ports gave before that instant; the solver refuses a view that holds a result or a later-round pairing. Pick'em Points are KallBet's (Swiss 1 per correct 3-0, 0-3 and advance pick; Playoffs 1 per correct slot). The difference is the mean Points per Stage minus the baseline's, with the Event bootstrap; under 10 Stages the verdict is `no evidence`. Stages of the Holdout are not read.

The field is 6 simulated Members who submit the baseline's solver on ratings disturbed by a normal noise of 0.5 on the logit; the bot trails when its public Pick'em Points (Stages that ended before) are below the best Member's; the variance-seeking variant then discounts the field's shared picks by 50%. 5000 simulations per Swiss Pick'em.

## Variants against the baseline

| Variant | Total Points | Stages | Difference per Stage | 90% interval | Detectable | Verdict | Mean place | Events first |
|---|---:|---:|---:|---|---:|---|---:|---:|
| `baseline` | — | 0 | — | — | — | **not testable**: no Stage was scored | — | 0 |
| `calibrated` | — | 0 | — | — | — | **not testable**: no Stage was scored | — | 0 |
| `learned` | — | 0 | — | — | — | **not testable**: no Stage was scored | — | 0 |
| `variance_seeking` | — | 0 | — | — | — | **not testable**: no Stage was scored | — | 0 |

`calibrated` rated 0 pairs itself and left 0 to the baseline's rating.

`learned` rated 0 pairs itself and left 0 to the baseline's rating.

## Pick'em Points per Stage

| Event | Stage | Picks | baseline | calibrated | learned | variance_seeking | trailing | best Member |
|---|---|---:|---:|---:|---:|---:|---|---:|

## Stages not scored

- BLAST Premier Groups Fall 2024 / Group A: no opening view: no Liquipedia Stage revision saved
- BLAST Premier Groups Fall 2024 / Group B: no opening view: no Liquipedia Stage revision saved
- BLAST Premier Groups Fall 2024 / Group C: no opening view: no Liquipedia Stage revision saved
- BLAST Premier Groups Fall 2024 / Group D: no opening view: no Liquipedia Stage revision saved
- BLAST Premier Groups Fall 2024 / Play-In: no opening view: no Liquipedia Stage revision saved
- IEM Cologne 2024 / Play-In: no opening view: no Liquipedia Stage revision saved
- IEM Cologne 2024 / Group B: no opening view: no Liquipedia Stage revision saved
- IEM Cologne 2024 / Group A: no opening view: no Liquipedia Stage revision saved
- IEM Cologne 2024 / Playoffs: no opening view: no Liquipedia Stage revision saved
- BLAST Premier Showdown Fall 2024 / Playoffs: no opening view: no Liquipedia Stage revision saved
- BetBoom Dacha Belgrade season 2 2024 / Group A: no opening view: no Liquipedia Stage revision saved
- BetBoom Dacha Belgrade season 2 2024 / Group B: no opening view: no Liquipedia Stage revision saved
- BetBoom Dacha Belgrade season 2 2024 / Playoffs: no opening view: no Liquipedia Stage revision saved
- ESL Pro League Season 20 2024 / Group A: no opening view: no Liquipedia Stage revision saved
- ESL Pro League Season 20 2024 / Group B: no opening view: no Liquipedia Stage revision saved
- ESL Pro League Season 20 2024 / Group C: no opening view: no Liquipedia Stage revision saved
- ESL Pro League Season 20 2024 / Group D: no opening view: no Liquipedia Stage revision saved
- ESL Pro League Season 20 2024 / Playoffs: no opening view: no Liquipedia Stage revision saved
- BLAST Premier Final Fall 2024 / Group A: no opening view: no Liquipedia Stage revision saved
- BLAST Premier Final Fall 2024 / Group B: no opening view: no Liquipedia Stage revision saved
- BLAST Premier Final Fall 2024 / Playoffs: no opening view: no Liquipedia Stage revision saved
- IEM Rio 2024 / Group A: no opening view: no Liquipedia Stage revision saved
- IEM Rio 2024 / Group B: no opening view: no Liquipedia Stage revision saved
- IEM Rio 2024 / Playoffs: no opening view: no Liquipedia Stage revision saved
- Thunderpick World Championship 2024 / Group A: no opening view: no Liquipedia Stage revision saved
- Thunderpick World Championship 2024 / Group B: no opening view: no Liquipedia Stage revision saved
- Thunderpick World Championship 2024 / Group C: no opening view: no Liquipedia Stage revision saved
- Thunderpick World Championship 2024 / Group D: no opening view: no Liquipedia Stage revision saved
- Thunderpick World Championship 2024 / Playoffs: no opening view: no Liquipedia Stage revision saved
- Perfect World Major Shanghai: European RMR A 2024 / Group Stage: no opening view: no Liquipedia Stage revision saved
- Perfect World Major Shanghai: European RMR B 2024 / Group Stage: no opening view: no Liquipedia Stage revision saved
- Perfect World Major Shanghai: American RMR 2024 / Group Stage: no opening view: no Liquipedia Stage revision saved
- BLAST Premier World Final 2024 / Group A: no opening view: no Liquipedia Stage revision saved
- BLAST Premier World Final 2024 / Group B: no opening view: no Liquipedia Stage revision saved
- BLAST Premier World Final 2024 / Playoffs: no opening view: no Liquipedia Stage revision saved
- Perfect World Major Shanghai Major 2024 / Opening Stage: no opening view: no Liquipedia Stage revision saved
- Perfect World Major Shanghai Major 2024 / Elimination Stage: no opening view: no Liquipedia Stage revision saved
- Perfect World Major Shanghai Major 2024 / Playoffs: no opening view: no Liquipedia Stage revision saved
- BLAST Premier Bounty Closed Qualifier 2025 / Playoffs: no opening view: no Liquipedia Stage revision saved
- IEM Katowice 2025 / Play-in: no opening view: no Liquipedia Stage revision saved
- IEM Katowice 2025 / Group A: no opening view: no Liquipedia Stage revision saved
- IEM Katowice 2025 / Group B: no opening view: no Liquipedia Stage revision saved
- IEM Katowice 2025 / Playoffs: no opening view: no Liquipedia Stage revision saved
- BLAST Premier Bounty season 1 2025 / Playoffs: no opening view: no Liquipedia Stage revision saved
- PGL Cluj-Napoca 2025 / Group Stage: no opening view: no Liquipedia Stage revision saved
- PGL Cluj-Napoca 2025 / Playoffs: no opening view: no Liquipedia Stage revision saved
- ESL Pro League Season 21 2025 / Group Stage: no opening view: no Liquipedia Stage revision saved
- ESL Pro League Season 21 2025 / Playoffs: no opening view: no Liquipedia Stage revision saved
- BLAST Open Lisbon Spring 2025 / Group A: no opening view: no Liquipedia Stage revision saved
- BLAST Open Lisbon Spring 2025 / Group B: no opening view: no Liquipedia Stage revision saved
- BLAST Open Lisbon Spring 2025 / Playoffs: no opening view: no Liquipedia Stage revision saved
- PGL Bucharest 2025 / Group Stage: no opening view: no Liquipedia Stage revision saved
- PGL Bucharest 2025 / Playoffs: no opening view: no Liquipedia Stage revision saved
- IEM Melbourne 2025 / Group A: no opening view: no Liquipedia Stage revision saved
- IEM Melbourne 2025 / Group B: no opening view: no Liquipedia Stage revision saved
- IEM Melbourne 2025 / Playoffs: no opening view: no Liquipedia Stage revision saved
- BLAST.tv Major Austin: European Regional Qualifier 2025 / Group Stage: no opening view: no Liquipedia Stage revision saved
- BLAST.tv Major Austin: European Regional Qualifier 2025 / Play-Ins: no opening view: no Liquipedia Stage revision saved
- BLAST Rivals Spring 2025 / Group B: no opening view: no Liquipedia Stage revision saved
- BLAST Rivals Spring 2025 / Group A: no opening view: no Liquipedia Stage revision saved
- BLAST Rivals Spring 2025 / Playoffs: no opening view: no Liquipedia Stage revision saved
- PGL Astana 2025 / Group Stage: no opening view: no Liquipedia Stage revision saved
- PGL Astana 2025 / Playoffs: no opening view: no Liquipedia Stage revision saved
- IEM Dallas 2025 / Group A: no opening view: no Liquipedia Stage revision saved
- IEM Dallas 2025 / Group B: no opening view: no Liquipedia Stage revision saved
- IEM Dallas 2025 / Playoffs: no opening view: no Liquipedia Stage revision saved
- BLAST.tv Major Austin 2025 / Stage 1: no opening view: no Liquipedia Stage revision saved
- BLAST.tv Major Austin 2025 / Stage 2: no opening view: no Liquipedia Stage revision saved
- BLAST.tv Major Austin 2025 / Stage 3: no opening view: no Liquipedia Stage revision saved
- BLAST.tv Major Austin 2025 / Playoffs: no opening view: no Liquipedia Stage revision saved
- FISSURE PLAYGROUND Season 1 2025 / Group A: no opening view: no Liquipedia Stage revision saved
- FISSURE PLAYGROUND Season 1 2025 / Group B: no opening view: no Liquipedia Stage revision saved
- FISSURE PLAYGROUND Season 1 2025 / Group D: no opening view: no Liquipedia Stage revision saved
- FISSURE PLAYGROUND Season 1 2025 / Group C: no opening view: no Liquipedia Stage revision saved
- FISSURE PLAYGROUND Season 1 2025 / Playoffs: no opening view: no Liquipedia Stage revision saved
- IEM Cologne 2025 / Play-In: no opening view: no Liquipedia Stage revision saved
- IEM Cologne 2025 / Group A: no opening view: no Liquipedia Stage revision saved
- IEM Cologne 2025 / Group B: no opening view: no Liquipedia Stage revision saved
- IEM Cologne 2025 / Playoffs: no opening view: no Liquipedia Stage revision saved
- BLAST Bounty Fall 2025 / Qualifier: no opening view: no Liquipedia Stage revision saved
- BLAST Bounty Fall 2025 / Playoffs: no opening view: no Liquipedia Stage revision saved
- Esports World Cup 2025 / Playoffs: no opening view: no Liquipedia Stage revision saved
- BLAST Open London Fall 2025 / Group A: no opening view: no Liquipedia Stage revision saved
- BLAST Open London Fall 2025 / Group B: no opening view: no Liquipedia Stage revision saved
- BLAST Open London Fall 2025 / Playoffs: no opening view: no Liquipedia Stage revision saved
- FISSURE PLAYGROUND Season 2 2025 / Group Stage: no opening view: no Liquipedia Stage revision saved
- FISSURE PLAYGROUND Season 2 2025 / Playoffs: no opening view: no Liquipedia Stage revision saved
- ESL Pro League Season 22 2025 / Stage 1: no opening view: no Liquipedia Stage revision saved
- ESL Pro League Season 22 2025 / Stage 2: no opening view: no Liquipedia Stage revision saved
- ESL Pro League Season 22 2025 / Playoffs: no opening view: no Liquipedia Stage revision saved
- CS Asia Championships 2025 / Group A: no opening view: no Liquipedia Stage revision saved
- CS Asia Championships 2025 / Group B: no opening view: no Liquipedia Stage revision saved
- CS Asia Championships 2025 / Playoffs: no opening view: no Liquipedia Stage revision saved
- Thunderpick World Championship 2025 / Group A: no opening view: no Liquipedia Stage revision saved
- Thunderpick World Championship 2025 / Group B: no opening view: no Liquipedia Stage revision saved
- Thunderpick World Championship 2025 / Playoffs: no opening view: no Liquipedia Stage revision saved
- PGL Masters Bucharest 2025 / Group Stage: no opening view: no Liquipedia Stage revision saved
- PGL Masters Bucharest 2025 / Playoffs: no opening view: no Liquipedia Stage revision saved
- IEM Chengdu 2025 / Group A: no opening view: no Liquipedia Stage revision saved
- IEM Chengdu 2025 / Group B: no opening view: no Liquipedia Stage revision saved
- IEM Chengdu 2025 / Playoffs: no opening view: no Liquipedia Stage revision saved
- StarLadder Major Budapest 2025 / Stage 1: no opening view: no Liquipedia Stage revision saved
- StarLadder Major Budapest 2025 / Stage 2: no opening view: no Liquipedia Stage revision saved
- StarLadder Major Budapest 2025 / Stage 3: no opening view: no Liquipedia Stage revision saved
- StarLadder Major Budapest 2025 / Playoffs: no opening view: no Liquipedia Stage revision saved
- BLAST Rivals Fall 2025 / Group A: no opening view: no Liquipedia Stage revision saved
- BLAST Rivals Fall 2025 / Group B: no opening view: no Liquipedia Stage revision saved
- BLAST Rivals Fall 2025 / Playoffs: no opening view: no Liquipedia Stage revision saved
- BLAST Bounty Winter 2026 / Qualifier: no opening view: no Liquipedia Stage revision saved
- BLAST Bounty Winter 2026 / Playoffs: no opening view: no Liquipedia Stage revision saved
- IEM Krakow 2026 / Play-In: no opening view: no Liquipedia Stage revision saved
- IEM Krakow 2026 / Group A: no opening view: no Liquipedia Stage revision saved
- IEM Krakow 2026 / Group B: no opening view: no Liquipedia Stage revision saved
- IEM Krakow 2026 / Playoffs: no opening view: no Liquipedia Stage revision saved
- PGL Cluj-Napoca 2026 / Group Stage: no opening view: no Liquipedia Stage revision saved
- PGL Cluj-Napoca 2026 / Playoffs: no opening view: no Liquipedia Stage revision saved
- BLAST Open Rotterdam Spring 2026 / Group A: no opening view: no Liquipedia Stage revision saved
- BLAST Open Rotterdam Spring 2026 / Group B: no opening view: no Liquipedia Stage revision saved
- BLAST Open Rotterdam Spring 2026 / Playoffs: no opening view: no Liquipedia Stage revision saved
- ESL Pro League Season 23 2026 / Stage 1: no opening view: no Liquipedia Stage revision saved
- ESL Pro League Season 23 2026 / Stage 2: no opening view: no Liquipedia Stage revision saved
- ESL Pro League Season 23 2026 / Playoffs: no opening view: no Liquipedia Stage revision saved
- PGL Bucharest 2026 / Group Stage: no opening view: no Liquipedia Stage revision saved
- PGL Bucharest 2026 / Playoffs: no opening view: no Liquipedia Stage revision saved
- IEM Rio 2026 / Group A: no opening view: no Liquipedia Stage revision saved
- IEM Rio 2026 / Group B: no opening view: no Liquipedia Stage revision saved
- IEM Rio 2026 / Playoffs: no opening view: no Liquipedia Stage revision saved
- IEM Cologne Major 2026 / Stage 1: no opening view: no Liquipedia Stage revision saved
- IEM Cologne Major 2026 / Stage 2: no opening view: no Liquipedia Stage revision saved
- IEM Cologne Major 2026 / Stage 3: no opening view: no Liquipedia Stage revision saved
- IEM Cologne Major 2026 / Playoffs: no opening view: no Liquipedia Stage revision saved
- BLAST Rivals Spring 2026 / Group A: no opening view: no Liquipedia Stage revision saved
- BLAST Rivals Spring 2026 / Group B: no opening view: no Liquipedia Stage revision saved
- BLAST Rivals Spring 2026 / Playoffs: no opening view: no Liquipedia Stage revision saved
- PGL Astana 2026 / Group Stage: no opening view: no Liquipedia Stage revision saved
- PGL Astana 2026 / Playoffs: no opening view: no Liquipedia Stage revision saved
- IEM Atlanta 2026 / Group A: no opening view: no Liquipedia Stage revision saved
- IEM Atlanta 2026 / Group B: no opening view: no Liquipedia Stage revision saved
- IEM Atlanta 2026 / Playoffs: no opening view: no Liquipedia Stage revision saved
- CS Asia Championships 2026 / Group B: no opening view: no Liquipedia Stage revision saved
- CS Asia Championships 2026 / Group A: no opening view: no Liquipedia Stage revision saved
- CS Asia Championships 2026 / Playoffs: no opening view: no Liquipedia Stage revision saved
- XSE Pro League 2026 / Group Stage: no opening view: no Liquipedia Stage revision saved
- XSE Pro League 2026 / Playoffs: no opening view: no Liquipedia Stage revision saved
- BLAST Bounty Summer 2026 / Qualifier: no opening view: no Liquipedia Stage revision saved
- BLAST Bounty Summer 2026 / Playoffs: no opening view: no Liquipedia Stage revision saved
- Esports World Cup 2026 / Group A: no opening view: no Liquipedia Stage revision saved
- Esports World Cup 2026 / Group C: no opening view: no Liquipedia Stage revision saved
- Esports World Cup 2026 / Group D: no opening view: no Liquipedia Stage revision saved
- Esports World Cup 2026 / Group B: no opening view: no Liquipedia Stage revision saved
- Esports World Cup 2026 / Playoffs: no opening view: no Liquipedia Stage revision saved
- BLAST Open Porto Fall 2026 / Group A: no opening view: no Liquipedia Stage revision saved
- BLAST Open Porto Fall 2026 / Group B: no opening view: no Liquipedia Stage revision saved
- BLAST Open Porto Fall 2026 / Playoffs: no opening view: no Liquipedia Stage revision saved
- FISSURE PLAYGROUND Season 3 2026 / Group A: no opening view: no Liquipedia Stage revision saved
- FISSURE PLAYGROUND Season 3 2026 / Group B: no opening view: no Liquipedia Stage revision saved
- FISSURE PLAYGROUND Season 3 2026 / Playoffs: no opening view: no Liquipedia Stage revision saved
- StarLadder StarSeries Fall 2026 / Playoffs: no opening view: no Liquipedia Stage revision saved
- ESL Pro League Season 24 2026 / Group Stage: no opening view: no Liquipedia Stage revision saved
- ESL Pro League Season 24 2026 / Playoffs: no opening view: no Liquipedia Stage revision saved
- Thunderpick World Championship 2026 / Group A: no opening view: no Liquipedia Stage revision saved
- Thunderpick World Championship 2026 / Group B: no opening view: no Liquipedia Stage revision saved
- Thunderpick World Championship 2026 / Playoffs: no opening view: no Liquipedia Stage revision saved
- PGL Masters Bucharest 2026 / Group Stage: no opening view: no Liquipedia Stage revision saved
- PGL Masters Bucharest 2026 / Playoffs: no opening view: no Liquipedia Stage revision saved
- IEM Beijing 2026 / Group A: no opening view: no Liquipedia Stage revision saved
- IEM Beijing 2026 / Group B: no opening view: no Liquipedia Stage revision saved
- IEM Beijing 2026 / Playoffs: no opening view: no Liquipedia Stage revision saved
- BLAST Rivals Fall 2026 / Group A: no opening view: no Liquipedia Stage revision saved
- BLAST Rivals Fall 2026 / Group B: no opening view: no Liquipedia Stage revision saved
- BLAST Rivals Fall 2026 / Playoffs: no opening view: no Liquipedia Stage revision saved
