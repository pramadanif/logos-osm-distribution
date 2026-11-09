#!/usr/bin/env bash
# Serves a tiny fixture with Geofabrik's file layout on 127.0.0.1:<port> for CI (H26: CI never
# downloads from download.geofabrik.de). Writes <region path>-<YYMMDD>.osm.pbf and its .md5.
# Usage: tools/fake-geofabrik.sh <dir> <port> <geofabrik_path> <YYMMDD> [size_bytes]
# Prints OSM_GEOFABRIK_BASE=http://127.0.0.1:<port>/ and keeps serving in the background.
set -euo pipefail
dir="$1"; port="$2"; path="$3"; snap="$4"; size="${5:-3145728}"
mkdir -p "$dir/$(dirname "$path")"
f="$dir/$path-$snap.osm.pbf"
python3 -I - "$f" "$size" <<'PY'
import sys
f, size = sys.argv[1], int(sys.argv[2])
head = b"\x00\x00\x00\x0e\x0a\x09OSMHeader"          # looks like a PBF header to our sniffing
x = 0x9E3779B97F4A7C15
body = bytearray(head)
while len(body) < size:
    x ^= (x << 13) & (2**64 - 1); x ^= x >> 7; x ^= (x << 17) & (2**64 - 1)
    body += x.to_bytes(8, "little")
open(f, "wb").write(bytes(body[:size]))
PY
md5="$(python3 -I -c 'import hashlib,sys; print(hashlib.md5(open(sys.argv[1],"rb").read()).hexdigest())' "$f")"
echo "$md5  $(basename "$f")" > "$f.md5"
# Whole subshell redirected and backgrounded, so no inherited stdout keeps a caller's pipe open.
(cd "$dir" && exec python3 -I -m http.server "$port" --bind 127.0.0.1) </dev/null >/dev/null 2>&1 &
for _ in $(seq 1 20); do curl -fsS "http://127.0.0.1:$port/$path-$snap.osm.pbf.md5" >/dev/null 2>&1 && break; sleep 0.5; done
curl -fsS "http://127.0.0.1:$port/$path-$snap.osm.pbf.md5" >/dev/null
echo "OSM_GEOFABRIK_BASE=http://127.0.0.1:$port/"
