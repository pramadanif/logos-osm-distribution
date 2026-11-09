# Test traffic on shared networks (H40)

Everything this project sent to networks or services other people use. Local sequencers and
local Storage nodes are not listed (they are not shared).

| Date | Service | What | Volume |
|------|---------|------|--------|
| 2026-11-09 | download.geofabrik.de | `index-v1-nogeom.json` (2 fetches), HEAD of all 72 `-latest.osm.pbf` (size survey, 0.3 s apart), a few `.md5` files and HEAD/redirect checks | about 1 MB |
| 2026-11-09 | download.geofabrik.de | `asia/china/henan-261101.osm.pbf` (50,702,303 bytes) for local host/e2e runs, 4 full downloads before the verified cache was reused | about 200 MB |
| 2026-11-09 | GitHub releases | `logos-storage-nim` v0.5.3 binaries (macOS locally; Linux in CI per run) | ~9 MB local, ~59 MB per CI run |

| 2026-11-09 | lez-faucet `claimd` (v0.3 testnet) | one claim for deployer `Public/shgMqPgBJpwrXRjhhbjRDrxKFXnbogjyXBooKL3wDFC`, request `072d7288-2f16-4084-80ca-d1cbd58919b8`, tx `a7cb7da8cc66108a8aa7f712fbf3bf3b8c4854500d4e6b1ced399a1e5c6d5bd5` | 0.1 LEZ |
| 2026-11-09 | LEZ v0.3 public testnet `https://testnet.lez.logos.co` | program deploy (header + 5 segments, 6 transactions), program `5VZeCZKSzkQyYb2nPm4ShDAKpkwncvbQh58XSg4cCAcS`, ImageID `f329edb2…` | fee 3,629,536 lepta |

Nothing has been uploaded to a public Logos Storage network yet.
