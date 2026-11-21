#!/usr/bin/env bash
# Restore the database from a backup made by backup.sh. Run on the server.
#
#   ./restore.sh backups/kallbet-20260101T030000Z.db.gz
#
# Stops the app, replaces /data/kallbet.db with the backup, starts the app.
# The current database is first copied to /data/kallbet.db.before-restore in
# the volume, so a wrong restore can be undone.
set -euo pipefail

[ $# -eq 1 ] && [ -f "$1" ] || { echo "usage: $0 BACKUP.db.gz" >&2; exit 2; }
file=$(cd "$(dirname "$1")" && pwd)/$(basename "$1")
gzip -t "$file"

cd "${KALLBET_DIR:-$(cd "$(dirname "$0")" && pwd)}"

docker compose stop kallbet
gzip -dc "$file" | docker compose run --rm -T --no-deps --entrypoint sh kallbet -euc '
  [ ! -f /data/kallbet.db ] || cp /data/kallbet.db /data/kallbet.db.before-restore
  rm -f /data/kallbet.db-wal /data/kallbet.db-shm
  cat > /data/kallbet.db.new
  [ "$(sqlite3 /data/kallbet.db.new "PRAGMA integrity_check;")" = ok ]
  mv /data/kallbet.db.new /data/kallbet.db
'
docker compose up -d kallbet
echo "restored from $file"
