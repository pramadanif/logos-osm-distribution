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
| 2026-11-21 | Logos Storage testnet (`logos.test`) | our public node `139.190.104.17:8070` joined the network; one 1 MiB random test file uploaded to it (CID `zDvZRwzm5R1mLaFHkPqqFqSJFMUJKvAXzTCSGJWEQaZuGvCyvaEG`) and fetched once by a local node | 1 MiB |
| 2026-11-23 | LEZ v0.3 public testnet | program redeploy for Geofabrik's new layout (header + 5 segments), program `9zuXeX1kzeukr1crbuWN92JirV3m7KMy9pvho9BCLhWw`, ImageID `60bb8c60…` (built and tested in CI e2e run 44884998387); one earlier attempt stopped at segment 4 (`transaction did not finalize`) | about 4.1 M lepta for both attempts |
| 2026-11-23 | download.geofabrik.de | `asia/china/henan/henan-261101.osm.pbf.md5` and HEAD checks (the cached 261101 file was reused after its MD5 matched) | a few kB |
| 2026-11-23 | our public Storage node | `china/henan` 261101 (50,702,303 bytes) uploaded through an SSH tunnel to its local API, CID `zDvZRwzmA5LrY5Be279q4gpJqxk6v6XrS59rHkKakguRwwFPEMfH` | 50.7 MB |
| 2026-11-23 | LEZ v0.3 public testnet | `register` china/henan in program `9zuXeX1k…`, tx `c6b27ecd966d2eacdc1cd127d45846afd306e074cac6030d99da95c06b3e6eb1`; included more than 2 minutes after sending (the CLI reported S103 at 120 s; the entry appeared later) | fee 419,128 lepta |
| 2026-11-23 | download.geofabrik.de | `osm discover --versions` once: the index and one HEAD per region (72, 0.3 s apart) | about 0.6 MB |
