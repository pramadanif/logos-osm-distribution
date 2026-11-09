#!/usr/bin/env bash
# Captures real region-account bytes from a running sequencer into osm_sdk test fixtures (H32).
# Usage: RPC=http://127.0.0.1:3040 PROGRAM=<header> tools/capture-registry-fixture.sh <out.json> <region>=<account>...
set -euo pipefail
: "${RPC:?}" "${PROGRAM:?}"
out="$1"; shift
python3 -I - "$RPC" "$PROGRAM" "$out" "$@" <<'PY'
import json, sys, urllib.request, datetime
rpc, program, out, pairs = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4:]
def call(method, params):
    req = urllib.request.Request(rpc, data=json.dumps({"jsonrpc": "2.0", "id": 1, "method": method, "params": params}).encode(),
                                 headers={"content-type": "application/json"})
    return json.load(urllib.request.urlopen(req, timeout=30))["result"]
accounts = []
for p in pairs:
    region, account = p.split("=", 1)
    shards = call("getAccount", [account])["data"]["shards"]
    if program not in shards:
        sys.exit(f"{region}: no shard for {program} at {account}")
    accounts.append({"region": region, "account": account, "bytes": shards[program]})
json.dump({"captured": datetime.date.today().isoformat(), "rpc": rpc, "program": program,
           "lez": "v0.3.1 standalone", "accounts": accounts}, open(out, "w"), indent=1)
open(out, "a").write("\n")
print(len(accounts), "accounts")
PY
