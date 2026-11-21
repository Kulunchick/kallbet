#!/usr/bin/env bash
# Daily SQLite backup, run on the server (see docs/deploy.md for the cron line).
#
# Uses sqlite3's online `.backup` inside the running container (safe while the
# app writes), checks the copy, and streams it gzipped to the host. Files are
# kept for KALLBET_BACKUP_DAYS days (default 7), then deleted.
#
# Environment (all optional):
#   KALLBET_DIR          directory with docker-compose.yml (default: this script's directory)
#   KALLBET_BACKUP_DIR   where backups go (default: $KALLBET_DIR/backups)
#   KALLBET_BACKUP_DAYS  retention in days (default: 7)
set -euo pipefail

dir=${KALLBET_DIR:-$(cd "$(dirname "$0")" && pwd)}
backup_dir=${KALLBET_BACKUP_DIR:-$dir/backups}
days=${KALLBET_BACKUP_DAYS:-7}

mkdir -p "$backup_dir"
cd "$dir"

stamp=$(date -u +%Y%m%dT%H%M%SZ)
final="$backup_dir/kallbet-$stamp.db.gz"
tmp="$final.partial"
trap 'rm -f "$tmp"' EXIT

# Inside the container: take the backup, verify it, stream it out, clean up.
docker compose exec -T kallbet sh -euc '
  tmp=$(mktemp /tmp/backup.XXXXXX)
  trap "rm -f $tmp" EXIT
  sqlite3 /data/kallbet.db ".backup $tmp"
  [ "$(sqlite3 "$tmp" "PRAGMA integrity_check;")" = ok ] || { echo "integrity_check failed" >&2; exit 1; }
  gzip -c "$tmp"
' > "$tmp"

# A valid gzip of a non-trivial file, or the run counts as failed.
gzip -t "$tmp"
mv "$tmp" "$final"
echo "backup written: $final"

# Retention: delete our own backups older than $days days.
find "$backup_dir" -maxdepth 1 -type f -name 'kallbet-*.db.gz' -mmin +$((days * 24 * 60)) -print -delete \
  | sed 's/^/pruned: /'
