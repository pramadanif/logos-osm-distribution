#!/usr/bin/env bash
# H17 / SC-E.6: numbers and absolute claims in the docs must match the code and the data.
#  1. every `proof:` line in docs/claims.md exits 0;
#  2. region counts stated in the docs equal regions/predefined.json;
#  3. the cycle table in docs/cycles.md equals the raw benchmark JSON in probes/data/;
#  4. the program id and ImageID in README equal networks.json;
#  5. any "<N> tests" claim in README or the solution draft equals `cargo test` output.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"; cd "$root"
fails=0
pass() { echo "PASS $1"; }
fail() { echo "FAIL $1"; fails=$((fails+1)); }

n=0
while IFS= read -r line; do
  cmd="${line#proof: }"; n=$((n+1))
  if bash -c "set -o pipefail; $cmd" >/dev/null 2>&1; then pass "claim proof $n"; else fail "claim proof $n: $cmd"; fi
done < <(grep '^proof: ' docs/claims.md)

read -r total countries leaves < <(python3 -I -c 'import json; r=json.load(open("regions/predefined.json"))["regions"]; print(len(r), sum(x["level"]=="country" for x in r), sum(x["level"]=="subregion" for x in r))')
[ "$total/$countries/$leaves" = "72/48/24" ] && pass "regions 72/48/24" || fail "regions $total/$countries/$leaves"
grep -q "48 countries at country level and 24 subregions" README.md && pass "README region counts" || fail "README region counts"

python3 -I - <<'PY' && pass "cycles.md equals probes/data" || fail "cycles.md differs from probes/data"
import json, glob, re, sys
raw = json.load(open(sorted(glob.glob("probes/data/cycles-v03*.json"))[-1]))["rows"]
by = {}
for r in raw:
    by.setdefault(r["instruction"], {})[r["phase"]] = r
doc = open("docs/cycles.md").read()
for name, n in [("register", 1), ("register_batch n=1", 1), ("register_batch n=5", 5), ("register_batch n=10", 10), ("register_batch n=16", 16)]:
    p, a = by[name]["plan"]["user_cycles"], by[name]["apply"]["user_cycles"]
    op = "register" if name == "register" else "register_batch"
    row = f"| `{op}` | {n} | {p:,} | {a:,} | {p + a:,} |"
    if row not in doc:
        print("missing row:", row); sys.exit(1)
PY

python3 -I - <<'PY' && pass "cycles.md 0.4 table equals probes/data" || fail "cycles.md 0.4 table differs from probes/data"
import json, glob, sys
rows = json.load(open(sorted(glob.glob("probes/data/cycles-v04-*.json"))[-1]))["rows"]
doc = open("docs/cycles.md").read()
for case, op in [("register", "Register"), ("unhost", "Unhost")]:
    r = [x for x in rows if x.get("case") == case]
    get = lambda who: next((x["user_cycles"] for x in r if x["receiver"] == who), None)
    cell = lambda v: "—" if v is None else f"{v:,}"
    total = sum(x["user_cycles"] for x in r); padded = sum(x["padded_cycles"] for x in r)
    row = f"| `{op}` | {cell(get('host actor'))} | {cell(get('clock'))} | {cell(get('region actor'))} | {total:,} | {padded:,} |"
    if row not in doc:
        print("missing row:", row); sys.exit(1)
PY

python3 -I - <<'PY' && pass "README deployment equals networks.json" || fail "README deployment differs from networks.json"
import json, sys
p = json.load(open("networks.json"))["profiles"]["v03-public"]
t = open("README.md").read()
sys.exit(0 if p["program"] in t and p["program_image_id"] in t else 1)
PY

claims="$(grep -ohE '\b[0-9]+ tests\b' README.md docs/SOLUTION_DRAFT.md 2>/dev/null || true)"
if [ -n "$claims" ]; then
  actual="$(cargo test --locked -q 2>&1 | grep -oE 'test result: ok\. [0-9]+ passed' | awk '{s+=$4} END {print s}')"
  for c in $claims; do :; done
  echo "$claims" | grep -oE '^[0-9]+' | while read -r c; do [ "$c" = "$actual" ] || { echo "FAIL test count claim $c vs $actual"; exit 1; }; done && pass "test count claims" || fail "test count claims"
else
  pass "no test count claims"
fi

echo "SUMMARY fails=$fails"
[ "$fails" -eq 0 ]
