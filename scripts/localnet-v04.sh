#!/usr/bin/env bash
# SC-A.5: the actor registry on a local LEZ v0.4.0-rc1 standalone sequencer, end to end.
# Starts the sequencer, makes a wallet from the genesis keys in LEZ's own Justfile (read at run
# time, never copied into this repository), deploys the guest with the program-loader, then runs
# probes/b5_negative_v04.sh (register, copy, repeat, unhost, batch, refused cases).
#
#   LEZ_DIR=<built logos-execution-zone v0.4.0-rc1 checkout> \
#   GUEST_BIN=<osm_registry_v04.bin from cargo risczero build> \
#   CHAIN=<osm-chain-v04> WORK=<empty dir> scripts/localnet-v04.sh
#
# PORT (default 3041) is the RPC port. Exits non-zero when any step or probe fails; a missing
# binary or a sequencer that does not start is a failure, never a skip.
set -euo pipefail
: "${LEZ_DIR:?}" "${GUEST_BIN:?}" "${CHAIN:?}" "${WORK:?}"
PORT="${PORT:-3041}"
SEQ="$LEZ_DIR/target/release/sequencer_service"
W="$LEZ_DIR/target/release/wallet"
for f in "$SEQ" "$W" "$GUEST_BIN" "$CHAIN" "$LEZ_DIR/Justfile"; do
  [ -e "$f" ] || { echo "missing: $f" >&2; exit 2; }
done
mkdir -p "$WORK"; WORK="$(cd "$WORK" && pwd)"
umask 077
t() { perl -e "alarm $1; exec @ARGV[1..\$#ARGV]" "$@" </dev/null; }

# --- sequencer ---------------------------------------------------------------------------------
# A fresh chain every run, set up the way LEZ's own `just run-sequencer-standalone` does (the
# funding key is the local Bedrock node's public key, read from LEZ's Justfile).
fund_key=$(grep -E '^BEDROCK_FUNDING_PUBLIC_KEY' "$LEZ_DIR/Justfile" | grep -oE '[0-9a-f]{64}' || true)
[ -n "$fund_key" ] || { echo "no BEDROCK_FUNDING_PUBLIC_KEY in $LEZ_DIR/Justfile" >&2; exit 2; }
rm -rf "$WORK/sequencer_home"
(cd "$LEZ_DIR" && "$SEQ" setup "$WORK/sequencer_home" --config lez/sequencer/service/networks/local.json \
  --funding-public-key "$fund_key" --create-channel) > "$WORK/setup.log" 2>&1 \
  || { echo "sequencer setup failed" >&2; tail -20 "$WORK/setup.log" >&2; exit 3; }
RUST_LOG=info,risc0_zkvm=warn,kameo=warn "$SEQ" start "$WORK/sequencer_home" \
  --rpc-listen-address "127.0.0.1:$PORT" --metrics-listen-address "127.0.0.1:$((PORT + 6000))" > "$WORK/sequencer.log" 2>&1 &
SEQ_PID=$!
trap 'kill $SEQ_PID 2>/dev/null || true' EXIT
for _ in $(seq 1 90); do grep -q "RPC Server on" "$WORK/sequencer.log" && break; kill -0 $SEQ_PID 2>/dev/null || break; sleep 2; done
grep -q "RPC Server on" "$WORK/sequencer.log" || { echo "sequencer did not start" >&2; tail -30 "$WORK/sequencer.log" >&2; exit 3; }
RPC="http://127.0.0.1:$PORT"

# --- wallet ------------------------------------------------------------------------------------
export LEE_WALLET_HOME_DIR="$WORK/wallet"
mkdir -p "$LEE_WALLET_HOME_DIR"
cat > "$LEE_WALLET_HOME_DIR/wallet_config.json" <<EOF
{ "sequencer": { "sequencer_addr": "$RPC" },
  "seq_poll_timeout": "30s", "seq_tx_poll_max_blocks": 15, "seq_poll_max_retries": 10, "seq_block_poll_max_amount": 100 }
EOF
LEE_WALLET_PASSWORD="$(python3 -I -c 'import secrets; print(secrets.token_urlsafe(24))')"
export LEE_WALLET_PASSWORD
t 60 "$W" check-health > "$WORK/health.out" 2>&1 || { echo "wallet check-health failed" >&2; exit 4; }
rm -f "$WORK/health.out"   # holds the new wallet's recovery phrase
keys=$(grep "import public --private-key" "$LEZ_DIR/Justfile" | grep -oE '[0-9a-f]{64}' || true)
[ -n "$keys" ] || { echo "no genesis keys found in $LEZ_DIR/Justfile" >&2; exit 4; }
for k in $keys; do t 60 "$W" account import public --private-key "$k" > /dev/null 2>&1; done
HOSTS=(); while read -r a; do HOSTS+=("$a"); done < <(t 60 "$W" account list 2>/dev/null | grep -oE 'Public/[1-9A-HJ-NP-Za-km-z]{32,44}' | head -2)
[ "${#HOSTS[@]}" -eq 2 ] || { echo "need two funded genesis accounts, got ${#HOSTS[@]}" >&2; exit 4; }

# --- deploy ------------------------------------------------------------------------------------
# The program-loader splits the ELF into 96 KiB segments, one account each. It does not use
# --fee-payer for segment uploads (docs/BUGS_FILED.md), so every new account is funded first.
# The wallet decides how many segments the ELF needs (it is not the file size / 96 KiB), so ask
# it first: a deploy with one segment account fails before sending and names the right count.
new() { t 60 "$W" account new public 2>&1 | grep -oE 'Public/[1-9A-HJ-NP-Za-km-z]{32,44}' | head -1; }
H=$(new)
n=$(t 120 "$W" program-loader deploy --elf "$GUEST_BIN" --header "$H" --segments "$(new)" --fee-payer "${HOSTS[0]#Public/}" 2>&1 \
  | grep -oE 'splits into [0-9]+ segment' | grep -oE '[0-9]+' || true)
[ -n "$n" ] || n=1
S=(); for _ in $(seq 1 "$n"); do S+=("$(new)"); done
for acc in "$H" "${S[@]}"; do
  t 200 "$W" auth-transfer send --from "${HOSTS[0]}" --to "$acc" --amount 100000000 > /dev/null 2>&1 \
    || { echo "funding $acc failed" >&2; exit 4; }
done
t 1800 "$W" program-loader deploy --elf "$GUEST_BIN" --header "$H" --segments "${S[@]}" --fee-payer "${HOSTS[0]#Public/}" \
  > "$WORK/deploy.log" 2>&1 || { echo "deploy failed" >&2; grep -v "^Transaction data" "$WORK/deploy.log" | tail -20 >&2; exit 4; }
PROGRAM="${H#Public/}"
echo "program $PROGRAM ($n segments) on $RPC"

# --- probes ------------------------------------------------------------------------------------
CHAIN="$CHAIN" RPC="$RPC" PROGRAM="$PROGRAM" HOST_A="${HOSTS[0]}" HOST_B="${HOSTS[1]}" \
  "$(dirname "$0")/../probes/b5_negative_v04.sh" | tee "$WORK/probes.txt"
