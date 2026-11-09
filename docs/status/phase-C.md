# Phase C: SDK + CLI (started early, offline parts only)

Started 2026-11-09 while Phase −1, A and B wait on human gates. Nothing here touches LEZ.
Statuses stay NOT LOCKED until each SC is complete, including its on-chain part.

| SC | Status | What exists / what is missing |
|----|--------|-------------------------------|
| SC-C.1 discovery from index, filtered, version + level | NOT LOCKED | `osm_sdk::discovery` + `geofabrik::discover`, fixture captured from the live index (80 features), `osm discover`. Live snapshot version per region works (`osm snapshot`). Missing: registry hosted state |
| SC-C.2 host workflow | NOT LOCKED | upload with retry (`storage::upload_with_retry`), REST client. Missing: PBF download, register |
| SC-C.3 local import checksum failure graceful | NOT LOCKED | `hash::verify_import` (S130 message names both MD5s and the snapshot, S132 for non-PBF), `osm verify-file`. Missing: store + register |
| SC-C.4 query by region / parent / CID | LOCKED | `osm_sdk::registry` (PDA derivation equal to `spel pda` on 4 real vectors, bounded decoder, queries sorted by timestamp) tested on bytes captured from the v0.3.1 localnet; `osm lookup --region/--parent/--cid` run against it: parent `china` returned 4 leaves, unreachable RPC exit 3 |
| SC-C.5 update check | NOT LOCKED | snapshot lookup exists; comparison with registry missing |
| SC-C.6 download, Geofabrik fallback, offline | NOT LOCKED | `RestStorage::download_to_file` (atomic), offline discovery tested with Geofabrik disabled, unreachable and HTTP 429. Missing: real node, fallback download |
| SC-C.6b streamed multi-GB, payer balance, wallet password | NOT LOCKED | upload streams from file with Content-Length; MD5 uses a 1 MiB buffer (3 MiB test). Missing: multi-GB run, balance, wallet |
| SC-C.7 CLI covers P-U3 | NOT LOCKED | `osm regions|discover|snapshot|verify-file`, `--json`, exit codes from docs/error-codes.md |
| SC-C.8 docs/sdk.md | NOT LOCKED | not written |
| SC-C.9 TxNotIncluded, bounds, fuzz, data dir, cache cap | NOT LOCKED | confirmed submit: `lez_adapter_v03` waits for the change to be visible, else S103 TxNotIncluded; already-hosted entries send nothing (localnet run, commit 37594fe). Data dir modes and validated atomic writes tested. Missing: fuzzing, cache cap, free-disk check |

Evidence: `cargo test --locked` on a fresh clone of 9bee462: 47 passed, 0 failed.
Live check (2 small requests, no PBF download): `osm snapshot malaysia-singapore-brunei --json`
returned snapshot 261108, MD5 9ca902dc90bfb86773189636aa363691, exit 0.
