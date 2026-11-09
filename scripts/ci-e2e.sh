#!/usr/bin/env bash
# Full end-to-end on fresh infrastructure (SC-E.1, H6): builds and starts a standalone LEZ v0.3.1
# sequencer, deploys the registry program built from this commit, starts two Storage nodes and a
# fixture Geofabrik server, then runs scripts/e2e-local.sh. Used by CI; runnable locally.
#
# Needs: cargo, docker (guest build), cargo-risczero 3.0.5, curl, python3, unzip.
#   WORK=<dir> STORAGE_BIN=<logos-storage binary> scripts/ci-e2e.sh
# Every missing tool or failed step exits non-zero; nothing is skipped (H6, §0.4 rule 4).
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
: "${WORK:?}" "${STORAGE_BIN:?}"
mkdir -p "$WORK"; WORK="$(cd "$WORK" && pwd)"
for t in cargo docker curl python3 unzip git; do command -v "$t" >/dev/null || { echo "missing tool: $t" >&2; exit 1; }; done
cargo risczero --version >/dev/null || { echo "missing tool: cargo-risczero" >&2; exit 1; }

LEZ_TAG=v0.3.1; LEZ_SHA=1a003bfbfb89fcf987f62975cd08a401169e1b8c   # VERSIONS.md LEZ_V03
lez="$WORK/lez-$LEZ_TAG"
if [ ! -d "$lez/.git" ]; then git -c advice.detachedHead=false clone -q --depth 1 --branch "$LEZ_TAG" https://github.com/logos-blockchain/logos-execution-zone.git "$lez"; fi
[ "$(git -C "$lez" rev-parse HEAD)" = "$LEZ_SHA" ] || { echo "LEZ $LEZ_TAG is not $LEZ_SHA" >&2; exit 1; }

echo "::group::build LEZ sequencer + wallet"
# Upstream build scripts download prebuilt artifacts (rapidsnark, circuits); retry on network errors.
for attempt in 1 2 3; do
  if (cd "$lez" && cargo build --release --locked --features standalone -p sequencer_service && cargo build --release --locked -p wallet); then break; fi
  [ "$attempt" -lt 3 ] || { echo "LEZ build failed 3 times" >&2; exit 1; }
  echo "LEZ build attempt $attempt failed; retrying in 60 s"; sleep 60
done
echo "::endgroup::"
echo "::group::build registry program (docker) and binaries"
(cd "$root" && cargo risczero build --manifest-path programs/osm_registry/methods/guest/Cargo.toml)
(cd "$root/programs/osm_registry" && cargo build --locked -p lez_adapter_v03 --bin osm-chain)
(cd "$root" && cargo build --locked -p osm_cli)
echo "::endgroup::"

echo "::group::start sequencer (loopback only)"
seq_home="$WORK/seq-home"; mkdir -p "$seq_home"
(cd "$lez/lez/sequencer/service" && RUST_LOG=info,kameo=warn "$lez/target/release/sequencer_service" \
  --listen-address 127.0.0.1 --home "$seq_home" configs/debug/sequencer_config.json >"$WORK/sequencer.log" 2>&1 &)
for _ in $(seq 1 60); do
  curl -fsS -X POST http://127.0.0.1:3040 -H 'content-type: application/json' \
    -d '{"jsonrpc":"2.0","id":1,"method":"getLastBlockId","params":[]}' >/dev/null 2>&1 && break; sleep 2
done
curl -fsS -X POST http://127.0.0.1:3040 -H 'content-type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"getLastBlockId","params":[]}' >/dev/null || { tail -30 "$WORK/sequencer.log" >&2; exit 1; }
echo "::endgroup::"

echo "::group::wallet with the localnet test accounts (published in LEZ's Justfile; local only)"
export LEE_WALLET_HOME_DIR="$WORK/wallet"; mkdir -p "$LEE_WALLET_HOME_DIR"; chmod 700 "$LEE_WALLET_HOME_DIR"
cat > "$LEE_WALLET_HOME_DIR/wallet_config.json" <<'JSON'
{ "sequencer": { "sequencer_addr": "http://127.0.0.1:3040" },
  "sequencers": [{ "sequencer_addr": "http://127.0.0.1:3040" }],
  "seq_poll_timeout": "30s", "seq_tx_poll_max_blocks": 15, "seq_poll_max_retries": 10,
  "seq_block_poll_max_amount": 100, "calibration_limit": 100 }
JSON
W="$lez/target/release/wallet"
printf '\n' | "$W" check-health >/dev/null   # creates storage with an empty password (throwaway)
for k in $(grep "import public --private-key" "$lez/Justfile" | grep -oE '[0-9a-f]{64}'); do
  "$W" account import public --private-key "$k" </dev/null >/dev/null
done
payer="$("$W" account list </dev/null 2>/dev/null | grep -oE 'Public/[1-9A-HJ-NP-Za-km-z]{32,44}' | head -1)"
[ -n "$payer" ] || { echo "no funded localnet account in the wallet" >&2; exit 1; }
echo "::endgroup::"

echo "::group::deploy program"
program="$(LEE_WALLET_HOME_DIR="$LEE_WALLET_HOME_DIR" WALLET="$W" PAYER="$payer" "$root/scripts/deploy-program.sh" | tail -1)"
echo "program=$program"
echo "::endgroup::"

echo "::group::storage nodes + fixture Geofabrik"
eval "$("$root/scripts/storage-two-nodes.sh" "$STORAGE_BIN" "$WORK/storage" | sed 's/^/export /')"
eval "$("$root/tools/fake-geofabrik.sh" "$WORK/geofabrik" 8099 asia/china/henan 261101 | sed 's/^/export /')"
echo "::endgroup::"

OSM_RPC=http://127.0.0.1:3040 OSM_PROGRAM="$program" OSM_HOST_ACCOUNT="${payer#Public/}" \
OSM_CHAIN="$root/programs/osm_registry/target/debug/osm-chain" OSM_BIN="$root/target/debug/osm" \
REGION=china/henan SNAPSHOT=261101 "$root/scripts/e2e-local.sh"

# P-R4 / H9: with download.geofabrik.de unreachable at the network level, discovery falls back to
# the bundled set and a hosted region still downloads. CI only (needs to edit /etc/hosts).
if [ "${BLOCK_GEOFABRIK:-0}" = "1" ]; then
  echo "::group::Geofabrik blocked"
  echo "0.0.0.0 download.geofabrik.de" | sudo tee -a /etc/hosts >/dev/null
  if curl -fsS --max-time 10 https://download.geofabrik.de/ >/dev/null 2>&1; then echo "Geofabrik still reachable; block failed" >&2; exit 1; fi
  unset OSM_GEOFABRIK_BASE
  osm="$root/target/debug/osm"
  "$osm" discover --json | python3 -I -c 'import json,sys; d=json.load(sys.stdin); assert len(d["regions"])==72 and d["source"].startswith("offline"), d["source"]; print("   discover:", d["source"][:60])'
  d2="$WORK/offline-dl"
  OSM_RPC=http://127.0.0.1:3040 OSM_PROGRAM="$program" "$osm" download --region china/henan --dir "$d2" --storage "$OSM_STORAGE_B" --json \
    | python3 -I -c 'import json,sys; d=json.load(sys.stdin); assert d["source"]=="storage", d; print("   download from storage with Geofabrik blocked:", d["from"])'
  echo "OFFLINE OK"
  echo "::endgroup::"
fi

# H32: capture the region account this run wrote and run the decoder tests on those real bytes.
echo "::group::decoder on freshly captured accounts"
acct="$(cd "$root" && cargo run -q --locked -p osm_sdk --example region_account -- "$program" china/henan)"
RPC=http://127.0.0.1:3040 PROGRAM="$program" "$root/tools/capture-registry-fixture.sh" "$WORK/live-fixture.json" "china/henan=$acct"
(cd "$root" && OSM_REQUIRE_LIVE_FIXTURE=1 OSM_LIVE_FIXTURE="$WORK/live-fixture.json" cargo test -q --locked -p osm_sdk --test fixture_live)
echo "::endgroup::"
