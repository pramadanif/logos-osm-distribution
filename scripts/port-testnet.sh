#!/usr/bin/env bash
# Port to a (new or reset) network in one run (W8): build the program, deploy it, replay the
# coverage manifest (CIDs live in Logos Storage, independent of the chain), and verify.
#   PROFILE=<networks.json profile> LEE_WALLET_HOME_DIR=<wallet> WALLET=<lez wallet bin> \
#   PAYER=<Public/...> OSM_CHAIN=<osm-chain bin> OSM_STORAGE=<Storage API> \
#   [MANIFEST=coverage/manifest.json] scripts/port-testnet.sh
# Prints the new program id and the verification result; updates nothing in git by itself.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
: "${PROFILE:?}" "${LEE_WALLET_HOME_DIR:?}" "${WALLET:?}" "${PAYER:?}" "${OSM_CHAIN:?}" "${OSM_STORAGE:?}"
manifest="${MANIFEST:-$root/coverage/manifest.json}"
[ -f "$manifest" ] || { echo "no coverage manifest at $manifest" >&2; exit 2; }
rpc="$(python3 -I -c 'import json,sys; print(json.load(open(sys.argv[1]))["profiles"][sys.argv[2]]["rpc"] or "")' "$root/networks.json" "$PROFILE")"
[ -n "$rpc" ] || { echo "profile $PROFILE has no rpc" >&2; exit 2; }
echo "1. build program (Docker, reproducible ImageID)"
image="$(cd "$root" && cargo risczero build --manifest-path programs/osm_registry/methods/guest/Cargo.toml 2>&1 | grep -oE 'ImageID: [0-9a-f]{64}' | tail -1 | cut -d' ' -f2)"
[ -n "$image" ] || { echo "guest build failed" >&2; exit 1; }
echo "   ImageID $image"
(cd "$root" && cargo build --release --locked -p osm_cli)
echo "2. deploy to $rpc"; program="$("$root/scripts/deploy-program.sh" | tail -1)"; echo "   program $program"
echo "3. replay coverage"; "$root/target/release/osm" bulk-register --manifest "$manifest" --rpc "$rpc" --program "$program" --host "${PAYER#Public/}"
echo "4. verify"; work="$(mktemp -d)"; trap 'rm -rf "$work"' EXIT
"$root/target/release/osm" verify-coverage --rpc "$rpc" --program "$program" --storage "$OSM_STORAGE" --dir "$work" || status=$?
echo "Record in networks.json: profiles.$PROFILE.program=$program program_image_id=$image"
exit "${status:-0}"
