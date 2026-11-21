#!/usr/bin/env bash
# Build the image locally, ship it to the server over SSH, restart the stack
# and prune old images. Usage, from anywhere:  deploy/deploy.sh
#
# Settings come from the environment or from deploy/deploy.env (git-ignored;
# see deploy/deploy.env.example):
#   KALLBET_DEPLOY_HOST      ssh target (required)
#   KALLBET_DEPLOY_DIR       directory on the server (default /opt/kallbet)
#   KALLBET_DEPLOY_SSH_OPTS  extra ssh options (optional)
#   KALLBET_KEEP_IMAGES      kallbet image tags kept for rollback (default 2)
set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
root=$(dirname "$here")

if [ -f "$here/deploy.env" ]; then
  # Variables already in the environment win over the file.
  while IFS='=' read -r key value; do
    case $key in ''|\#*) continue ;; esac
    [ -n "${!key:-}" ] || export "$key=$value"
  done < "$here/deploy.env"
fi

host=${KALLBET_DEPLOY_HOST:?set KALLBET_DEPLOY_HOST (see deploy/deploy.env.example)}
remote_dir=${KALLBET_DEPLOY_DIR:-/opt/kallbet}
keep=${KALLBET_KEEP_IMAGES:-2}
# Word-splitting of the options is intended.
# shellcheck disable=SC2206
ssh_opts=(${KALLBET_DEPLOY_SSH_OPTS:-})

# The command is expanded on this side on purpose.
# shellcheck disable=SC2029
remote() { ssh ${ssh_opts[@]+"${ssh_opts[@]}"} "$host" "$@"; }

tag=$(git -C "$root" rev-parse --short=12 HEAD)
if [ -n "$(git -C "$root" status --porcelain)" ]; then
  tag="$tag-dirty-$(date -u +%Y%m%d%H%M%S)"
  echo "warning: working tree has uncommitted changes; deploying as $tag" >&2
fi

echo "==> building kallbet:$tag"
docker build -t "kallbet:$tag" "$root"

echo "==> shipping files to $remote_dir"
remote "mkdir -p '$remote_dir/backups'"
tar -C "$root" -cf - docker-compose.yml -C "$here" backup.sh restore.sh \
  | remote "tar -C '$remote_dir' -xf - && chmod +x '$remote_dir/backup.sh' '$remote_dir/restore.sh'"
remote "test -f '$remote_dir/kallbet.env'" \
  || { echo "error: $remote_dir/kallbet.env is missing on the server (docs/deploy.md, first-time setup)" >&2; exit 1; }

echo "==> shipping image"
docker save "kallbet:$tag" | gzip | remote "gunzip | docker load"

echo "==> compose up"
remote "set -e; cd '$remote_dir'
  docker tag 'kallbet:$tag' kallbet:latest
  docker compose up -d --remove-orphans
  echo '==> pruning old images'
  docker images kallbet --format '{{.Tag}}' | grep -v '^latest\$' | tail -n +$((keep + 1)) \
    | xargs -r -I{} docker rmi kallbet:{}
  docker image prune -f"

echo "==> deployed kallbet:$tag"
