# Criteria that could pick more winners (ticket #56)

**Parent:** #33. **Date checked:** 2026-11-26 (every web source below, and every archive check).
**Question:** #48 showed that nearly every promoted Candidate Signal improves log-loss by repairing the
Valve rating's overconfidence, but almost none picks more winners. KallBet scores only the picked winner
(1 Point per correct Prediction), and the anchor's accuracy stays at about 65.7%. Which further criteria could
raise accuracy, or tell a Bot User when its pick is reliable, and can we test them with data we hold or
can get for free?

Research only: no code was written, no rate-limited API was called, no LLM provider was run. Sources
for Matches are the frozen archive `D:/Projects/KallBet/target/research-archive-t48-snapshot2`
(read-only, offline, SHA-256 `cd61333b…55a7`). Numbers marked **[archive check]** are mine, from that
archive, with throw-away Python (`python -I`, not committed); the procedure is in the
[appendix](#appendix-archive-checks) so a Candidate Signal ticket can redo them properly.

## Summary

1. **Accuracy and log-loss are different problems.** #48's gain is calibration: the anchor's log-loss
   (0.804) is worse than a coin flip (0.693) because the Valve rating is overconfident, and rescaling
   it fixes that without changing which side is favoured. A pick is the sign of `p − 0.5`, so only a
   criterion that moves `p` *across* 0.5 changes a Prediction. Of the 26 signals, the ones that reach for
   new information (`own_rating`, `per_map_rating`, `bo3_veto_sim`) even *lowered* accuracy against the anchor
   (−1.2, −1.7 and −2.9 percentage points, report.md).
2. **The biggest lead is a data item we hold but never parsed.** Every cached HLTV match page (1975)
   carries a point-in-time **HLTV rank and VRS rank badge for both teams** in the raw HTML. The better VRS
   rank won **72.0%** of 1968 Matches **[archive check]**, against 65.7% for the anchor on the lab's
   2243 Matches. The sets differ and the badge needs a leakage audit before anyone believes the gap
   (see C1), but nothing else in this document has a gap that size.
   *Ticket #57's audit (C1): the VRS badge is not pre-Match (it already moves with the Match's own result);
   the dated Valve snapshot before the Match picks 65.9% of the same Matches. The HLTV badge is point-in-time.*
3. **Margin of victory does not look like the lever.** Updating a team rating by round share instead of
   map win/loss gave the same accuracy (66.1% vs 66.0%, 889 held-out series) **[archive check]**. What
   helped was rating at *map* level rather than series level (+1.9 points, inside the noise).
4. **No criterion here can make a Prediction abstain**, because an unmade Prediction scores nothing.
   Confidence is useful only for deciding *which source to follow when they disagree*.

## Ranked candidates

"Effect" is the expected change in pick accuracy against the anchor (65.7%), with the reason for the
width. Where I could not measure it, the range is a judgement and says so. Series have a standard
error of about ±1.0 point at 2000 Matches and ±1.6 points at 900, so nothing under ±2 points is
detectable on this archive without a paired test; the Signal Lab's paired Event resampling is the right tool.

| Rank | Criterion | Effect on accuracy | Data in archive? | Cost to get otherwise | Priority |
|---:|---|---|---|---|---|
| 1 | **C1** Pre-match VRS / HLTV rank (rank gap) from HLTV page badges | +1 to +6 points; wide until audited | Yes: raw HTML of all 1975 pages (not yet parsed) | none | **Build** |
| 2 | **C2** Accuracy-fitted stacker: market line + Valve rating + C1 + map-level rating | +0.5 to +2 | Yes, all parts | none | **Build** |
| 3 | **C3** Wider market coverage: HLTV page odds, closing vs opening, extra books | unknown; 0 to +2 | Partly: 777 of 2243 Matches have a line; odds also in all 1975 HLTV pages (provenance unverified) | free tiers only; OddsPapi/Pinnacle history is rate-limited | **Build after an audit** |
| 4 | **C4** Map-level rating as the base rating (win/loss per map, recency-weighted) | 0 to +2 | Yes: HLTV maps (1975), Liquipedia maps (62 Events) | none | **Build** |
| 5 | **C5** Disagreement rule: follow the market or the ranks, by gap size | +0.5 to +1.5 | Yes | none | **Build** (on top of C1 and C3) |
| 6 | C6 Margin-of-victory (round differential) rating | 0 to +0.5 | Yes | none | Skip |
| 7 | C7 Side-specific (CT/T) map strength | 0 to +0.3 | Partly (Liquipedia has first side; HLTV pages lose it) | parse only | Skip |
| 8 | C8 Map-pool depth and permaban patterns | ≤ 0 | Yes; `bo1_veto_sim` and `bo3_veto_sim` already exist | none | Skip |
| 9 | C9 HLTV ranking *points* | ≈ 0 beyond C1 | No: only the rank number is on the pages | scrape of `/ranking/teams`, blocked by our HLTV rules | Skip |
| 10 | C10 Player Rating 3.0 team form | 0 to +0.5 | Yes: per-player, per-map | none | Skip (`lineup_strength` was +0.003 log-loss) |
| 11 | C11 Differentiating from the League (not accuracy) | 0 on accuracy | n/a | none | Note only |

## Evidence from the archive, at a glance

All **[archive check]**; walk-forward over the 1974 decided HLTV Matches (2024-07 to 2026-11), each
team needing 5 prior Matches in the file, K tuned on the first half and scored on the second.

| Predictor | Matches | Accuracy | Log-loss |
|---|---:|---:|---:|
| Series-level Elo, win/loss (K=48) | 889 held out | 64.1% | 0.634 |
| Map-level Elo, win/loss (K=24) | 889 | 66.0% | 0.636 |
| Map-level Elo on round share (K=24) | 889 | 66.1% | 0.640 |
| Mean HLTV Rating 3.0 of last 20 Matches, higher wins | 889 | 63.3% | n/a |
| Better VRS rank on the page | 1968 | **72.0%** | n/a |
| Better HLTV rank on the page | 1968 | 65.4% | n/a |
| Anchor (market 0.80 + Valve rating 0.20), lab report | 2243 | 65.7% | 0.804 |

Where the two rank badges point at different teams (255 Matches), the VRS side won 192 (75%) and the
HLTV side 63 (25%). VRS accuracy by rank gap: gap 0–4 → 61.7% (579 Matches), 5–14 → 72.8% (815),
15–39 → 78.9% (445), 40+ → 89.1% (129). Confidence of the map-level Elo (distance from 0.5) against pick
accuracy: <0.1 → 61.9% (583), 0.1–0.2 → 72.8% (257), 0.2–0.3 → 79.5% (44).

**Do not compare the 72.0% to 65.7% as if it were a result.** The Matches differ (1968 with a page vs 2243
scored), and the 65.7% is the anchor *blended with the market*. The comparison tells us where to look,
not how much we gain.

## Candidates

### C1. Pre-match VRS / HLTV rank, from the HLTV match page

- **Measures.** Each team's ordinal rank in Valve's Regional Standings and in HLTV's world ranking at the
  time of the Match, shown as `ranking-badge vrs-ranking` and `ranking-badge hltv-ranking` on every
  `/matches/*` page. The badge links carry the standings date (e.g. `/valve-ranking/teams/2025/august/1`,
  `/ranking/teams/2025/july/28/<team>`), so the data is meant as of the Match, not as of the fetch.
- **Evidence it predicts winners.** Valve's own repository says the standings exist to "predict future
  match results" and publishes a calibration check of expected against observed win rates, with Spearman
  ρ = 0.98 but a slope shallower than ideal, i.e. the rating is overconfident, which is the same defect #48 found
  ([readme](https://github.com/ValveSoftware/counter-strike_regional_standings/blob/main/readme.md),
  checked 2026-11-26). The model is Glicko with a fixed RD of 75, updated from match win/loss with bounty,
  opponent-network and LAN factors ([model/ranking.js](https://github.com/ValveSoftware/counter-strike_regional_standings/blob/main/model/ranking.js),
  2026-11-26). Our archive check above is the first test on Matches of this project.
- **Why accuracy could move.** The anchor's Valve rating is, by the lab's own reading, one input that
  enters at weight 0.20. If the rating it reads is staler than the page's badge, the badge is a fresher input to the same
  signal. I did not check how the lab dates its Valve rating; that is the first thing the Candidate
  Signal ticket has to settle.
- **Data.** Held: raw `hltv/matches/hltv-match-<id>.html.gz` (1975 pages). The parsed `HltvMatchRecord`
  has no rank field, so a parser change is needed (`sidecar/src/adapters/hltv_research/parse.rs`). Rank number only, no points.
- **Risks to audit first.** (a) Leakage: for 1947 of 1968 pages the badge date equals the Match date
  and for 21 it is the next day; Valve publishes weekly, so confirm that the rank shown is the pre-Match
  standing and does not already include the Match. No page has a badge dated before the Match day,
  so the audit cannot be a date filter. Compare the badge with the published Valve standings of the
  preceding week and check that a Match's own result never moves its rank. (b) Rank is ordinal; a gap table or rank→probability map must be fitted per format on prior Events, like the lab's other signals.
  (c) Coverage starts where HLTV pages start (2024-07-29), and the 8 walkover pages have no rank.
- **Cost.** Zero requests.
- **Priority.** Build first.

#### C1 leakage audit (ticket #57)

Question: does a badge show the rank as of the Match, or as of the page fetch (2026-11-26)? And if as of
the Match, is it the rank *before* the Match? Reproduce with
`python -I sidecar/scripts/audit_rank_badges.py <research archive>` (offline, read-only; all 1975 pages
with a parsed record and raw HTML, the archive's 48 dated Valve standings).

| Check | VRS badge | HLTV badge |
|---|---|---|
| Badge dated by the fetch day | 0 of 3940 badges | 0 of 3940 |
| Badge date against the Match day | the Match day (3898) or the day after (42): the link is built from the Match date, not from a publication | 0–7 days before (a weekly ranking): 1 to 7 days for 3828, the Match day itself for 112 (Monday Matches) |
| Agreement with the Valve snapshot of the same date (where the archive has one) | 107 of 233 team ranks equal, 181 within 2 | no dated HLTV source in the archive |
| A team's next page, 1 day later, rank changed: the change follows the result of the **earlier** page's Match (already played) | 688 of 1168 (59%) | 101 of 174 (58%) |
| ... follows the result of the **later page's own** Match (possible only if the badge already holds it) | **966 of 1168 (83%)** | 91 of 174 (52%, chance) |
| Same, next page within 14 days: earlier / own | 998 / **1305** of 1656 | 258 / 204 of 417 |
| Better rank picks the winner (1928 Matches with both badges and a Valve snapshot before the Match day) | badge 1386 (71.9%); latest Valve snapshot before the Match day 1270 (65.9%) | badge 1259 (65.3%) |

**Verdict.**

- *Not fetch-time.* No badge carries the fetch date; both are dated near the Match.
- *VRS badge: not point-in-time; must not be used.* Its rank moves with the Match's own result far more
  often than with the previous Match's (83% against 59%). It is HLTV's VRS figure for the Match day after that
  day's results, not the standing a bettor could read before the Match. Most of the 72.0% headline (item 2 above)
  is this leak: the dated Valve snapshot before the Match picks 65.9% of the same Matches. `rank_gap` builds its VRS
  part from the archive's dated Valve snapshots instead (latest strictly before the cutoff, known once its UTC day is
  over, as the anchor's own rating reads them).
- *HLTV badge: point-in-time when dated by its ranking.* It links to the weekly ranking on or before the Match
  day, and its changes follow the earlier Match's result, not the own one (52%, chance). `rank_gap` reads it as
  known once the ranking's UTC day is over, so the 112 badges published on the Match day itself are not used for
  that Match; at most 35 days old. Any page may tell a team's rank (a later page carries the same earlier ranking),
  except a Holdout Match's page for a training-side view. The remaining risk is that HLTV re-renders an old
  ranking with a revised number; no dated HLTV source in the archive can check that, so it is not checked.
- Parser: `hltv_research::rank_badges::parse_rank_badges`, tested on the saved pages in `sidecar/tests/fixtures/research/`.
- *Lab result* (`rank_gap` alone, research archive digest `a556cc44…7c8a`, seed 33, protocol signal-lab/3; added after the
  list was fixed): computed on 2242 of 2243 Matches, paired 2242 over 61 Events; Δ log-loss −0.173 (90% interval
  −0.235 to −0.116, detectable 0.090), Δ Brier −0.025, Δ accuracy −0.18 points (interval −0.78 to +0.37); Matches with a
  market line (776) Δ log-loss −0.009. Verdict **promote**. The fitted weights say where the gain comes from: on the
  1466 Matches with no market line the combiner shrinks the overconfident Valve rating (weight 0.08) and leans on the
  rank gaps (HLTV 0.30, VRS 0.20); with a market line the gaps get almost nothing. So it is better probabilities,
  not more winners picked: the HLTV badge was the only new information, and it picks fewer winners (65.3%) than the
  dated Valve snapshot (65.9%).

### C2. Accuracy-fitted stacker (the "ensemble" candidate)

- **Measures.** A combiner whose inputs are the market probability, the Valve rating, C1's rank gap and
  a map-level rating, fitted on the Events that ended before (the lab's walk-forward protocol), with the
  fit judged on accuracy as well as log-loss.
- **Evidence.** The lab's forward selection added only `own_rating`, then `lineup_strength` (−0.0004)
  and `lan_online` (−0.0001): the signals competed on log-loss, and every one of them repairs the
  same overconfidence, so they are largely redundant (report.md, "Frozen combination"). Redundancy in log-loss does not make
  inputs redundant in accuracy: the ratings differ most where they disagree. Published work on rating
  ensembles for football finds that Elo-type ratings are competitive with, but not clearly
  better than, bookmaker odds (Hvattum & Arntzen, [Int. J. Forecasting 26(3), 2010](https://ideas.repec.org/a/eee/intfor/v26yi3p460-470.html)),
  and esports prediction work uses Elo/Glicko/TrueSkill as baselines or features, not as final models
  ([EsportsBench preprint](https://cthorrez.github.io/papers/esportsbench/EsportsBench_preprint.pdf), secondary).
  CS:GO and CS2 models reach 62–64% on ratings alone ([CTU thesis](https://dspace.cvut.cz/server/api/core/bitstreams/7d11f698-b099-4a6a-acd3-2947f72c6aa1/content): Elo 64%, random forest 63%;
  [UCT dissertation](https://open.uct.ac.za/items/26d92119-6b96-4af2-8408-232cb8f62200): best 62.7%, TrueSkill). Both are student theses, so treat the numbers as
  indicative only.
- **Data.** Held.
- **Expected effect.** +0.5 to +2 points, mostly from C1 entering a combiner, small otherwise. I expect no
  gain from adding a fifth rating to four.
- **Priority.** Build, after C1 (it is how C1 gets used).

### C3. More market coverage: other bookmakers, closing vs opening

- **Measures.** The implied probability of the market, which already outweighs the Valve rating 4:1 in the
  anchor, over more Matches and from more books.
- **Evidence.** The lab could not judge the source comparison: only 28 Matches carry Pinnacle, Polymarket and Kalshi lines
  together. `line_movement` (+0.0008 log-loss, 744 Matches) shows no value in movement. Primary evidence that closing prices
  are the sharpest is thin in what I could find: practitioner pages say Pinnacle's close is the benchmark, one
  senior thesis finds large relative line movements less accurate ([Harvard, 2019](https://dash.harvard.edu/bitstream/handle/1/37364609/BOUCHARD-SENIORTHESIS-2019.pdf?sequence=1)),
  and I found no peer-reviewed opening-vs-closing study. Treat "closing beats opening" as plausible, not established.
- **Data.** 777 of 2243 Matches have a pre-cutoff line (469 Pinnacle fixtures, 3191 Polymarket and 6627 Kalshi files). Separately,
  all 1975 cached HLTV pages carry a two-sided decimal-odds widget (`team-odds-odds`). I could not verify its provider or its time
  (the page was cached after the Match), and my name-based alignment of the two odds to `team_a`/`team_b` failed on most pages, so I
  have no accuracy figure for it. If it is a close, it would triple market coverage for free.
- **Step zero, no new data.** Report the anchor's *accuracy* separately on the 777 Matches with a line and the 1466 without.
  The report publishes log-loss on the market subset only. That one table says whether coverage can matter.
- **Cost otherwise.** OddsPapi and PandaScore are rate-limited and excluded here; Polymarket and Kalshi history is public but already collected.
- **Priority.** Audit the HLTV widget, then build if it is a real close.

### C4. Map-level rating as the base rating

- **Measures.** Elo (or Glicko) updated per map played instead of per series, combined to a series probability
  with the Bo1/Bo3/Bo5 conversion `signals::vrs::series_probability` already has.
- **Evidence.** Archive check: +1.9 points over series-level Elo (66.0% vs 64.1%, 889 Matches, 1 standard error ≈ 1.6),
  with the same log-loss. Valve's own model works at match level
  (`glicko.singleMatch(winTeam, loseTeam, informationContent)`), so a map-level rating is information it does not use. #48's `per_map_rating`
  is already a map rating, but it blends with the anchor and its accuracy fell (−1.7 points). So this is only worth building
  if C2 uses it as an input rather than as a replacement.
- **Data.** Held (HLTV maps for 1975 Matches, Liquipedia maps for 62 Events).
- **Priority.** Build as an input of C2, not on its own.

### C5. Disagreement and confidence rule

- **Measures.** Which source to follow when the market and the ranks pick different teams, and how sure a pick is.
- **Evidence.** Archive check: accuracy rises with distance from 0.5 (61.9% → 72.8% → 79.5%) and with
  VRS rank gap (61.7% → 89.1%). The two rank systems disagree on 13% of Matches and VRS wins 75% of those,
  so "follow VRS over HLTV rank" is a rule the archive already supports. No signal in the lab is
  tested on the Matches where the inputs disagree, which is the only place a pick can change.
- **Not an abstention rule.** A Member who does not predict a Match scores 0; confidence therefore cannot
  save a point. It can pick the better of two sources, and it can feed the Rationale's "goes against the market favourite" flag.
- **Data.** Held. **Priority.** Build with C2; no separate ticket needed.

### C6. Margin of victory (round differential)

- **Measures.** Rating updates scaled by how decisively a map or series was won.
- **Evidence from the literature.** In tennis a margin-aware Elo variant predicted 65.8% against 63.7% for plain
  Elo ([Angelini, Candila, De Angelis, J. Quant. Anal. Sports 2021](https://econpapers.repec.org/RePEc:bpj:jqsprt:v:17:y:2021:i:3:p:203-219:n:2),
  secondary summary); football Elo scales the update by goal difference, `k = k0(1+δ)^λ`
  (Hvattum & Arntzen above); and Kovalchik's "Predicting outcomes for games of skill by redefining what it means to win"
  ([arXiv 1802.00527](https://arxiv.org/abs/1802.00527)) is the margin-of-victory Elo reference; I did not read its results. Goddard (2005), as cited in
  [Constantinou & Fenton](https://qmro.qmul.ac.uk/xmlui/bitstream/123456789/10776/5/CONSTANTINOU%20Determing%20the%20Level%20of%20Ability%202012%20Accepted.pdf),
  found no significant difference between goal-based and result-based prediction in football.
- **Archive check.** Round-share updates gave 66.1% against 66.0% for map win/loss (889 Matches, K tuned on the first half). Not distinguishable.
  CS2 maps end at 13 (or 16, overtime), so round differential is bounded and noisy; a 13–11 map says less than a 7–2 football goal margin.
- **Data.** Held (every played map has both scores). **Priority.** Skip: the ticket's lead hypothesis does not hold up.
  I would only revisit it if a map-level rating (C4) is built anyway, as one more parameter to try.

### C7. Side-specific (CT/T) map strength

- **Measures.** A team's rating on a map split by the side it plays first or by CT and T halves.
- **Evidence.** Descriptive only. Per-map CT/T balance differs (e.g. Anubis reported 55.5% T-side, Inferno 51.9%
  in pro play, [dust2.us](https://dust2.us/news/59104/anubis-is-historically-and-currently-the-most-t-sided-map-in-cs2),
  [bo3.gg](https://bo3.gg/news/cs2-austin-major-maps-pickrate-and-side-balance), secondary and inconsistent between sources). I found no
  published predictive model built on team-specific side ratings. The map-level side bias is already in any map rating,
  and a team-specific skew would be estimated on a few rounds per (team, map, side).
- **Data.** HLTV `half_scores` is `"(8:4; 4:8) (6:10)"` with no side labels, so the parsed records cannot say which side was CT.
  Liquipedia Stage revisions have `t1firstside` and the half-by-half scores for 62 Events, so sides are recoverable there.
- **Priority.** Skip; revisit only for an explanatory (Rationale) feature.

### C8. Map-pool depth and permaban patterns

- **Evidence.** Already built twice: `bo1_veto_sim` (319 Bo1s, +0.9 points) and `bo3_veto_sim` (1850 Matches, −2.9 points
  accuracy, log-loss gain is the same overconfidence repair). It entered no forward-selection step.
  Modelling who bans what has not made picks better, and its extra parameters (7 maps × ban/pick × team) are the sparsest in the data.
- **Priority.** Skip.

### C9. HLTV ranking points

- **Evidence.** HLTV describes its ranking as based on the past year with decay, recent form over 2 months and the last
  10 LAN events, and does not publish the formula ([Liquipedia, HLTV](https://liquipedia.net/counterstrike/HLTV), secondary, 2026-11-26). It is
  decided by results that VRS also uses. Archive check: when the two ranks disagree, VRS is right 75% of the time.
- **Data.** Only the rank number is on the match page; points are on `/ranking/teams`, which our HLTV collection rules do not cover.
- **Priority.** Skip. C1 gets everything the page offers.

### C10. Player Rating 3.0 and player form

- **Evidence.** HLTV's Rating 3.0 is Rating 2.1 adjusted for economy and "Round Swing" (win-probability change per kill),
  reweighted on 2025-10-29 ([dust2.us](https://www.dust2.us/news/64901/hltv-launches-rating-30), secondary; HLTV's own post was not retrieved).
  Archive check: the mean Rating 3.0 of the last 20 Matches picks 63.3% (889 Matches), below map Elo. The lab's `lineup_strength`
  (−0.003 log-loss) and `player_form` (0.000) add nothing. Players matter through results, which the team ratings already carry.
- **Priority.** Skip. One exception: a stand-in's Rating when a roster changes is only a log-loss-neutral covariate (`roster_stand_in` ≈ 0).

### C11. Differentiating from the League (not an accuracy criterion)

A League is won on Points relative to other Members. If every Member follows the favourite, a Bot User that also always follows it
ties them; separation comes from the Matches where picks differ. This is my reasoning, not a sourced finding; it is a
different objective from accuracy and belongs to the Bot User's strategy, not to a Candidate Signal. The `catch-up.md` simulations
are the place to test it.

## Recommendation: build next, at most five

| # | Candidate Signal | Needs | Data source |
|---:|---|---|---|
| 1 | **`rank_gap`** (C1): VRS and HLTV rank at the Match, mapped to a probability per format | Parser change that reads the two `ranking-badge` anchors from the raw HTML (rank, standings date, team id); an audit that the rank is pre-Match; a gap→probability fit per Event. | Held: `hltv/matches/*.html.gz` (1975 pages) |
| 2 | **`stacked_accuracy`** (C2 + C5): combiner over market, Valve rating, `rank_gap`, map-level rating; reported on the disagreement Matches | Existing walk-forward protocol; the report must show accuracy on the Matches where inputs disagree, with paired Event resampling | Held |
| 3 | **`market_close_html`** (C3): the odds widget on HLTV pages as a market line | Verify provider, time of the odds, and team alignment on a sample of pages; then a loader. If it is not pre-Match, drop it. Step zero: anchor accuracy on the 777 vs 1466 split | Held (pages); no new request |
| 4 | **`map_elo`** (C4): map-level win/loss Elo with recency weight, series probability by the existing Bo conversion | Replay of `per_map_rating`'s Elo without the anchor blend, as an input of #2 | Held (HLTV maps, Liquipedia maps) |
| 5 | **`rank_vs_market_rule`** (C5): which source to follow when the market and the ranks disagree | Needs #1 and #3; can be one section of #2 rather than a new signal | Held |

Not recommended: margin of victory, side-specific strength, veto/permaban, HLTV points, player form. Each has a measured or sourced reason above.

Two caveats for whoever builds these. First, none of the five has a live input port yet (`promotion::LIVE` holds only
`rating_calibration` and `learned_weights`); #1 would need the HLTV page, which the live sidecar does not fetch, so its first
live form is a daily Valve ranking read, not the page badge. Second, #48's Holdout (PGL Major Singapore 2026) is still pending: any of
the five must be added to the list as "added after list fixed", like `bo3_veto_sim`, and the Holdout still evaluates only the frozen combination.

## Appendix: archive checks

All offline, `python -I`, read-only, on `target/research-archive-t48-snapshot2/hltv/matches/`.

- **Set.** 1974 parsed Match records with a decided winner (`maps[*].winner`, wins `a` > `b`); Bo1/Bo3/Bo5 mixed.
- **Elo checks.** Sort by `started_at`. Each team starts at 0. A Match is scored only when both teams have at least 5 prior Matches,
  then the update follows. Series-level: one update per Match, `expected = 1/(1+10^(-Δ/400))`. Map-level: one update per played map.
  Series probability from the map probability by the Bo3 formula `p²(3−2p)` (Bo1: `p`). Round-share: update target is
  `team_a_score / (team_a_score + team_b_score)`, with the step doubled because that target spans a narrower range. K from
  {8, 16, 24, 32, 48, 64} minimising log-loss on the first half by date; scored on the second half (889 Matches).
- **Rank checks.** Regex over the raw page: `vrs-ranking"><span>VRS: </span>#(\d+)` and `hltv-ranking"><span>HLTV: </span>#(\d+)`,
  first two matches per page (team a, team b in page order, which is the order of `team_a`/`team_b`); the pick is the lower rank. 1968 of
  1975 pages have both and a decided winner.
  Badge dates (from the link) equal the Match date for 1947 and the following day for 21.
- **Rating 3.0 form.** Mean `overall_stats[*].players[*].rating` per team per Match; the pick is the team with the higher mean over its last 20 Matches (≥5 prior), second half only.
- **Not reproduced here.** The odds widget: names on the widget (e.g. "ex-CPH Wolves") differ from the parsed team names on most pages,
  so the alignment needs a team-id source, which the widget does not carry.
- **Why these are not results.** They are single-pass checks on a set that differs from the lab's, with no paired Event resampling, no Holdout
  and K chosen by me. They say where to look, not how much we gain.
