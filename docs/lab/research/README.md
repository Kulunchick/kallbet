# Signal Lab on the research archive: reading (ticket #48)

The full report is [report.md](report.md), with the numbers per Event. Everything
machine-readable is in [report.json](report.json). [catch-up.md](catch-up.md) (simulated Leagues),
[pickem.md](pickem.md) and [llm.md](llm.md) sit next to it. This page says what the report means.
It is the second run: the archive is now complete except for the Holdout. What changed since the
first run, and why, is at the end.

## How it was made, and how to make it again

```
cargo run --release -p kallbet-ai -- lab --archive <ARCHIVE> --out docs/lab/research --seed 33
```

Then delete `docs/lab/research/audit.sqlite`, the run record, which is not committed. The run
needs no network and calls no provider: it reads only the archive directory.

- **Archive.** A frozen copy of `target/research-archive`, taken 2026-11-26 20:45 UTC after the
  collectors had finished. It is kept at `D:/Projects/KallBet/target/research-archive-t48-snapshot2`.
  The live directory may still grow, which moves its digest. The SHA-256 over the replayed files is
  `cd61333be4709fc8cab6b1951f7b01cf253c15f616e80fa58423ac75239455a7` (13,433 files). The archive is
  not in the repository.
- **Protocol.** `signal-lab/2`, seed 33, 2000 Event resamples, 90% intervals, a floor of 100 paired
  Matches. There are 26 Candidate Signals: the fixed list plus `bo3_veto_sim`, which is marked
  "added after list fixed".
- **Reproducibility.** Two runs of that command on that archive gave byte-identical `report.md`,
  `report.json`, `catch-up.md`, `pickem.md` and `pickem.json`.
- **Time.** 11 min 26 s to 11 min 53 s per run (release build, Windows workstation), up from about 8 min 50 s
  in the first run because the archive is larger. Nothing was trimmed.

## What the archive held

62 Events (tier S and A, July 2024 to 15 November 2026). 2288 scored Matches: 2243 with the anchor
(a Valve rating or a market line), 777 with a market line, 1893 of them Bo3.

| Input | At this run | Complete? |
|---|---|---|
| Valve rating | 2243 rated Matches | yes |
| Market history | 469 Pinnacle fixtures, 3191 Polymarket and 6627 Kalshi files; 777 Matches with a pre-cutoff line, 28 priced by all three sources | yes |
| Liquipedia daily rosters | 577 revisions; 2095 Matches computed for `roster_stand_in` | yes (finished 2026-11-26) |
| Liquipedia post-Match maps | 2117 Matches for `per_map_rating` | yes |
| HLTV match pages | 1975 Matches, 1970 with a veto. 8 walkover pages had no stats and are skipped as the collector recorded them. | yes |
| Liquipedia opening revisions (Pick'em) | none | not collected for the research archive |
| PGL Major Singapore 2026 | none, Stage 1 included | needed for the Holdout |

## The Candidate Signals

Δ is the variant minus the anchor in mean log-loss on the Matches both predicted; below zero is
better. "Detectable" is the smallest effect these Matches would show four times in five.

| Candidate Signal | Verdict | Paired | Δ log-loss [90%] | Detectable | Market subset (n, Δ) |
|---|---|---:|---|---:|---|
| `rating_calibration` | promote | 2193 | −0.176 [−0.240, −0.122] | 0.089 | 776, −0.016 |
| `learned_weights` | promote | 2243 | −0.170 [−0.230, −0.115] | 0.087 | 777, −0.012 |
| `own_rating` | promote | 2168 | −0.168 [−0.233, −0.108] | 0.093 | 758, −0.013 |
| `per_map_rating` | promote | 2109 | −0.165 [−0.234, −0.104] | 0.097 | 724, −0.011 |
| `bo3_veto_sim` (added after list fixed) | promote | 1850 | −0.170 [−0.249, −0.101] | 0.112 | 702, −0.018 |
| `bo1_veto_sim` | promote | 319 | −0.088 [−0.132, −0.034] | 0.074 | 54, −0.003 |
| `lan_online` | promote | 2078 | −0.005 [−0.008, −0.002] | 0.004 | 756, −0.002 |
| `lineup_strength` | promote | 1950 | −0.003 [−0.005, −0.001] | 0.003 | 641, −0.002 |
| `stakes` | no evidence (market subset worse) | 630 | −0.038 [−0.065, −0.012] | 0.040 | 216, +0.018 |
| `head_to_head` | no evidence | 1280 | −0.004 [−0.013, +0.005] | 0.013 | 561, +0.007 |
| `recent_form` | no evidence | 1980 | −0.001 [−0.005, +0.004] | 0.007 | 697, +0.006 |
| `rest_schedule_load` | no evidence | 2128 | −0.000 [−0.004, +0.004] | 0.006 | 758, −0.002 |
| `roster_stand_in` | no evidence | 2056 | −0.000 [−0.002, +0.002] | 0.003 | 651, −0.001 |
| `player_form` | no evidence | 1567 | +0.000 [−0.000, +0.000] | 0.000 | 483, +0.000 |
| `series_format` | no evidence | 1710 | +0.000 [−0.001, +0.002] | 0.002 | 644, +0.002 |
| `line_movement` | no evidence | 744 | +0.001 [−0.004, +0.007] | 0.008 | 744, +0.001 |
| `favourite_longshot` | no evidence | 746 | +0.001 [−0.000, +0.002] | 0.002 | 746, +0.001 |
| `region_travel` | no evidence (107 Matches, 2 Events) | 107 | +0.001 [+0.000, +0.001] | 0.001 | 105, +0.002 |
| `roster_stability` | no evidence | 1534 | +0.005 [−0.001, +0.010] | 0.008 | 621, +0.002 |
| `market_source_pinnacle` / `_polymarket` / `_kalshi` / `_average` | no evidence (28 Matches, under the floor) | 28 | 0, +0.023, +0.016, +0.012 | — | — |
| `roster_coach_change` | harmful | 1903 | +0.004 [+0.000, +0.009] | 0.007 | 615, −0.000 |
| `roster_change_30d` | harmful | 1563 | +0.004 [+0.002, +0.008] | 0.005 | 504, +0.002 |
| `llm_adjustment` | not testable (pending) | 0 | — | — | — |

## Frozen combination and Holdout

Forward selection considered the eight `promote`s. The training side's walk-forward log-loss
started from the anchor's 0.8044:

| Step | Entered | Log-loss after | Added [90%] |
|---|---|---:|---|
| 1 | `own_rating` | 0.6327 | −0.1716 [−0.2316, −0.1154] |
| 2 | `lineup_strength` | 0.6323 | −0.0004 [−0.0007, −0.0003] |
| 3 | `lan_online` | 0.6322 | −0.0001 [−0.0002, +0.0001] |

The frozen combination is **`own_rating` + `lineup_strength` + `lan_online`**, in the combiner with
the market line and the Valve rating. After `own_rating`, the other rating fixes added nothing.
At step 1 they scored 0.6329 to 0.6342, because they repair the same overconfidence. Steps 2 and 3
are tiny, and step 3's interval includes zero. Selection has no significance gate: it stops only
when nothing lowers the log-loss.

**Live status** (`sidecar/src/promotion.rs`). A promoted criterion is selectable per bot, but only
the members in `promotion::LIVE` change a live Prediction: `rating_calibration` and
`learned_weights`. Their fits are in `report.json` under `deployment`. None of the frozen
combination's three members has a live input port: `own_rating` needs a results history,
`lineup_strength` needs HLTV player ratings, and `lan_online` needs dated Event types and each
team's results by environment. A bot
promoted with `combination` therefore loads, but decides every Match by the anchor and gives
"<member> needs …, which the live sidecar has no port for" as the reason. The same holds for
`per_map_rating`, `bo1_veto_sim` and `bo3_veto_sim`. A bot gets the measured gain live today only
through `rating_calibration` or `learned_weights`, each worth about −0.17 on its own.

**Holdout: pending, not evaluated.** No Holdout Match is in the archive. The Holdout is PGL Major
Singapore 2026 after Stage 1, plus anything completed after 2026-11-25 23:46:46 UTC, and the
archive holds no Singapore Stage at all. Once Singapore's later Stages are played and collected,
one more run of the command above evaluates the frozen combination once. The report's Holdout
section then states the evaluation date. The selection is recomputed on that run's training side,
which does not change when only Holdout Stages are added.

## Plain reading

- **Worth promoting:** a fix for the overconfident Valve rating. The anchor's log-loss is 0.804,
  worse than a coin flip (0.693). Each of `rating_calibration`, `learned_weights`, `own_rating`,
  `per_map_rating` and `bo3_veto_sim` takes off about 0.17. Each also helps the Matches with a
  market line by about −0.01 to −0.02. These five are one effect, not five. The live choice is
  `rating_calibration` or `learned_weights`, the two that can run live. `bo1_veto_sim` helps on its
  319 Bo1s for the same reason.
- **Promoted, but too small to matter:** `lineup_strength` (−0.003) and `lan_online` (−0.005).
  Their intervals exclude zero, so the rule promotes them. Inside the combination, though, they add
  0.0004 and 0.0001. Read them as rule results, not as new criteria. The Holdout number is the one
  to quote.
- **Not worth promoting:** recent form, head-to-head, rest and schedule, series format, stand-ins,
  roster stability, player form, line movement, favourite-longshot correction and region/travel.
  All sit within about ±0.01 of zero.
  - `roster_change_30d` and `roster_coach_change` are slightly harmful (+0.004 each).
  - `stakes` improves the whole set but worsens the Matches with a market line (+0.018), so the
    rule does not promote it.
  - `bo3_veto_sim` was added after the list was fixed and is reported with that mark. It entered no
    step of the selection.
- **Could not be judged for want of data:**
  - Market source comparison: only 28 Matches were priced by Pinnacle, Polymarket and Kalshi
    together.
  - Region and travel: 107 Matches in 2 Events.
  - The LLM adjustment: not run yet.
  - Pick'em on the research archive: no opening revisions.
- **What the archive could not have seen:** effects smaller than the "Detectable" column. That is
  under 0.001 for player form and region, about 0.002 for series format and the favourite-longshot
  correction, 0.003 to 0.008 for stand-ins, roster stability, rest, form and line movement, 0.013
  for head-to-head, and 0.04 for stakes. With 26 candidates on one archive, a small `promote` can be
  chance. The Holdout is the guard.

## Not about one Match

- **Catch-up mode** ([catch-up.md](catch-up.md), 62 training-side Events). Against 12
  favourite-followers, the chance of finishing first rises with the catch-up policy:
  - at 10% noise, from 0.192 to 0.394;
  - at 25% noise, from 0.395 to 0.450.

  The expected place gets worse, because the policy trades average place for the chance of first.
  Against market-only Members, first rises from 0.963 to 0.984.
- **Pick'em variants:** not testable on the research archive. None of its 171 Stages has a saved
  Liquipedia opening revision, the opening pairings. On the saved archive (6 Majors,
  [../pickem.md](../pickem.md)), all three variants are `no evidence` on 18 Stages.

## LLM track: pending

The recorded answers do not exist yet. Producing them is the operator's run, with a budget of runs
per subscription:

```
cargo run --release -p kallbet-ai -- lab --archive <ARCHIVE> --out docs/lab/research --seed 33 --llm <LLM_FILE> --answers docs/lab/research/llm-answers.json --budget claude=<RUNS> --budget codex=<RUNS> --budget antigravity=<RUNS>
```

`<ARCHIVE>` is the frozen research snapshot above. The lab refuses `--llm` without an explicit
`--archive`, because its default is the small test archive. `<LLM_FILE>` starts from
[../llm.example.json](../llm.example.json). After that run, the same command with
`--answers docs/lab/research/llm-answers.json` and no `--llm` or `--budget` reproduces it with no
provider. This run called no provider.

## Request counters of the limited Data Sources

The lab makes no requests. These are the collectors' own ledgers. The first run's column was read
at 14:52 UTC; the second run's ledgers were read in the snapshot (20:45 UTC) and again in the live
archive after the runs, and they were identical.

| Data Source | Ledger | First run | Second run (before = after) |
|---|---|---:|---:|
| OddsPapi (billable, monthly) | `catalog/market-ledger.json`, 2026-11 | 62 | 62 |
| PandaScore | `collector.db` `spent`, 2026-11-26 | 201 | 201 |
| Valve standings (GitHub) | same, 2026-11-26 | 48 | 50 |
| Liquipedia | same, 2026-11-26 | 300 | 664 |
| HLTV `/matches/*` | `hltv/manifest.json`, 2026-11-26 | 400 | 3966 (no stop recorded) |
| LLM subscriptions | — | 0 runs | 0 runs |

The rises between the two runs are the collectors finishing, not the lab. The Polymarket and
Kalshi reads through the remote host are public and not ledgered: `sources-market.tsv` holds 10,483
rows.

## What changed since the first run, and why

The first run used archive `beabb356…` (8285 files) from 2026-11-26 14:52 UTC.

- **Complete data.** This moved the most.
  - Rosters now cover every Event: `roster_stand_in` computed on 2095 Matches instead of 828, and
    the other roster signals grew alike.
  - HLTV went from 200 pages to 1975. `lineup_strength` (from not testable to promote), `player_form`
    (from not testable to no evidence) and `bo1_veto_sim` (from not testable to promote, 319 Bo1s)
    could all be measured.
  - `lan_online` grew from 752 Matches to 2078.
  - `region_travel` got its first 107 Matches.
  - The market grew from 766 Matches with a line to 777.
  - `roster_coach_change` moved from no evidence to harmful on 2.6 times the Matches.
  - The frozen combination grew from `own_rating` alone to `own_rating` + `lineup_strength` +
    `lan_online`.
- **The combiner fix from the first run** (line search in `combiner.rs`) is in both runs. It is
  what made `learned_weights` and forward selection meaningful at all.
- **Review fixes** in `spec-33-signal-lab`.
  - Finding 1: a Match without a reported finish is no longer visible to the results or player
    histories, and HLTV finishes are no longer estimated.
  - Finding 9: only complete Events train another Event, and an incomplete Event is Holdout.
  - Finding 3: no Stage of the evaluated Event trains it.
  - Finding 2: Holdout results reach no training view.
  - Finding 21: a Match's context is sliced at its cutoff.

  On the training side these shift the rating signals by under 0.001. For example,
  `rating_calibration` moved from −0.1766 to −0.1759 and `own_rating` from −0.1687 to −0.1677. The
  anchor's log-loss moved from 0.8055 to 0.8044, mostly because of the added market lines.
- **Map-name canonicalisation (#52).** `per_map_rating` now joins HLTV "Dust2" and Liquipedia
  "Dust II". It moved from −0.1662 to −0.1654 on the same 2109 Matches.
- **`bo3_veto_sim` (#52)** is new and marked "added after list fixed". It promotes (−0.170 on 1850
  Bo3s) but entered no step of the selection.
- The Holdout was pending in both runs, for the same reason: Singapore is absent.
