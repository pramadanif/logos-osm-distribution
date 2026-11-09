# Test traffic on shared networks (H40)

Everything this project sent to networks or services other people use. Local sequencers and
local Storage nodes are not listed (they are not shared).

| Date | Service | What | Volume |
|------|---------|------|--------|
| 2026-11-09 | download.geofabrik.de | `index-v1-nogeom.json` (2 fetches), HEAD of all 72 `-latest.osm.pbf` (size survey, 0.3 s apart), a few `.md5` files and HEAD/redirect checks | about 1 MB |
| 2026-11-09 | download.geofabrik.de | `asia/china/henan-261101.osm.pbf` (50,702,303 bytes) for local host/e2e runs, 4 full downloads before the verified cache was reused | about 200 MB |
| 2026-11-09 | GitHub releases | `logos-storage-nim` v0.5.3 binaries (macOS locally; Linux in CI per run) | ~9 MB local, ~59 MB per CI run |

No transaction has been sent to any public LEZ network, and nothing has been uploaded to a public
Logos Storage network.
