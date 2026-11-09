#!/usr/bin/env bash
# H35: every script is strict (set -euo pipefail) and fails with a non-zero status when its
# required input is missing, instead of carrying on or exiting 0.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
fails=0
for s in "$root"/scripts/*.sh "$root"/tools/*.sh "$root"/probes/*.sh; do
  name="${s#$root/}"
  if ! grep -q '^set -euo pipefail' "$s"; then echo "FAIL $name: no 'set -euo pipefail'"; fails=$((fails+1)); continue; fi
  # Empty environment except PATH/HOME, no arguments, stdin closed, 60 s cap.
  if env -i PATH="$PATH" HOME="$HOME" timeout 60 bash "$s" </dev/null >/dev/null 2>&1; then
    echo "FAIL $name: exited 0 without its required input"; fails=$((fails+1))
  else
    echo "PASS $name"
  fi
done
echo "SUMMARY fails=$fails"
[ "$fails" -eq 0 ]
