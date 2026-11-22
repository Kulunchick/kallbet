# PandaScore fixtures

Three kinds of file. The `real-*` files are unedited responses of the live API; nothing in them was trimmed or changed (checked: no token, e-mail or account detail). The two others say in their own keys (`_source`, `_constructed`) what they are.

## Real captures (`real-*.json`)

Captured on 2026-11-22 with the owner's token (header `Authorization: Bearer <token>`), on the owner's plan, base `https://api.pandascore.co`. All requests answered 200.

| File | Request | What it holds |
|---|---|---|
| `real-series-search.json` | `GET /csgo/series?search[name]=Major&sort=-begin_at&page[size]=20` | 7 series named "...Major...", 2017 to 2026 (Cologne 2026, Shanghai 2024, Rio 2022, Berlin 2019, ...). Each embeds its `league` and its `tournaments` (Stage names and `has_bracket`: "Opening Stage"/"Elimination Stage", "Challengers Stage"/"Legends Stage", "Champions Stage", "Group Stage"). |
| `real-series-running.json` | `GET /csgo/series/running?page[size]=5` | 5 running series, mostly small events whose `name` is empty and `full_name` is only "Season 14 2026" (the league tells them apart); Elisa Open Suomi (11156) and European Pro League (11177) with `has_bracket: true` group stages. |
| `real-series-upcoming.json` | `GET /csgo/series/upcoming?page[size]=5` | 3 upcoming series, including PGL Major Singapore 2026 (11176). |
| `real-series-range.json` | `GET /csgo/series?range[begin_at]=2026-05-01,2026-07-01&page[size]=5` | 5 series that began in the range (plain dates). |
| `real-major-serie-filter.json` | `GET /csgo/series?filter[id]=10488` | IEM Cologne Major 2026 (finished): `name` "Cologne Major", `full_name` "Cologne Major 2026", league "IEM". |
| `real-major-tournaments.json` | `GET /series/10488/tournaments` | Its four tournaments: Stage 1, 2, 3 (`has_bracket: false`) and Playoffs (`true`). Embedded `matches` are slim: no `opponents`, no `results`. |
| `real-major-stage1-matches.json` | `GET /tournaments/20708/matches?page[size]=100` | The 33 matches of Stage 1 (a full Swiss: 20 Bo1, 13 Bo3, all finished), named "Round 5: NRG vs BIG". The round is in the name; no record or bucket field. |
| `real-major-playoffs-brackets.json` | `GET /tournaments/20710/brackets` | The 7 finished Playoffs matches (4 quarterfinals, 2 semifinals, Bo5 grand final) with `previous_matches` (`type: winner`). The order inside `previous_matches` is not top to bottom (the semifinal lists quarterfinal 4 before 3). |
| `real-upcoming-major-tournaments.json` | `GET /series/11176/tournaments` | PGL Major Singapore 2026 before it starts: four tournaments with 33/33/33/8 not-started matches (embedded form: no opponents; names such as "Round 1: TBD vs TBD"). |
| `real-nonswiss-serie-filter.json` | `GET /csgo/series?filter[id]=11156` | Elisa Open Suomi Season 14 (running). |
| `real-nonswiss-tournaments.json` | `GET /series/11156/tournaments` | Its "Group Stage" (round robin, 28 matches, yet `has_bracket: true`) and "Playoffs" (double elimination, 6 not-started matches named "Upper Bracket Semifinal 1: TBD vs TBD"; embedded form). |

## Documentation example (`doc-example-csgo_matches-1-0.json`)

Official OpenAPI example of `GET /csgo/matches?page[size]=1`, saved 2026-11-21, unedited: a not-started Bo3 "Grand final: TBD vs TBD" of CCT Europe 2025 with nested `serie` and `tournament`. Kept because no real capture has a match with both opponents TBD (`opponents: []`); the real `matches` of an upcoming Major were only captured in the embedded form.

## Constructed (`constructed-statuses.json`): NOT a real response

Hand-built from the documented match lifecycle, for exactly the cases no real capture contains: a forfeit (finished, `forfeit: true`), running, postponed, canceled, and a match with one opponent still TBD. Teams, ids and dates are invented. Remove an entry once a real capture shows the same status.

## What the real captures settled

- Stages: a Major's Swiss Stages are plain tournaments with `has_bracket: false`, named "Stage <n>" (2026) or "Opening/Elimination Stage", "Challengers/Legends Stage" (older Majors). The Playoffs have `has_bracket: true`. `has_bracket: true` also appears on round-robin "Group Stage" and "Play-In Group A", so it does not mean "knockout".
- Swiss: the round is in the match name; the record bucket must still be replayed, and the replay reproduces the buckets of a real 16-team Swiss (8, 8, 8, 6, 3 matches in rounds 1 to 5) and the Bo3 for every 2-win or 2-loss match.
- Names: the serie is "Cologne Major 2026" and the league "IEM" is separate; the Event name is therefore built as league + full_name, which the fallback chain matches against the HLTV name.
- All requests the fetch layer makes answer 200 on the owner's plan (page size `page[size]`, date range as plain dates).

## Still unverified

- Real `/tournaments/{id}/matches` of a running Swiss round (live matches, `running` status, a match with a TBD opponent in a running Stage) and of Stages 2 and 3 (their matches were not captured).
- Real `postponed`, `canceled` and forfeit matches (constructed only).
- `/tournaments/{id}/brackets` of a double-elimination bracket or a group phase (only the single-elimination Cologne bracket was captured; the double-elimination parse is from the embedded form without `previous_matches`).
- Whether PandaScore's top-to-bottom bracket order matches HLTV's (slots are ordered by the number in the match name, "Quarterfinal 3").
- `page[number]` beyond page 1 (no capture has more than one page).
- `search[name]` matches only the serie's own `name`, not the league or `full_name`; the adapter only uses it when an Event has no dates.
