# kbml: learned Match-winner models against the Signal Lab anchor

A Python prototype to test whether a learned model picks more CS2 Match winners than the Signal
Lab's anchor (0.80 market + 0.20 Valve rating, logit space), with the honesty rules of the Rust
lab (`signal-lab/4`, `sidecar/src/adapters/lab/`) but without a release build. KallBet is
points-only (1 Point per correctly picked winner); market prices are only a public estimate of each
team's chance. Nothing here calls a network or a provider: it reads one archive directory.

Not part of the cargo or npm builds. No Rust or web code is touched.

## Commands (from `research/ml/`)

```
uv python install 3.12          # once; the system Python 3.15 lacks wheels
uv sync                         # numpy, pandas, pyarrow, scikit-learn, lightgbm, torch (CPU), pytest

uv run python -m kbml.table     # (re)build the Match table cache from the archive (~35 s)
uv run python -m kbml.parity    # Python anchor vs docs/lab/research/report.json: must print PARITY OK
uv run python -m kbml.run --model logreg,lgbm,mlp --features baseline   # ~1.5 min (mlp is most of it)
uv run pytest -q                # leak tests, walk-forward rules, bootstrap port, parity
```

Options of `kbml.run`: `--model` (comma list of `logreg`, `lgbm`, `mlp`), `--features` (comma list
of groups or sets, below), `--archive DIR` (default the research snapshot; env `KBML_ARCHIVE`),
`--rebuild` (ignore the cache), `--no-train-extra` (fit only on in-scope rows), `--min-train N`
(default 150), `--seed` (33), `--resamples` (2000), `--save` (per-Match predictions and summary
under the cache's `runs/`).

## Data

- Archive (read only): `D:/Projects/KallBet/target/research-archive-t48-snapshot2`.
- Cache: `D:/Projects/KallBet/target/ml-cache/<archive dir name>/` (`matches.parquet`,
  `maps.parquet`, `meta.json`), never in the repository. Env `KBML_CACHE` moves it.

`kbml/archive.py` reads the archive as `backtest/archive.rs` does (file kind from content, name
order). `kbml/anchor.py` ports the Valve rating (`vrs.rs`, `ReplaySignal`), the market chain
(`ReplayOdds`: OddsPapi Pinnacle → Polymarket → Kalshi, last de-vigged price strictly before the
cutoff, same two teams within 6 h of the Scheduled Start) and `debate::base`. `kbml/table.py`
builds one row per Match: Event (tier, `is_qualifier`: name contains RMR or Qualifier, `in_scope`),
Stage, teams and keys, format, scheduled/actual start, cutoff (min(Scheduled Start − 15 min, actual
start)), known finish, winner, map score, market probability and source, Valve rating and points,
anchor, linked HLTV match JSON (same team pair, nearest start within ±6 h) and the Liquipedia roster
revision current at the cutoff. A second table holds each linked HLTV record's played maps.

**Holdout** (PGL Singapore 2026 after Stage 1, and any Stage beginning at or after 2026-11-25
23:46:46 UTC or whose Event ended then or is incomplete): those Matches are dropped while the table
is built: no winner, market or rating of theirs is computed or cached.

**Scope**: `in_scope` = tier S/A, not a qualifier, Event begun on or after 2024-07-29 (the research
archive's first Event, scored by `signal-lab/4`). Evaluation is on in-scope Matches with an anchor;
other rows (tier B, older Events from a wider archive) may train and always feed history; a
qualifier never trains.

## Point in time

`kbml/history.py`: a result is visible at a cutoff only if its known finish is set and strictly
before it. `Timeline.advance_to(cutoff)` releases results (and played maps) in finish order and
never goes backwards; feature groups learn only from what it releases and describe a Match from
its `MatchInfo`, which refuses result columns. `tests/test_point_in_time.py` injects and flips later
results and shows earlier features unchanged.

## Features (`kbml/features.py`)

One class per group, registered with `@register`, each column declared `anti` (flips sign when the
teams swap) or `sym`. Groups: `anchor` (market, Valve, blend logits and flags), `elo` (series Elo),
`map_elo` (HLTV map Elo, overall and per map pool), `form` (last 10 results, round difference of last
15 maps), `h2h`, `rest` (days since last result, results in 7 days), `lan` (venue and venue form),
`format`. Sets: `baseline` (all), `anchor`, `no_anchor`.

## Models and evaluation (`kbml/models.py`, `kbml/evaluate.py`)

`logreg`, `lgbm`, `mlp` (torch) share `fit/predict`; each trains on both team orientations and
predicts p = (f(x) + 1 − f(swap x)) / 2. Walk-forward by Event: an Event is predicted by a model fitted
only on complete Events that ended before it began (fewer than 150 training Matches → the anchor).
Hyperparameters come from an inner walk-forward on the training Events only. Metrics on the same
Matches as the anchor: accuracy (primary here), log-loss, Brier; differences with the lab's
Event-level paired bootstrap (its RNG ported bit for bit), 2000 resamples, 90%, seed 33.
