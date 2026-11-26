# Historical CS2 research archive

From the repository root, collect completed tier-S and tier-A tournament stages and extend the archive through a completed UTC day.

```sh
python sidecar/scripts/collect_research.py --until 2026-11-25
cargo run -p kallbet-ai -- backtest --archive target/research-archive --out target/research-report
```

Repeat the collection command with the same date to resume. An exit status of 2 indicates partial coverage. The daily UTC caps default to 100 PandaScore requests, 60 Valve requests (GitHub) and 100 Liquipedia requests. Use `--panda-cap`, `--valve-cap`, and `--liquipedia-cap` to change them. Each attempt counts before transport, including failures. Cached requests consume zero. The collector prints spending against each cap. Supply `PANDASCORE_TOKEN` in the environment or use `--env PATH` to read a local env file.

The default directory is `target/research-archive/`. Root payloads use gzip. The PandaScore stage responses keep both schedule fields, results and game records as supplied. The dated tournament catalogs under `catalog/` retain tier, type, region and dates, including null fields when the source has no value. Valve's commit and tree under `catalog/` identify the global standings files. `roster-<serie_id>-<UTC date>.json.gz` holds the last available Liquipedia revision before that day began. The collector reports ambiguous page matches. Supply a JSON object such as `{"12345":"Organizer/2026/Event"}` with `--titles PATH` after verifying the Event's title.

`sources.tsv` gives the relative path, public request URL, retrieval time in UTC and SHA-256 of each uncompressed source payload. Keep the SQLite collector ledger when resuming. Replay reads the root payloads, writes their digest to its report, and gives rating, market and roster coverage for the scored Matches. The test fixtures have a separate directory.

The live sidecar keeps dated RSS items under `rss:item` in its audit database. `Audit::news_before(cutoff)` returns items whose publication and first fetch both predate the cutoff. Repeated observations of the same content retain the original fetch time.

## Played maps from Liquipedia

From the repository root, run the map collection against a writable research archive that already holds the PandaScore Stage responses.

```sh
python sidecar/scripts/collect_research.py --maps-only --archive target/research-archive --until 2026-11-25 --liquipedia-cap 100
cargo run -p kallbet-ai -- lab --archive target/research-archive --signal per_map_rating --signal bo1_veto_sim --out target/map-lab
```

For Stages beyond those with verified paths in the small saved archive, add `--stage-titles PATH` with a JSON object keyed by PandaScore **tournament** ID, such as `{"20710":"Intel Extreme Masters/2026/Cologne/Playoffs"}`. The collector prints IDs without verified paths and returns status 2 for partial coverage. It uses the shared Liquipedia request budget, custom User-Agent, gzip and 2.1-second minimum spacing. Each completed Stage uses one MediaWiki revision request and saves the response as `maps-<tournament_id>.json.gz` with CC-BY-SA 3.0 source attribution in `sources.tsv`. Repeat the command with the same verified titles to resume.

The replay links played maps by Stage and normalized team pair, checks the series score, and requires a recorded PandaScore Match finish. Ambiguous pairs and records without confirmed scores or finish times are skipped. Completed map outcomes are dated by the Match finish, so ratings use only Matches that finished strictly before the evaluation cutoff. The rating averages across equally likely sets of historical maps for the two teams and starts an unrated team-map combination at neutral Elo. The Lab reports `bo1_veto_sim` as `not testable` because the authorized sources lack ordered bans and picks.
