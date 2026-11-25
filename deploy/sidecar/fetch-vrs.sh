#!/bin/sh
# Download Valve's global Regional Standings snapshots into a directory, skipping
# the ones already there. Usage: fetch-vrs.sh <directory> [year ...]
# (years default to the current one and the previous one). Needs curl and no key;
# GitHub's unauthenticated API allows 60 requests an hour, one per year here.
# Run it on a schedule (docs/sidecar.md): the sidecar lists the directory
# again every 10 minutes and reads new snapshots without a restart.
set -eu
dir=${1:?usage: fetch-vrs.sh <directory> [year ...]}
shift
if [ "$#" -eq 0 ]; then
  year=$(date -u +%Y)
  set -- "$((year - 1))" "$year"
fi
repo=ValveSoftware/counter-strike_regional_standings
mkdir -p "$dir"
for year in "$@"; do
  names=$(curl -fsS "https://api.github.com/repos/$repo/contents/live/$year" \
    | grep -o '"name": *"standings_global_[0-9_]*\.md"' | grep -o 'standings_global_[0-9_]*\.md') || names=
  for name in $names; do
    [ -f "$dir/$name" ] && continue
    curl -fsS -o "$dir/$name.partial" "https://raw.githubusercontent.com/$repo/main/live/$year/$name"
    mv "$dir/$name.partial" "$dir/$name"
    echo "fetched $name"
  done
done
