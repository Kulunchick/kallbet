# Signal Lab on the research archive: reading (ticket #48)

The full report is [report.md](report.md) (numbers per Event in it, everything machine-readable in
[report.json](report.json)), with [catch-up.md](catch-up.md) (simulated Leagues),
[pickem.md](pickem.md) and [llm.md](llm.md) next to it. This page says what the report means.

## How it was made, and how to make it again

```
cargo run --release -p kallbet-ai -- lab --archive <ARCHIVE> --out docs/lab/research --seed 33
```

then delete `docs/lab/research/audit.sqlite` (the run record, not committed). No network, no
provider: the run reads only the archive directory.

- Archive: a frozen copy of `target/research-archive` taken 2026-11-26 14:52 UTC, kept at
  `D:/Projects/KallBet/target/research-archive-t48-snapshot` (the live directory keeps growing
  while the collectors run, so its digest moves). SHA-256 over the replayed files
  `beabb356eda6798dc620e39797cf5ee1e47a9b7ebbae11565ef04eaa56de54e4`, 8285 files. The archive is
  not in the repository.
- Protocol `signal-lab/2`, seed 33, 2000 Event resamples, 90% intervals, floor 100 paired Matches.
- Two runs of that command on that archive gave byte-identical `report.md`, `report.json`,
  `catch-up.md`, `pickem.md` and `pickem.json`.
- Time: 8 min 44 s to 8 min 56 s per run (release build, Windows workstation), for all 25 Candidate
  Signals, forward selection, 2000 catch-up trials per setting and the Pick'em pass. Nothing was
  trimmed.

## What the archive held

62 Events (tier S and A, July 2024 to 15 November 2026), 2288 scored Matches, 2243 with the
anchor (a Valve rating or a market line), 766 with a market line. Coverage of the inputs that the
Candidate Signals need:

| Input | Matches or files at the run | Still being collected |
|---|---|---|
| Valve rating | 2243 rated Matches | complete |
| Market line (Pinnacle, Polymarket, Kalshi) | 766 Matches with a line; 27 priced by all three sources | yes: 1048 more Polymarket and Kalshi files had already arrived by the end of this run |
| Liquipedia rosters (daily revisions) | 24 Events, July 2024 to August 2025: 828 Matches for `roster_stand_in` | 2 more collection days |
| Liquipedia post-Match map results | 1826 Matches linked | complete for the current Events |
| HLTV match pages (vetoes, player ratings) | 200 Matches, 14 August to 15 November 2026, all Bo3 or Bo5 | ~1783 left, about 9 days at 200 a day, newest first |
| PGL Major Singapore 2026 | none (its Stage 1 was still running when the Events were collected) | needed for the Holdout |

## The Candidate Signals

Δ is the variant minus the anchor in mean log-loss on the Matches both predicted (below zero is
better). "Detectable" is the smallest effect these Matches would show four times in five.

| Candidate Signal | Verdict | Paired | Δ log-loss [90%] | Detectable | Will it change after collection? |
|---|---|---:|---|---:|---|
| `rating_calibration` | promote | 2193 | −0.177 [−0.241, −0.123] | 0.089 | little |
| `learned_weights` | promote | 2243 | −0.171 [−0.231, −0.115] | 0.087 | the market+rating weights, as market lines arrive |
| `own_rating` | promote | 2168 | −0.169 [−0.234, −0.109] | 0.093 | little |
| `per_map_rating` | promote | 2109 | −0.166 [−0.235, −0.104] | 0.097 | little |
| `lan_online` | promote | 752 | −0.007 [−0.010, −0.004] | 0.005 | coverage, as Liquipedia revisions arrive |
| `stakes` | no evidence (market subset worse) | 630 | −0.040 [−0.068, −0.015] | 0.040 | yes, the market subset grows |
| `recent_form` | no evidence | 1980 | −0.001 [−0.005, +0.004] | 0.007 | little |
| `head_to_head` | no evidence | 1280 | −0.004 [−0.013, +0.005] | 0.013 | little |
| `rest_schedule_load` | no evidence | 2128 | −0.000 [−0.004, +0.004] | 0.006 | little |
| `series_format` | no evidence | 1710 | +0.000 [−0.001, +0.002] | 0.002 | little |
| `line_movement` | no evidence | 733 | +0.000 [−0.004, +0.007] | 0.008 | yes, market history |
| `favourite_longshot` | no evidence | 735 | +0.001 [−0.001, +0.002] | 0.002 | yes, market history |
| `market_source_pinnacle` / `_polymarket` / `_kalshi` / `_average` | no evidence (27 Matches, under the floor) | 27 | 0, +0.024, +0.016, +0.013 | — | yes: needs Matches all three sources priced |
| `roster_stand_in` | no evidence | 791 | +0.001 [−0.004, +0.004] | 0.006 | yes, rosters |
| `roster_change_30d` | harmful | 585 | +0.009 [+0.003, +0.018] | 0.012 | yes, rosters |
| `roster_stability` | no evidence | 332 | +0.015 [−0.003, +0.035] | 0.029 | yes, rosters |
| `roster_coach_change` | no evidence | 728 | +0.002 [−0.002, +0.006] | 0.005 | yes, rosters |
| `lineup_strength` | not testable | 0 | — | — | yes: rosters end August 2025 and HLTV pages begin August 2026, so no Match has both yet |
| `player_form` | not testable | 0 | — | — | yes, same reason |
| `bo1_veto_sim` | not testable | 0 | — | — | yes: none of the 200 HLTV pages is a Bo1; the Swiss Bo1s come with the older pages |
| `region_travel` | not testable | 0 | — | — | perhaps: no Match had a dated LAN host region and dated VRS regions for both teams |
| `llm_adjustment` | not testable (pending) | 0 | — | — | when the operator runs the LLM track |

## Frozen combination and Holdout

Forward selection considered the five `promote`s. `own_rating` entered first and lowered the
walk-forward log-loss of the training side from 0.8055 (the anchor) to 0.6329: −0.173 [−0.232,
−0.116]. After it, none of the other four lowered it further. They all repair the same fault, the
overconfident Valve-rating conversion; at step one they scored 0.6332 to 0.6344. The frozen
combination is **`own_rating`** in the combiner with the market line and the Valve rating.

**Holdout: pending, not evaluated.** No Holdout Match is in the archive. The Holdout is PGL Major
Singapore 2026 after Stage 1, plus anything completed after 2026-11-25 23:46:46 UTC, and the
research archive does not yet hold Singapore at all. The final number needs Singapore's later
Stages to be finished and collected into the archive. After that, one more run of the command
above evaluates the frozen combination once, and the report gives that run's Holdout section.
The date of the run is in the run record.

## Plain reading

- **Worth promoting:** a fix for the overconfident Valve rating. On 2243 Matches the anchor's
  log-loss is 0.806, worse than a coin flip (0.693). Each of `rating_calibration`,
  `learned_weights`, `own_rating` and `per_map_rating` takes off about 0.17. They also slightly
  help the Matches that have a market line (about −0.014 to −0.018 there), so the rule's market
  guard holds. They are one effect, not four: once one is in, the others add nothing.
  `rating_calibration` and `learned_weights` can run live today (#47). `own_rating`, the frozen
  combination, cannot: the live sidecar has no results-history port.
- **`lan_online` promotes on a small effect** (−0.007). It reads 752 Matches and only 62 with a
  market line. That is consistent with an effect the market already prices. Read it as a weak
  rule result, not a reason to add a criterion. The Holdout number is the one to quote.
- **Not worth promoting:** recent form, head-to-head, rest and schedule, series format, line
  movement, favourite-longshot correction, and the roster flags. Each of their intervals sits
  within about ±0.01 of zero. `roster_change_30d` is even slightly harmful (+0.009). `stakes`
  improves the whole set but makes the market Matches worse, so by the written rule it is not
  promoted.
- **Could not be judged for want of data:** lineup strength, player form, the Bo1 veto
  simulation, region and travel, the market source comparison (27 Matches), and the LLM
  adjustment.
- **What the archive could not have seen:** effects smaller than the "Detectable" column. That
  is about 0.002 for series format and the favourite-longshot correction, 0.006 to 0.008 for rest,
  form, stand-ins and line movement, 0.013 for head-to-head, 0.03 for roster stability, and 0.04
  for stakes. The fixed list was tested on one archive, so some `promote`s can be chance. The
  Holdout guards against that.

## Not about one Match

- **Catch-up mode** ([catch-up.md](catch-up.md), 62 Events). Against 12 favourite-followers with
  10% noise, the chance of finishing first rises from 0.180 to 0.397. At 25% noise it rises from
  0.386 to 0.451. The expected place gets worse in both settings, because the policy trades
  average place for the chance of first. Against market-only Members, first rises from 0.931 to
  0.965.
- **Pick'em variants:** not testable on the research archive. None of its 171 Stages has a saved
  Liquipedia opening revision (the opening pairings), so no Stage could be scored. On the saved
  archive (6 Majors, [../pickem.md](../pickem.md)) all three variants are `no evidence` on 18
  Stages.

## LLM track: pending

The recorded answers do not exist yet, and producing them is the operator's run, with a budget
of runs per subscription:

```
cargo run --release -p kallbet-ai -- lab --archive <ARCHIVE> --out docs/lab/research --seed 33 --llm <LLM_FILE> --answers docs/lab/research/llm-answers.json --budget claude=<RUNS> --budget codex=<RUNS> --budget antigravity=<RUNS>
```

`<ARCHIVE>` is the frozen research snapshot above (the lab refuses `--llm` without an explicit
`--archive`: its default is the small test archive), and `<LLM_FILE>` starts from
[../llm.example.json](../llm.example.json). After that run, the same command with
`--answers docs/lab/research/llm-answers.json` and no `--llm` or `--budget` reproduces it with no
provider. This run called no provider.

## Request counters of the limited Data Sources

The lab makes no requests. These counters are the collectors' own ledgers in the live archive,
read before the snapshot (14:52 UTC) and again after the runs (15:34 UTC):

| Data Source | Ledger | Before | After |
|---|---|---:|---:|
| OddsPapi (billable, monthly) | `catalog/market-ledger.json`, 2026-11 | 62 | 62 |
| PandaScore | `collector.db` `spent`, 2026-11-26 | 201 | 201 |
| Valve standings (GitHub) | same, 2026-11-26 | 48 | 48 |
| Liquipedia | same, 2026-11-26 | 300 | 300 |
| HLTV `/matches/*` (400 a day) | `hltv/manifest.json`, 2026-11-26 | 400 | 400 (no stop) |
| LLM subscriptions | — | 0 runs | 0 runs |

During the runs, the market collector saved Polymarket and Kalshi files through the remote host.
Those are public reads with no ledgered limit: `sources-market.tsv` grew from 7479 to 8527 rows.
The `collector.db` ledger only holds 2026-11-26; earlier days of the esports collector are in the
#35 worktree's ledger.

## Two fixes this run needed

- **The combiner's Newton fit now has a line search** (`combiner.rs`). On this archive, the
  rating-only fit of `learned_weights` jumped between −8.98 and 33.4 instead of settling at
  ≈0.28. Before the fix, `learned_weights` read as harmful (+2.25), `stakes` (which shares the
  routine) as +0.41, and forward selection froze nothing. The minimum the function documents is
  unchanged. On the saved archive the fit already converged, so its numbers are unchanged.
  Test: `lab_combiner::a_near_certain_and_often_wrong_rating_is_fitted_to_its_minimum_not_a_cycle`.
- **The #36 map collector's per-serie Liquipedia bundles are read**
  (`hltv/liquipedia/event-<serie>.json.gz`). Before, only `maps-<tournament>` files were read, and
  `per_map_rating` read "not testable" with 1826 linkable Matches in the archive. Test:
  `lab_maps::a_serie_bundle_from_the_map_collector_links_like_a_maps_file`.
