# Context Enricher Fixtures Provenance

Unmodified real responses captured on **2026-11-25** for Seam 2 parser and Seam 1 integration tests (ticket #26).

## `liquipedia/` (MediaWiki Historical Revisions)

Fetched with `User-Agent: KallBet/0.1 (https://kallbet.kulunchick.tech)` and `Accept-Encoding: gzip`, spaced at least 2 seconds apart. Content is licensed under CC-BY-SA 3.0 (<https://creativecommons.org/licenses/by-sa/3.0/>).

| File | Request | Date fetched | Notes |
| --- | --- | --- | --- |
| `pgl-singapore-2026-rev-before-start.json` | `GET https://liquipedia.net/counterstrike/api.php?action=query&prop=revisions&rvprop=ids\|timestamp\|content&rvslots=main&rvlimit=1&rvstart=2026-11-24T00:00:00Z&rvdir=older&formatversion=2&format=json&titles=PGL/2026/Singapore` | 2026-11-25 | Root Major page as of `2026-11-23T19:49:30Z` (`revid: 3473596`): 32 teams in `{{TeamParticipants}}`, coaches (`csub`), active stand-ins (`role=sub\|played=true`), benched starters (`played=false`), unused subs (`status=sub\|played=false`), and tournament rename notes (`1win Team` -> `The One`, `BC.Game Esports` -> `GOSH`). |
| `pgl-singapore-stage1-before-start.json` | `GET https://liquipedia.net/counterstrike/api.php?action=query&prop=revisions&rvprop=ids\|timestamp\|content&rvslots=main&rvlimit=1&rvstart=2026-11-24T00:00:00Z&rvdir=older&formatversion=2&format=json&titles=PGL/2026/Singapore/Stage%201` | 2026-11-25 | Stage 1 subpage as of `2026-11-16T14:38:40Z` (`revid: 3471273`), containing brackets/matchlists and no `{{TeamParticipants}}`. |

## `rss/` (CS2 News RSS 2.0 Feeds)

Fetched on 2026-11-25 with `curl.exe -4 -sS -L --compressed -A "KallBet/0.1 (https://kallbet.kulunchick.tech)"`.

| File | Request | Date fetched | Notes |
| --- | --- | --- | --- |
| `hltv-news.xml` | `GET https://www.hltv.org/rss/news` | 2026-11-25 | RSS 2.0, RFC 2822 GMT dates, HTML entities (`&quot;`), live Major headlines. |
| `dust2us-news.xml` | `GET https://www.dust2.us/rss` | 2026-11-25 | RSS 2.0, RFC 2822 `-0500` dates, Major and NA roster headlines. |
| `dust2dk-news.xml` | `GET https://www.dust2.dk/rss` | 2026-11-25 | RSS 2.0, RFC 2822 `+0100` dates, Danish UTF-8 headlines. |
| `steam-cs2-news.xml` | `GET https://store.steampowered.com/feeds/news/app/730/` | 2026-11-25 | RSS 2.0, RFC 2822 `+0000` dates, CDATA links, HTML-escaped patch notes. |
