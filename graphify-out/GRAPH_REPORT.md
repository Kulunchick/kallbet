# Graph Report - KallBet  (2026-11-21)

## Corpus Check
- Corpus is ~2,099 words - fits in a single context window. You may not need a graph.

## Summary
- 45 nodes · 53 edges · 9 communities (6 shown, 3 thin omitted)
- Extraction: 94% EXTRACTED · 6% INFERRED · 0% AMBIGUOUS · INFERRED: 3 edges (avg confidence: 0.78)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- Pick'em Scoring
- Project & Agent Docs
- Leagues & Telegram Login
- Data Sources Fallback
- Reveal & Lock
- Match Lifecycle
- SQLite Decision
- KallBet Cargo Package

## God Nodes (most connected - your core abstractions)
1. `Member` - 5 edges
2. `Event` - 5 edges
3. `Fallback chain of Data Sources` - 5 edges
4. `Reveal locks the Match; Lock never undone` - 5 edges
5. `Stage` - 4 edges
6. `Match` - 4 edges
7. `Prediction` - 4 edges
8. `Lock` - 4 edges
9. `Pick'em` - 4 edges
10. `User` - 3 edges

## Surprising Connections (you probably didn't know these)
- `HLTV canonical ids and mapping table` --references--> `Event`  [INFERRED]
  docs/adr/0001-data-sources-fallback-chain.md → GLOSSARY.md
- `Reveal locks the Match; Lock never undone` --references--> `Pick'em`  [EXTRACTED]
  docs/adr/0003-reveal-locks-the-match.md → GLOSSARY.md
- `Telegram Login Widget as only sign-in` --references--> `User`  [EXTRACTED]
  docs/adr/0004-telegram-login.md → GLOSSARY.md
- `Reveal locks the Match; Lock never undone` --references--> `Lock`  [EXTRACTED]
  docs/adr/0003-reveal-locks-the-match.md → GLOSSARY.md
- `Reveal locks the Match; Lock never undone` --references--> `Reveal`  [EXTRACTED]
  docs/adr/0003-reveal-locks-the-match.md → GLOSSARY.md

## Import Cycles
- None detected.

## Hyperedges (group relationships)
- **KallBet architecture decision records** — docs_adr_0001_data_sources_fallback_chain, docs_adr_0002_sqlite, docs_adr_0003_reveal_locks_the_match [INFERRED 0.75]
- **Agent skill configuration docs** — docs_agents_domain, docs_agents_issue_tracker, docs_agents_triage_labels [EXTRACTED 1.00]

## Communities (9 total, 3 thin omitted)

### Community 0 - "Pick'em Scoring"
Cohesion: 0.22
Nodes (10): Leaderboard, Pick, Pick'em, Pick'em Leaderboard, Playoff Points, Playoffs, Point, Prediction (+2 more)

### Community 1 - "Project & Agent Docs"
Cohesion: 0.29
Nodes (6): ADR 0001: Data sources fallback chain, Domain Docs, Issue tracker: GitHub, Wayfinding operations, Triage Labels, KallBet

### Community 2 - "Leagues & Telegram Login"
Cohesion: 0.25
Nodes (8): ADR 0004: Telegram login, Telegram Login Widget as only sign-in, Invite Link, League, League Admin, Member, Nickname, User

### Community 3 - "Data Sources Fallback"
Cohesion: 0.47
Nodes (6): Circuit breaker per source, Fallback chain of Data Sources, HLTV canonical ids and mapping table, Data Source, Event, Override

### Community 4 - "Reveal & Lock"
Cohesion: 0.60
Nodes (5): ADR 0003: Reveal locks the Match, Reveal locks the Match; Lock never undone, Eligible Member, Lock, Reveal

### Community 5 - "Match Lifecycle"
Cohesion: 0.67
Nodes (3): Match, Scheduled Start, Void Match

## Knowledge Gaps
- **13 isolated node(s):** `KallBet`, `KallBet`, `Nickname`, `Invite Link`, `Swiss Stage` (+8 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 15 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **3 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Fallback chain of Data Sources` connect `Data Sources Fallback` to `Project & Agent Docs`?**
  _High betweenness centrality (0.297) - this node is a cross-community bridge._
- **What connects `KallBet`, `KallBet`, `Nickname` to the rest of the system?**
  _13 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Why does `Event` connect `Data Sources Fallback` to `Pick'em Scoring`, `Leagues & Telegram Login`, `Match Lifecycle`?**
  _High betweenness centrality (0.290) - this node is a cross-community bridge._
- **Why does `Member` connect `Leagues & Telegram Login` to `Pick'em Scoring`, `Reveal & Lock`?**
  _High betweenness centrality (0.245) - this node is a cross-community bridge._