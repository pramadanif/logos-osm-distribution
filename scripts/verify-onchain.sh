#!/usr/bin/env bash
# One command to verify everything on a network from a clean clone (W1), public endpoints only:
#  1. the program header on chain carries the ImageID recorded for this deployment (E59);
#  2. coverage: every hosted CID fetched from Logos Storage and checked against Geofabrik's
#     published MD5, counted by the prize rules (scripts/verify-coverage.sh).
#   OSM_STORAGE=<Storage node API> [PROFILE=<networks.json profile>] scripts/verify-onchain.sh
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
profile="${PROFILE:-$(python3 -I -c 'import json,sys; print(json.load(open(sys.argv[1]))["default"])' "$root/networks.json")}"
get() { python3 -I -c 'import json,sys; print(json.load(open(sys.argv[1]))["profiles"][sys.argv[2]].get(sys.argv[3]) or "")' "$root/networks.json" "$profile" "$1"; }
rpc="$(get rpc)"; program="$(get program)"; want="$(get program_image_id)"
[ -n "$rpc" ] && [ -n "$program" ] && [ -n "$want" ] || { echo "profile $profile lacks rpc, program or program_image_id" >&2; exit 2; }
echo "1. program header $program on $rpc"
got="$(curl -fsS --max-time 30 -X POST "$rpc" -H 'content-type: application/json' \
  -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"getAccount\",\"params\":[\"$program\"]}" \
  | python3 -I -c 'import json,sys; s=json.load(sys.stdin)["result"]["data"]["shards"]; v=[x for x in s.values() if len(x)>=32]; print(bytes(v[0][:32]).hex() if v else "")')"
[ "$got" = "$want" ] || { echo "   ImageID on chain $got differs from recorded $want" >&2; exit 1; }
echo "   ImageID $got matches networks.json"
echo "2. coverage"
OSM_RPC="$rpc" OSM_PROGRAM="$program" exec "$root/scripts/verify-coverage.sh" "$@"
