# Codebase layout and conventions

One Rust binary (axum, sqlx + SQLite) serves the JSON API under `/api` and the built React SPA (`web/`). The layout is arranged so that tickets worked on in parallel touch different files. Keep it that way: add files, avoid editing shared ones.

## Commands

| What | Where | Command |
|---|---|---|
| Backend tests (Seam 1 and 2, unit, doctests) | repo root | `cargo test` |
| Lint | repo root | `cargo clippy --all-targets` |
| Frontend build (typecheck + bundle to `web/dist`) | `web/` | `npm run build` |
| Locale key check (en/ru/uk identical) | `web/` | `npm run check:locales` |
| Tests of the locale check itself | `web/` | `npm test` |
| Run locally | repo root | `npm run build` in `web/`, copy `.env.example` to `.env`, then `cargo run` → http://localhost:8095 |
| Frontend with hot reload | `web/` | `npm run dev` (proxies `/api` to the running binary on 8095) |

First time: `npm install` in `web/`. No database setup: the binary creates and migrates the SQLite file on start.

Before reporting a ticket done, run `cargo test`, `npm run build` and `npm run check:locales`.

## Backend map (`src/`)

| Path | Holds |
|---|---|
| `main.rs` | Start-up only: config, database, state, serve. |
| `runtime.rs` | Production wiring: one function per port (`match_source`, `notifier`, `clock`) plus `spawn_background_tasks`. Replace a placeholder inside its own function. |
| `config.rs` | `Config`, read from env. All secrets come from here. |
| `state.rs` | `AppState { db, clock, source, notifier, config }`. |
| `app.rs` | `build_router`: `/api` + SPA fallback. This is Seam 1. |
| `api/<area>.rs` | One file per API area, each with `pub fn router() -> Router<AppState>`. |
| `auth/` | `session.rs`: sessions and the `CurrentUser` extractor. Telegram verification goes in `auth/telegram.rs`. |
| `domain/` | Pure code, no I/O: `model.rs` (canonical Event, Stage, Match, Team, `EventSnapshot`), `ids.rs`, `language.rs`, `user.rs`, and one file per policy (`lock_reveal.rs`, `scoring.rs`, `pickem.rs`, `effective_match.rs`). |
| `ports/` | The three injected traits: `Clock`, `MatchSource`, `Notifier`, with their placeholder production adapters. |
| `store/<aggregate>.rs` | SQL. Plain `async fn`s taking `&SqlitePool`. |
| `sources/` | Data Source adapters (`hltv`, `pandascore`, `liquipedia`) and `fallback`. |
| `poller.rs`, `notifications.rs` | Background work. |
| `testing/` | Test fakes and the Seam 1 harness. Never used by production code. |
| `migrations/` | SQL migrations, embedded at compile time. |
| `tests/` | Integration tests, one file per area. |

Many of these files are stubs holding only a doc comment that names the owning ticket. They exist so a ticket fills in its own file and leaves `mod.rs` alone. A stub may grow into a directory module (`hltv.rs` → `hltv/mod.rs`).

## Adding to the API

1. Put handlers in the area's file, `src/api/<area>.rs`, and register them in its `router()` with paths relative to `/api`:
   ```rust
   pub fn router() -> Router<AppState> {
       Router::new().route("/leagues", post(create)).route("/leagues/{id}", get(show))
   }
   ```
2. A brand-new area needs its own file plus two lines in `src/api/mod.rs` (`pub mod x;` and `.merge(x::router())`). Keep both lists alphabetical.
3. Handlers return `ApiResult<T>` (`src/error.rs`). Take `CurrentUser(user): CurrentUser` to require a session (401 otherwise). Take `State(state): State<AppState>` for the database and ports.
4. SQL goes in `src/store/<aggregate>.rs`, decisions in `src/domain/`. Handlers stay thin: load, call the pure policy, persist, respond.

Rules that hold everywhere:

- **Time** comes from `state.clock.now()`. Never call `Utc::now()` outside `SystemClock`, or tests cannot move time.
- **Errors** are `{"error": {"code": "snake_case"}}` with a fitting status. The server sends no human-readable error text. The SPA shows the translation of `errors.<code>`, so a new code needs a key in all three languages.
- **Not a Member** of a League answers `404 not_found`, the same as a League that does not exist.
- **JSON** uses snake_case fields. Timestamps are RFC 3339 UTC. Ids are numbers.
- **SQL** uses the runtime functions (`sqlx::query`, `sqlx::query_as`), not the `query!` macros, so building needs no database.
- **Vocabulary** in names, routes, JSON and UI text follows `GLOSSARY.md`. One exception: a Match is `game` in Rust names (`game: &Match`, `Inputs { game, .. }`), because `match` is a keyword; JSON and UI still say Match. A Leaderboard row is a `LeaderboardRow` (never "standing"; HLTV's and Liquipedia's own Swiss "standings" tables are their markup, not ours).

## Adding a migration

Add one new file per ticket: `migrations/<NNNN>_<what>.sql`, where `NNNN` is the ticket number times 100 (ticket #4 → `0400_leagues.sql`; a second file from the same ticket → `0401_…`). Parallel tickets therefore never collide on a version number.

- Never edit a migration that is already on `spec-1-kallbet`: sqlx checksums the files and refuses to start on a mismatch. Change the schema with a new migration (`ALTER TABLE …`).
- A migration may rely only on tables from lower-numbered tickets it is blocked by.
- Timestamps are `TEXT`, bound from `chrono::DateTime<Utc>`. Booleans are `INTEGER`. Canonical ids (Event, Match, team) are the HLTV ids, stored as `INTEGER`.
- Foreign keys are enforced (`PRAGMA foreign_keys = ON`).
- Tests pick new migrations up on their own. No registration step.
- Migrations with a lower number than ones already applied still get applied, so merge order does not matter. If a local `kallbet.db` holds a migration your branch lacks, the binary refuses to start: delete the file.

## Writing a Seam 1 test

Create `tests/api_<area>.rs` (its own file, so it conflicts with nobody). Model: `tests/api_dev_login.rs`.

```rust
use axum::http::StatusCode;
use chrono::Duration;
use kallbet::ports::Clock;                     // for app.clock.now()
use kallbet::domain::{EventId, MatchId, StageKind};
use kallbet::testing::{EventSnapshotBuilder, TestApp};
use serde_json::json;

#[tokio::test]
async fn a_prediction_is_rejected_after_lock() {
    let app = TestApp::new().await;                  // fresh in-memory DB, fakes, dev login on
    app.source.put_event(
        EventSnapshotBuilder::new(7148, "Test Major")
            .stage(StageKind::Swiss, "Stage 1")
            .swiss_match(101, (1, 0, 0), "NAVI", "FaZe", app.clock.now() + Duration::hours(2))
            .build(app.clock.now()),
    );
    let alice = app.user("alice").await;             // signed in; keeps its cookies

    app.clock.advance(Duration::minutes(106));       // 14 minutes before the start

    let res = alice.put("/api/…", &json!({ "team_id": 1 })).await;
    assert_eq!(res.status, StatusCode::CONFLICT);
    assert_eq!(res.error_code(), "match_locked");
}
```

What the harness gives you (`src/testing/`):

- `TestApp::new()`, or `TestApp::builder()` with `.dev_login(false)`, `.web_dir(..)`, `.config(|c| ..)`, `.source(arc)` (another source behind the port) or `.source_with(|db, clock| arc)` (one that needs the test's database and clock, e.g. a `FallbackSource` of fakes; model: `tests/fallback_chain.rs`), then `.build()`.
- `app.user("nickname")`: a client signed in as that User (created on first use). `app.anonymous()`: a client with no session. Clients have `get`, `post`, `post_empty`, `put`, `patch`, `delete`. They keep cookies like a browser.
- `TestResponse`: `status`, `json()`, `json_as::<T>()`, `text()`, `header(name)`, `error_code()`.
- `app.clock` (`FakeClock`): `now()`, `advance(duration)`, `set(instant)`. It starts at 2026-01-10 12:00 UTC and only moves when told to.
- `app.source` (`ScriptedSource`): `put_event`, `update_event`, `update_match`, `postpone`, `start`, `finish`, `finish_without_winner`, `cancel`, `fail_with` / `recover`, `alias(stage_id, event_id)` (HLTV's Stage-of-a-Major behaviour), and the call counters `list_events_calls()`, `fetch_event_calls(id)`, `total_fetch_event_calls()`. `ScriptedSource::of(kind)` makes one that reports as another Data Source; wrap it in `ExternalFake::new(arc)` to use it as a fallback adapter (`ExternalSource`) in a `FallbackSource`.
- `app.notifier` (`RecordingNotifier`): `sent()`, `sent_to(user_id)`, `clear()`, `fail_with` / `recover`.
- `app.state`: the same `AppState` the router uses. Use it to drive non-HTTP entry points (a poller step) against the same database and fakes.
- `TEST_BOT_TOKEN`: the bot token in the test config, for signing Telegram login payloads.
- `TelegramLogin::new(app.clock.now(), telegram_id, "Name")` (+ `.username()`, `.photo_url()`, `.language()`): builds a signed `POST /api/auth/telegram` body via `.body()`; `.body_signed_with(token)` and `.body_tampered(field, value)` make forgeries. To act as a *Telegram-created* User (instead of `app.user(..)`, which uses the dev login), post that body with `app.anonymous()`.
- `client.cookie(name)` / `client.set_cookie(name, value)`: read or hand a cookie to a client (e.g. replay a session cookie after logout).

Test rules (spec, Testing Decisions):

- Assert what a client can observe: responses, what another Member sees, Points, notifications sent. Do not query the database to check a result.
- Each test builds its own `TestApp`. Nothing is shared, so tests run in parallel.
- Background work must be steppable. Give the poller a method that does one pass based on `clock.now()`; tests advance the clock and call it. Tests never sleep.
- The in-memory database has a single connection. Do not hold a transaction open while awaiting another query on the pool: it deadlocks.

Seam 2 (parser) tests keep fixtures under `tests/fixtures/<source>/` and feed them to the pure parser. No network. Fixtures are real saved responses, never hand-written; list each one with its provenance in `tests/fixtures/<source>/README.md`. A test may derive a broken variant from a real fixture in code (e.g. a renamed class) to check that changed markup is an error.

## Data Sources

- Every adapter implements `MatchSource` and returns `SourceError::Parse` for anything it does not understand, never empty data. `Unavailable` is for network, browser, Cloudflare and HTTP failures.
- `fetch_event(id)` may return a snapshot whose `event.id` differs from `id`: HLTV gives each Stage of a Major its own event id, and the canonical Event is the Major (the parent HLTV event). Store and key by `snapshot.event.id`.
- Team logos are never sent to the SPA as Data Source URLs (`Team::logo_url`). The SPA shows `/api/teams/{id}/logo`; the HLTV adapter downloads each logo once (`sources::hltv::LogoCache`).
- HLTV adapter: `sources::hltv::HltvSource::new(Arc::new(ChromePages::new(url)), clock).with_logos(LogoCache::new(dir))`. Its pure parser is `sources::hltv::parse`. `tests/hltv_live.rs` runs it against a real browser and HLTV (ignored by default; see the file for how to run it).

## Fallback chain (ticket #9) — read before #10, #11, #13

`state.source` in production is `sources::fallback::FallbackSource` (`runtime::match_source`): HLTV → PandaScore → Liquipedia, one circuit breaker each. Callers just use `MatchSource`; nothing else changes.

**Writing a fallback adapter (#10, #11).** Implement `sources::fallback::ExternalSource` in your own file (`sources/pandascore.rs`, `sources/liquipedia.rs`). Both already hold a placeholder type (`PandaScoreSource::new(token)`, `LiquipediaSource::new()`) that answers `NotConfigured` and is already wired into the chain in `runtime::match_source`: replace its body, keep the type (add constructor arguments if you need them, e.g. a `Clock` for rate limiting, and adjust that one call in `runtime.rs`). You never produce canonical ids yourself:

- `kind()` → your `DataSourceKind`.
- `list_events()` → `Vec<ExternalEvent { external_id, canonical: Option<EventId>, name, starts_at, ends_at }>`.
- `find_events(&EventHint { id, name, starts_at, ends_at })` → candidates for the canonical Event (search by name/dates, return a few; empty if nothing is close). `name` is empty when the Event was never stored (HLTV was down when the League was created): then only a candidate with `canonical == Some(id)` is accepted, so search by HLTV id if your source can. Called only until the Event is mapped.
- `fetch_event(external_id)` → `ExternalSnapshot { event, stages: Vec<Stage>, matches: Vec<ExternalMatch> }` for the **whole** Event (every Stage of a Major). `ExternalMatch { external_id, canonical: Option<MatchId>, stage (an ordinal of your own `stages`), placement, team_a/team_b: Option<ExternalTeam { external_id, name, canonical: Option<TeamId> }>, format, scheduled_start, actual_start, status, winner: Option<Side::A|B>, score }`, score in your record's team order.
- Errors as for `MatchSource`: `NotConfigured` (skipped, not a failure), `Unavailable`, `Parse` (both count for the breaker), `NotFound`.
- Put an id your data carries in `canonical` (Liquipedia: a Match template's `hltv=` → `ExternalMatch::canonical`, an infobox HLTV event link → `ExternalEvent::canonical`); it links directly, bypassing name/time matching.
- Parser tests are Seam 2 as usual (fixture → `ExternalSnapshot`). "Records are mapped to canonical ids through the mapping table" is the chain's job and already tested in `tests/fallback_chain.rs`; to check your adapter end to end, compose it in a `FallbackSource` like that file does.

**How the chain behaves** (`sources/fallback/mod.rs`, pure rules in `mapping.rs`):

- `fetch_event(id)`: first resolve aliases (an HLTV Stage event id that HLTV once answered with its Major is stored as an `hltv`/`event` mapping, so a pasted Stage link keeps resolving to the Major while HLTV is down). HLTV answers → its snapshot is returned unchanged; `NotFound` from HLTV ends the search. Otherwise each fallback in order: find its Event (mapping, else `find_events` + `mapping::choose_event`: carried HLTV id, else best name similarity ≥ 0.4 with overlapping dates, ambiguous = none), fetch, link, merge. All fail → `Unavailable` if any source failed, else `NotFound`, else `NotConfigured`.
- **Breaker**: `NotConfigured` never counts; `Unavailable` and `Parse` count; `NotFound` counts as a success. After `KALLBET_BREAKER_FAILURES` (3) in a row the source is skipped for `KALLBET_BREAKER_OPEN_MINUTES` (15), then tried once (half-open: exactly one call goes through, concurrent ones are refused until it reports back): success closes it, failure reopens it. Time comes from `Clock`. This is the poller's back-off. In-memory, per adapter, two per adapter: one for `fetch_event`, one for `list_events`/`find_events`, so a working Event list does not reset the count of failing Event pages (tests: `tests/fallback_breaker.rs`). An HLTV parse error is a failure like any other, so the chain falls through to fallbacks. A Swiss Match drawn in the chart but in neither team's standings rows (as during a live round) is not an error: the parser leaves it out of that snapshot, and the stored Match stays as it was (in only one team's rows is still an error). This is checked on a fixture derived from a real page; no real page with a live Swiss round has been saved yet, so the real markup of that state is unverified.
- **Linking a record**, first hit wins: manual mapping → id carried by the record → earlier automatic mapping → matching. Teams match by normalised name (`fallback::names::team_key`: lowercase alphanumerics without "team", "esports", "gaming", "clan", ...), this Event's teams first, then all teams. A Match matches a stored Match of the same Stage (the record's Stage mapped by kind and position) by teams and Scheduled Start within ± `KALLBET_MATCH_WINDOW_HOURS` (6); one agreeing team plus the time suffices, and the other team is then inferred and mapped (unless another record of the same snapshot is that team). Every link is stored in `external_ids` (origin `auto` / `source`).
- **Unmatched** (Event has HLTV data): a record with a team that cannot be linked, with no teams, or with a team pair that already meets in that Stage at another time, is left out and flagged in `unmatched_records` (kinds `match` and `team`; `event` with an empty external id when no Event fitted). Flags are replaced per (Event, source) on each fetch from that source, so a fix clears its flag on the next fetch. A Match whose two teams are linked but which is not stored yet (a new Swiss round while HLTV cannot parse it) is **added** with a local id.
- **Merge rule** (lower priority never spoils higher priority): a fallback updates only the progress fields of a stored Match (Scheduled Start, actual start, status, winner, score), fills teams still TBD, and keeps Stage, placement, format and HLTV team names; it never moves a Match backwards (finished → live → upcoming); a record whose winner is not one of the stored Match's two teams (a linked record whose source changed a team) is flagged as unmatched and not merged; it never touches a Match a higher-priority source has settled (finished/cancelled) — a different winner there is recorded as a conflict and the stored result stands. A settled result from a lower-priority source is overwritten by a higher one (HLTV always overwrites), with a conflict when the winner differs. Stored Matches a source does not report are left as they are (`save_snapshot` never deletes). The merged snapshot contains only what that source may write, and its Matches get `source` = that source.
- **Conflicts** (`source_conflicts`): `{match_id, kept_source, kept_winner, other_source, other_winner (NULL = Void), detected_at}`, keyed by (match, other source); kept even if the sources later agree.

**Ids of Matches not on HLTV (decision; #13 uses it too).** Canonical ids stay HLTV's positive `i64`s; a Match or team with no known HLTV id gets a **negative local id** (-1, -2, ...) from `store::external_ids::allocate_local_ids(db, IdKind::Match | Team, count)` (returns the first; the rest count down). #13: take a hand-added Match's id from there. When a fresh HLTV snapshot shows a local Match (same teams by name, or one team plus the time, within the window), the chain re-keys it to the HLTV id before the poller saves the snapshot (`rekey_match` / `rekey_team`: matches, predictions, match_locks, picks, conflicts, mappings), so Predictions follow. **Any new table with a match or team id column must be added to `rekey_match` / `rekey_team`** (#13 added `overrides` and the `notification_log` keys). Hand-added Matches (#13) are never re-keyed: they are League data outside `base_view`, see "Overrides (ticket #13)".

**HLTV down and the Event never fetched from HLTV**: if the Event row exists (from an earlier HLTV Event list, or a League), fallbacks find it by name and dates; its Matches and teams get local ids (nothing to link to), and are adopted when HLTV returns. If nothing is known but the id, only a fallback whose candidate carries that HLTV id can serve it; otherwise League creation answers 502 `source_unavailable` and the user retries. `list_events` falls back the same way but only returns Events it can link to a canonical id (mapping, carried HLTV id, or a stored Event matched by name); others are dropped.

**For #13 (store functions, `store/external_ids.rs`)**: `unmatched(db, event) -> Vec<UnmatchedRecord { source, kind, external_id, description, seen_at }>`, `conflicts(db, event) -> Vec<SourceConflict>`, and the fix `set_manual(db, source, IdKind::Team | Match | Event, external_id, canonical_id, now)`: a manual row is never overwritten automatically and wins over every other link from the next fetch on. These are base-data fixes (they apply to every League on the Event). #13 decided League Admins do **not** get `set_manual` (spec story 76): they see `unmatched`/`conflicts` read-only (`GET /api/leagues/{id}/data-warnings`) and fix their own League with a teams Override.

**PandaScore adapter (#10)**: `sources/pandascore/` (`parse.rs` pure parser, `mod.rs` REST fetch with `PandaScoreSource::new(token)` / `with_base_url`). No token (or an empty one) → `NotConfigured` before any request. Token goes in the `Authorization: Bearer` header; errors never contain it. Serie = Event, tournament = Stage (`has_bracket` → Playoffs; otherwise Swiss if the name says "Swiss" or "Stage <n>", else Groups: a guess, docs never mention Swiss). PandaScore has no Swiss round field, so the parser replays results in Scheduled Start order (equal records `w`-`l` → round `w+l+1`; a TBD team or unequal records → `Unplaced`). Playoffs slots come from `previous_matches` of `/tournaments/{id}/brackets` (single-elimination tree only, else `Unplaced`). `postponed` → Upcoming with no Scheduled Start; `canceled` without a forfeit → Cancelled. The `/csgo/series...` paths and filters are unverified; see `tests/fixtures/pandascore/README.md` for what is real, what is constructed, and which requests the owner should capture. Tests: `tests/pandascore_parser.rs` (Seam 2), `tests/pandascore_source.rs` (fetch layer against a local server).

Tests: `tests/fallback_chain.rs` (fallback on failure, order, breaker open/half-open/close, unconfigured skipped, carried ids, unmatched flags, manual mapping, conflicts both ways, no rollback, local ids and adoption with Predictions, never-fetched Event, Stage link). Unit tests for names in `sources/fallback/names.rs`.

## Liquipedia adapter (ticket #11)

`sources/liquipedia/`: `LiquipediaSource::new(clock)` (production; `::with(wiki, pause, clock)` for tests), `parse.rs` (pure, Seam 2), `wikitext.rs` (template reader), `limiter.rs` (rate limits). Tests: `tests/liquipedia_parser.rs` (fixtures in `tests/fixtures/liquipedia/`, README has provenance), `tests/liquipedia_source.rs` (limits, headers, failures, and the adapter inside a `FallbackSource`), `tests/liquipedia_live.rs` (ignored, real API).

- **API rules** (<https://liquipedia.net/api-terms-of-use>): at most 1 request per 2 s; `action=parse` at most 1 per 30 s (never used); custom User-Agent `KallBet/0.1 (https://kallbet.kulunchick.tech)`; gzip; cache. `limiter::RateLimiter` books slots from the injected `Clock`; the waiting goes through the `Pause` port (tests move a fake clock, nothing sleeps). `expandtemplates` is held to the 30 s class too. A wait over 60 s fails as `Unavailable`; a 429 backs off for `Retry-After` (default 2 min), a 403 for 10 min. `reqwest` gained the `gzip` feature.
- **Event id** = the root page title (`Intel Extreme Masters/2026/Cologne`). `fetch_event` = `list=allpages` (Stage subpages) + one `action=query` for all pages + at most one `expandtemplates` for team keys not yet known (names are cached for the process). Stages come from `{{SwissStandings}}` (Swiss) and `{{Bracket|Bracket/N}}` with N a power of two (Playoffs); any other bracket (double elimination) is a `Groups` Stage with `Unplaced` Matches, like HLTV's. A Matchlist-only page yields no Stage.
- **What is derived**: winner from map round totals (overtime included) or `score=W`; Swiss record from the High/Mid/Low Matchlist order of each Round (a different count of lists is a `Parse` error); format from the number of `mapN` slots; times from `{{Abbr/CEST}}`-style zones (unknown zone, e.g. `CST`, is a `Parse` error); `Live` = unfinished and started less than 12 h ago, or a map already played. `hltv=` goes to `ExternalMatch::canonical`. Empty TBD slots with no teams and no `hltv=` are left out. Team external id = template key; names via `{{Team|key}}` (not `{{TeamName}}`).
- **Not done**: `list_events` answers `NotConfigured` (the lists are Lua-generated, only reachable through `action=parse`); `find_events` searches by the HLTV name and keeps pages with an `Infobox league`, so it needs a stored name. No cancelled/postponed state (never seen on a page). No `canonical` Event or team ids (Liquipedia pages carry no HLTV Event id).

## Frontend map (`web/`)

| Path | Holds |
|---|---|
| `src/pages/<area>/routes.tsx` | Default-exports the area's `RouteObject[]`. Collected automatically by `src/app/router.tsx`. |
| `src/pages/<area>/` | The area's pages, components and API calls (`api.ts`). |
| `src/locales/<lang>/<namespace>.json` | Translations, one namespace per area. Loaded automatically by `src/i18n/index.ts`. |
| `src/components/` | Shared pieces: `Logo`, `LanguageSwitcher`, `HeaderUser` (the header's User corner), `ServerStatus`, `Avatar` (a User's avatar, initial as fallback), `TeamLogo` (a team's logo from `/api/teams/{id}/logo`, nothing when it has none), `Medal`. |
| `src/app/Layout.tsx` | Header, page outlet, footer. |
| `src/api/client.ts` | `api<T>(path, {method, body})`, `ApiError` (`status`, `code`), `errorMessage(t, error)`. |
| `src/api/useLoad.ts` | `useLoad(load, deps)` → `{result: loading | error | ready, reload}`: how every page loads its data. |
| `src/styles/theme.css` | Theme tokens and utilities. |
| `src/assets/logo.svg`, `public/favicon.svg` | The logo (currently a placeholder). |
| `scripts/check-locales.mjs` | The locale key check. |

Adding a page: create `src/pages/<area>/routes.tsx` and its locale namespace in all three languages. Neither the router nor the i18n setup needs editing.

```tsx
// src/pages/leagues/routes.tsx
const routes: RouteObject[] = [{ path: "/leagues", element: <MyLeaguesPage /> }];
export default routes;

// in a component
const { t } = useTranslation("leagues");   // src/locales/<lang>/leagues.json
```

Locales:

- Every namespace file exists for `en`, `ru` and `uk` with the same keys; `npm run check:locales` fails otherwise. Plural forms (`key_one`, `key_few`, `key_many`, `key_other`) count as one key.
- Put an area's strings, including its `errors.<code>` texts, in the area's own namespace. `common.json` is shared and therefore a merge hotspot: add to it only for strings used across areas.
- No user-visible string is hard-coded in a component.

Theme (dark only):

- Colours: `ink` (page), `panel` (cards), `raised`, `line` (borders), `fg`, `mute`, `emerald-a/b`, `gold-a/b`, `danger`. Use them as Tailwind classes: `bg-panel`, `text-mute`, `border-line`.
- Utilities: `bg-emerald-gradient`, `bg-gold-gradient`, `text-emerald-gradient`, `text-gold-gradient`, `glow-emerald`, `glow-gold`, `text-glow-gold`, `card`, `btn-primary`, `btn-ghost`.
- Emerald means my Prediction, correct, and primary actions. Gold means Points, the leader and headings.
- Fonts: `font-display` (Unbounded) for headings, Inter for body. Both are bundled, with Cyrillic.

## Configuration and secrets

Settings come from environment variables; `.env` is loaded for local development and is git-ignored. `.env.example` lists every variable.

| Variable | Default | Meaning |
|---|---|---|
| `KALLBET_BIND` | `127.0.0.1:8095` | Listen address. |
| `DATABASE_URL` | `sqlite://kallbet.db` | SQLite file. |
| `KALLBET_WEB_DIR` | `web/dist` | The built SPA. |
| `KALLBET_DEV_LOGIN` | off | `true` mounts `POST /api/dev/login`. Never set in production. |
| `BOT_TOKEN` | unset | Telegram bot token (secret). Unset: `POST /api/auth/telegram` answers 503 `login_unavailable`. |
| `BOT_USERNAME` | unset | The bot's @username (no @), needed by the Login Widget in the SPA (`GET /api/auth/config`). |
| `PANDASCORE_TOKEN` | unset | PandaScore token (secret). Unset means the adapter is skipped. |
| `KALLBET_CHROME_URL` | unset | DevTools endpoint of the headless Chrome the HLTV adapter uses (`http://chrome:9222` in compose; a hostname is resolved to an IP). Unset means the HLTV adapter is skipped. |
| `KALLBET_LOGO_DIR` | `logos` | Downloaded team logos, served at `/api/teams/{id}/logo`. |
| `KALLBET_BREAKER_FAILURES` | `3` | Consecutive failures that open a Data Source's circuit breaker. |
| `KALLBET_BREAKER_OPEN_MINUTES` | `15` | How long an open breaker skips its Data Source before one more try. |
| `KALLBET_MATCH_WINDOW_HOURS` | `6` | ± hours within which two sources' start times count as the same Match. |

A new setting needs a field in `Config`, a line in `Config::from_lookup`, a value in `Config::for_tests`, and an entry in `.env.example`. `Config`'s `Debug` output redacts secrets; keep it that way for new ones (URLs that may carry credentials, like `KALLBET_CHROME_URL`, print only `config::url_origin`). Request logs carry the method and `app::loggable_path` (Invite Link tokens replaced, no query), never the raw URI.

Dev login, when enabled: `POST /api/dev/login` with `{"nickname": "alice"}` signs in as that User, creating it if needed. When disabled the route does not exist (404).

## Leagues and membership (ticket #4)

- **Gate every League-scoped endpoint** with `api::leagues::require_member(&state, &user, league_id)` (404 `not_found` for non-Members and missing Leagues) or `require_admin` (adds 403 `forbidden`). Both return the `store::leagues::League` and the caller's `Membership`.
- **Membership is a stretch of time**: table `memberships` has `joined_at` and `left_at` (NULL while current). Leaving sets `left_at`; joining again inserts a new row. `store::leagues::all_memberships(db, league)` returns every stretch, for the Eligible Member rule. `store::leagues::members` lists current Members only. At most one current row per User per League (partial unique index). #12's kick should call `end_membership`.
- Roles are `domain::Role` (`member`, `admin`). The creator is `leagues.creator_id` and also an admin Member; the creator cannot leave (409 `creator_cannot_leave`).
- The `events` table (0400) holds only id, name and dates, upserted from `MatchSource::fetch_event` when a League is created. #5 adds Stages, teams and Matches in its own migration and fills `store/events.rs`.
- Invite Link: the secret is `leagues.invite_token`; the SPA link is `/join/<token>`. API: `GET /api/invites/{token}` (preview) and `POST /api/invites/{token}/join`; an unknown or reissued token is 404 `invite_invalid`. Only admins receive `invite_token` in the League JSON.
- Data Source failures reach clients as 502 `source_unavailable` (`api::events::source_error`).
- Frontend: the header (`HeaderUser`) links to `/leagues`. `/join/<token>` signed out shows a sign-in prompt linking to `/login?next=/join/<token>`; `LoginPage` honours `?next=` (same-site paths only) after any sign-in, so pages can send visitors through login and back the same way.

## League management (ticket #12)

- Endpoints (all gated by `require_admin` unless noted): `DELETE /api/leagues/{id}/members/{user_id}` (kick = `end_membership`; 409 `creator_protected` for the creator, 409 `use_leave` for yourself, 404 when the target is not a current Member), `PUT /api/leagues/{id}/members/{user_id}/role` (`{"role": "member"|"admin"}`, 204; creator is 409 `creator_protected`), `POST /api/leagues/{id}/delete` (creator only: 403 for other Members, 422 `confirmation_mismatch` unless `{"confirm_name"}` equals the League name).
- A kicked Member has a `left_at` like a Member who left, so they stop being Eligible from that moment by the same rule. They can rejoin through a valid Invite Link; an admin who wants them gone for good reissues the link.
- **Deleting a League must keep working as tables grow.** `store::leagues::delete` is a single `DELETE FROM leagues`. Every later table that references a League (directly, or through `memberships`) must declare `REFERENCES leagues (id) ON DELETE CASCADE` (or cascade from another table that does), as `memberships` does. Never reference `leagues` without a cascade, and do not add a hand-written delete step per table. Tests: delete a League that has your rows and check they are gone through the API.
- **Rule for #5 and #13 reviewers: no API path lets anyone create or change another Member's Prediction.** A Prediction handler takes the User only from `CurrentUser`, never from the path or body, and writes only for that User. No admin/creator override exists. The test is `tests/api_prediction_ownership.rs` (#6); extend it when adding any Prediction-writing route.

## Matches, Lock and the poller (ticket #5)

How League logic gets its Matches — read this before #6, #7, #13, #14, #16, #17:

- **One entry point**: `league_matches::evaluate(db, &league, now) -> LeagueMatches`. It loads the League's **effective Match view**, its membership stretches, all its Predictions and stored stamps, runs the Lock/Reveal policy for every Match, **persists new stamps**, and returns `{ view, outcomes, predictions, members, profiles, … }`. The Matches API, the Prediction endpoint and the poller all call it. Never read `store::events::base_view` (raw source rows) from League logic.
- **Effective Match view**: `domain::effective_match::effective_view(base, &LeagueOverrides) -> EventView` is pure and applies the League's Overrides (see "Overrides (ticket #13)"). `EventView { event, stages, teams, matches: Vec<SourcedMatch { data: Match, source, updated_at }> }`. Hidden Matches drop out of `matches`; hand-added ones are appended.
- **Lock/Reveal policy**: `domain::lock_reveal::evaluate(&Inputs { game, members, predicted, stored, now }) -> Outcome { stamps, lock_time, transitions }`. Lock = earliest of Scheduled Start − 15 min, actual start (or "now" if started without a time) and cancellation, once it is ≤ now; once stored it never changes. `members` (every `MemberSpan { user_id, joined_at, left_at }`) and `predicted` (users with a Prediction) drive Reveal (#6, see below); `Outcome.eligible` lists the Eligible Members. `store::predictions::record` persists transitions with COALESCE, so a stamp is never overwritten.
- **Schema** (`migrations/0500_matches_and_predictions.sql`): `stages`, `teams`, `matches` (base data; `placement_kind` + `swiss_*` / `playoff_*` columns; `source`, `updated_at` = last successful poll), `polls` (cadence bookkeeping), `predictions (league_id, match_id, user_id, team_id, updated_at)`, `match_locks (league_id, match_id, locked_at, revealed_at)`.
- **Poller** (`poller.rs`): `poller::tick(&state)` is one pass at `clock.now()`; production runs it every 20 s (`runtime::spawn_background_tasks`). Order: evaluate every active League (stores Locks due by the data we had, before a postponement can move them) → poll the Event list (6 h; stored in `event_list` by `poller::refresh_event_list`, and `GET /api/events` serves that copy, asking the source only while nothing is stored yet) → poll each Event with a non-archived League (5 min, 1 min if a Match is live or starts within 45 min, by the source's data or any active League's effective view without hand-added Matches, so a Scheduled Start Override counts) → evaluate every active League again and call `after_poll(state, league, matches, now)`. **#7, #16, #17 each add one call inside `after_poll`.** It runs on every tick, not only when data changed, so make those hooks idempotent (scores recomputed, notifications deduped). A Match first seen live without a source `actual_start` gets `now` stamped.
- **API**: `GET /api/leagues/{id}/matches` → `{ event, stages: [{ ordinal, kind, name, groups: [{ kind: "record", wins, losses | kind: "playoff_round", rounds_before_final | kind: "unplaced", matches: [MatchCard] }] }] }`. `MatchCard` (`api/matches.rs`) has `team_a/team_b {id,name}|null`, `format`, `bracket_slot` (Playoffs slot within its round, else `null`), `scheduled_start`, `actual_start`, `status`, `winner`, `score`, `lock { locks_at, locked_at, locked }`, `my_prediction`, `source`, `updated_at`. It also has `predictions` (#6) and `my_result`/`my_points` (#7), see their sections; build fields from `LeagueMatches` in `view_for`. Never put other Members' Predictions in it before Reveal.
- `PUT /api/leagues/{id}/matches/{match_id}/prediction` `{team_id}` → `{match_id, user_id, team_id, updated_at}`; errors 404 `not_found`, 409 `match_locked`, 422 `team_not_in_match`. It evaluates before (reject if locked) and after the upsert (where #6's Reveal on the last Prediction fires). #17 adds the archived check here.
- **Writes that depend on Lock are one write transaction** (ADR-0003). A Prediction, a Pick'em, every Override write and the poller's save of a fetched snapshot open `db::begin_write(&state.db)` (`BEGIN IMMEDIATE`), read `state.clock.now()` only once it is open, call `league_matches::evaluate_in(&mut tx, ..)` (stores every Lock due by the data as it is), check, write, evaluate again and end with `db::finish(tx, result)` (commits on success and on 4xx, so stamps stored by the evaluation are kept). The Prediction and Pick'em store writes are also conditional on no stored Lock (`predictions::upsert` returns `None`, `pickems::save` returns `false`). Never move a Scheduled Start (Override, snapshot) without evaluating the League at the current clock first in the same transaction. Inside the transaction use only the transaction's connection: the in-memory test pool has one connection, so a pool query deadlocks. Tests: `tests/lock_atomicity.rs` (interleavings driven through store functions and a slow fake source).
- **Pick'em (#14)**: a Stage's Matches and its first start come from `LeagueMatches.view` (`EventView::matches` filtered by `data.stage`); its Lock is the first `actual_start`/live status in the Stage.
- **SPA**: `/leagues/:id/matches` (`web/src/pages/matches/`). `MatchCard.tsx` renders `PredictionsLine.tsx` (#6); #7's ✓/✗ goes next to the score; team logos via `TeamLogo`. A Playoffs Stage is drawn as a bracket (`PlayoffsBracket` in `MatchesPage.tsx`); the layout and the card state (a Void Match is cancelled or finished without a winner, and says so with a visible badge, never "Finished") are pure functions in `layout.ts`, tested by `web/scripts/matches-layout.test.mjs` (`npm test`). Strings in the `matches` namespace. The League page links to it.
- Local manual testing: the production `MatchSource` is the fallback chain (#9). Without `KALLBET_CHROME_URL` (and until #10/#11 land) every source is skipped, so a fresh League shows no Matches unless the DB is seeded by hand.

## Reveal (ticket #6)

- **Rule** (`domain::lock_reveal::evaluate`, ADR-0003): while open, if every Eligible Member has predicted (and there is at least one), the Match Reveals and Locks at that instant (`Locked(now)` + `Revealed(now)`). Otherwise it Reveals at Lock, stamped with `locked_at`. A stored Lock without a Reveal (data from before #6) is Revealed at its Lock on the next evaluation. Stamps are never cleared.
- **Eligible Members** (`lock_reveal::eligible(members, locked_at)`): current Members (`left_at` NULL) whose current stretch began at or before `locked_at`; while open, every current Member. So leaving or being kicked removes a Member at once (the next evaluation may Reveal: any later API call or poller tick does it), and a Member who joins, or rejoins, after Lock does not count. N counts only Eligible Members' Predictions.
- **API**: `MatchCard.predictions = { count, eligible, revealed_at, revealed }`. `revealed` is `null` before Reveal and then `[{ user_id, nickname, avatar_url, team_id }]` for the Eligible Members who predicted. It is built only in `api::matches::predictions_view`; it is the only place other Members' Predictions leave the server. `LeagueMatches.profiles` (current Members with Nickname/avatar) feeds it.
- **For #16 (Reveal announcement exactly once)**: the moment of Reveal is `match_locks.revealed_at`, exposed as `LeagueMatches.outcome(id).stamps.revealed_at`. A Reveal can be stored by any caller of `league_matches::evaluate` (the Prediction endpoint fires the early one), so do not rely on seeing `Transition::Revealed` in `after_poll`. Instead, in `after_poll`, announce every Match whose `stamps.revealed_at` is set and dedupe through the notification log with a key such as `reveal:<league>:<match>`; skip ones revealed long ago (e.g. `now - revealed_at` over a few hours) if old data must not trigger a burst.
- Tests: `tests/api_reveal.rs` (each rule, the no-leak check, Lock and Reveal per League); `tests/api_prediction_ownership.rs` (#12's rule).
- SPA: `web/src/pages/matches/PredictionsLine.tsx` shows "N of M predicted", then avatars split by team. `MatchesPage` reloads after a Prediction so a Reveal it triggered shows at once.

## Pick'em (ticket #14; #15 adds the Playoffs variant)

- **Stage ordinals can change** when HLTV returns with a Stage a fallback did not list: `store::events::save_snapshot` then moves everything kept by ordinal (`pickems`, `picks`, `pickem_locks`, hand-added `matches`) to the new ordinal, read from the stored Matches the snapshot places elsewhere (`move_stage_data`). **A new table keyed by Stage ordinal must be added there.**
- **Storage is shape-generic** (`migrations/1400_swiss_pickem.sql`): `pickems (league, stage_ordinal, user)` + `picks (…, kind, slot, team_id)` (cascade from `leagues`), and `pickem_locks (league, stage_ordinal, locked_at)`. Swiss kinds: `three_nil`, `zero_three`, `advance` (slot = list position). **#15: use kind `bracket`, slot = bracket slot**; a team may repeat across slots, so team uniqueness is a domain rule, not a constraint. `domain::pickem::{Pick, PickKind}` already has `Bracket`.
- **Domain** (`domain/pickem.rs`, pure, reads the effective view only): `SwissPickem::{validate, to_picks, from_picks}`, `swiss_teams`, `swiss_outcomes`, `score_swiss`; shared by every shape: `lock(view, stage, stored, now)`, `expected_lock`, `rank` (shared places, 1-1-3). #15 adds its own `BracketPickem` validate/score next to them and reuses `lock`/`rank`.
- **A Stage's team list**: a Swiss Stage's round 1 contains all its teams, so `swiss_teams` is the teams of its round-1 Matches (all Matches of the Stage if the Data Source gives no round numbers). It is known from the first poll, long before later rounds are paired. For the Playoffs the candidates are the teams of the paired bracket slots; later slots are filled by the picker's own earlier picks (#15 decides how to treat TBD first-round slots).
- **Lock** = the Stage's first Match starting: earliest of actual start (or "now" if seen live) and Scheduled Start, not cancelled Matches. Unlike a Match Lock it has no 15-minute lead. It is stored once (`store::pickems::evaluate_lock`) and `league_matches::evaluate` calls it for every Swiss Stage (**#15: extend that loop to the Playoffs Stage**), so the poller, GET and PUT all stamp it and a later postponement cannot reopen it. Never a Reveal on "everyone picked" (ADR-0003): hidden until Lock only.
- **Scoring** (only for a Stage whose Pick'em has Locked: a result can arrive earlier, e.g. a forfeit before the planned start, and Points would leak Picks): 1 Point per correct Pick among *settled* teams (3 wins or 3 losses in finished, non-Void Matches of the Stage): `three_nil` needs exactly 3-0, `zero_three` exactly 0-3, `advance` 3 wins with at least one loss. Pick'em Points are separate from Match Points.
- **API** (`api/pickem.rs`): `GET|PUT /api/leagues/{id}/stages/{ordinal}/pickem`. GET → `{stage, teams, lock {locks_at, locked_at, locked}, my_pickem|null, pickems|null}` (`pickems`, everyone's, is `null` until Lock). PUT body `{three_nil[2], zero_three[2], advance[6]}`; errors 404 `not_found` (non-Member, or a Stage without Pick'em: anything but a Swiss Stage today), 409 `pickem_locked`, 422 `pickem_wrong_count` / `pickem_duplicate_team` / `pickem_team_not_in_stage`. JSON carries `shape: "swiss"`; **#15 returns `shape: "playoffs"` from the same endpoints for the Playoffs Stage** (`pickem_stage` in `api/pickem.rs` is the gate to widen). The Pick'em Leaderboard is `GET /api/leagues/{id}/pickem-leaderboard` → `{entries: [{place, user_id, nickname, avatar_url, points}]}`, served from `api/pickem.rs` (not `leaderboard.rs`); **#15 adds its Stage scores to the sum in `leaderboard`**.
- **SPA**: `/leagues/:id/pickem` (`web/src/pages/pickem/`, namespace `pickem`): leaderboard plus one `PickemStage` per Swiss Stage (form until Lock, everyone's Pick'ems after). #15 adds a bracket component for Playoffs Stages next to it.
- Tests: `tests/api_pickem.rs` (setup has a Swiss Stage of 16 teams and a Playoffs Stage).

### Playoffs Pick'em (ticket #15)

- **Same endpoints, second shape.** `GET|PUT /api/leagues/{id}/stages/{ordinal}/pickem` serve a Playoffs Stage too (`pickem_stage` accepts Swiss and Playoffs; a Groups Stage still answers 404, so an Event without a Swiss Stage offers only the Playoffs Pick'em). Saved Pick'ems are `{shape: "playoffs", picks: [{rounds_before_final, slot, team_id}], updated_at}`; `PickemJson` is an enum tagged by `shape`. The GET also carries `bracket: {ready, slots: [{rounds_before_final, slot, team_a, team_b, leaf, winner}]}`; `teams` is the first-round teams. PUT takes `{picks: [...]}` (a body of the wrong shape for the Stage is 422 `pickem_wrong_count`). New code: 422 `pickem_bracket_not_ready`.
- **Bracket shape** (`domain::pickem::Bracket::of`) comes from the Stage's Matches' `Playoffs` placements: the deepest `rounds_before_final` R with a placed Match gives `2^R` first slots, and the tree runs from the final (0,0) down to there, so 4 teams = semis + final (3 slots), 8 = 7, 16 = 15. Winners of slots `2n`, `2n+1` meet in slot `n`. A slot with no Match in the data is still a slot with unknown teams. A shallower slot whose two feeders have no Match is also a first slot (a bye). Placements outside the tree (round > 7, slot out of range) and `Unplaced` Matches are ignored.
- **Picks storage**: kind `bracket`, `slot` = heap index `2^rounds_before_final + slot` (final 1, semis 2-3, quarterfinals 4-7), see `slot_id`.
- **Validation** (`BracketPickem::validate`): every slot exactly once (else `pickem_wrong_count`), each winner a participant of its slot: a first slot's two teams, otherwise the winners picked in its two feeder slots (else `pickem_team_not_in_stage`). A team may win several slots, so there is no duplicate rule.
- **TBD first-round teams**: while any first slot lacks a team the bracket is `ready: false`, PUT answers `pickem_bracket_not_ready` and the SPA shows a notice instead of the form. Lock is unaffected: it is still the Stage's first Match start, and `league_matches::evaluate` now stamps it for Playoffs Stages too. Teams are always read from the effective view, so a later poll that fills the slots opens the form (if not yet Locked).
- **Scoring** (`score_bracket`): 1 Point per slot whose picked winner equals the winner of that slot's finished, non-Void Match. It is in the same sum as the Swiss Stages in `GET .../pickem-leaderboard` and in each revealed Member's `points`.
- **Archive**: the PUT handler (`api/pickem.rs::save`) calls `require_writable(&league)` right after `require_member`, before either shape is validated, so both are read-only in an archived League.
- **SPA**: `pages/pickem/bracket.ts` is the pure logic (`setWinner` propagates a pick into the next round and clears dependent picks, `participants`, `isComplete`, `toPicks`/`fromPicks`), tested by `web/scripts/bracket.test.mjs` (`npm test`). `PlayoffsStage.tsx` renders it (form until Lock, everyone's read-only brackets after, with ✓/✗ once a slot is decided). `PickemPage` picks `PlayoffsStage` or `PickemStage` by Stage kind.
- Tests: `tests/api_pickem_playoffs.rs` (8-team bracket as the only Stage; 4, 16 teams; TBD first round; Event without a Swiss Stage). `tests/api_pickem.rs` no longer expects the Playoffs Stage to be 404.

## Scoring and Leaderboard (ticket #7)

- **Points are never stored.** The Leaderboard is computed on every read from `LeagueMatches` (the effective Match view + the League's Predictions), so a result from a poll, and #13's Overrides, are reflected at once with no recalculation step. Consequently `poller::after_poll` has no scoring hook; do not add one.
- **Pure policy** in `domain/scoring.rs`: `judge(&Match, Option<TeamId>) -> Verdict` (`Correct | Wrong | Missing | Void | Pending`; a Void Match is `Void` whatever was predicted), and `leaderboard(members, &[ScoredMatch { game, playoffs }], pick_of) -> Vec<LeaderboardRow>` with shared places (1, 1, 3). `LeaderboardRow { place, user_id, points, playoff_points, correct, wrong }` and `percent_correct()` (correct / (correct + wrong), rounded; Void, Pending and Missing are not in the denominator; `None` before any).
- **Reuse**: `api::leaderboard::ranking(&LeagueMatches, &[UserId]) -> Vec<LeaderboardRow>` ranks the League's Members over every Match. #16 "Points gained today": call `scoring::leaderboard` with only that day's Matches (it is the same function over a subset) and read `points`; the current place comes from `ranking`. #17 medals: map `LeaderboardRow::place` 1/2/3 to 🥇🥈🥉, shared places share the medal. Only pass current Members (`store::leagues::members`), so Members who left or were kicked drop off; late joiners simply have `Missing` for earlier Matches.
- Playoffs Matches are those whose Stage `kind` is `playoffs`.
- **API**: `GET /api/leagues/{id}/leaderboard` → `{ rows: [{ place, user_id, nickname, avatar_url, points, playoff_points, percent_correct|null }] }`, ordered by place then nickname. `MatchCard` gained `my_result` (`pending|correct|wrong|missing|void`) and `my_points`. `GET /api/leagues` items gained `my_place` (null if not ranked) and `my_points`.
- **SPA**: `/leagues/:id/leaderboard` (`web/src/pages/leaderboard/`, namespace `leaderboard`, place 1 gold, my row highlighted). #17 should render medals in this page. #14's Pick'em Leaderboard is separate.

## Archive (ticket #17)

- **The final** is the Playoffs Match with `rounds_before_final == 0`, finished with a winner, read from the League's *effective* Match view (`domain::league::final_decided`). A Void final (cancelled, or finished with no winner) does not archive. So when #13 sets a winner by Override in one League, only that League archives.
- **Archiving** is `leagues.archived_at`, set once by `store::leagues::archive` (`WHERE archived_at IS NULL`, never cleared, even if the final's data later changes). `poller::after_poll` calls `poller::archive_if_final_decided(state, league, matches, now)`; it is idempotent. **#13: call it right after an Override sets a winner** (with a freshly evaluated `LeagueMatches`) so the League archives at once instead of on the next tick. The poller already skips Events with only archived Leagues, and Locks need no more evaluation.
- **Read-only guard**: `api::leagues::require_writable(&league)` returns 409 `league_archived`. Call it right after `require_member`/`require_admin` (before reading the body or validating) in **every write that changes the game**. Applied today to: Prediction PUT, Pick'em PUT, join via Invite Link, kick, and every Override endpoint (#13; test `archived_leagues_reject_overrides` in `tests/api_overrides.rs`).
- **Still allowed after archiving**: leaving, deleting (creator), reissuing the Invite Link and changing roles (none of them changes a score). Joining and kicking are rejected so the final Leaderboard keeps its Members; leaving does remove the leaver from the Leaderboard, which is accepted.
- **Medals**: `web/src/components/Medal.tsx` renders 🥇🥈🥉 for places 1-3; shared places share a medal because places are already numbered 1, 1, 3. `LeaderboardPage` and `PickemPage` show it only when `GET /api/leagues/{id}` has `archived_at` set. The League page shows an archived banner and hides the Invite Link and Kick controls.
- Tests: `tests/api_archive.rs`.

## Overrides (ticket #13)

- **Where they apply**: `domain::effective_match::effective_view(base, &LeagueOverrides)` is the only place. `league_matches::effective_view_and_overrides` loads the inputs (`store::overrides::for_league`: the League's hand-added Matches + the Overrides in force) and `league_matches::evaluate` uses it, so Lock/Reveal, scoring, Pick'em, notifications and archive all follow Overrides with no extra step. `LeagueMatches.overrides` carries them for display. `league_matches::admin_view` is the same view **with** hidden Matches (what admins work on). `league_matches::effective_view` keeps its old signature.
- **Fields** (`domain::effective_match::Field`, re-exported as `store::overrides::Field`; `MatchOverrides::fields()` returns them): `scheduled_start` (replaces the Scheduled Start), `result` (winner → Finished with that winner; `winner` NULL → Void, i.e. Cancelled; the score is dropped unless it agrees), `teams` (team A / team B, chosen from the Event's teams: the League-scoped "fix a team mapping"; a source winner keeps its side), `hidden`, (in the effective view a winner is always one of the Match's two teams: a `result` Override whose winner no longer plays is not applied, and a stored winner who does not play makes the result absent, Finished turning Live), and `added` (history record of a hand-added Match).
- **Storage** (`migrations/1300_overrides.sql`): table `overrides` is an **append-only history** (`league_id` cascade, `match_id` without FK, value columns `scheduled_start`/`winner`/`team_a`/`team_b`, `cleared`, `author_id`, `created_at`). The Override in force for (match, field) is the latest row unless it has `cleared = 1` (`store::overrides::in_force`). A newer Override replaces an older one; removing writes a `cleared` row. Nothing is ever deleted, so the history keeps everything.
- **Hand-added Matches** are `matches` rows with a local negative id (`allocate_local_ids`) and the new column `matches.league_id` (cascade from `leagues`). `store::events::base_view` leaves them out (`league_id IS NULL`), so other Leagues, the poller's cadence and the fallback chain never see them: they are never linked to a fallback record nor adopted by HLTV. If the Data Sources later list the same Match, the League has both and an admin hides one. Their `source` column is `'manual'`; `MatchCard.source` is `null` for them.
- **Decisions**: an Override wins over every Data Source update until it is removed. ADR-0003 holds: a Scheduled Start Override moves a Lock that has not happened; stored Lock/Reveal stamps are never cleared (a hidden Match keeps its stamps and comes back with them). **Predictions after a teams Override**: a Prediction counts only while its team plays the Match in the effective view (filtered once in `league_matches::evaluate`); otherwise it is as good as none (no Reveal count, `missing` in scoring, not shown). The row is kept, so removing the Override restores it; while the Match is open the Member can predict again. A Match that already Revealed stays Locked.
- **API** (`api/overrides.rs`): `GET /api/leagues/{id}/overrides` (Members) → `{entries (newest first: id, match_id, match, field, cleared, scheduled_start, winner, void, team_a, team_b, author {user_id, nickname}, created_at), teams, stages, hidden}`; `POST /api/leagues/{id}/matches/{match_id}/overrides` (admins) body `{"field": "scheduled_start"|"result"|"teams"|"hidden", ...}` → 201 entry; `DELETE .../overrides/{field}` (admins) → 200 the `cleared` entry, 404 if not in force; `POST /api/leagues/{id}/matches` (admins) `{stage, team_a, team_b, scheduled_start, format?, placement?}` → 201 `{id}`; `GET /api/leagues/{id}/data-warnings` (admins only, 403 otherwise) → `{conflicts, unmatched}` of the Event, read-only. Every write calls `require_admin` then `require_writable`, evaluates the League at once and calls `poller::archive_if_final_decided`. Errors: `team_not_in_match`, `team_not_in_event`, `same_team`, `stage_not_found`, `override_invalid` (422). Nothing here calls `external_ids::set_manual` (it would change every League on the Event).
- `MatchCard` gained `overridden` (fields in force) and `added_by_hand`; `source` is now nullable.
- **Re-keying** (`rekey_match` / `rekey_team`) moves `overrides.match_id` / `winner` / `team_a` / `team_b`, and the `reminder:`/`reveal:` keys in `notification_log`, so a Reveal announced under a local id is not announced again after HLTV adopts the Match.
- **SPA**: `/leagues/:id/overrides` (`web/src/pages/overrides/`, namespace `overrides`): history for everyone; for admins of a non-archived League a "Correct a Match" form, the Overrides in force with Remove / Show again, "Add a Match by hand" and the warnings. Match cards show "Override: …" and "Added by a League Admin" (`matches` namespace).
- Tests: `tests/api_overrides.rs` (scoping, Lock and ADR-0003, source updates vs Overrides and removal, result/Void recalculation, archive at once, archived Leagues reject Overrides, admin-only, validation, hidden, hand-added, history, teams + Predictions, warnings gate, League deletion), `tests/override_rekey.rs` (Overrides follow a re-key, Reveal not announced twice, warnings with real conflict/unmatched data from a fallback chain).

## Accounts (ticket #3)

- API: `POST /api/auth/telegram` (body `{auth: <widget payload>, language?}`; `language` only seeds a new User), `POST /api/auth/logout` (204), `GET /api/auth/config`, `GET|PATCH /api/me` (`{nickname?, language?}`). Error codes: `invalid_login`, `login_expired` (401), `login_unavailable` (503), `nickname_invalid`, `language_invalid` (422), `nickname_taken` (409). Payloads older than 10 minutes are rejected.
- Nickname rules and the Telegram-name default live in `domain/user.rs` (`validate_nickname`, `nickname_from_name`, `nickname_attempt`). #16 adds its preference fields to the same `PATCH /api/me`.
- SPA: `web/src/pages/account/session.ts` is the shared store of the signed-in User (`useSession()` gives `{loading, user}`); it applies the saved language to the UI and saves language switches made while signed in. Pages needing a User redirect to `/login` when `user` is null. `components/Avatar` shows a User's avatar. Strings of this area are in the `account` namespace.
- Telegram Login Widget needs BotFather `/setdomain`; locally use the dev login form on `/login` (shown when `KALLBET_DEV_LOGIN=true`).

## Notifications (ticket #16)

- **Entry point**: `notifications::after_poll(state, league, matches, now)`, the one call #16 makes inside `poller::after_poll`. **Put it before #17's archive call**: a League archived at the final stops being passed to `after_poll`, and the final's match day digest must go out in the same pass that archives (an archived League sends nothing further; the function also returns at once for `archived_at.is_some()`).
- **Layers**: `notifications/due.rs` decides what is due, pure, from `LeagueMatches` (the effective view; Overrides apply once #13 feeds them in). `notifications/messages.rs` holds every text, in en/ru/uk, in one catalog (`key -> template with {placeholders}`, no plural forms: a number follows a label). `notifications/mod.rs` does dedupe, preferences, rendering in the User's language and the send. Adding a message: a key constant and a template in all three catalogs; unit tests fail if the languages differ in keys or placeholders. Texts are plain (no Telegram parse mode), so names need no escaping.
- **Dedupe**: table `notification_log (key PK, league_id, user_id, sent_at)`, cascading from leagues and users, so League deletion keeps working. Keys: `reminder:<league>:<match>:<user>`, `reveal:<league>:<match>:<user>`, `digest:<league>:<UTC date>:<user>`. Everything is per League, so a Member of two Leagues on one Event gets a message per League, with the League's name in it.
- **Port change**: `Notifier::send` returns `Result<Delivery, NotifyError>`. `Delivery::Sent`, or `Delivery::Undeliverable` (the User never started the bot or blocked it, Telegram does not know the chat, or no bot is configured): not an error, and the log entry **is written**, so the poller does not retry forever. `NotifyError` is a passing failure (network, 429, 5xx, bad token): **no** log entry, the next 20-second pass tries again, and that is bounded because each type has a time window (below). A turned-off type writes no entry either.
- **Production adapter**: `ports/telegram.rs` `TelegramNotifier` (reqwest + rustls): `POST https://api.telegram.org/bot<BOT_TOKEN>/sendMessage {chat_id, text}`; 403 and 400 "chat not found" are `Undeliverable`; no `BOT_TOKEN` sends nothing. The token is never logged (errors drop the URL). Tested against a local listener (`tests/telegram_notifier.rs`); never against Telegram itself. Dev-login Users have negative Telegram ids, so the real bot answers "chat not found" for them.
- **Preferences**: columns `users.notify_reminder|reveal|digest` (default on). `GET /api/me` has `notifications: {reminder, reveal, digest}`; `PATCH /api/me` takes the same object, partial (`{"notifications": {"digest": false}}`). SPA: three checkboxes on the profile page (`account` namespace).
- **Reminder**: for each Match that is Upcoming, not Locked, with both teams known, from Lock minus 30 min (`REMINDER_LEAD`; Lock is the League's effective one from the Lock/Reveal policy) until Lock, to every Eligible Member without a Prediction. A Match the poller first sees inside the window still gets a **late reminder** (the text says the real minutes left): a late reminder beats none while the Match is open. The key has no Lock time, so a postponed Match does not remind the same Member twice.
- **Reveal**: announced for every Match whose stored `revealed_at` is at most 3 h old (`REVEAL_ANNOUNCE_WINDOW`), which is what stops a burst of old Reveals when the feature first runs on existing data (or after long downtime). Recipients are the Eligible Members (`outcome.eligible`), so a Member who joined after the Lock, or left, is not told, and a Member who did not predict is. Skipped: Void or cancelled Matches, and Matches nobody predicted. The text carries the split of the Eligible Members' picks ("NAVI 2 : 1 FaZe"), which is already public after Reveal.
- **Match day**: the UTC date of a Match's start, where start is the earlier of its actual start and its current Scheduled Start (so a postponed Match moves to its new day, an early start counts early, and a Match the poller saw late, whose actual start was stamped "now", stays on its scheduled day). KallBet stores no User time zone, so one fixed zone keeps the rule stable. **Daily digest** for a day: sent once all its Matches are done (Finished or Cancelled; a Match whose start is over 12 h ago without a result counts as done, `STALE_AFTER_START`), at least one scored (Finished, non-Void) Match, and now is within 6 h after the day's end (`DIGEST_GRACE`, so history is not announced). Recipients: current Members who joined before the day ended. Text: Points gained over that day's Matches (`scoring::leaderboard` over the day's Matches), the current place and total from `api::leaderboard::ranking` at that moment.
- Tests: `tests/api_notifications.rs` (Seam 1, with the fake clock and `RecordingNotifier`), unit tests in `notifications/messages.rs`, `tests/telegram_notifier.rs`. `RecordingNotifier::make_unreachable(user)` makes a User answer `Undeliverable`; `undeliverable()` lists those attempts; `fail_with` makes sends a passing failure.
