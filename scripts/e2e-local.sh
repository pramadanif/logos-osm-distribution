#!/usr/bin/env bash
# End-to-end on local infrastructure (SC-E.1, H6): host -> query -> download -> verify.
# Needs a running LEZ v0.3 sequencer with the program deployed, two Storage nodes (node B knows
# only node A), a wallet with a funded hosting account, and the built `osm` and `osm-chain`.
# Any missing piece is a failure, never a skip.
#
#   OSM_RPC OSM_PROGRAM OSM_HOST_ACCOUNT LEE_WALLET_HOME_DIR OSM_CHAIN \
#   OSM_STORAGE_A OSM_STORAGE_B [OSM_BIN] [REGION] [SNAPSHOT] scripts/e2e-local.sh
set -euo pipefail
: "${OSM_RPC:?}" "${OSM_PROGRAM:?}" "${OSM_HOST_ACCOUNT:?}" "${LEE_WALLET_HOME_DIR:?}" "${OSM_CHAIN:?}"
: "${OSM_STORAGE_A:?}" "${OSM_STORAGE_B:?}"
OSM="${OSM_BIN:-$(cd "$(dirname "$0")/.." && pwd)/target/debug/osm}"
REGION="${REGION:-china/henan}"; SNAPSHOT="${SNAPSHOT:-261101}"
for bin in "$OSM" "$OSM_CHAIN"; do [ -x "$bin" ] || { echo "missing binary: $bin" >&2; exit 1; }; done
curl -fsS -X POST "$OSM_RPC" -H 'content-type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"getLastBlockId","params":[]}' >/dev/null || { echo "sequencer not reachable at $OSM_RPC" >&2; exit 1; }
for s in "$OSM_STORAGE_A" "$OSM_STORAGE_B"; do curl -fsS "$s/debug/info" >/dev/null || { echo "storage node not reachable at $s" >&2; exit 1; }; done
work="$(mktemp -d)"; trap 'rm -rf "$work"' EXIT

echo "1. host $REGION $SNAPSHOT via node A"
OSM_STORAGE="$OSM_STORAGE_A" "$OSM" host --region "$REGION" --snapshot "$SNAPSHOT" --cache-dir "$work/cache" --json > "$work/host.json"
cid="$(python3 -I -c 'import json,sys; d=json.load(open(sys.argv[1])); assert d["ok"], d; print(d["hosted"][0]["cid"])' "$work/host.json")"
echo "   cid=$cid"

echo "2. query the registry by region and by CID"
"$OSM" lookup --region "$REGION" --json | python3 -I -c 'import json,sys; d=json.load(sys.stdin); c=sys.argv[1]; assert any(e["cid"]==c for e in d["entries"]), d; print("   region lookup ok")' "$cid"
"$OSM" lookup --cid "$cid" --json | python3 -I -c 'import json,sys; d=json.load(sys.stdin); r=sys.argv[1]; assert any(e["region"]==r for e in d["entries"]), d; print("   cid lookup ok")' "$REGION"

echo "3. download via node B with Geofabrik disabled"
"$OSM" --no-geofabrik download --region "$REGION" --dir "$work/dl" --storage "$OSM_STORAGE_B" --json > "$work/dl.json"
python3 -I -c 'import json,sys; d=json.load(open(sys.argv[1])); assert d["source"]=="storage", d; print("   from", d["from"])' "$work/dl.json"

echo "4. verify against Geofabrik's published MD5 for the snapshot"
path="$(python3 -I -c 'import json,sys; print(json.load(open(sys.argv[1]))["path"])' "$work/dl.json")"
want="$(curl -fsS "$(python3 -I -c 'import json,sys; print(json.load(open(sys.argv[1]))["hosted"][0]["source_url"])' "$work/host.json").md5" | cut -d' ' -f1)"
got="$(python3 -I -c 'import hashlib,sys; h=hashlib.md5(); f=open(sys.argv[1],"rb"); [h.update(b) for b in iter(lambda: f.read(1<<20), b"")]; print(h.hexdigest())' "$path")"
[ "$got" = "$want" ] || { echo "MD5 mismatch: got $got want $want" >&2; exit 1; }
echo "   MD5 $got equals Geofabrik's published MD5"
echo "E2E OK"
