#!/usr/bin/env bash
# Regenerates osm_sdk/tests/fixtures/geofabrik-index-subset.json from the live Geofabrik index (H32).
# Keeps the 72 predefined leaves, the 4 decomposed parents and a few look-alikes; drops geometry.
# Manual only: CI never calls Geofabrik (H26).
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
if [[ -n "${INDEX_FILE:-}" ]]; then
  cp "$INDEX_FILE" "$tmp/index.json"   # an index downloaded earlier, to avoid a second fetch
else
  curl -fsS --max-time 60 https://download.geofabrik.de/index-v1-nogeom.json -o "$tmp/index.json"
fi
python3 -I - "$tmp/index.json" "$root/regions/predefined.json" "$root/osm_sdk/tests/fixtures/geofabrik-index-subset.json" <<'PY'
import json, sys, datetime
idx = json.load(open(sys.argv[1]))
want = {r["geofabrik_id"] for r in json.load(open(sys.argv[2]))["regions"]}
want |= {"us", "india", "china", "russia", "georgia", "britain-and-ireland", "united-kingdom", "south-africa-and-lesotho"}
feats = [f for f in idx["features"] if f["properties"]["id"] in want]
missing = want - {f["properties"]["id"] for f in feats}
if missing:
    sys.exit(f"index lacks: {sorted(missing)}")
out = {"type": "FeatureCollection", "captured": datetime.date.today().isoformat(), "features": feats}
json.dump(out, open(sys.argv[3], "w"), indent=1)
open(sys.argv[3], "a").write("\n")
print(len(feats), "features")
PY
