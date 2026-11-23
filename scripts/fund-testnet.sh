#!/usr/bin/env bash
# Funds a LEZ account for registry writes (H25), by the first route that works:
#  1. the profile's faucet (`claimd` behind logos-co/lez-faucet): POST /v1/claim, then poll the
#     claim until it is included and check that the balance went up;
#  2. for a local chain only (ALLOW_GENESIS=1): a transfer from a funded genesis account in the
#     LEZ wallet at LEE_WALLET_HOME_DIR (the genesis keys are imported from LEZ's own Justfile at
#     run time by scripts/localnet-v04.sh, never copied into this repository).
#
#   PROFILE=<networks.json profile> ACCOUNT=<Public/... or base58> scripts/fund-testnet.sh
#   [ALLOW_GENESIS=1 WALLET=<wallet bin> LEE_WALLET_HOME_DIR=<dir> FROM=<Public/...> AMOUNT=<lepta>]
#
# Prints the new balance; exits non-zero if the account was not funded.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
: "${PROFILE:?}" "${ACCOUNT:?}"
acct="${ACCOUNT#Public/}"
field() { python3 -I -c 'import json,sys; print(json.load(open(sys.argv[1]))["profiles"][sys.argv[2]].get(sys.argv[3]) or "")' "$root/networks.json" "$PROFILE" "$1"; }
rpc="$(field rpc)"; faucet="$(field faucet)"
[ -n "$rpc" ] || { echo "profile $PROFILE has no rpc" >&2; exit 2; }
balance() {
  curl -fsS --max-time 30 -X POST "$rpc" -H 'content-type: application/json' \
    -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"getAccountBalance\",\"params\":[\"$acct\"]}" \
    | python3 -I -c 'import json,sys; print(int(json.load(sys.stdin)["result"]))'
}
before="$(balance)"
echo "balance before: $before lepta"

if [ -n "$faucet" ]; then
  rid="$(python3 -I -c 'import uuid; print(uuid.uuid4())')"
  echo "claiming from $faucet (request $rid)"
  if curl -fsS --max-time 30 -X POST "$faucet/v1/claim" -H 'content-type: application/json' \
       -d "{\"recipient\":\"Public/$acct\",\"request_id\":\"$rid\"}"; then
    echo
    status=""
    for _ in $(seq 1 30); do
      status="$(curl -fsS --max-time 20 "$faucet/v1/claim/$rid" | python3 -I -c 'import json,sys; print(json.load(sys.stdin).get("status",""))' || true)"
      [ "$status" = included ] || [ "$status" = failed ] && break
      sleep 10
    done
    echo "claim status: ${status:-unknown}"
  else
    echo "faucet refused the claim (one per account per day)" >&2
  fi
  after="$(balance)"
  if [ "$after" -gt "$before" ]; then echo "balance after: $after lepta"; exit 0; fi
fi

if [ "${ALLOW_GENESIS:-0}" = 1 ]; then
  : "${WALLET:?}" "${LEE_WALLET_HOME_DIR:?}" "${FROM:?}"
  echo "transferring ${AMOUNT:-100000000} lepta from $FROM (local chain)"
  "$WALLET" auth-transfer send --from "$FROM" --to "Public/$acct" --amount "${AMOUNT:-100000000}" </dev/null >/dev/null
  after="$(balance)"
  if [ "$after" -gt "$before" ]; then echo "balance after: $after lepta"; exit 0; fi
fi

echo "account not funded (balance $before lepta)" >&2
exit 1
