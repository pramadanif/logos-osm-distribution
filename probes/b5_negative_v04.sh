#!/usr/bin/env bash
# SC-B.5 / H30 / H31 / H37 on LEZ 0.4: the v0.3 negative and repeat-signer cases against a REAL
# 0.4 sequencer running the actor registry, plus the 0.4-only case of a message sent straight to
# a region actor. Every case reads the region account back (H33).
#
# Needs: a running 0.4 sequencer, the deployed actor program, a 0.4 wallet with two funded
# public accounts, and osm-chain-v04 built from programs/osm_registry_v04/client.
#   CHAIN=<osm-chain-v04> RPC=<url> PROGRAM=<program account> HOST_A=<account> HOST_B=<account> \
#   LEE_WALLET_HOME_DIR=<wallet dir> LEE_WALLET_PASSWORD=<password> probes/b5_negative_v04.sh
# REGION (default thailand) and BATCH_REGIONS (default "egypt nigeria morocco") must be regions
# whose accounts are still empty. Prints one "PASS|FAIL <case> <detail>" line per case and exits
# non-zero if any case fails.
set -euo pipefail
: "${CHAIN:?}" "${RPC:?}" "${PROGRAM:?}" "${HOST_A:?}" "${HOST_B:?}" "${LEE_WALLET_HOME_DIR:?}"
fails=0
now_ms() { python3 -I -c 'import time; print(int(time.time()*1000))'; }
c() { "$CHAIN" "$1" --rpc "$RPC" --program "$PROGRAM" "${@:2}" </dev/null 2>/dev/null || true; }
state() { "$CHAIN" state --rpc "$RPC" --program "$PROGRAM" --region "$1" </dev/null | tail -1; }
# Unchecked register: host region parent level cid url md5 version size ts
raw() { c probe-register --host "$1" --region "$2" --parent "$3" --level "$4" --cid "$5" --source-url "$6" \
  --md5 "$7" --version "$8" --size "$9" --timestamp "${10}" >/dev/null; sleep "${SETTLE:-20}"; }
check() { # name python-expr(s=state) detail region
  local s; s="$(state "$4")"
  if python3 -I -c "import json,sys; s=json.loads(sys.argv[1]); sys.exit(0 if ($2) else 1)" "$s" 2>/dev/null; then
    echo "PASS $1 $3"
  else
    echo "FAIL $1 $3 state=$s"; fails=$((fails+1))
  fi
}
same() { echo "json.dumps(s,sort_keys=True)==json.dumps(json.loads('''$1'''),sort_keys=True)"; }
CID1=zDvZRwzmDhJHVcQyvkFBDXtck1bhACLTjt94z6puHN3dsfm5U7wb
CID2=zDvZRwzm3VHNvgC5Q2UWDxYHGYmMN2mcY3LniFRHvkVUYNMqiZbH
CID3=zDvZRwzmAkSzTDPyfB3ZbR4Wf4PFGzEeGm7wrQ8m1Xx4GtqzYzXh
R="${REGION:-thailand}"; URL="https://download.geofabrik.de/asia/$R-261101.osm.pbf"; MD5=0123456789abcdef0123456789abcdef
A="${HOST_A#Public/}"; B="${HOST_B#Public/}"

c register --host "$A" --entry "$R|$CID1|$URL|$MD5|261101|1000" >/dev/null
check p00_create "len(s['history'])==1 and s['history'][0]['registered_by']=='$A'" "first register creates the region account" "$R"

c register --host "$A" --entry "$R|$CID1|$URL|$MD5|261101|1000" >/dev/null
check p03a_same_signer_repeat "len(s['history'])==1 and s['history'][0]['hosts']==['$A']" "same signer, same CID: idempotent" "$R"

c register --host "$B" --entry "$R|$CID1|$URL|$MD5|261101|1000" >/dev/null
check p01_copy_by_other_signer "len(s['history'])==1 and s['history'][0]['registered_by']=='$A' and s['history'][0]['hosts']==['$A','$B']" "copied register only adds the copier as a host" "$R"

c register --host "$A" --entry "$R|$CID2|$URL|$MD5|261101|1000" >/dev/null
check p03b_same_signer_new_cid "len(s['history'])==2 and s['history'][1]['cid']=='$CID2'" "same signer, new CID appends" "$R"

c unhost --host "$B" --region "$R" --cid "$CID1" >/dev/null
check p03c_unhost "s['history'][0]['hosts']==['$A'] and s['history'][0]['hosted']" "B removes only itself" "$R"

before="$(state "$R")"
raw "$A" "$R" "" 0 $CID3 "$URL" $MD5 261101 1000 "$(( $(now_ms) - 3600000 ))"
check p06a_timestamp_past_window "$(same "$before")" "timestamp 1 h old refused by the clock check" "$R"
raw "$A" "$R" "" 0 $CID3 "$URL" $MD5 261101 1000 "$(( $(now_ms) / 1000 ))"
check p06b_timestamp_in_seconds "$(same "$before")" "seconds instead of ms refused" "$R"
raw "$A" "$R" "" 0 $CID3 "https://download.geofabrik.de/asia/$R-latest.osm.pbf" $MD5 261101 1000 "$(now_ms)"
check p04d_undated_source "$(same "$before")" "-latest source URL refused" "$R"
raw "$A" "$R" "" 0 $CID3 "$URL" $MD5 261101 18446744073709551615 "$(now_ms)"
check p06c_size_u64_max "$(same "$before")" "size u64::MAX refused" "$R"
raw "$A" "$R" "" 0 $CID3 "$URL" $MD5 261201 1000 "$(now_ms)"
check p05_version_not_url_date "$(same "$before")" "version differs from the URL's date" "$R"
raw "$A" "$R" "" 0 not-a-cid "$URL" $MD5 261101 1000 "$(now_ms)"
check p07_bad_cid "$(same "$before")" "malformed CID refused" "$R"

raw "$A" planet "" 0 $CID1 https://download.geofabrik.de/planet-261101.osm.pbf $MD5 261101 1000 "$(now_ms)"
check p04a_region_outside_set "s=={}" "planet never gets an account" planet
raw "$A" india "" 0 $CID1 https://download.geofabrik.de/asia/india-261101.osm.pbf $MD5 261101 1000 "$(now_ms)"
check p04b_decomposed_country "s=={}" "india (decomposed) never gets an account" india
raw "$A" india/central-zone china 1 $CID1 https://download.geofabrik.de/asia/india/central-zone-261101.osm.pbf $MD5 261101 1000 "$(now_ms)"
check p04c_wrong_parent "s=={}" "wrong parent refused" india/central-zone

# 0.4 only: an authorized message straight to a region actor, skipping the host actor.
c probe-direct-apply --host "$A" --region "$R" --cid $CID3 >/dev/null; sleep "${SETTLE:-20}"
check p12_direct_to_region_actor "$(same "$before")" "region actor refuses a message not sent by a host actor" "$R"

# --- RegisterBatch ---------------------------------------------------------------------------
read -r B1 B2 B3 <<<"${BATCH_REGIONS:-egypt nigeria morocco}"
burl() { echo "https://download.geofabrik.de/africa/$1-261101.osm.pbf"; }
c register --host "$A" --entry "$B1|$CID1|$(burl "$B1")|$MD5|261101|1000" \
  --entry "$B2|$CID1|$(burl "$B2")|$MD5|261101|1000" --entry "$B3|$CID2|$(burl "$B3")|$MD5|261101|1000" >/dev/null
for r in "$B1" "$B2" "$B3"; do
  check "p08_batch_$r" "len(s['history'])==1 and s['history'][0]['registered_by']=='$A'" "one RegisterBatch transaction registers $r" "$r"
done

echo "SUMMARY fails=$fails"
[ "$fails" -eq 0 ]
