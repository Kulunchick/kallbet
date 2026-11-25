# Odds API fixtures

Captured on 2026-11-25. The response bodies are unchanged. The OddsPapi queries also supplied an API key, which is absent from the bodies.

| File | Request |
| --- | --- |
| `oddspapi-odds-by-tournaments-pgl-singapore-pinnacle.json` | `GET https://api.oddspapi.io/v4/odds-by-tournaments?tournamentIds=400014,51216&bookmaker=pinnacle%2B30&language=en&verbosity=3` at about 17:20 UTC |
| `oddspapi-historical-odds-od174000143389068-pinnacle.json` | `GET https://api.oddspapi.io/v4/historical-odds?fixtureId=od174000143389068&bookmakers=pinnacle%2B30` at about 17:20 UTC. Ninjas in Pyjamas vs M80 started at 05:03:15 UTC on November 24. The response includes in-play prices. |
| `polymarket-event-cs2-hero-faze-2026-11-26-open.json` | `GET https://gamma-api.polymarket.com/events?slug=cs2-hero-faze-2026-11-26` at about 16:45 UTC, from a remote server |
| `kalshi-events-KXCS2GAME-open.json` | `GET https://api.elections.kalshi.com/trade-api/v2/events?series_ticker=KXCS2GAME&status=open&with_nested_markets=true&limit=30` at about 16:45 UTC, from a remote server |
