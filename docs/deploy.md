# Deploying KallBet

KallBet runs on one small VPS as a docker compose stack behind the server's existing Caddy, at `https://kallbet.kulunchick.tech` (the domain is behind Cloudflare). Nothing that identifies the server is committed: the deploy script takes the host from your environment or an untracked file.

## What runs where

| Piece | Where | Notes |
|---|---|---|
| `kallbet` | container, published on `127.0.0.1:8095` only | One Rust binary: API plus the built SPA. SQLite lives in the named volume `kallbet-data` (`/data/kallbet.db`, ADR-0002). |
| `chrome` | container, `chromedp/headless-shell:stable` | No published port. Reachable only from `kallbet` over the compose network. |
| Caddy | already on the server | Gets one site block (`deploy/Caddyfile.snippet`). |
| backups | host cron + `backups/` in the deploy directory | `deploy/backup.sh`, daily, 7 days kept. |

Files: `Dockerfile`, `.dockerignore`, `docker-compose.yml`, and `deploy/` (`deploy.sh`, `backup.sh`, `restore.sh`, `Caddyfile.snippet`, `deploy.env.example`).

The image is multi-stage (Node builds `web/dist`, Rust builds the binary on musl, the runtime is Alpine with the binary, the SPA and the `sqlite3` CLI), about 9 MB. The `chrome` image is about 160 MB and is pulled once. The deploy script keeps the two newest `kallbet` image tags on the server for rollback and prunes everything else.

## Configuration

The server's env file `kallbet.env` (in the deploy directory, never committed) holds the secrets:

```
BOT_TOKEN=...
BOT_USERNAME=...        # the bot's @username without the @, needed by the Telegram login button
PANDASCORE_TOKEN=...
```

`docker-compose.yml` fixes the rest, so the env file cannot break them: `KALLBET_BIND=0.0.0.0:8095` (inside the container; the host side is loopback only), `DATABASE_URL=sqlite:///data/kallbet.db`, `KALLBET_DEV_LOGIN=false`.

**Chrome endpoint for the HLTV adapter (ticket #8):** the variable is `KALLBET_CHROME_URL`, set by compose to `http://chrome:9222`. It is not read by `src/config.rs` yet; ticket #8 adds the field (and its `.env.example` line). Local development without Docker can point it at any local Chrome started with `--remote-debugging-port=9222`. Chrome's DevTools HTTP endpoint rejects requests whose `Host` header is neither `localhost` nor an IP address, so a client that connects to `chrome:9222` by service name must send a `Host: localhost` header or resolve the name to an IP first (see the notes in ticket #8).

## First-time setup (once)

On the server (needs Docker with the compose plugin, and `ssh` access for you):

1. Create the deploy directory and the env file:
   ```sh
   sudo mkdir -p /opt/kallbet/backups && sudo chown -R "$USER" /opt/kallbet
   install -m 600 /dev/null /opt/kallbet/kallbet.env
   $EDITOR /opt/kallbet/kallbet.env      # BOT_TOKEN=..., BOT_USERNAME=..., PANDASCORE_TOKEN=...
   ```
   Use another directory if you prefer and set `KALLBET_DEPLOY_DIR` to match.
2. Run the first deploy from your machine (next section). It copies the compose file and scripts into the directory.
3. Caddy, in this order:
   ```sh
   sudo cp /etc/caddy/Caddyfile /etc/caddy/Caddyfile.bak-$(date +%Y%m%d)   # back it up first; adjust the path to where your Caddyfile lives
   sudo tee -a /etc/caddy/Caddyfile < deploy/Caddyfile.snippet               # or paste the block by hand
   sudo caddy validate --config /etc/caddy/Caddyfile
   sudo systemctl reload caddy
   ```
   If validation fails, restore the `.bak` copy. The server's Caddy already trusts Cloudflare's `CF-Connecting-IP`; the snippet does not touch that.
4. Schedule the backup (cron, as the user that can run docker):
   ```sh
   ( crontab -l 2>/dev/null; echo '17 3 * * * /opt/kallbet/backup.sh >> /opt/kallbet/backups/backup.log 2>&1' ) | crontab -
   ```
   This takes a backup at 03:17 server time. `backup.sh` deletes its own `kallbet-*.db.gz` files older than 7 days (`KALLBET_BACKUP_DAYS` changes that). Run it once by hand to see it work.
5. Manual steps outside the repo (owner only), see the last section.

## Deploying

On your machine, with Docker and `ssh` access:

```sh
cp deploy/deploy.env.example deploy/deploy.env      # once; git-ignored
$EDITOR deploy/deploy.env                           # KALLBET_DEPLOY_HOST, KALLBET_DEPLOY_DIR
deploy/deploy.sh
```

The same variables can be exported in the shell instead. The script: builds `kallbet:<git sha>`, copies `docker-compose.yml`, `backup.sh` and `restore.sh` to the server, checks that `kallbet.env` exists there, streams the image over ssh (`docker save | gzip | ssh ... docker load`), tags it `kallbet:latest`, runs `docker compose up -d`, then removes `kallbet` image tags beyond the newest two and runs `docker image prune -f`.

Check afterwards: `ssh <host> 'cd /opt/kallbet && docker compose ps'` (both services up, `kallbet` healthy) and `curl https://kallbet.kulunchick.tech/api/health` returns `{"status":"ok"}`.

Rollback: `ssh <host>`, then `docker tag kallbet:<previous-tag> kallbet:latest && cd /opt/kallbet && docker compose up -d`. Tags are listed by `docker images kallbet`. Migrations only move forward, so after a schema change roll back with a restore (below) rather than an older image.

Logs: `docker compose logs -f kallbet`.

## Backup and restore

`backup.sh` runs `sqlite3 .backup` inside the running container (safe while the app writes), checks the copy with `PRAGMA integrity_check`, and writes `backups/kallbet-<UTC timestamp>.db.gz` on the host through a `.partial` file, so a failed run leaves no half file. Then it deletes backups older than the retention.

The backups sit on the same disk as the database, so they cover mistakes and corruption, not the loss of the VPS. Copy `backups/` elsewhere from time to time if that matters.

Restore (the app is down for a few seconds):

```sh
cd /opt/kallbet
ls backups/                                   # pick a file
./restore.sh backups/kallbet-20260101T031700Z.db.gz
```

`restore.sh` stops `kallbet`, copies the current database to `/data/kallbet.db.before-restore` inside the volume, writes the backup in place (checked with `integrity_check` before the swap), and starts `kallbet` again. Verify with `curl http://127.0.0.1:8095/api/health` on the server.

## Manual steps left to the owner

- Put `BOT_TOKEN`, `BOT_USERNAME` and `PANDASCORE_TOKEN` in the server's `kallbet.env`.
- Add the Caddy site block (with the Caddyfile backup first) and reload Caddy.
- Make sure the DNS record for `kallbet.kulunchick.tech` exists and is proxied through Cloudflare.
- Install the backup cron line.
- BotFather: send `/setdomain` to the bot and give it `kallbet.kulunchick.tech`. Telegram login does not work until this is done (ticket #19).
