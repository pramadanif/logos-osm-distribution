#!/usr/bin/env bash
# Preflight (plan section 7, H16): 29 checks; exit 0 only when all pass. See scripts/preflight_submission.py.
set -euo pipefail
exec python3 -I "$(cd "$(dirname "$0")" && pwd)/preflight_submission.py" "$@"
