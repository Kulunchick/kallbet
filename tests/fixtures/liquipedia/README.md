# Liquipedia fixtures

Real responses from the Counter-Strike wiki's MediaWiki API (`https://liquipedia.net/counterstrike/api.php`), captured on 2026-11-21 with the User-Agent `KallBet/0.1 (https://kallbet.kulunchick.tech)`, at least 2 seconds apart. Never hand-write or hand-edit wikitext here: when Liquipedia changes, capture again and fix the parser. Content is CC-BY-SA 3.0 (Liquipedia).

Raw API answers (JSON, unmodified):

| File | Request | Holds |
|---|---|---|
| `api-query-cologne-pages.json` | `action=query&prop=revisions&rvprop=content|timestamp&rvslots=main&formatversion=2&titles=<root>|<Stage 1>|<Playoffs>` | IEM Cologne Major 2026: root page (infobox, a showmatch), Stage 1 (Swiss, finished, Bo1 and Bo3), Playoffs (single elimination, Bo3, Bo5 final). Finished event. |
| `api-allpages-cologne.json` | `action=query&list=allpages&apprefix=Intel Extreme Masters/2026/Cologne/` | The Stage subpages of that Major. |
| `api-search-iem-cologne.json` | `action=query&list=search&srsearch=IEM Cologne Major 2026&srnamespace=0&srprop=` | Search hits: the Event page first, then players and teams. |
| `api-section0-search-candidates.json` | `action=query&prop=revisions&rvslots=main&rvsection=0&titles=<Event>|Karrigan|Aurora Gaming` | Section 0 of an Event page (its infobox) next to two pages that have none. |
| `api-teamnames-cologne.json` | `action=expandtemplates&prop=wikitext&text=<key>@@{{Team|<key>}}` per line, 24 team keys of that Major plus `navi` | What the wiki renders for a team template key. |
| `api-missing-page.json` | `action=query&prop=revisions&titles=Nonexistent Event zzqx/2026` | A page that does not exist (`"missing": true`). |

Page wikitext (the `content` of `action=query` answers for the page, saved as text; `*.wikitext`):

| File | Page | State at capture |
|---|---|---|
| `singapore-stage1-upcoming.wikitext` | `PGL/2026/Singapore/Stage 1` | upcoming Major Stage: 16 teams, Round 1 paired, rounds 2 to 5 are empty slots |
| `ligaen-season33.wikitext` | `Dust2.dk/Ligaen/Season 33` | POWER Ligaen finals (HLTV 9496): not Swiss, `Bracket/4U2L1D` double elimination, Grand Final unplayed |
| `cct-europe-series14.wikitext` | `CCT/2026/Europe/Series 14` | ongoing Event with a Swiss group stage and a `Bracket/16` Playoffs on one page; forfeits (`score=W` / `score=FF`), TBD slots |

Notes that shaped the parser:

- Teams are template keys (`m80`), never names. `{{Team|key}}` renders the name; `{{TeamName|key}}` is unusable (answers `100x50px` for teams with a logo).
- No Match has a winner or a Swiss record in its wikitext. Winners come from the maps' round totals (overtime in `o<N>t1t`...), forfeits from `score=W`; records from the order of the High / Mid / Low Matchlists in each Round.
- Not captured because no such page was found: a cancelled or postponed Match (Liquipedia has no per-Match status field in anything we saw), a mid-Stage Swiss page with a live Match.
- The Cologne Major's Stage 2 and Stage 3 pages exist (`api-allpages-cologne.json`) but are not saved: Stage 1 stands for all Swiss Stages.
