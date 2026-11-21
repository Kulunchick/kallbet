# Match data from a fallback chain of Data Sources, keyed by HLTV ids

HLTV has no official API and sits behind Cloudflare, so scraping it (via headless Chrome) is fragile. Each Data Source (HLTV, PandaScore, Liquipedia) sits behind one adapter interface that maps its data into our own Event/Match/Team model. A fallback chain queries them in that priority order, and a circuit breaker per source skips a failing one for a while. HLTV ids are canonical. The other sources are linked to our records through a mapping table (source, external id → our id), auto-matched by normalised team name and start time, with mismatches fixable by hand. League-scoped Overrides sit above the whole chain, so a broken scraper never stalls a League.

## Considered Options

- **HLTV only**: rejected. One Cloudflare change would stop the game mid-Major.
- **PandaScore only**: clean JSON, but users pick Events by HLTV link and expect HLTV team names.

## Consequences

- When sources disagree on a winner, the highest-priority available source wins and League Admins see a conflict warning.
- Only Events with at least one League are polled, to keep HLTV traffic minimal.
