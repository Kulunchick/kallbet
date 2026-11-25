# Retrospective benchmark, 2026-11-25 (ticket #31, anonymized slice rerun the same evening)

`report.md` (and the same data in `report.json`) is the benchmark of spec #21:
the sidecar's live pipeline replayed over every completed CS2 Major Stage we
hold point-in-time data for, plus the 24 finished Matches of PGL Major
Singapore 2026 Stage 1, with a leakage guard that aborts on any datum observed
at or after its cutoff. How the runner works is in `docs/agents/codebase.md`,
"Retrospective backtest (ticket #31)"; the data and its provenance are in
`sidecar/tests/fixtures/backtest/README.md`.

## What ran

- **Non-LLM track** (`VrsOnly`, `MarketOnly`, `HybridAnchored` without an LLM,
  and the `PickemSolver` with 25,000 simulations): all 19 saved Stages of six
  Majors, 488 scored Matches. Predicted where data allowed: 415 Matches with a
  VRS rating (all but Copenhagen 2024, which predates Valve's global
  standings), 112 with both a market line and a rating (Cologne 2026 and
  Singapore). Pick'em: 18 final Stages, 71 Points in total; Singapore Stage 1
  pending (Rounds 4 and 5 are played on 2026-11-26/27).
- **LLM track**, on 2026-11-25 from 20:25 to 21:04 UTC, budget as in the spec:
  the 24 finished Singapore Stage 1 Matches with real names (after every
  model's training cutoff) and an anonymized slice of 6 Matches of IEM Cologne
  Major 2026 Stage 1 (`BlindAnonymizer`). 30 of 30 planned Matches answered by
  both LLM profiles; 120 CLI runs, no quota error, no tool and no web access
  (`Mode::Backtest`).
- **Anonymized slice rerun**, on 2026-11-25 from 22:43 to 22:55 UTC, after the
  spec-21 review taught the roster parser the older `{{TeamCard}}` template:
  the six Cologne dossiers now carry both teams' rosters (masked as
  `PLAYER_01..10`), so the recorded answers of the first run no longer matched
  them and those six Matches were asked again, with the same seating and
  models: 24 CLI runs (6 x single agent + three seats), no quota error. The
  run was restarted once after its first three Claude answers to add agy's
  `download_url(*)` rule to the no-tools settings (below); those three answers
  were kept, not asked again. The six stale answers were removed from
  `llm-answers.json`, which holds the 120 answers the report uses.
  - Single agent: Claude (`claude -p`, model `sonnet`).
  - 3-agent debate (mode `always`): Market Advocate Claude, Devil's Advocate
    Antigravity (`agy`, `gemini-3.8-flash-medium`, with Claude as its
    fallback, never used), Chief Arbiter Claude.
- Probes before the run (one tiny call each): Claude answered; Codex answered
  "You've hit your usage limit"; Antigravity answered "Individual quota
  reached ... Resets in 18m56s", and answered after the reset at 20:25 UTC.
  Before the rerun: Claude and Antigravity answered (22:42 UTC).
- **Tool lockdown, a caveat on the first run.** The review found that the agy
  no-tools settings denied six permission rules but not `download_url(*)`,
  which `agy.exe` also knows. The 24 Singapore Devil's Advocate answers of the
  first run (Antigravity) were made under those settings; whether any of those
  runs fetched a page cannot be told from the saved answers. The rerun
  of the anonymized slice ran with `download_url(*)` denied. Claude's runs were
  never affected (`--tools ""`).

## What did not run

- **Codex**: out of quota (usage limit) during the whole session, so the spec's
  seating (Codex as Devil's Advocate) was not run and there are no Codex
  numbers. To produce them, once Codex answers again, from the repository root:

  ```sh
  # llm-codex.json: docs/backtest/llm.example.json with each "binary" set where
  # the CLI is not on PATH; it seats Claude / Codex / Antigravity as the spec does.
  cargo run --release -p kallbet-ai -- backtest --out target/backtest-codex \
    --llm llm-codex.json --answers docs/backtest/llm-answers-codex.json
  ```

  (A new answers file: the recorded one would answer every Match without a call,
  since answers are reused per profile and dossier, not per provider.)
- **Singapore Stage 1 Pick'em Points**: pending; the Stage was not finished.
  Rerun after a fresh PandaScore list (`collect_backtest.py pandascore 22378`).
- No news in any dossier (no dated news archive). Rosters for Singapore,
  Cologne 2026 and Shanghai 2024 (the Rosters column of the report); the
  other Majors' Matches have no saved Event revision.
- No anonymized comparison of the Singapore Matches (`--blind-direct`): not in
  the budget.

## Reading it

- Over the 112 Matches with both inputs, the market (62.5%, log-loss 0.633,
  Brier 0.221) beats VRS (59.8%, 0.734, 0.258); the 80/20 anchor follows the
  market (62.5%, 0.634, 0.222). Across all 415 rated Matches: VRS 61.4%,
  market-or-VRS 62.2%. Bo3 is more predictable than Bo1 for every profile.
- Singapore Stage 1 so far was a Stage of upsets: every source's favourite won
  fewer than half of the 19 Matches all three priced (Pinnacle 36.8%,
  Polymarket 42.1%, Kalshi 47.4%), and VRS (54.2%) did best on its 24 Matches.
- The LLMs changed the anchor on 3 of 30 Matches, all Singapore Matches with a
  stand-in in the dossier's Liquipedia revision (the evidence cited), by 1.5 to
  3.5 points; one changed the pick (9z vs Dendele, Bo3) and was wrong. On these
  numbers neither the single agent nor the debate improves on the anchor
  (Singapore: 11 correct each against the anchor's 12, log-loss 0.744 / 0.745
  against 0.747). 24 and 6 Matches are far too few to tell the two apart.
- On the anonymized slice, now with rosters, neither LLM profile moved the
  anchor on any of the six Matches (3 correct each, as the anchor).
- The report now also lists every Singapore Match with each profile's pick,
  and the solver's Stage 1 Pick'em with each picked team's record and state.
- The archive's SHA-256 in the report's first line changed with this rerun
  because the archive's own README (part of the hashed archive) was corrected;
  no data file changed.

## Reproducing it

From the repository root, without any provider or network:

```sh
cargo run --release -p kallbet-ai -- backtest --out target/backtest \
  --answers docs/backtest/llm-answers.json
```

`llm-answers.json` holds every LLM answer of the run (the audit's
`debate_turns`, status `ok`); with it, every LLM Match is answered from the
record and the report is identical to this one (the archive's SHA-256 is in its
first line). It was checked: the live run's tables and a rerun from the record
are the same number for number.

OddsPapi requests: the account counter was 6 of 250 before this ticket's second
implementer started and 9 after (three fixture lists; the history endpoint and
`/v4/account` are free). The spec-21 review made no OddsPapi request: the counter read 9 of 250
before and after it (2026-11-25, through the free `/v4/account`).
