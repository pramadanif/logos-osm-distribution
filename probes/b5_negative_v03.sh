#!/usr/bin/env bash
# SC-B.5 / H30 / H31 / H37: negative and repeat-signer tests against a REAL LEZ v0.3 sequencer.
# Every case reads the region account back; "included in a block" is never taken as success (H33).
#
# REGION picks an Asian country region whose shard is still empty (default thailand).
# Needs: a running sequencer, a deployed program, a wallet with two funded public accounts.
#   SPEL=<spel binary> PROGRAM=<header account> HOST_A=<account> HOST_B=<account> \
#   LEE_WALLET_HOME_DIR=<wallet dir> probes/b5_negative_v03.sh
# Prints one "PASS|FAIL <case> <detail>" line per case and exits non-zero if any case fails.
set -euo pipefail
: "${SPEL:?}" "${PROGRAM:?}" "${HOST_A:?}" "${HOST_B:?}" "${LEE_WALLET_HOME_DIR:?}"
cd "$(dirname "$0")/../programs/osm_registry"
IDL=osm_registry-idl.json
fails=0
now_ms() { python3 -I -c 'import time; print(int(time.time()*1000))'; }
spel() { "$SPEL" --idl "$IDL" -p "$PROGRAM" -- "$@" </dev/null 2>&1; }
pda() { spel pda region_account --region "$1" | tail -1; }
state() { # region -> compact JSON of RegionAccount, or {} when the shard is empty
  # `spel inspect` exits 1 for an empty shard; empty is a valid answer here, so keep its output.
  { "$SPEL" inspect "$(pda "$1")" --idl "$IDL" --type RegionAccount --program "$PROGRAM" </dev/null 2>/dev/null || true; } \
    | python3 -I -c 'import sys,json; t=sys.stdin.read(); i=t.find("\n{"); print(json.dumps(json.loads(t[i:]),sort_keys=True) if i>=0 else "{}")'
}
reg() { # host region parent level cid url md5 version size ts
  spel register --host "$1" --region "$2" --parent "$3" --level "$4" --cid "$5" --source-url "$6" \
    --checksum "$7" --version "$8" --size-bytes "$9" --timestamp "${10}" | grep -E "tx_hash|confirmed|❌" || true
}
check() { # name expr-python(s=state json) detail
  local s; s="$(state "$4")"
  if python3 -I -c "import json,sys; s=json.loads(sys.argv[1]); sys.exit(0 if ($2) else 1)" "$s"; then
    echo "PASS $1 $3"
  else
    echo "FAIL $1 $3 state=$s"; fails=$((fails+1))
  fi
}
CID1=zDvZRwzmDhJHVcQyvkFBDXtck1bhACLTjt94z6puHN3dsfm5U7wb
CID2=zDvZRwzm3VHNvgC5Q2UWDxYHGYmMN2mcY3LniFRHvkVUYNMqiZbH
# A fresh country region per run (the cases assume an empty shard); default thailand.
R="${REGION:-thailand}"; URL="https://download.geofabrik.de/asia/$R-261101.osm.pbf"; MD5=0123456789abcdef0123456789abcdef
A_ID="Public/${HOST_A#Public/}"; B_ID="Public/${HOST_B#Public/}"

reg "$HOST_A" $R "" 0 $CID1 $URL $MD5 261101 1000 "$(now_ms)" >/dev/null
check p00_create "len(s['history'])==1 and s['history'][0]['registered_by']=='$A_ID'" "first register creates the region shard" $R

reg "$HOST_A" $R "" 0 $CID1 $URL $MD5 261101 1000 "$(now_ms)" >/dev/null
check p03a_same_signer_repeat "len(s['history'])==1 and s['history'][0]['hosts']==['$A_ID']" "same signer, same CID: idempotent" $R

reg "$HOST_B" $R "" 0 $CID1 $URL $MD5 261101 1000 "$(now_ms)" >/dev/null
check p01_copy_by_other_signer "len(s['history'])==1 and s['history'][0]['registered_by']=='$A_ID' and s['history'][0]['hosts']==['$A_ID','$B_ID']" "copied register only adds the copier as a host; ours stays" $R

reg "$HOST_A" $R "" 0 $CID2 $URL $MD5 261101 1000 "$(now_ms)" >/dev/null
check p03b_same_signer_new_cid "len(s['history'])==2 and s['history'][1]['cid']=='$CID2'" "same signer, third call, new CID appends" $R

spel unhost --host "$HOST_B" --region $R --cid $CID1 >/dev/null || true
check p03c_unhost "s['history'][0]['hosts']==['$A_ID'] and s['history'][0]['hosted']" "B removes only itself" $R

before="$(state $R)"
reg "$HOST_A" $R "" 0 $CID1 $URL $MD5 261101 1000 "$(( $(now_ms) - 3600000 ))" >/dev/null
check p06a_timestamp_past_window "json.dumps(s,sort_keys=True)==json.dumps(json.loads('''$before'''),sort_keys=True)" "timestamp 1 h old rejected, state unchanged" $R
reg "$HOST_A" $R "" 0 $CID1 $URL $MD5 261101 1000 "$(( $(now_ms) / 1000 ))" >/dev/null
check p06b_timestamp_in_seconds "json.dumps(s,sort_keys=True)==json.dumps(json.loads('''$before'''),sort_keys=True)" "seconds instead of ms rejected" $R

reg "$HOST_A" planet "" 0 $CID1 https://download.geofabrik.de/planet-261101.osm.pbf $MD5 261101 1000 "$(now_ms)" >/dev/null
check p04a_region_outside_set "s=={}" "planet never gets a shard" planet
reg "$HOST_A" india "" 0 $CID1 https://download.geofabrik.de/asia/india-261101.osm.pbf $MD5 261101 1000 "$(now_ms)" >/dev/null
check p04b_decomposed_country "s=={}" "india (decomposed) never gets a shard" india
reg "$HOST_A" india/central-zone china 1 $CID1 https://download.geofabrik.de/asia/india/central-zone-261101.osm.pbf $MD5 261101 1000 "$(now_ms)" >/dev/null
check p04c_wrong_parent "s=={}" "wrong parent rejected" india/central-zone
reg "$HOST_A" $R "" 0 $CID1 https://download.geofabrik.de/asia/$R-latest.osm.pbf $MD5 261101 1000 "$(now_ms)" >/dev/null
check p04d_undated_source "json.dumps(s,sort_keys=True)==json.dumps(json.loads('''$before'''),sort_keys=True)" "-latest source URL rejected" $R
reg "$HOST_A" $R "" 0 $CID1 $URL $MD5 261101 18446744073709551615 "$(now_ms)" >/dev/null
check p06c_size_u64_max "json.dumps(s,sort_keys=True)==json.dumps(json.loads('''$before'''),sort_keys=True)" "size u64::MAX rejected" $R

echo "SUMMARY fails=$fails"
[ "$fails" -eq 0 ]
