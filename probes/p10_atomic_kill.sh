#!/usr/bin/env bash
# H38 / p10: kill -9 a process in the middle of atomic writes, N times; the file must always be a
# complete, valid document (old or new), never empty or truncated, and stale temp files from
# killed writers are cleaned up (at most the one from the last killed run may remain).
#   probes/p10_atomic_kill.sh [rounds]
# no-required-input
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
rounds="${1:-200}"
(cd "$root" && cargo build -q --release -p osm_sdk --example atomic_writer)
bin="$root/target/release/examples/atomic_writer"
work="$(mktemp -d)"; trap 'rm -rf "$work"' EXIT
f="$work/settings.json"
echo '{"fill":"seed"}' > "$f"
bad=0
for i in $(seq 1 "$rounds"); do
  "$bin" "$f" & pid=$!
  # Kill after a random 1–60 ms, i.e. at varying points inside the write sequence.
  python3 -I -c "import random,time; time.sleep(random.uniform(0.001, 0.06))"
  kill -9 "$pid" 2>/dev/null || true; wait "$pid" 2>/dev/null || true
  if ! python3 -I -c 'import json,sys; json.load(open(sys.argv[1]))' "$f" 2>/dev/null; then
    bad=$((bad+1)); echo "round $i: file is not valid JSON ($(wc -c < "$f") bytes)"
  fi
done
leftovers=$(find "$work" -name '.settings.json.tmp-*' | wc -l | tr -d ' ')
echo "rounds=$rounds invalid=$bad temp_files_left=$leftovers"
[ "$bad" -eq 0 ] && [ "$leftovers" -le 1 ] && echo "PASS p10_atomic_kill" || { echo "FAIL p10_atomic_kill"; exit 1; }
