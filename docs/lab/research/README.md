# Signal Lab on the research archive: reading (ticket #48)

The full report is [report.md](report.md), with the numbers per Event. Everything
machine-readable is in [report.json](report.json). [catch-up.md](catch-up.md) (simulated Leagues),
[pickem.md](pickem.md) and [llm.md](llm.md) sit next to it. This page says what the report means.
It is the third run, the last of spec #33 before the Holdout: the same archive snapshot as the
second run, protocol `signal-lab/4`, and nine more Candidate Signals added after the list was
fixed. What changed, and why, is at the end.

## How it was made, and how to make it again

```
cargo run --release -p kallbet-ai -- lab --archive <ARCHIVE> --out docs/lab/research --seed 33
```

Then delete `docs/lab/research/audit.sqlite`, the run record, which is not committed. The run
needs no network and calls no provider: it reads only the archive directory.

- **Archive.** A frozen copy of `target/research-archive`, taken 2026-11-26 20:45 UTC after the
  collectors had finished. It is kept at `D:/Projects/KallBet/target/research-archive-t48-snapshot2`.
  The SHA-256 over the replayed files is
  `a556cc446e2f5e75d4494b0fee8b4240e58e8147b3954e23cd959d5cdc0c7c8a` (15,405 files). It is the same
  directory as the second run (`cd61333b…`, 13,433 files): the lab now also reads the raw HLTV
  match pages for their rank badges (`rank_gap`, #57), so more files enter the digest. The archive
  is not in the repository.
- **Protocol.** `signal-lab/4`, seed 33, 2000 Event resamples, 90% intervals, a floor of 100 paired
  Matches. 35 Candidate Signals: the fixed list (25, counting the four market sources), plus ten
  marked "added after list fixed" (post hoc): `bo3_veto_sim` (#52), `bo5_veto_sim` and `veto_sim`
  (#54), `rank_gap` (#57), `stacker` (#58), `stacker_no_map_elo` (#59), `market_rank_rule` (#60),
  `round_diff_rating` (#61) and `bo1_pool_depth` (#62).
- **Reproducibility.** The report holds no wall-clock time; the same command on that archive gives
  the same bytes (checked under `signal-lab/2`). This report is one run.
- **Time.** 13 min 22 s (release build, Windows workstation). Nothing was trimmed.

## What the archive held

62 Events (tier S and A, July 2024 to 15 November 2026), 2288 Matches. Under `signal-lab/4` five
of them are qualifiers or RMRs (162 Matches: the three Shanghai 2024 RMRs, the BLAST Bounty Closed
Qualifier 2025 and the Austin 2025 European Regional Qualifier). Their Matches are neither scored
nor fitted on, but stay in the results, player and map histories. That leaves 57 Events and 2126
scored Matches (tier S 971, tier A 1155): 2087 with the anchor (a Valve rating or a market line),
777 with a market line.

| Input | At this run | Complete? |
|---|---|---|
| Valve rating | 2087 scored Matches with the anchor (2243 with the qualifiers) | yes |
| Market history | 469 Pinnacle fixtures, 3191 Polymarket and 6627 Kalshi files; 777 Matches with a pre-cutoff line, 28 priced by all three sources | yes |
| Liquipedia daily rosters | 577 revisions; 1900 paired Matches for `roster_stand_in` | yes |
| Liquipedia post-Match maps | 1974 paired Matches for `per_map_rating` | yes |
| HLTV match pages | 1975 Matches, 1970 with a veto; their raw pages give the HLTV rank badges `rank_gap` reads (see the leakage finding below) | yes |
| Liquipedia opening revisions (Pick'em) | none | not collected for the research archive |
| PGL Major Singapore 2026 | none, Stage 1 included | needed for the Holdout |

## The Candidate Signals

Δ is the variant minus the anchor on the Matches both predicted. In mean log-loss, below zero is
better. In accuracy (the share of Matches whose favourite won), above zero is better. A promotion
needs the 90% log-loss interval below zero, the Matches with a market line not worse, and the 90%
interval of the accuracy difference (same Matches, same 2000 resamples and seed) not wholly below
zero, since KallBet scores only the picked winner. The last two columns are secondary views that
never change a verdict: the tier-S slice (#55, chosen after results were seen) and the run with
the qualifiers scored and fitted on again (`--include-qualifiers`). "(post hoc)" marks a signal
added after the list was fixed.

| Candidate Signal | Verdict | Paired | Δ log-loss [90%] | Winners picked, anchor → variant | Δ accuracy [90%] | Market subset (n, Δ) | Tier S: verdict, paired, Δ log-loss | With qualifiers: verdict, paired, Δ log-loss |
|---|---|---:|---|---|---|---|---|---|
| `rating_calibration` | promote | 2037 | −0.167 [−0.233, −0.108] | 65.7% → 65.6% | −0.1 pp [−0.4, +0.2] | 776, −0.016 | promote, 894, −0.094 | promote, 2193, −0.176 |
| `learned_weights` | promote | 2087 | −0.161 [−0.222, −0.105] | 65.8% → 65.7% | −0.1 pp [−0.5, +0.2] | 777, −0.012 | promote, 943, −0.091 | promote, 2243, −0.170 |
| `market_source_pinnacle` | no evidence | 28 | +0.000 [+0.000, +0.000] | 57.1% → 57.1% | +0.0 pp [+0.0, +0.0] | 28, +0.000 | not testable | no evidence, 28, +0.000 |
| `market_source_polymarket` | no evidence | 28 | +0.023 [+0.023, +0.023] | 57.1% → 57.1% | +0.0 pp [+0.0, +0.0] | 28, +0.023 | not testable | no evidence, 28, +0.023 |
| `market_source_kalshi` | no evidence | 28 | +0.016 [+0.016, +0.016] | 57.1% → 57.1% | +0.0 pp [+0.0, +0.0] | 28, +0.016 | not testable | no evidence, 28, +0.016 |
| `market_source_average` | no evidence | 28 | +0.012 [+0.012, +0.012] | 57.1% → 57.1% | +0.0 pp [+0.0, +0.0] | 28, +0.012 | not testable | no evidence, 28, +0.012 |
| `line_movement` | no evidence | 744 | +0.001 [−0.003, +0.007] | 67.2% → 67.3% | +0.1 pp [−0.8, +1.0] | 744, +0.001 | no evidence, 322, −0.004 | no evidence, 744, +0.001 |
| `favourite_longshot` | no evidence | 746 | +0.001 [−0.000, +0.002] | 67.2% → 66.9% | −0.3 pp [−0.6, +0.0] | 746, +0.001 | no evidence, 324, +0.000 | no evidence, 746, +0.001 |
| `own_rating` | promote | 2028 | −0.158 [−0.223, −0.100] | 65.6% → 64.7% | −0.9 pp [−2.4, +0.3] | 758, −0.013 | promote, 929, −0.087 | promote, 2168, −0.168 |
| `recent_form` | no evidence | 1859 | −0.001 [−0.006, +0.004] | 65.8% → 65.5% | −0.3 pp [−1.0, +0.3] | 697, +0.006 | no evidence, 847, −0.001 | no evidence, 1980, −0.001 |
| `head_to_head` | no evidence | 1244 | −0.003 [−0.011, +0.005] | 66.3% → 65.7% | −0.6 pp [−1.8, +0.5] | 561, +0.005 | no evidence, 583, +0.001 | no evidence, 1280, −0.004 |
| `llm_adjustment` | not testable | 0 | — | — | — | — | — | — |
| `lineup_strength` | promote | 1812 | −0.003 [−0.005, −0.001] | 65.9% → 65.6% | −0.3 pp [−0.8, +0.2] | 641, −0.002 | no evidence, 888, −0.002 | promote, 1950, −0.003 |
| `player_form` | no evidence | 1480 | +0.000 [−0.000, +0.000] | 66.8% → 66.7% | −0.1 pp [−0.3, +0.1] | 483, +0.000 | harmful, 728, +0.000 | no evidence, 1567, +0.000 |
| `per_map_rating` | promote | 1974 | −0.155 [−0.224, −0.093] | 65.3% → 64.0% | −1.3 pp [−3.0, +0.3] | 724, −0.011 | promote, 915, −0.084 | no evidence, 2109, −0.165 |
| `bo1_pool_depth` (post hoc) | no evidence | 251 | −0.005 [−0.011, +0.003] | 60.2% → 60.6% | +0.4 pp [−0.9, +1.3] | 49, +0.011 | no evidence, 217, −0.003 | no evidence, 312, −0.024 |
| `bo1_veto_sim` | promote | 251 | −0.082 [−0.133, −0.021] | 59.4% → 61.4% | +2.0 pp [+0.0, +4.4] | 54, −0.003 | promote, 217, −0.087 | promote, 319, −0.088 |
| `bo3_veto_sim` (post hoc) | no evidence (fewer winners picked) | 1771 | −0.161 [−0.240, −0.088] | 66.5% → 63.6% | −2.8 pp [−4.5, −1.4] | 702, −0.018 | no evidence, 695, −0.071 | no evidence, 1850, −0.170 |
| `bo5_veto_sim` (post hoc) | no evidence | 52 | −0.116 [−0.322, +0.063] | 75.0% → 69.2% | −5.8 pp [−13.5, +0.0] | 21, +0.017 | no evidence, 20, +0.082 | no evidence, 52, −0.116 |
| `veto_sim` (post hoc) | no evidence (fewer winners picked) | 2074 | −0.151 [−0.221, −0.087] | 65.8% → 63.5% | −2.3 pp [−3.8, −1.0] | 777, −0.016 | promote, 932, −0.071 | no evidence, 2221, −0.157 |
| `rank_gap` (post hoc) | promote | 2086 | −0.163 [−0.226, −0.106] | 65.8% → 65.6% | −0.2 pp [−0.7, +0.3] | 776, −0.009 | promote, 943, −0.093 | promote, 2242, −0.173 |
| `market_rank_rule` (post hoc) | no evidence | 2087 | +0.003 [−0.001, +0.009] | 65.8% → 65.3% | −0.5 pp [−1.4, +0.3] | 777, +0.009 | no evidence, 943, +0.001 | no evidence, 2243, +0.003 |
| `round_diff_rating` (post hoc) | promote | 2084 | −0.154 [−0.223, −0.090] | 65.8% → 64.3% | −1.6 pp [−3.4, +0.1] | 777, −0.016 | no evidence, 940, −0.080 | no evidence, 2240, −0.162 |
| `stacker` (post hoc) | promote | 2087 | −0.162 [−0.226, −0.104] | 65.8% → 65.8% | +0.0 pp [−0.7, +0.6] | 777, −0.006 | no evidence, 943, −0.089 | promote, 2243, −0.171 |
| `stacker_no_map_elo` (post hoc) | promote | 2087 | −0.163 [−0.227, −0.105] | 65.8% → 65.6% | −0.2 pp [−0.7, +0.3] | 777, −0.009 | no evidence, 943, −0.089 | promote, 2243, −0.172 |
| `roster_stand_in` | no evidence | 1900 | +0.000 [−0.001, +0.001] | 66.1% → 66.2% | +0.1 pp [+0.0, +0.2] | 651, −0.001 | no evidence, 941, +0.000 | no evidence, 2056, −0.000 |
| `roster_change_30d` | harmful | 1427 | +0.005 [+0.001, +0.008] | 65.7% → 65.7% | +0.0 pp [−0.3, +0.3] | 504, +0.002 | no evidence, 759, +0.003 | harmful, 1563, +0.004 |
| `roster_stability` | harmful | 1471 | +0.004 [+0.002, +0.006] | 65.7% → 65.4% | −0.3 pp [−0.8, +0.2] | 621, +0.001 | harmful, 717, +0.005 | no evidence, 1534, +0.005 |
| `roster_coach_change` | no evidence | 1806 | +0.004 [−0.000, +0.008] | 65.8% → 65.6% | −0.2 pp [−0.6, +0.1] | 615, +0.002 | no evidence, 904, +0.004 | harmful, 1903, +0.004 |
| `rest_schedule_load` | no evidence | 1988 | +0.001 [−0.003, +0.004] | 65.6% → 65.6% | +0.1 pp [−0.7, +0.7] | 758, −0.002 | no evidence, 889, +0.004 | no evidence, 2128, −0.000 |
| `stakes` | no evidence (market subset worse) | 630 | −0.038 [−0.065, −0.012] | 65.4% → 65.4% | +0.0 pp [+0.0, +0.0] | 216, +0.018 | no evidence, 476, −0.026 | no evidence, 630, −0.038 |
| `lan_online` | promote | 1952 | −0.005 [−0.007, −0.002] | 65.5% → 65.2% | −0.3 pp [−0.7, +0.1] | 756, −0.002 | no evidence, 886, −0.004 | no evidence, 2078, −0.005 |
| `series_format` | no evidence | 1614 | +0.000 [−0.000, +0.001] | 65.4% → 65.1% | −0.3 pp [−0.7, +0.0] | 644, +0.001 | no evidence, 798, +0.000 | no evidence, 1710, +0.000 |
| `region_travel` | no evidence | 107 | +0.001 [+0.000, +0.001] | 68.2% → 67.3% | −0.9 pp [−0.9, +0.0] | 105, +0.002 | no evidence, 107, +0.001 | no evidence, 107, +0.001 |

"Detectable" (the smallest log-loss effect these Matches would show four times in five) is in
[report.md](report.md): about 0.09 to 0.10 for the rating signals, 0.08 for `bo1_veto_sim`, 0.29
for `bo5_veto_sim`, under 0.01 for the small adjustments.

`market_rank_rule` names its own subset: on the 127 Matches where the market favourite and the
rank favourite differ, it picks 44.9% of winners against the anchor's 53.5%.

## Frozen combination and Holdout

Forward selection considered the eleven `promote`s. The training side's walk-forward log-loss
started from the anchor's 0.7943:

| Step | Entered | Log-loss after | Added [90%] |
|---|---|---:|---|
| 1 | `round_diff_rating` | 0.6302 | −0.1641 [−0.2261, −0.1073] |
| 2 | `rank_gap` | 0.6291 | −0.0011 [−0.0034, +0.0010] |
| 3 | `lineup_strength` | 0.6289 | −0.0002 [−0.0003, −0.0001] |

The frozen combination is **`round_diff_rating` + `rank_gap` + `lineup_strength`**, in the combiner
with the market line and the Valve rating. At step 1 every rating fix scored between 0.6302 and
0.6333: they all repair the same overconfidence, and `round_diff_rating` won by 0.001. Steps 2 and
3 are small, and step 2's interval crosses zero. Selection has no significance gate: it stops
only when nothing lowers the log-loss. Two of the three members are post hoc.

**Live status** (`sidecar/src/promotion.rs`). A promoted criterion is selectable per bot, but only
the members in `promotion::LIVE` change a live Prediction: `rating_calibration` and
`learned_weights`. Their fits are in `report.json` under `deployment`. No member of the frozen
combination has a live input port: `round_diff_rating` needs HLTV map round scores of earlier
Matches, `rank_gap` VRS and HLTV ranks, and `lineup_strength` HLTV player ratings. A bot promoted
with `combination` therefore loads, but decides every Match by the anchor and says which input is
missing. The same holds for every other promoted signal alone (`own_rating`, `per_map_rating`,
`bo1_veto_sim`, `rank_gap`, `round_diff_rating`, `stacker`, `stacker_no_map_elo`, `lan_online`,
`lineup_strength`). A bot gets the measured gain live today only through `rating_calibration`
(−0.167) or `learned_weights` (−0.161), which pick as many winners as the anchor (−0.1 pp,
interval across zero).

**Holdout: pending, not evaluated.** No Holdout Match is in the archive. The Holdout is PGL Major
Singapore 2026 after Stage 1, plus anything completed after 2026-11-25 23:46:46 UTC, and the
archive holds no Singapore Stage at all. Once Singapore's later Stages are played and collected,
one more run of the command above evaluates the frozen combination once. The report's Holdout
section then states the evaluation date.

## Plain reading

- **What improves probabilities:** a fix for the overconfident Valve rating. The anchor's log-loss
  on the scored Matches is 0.794, worse than a coin flip (0.693). Every rating fix takes off about
  0.15 to 0.17: `rating_calibration`, `learned_weights`, `own_rating`, `per_map_rating`,
  `round_diff_rating`, `rank_gap`, `stacker`, `stacker_no_map_elo`, and the veto simulations.
  These are one effect, not ten. On the tier-S slice the gain halves (about −0.09) but stays clear.
  On the Matches with a market line it is −0.01 to −0.02, because the market is already
  calibrated. `bo1_veto_sim` helps on its 251 Bo1s (−0.082). `lineup_strength` and `lan_online`
  are promoted by the rule but tiny (−0.003, −0.005).
- **What picks more winners:** nothing, measurably. No Candidate Signal's accuracy interval lies
  wholly above zero. The best is `bo1_veto_sim`, +2.0 pp on 251 Bo1s with an interval touching
  zero [+0.0, +4.4]. `stacker`, built to pick winners (#58), picks exactly as many as the anchor
  (65.8%). `market_rank_rule` (#60) picks fewer (−0.5 pp overall; 44.9% against 53.5% on the 127
  disagreements). The map simulations `bo3_veto_sim` and `veto_sim` pick clearly fewer (−2.8 and
  −2.3 pp) and are not promoted; `per_map_rating` (−1.3 pp) and `round_diff_rating` (−1.6 pp) lean
  the same way, with intervals just reaching zero.
- **What this means for a KallBet bot:** KallBet scores the picked winner, and the anchor (the
  market where there is one) already picks it as well as anything measured here. The rating fixes
  give a better-calibrated probability, which matters for the Rationale's win percentage and for
  Pick'em, not for Match Points. The live choice is `rating_calibration` or `learned_weights`, the
  two that can run live; expect the same picks as the anchor.
- **Not worth promoting:** recent form, head-to-head, rest and schedule, series format, stand-ins,
  player form, line movement, favourite-longshot correction, region/travel, Bo1 pool depth and the
  market-vs-rank rule. All sit within about ±0.01 of zero. `roster_change_30d` and
  `roster_stability` are slightly harmful (+0.004 to +0.005). `stakes` improves the whole set but
  worsens the Matches with a market line (+0.018).
- **Could not be judged for want of data:** the market source comparison (28 Matches), the Bo5
  veto simulation (52 Bo5s), region and travel (107 Matches in 2 Events), the LLM adjustment (not
  run), Pick'em on the research archive (no opening revisions).
- **Caution:** with 35 candidates on one archive, ten of them chosen after earlier results were
  seen, a small `promote` can be chance. The Holdout is the guard, and it is pending.

## Not about one Match

- **Catch-up mode** ([catch-up.md](catch-up.md), 62 training-side Events; the simulated Leagues
  replay whole Events, so the qualifier rule does not change them). Against 12
  favourite-followers, the chance of finishing first rises with the catch-up policy:
  - at 10% noise, from 0.192 to 0.394;
  - at 25% noise, from 0.395 to 0.450.

  The expected place gets worse, because the policy trades average place for the chance of first.
  Against market-only Members, first rises from 0.963 to 0.984.
- **Pick'em variants:** not testable on the research archive. None of its Stages has a saved
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

## What changed since the previous runs, and why

### Third run (`signal-lab/4`, this one) against the second (`signal-lab/3`)

- **Qualifiers and RMRs left out (`signal-lab/4`, #55).** Five Events, 162 Matches, are neither
  scored nor fitted on; their results still feed every history. The anchor's log-loss on the
  scored Matches went from 0.8044 to 0.7943, and the rating fixes' gains shrank slightly (for
  example `rating_calibration` −0.176 → −0.167). Verdicts that changed because of it:
  `per_map_rating` and `lan_online` from no evidence (fewer winners picked) to promote (their
  accuracy intervals now reach zero: −1.3 pp and −0.3 pp); `roster_coach_change` from harmful to
  no evidence; `roster_stability` from no evidence to harmful. The with-qualifiers column shows the
  old population, where `per_map_rating` and `lan_online` stay no evidence.
- **Tier-S slice (#55)**, a secondary view: the same signals evaluated on tier-S Events only,
  fitted walk-forward on all tiers. It changes no verdict.
- **New post-hoc Candidate Signals**, each marked "added after list fixed":
  - `bo5_veto_sim`, `veto_sim` (#54): Bo5 and format-routed veto simulation. `veto_sim` is no
    evidence (fewer winners picked), `bo5_veto_sim` under the floor.
  - `rank_gap` (#57): log rank gaps from the dated Valve standings and HLTV's weekly ranking.
    **Leakage finding:** the VRS badge on HLTV match pages is not point-in-time. It moves with the
    Match's own result in 83% of day-to-day changes (59% with the previous Match's), and most of
    its 72% "better rank wins" rate is that leak: the dated Valve snapshot before the Match gives
    65.9%. `rank_gap` therefore reads VRS from the dated Valve snapshots and takes only the HLTV
    badge from the pages (known once its ranking's day is over). Audit:
    [criteria-research.md](../../signal-lab/criteria-research.md#c1-leakage-audit-ticket-57),
    script `sidecar/scripts/audit_rank_badges.py`. Promoted; picks as many winners as the anchor.
  - `stacker` (#58): a combiner whose penalty is chosen by winners picked on later training
    Events. Promoted; no more winners picked.
  - `stacker_no_map_elo` (#59): `stacker` without `per_map_rating`, an ablation. Within 0.001 of
    `stacker`: the map-level Elo adds nothing to it.
  - `market_rank_rule` (#60): follows the rank favourite against the market where that won more
    often before. No evidence, and worse on its own subset.
  - `round_diff_rating` (#61): opponent-adjusted Elo on round differential. Promoted, and the first
    member of the frozen combination.
  - `bo1_pool_depth` (#62): Bo1 map-pool depth and first-ban features. No evidence.
- **Frozen combination** changed from `own_rating` + `lineup_strength` to `round_diff_rating` +
  `rank_gap` + `lineup_strength`. It still has no live port, so live bots are unaffected.
- **Report fixes made at this run.** The Pick'em report listed the excluded qualifier Stages as
  "in the Holdout"; it now names them as qualifiers. Stray indentation in the qualifier and tier
  texts of `report.md` was removed.

### Second run (`signal-lab/3`) against the first run

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
- **Verdict rule `signal-lab/3`** (second run). A promotion also needs the winners picked not to fall:
  `per_map_rating`, `bo3_veto_sim` and `lan_online` went from promote to no evidence ("better
  probabilities but fewer winners picked"). The frozen combination lost `lan_online`. Every number
  is otherwise unchanged; the report now gives each candidate's accuracy interval.
- **Map-name canonicalisation (#52).** `per_map_rating` now joins HLTV "Dust2" and Liquipedia
  "Dust II". It moved from −0.1662 to −0.1654 on the same 2109 Matches.
- **`bo3_veto_sim` (#52)** is new and marked "added after list fixed". Under `signal-lab/2` it
  promoted (−0.170 on 1850 Bo3s) but entered no step of the selection; under `signal-lab/3` it is
  no evidence (fewer winners picked).
- The Holdout was pending in both runs, for the same reason: Singapore is absent.
