# Running the AI sidecar

`kallbet-ai` is an optional second program next to KallBet. It signs in as **Bot Users** (the AI Oracle, "Market Favourite", "VRS Baseline"), joins your Leagues through their Invite Links, and writes Predictions and Pick'ems over KallBet's HTTP API. It never touches KallBet's database; it keeps its own SQLite file. Design: spec #21 and `docs/agents/codebase.md` ("AI Sidecar" onwards). This page is the operator's runbook.

Two ways to run it, with the same binary and the same configuration file format:

| | Where | Subscriptions |
|---|---|---|
| **Local** | your workstation, against `https://kallbet.kulunchick.tech` (or a local KallBet) | your already signed-in `claude`, `codex`, `agy` |
| **VPS** | the `sidecar` service of `docker-compose.yml` (profile `sidecar`) | CLIs signed in once inside the container, or an OAuth proxy, or API keys |

Plain `docker compose up -d` does not start the sidecar: the existing `kallbet` + `chrome` stack is unchanged.

## 1. Secret and Invite Links (both ways)

1. **Generate the shared secret** once and keep it out of git:
   ```sh
   openssl rand -hex 32
   ```
   Put it in the server's `kallbet.env` as `KALLBET_BOT_SECRET=...` and recreate `kallbet` (`docker compose up -d --force-recreate kallbet`). Until it is set the server answers `503 bot_auth_unavailable` and no Bot User can register. The sidecar needs the **same** value as `KALLBET_BOT_SECRET` (local `.env`, or `sidecar.env` on the server).
2. **Invite Links.** A League Admin opens the League in KallBet and copies its Invite Link (`https://kallbet.kulunchick.tech/join/<code>`). Put it in the `invite_links` of each bot in the sidecar configuration. The sidecar joins every configured League on start and ignores Leagues it already belongs to. To remove a bot, kick it with the normal League controls and delete the link from the configuration. An Invite Link is a credential: keep the configuration file private (it is git-ignored under the names below) and reissue the link in KallBet if it leaks.
3. **Configuration file.** Start from one of `deploy/sidecar/`:

   | File | For |
   |---|---|
   | `sidecar.local.example.json` | a workstation: relative paths, `platform_url` is the public site |
   | `sidecar.compose.example.json` | the Compose service: `/data/...` paths, `platform_url` `http://kallbet:8095`, `claude`/`codex`/`antigravity` CLIs |
   | `sidecar.proxy.example.json` | the Compose service with an OAuth/API proxy instead of CLIs (small image) |

   Each has the three profiles: **AI Oracle** (`HybridAnchored`, Claude as Market Advocate, Codex as Devil's Advocate, Antigravity as Chief Arbiter, each falling back to the other two), **Market Favourite** (`MarketOnly`) and **VRS Baseline** (`VrsOnly`); the two baselines make no LLM calls. Drop any bot you do not want. Replace `REPLACE_WITH_INVITE_CODE`. Event-specific parts for PGL Major Singapore 2026 are the HLTV Event id `8302` in `context.liquipedia_pages` and in the odds variables below.
4. **Environment.** `deploy/sidecar/sidecar.env.example` lists every variable with its default and what it does. The ones that matter: `KALLBET_BOT_SECRET`, `SIDECAR_CONFIG`, and for market odds `ODDSPAPI_API_KEY` with `SIDECAR_ODDSPAPI_TOURNAMENTS` and `SIDECAR_POLYMARKET_TAGS` (without them the odds chain skips that source, and `MarketOnly` falls back to VRS).
5. **Valve standings.** Both baselines and the Oracle's statistical side need Valve's regional standings. `deploy/sidecar/fetch-vrs.sh <dir>` downloads the global snapshots (needs `curl`, no key). The sidecar reads the directory at start, so re-run the script as Valve publishes (monthly) and restart the sidecar.

## 2. Local run (workstation)

```sh
cp deploy/sidecar/sidecar.local.example.json sidecar.local.json    # edit the Invite Links
cp deploy/sidecar/sidecar.env.example .env                          # set KALLBET_BOT_SECRET
deploy/sidecar/fetch-vrs.sh vrs
cargo run --release -p kallbet-ai
```

`.env` in the working directory is read automatically. The first tick happens at once, then one per `SIDECAR_TICK_SECONDS` (60). Ctrl-C stops it cleanly. `cargo run -p kallbet-ai -- run --once` does a single tick and exits (non-zero if it failed): good for a first check.

**Subscriptions locally:** sign each CLI in once, the way you normally do; the sidecar starts them as subprocesses with their tools disabled and uses your login.

```sh
claude auth login        # Claude subscription
codex login              # or: codex login --device-auth
agy                      # sign in on first start
```

On Windows an npm `.cmd` shim is not found by name: give the full path of the executable as `binary` in the provider (`"claude": {"kind": "claude", "binary": "C:\\...\\claude.exe"}`).

To try it against a local KallBet instead, set `platform_url` to `http://localhost:8095`, use the server's `KALLBET_BOT_SECRET` from its `.env`, and take Invite Links from that instance.

## 3. VPS run (Docker Compose)

Files go in the deploy directory (`/opt/kallbet`) beside `docker-compose.yml` and `kallbet.env`:

```sh
cd /opt/kallbet
cp sidecar.compose.example.json sidecar.json          # edit the Invite Links; chmod 600
cp sidecar.env.example sidecar.env                    # set KALLBET_BOT_SECRET, ODDSPAPI_API_KEY; chmod 600
mkdir -p vrs && ./fetch-vrs.sh vrs                    # Valve snapshots, bind-mounted read-only
```

(`deploy.sh` with `KALLBET_DEPLOY_SIDECAR=true` ships `Dockerfile.sidecar`'s image and these example files next to the compose file; it never copies `sidecar.json` or `sidecar.env`, which stay on the server.)

Choose the image with `KALLBET_AI_TARGET` (shell or `/opt/kallbet/.env`):

- `sidecar` (default, about 15 MB): the binary and its baseline bots, plus any `http` LLM provider. Use it with `sidecar.proxy.example.json`.
- `sidecar-cli` (about 400 MB more): adds Node.js and the `claude` and `codex` CLIs, pinned in `Dockerfile.sidecar`. **`agy` is not packaged** (this repository bundles no Antigravity install); in this image an `antigravity` provider is simply "not configured" and every chain skips it, so the Oracle debates on Claude and Codex. To use Antigravity on the VPS, run it behind an `http` provider.

```sh
docker compose --profile sidecar up -d --build sidecar      # or ship the image with deploy.sh
```

The container runs as an unprivileged user with a persistent volume `sidecar-data` on `/data`: the SQLite file (`/data/sidecar.db`, every dossier and debate turn, append-only), the heartbeat, and `$HOME=/data/home`, where the CLIs keep their sign-in. Logs are capped at 3 files of 10 MB.

### Authenticating the subscriptions on the VPS

**CLI image (`sidecar-cli`).** Sign in once inside the container; the credentials land on the volume and survive restarts and upgrades:

```sh
docker compose --profile sidecar run --rm -it --entrypoint claude sidecar auth login
docker compose --profile sidecar run --rm -it --entrypoint codex  sidecar login --device-auth
```

Each prints a URL; open it in any browser, sign in, paste the code back if asked. Check with `... --entrypoint claude sidecar auth status` and `... --entrypoint codex sidecar login status`. A subscription whose session expires makes its provider fail with `Auth`; the chain moves to the next one and the log says so (`llm: provider failed`). Sign in again the same way.

**OAuth proxy or API key (small image).** Run a lightweight subscription gateway as another service on the `internal` network (or use an API key), then point a provider at it:

```json
"llm": {"providers": {"proxy": {"kind": "http", "api": "anthropic",
        "base_url": "http://llm-proxy:8080/v1", "model": "claude-opus-5-5",
        "api_key_env": "KALLBET_AI_LLM_API_KEY"}}}
```

`api_key_env` names the variable that holds the key (set it in `sidecar.env`); leave it out for a proxy that holds the subscription itself. `api` is `anthropic` or `openai`. See `sidecar.proxy.example.json` and "LLM Runtimes" in `docs/agents/codebase.md`. Which proxy to run is your choice; the sidecar only needs an Anthropic- or OpenAI-compatible `/v1` endpoint.

## 4. Verify

| Check | How |
|---|---|
| Server accepts bots | `curl -s -o /dev/null -w '%{http_code}\n' -X POST https://kallbet.kulunchick.tech/api/auth/bot -H 'Authorization: Bearer wrong'` is `401` (`503` means `KALLBET_BOT_SECRET` is unset on the server) |
| Config and secret are right | `kallbet-ai run --once` (VPS: `docker compose --profile sidecar run --rm sidecar run --once`) logs `single tick ok` and exits 0 |
| Loop is alive | `docker compose --profile sidecar ps`: `sidecar` is `healthy`. Or `docker compose --profile sidecar exec sidecar kallbet-ai health`: `healthy: last activity 12 s ago, 0 failed ticks in a row`. It turns unhealthy after 30 minutes without a tick, or 10 failed ticks in a row (`SIDECAR_HEALTH_*`). |
| Bots joined | the three bots are among the League's Members, and their Predictions carry the bot badge after Reveal |
| Predictions arrive | after the first tick with an open Match, each bot has a Prediction in the League (hidden until Reveal) |
| Logs | `docker compose logs -f sidecar` shows `kallbet-ai starting`, then `tick ok submitted=N elapsed_ms=...` once per tick. `SIDECAR_LOG=debug`, `SIDECAR_LOG_FORMAT=json` for more or for machine-readable lines |

**Shutdown.** `SIGINT` and `SIGTERM` (what `docker stop` sends) end the loop at once and exit 0. A tick in progress is dropped, which is safe: every step is recorded in the audit file first, and a child CLI process dies with its tick. The next start continues from the audit.

**Secrets in logs.** The Bot secret, API keys, bearer tokens, `apiKey=` style query values and Invite Link codes are removed from every log line, and `Debug` output of the types that hold them shows `[REDACTED]`. Every environment variable whose name contains `SECRET`, `TOKEN`, `KEY` or `PASSWORD` is removed by value as well. Prompts and dossiers are not logged.

## 5. Operating notes

- **One sidecar per Bot identity.** Two processes with the same `bot_key` revoke each other's token on every registration (it rotates per tick). Do not run the local sidecar and the VPS one with the same bot keys at once; use different `bot_key`s to compare.
- **Quota.** A subscription that hits its limit opens that provider's breaker for `llm.cooldown_minutes` (30) and the chain fails over (Claude → Codex → Antigravity). OddsPapi is capped at 200 requests a month by the sidecar itself.
- **Upgrade.** `deploy.sh` (with the sidecar switch) or `docker compose --profile sidecar up -d --build sidecar`. The volume keeps the audit file and the CLI sign-in. Migrations of the audit file only move forward.
- **Stop for good.** `docker compose --profile sidecar stop sidecar`. The bots stay Members; kick them in the League if they should go.
- **Backtest.** The retrospective benchmark is a subcommand (`kallbet-ai backtest`, ticket #31).
