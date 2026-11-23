#!/usr/bin/env bash
# Stages the coverage set (coverage/regions.txt) into a Storage node and coverage/manifest.json,
# one region at a time, with no transaction (`osm stage`). Resumable: a region already in the
# manifest for this month's snapshot is skipped, and the manifest is written after each region.
#
#   OSM_BIN=<osm> OSM_STORAGE=<Storage API> CACHE_DIR=<dir> [MANIFEST=coverage/manifest.json]
#   [REGIONS=coverage/regions.txt] [CACHE_CAP_GIB=3] scripts/stage-coverage.sh
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
: "${OSM_BIN:?}" "${OSM_STORAGE:?}" "${CACHE_DIR:?}"
manifest="${MANIFEST:-$root/coverage/manifest.json}"
regions="${REGIONS:-$root/coverage/regions.txt}"
snap="${SNAPSHOT:-$(date -u +%y%m)01}"
done_regions() {
  [ -f "$manifest" ] || return 0
  python3 -I -c 'import json,sys; [print(e["region"]) for e in json.load(open(sys.argv[1]))["entries"] if e["snapshot"]==int(sys.argv[2])]' "$manifest" "$snap"
}
skip="$(done_regions)"
n=0; staged=0
while read -r r; do
  case "$r" in ''|\#*) continue ;; esac
  n=$((n + 1))
  if printf '%s\n' "$skip" | grep -qx "$r"; then echo "$r: already staged for $snap"; continue; fi
  "$OSM_BIN" stage --region "$r" --snapshot "$snap" --storage "$OSM_STORAGE" --cache-dir "$CACHE_DIR" \
    --cache-cap-gib "${CACHE_CAP_GIB:-3}" --manifest "$manifest"
  staged=$((staged + 1))
done < "$regions"
echo "staged $staged of $n regions for snapshot $snap into $manifest"
