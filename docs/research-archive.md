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
