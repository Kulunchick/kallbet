# Replay archive of completed CS2 Majors (ticket #31)

`kallbet-ai backtest` replays the sidecar's live pipeline over these files, and
the tests in `sidecar/tests/backtest*.rs` read them. Every file is a real
response, saved gzip-compressed. `sources.tsv` lists, for every file, the
request it answers (credentials are never part of it: the OddsPapi `apiKey`
query parameter and PandaScore's `Authorization` header are left out).

What a file is, the loader (`src/adapters/backtest/archive.rs`) reads from its
content. Only two kinds are named by their file name: `lp-<tournament>` is the
Liquipedia Stage revision of that PandaScore tournament, and
`polymarket-history-<event slug>` the price history of that Polymarket event.

| Files | Source | What the replay uses it for |
|---|---|---|
| `panda-<tournament>` (19) | PandaScore `GET /tournaments/{id}/matches?per_page=100` | Ground truth: teams, format (`number_of_games`), `scheduled_at`, `begin_at`, `status`, `winner_id`, `forfeit`. The Stages of six Majors: Copenhagen 2024, Shanghai 2024, Austin 2025, Budapest 2025, Cologne 2026, and PGL Major Singapore 2026 Stage 1 (its 24 Matches finished by 2026-11-25 18:28 UTC; Rounds 4 and 5 not played yet). |
| `lp-<tournament>` (19) | Liquipedia `action=query&prop=revisions&rvdir=older&rvstart=<cutoff − 1 s>` of the Stage page (Copenhagen's Playoffs: the Event page, which holds its bracket) | The pre-Stage Pick'em view: the Stage's teams, Round 1 pairings (or the Playoffs quarterfinals), seeds. `rvstart` is one second before the Stage cutoff (earliest Scheduled Start or actual start of its Matches). |
| `roster-<match>` (26), `event-iem-cologne-2026-before-stage1` | Liquipedia, same query on the Event page before a Match's cutoff | Roster facts (`{{TeamParticipants}}`) for the dossier. Only the Singapore revisions use that template; the 2024 Shanghai and the June 2026 Cologne pages list teams with `{{TeamCard}}`, which #26's parser does not read, so those dossiers hold no roster. |
| `standings_global_<date>.md` (7) | Valve, `raw.githubusercontent.com/ValveSoftware/counter-strike_regional_standings/f8c55f250fdef796d708aa32814587932154fa04/live/<year>/…` (2026-11-02: copy of `tests/fixtures/vrs/`) | VRS ratings. Global standings start in August 2024, so Copenhagen 2024 has none. |
| `cologne-fixtures`, `oddspapi-fixtures-*` (4) | OddsPapi `GET /v4/fixtures?tournamentId=…&statusId=2&from=…&to=…` (Cologne 46101, Singapore 400014) | Links a Match to its OddsPapi fixture (`oddspapi::match_settled_fixture`). `cologne-fixtures` stops at 100 fixtures; the Stage 1 and Stage 2 lists confirm that the missing Cologne lines are not on OddsPapi (Sharks are listed under another name, `Dendele CS`). |
| `id…`, `od…` (120) | OddsPapi `GET /v4/historical-odds?fixtureId=…&bookmakers=pinnacle%2B30` (free) | Pinnacle series-winner tick history. **Trimmed**: each file keeps only market `171` (the series winner, the only market the parser reads) of every bookmaker; the other markets of the response were removed by `collect_backtest.py trim-odds`. Nothing else was changed. OddsPapi has no history for three Singapore fixtures (404: the Matches of BC.Game, PandaScore's GOSH). |
| `polymarket-events-pgl-singapore-2026-closed-0` | Polymarket `GET https://gamma-api.polymarket.com/events?tag_slug=pgl-major-singapore-2026&closed=true&limit=100&offset=0` | The 24 closed Singapore Stage 1 Match events (plus 2 props), with each moneyline market's outcomes, `gameStartTime` and `clobTokenIds`. |
| `polymarket-history-<slug>` (24) | Polymarket `GET https://clob.polymarket.com/prices-history?market=<first clobTokenIds entry>&startTs=<start − 48 h>&endTs=<start + 2 h>&fidelity=5` | `{history: [{t, p}]}` for the first outcome. Holds in-play points; the replay reads only points before the cutoff. |
| `kalshi-markets-KXCS2GAME-settled-1795392000` | Kalshi `GET https://api.elections.kalshi.com/trade-api/v2/markets?series_ticker=KXCS2GAME&status=settled&limit=200&min_close_ts=1795392000&max_close_ts=1795651200` | 200 settled CS2 Match markets, of which the 48 Singapore Stage 1 ones (one per team) are used. |
| `kalshi-candles-<ticker>` (48) | Kalshi `GET …/series/KXCS2GAME/markets/<ticker>/candlesticks?start_ts=<start − 6 h>&end_ts=<start + 2 h>&period_interval=1` | Minute candles; the replay takes the mid of `yes_bid`/`yes_ask` closes of the last candle that ended before the cutoff, for both teams, normalised. |

Fetched on 2026-11-25. The first implementer of #31 collected the PandaScore,
OddsPapi, Valve and most Liquipedia files (commit `ebaf817`, by the scratch
script `collect-local.py`, since folded into `sidecar/scripts/collect_backtest.py`);
its second implementer refetched eight Stage revisions whose saved revision was
not strictly before the Stage cutoff (they had been taken against the actual
start, not the earlier Scheduled Start), added the Cologne Event revision, the
three OddsPapi fixture lists (three billable requests; the account counter went
from 6 to 9 of 250), the missing histories, and the Polymarket and Kalshi files.
Polymarket and Kalshi do not answer from the workstation: those files were
fetched with `curl` on a remote host, unchanged.

To collect again: `python -I sidecar/scripts/collect_backtest.py --help`.
