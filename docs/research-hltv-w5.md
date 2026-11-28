# HLTV Rating 3.0 audit, Liquipedia-free recovery, demo pilot (W5, 2026-11-28)

Everything below was written under `D:/Projects/KallBet/target/research-archive-ext/derived/` (`rating-audit/`, `recovery/`,
`demo-pilot/`). The original archive and the snapshots were not touched. New HLTV pages are in `hltv/matches/` with the usual
`hltv/sources.tsv` rows. The scripts are in `sidecar/scripts/` (`hltv_*.py`) and `sidecar/scripts/demo_pilot/`.

## 1. Rating 3.0 audit (saved pages only, no request)

Inputs: all 11,214 saved `hltv/matches/*.html.gz` (10,643 with player statistics, 571 without a stats widget).
Every `table.totalstats | ctstats | tstats` of the `#all-content` block and of each map block was read: 1,046,145
player x scope x side rows, 2,738 players, 10,643 Matches.

### Columns that exist (two header layouts in total)

| Layout | Columns (per player) | Matches (start) | Pages | Notes |
|---|---|---|---:|---|
| Rating 2.0 | K-D, eK-eD, +/-, ADR, eADR, KAST, eKAST, Rating 2.0 | 2023-08-30 .. 2023-10-08 | 93 | The eco-adjusted columns are placeholders (`null-null`, `0.0`, `0.0%`), 0 real values. |
| Rating 3.0 | K-D, eK-eD, **Swing** (`roundSwing`), ADR, eADR, KAST, eKAST, Rating 3.0 | 2023-10-16 .. 2026-11-15 | 10,550 | eco-adjusted values real on 100% (all) / 99.6-99.9% (CT, T tables); Swing real on 100%. |

* No "Eco Swing", no eRating, no KPR/DPR/opening columns and no separate Rating 2.x in the 3.0 era exist on these pages.
  `+/-` exists only in the 2.0 layout (derive it from K-D). MK rating exists only in the POTM card (not extracted).
* `ctstats` and `tstats` (CT-only / T-only) exist on 10,643 / 10,643 pages with the same layout as `totalstats`, overall and per map.
* The switch 2.0 -> 3.0 is by **match date**, between 2023-10-08 and 2023-10-16 (the archive has no Match in between).
* **Fetch date has no spread**: every page was fetched 2026-11-26 .. 2026-11-28 (UTC). A "same match, fetched earlier vs later"
  comparison is therefore impossible from the archive; the audit rests on the match-date axis.

### Is an old value recomputed with the current formula? (AsOf risk)

Yes for the whole 3.0 range: Matches from 2023-10-16 onward carry Rating 3.0, a rating HLTV introduced later (2025, from general
knowledge; the archive itself has no launch date), so the page shows a recomputed value, not one frozen when the Match was played.
Evidence the formula is **one and the same** over the whole range (`formula_stability.csv`: OLS of Rating 3.0 on KPR, DPR,
KAST, ADR and Swing per half-year, per-map rows, n = 3.4k .. 55.8k each):

| half-year | n | intercept | KPR | DPR | KAST | ADR | Swing | R2 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| 2023H2 | 3,410 | 0.474 | 0.495 | -0.168 | 0.163 | 0.382 | 4.467 | 0.961 |
| 2024H1 | 9,590 | 0.489 | 0.477 | -0.201 | 0.173 | 0.395 | 4.363 | 0.963 |
| 2024H2 | 38,266 | 0.483 | 0.484 | -0.207 | 0.175 | 0.400 | 4.327 | 0.960 |
| 2025H1 | 43,754 | 0.489 | 0.485 | -0.214 | 0.177 | 0.398 | 4.323 | 0.960 |
| 2025H2 | 47,409 | 0.479 | 0.482 | -0.203 | 0.178 | 0.406 | 4.323 | 0.959 |
| 2026H1 | 55,794 | 0.486 | 0.482 | -0.204 | 0.177 | 0.402 | 4.331 | 0.961 |
| 2026H2 | 41,702 | 0.487 | 0.486 | -0.209 | 0.180 | 0.397 | 4.305 | 0.961 |

Mean Rating 3.0 per quarter is flat (1.07-1.08, `column_coverage_by_quarter.csv`), Swing is zero-sum (mean -0.02, sd 4.1).
No break at any date, so no sign of a half-way formula change in the saved data.

What the archive cannot show: (a) constants fitted with data after the Match (a small information leak that is the same for
every Match, because the formula is the same), (b) a later HLTV revision of 3.0 (training values and live values would then differ).
Both are not testable with pages fetched in a 3-day window.

### Recommendation (rolling history features of prior Matches only; the table is post-match for its own Match)

| Field | Safe as rolling history? | Why |
|---|---|---|
| kills, deaths, ADR, KAST (traditional) | **Yes, safest**; all dates incl. 2023H2 | formula-free counts and rates |
| Rating 3.0, Swing, eK-eD, eADR, eKAST (overall, per map, CT, T) | **Yes, from 2023-10-16 only**, with a `rating_version` flag | one formula over the whole range (table above); retro-computed, so treat as a small constant AsOf risk, and keep the live pipeline on the same page source |
| Rating 2.0 (93 pages, <= 2023-10-08) | **No** (not comparable with 3.0, no eco/Swing) | the table nulls it (`rating3` is empty on 2.0 rows) |
| Matches without a stats widget (571) | n/a | no rows; match-level data only |
| POTM card, MK rating, anything of the Match itself | No for that Match | post-match |

Extract: `derived/rating-audit/hltv_player_match_stats.parquet` (1,046,145 rows). Key = `match_id, scope, side, player_id`;
`scope` = `all` or the mapstats id, `map_name` filled for map scopes, `side` = `all|ct|t`, `map_rounds` = total rounds of the map
(CT/T rows cover about half), `started_at`/`fetched_at` for the point-in-time filter (use `started_at < M.started_at`).
Columns: `kills, deaths, adr, kast_pct` (all pages) and `rating3, swing_pct, ekills, edeaths, eadr, ekast_pct` (3.0 pages only).
Raw, unfiltered: `player_stats_all_columns.parquet` (every cell as text), `page_headers.parquet`, `pages.parquet`.

## 2. Recovery of Liquipedia-less series and failed parses

* **Series without a Liquipedia event page**: 166 of 423 series in the archive by this count (the notes say 161; this script counts a
  series as titled when `hltv/liquipedia/event-<id>` or `roster-<id>` exists). Their finished Matches: 3,243.
* **Composite link** (PandaScore Match <-> saved HLTV page): start within +-24 h, same Bo format, each team = same name key
  (port of `team_names::team_key`) **or** roster Jaccard >= 0.6 (HLTV lineup against the lineups the PandaScore team showed on
  already linked pages within +-120 days), unique answer only. Result: **9 Matches** (7 tier C, 1 B, 1 S/A 2023-24); 9/9 agree
  with the page's own series score. Small because the earlier collector had already fetched nearly every page it could name.
* **What recovered most: pages named by saved pages.** The past-matches and head-to-head widgets of the saved pages list
  `/matches/<id>/<slug>` ids (1.0M rows). Ids not on disk whose slug is `<teamA>-vs-<teamB>-...` of an unlinked PandaScore Match and
  whose estimated date (exact for h2h rows, id-interpolated +-5 days otherwise) fits were fetched with the collector, in 3 rounds
  (1,325 + 391 + 82 pages = **1,798 pages, 3,596 requests**, 4 connections x >= 3 s, 0 challenges / 403 / 429). A 4th round would
  name 33 more ids (yield has fallen to 0.4 per request); it was not run.
* **Same pair, same day** (ESL Challenger style): 250 Matches had two pages inside +-6 h; resolved one-to-one by series score,
  then smallest start gap.

| | before | after |
|---|---:|---:|
| HLTV pages on disk | 9,416 | 11,214 |
| finished Matches linked (old rule, unique) | 7,526 | 9,047 |
| + resolved same-pair-same-day | 0 | 250 |
| + composite | 0 | 9 |
| **linked Matches** | **7,526** | **9,306** (+1,780) |
| Matches of series without a title that are linked | 27 | 1,320 |
| tier S/A since 2024-08-01 (evaluation block) linked / not | 1,900 / 376 | 2,070 / 206 |

Linked by block after the run: S/A 2023-09..2024-07 637 (+1 composite) of 680; S/A 2024-08+ 2,070 of 2,276; tier B 828 (+1) of 1,207;
tier C 5,762 (+7) of 8,773. `derived/recovery/links.tsv` is the full Match -> HLTV id list (`linked` = baseline | composite,
`base_via` = unique | resolved).

* **Failed parses**: the 570 raw-only pages (562 in the manifest + 8 HTML-only): all have a teams box and maps but **no
  `.matchstats` widget** at all (showmatches, low-tier events). `parse.rs` now accepts such a page when at least one map has a
  score (`overall_stats: []`); a stats widget whose tables do not parse is still an error. Re-run from the raw cache with
  **0 requests: 566 recovered**, 4 remain (Match deleted / maps TBA). `fetched_at` of those JSON files was set back to the
  `sources.tsv` time. Test: `sidecar/tests/hltv_research_parser.rs`.

## 3. CS2 demo pilot (20 Matches, tier S/A, Bo3, newest first, <= 2 per event)

Downloads (`/download/demo/<id>` -> 302 -> `r2-demos.hltv.org/*.rar`), one at a time, one connection, 6 s spacing, same stop rule
(none triggered). 20 of 20 downloaded, extracted (Windows `tar.exe`, bsdtar) and parsed with `demoparser2` 0.4x under `python -I`;
the .rar/.dem files were deleted after each parse and the temp folder was removed. Kept: `derived/demo-pilot/`
`demo_rounds.parquet` (1,152 rounds, one row per round: freeze-end average equipment of both teams, buy class, winner),
`demo_team_map.parquet` (map x team), `demo_team_summary.parquet` (Match x team), `pilot_log.tsv`, `demo_downloads.tsv`
(final URL, bytes, seconds), `selection.tsv`.

| Measure | Value |
|---|---|
| Demos / maps / rounds | 20 / 50 / 1,152 |
| Download size (.rar) | 13.05 GB total, mean 653 MB per Match (402 MB .. 1.22 GB), about 261 MB per map |
| Unpacked (.dem) | 20.29 GB, mean 1.01 GB per Match (about 406 MB per map) |
| Time per demo | download 52.5 s (mean, 12.4 MB/s), unpack 13.7 s, parse 8.6 s: about 75 s, plus 12 s of request spacing (2 requests per demo) |
| Disk use | peak 3.35 GB in the temp folder (one Match); 0 after the run; kept tables < 1 MB |
| Requests | 42 (20 demos x 2 hops + the aborted first attempt of demo 113710, 2 requests: the 30 s client timeout cut it at 455 MB; the demo timeout is now 30 min) |
| Sanity | pistol rounds 100/100 with both teams < 2,000 equipment; pistol win rate 0.50; round 2 after a won pistol: 83% (n = 100); after a lost pistol: 17% (n = 100); eco vs full 7.4% (149 rounds); force vs full 27.4% (259) |

Method notes: `current_equip_value` averaged over a team's 5 players at `round_freeze_end`; classes: full >= 4,000,
eco < 1,500, force between (pistol rounds separate). Pistol rounds are the first round of the map and the first round after the
side swap (not "rounds 1 and 13": a tech-pause restore replays a round, so the round numbering of the round_end event is shifted;
the sequence is rebuilt from `total_rounds_played`, last attempt wins). Team identity is `team_clan_name`; join to a Match by `match_id`.
`full_v_full_rounds` counts each round once per team (a round appears twice).

### Cost to scale to all in-scope Matches (estimate from the pilot, not started)

| Population | Matches with a demo link | Maps | Download | Wall time (sequential) | Requests |
|---|---:|---:|---:|---:|---:|
| In scope (tier S/A, non-qualifier, since 2024-08-01) | 1,932 | 5,386 | about 1.4 TB transferred (never stored) | about 51 h | 3,864 |
| All tier S/A with a page (+ 2023-09..07 and qualifiers) | 2,704 | 7,270 | about 1.9 TB | about 70 h | 5,408 |
| Tier B/C with a page | 6,596 | 18,908 | about 5 TB | about 180 h | 13,192 |

Peak local disk stays about 3 GB (one Match at a time). 1.4 TB at the observed 12 MB/s is the real cost; request count is small.
Pipelining the parse of Match n with the download of Match n+1 would take about 17% off the time. Not scaled up; the operator decides.

## Exact commands

```sh
cd C:/Users/Mykola/orca/workspaces/KallBet/ml-data2        # branch Kulunchick/ml-data2
A=D:/Projects/KallBet/target/research-archive-ext
PY=<venv python 3.12 with lxml cssselect pandas pyarrow demoparser2>          # python 3.15 has no demoparser2 wheel
# 1. audit
$PY sidecar/scripts/hltv_stats_extract.py --archive $A --out $A/derived/rating-audit [--append]
$PY sidecar/scripts/hltv_rating_audit.py  --dir $A/derived/rating-audit
# 2. recovery (pass 1 = index saved pages; then link; fetch the printed ids with the collector; repeat)
$PY sidecar/scripts/hltv_page_links.py   --archive $A --out $A/derived/recovery [--only ID,ID,...]
$PY sidecar/scripts/hltv_recover_links.py                                   # writes fetch_candidates.tsv, links.tsv
tail -n +2 $A/derived/recovery/fetch_candidates.tsv >> $A/hltv/targets.tsv
cd $A && C:/Users/Mykola/AppData/Local/Temp/kb-hltv-t/release/kallbet-ai.exe hltv-collect --archive $A --out $A/hltv \
    --concurrency 4 --min-interval-secs 3 --ios-version 17.2.1 --daily-budget 17000        # also re-parses cached raw pages
# 3. demo pilot
$PY sidecar/scripts/demo_pilot/select_matches.py
$PY sidecar/scripts/demo_pilot/run_pilot.py --selection $A/derived/demo-pilot/selection.tsv --tmp D:/Projects/KallBet/target/demo-pilot-tmp \
    --out $A/derived/demo-pilot --collector C:/Users/Mykola/AppData/Local/Temp/kb-hltv-t/release/kallbet-ai.exe --python $PY
$PY -I sidecar/scripts/demo_pilot/aggregate_pilot.py $A/derived/demo-pilot
# collector build (adds `hltv-collect --demo-ids FILE --out DIR`, streams to disk, >= 6 s, stop.json on 403/429/challenge)
cd sidecar && cargo build --release --features hltv-impersonate        # env: cmake, LLVM, CMAKE_TOOLCHAIN_FILE as in the expand report
```
