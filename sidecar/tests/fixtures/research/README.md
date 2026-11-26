# Research fixtures (ticket #36)

Real, unedited responses saved on `2026-11-26` as evidence for
`docs/signal-lab/map-veto-player-research.md`, and used by the Seam 2 parser
tests in `sidecar/tests/hltv_research_parser.rs` and the local stand-in server
collector tests in `sidecar/tests/hltv_research_collector.rs`.

No file contains any credential, cookie, API key, or bearer token.

| File | Source | Request | HTTP status | Captured (UTC) | Notes |
|---|---|---|---|---|---|
| `pandascore-csgo-match-1508646-403.json` | PandaScore | `GET https://api.pandascore.co/csgo/matches/1508646` (`Authorization` header omitted) | 403 | 2026-11-26 | `Access Denied` on the operator's current (Free) plan. Compare `tests/fixtures/pandascore/real-major-stage1-matches.json` (HTTP 200), which has slim `games` without map names, round scores, vetoes, or player stats. |
| `pandascore-csgo-game-219490-403.json` | PandaScore | `GET https://api.pandascore.co/csgo/games/219490` (`Authorization` header omitted) | 403 | 2026-11-26 | `Access Denied` on the current plan. |
| `pandascore-csgo-team-3249-stats-403.json` | PandaScore | `GET https://api.pandascore.co/csgo/teams/3249/stats` (`Authorization` header omitted) | 403 | 2026-11-26 | `Access Denied` on the current plan. |
| `liquipedia-cologne-playoffs-postmatch.json` | Liquipedia | `GET https://liquipedia.net/counterstrike/api.php?action=query&prop=revisions&rvprop=ids%7Ctimestamp%7Ccontent&rvslots=main&rvlimit=1&formatversion=2&format=json&titles=Intel+Extreme+Masters%2F2026%2FCologne%2FPlayoffs` | 200 | 2026-11-26 | Revision `3390154` (`2026-06-22T02:25:54Z`), gzip-decoded JSON. All 7 Playoffs Matches of IEM Cologne Major 2026 with map names, half-by-half + 4x OT scores, `t1firstside`, skipped maps, `hltv=` match IDs, and `stats=` mapstatsids. No map vetoes, no player stats. |
| `hltv-terms.html.gz` | HLTV | `GET https://www.hltv.org/terms` (iOS-app probe #1, `wreq` `SafariIos17_2`, UA `hltv/2.13.2 (org.hltv; build:1; iOS 17.2.1) Alamofire/4.9.1`) | 200 | 2026-11-26 | 162,162 B raw, gzipped unchanged. Terms of Service effective 27 Jan 2025 (Section 2.2 prohibits web scraping and data mining). |
| `hltv-match-2394898-vitality-vs-fut.html.gz` | HLTV | `GET https://www.hltv.org/matches/2394898/vitality-vs-fut-iem-cologne-major-2026` (iOS-app probe #3, same fixed client) | 200 | 2026-11-26 | 881,745 B raw, gzipped unchanged. Finished Bo3 (Vitality 2–1 FUT, 2026-06-11): 3 maps with half scores, 7-step Bo3 veto, series + per-map player stats (K-D, eK-eD, Swing, ADR, eADR, KAST, eKAST, Rating 3.0). |
| `hltv-match-2394774-heroic-vs-sharks.html.gz` | HLTV | `GET https://www.hltv.org/matches/2394774/heroic-vs-sharks-iem-cologne-major-2026-stage-1` (iOS-app probe #4, same fixed client) | 200 | 2026-11-26 | 495,932 B raw, gzipped unchanged. Finished Bo1 (HEROIC 10–13 Sharks, 2026-06-02): Nuke with half scores, 7-step Bo1 veto, player stats. |
| `hltv-mapstats-230770-403-challenge.html` | HLTV | `GET https://www.hltv.org/stats/matches/mapstatsid/230770/heroic-vs-sharks` (iOS-app probe #5, same fixed client) | 403 | 2026-11-26 | 5,664 B raw, `cf-mitigated: challenge` ("Just a moment..."). Triggered the non-negotiable stop rule; all HLTV probing stopped immediately after this 5th request. |

Probe #2 (`GET https://www.hltv.org/matches/2395002/match`) returned HTTP 301 (0-byte canonical slug redirect, no `cf-mitigated` header) and is recorded in the findings document.
