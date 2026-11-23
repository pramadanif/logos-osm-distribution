#!/usr/bin/env bash
# H27 / F.2: our public Logos Storage node serves data to a node it has never seen. A fresh local
# node bootstraps only from the public node's SPR (deploy/storage-node/public-node.json) and
# fetches the test CID over libp2p; the bytes must hash to the recorded MD5.
#   STORAGE_BIN=<logos-storage binary> [OSM_BIN=<osm> REGION=<region>] scripts/check-public-node.sh
# With OSM_BIN, also downloads REGION (default china/henan) the way a user does: resolve it in the
# registry of networks.json's default profile, fetch its CID through the same outside node with
# Geofabrik disabled, and check the MD5 the registry holds.
# With COVERAGE=<n>, also fetches the n smallest entries of coverage/manifest.json (the staged
# coverage set) through the same outside node and checks each against the MD5 recorded there.
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
if [ -n "${OSM_BIN:-}" ]; then
  region="${REGION:-china/henan}"
  read -r rpc program < <(python3 -I -c 'import json,sys; d=json.load(open(sys.argv[1])); p=d["profiles"][d["default"]]; print(p["rpc"], p["program"])' "$root/networks.json")
  "$OSM_BIN" --no-geofabrik download --region "$region" --dir "$work/maps" --storage "$api" --rpc "$rpc" --program "$program" \
    || { echo "FAIL $region: not downloaded from the registry through $addr"; exit 1; }
  echo "PASS $region resolved on $rpc ($program), fetched by CID through $addr, MD5 equals the registry"
fi
if [ -n "${COVERAGE:-}" ]; then
  n=0
  while read -r region ccid cmd5; do
    # The node runs at low CPU and IO priority beside other services, so a transfer can stall;
    # three tries of up to 10 minutes each.
    rm -f "$work/cov"
    for try in 1 2 3; do
      curl -fsS --max-time 600 "$api/data/$ccid/network/stream" -o "$work/cov" && break
      echo "$region: try $try did not complete" >&2; rm -f "$work/cov"; sleep 30
    done
    [ -f "$work/cov" ] || { echo "FAIL $region: $ccid not retrieved through $addr"; exit 1; }
    got="$(python3 -I -c 'import hashlib,sys
h=hashlib.md5()
with open(sys.argv[1],"rb") as f:
    for b in iter(lambda: f.read(1<<20), b""): h.update(b)
print(h.hexdigest())' "$work/cov")"
    [ "$got" = "$cmd5" ] || { echo "FAIL $region: md5 $got, manifest $cmd5"; exit 1; }
    echo "PASS $region: $ccid ($(wc -c < "$work/cov" | tr -d ' ') bytes) through $addr, MD5 equals Geofabrik's"
    rm -f "$work/cov"; n=$((n + 1))
  done < <(python3 -I -c 'import json,sys
es=sorted(json.load(open(sys.argv[1]))["entries"], key=lambda e: e["size_bytes"])
for e in es[:int(sys.argv[2])]: print(e["region"], e["cid"], e["md5"])' "$root/coverage/manifest.json" "$COVERAGE")
  [ "$n" -ge "$COVERAGE" ] || { echo "FAIL coverage: $n of $COVERAGE entries in coverage/manifest.json"; exit 1; }
fi

