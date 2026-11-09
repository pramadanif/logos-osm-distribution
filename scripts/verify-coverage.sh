#!/usr/bin/env bash
# One command for evaluators (W1, H7): verify on-chain coverage against Logos Storage and
# Geofabrik's published MD5s. Public endpoints only; nothing is written anywhere.
#   OSM_RPC=<sequencer> OSM_PROGRAM=<program id> OSM_STORAGE=<Storage node API> scripts/verify-coverage.sh
# Defaults come from networks.json's default profile. Exit 0 only when >= 25 regions and >= 15
# countries verify; per-entry results are printed either way.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
profile="$(python3 -I -c 'import json,sys; d=json.load(open(sys.argv[1])); print(d["default"])' "$root/networks.json")"
read_profile() { python3 -I -c 'import json,sys; d=json.load(open(sys.argv[1]))["profiles"][sys.argv[2]]; print(d.get(sys.argv[3]) or "")' "$root/networks.json" "$profile" "$1"; }
export OSM_RPC="${OSM_RPC:-$(read_profile rpc)}"
export OSM_PROGRAM="${OSM_PROGRAM:-$(read_profile program)}"
: "${OSM_STORAGE:?set OSM_STORAGE to a Logos Storage node API, e.g. http://127.0.0.1:8080/api/storage/v1}"
[ -n "$OSM_RPC" ] && [ -n "$OSM_PROGRAM" ] || { echo "profile $profile has no rpc/program" >&2; exit 2; }
osm="${OSM_BIN:-$root/target/release/osm}"
[ -x "$osm" ] || (cd "$root" && cargo build --release --locked -p osm_cli)
work="$(mktemp -d)"; trap 'rm -rf "$work"' EXIT
echo "network profile: $profile  rpc: $OSM_RPC  program: $OSM_PROGRAM"
exec "$osm" verify-coverage --dir "$work" "$@"
