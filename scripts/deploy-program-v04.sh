#!/usr/bin/env bash
# Deploys the LEZ 0.4 actor registry (programs/osm_registry_v04/guest) with the 0.4 wallet's
# program-loader and prints the program id (the header account, without "Public/").
#
#   WALLET=<0.4 wallet bin> LEE_WALLET_HOME_DIR=<wallet> LEE_WALLET_PASSWORD=<password> \
#   PAYER=<Public/... funded account> [GUEST_BIN=<osm_registry_v04.bin>] [FUND=<lepta>] \
#   scripts/deploy-program-v04.sh
#
# Learned on v0.4.0-rc1 (docs/status/lez-04-port.md, docs/BUGS_FILED.md):
# - the wallet decides the segment count, not file size / 96 KiB: a deploy with one segment
#   account fails before sending and names the right count, so it is asked first;
# - `--fee-payer` takes the bare account id and is not used for segment uploads, so the header
#   and every segment account are funded first (FUND lepta each, default 100,000,000).
# Exits non-zero on any failure; prints nothing on stdout but the program id.
set -euo pipefail
: "${WALLET:?}" "${LEE_WALLET_HOME_DIR:?}" "${LEE_WALLET_PASSWORD:?}" "${PAYER:?}"
root="$(cd "$(dirname "$0")/.." && pwd)"
bin="${GUEST_BIN:-$root/programs/osm_registry_v04/guest/target/riscv32im-risc0-zkvm-elf/docker/osm_registry_v04.bin}"
[ -f "$bin" ] || { echo "program not built: cargo risczero build --manifest-path programs/osm_registry_v04/guest/Cargo.toml" >&2; exit 1; }
fund="${FUND:-100000000}"
payer="${PAYER#Public/}"
t() { perl -e "alarm $1; exec @ARGV[1..\$#ARGV]" "$@" </dev/null; }
new() { t 60 "$WALLET" account new public 2>&1 | grep -oE 'Public/[1-9A-HJ-NP-Za-km-z]{32,44}' | head -1; }

header="$(new)"; [ -n "$header" ] || { echo "could not create the header account" >&2; exit 1; }
n=$(t 120 "$WALLET" program-loader deploy --elf "$bin" --header "$header" --segments "$(new)" --fee-payer "$payer" 2>&1 \
  | grep -oE 'splits into [0-9]+ segment' | grep -oE '[0-9]+' || true)
[ -n "$n" ] || n=1
segs=(); for _ in $(seq 1 "$n"); do segs+=("$(new)"); done
for acc in "$header" "${segs[@]}"; do
  t 200 "$WALLET" auth-transfer send --from "Public/$payer" --to "$acc" --amount "$fund" > /dev/null 2>&1 \
    || { echo "funding $acc failed" >&2; exit 1; }
done
log="$(mktemp)"; trap 'rm -f "$log"' EXIT
t 1800 "$WALLET" program-loader deploy --elf "$bin" --header "$header" --segments "${segs[@]}" --fee-payer "$payer" > "$log" 2>&1 \
  || { grep -v "^Transaction data" "$log" | tail -20 >&2; echo "deploy failed" >&2; exit 1; }
echo "deployed $n segments" >&2
echo "${header#Public/}"
