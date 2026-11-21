# HLTV fixtures

Real pages from hltv.org, saved as the rendered DOM (`document.documentElement.outerHTML`) through headless Chrome/Edge over CDP on 2026-11-21. Never hand-write or hand-edit HLTV markup here: when HLTV changes, save the page again and fix the parser.

To keep the repository small, `<script>`, `<style>`, `<noscript>` and `<iframe>` elements and HTML comments were removed after saving; nothing else was changed. The parser does not read any of them.

| File | Page | State at capture |
|---|---|---|
| `events-list.html` | `/events` | ongoing, big and small upcoming events |
| `event-8301-cologne-major-parent.html` | IEM Cologne Major 2026 (parent: Stage 3 Swiss + Playoffs, Bo5 final) | finished |
| `event-9028-cologne-stage1.html` | its Stage 1 (Swiss, Bo1 and Bo3) | finished |
| `event-9029-cologne-stage2.html` | its Stage 2 (Swiss) | finished |
| `event-8302-major-singapore.html` | PGL Major Singapore 2026 (parent, links Stages 9303 and 9304) | upcoming |
| `event-9303-stage1-swiss.html` | its Stage 1 (Swiss, round 1 paired) | upcoming |
| `event-9469-swiss-plus-playoffs.html` | NODWIN Clutch Series 14: Swiss + Bo3 Playoffs, one match live, a renamed team | ongoing |
| `event-9496-double-elim-nonswiss.html` | POWER Ligaen S33 Finals: double elimination only, final live | ongoing |
| `event-not-found.html` | `/events/99999999/event`: HLTV's 404 page for an unknown event id | |
| `matches.html` | `/matches` (live and upcoming list) | |
| `cloudflare-challenge.html` | Cloudflare's "Just a moment..." page instead of an event page (not trimmed) | |

Not captured, because no such page could be found: a cancelled or postponed Match, the Singapore Stage 2 page (9304, Cloudflare blocked the capture), and a Swiss Stage page with a live Match.
