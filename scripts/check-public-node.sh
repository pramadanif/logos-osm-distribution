#!/usr/bin/env bash
# H27 / F.2: our public Logos Storage node serves data to a node it has never seen. A fresh local
# node bootstraps only from the public node's SPR (deploy/storage-node/public-node.json) and
# fetches the test CID over libp2p; the bytes must hash to the recorded MD5.
#   STORAGE_BIN=<logos-storage binary> scripts/check-public-node.sh
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
: "${STORAGE_BIN:?}"
[ -x "$STORAGE_BIN" ] || { echo "not executable: $STORAGE_BIN" >&2; exit 2; }
field() { python3 -I -c 'import json,sys; print(json.load(open(sys.argv[1]))[sys.argv[2]])' "$root/deploy/storage-node/public-node.json" "$1"; }
spr="$(field spr)"; cid="$(field test_cid)"; want="$(field test_md5)"; addr="$(field address)"
work="$(mktemp -d)"; api=http://127.0.0.1:18081/api/storage/v1
"$STORAGE_BIN" --network=logos.test --data-dir="$work/data" --api-bindaddr=127.0.0.1 --api-port=18081 \
  --listen-port=18071 --log-level=warn --bootstrap-node="$spr" > "$work/node.log" 2>&1 &
pid=$!; trap 'kill $pid 2>/dev/null || true; rm -rf "$work"' EXIT
for _ in $(seq 1 30); do curl -fs "$api/debug/info" >/dev/null 2>&1 && break; sleep 1; done
# A new node needs a moment to find the provider; retry for up to 2 minutes.
for _ in $(seq 1 24); do
  curl -fs --max-time 180 "$api/data/$cid/network/stream" -o "$work/got" && break
  rm -f "$work/got"; sleep 5
done
[ -f "$work/got" ] || { echo "FAIL $addr: $cid not retrieved"; tail -5 "$work/node.log"; exit 1; }
got="$(python3 -I -c 'import hashlib,sys; print(hashlib.md5(open(sys.argv[1],"rb").read()).hexdigest())' "$work/got")"
[ "$got" = "$want" ] || { echo "FAIL $addr: md5 $got, recorded $want"; exit 1; }
echo "PASS $addr served $cid ($(wc -c < "$work/got" | tr -d ' ') bytes, md5 $got)"
