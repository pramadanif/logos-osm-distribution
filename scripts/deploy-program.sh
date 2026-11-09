#!/usr/bin/env bash
# Deploys the built registry program with the LEZ wallet's program-loader and prints the header
# account (the program id since LEZ v0.2.5). Creates fresh header + segment accounts each run.
# Usage: LEE_WALLET_HOME_DIR=<dir> WALLET=<wallet bin> PAYER=<Public/...> scripts/deploy-program.sh
set -euo pipefail
: "${LEE_WALLET_HOME_DIR:?}" "${WALLET:?}" "${PAYER:?}"
root="$(cd "$(dirname "$0")/.." && pwd)"
bin="$root/programs/osm_registry/methods/guest/target/riscv32im-risc0-zkvm-elf/docker/osm_registry.bin"
[ -f "$bin" ] || { echo "program not built: run make -C programs/osm_registry build" >&2; exit 1; }
seg=$((96 * 1024))   # MAX_SEGMENT_DATA_LEN, LEZ v0.3.1 lez/programs/program_loader/core/src/lib.rs
size=$(wc -c < "$bin"); n=$(( (size + seg - 1) / seg ))
new() { "$WALLET" account new public </dev/null 2>&1 | grep -oE 'Public/[1-9A-HJ-NP-Za-km-z]{32,44}' | head -1; }
header="$(new)"; segs=(); for _ in $(seq 1 "$n"); do segs+=("$(new)"); done
out="$("$WALLET" program-loader deploy --elf "$bin" --header "$header" --segments "${segs[@]}" --payer "$PAYER" </dev/null 2>&1 | grep -v '^Transaction data is')"
id="$(printf '%s\n' "$out" | sed -n 's/^Program deployed. Header account id: //p')"
[ -n "$id" ] || { printf '%s\n' "$out" | tail -20 >&2; echo "deploy failed" >&2; exit 1; }
echo "$id"
