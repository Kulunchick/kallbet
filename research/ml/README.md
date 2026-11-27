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
uv run pytest -q                # leak tests, HLTV point-in-time checks, walk-forward rules, parity
uv run python -m kbml.report    # phase 2: primary config, all models, ablation → REPORT.md (~20 min)
uv run python -m kbml.pickem_check --runs 200000 --jobs 3   # 28-stage Swiss Pick'em replay → PICKEM.md
uv run python -m kbml.pickem_forecast --device numpy        # Singapore Stage 2 forecast → forecasts/ (CPU)
```

GPU (optional): the `gpu` dependency group swaps in the CUDA 13.2 build of the same torch. Keep it in
its own environment so the default `.venv` stays on CPU torch:

```
$env:UV_PROJECT_ENVIRONMENT=".venv-gpu"; uv sync --no-default-groups --group gpu
uv run --no-default-groups --group gpu python -m kbml.pickem_forecast --runs 50000000 --random-starts 8
```

`kbml/swiss_gpu.py` vectorises the Swiss simulation over runs (1M runs per batch fits a 6 GB card);
`tests/test_swiss_gpu.py` checks it returns the numpy simulator's outcomes exactly from the same draws.

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

## HLTV pages (phase 2: `kbml/hltv_html.py`, `kbml/hltv_data.py`, `kbml/features_hltv.py`)

The raw match pages are parsed once (cached `hltv_pages.pkl`, ~1 min on all cores) into `pre`
(known before the Match: HLTV rank badge with its ranking date, lineups with stand-in and IGL/AWP
markers, past-3-months lists team and core with streaks, head-to-head summary and map listing,
stage line and coach substitutions of the note) and `post` (CT/T side of each regulation half,
per-player CT-only and T-only stats). Never read: the VRS badge (it holds the Match's own result),
odds widgets, fetch-relative "time ago" texts, map pick badges and the Match's own veto
(post-veto), POTM, VODs, comments, free `**` admin notes.

`pre` of a Match feeds only that Match (oriented to PandaScore team A; rows not strictly earlier are
dropped as a guard: none were). HLTV rank follows `rank_history.rs`: a ranking is known from the end
of its UTC day, read strictly before the cutoff, at most 35 days old. Everything a Match produced
(JSON stats overall and per map, maps, veto, `post`, its lineups) is an HLTV record released by the
timeline only once the Match's known finish is before the cutoff. `tests/test_hltv.py` checks each
field (synthetic page with leaky decoys, and on the archive: ranks, past-matches, streak badge,
head-to-head, lineups vs stats, half scores vs map scores, and that scrambling one Match's record or
page moves only what it may).

Groups: `rank`, `opp_form`, `past_form`, `hltv_h2h`, `players`, `map_pool` (pre-veto expected map
edge from earlier vetoes), `sides`, `standin`, `roster`, `stakes` (format × stage × anchor). Sets:
`hltv`, `full` (= `baseline` + `hltv`).

## Models and evaluation (`kbml/models.py`, `kbml/evaluate.py`)

`logreg`, `lgbm`, `mlp` (torch) share `fit/predict`; each trains on both team orientations and
predicts p = (f(x) + 1 − f(swap x)) / 2. Walk-forward by Event: an Event is predicted by a model fitted
only on complete Events that ended before it began (fewer than 150 training Matches → the anchor).
Hyperparameters come from an inner walk-forward on the training Events only. Metrics on the same
Matches as the anchor: accuracy (primary here), log-loss, Brier; differences with the lab's
Event-level paired bootstrap (its RNG ported bit for bit), 2000 resamples, 90%, seed 33.
