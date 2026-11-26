# Signal Lab Data Research: Point-in-Time Map, Veto, Player, and Round/Economy Data

**Ticket:** #36 (parent spec: #33)  
**Date checked:** 2026-11-26  
**Saved real evidence:** `sidecar/tests/fixtures/research/` (provenance in `sidecar/tests/fixtures/research/README.md`)

This document records what each candidate source actually returns for archived CS2 Matches, at what request cost, under which terms of use, and whether it can be used point-in-time by #44 (map-level Candidate Signals) and #45 (player-level Candidate Signals).

---

## 1. Summary Matrix (checked 2026-11-26)

| Source | Map names & per-map results | Veto order (picks/bans) | Player statistics | Round & economy data | Request cost | Point-in-time usable? | Terms / constraints |
|---|---|---|---|---|---|---|---|
| **PandaScore** (current Free plan) | **No** (HTTP 403 on `/csgo/matches/{id}` and `/csgo/games/{id}`; `/tournaments/{id}/matches` has slim `games` with winner/duration only) | **No** | **No** (HTTP 403 on `/csgo/teams/{id}/stats`, `/csgo/games/{id}`) | **No** | 1,000 req/hr free; detail endpoints require paid Historical/Live plan (~€400+/mo, out of scope) | N/A (not returned) | Free plan = fixtures/schedules/results only. |
| **Liquipedia** (post-Match Stage revisions, MediaWiki API) | **Yes** — map names, half-by-half T/CT & overtime round scores, `t1firstside`, skipped maps, `hltv=` match ID, `stats=` mapstatsid | **No** (0 veto templates on CS2 Stage/Playoffs pages) | **No** (rosters only, via `{{TeamParticipants}}`/`{{TeamCard}}`; no per-Match scoreboards) | **No** (half totals only, no round-by-round sequence or economy) | **1 req / Stage page** (33 Swiss or 7 Playoffs Matches per request) | **Yes** — for Matches finished strictly before the evaluated Match's cutoff | Free; [Liquipedia API Terms](https://liquipedia.net/api-terms-of-use): ≤ 1 req / 2 s, custom `User-Agent`, `gzip`, no `action=parse`, CC-BY-SA 3.0. |
| **Public datasets** (Kaggle, GitHub) | Partial (static third-party scrapes; uneven 2024–2026 coverage) | Rarely / inconsistent | Partial (often aggregated or stale) | Only in demo-parsed repos (e.g. `jankasen/cs2`, subset of LAN events) | 1 bulk download (manual) | Unreliable (no reproducible audit trail; missing recent/Holdout events) | Upstream HLTV-scraped data lacks clear redistribution license; cannot be automatically audited or extended for Holdout. |
| **GRID Open Access** | **Yes** (for GRID-partnered tournaments only) | **Partial** (where logged in series state) | **Yes** (GRID-partnered tournaments only) | **Yes** (server-direct JSONL telemetry: round win conditions, starting cash, buys, loss bonus) | Free tier API / File Download (rate-capped) | **Yes** (historical series files) | Requires operator application (`grid.gg/open-platform`); non-commercial/pre-revenue only; **excludes exclusive tier-1 commercial rights** (Majors, ESL, BLAST). |
| **HLTV** (iOS-app plain HTTP via `wreq` `SafariIos17_2`) | **Yes** on `/matches/*` (map names, final & half scores, `mapstatsid`) | **Yes** on `/matches/*` (full 7-step Bo1/Bo3/Bo5 veto sequence) | **Yes** on `/matches/*` (series & per-map: K-D, eK-eD, Swing, ADR, eADR, KAST, eKAST, Rating 3.0) | **No** (`/stats/matches/mapstatsid/*` returned **403 Cloudflare challenge**; stopped immediately) | 1 req / Match (unproven at volume; stopped after 5 total requests) | **Yes** (each Match carries Unix ms start timestamp) | **HLTV ToS §2.2 explicitly prohibits web scraping and data mining.** Operator decision required before any further collection. |

---

## 2. Source-by-Source Findings

### 2.1 PandaScore (current plan)

**Checked:** 2026-11-26, using the operator's `PANDASCORE_TOKEN` (in memory only, never saved) against `https://api.pandascore.co` on finished IEM Cologne Major 2026 Stage 1 Match `1508646` (NRG vs BIG, 2026-06-05) and Game `219490`.

- `GET /tournaments/20708/matches?page[size]=100` (already saved in `tests/fixtures/pandascore/real-major-stage1-matches.json`): **HTTP 200**. Each match includes a `games` array (`id`, `position`, `status`, `begin_at`, `end_at`, `length`, `winner`, `detailed_stats: true`), but **no map name, no round score, no veto, and no player statistics**.
- `GET /csgo/matches/1508646`: **HTTP 403** — `{"error":"Access Denied","message":"You are not authorized to access this page."}` (saved: `sidecar/tests/fixtures/research/pandascore-csgo-match-1508646-403.json`).
- `GET /csgo/games/219490`: **HTTP 403** — `{"error":"Access Denied","message":"You are not authorized to access this page."}` (saved: `sidecar/tests/fixtures/research/pandascore-csgo-game-219490-403.json`).
- `GET /csgo/teams/3249/stats`: **HTTP 403** — `{"error":"Access Denied","message":"You are not authorized to access this page."}` (saved: `sidecar/tests/fixtures/research/pandascore-csgo-team-3249-stats-403.json`).
- `GET /csgo/matches/1508646/stats`: **HTTP 404** — `{"error":"Route not found"}`.

**Conclusion:** On the current Free plan, PandaScore provides **none** of the four target data kinds. Unlocking CS2 map/round/player endpoints requires PandaScore's paid Historical or Live plan, which is out of scope for spec #33.

---

### 2.2 Liquipedia (post-Match Stage revisions)

**Checked:** 2026-11-26 via `https://liquipedia.net/counterstrike/api.php` (`action=query&prop=revisions&rvprop=ids|timestamp|content&rvslots=main&rvlimit=1&formatversion=2&format=json`, `User-Agent: KallBet/0.1 (https://kallbet.kulunchick.tech)`, `Accept-Encoding: gzip`, ≥ 2 s spacing), plus existing saved fixtures (`tests/fixtures/liquipedia/api-query-cologne-pages.json` and `sidecar/tests/fixtures/backtest/lp-*.json.gz`).

**Real saved evidence:** `sidecar/tests/fixtures/research/liquipedia-cologne-playoffs-postmatch.json` (`Intel Extreme Masters/2026/Cologne/Playoffs`, revid `3390154`, timestamp `2026-06-22T02:25:54Z`, 6,128 bytes). Also checked `Intel Extreme Masters/2026/Cologne/Stage 3` (revid `3413157`, 33/33 matches) and `StarLadder/2025/Major/Stage 1` (revid `3296139`, 33/33 matches).

What post-Match Liquipedia `{{Match}}` blocks carry:
- **Map names & results (YES):** Every played map (`|map1={{Map|map=Overpass|finished=true|t1firstside=ct|t1t=2|t1ct=7|t2t=5|t2ct=8|stats=231751|...}}`) plus overtime halves (`o1t1t`, `o1t1ct`, ..., up to 4x OT verified on Spirit vs G2 Map 3 Mirage, 25–22) and unplayed/skipped maps (`|map4={{Map|map=Dust II|finished=skip}}`).
- **Canonical cross-links (YES):** `|hltv=<match_id>` (33/33 on Cologne Stage 1, 33/33 on Stage 3, 7/7 on Playoffs, 33/33 on Budapest 2025 Stage 1) and `|stats=<mapstatsid>`.
- **Veto order (NO):** Zero `{{MapVeto}}` or `veto` parameters appear across all checked CS2 Stage and Playoffs pages. Skipped maps in Bo3/Bo5 show which maps were left unplayed, but neither ban order nor which team picked which map is recorded.
- **Player statistics (NO):** Individual match scoreboards are not embedded in wikitext (only linked out via `|stats=<mapstatsid>`).
- **Round & economy data (NO):** Half-by-half totals only.

**Point-in-time validity:** Map results of a completed Match are immutable historical facts once that Match ends. Filtering strictly to Matches whose finish time (`end_at`, or `scheduled_start + safe margin` when only start is known) is `< evaluated_match_cutoff` ensures zero temporal leakage. Unlike pre-Match roster lookups, collecting per-map history requires only **1 API call per completed Stage page** (covering all 7–33 Matches of that Stage), making it trivially affordable under Liquipedia's 1 req / 2 s limit.

---

### 2.3 Ready-Made Public Datasets & GRID Open Access

#### Public datasets (checked 2026-11-26)
- **Kaggle — "Counter-Strike Pro Matches"** (`cs2_all_tiers_games.csv`, `cs2_tier1_games.csv`): ~19k CS2 games with match timestamps, rosters, per-player K/D/A/ADR/KAST, map names, and round scores. Static third-party HLTV scrape; unclear redistribution rights; does not reliably include Bo1 veto sequences or live-updated Holdout events (e.g. PGL Major Singapore 2026).
- **Kaggle — "Counter-Strike Competitive Scene (2012–2026)"**: Historical tournaments, monthly rankings, and player stats; lacks per-Match veto order and reproducible request-level provenance.
- **GitHub — `jankasen/cs2`**: ETL pipeline parsing CS2 `.dem` demo files and HLTV boxscores into a ~472 MB Parquet lakehouse (`event_{id}/match_{id}/{map_name}/`) with rich combat/round/utility telemetry and HLTV Rating/ADR/KAST. However, it covers a curated subset of tier-1 LANs, omits veto sequences, and relies on its own Cloudflare-bypassing scraper.
- **GitHub — `Jamie-Packer/cs2-playstyle-roles`**: 2024 single-CSV playstyle/role metrics parsed via `awpy`; does not cover 2025–2026 archive or Holdout.

**Conclusion:** Ready-made datasets do not meet spec #33's requirements for reproducible provenance, complete coverage of all tier-S/A events through the Singapore 2026 Holdout, and clean licensing.

#### GRID Open Access (checked 2026-11-26)
- **What it carries:** Official game-server-direct telemetry via GRID's Central Data Feed and File Download API (historical JSONL series state & event files), including map names, per-map results, individual player statistics, and **full round-by-round + economy data** (starting cash, equipment/buy values, loss-streak counters, win conditions, bomb/kill events).
- **Coverage limitation:** GRID Open Access covers GRID-partnered circuits (most notably the Champion of Champions Tour / CCT and select partnered events) and **excludes exclusive commercial tier-1 rights** (Valve Majors, ESL Pro Tour, BLAST Premier main events). Consequently, it cannot cover our primary tier-S/A Major & top-LAN archive on its own.
- **Terms:** Free for independent developers, students, researchers, and pre-revenue non-commercial projects. Strictly forbids commercial monetization, betting products, and raw data redistribution outside the declared Purpose. Rights are sublicensed from rights holders and revocable.
- **What the operator should file (not filed during this ticket):**
  1. Go to `https://www.grid.gg/open-platform` and open the Open Access registration form in the operator's own name.
  2. Provide developer contact / identity details and repository reference (`Kulunchick/kallbet`).
  3. **Purpose of Use:** State explicitly: *"Non-commercial, zero-money personal research and retrospective statistical backtesting of CS2 match forecasting signals for a private friends-only prediction game (KallBet Signal Lab). No wagering, no commercial product, no redistribution of raw GRID feeds."*
  4. **Requested scope:** Counter-Strike 2 historical series state / File Download API (no live commercial feed needed).
  5. Expect approval in hours to a few days for non-commercial research; once approved, verify which tier-S/A series IDs in 2024–2026 are actually unlocked on the Open Access key before building any round/economy signal.

---

### 2.4 HLTV — Operator's iOS-App Plain-HTTP Approach

#### Rust crate evaluation (checked 2026-11-26)
| Candidate | Status (2026-11) | iOS Safari TLS + HTTP/2 profile? | Windows MSVC build | Alpine `musl` (`Dockerfile.sidecar`) | Verdict |
|---|---|---|---|---|---|
| **`wreq` `6.0.0-rc.31` + `wreq-util` `3.0.0-rc.14`** | Actively maintained (successor to yanked `rquest 0.0.0`); patched BoringSSL (`btls-sys 0.5.6`) | **Yes** (`Profile::SafariIos16_5`, `SafariIos17_2`, `SafariIos17_4_1`, `SafariIos18_1_1`, `SafariIos26`, `SafariIos26_2`), with `.headers(false)` so Alamofire headers are sent without Safari `sec-fetch-*` browser headers | **Verified builds** on `x86_64-pc-windows-msvc` with VS Build Tools (`cmake`, `cl`), LLVM (`libclang`), Git `perl`, and a 2-line CMake toolchain setting `OPENSSL_NO_ASM=ON` when `nasm` is absent (plus a short `CARGO_TARGET_DIR` to stay under Windows 260-char MAX_PATH) | **Kept out of default build** behind optional Cargo feature `hltv-impersonate` (`default = []`), so `Dockerfile.sidecar` (`cargo build -p kallbet-ai --release --locked`) is unchanged. Building *with* the feature on Alpine musl requires `build-base cmake perl clang-dev llvm-dev linux-headers` | **Chosen** (optional feature only) |
| `rquest` | Yanked (`0.0.0`) | Superseded by `wreq` | N/A | N/A | Rejected |
| `ratcurl` `0.1.0` | Minimal (`curl-sys`) | Chrome / Firefox only (no iOS Safari) | Requires libcurl + OpenSSL | Heavy C deps + OpenSSL | Rejected |
| `curl-impersonate-cli` `0.4.0` / `cuimp` `0.1.2` / `impersonate-rs` `0.1.3` | Thin subprocess/FFI wrappers | Depends on external `curl-impersonate` binary | Requires downloading/bundling external binaries | External glibc vs musl binary mismatch risk | Rejected |

**Verified TLS/HTTP2 fingerprint** (checked once against `https://tls.peet.ws/api/all` before any HLTV request, using `Profile::SafariIos17_2`, `Platform::IOS`, `.headers(false)`, UA `hltv/2.13.2 (org.hltv; build:1; iOS 17.2.1) Alamofire/4.9.1`):
- `http_version`: `h2`
- `ja3_hash`: `411afdbd1e2f89ae92225e114fc2be0d`
- `ja4`: `t13d2014h2_a09f3c656075_14788d8d241b`
- `akamai_fingerprint`: `2:0;4:2097152;3:100|10485760|0|m,s,p,a`

#### Probe log (2026-11-26, operator's workstation, isolated client, no Chrome, no shared cookies, 5 s spacing)

Fixed User-Agent for all 5 requests: `hltv/2.13.2 (org.hltv; build:1; iOS 17.2.1) Alamofire/4.9.1` with `Profile::SafariIos17_2`.

| # | URL | HTTP status | `cf-mitigated` | Challenge? | Raw bytes | Saved fixture | Observation |
|---|---|---|---|---|---|---|---|
| 1 | `https://www.hltv.org/terms` | **200** (`HTTP/2.0`) | *(empty)* | No | 162,162 | `hltv-terms.html.gz` | Full Terms of Service page. |
| 2 | `https://www.hltv.org/matches/2395002/match` | **301** (`HTTP/2.0`) | *(empty)* | No | 0 | *(redirect only)* | Canonical slug redirect (not a block). |
| 3 | `https://www.hltv.org/matches/2394898/vitality-vs-fut-iem-cologne-major-2026` | **200** (`HTTP/2.0`) | *(empty)* | No | 881,745 | `hltv-match-2394898-vitality-vs-fut.html.gz` | Full finished Bo3 page: 3 maps (`Anubis` 5–13, `Overpass` 13–2, `Nuke` 13–8) with half scores & `mapstatsid`s, complete 7-step Bo3 veto, series + per-map player stats (K-D, eK-eD, Swing, ADR, eADR, KAST, eKAST, Rating 3.0). |
| 4 | `https://www.hltv.org/matches/2394774/heroic-vs-sharks-iem-cologne-major-2026-stage-1` | **200** (`HTTP/2.0`) | *(empty)* | No | 495,932 | `hltv-match-2394774-heroic-vs-sharks.html.gz` | Full finished Bo1 page: `Nuke` 10–13 with half scores & `mapstatsid`, complete 7-step Bo1 veto, series & per-map player stats. |
| 5 | `https://www.hltv.org/stats/matches/mapstatsid/230770/heroic-vs-sharks` | **403** (`HTTP/2.0`) | `challenge` | **Yes** (`Just a moment...`, `_cf_chl_opt`) | 5,664 | `hltv-mapstats-230770-403-challenge.html` | **Stop rule triggered.** All HLTV probing ended immediately. |

- **Total HLTV requests made:** **5**
- **Not tested (ended by stop rule):**
  - Fixed iOS version per run vs. random iOS version per request comparison
  - `/results?event=...`, `/events/...`, and any internal mobile/JSON endpoints
  - Sustained volume (dozens or hundreds of `/matches/*` pages)

#### HLTV Terms of Service (read directly from `https://www.hltv.org/terms`, Effective 27th January 2025)
Provided by HLTV.org A/S (CVR 27652913, Copenhagen, Denmark):
- **§2.1:** Grants only a personal, limited, non-exclusive, revocable, non-transferable license to access and use the Website.
- **§2.2:** *"You are not allowed to: commercially exploit the Website or any of its content in any manner; modify in any manner any part of the Website or content, except for Your Content as described below; **conduct, facilitate or organize data mining or web scraping in relation to the Website and/or any of the content**; or construct and build a similar or competitive website, product, or service."*
- **§3.1–3.2:** All original content is protected by IP law and may not be used for business purposes without a license.

#### HLTV Plain Verdict
**A limited technical method exists for `/matches/*` pages only, unproven at volume.**
- **What works:** On 2026-11-26, `wreq` (`Profile::SafariIos17_2`) with the iOS Alamofire User-Agent retrieved full HTML for `/terms` and two historical `/matches/<id>/<slug>` pages (Bo3 and Bo1) without triggering a Cloudflare challenge. One request per Match provides map names, per-map scores (with halves), full veto order, and series + per-map player statistics (including Rating 3.0 and economy-adjusted metrics).
- **Limits:**
  1. `/stats/*` (e.g. `/stats/matches/mapstatsid/*`, economy/round-history pages) returned **403 `cf-mitigated: challenge`** immediately, so round-by-round and per-round economy data cannot be collected by this method.
  2. Volume tolerance, `/results` listing pages, mobile JSON endpoints, and fixed-vs-per-request UA behaviour were **not tested** because the stop rule halted the probe on request #5.
  3. **Legal/ToS constraint:** HLTV Terms of Service §2.2 explicitly prohibits web scraping and data mining. Any decision to run bulk historical collection beyond this 5-request probe belongs strictly to the operator, from a non-server IP, using the cache, rate limit, and hard stop rule in `kallbet-ai hltv-collect` (`--features hltv-impersonate`).

---

## 3. Collector Command & Build Isolation

- **Modules (`sidecar/src/adapters/hltv_research/`):**
  - `archive.rs`: typed JSON archive schema (`HltvMatchRecord`, `MapResult`, `VetoStep`, `PlayerMatchStats`, `LiquipediaStageMaps`) for #44 and #45.
  - `parse.rs`: pure `scraper`-based HTML parser for HLTV `/matches/<id>/<slug>` pages + Cloudflare challenge detector (tested in `sidecar/tests/hltv_research_parser.rs` on real saved fixtures).
  - `liquipedia_maps.rs`: pure wikitext parser for post-Match Liquipedia Stage pages (map names, half & overtime round totals, skipped maps, `hltv=` and `stats=` IDs; tested on `liquipedia-cologne-playoffs-postmatch.json`).
  - `collector.rs` & `cli.rs`: `kallbet-ai hltv-collect --match-id <ID> --url <URL> --out <DIR> [--min-interval-secs 5] [--ios-version 17.2.1]` — caches raw `.html.gz` and parsed `.json` with `sources.tsv` provenance, enforces rate limiting via injectable `Clock`/`Pause`, and halts permanently on any HTTP 403, HTTP 429, `cf-mitigated` header, or Cloudflare challenge body (tested against a local `axum` stand-in server in `sidecar/tests/hltv_research_collector.rs`).
- **Build isolation:** `wreq` and `wreq-util` are optional dependencies behind `sidecar`'s `hltv-impersonate` feature (`default = []`). Neither `kallbet` (the server crate) nor `kallbet-ai`'s default build (including `Dockerfile.sidecar` on Alpine musl) depends on `wreq`, `wreq-util`, or `btls-sys`. In the default build, `hltv-collect` refuses remote `hltv.org` URLs rather than sending a mismatched rustls fingerprint.

---

## 4. Recommendations for #44 (Maps) and #45 (Players)

### Ticket #44 — Map-level Candidate Signals (`per_map_rating` and `bo1_veto_sim`)

1. **`per_map_rating` (per-map team strength): BUILD using Liquipedia post-Match Stage revisions (recommended primary source).**
   - Liquipedia post-Match revisions already cover every completed S-tier and A-tier Event Stage in 1 API request per Stage page under the sidecar's existing Liquipedia API terms (CC-BY-SA 3.0, ≤ 1 req/2 s, `gzip`).
   - `hltv_research::liquipedia_maps::parse_stage_maps` extracts map names, winners, half-by-half + overtime round scores, skipped maps, and HLTV match IDs without touching HLTV or paying for PandaScore.
   - Strict point-in-time guard: feed a Match's map results into the walk-forward per-map rating only when that Match finished strictly before the evaluated Match's cutoff.
2. **`bo1_veto_sim` (Bo1 map veto simulation): `not testable` by default; testable ONLY if operator authorizes offline HLTV `/matches/*` collection.**
   - Neither Liquipedia nor Free PandaScore records map ban/pick veto sequences.
   - HLTV `/matches/<id>/<slug>` does carry full 7-step vetoes (reachable via the `hltv=` match IDs already on Liquipedia), and `hltv-collect` + `parse_match_page` parse them into `HltvMatchRecord.veto`.
   - **Recommendation:** Unless the operator explicitly signs off on offline workstation collection from HLTV `/matches/*` (given HLTV ToS §2.2 and unproven volume stability), #44 should implement and evaluate `per_map_rating` from Liquipedia and register `bo1_veto_sim` as **`not testable`** with reason: *"No free ToS-compliant source carries CS2 map ban/pick veto order (Liquipedia records played/skipped maps only; PandaScore gates CS2 detail endpoints behind 403 paid tiers; HLTV ToS §2.2 prohibits automated scraping)."*

### Ticket #45 — Player-level Candidate Signals (`lineup_strength` and `player_form`)

- **Recommendation: `not testable` by default; testable ONLY if operator authorizes offline HLTV `/matches/*` collection.**
- **Why:**
  - PandaScore Free plan returns `403 Access Denied` on `/csgo/matches/{id}`, `/csgo/games/{id}`, and `/csgo/teams/{id}/stats`.
  - Liquipedia post-Match Stage pages carry rosters (`{{TeamParticipants}}`/`{{TeamCard}}`) but no per-Match player statistics.
  - GRID Open Access requires an operator application and excludes exclusive commercial tier-1 rights (Majors, ESL, BLAST), so it cannot cover the tier-S/A archive even after approval.
  - HLTV `/matches/<id>/<slug>` carries complete point-in-time series & per-map player statistics (`K-D`, `eK-eD`, `Swing`, `ADR`, `eADR`, `KAST`, `eKAST`, `Rating 3.0`) in one request per Match, parsed by `hltv_research::parse::parse_match_page`, **but** bulk collection is gated on an explicit operator decision regarding HLTV ToS §2.2 and unproven multi-hundred-request volume behavior.
- **If the operator does NOT authorize HLTV bulk collection:** #45 should register both `lineup_strength` and `player_form` as **`not testable`** with the reason above and build no model.
- **If the operator DOES authorize offline HLTV `/matches/*` collection:** #45 can consume `HltvMatchRecord` JSON files produced by `kallbet-ai hltv-collect` (built with `--features hltv-impersonate`, run slowly from the workstation, never the server), pairing Liquipedia pre-Match roster revisions (for the starting five before cutoff) with historical player stats from Matches finished strictly before cutoff.
