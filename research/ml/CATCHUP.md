# Catch-up in Leagues: a Bellman DP policy instead of the fixed 40-60% window

FORECASTER.md found that with `lgbm-ext` probabilities the plain bot wins 12-member Leagues more
often than the anchor bot (+0.10), while the static catch-up rule of `docs/lab/research/catch-up.md`
(reverse a 50-60% favourite whenever trailing) gets worse (−0.05 to −0.11): calibrated probabilities
put about twice as many Matches in the window. This page tests a policy that decides each flip by
backward induction on the chance of finishing first.

## What a live bot can know at decision time (from the KallBet code)

- **Standings.** `GET /api/leagues/{id}/leaderboard` (`src/api/leaderboard.rs`) ranks the current
  Members by Match Points over decided Matches of the League's effective view. Points come from
  `scoring::judge`: one Point per correct Prediction times the Stage's points (`league_stage_points`,
  default 1); Missing, Void and Pending earn nothing. Equal Points share a place, including first
  (`scoring::leaderboard`: 1, 1, 3). So a tie for most Points is a first place.
- **Others' Predictions.** A Bot User sees no revealed Prediction of a Match before its Hard Lock
  (`api::matches`, `blind = viewer.is_bot && !outcome.is_hard_locked()`; ADR-0003 amendment), and it
  may change its own until Hard Lock. So when the bot decides a Match it does not know how anyone
  picked *that* Match. It can see others' picks of earlier, Hard-Locked Matches (revealed), but the
  sidecar does not read them today, and the lab simulation (`leagues.rs`) gives the bot only the
  published standings. The policy here uses only the standings.
- **Matches left.** The League's Match list (`api::matches`) shows the Event's known Matches; later
  Swiss rounds and Playoff slots appear once teams are known, so the exact count is not listed early
  but follows from the format (a 16-team Swiss Stage has 33 Matches, an 8-team bracket 7). The replay
  gives the bot the true number of the Event's remaining Matches (the format is known); results not
  yet published count as remaining.
- **Members.** The League's member list (`leagues::members`), so the League size is known.
- **The live sidecar** (`sidecar/src/scheduler.rs`, `ports::decision_policy`) has no catch-up rule
  and does not read the leaderboard today: catch-up exists only in the lab (`adapters/lab/leagues.rs`).

## The DP policy (`kbml/catchup_dp.py`)

State before a Match: r = Matches whose result is not yet published (this one included), d = the
bot's published Points minus the leader's (clipped to ±10), c0 = opponents tied for the lead, c1 =
opponents one Point behind the lead (the rest are treated as two behind). Value V_r = P(finish with
at least the leader's Points) = P(first, ties shared). Opponent model as the scenario's opponents:
each opponent independently takes the consensus side (the anchor favourite, or the market favourite)
and, for the favourite follower, reverses it with the noise probability; no consensus (missing
input) means every opponent misses. Our probability p decides the outcome. Future Matches are drawn
from the empirical distribution of P(consensus side wins) under our p over the scored Matches of
Events that ended before this Event began (walk-forward; a uniform 50-90% grid while fewer than 100
exist). V_0 = [d ≥ 0]; V_r is the expectation over the next Match type of the better of the two
picks (backward recursion, tables per League size, opponent model and Event). At a real Match the
bot computes both picks' values from its own p for that Match and V_{r−1}, and takes the underdog
only if that is strictly higher (ties: its favourite).

## Primary configuration (declared before its evaluation)

Committed before the DP or the re-tuned window were run on the archive. FORECASTER.md's catch-up
table (plain and static 40-60 bots, anchor and `lgbm-ext`) had been seen.

- **Policy:** the DP above, built with the true opponent model of the scenario (favourite follower,
  10% noise) and the League's size.
- **Probabilities:** `lgbm-ext` (walk-forward, phase 3 protocol, the 2087 scored Matches).
- **Comparison:** against the plain `lgbm-ext` bot (always its favourite), 12 members, opponents
  favourite followers with 10% noise.
- **Data and trials:** the 57 in-scope Events, 2000 paired trials, seed 33, the draws of
  `kbml/catchup.py` (both bots face identical opponents).
- **Rule:** DP "wins more 12-member Leagues than plain" only if the 90% interval of the paired
  difference in the first-place indicator (DP − plain), Hoeffding over the 2000 trials as in
  catch-up.md, lies wholly above 0.

Everything else (anchor probabilities, 4 members, 25% noise, market followers, the static and
re-tuned windows, expected place, Event-clustered bootstrap intervals, a DP built with the wrong
noise) is exploratory.
