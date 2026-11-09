#!/usr/bin/env bash
# Starts two local Logos Storage nodes (A, B; B bootstraps from A only) for the retrieval test.
# Usage: scripts/storage-two-nodes.sh <node-binary> <work-dir>
# Prints OSM_STORAGE_A=... and OSM_STORAGE_B=... lines for the caller to export.
# Exits non-zero if either node does not answer within 60 s (no skip, H6).
set -euo pipefail
bin="$1"; work="$2"
mkdir -p "$work/a" "$work/b"; chmod 700 "$work/a" "$work/b"
api() { echo "http://127.0.0.1:$1/api/storage/v1"; }
wait_up() {
  for _ in $(seq 1 60); do curl -fs "$(api "$1")/debug/info" >/dev/null && return 0; sleep 1; done
  echo "storage node on port $1 did not start; log:" >&2; tail -20 "$2" >&2; return 1
}
"$bin" --data-dir="$work/a" --api-port=8081 --listen-port=8071 --listen-ip=127.0.0.1 \
  --nat=extip:127.0.0.1 --no-bootstrap-node >"$work/a.log" 2>&1 &
wait_up 8081 "$work/a.log"
spr="$(curl -fs "$(api 8081)/spr" | python3 -I -c 'import json,sys; print(json.load(sys.stdin)["spr"])')"
"$bin" --data-dir="$work/b" --api-port=8082 --listen-port=8072 --listen-ip=127.0.0.1 \
  --nat=extip:127.0.0.1 --bootstrap-node="$spr" >"$work/b.log" 2>&1 &
wait_up 8082 "$work/b.log"
echo "OSM_STORAGE_A=$(api 8081)"
echo "OSM_STORAGE_B=$(api 8082)"
