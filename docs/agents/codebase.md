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
- **Vocabulary** in names, routes, JSON and UI text follows `GLOSSARY.md`.

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

- `TestApp::new()`, or `TestApp::builder()` with `.dev_login(false)`, `.web_dir(..)`, `.config(|c| ..)`, `.source(arc)` (to put a `FallbackSource` of fakes behind the port), then `.build()`.
- `app.user("nickname")`: a client signed in as that User (created on first use). `app.anonymous()`: a client with no session. Clients have `get`, `post`, `post_empty`, `put`, `patch`, `delete`. They keep cookies like a browser.
- `TestResponse`: `status`, `json()`, `json_as::<T>()`, `text()`, `header(name)`, `error_code()`.
- `app.clock` (`FakeClock`): `now()`, `advance(duration)`, `set(instant)`. It starts at 2026-01-10 12:00 UTC and only moves when told to.
- `app.source` (`ScriptedSource`): `put_event`, `update_event`, `update_match`, `postpone`, `start`, `finish`, `finish_without_winner`, `cancel`, `fail_with` / `recover`, and the call counters `list_events_calls()`, `fetch_event_calls(id)`, `total_fetch_event_calls()`. `ScriptedSource::of(kind)` makes one that reports as another Data Source.
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

Seam 2 (parser) tests keep fixtures under `tests/fixtures/<source>/` and feed them to the pure parser. No network.

## Frontend map (`web/`)

| Path | Holds |
|---|---|
| `src/pages/<area>/routes.tsx` | Default-exports the area's `RouteObject[]`. Collected automatically by `src/app/router.tsx`. |
| `src/pages/<area>/` | The area's pages, components and API calls (`api.ts`). |
| `src/locales/<lang>/<namespace>.json` | Translations, one namespace per area. Loaded automatically by `src/i18n/index.ts`. |
| `src/components/` | Shared pieces: `Logo`, `LanguageSwitcher`, `HeaderUser` (the header's User corner), `ServerStatus`. |
| `src/app/Layout.tsx` | Header, page outlet, footer. |
| `src/api/client.ts` | `api<T>(path, {method, body})`, `ApiError` (`status`, `code`), `errorMessage(t, error)`. |
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

A new setting needs a field in `Config`, a line in `Config::from_lookup`, a value in `Config::for_tests`, and an entry in `.env.example`. `Config`'s `Debug` output redacts secrets; keep it that way for new ones.

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
- **Rule for #5 and #13 reviewers: no API path lets anyone create or change another Member's Prediction.** A Prediction handler takes the User only from `CurrentUser`, never from the path or body, and writes only for that User. No admin/creator override exists. #5 must add the test (an admin and a creator trying to write for another Member get no way to do so); it could not be written in #12 because Predictions did not exist.

## Matches, Lock and the poller (ticket #5)

How League logic gets its Matches — read this before #6, #7, #13, #14, #16, #17:

- **One entry point**: `league_matches::evaluate(db, &league, now) -> LeagueMatches`. It loads the League's **effective Match view**, its membership stretches, all its Predictions and stored stamps, runs the Lock/Reveal policy for every Match, **persists new stamps**, and returns `{ view, outcomes, predictions, members }`. The Matches API, the Prediction endpoint and the poller all call it. Never read `store::events::base_view` (raw source rows) from League logic.
- **Effective Match view**: `domain::effective_match::effective_view(base) -> EventView` is pure and currently the identity. `EventView { event, stages, teams, matches: Vec<SourcedMatch { data: Match, source, updated_at }> }`. #13: add the League's Overrides as a parameter there and load them in `league_matches::effective_view` (the only caller). Hidden Matches simply drop out of `matches`; manually added ones are appended.
- **Lock/Reveal policy**: `domain::lock_reveal::evaluate(&Inputs { game, members, predicted, stored, now }) -> Outcome { stamps, lock_time, transitions }`. Lock = earliest of Scheduled Start − 15 min, actual start (or "now" if started without a time) and cancellation, once it is ≤ now; once stored it never changes. `members` (every `MemberSpan { user_id, joined_at, left_at }`) and `predicted` (users with a Prediction) are already passed in and unused: **#6 adds Reveal inside `evaluate`** (emit `Transition::Revealed`, and a Reveal also sets `locked_at`). `store::predictions::record` persists transitions with COALESCE, so a stamp is never overwritten.
- **Schema** (`migrations/0500_matches_and_predictions.sql`): `stages`, `teams`, `matches` (base data; `placement_kind` + `swiss_*` / `playoff_*` columns; `source`, `updated_at` = last successful poll), `polls` (cadence bookkeeping), `predictions (league_id, match_id, user_id, team_id, updated_at)`, `match_locks (league_id, match_id, locked_at, revealed_at)`.
- **Poller** (`poller.rs`): `poller::tick(&state)` is one pass at `clock.now()`; production runs it every 20 s (`runtime::spawn_background_tasks`). Order: evaluate every active League (stores Locks due by the data we had, before a postponement can move them) → poll the Event list (6 h) → poll each Event with a non-archived League (5 min, 1 min if a Match is live or starts within 45 min) → evaluate every active League again and call `after_poll(state, league, matches, now)`. **#7, #16, #17 each add one call inside `after_poll`.** It runs on every tick, not only when data changed, so make those hooks idempotent (scores recomputed, notifications deduped). A Match first seen live without a source `actual_start` gets `now` stamped.
- **API**: `GET /api/leagues/{id}/matches` → `{ event, stages: [{ ordinal, kind, name, groups: [{ kind: "record", wins, losses | kind: "playoff_round", rounds_before_final | kind: "unplaced", matches: [MatchCard] }] }] }`. `MatchCard` (`api/matches.rs`) has `team_a/team_b {id,name}|null`, `format`, `scheduled_start`, `actual_start`, `status`, `winner`, `score`, `lock { locks_at, locked_at, locked }`, `my_prediction`, `source`, `updated_at`. **#6** adds its `predictions` field (count, M, revealed list) and **#7** the caller's Points/correctness to `MatchCard` (marked by comments); build them from `LeagueMatches` in `view_for`. Never put other Members' Predictions in it before Reveal.
- `PUT /api/leagues/{id}/matches/{match_id}/prediction` `{team_id}` → `{match_id, user_id, team_id, updated_at}`; errors 404 `not_found`, 409 `match_locked`, 422 `team_not_in_match`. It evaluates before (reject if locked) and after the upsert (where #6's Reveal on the last Prediction fires). #17 adds the archived check here.
- **Pick'em (#14)**: a Stage's Matches and its first start come from `LeagueMatches.view` (`EventView::matches` filtered by `data.stage`); its Lock is the first `actual_start`/live status in the Stage.
- **SPA**: `/leagues/:id/matches` (`web/src/pages/matches/`). `MatchCard.tsx` has a marked slot for #6 ("N of M predicted", revealed avatars); #7's ✓/✗ goes next to the score. Strings in the `matches` namespace. The League page links to it.
- Local manual testing: the production `MatchSource` is still unconfigured until #9, so a fresh League shows no Matches unless the DB is seeded by hand.

## Accounts (ticket #3)

- API: `POST /api/auth/telegram` (body `{auth: <widget payload>, language?}`; `language` only seeds a new User), `POST /api/auth/logout` (204), `GET /api/auth/config`, `GET|PATCH /api/me` (`{nickname?, language?}`). Error codes: `invalid_login`, `login_expired` (401), `login_unavailable` (503), `nickname_invalid`, `language_invalid` (422), `nickname_taken` (409). Payloads older than 10 minutes are rejected.
- Nickname rules and the Telegram-name default live in `domain/user.rs` (`validate_nickname`, `nickname_from_name`, `nickname_attempt`). #16 adds its preference fields to the same `PATCH /api/me`.
- SPA: `web/src/pages/account/session.ts` is the shared store of the signed-in User (`useSession()` gives `{loading, user}`); it applies the saved language to the UI and saves language switches made while signed in. Pages needing a User redirect to `/login` when `user` is null. `components/Avatar` shows a User's avatar. Strings of this area are in the `account` namespace.
- Telegram Login Widget needs BotFather `/setdomain`; locally use the dev login form on `/login` (shown when `KALLBET_DEV_LOGIN=true`).
