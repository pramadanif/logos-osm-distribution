# Phase C: SDK + CLI (started early, offline parts only)

Started 2026-11-09 while Phase −1, A and B wait on human gates. Nothing here touches LEZ.
Statuses stay NOT LOCKED until each SC is complete, including its on-chain part.

| SC | Status | What exists / what is missing |
|----|--------|-------------------------------|
| SC-C.1 discovery from index, filtered, version + level | NOT LOCKED | `osm_sdk::discovery` + `geofabrik::discover`, fixture captured from the live index (80 features), `osm discover`. Live snapshot version per region works (`osm snapshot`). Missing: registry hosted state |
| SC-C.2 host workflow | NOT LOCKED | `osm host`: dated first-of-month snapshot, resumable download with free-disk check, MD5 vs Geofabrik, Storage upload with retry, confirmed register through `osm-chain`; several `--region` = one batch. Run on localnet (china/henan 261101, CID zDvZRwzmA5LrY5Be…). Missing: bulk host with opt-out in the UI, payer-balance check |
| SC-C.3 local import checksum failure graceful | NOT LOCKED | `hash::verify_import` (S130 message names both MD5s and the snapshot, S132 for non-PBF), `osm verify-file`. Missing: store + register |
| SC-C.4 query by region / parent / CID | LOCKED | `osm_sdk::registry` (PDA derivation equal to `spel pda` on 4 real vectors, bounded decoder, queries sorted by timestamp) tested on bytes captured from the v0.3.1 localnet; `osm lookup --region/--parent/--cid` run against it: parent `china` returned 4 leaves, unreachable RPC exit 3 |
| SC-C.5 update check per region path | LOCKED | `status::check_updates` (registered vs Geofabrik first-of-month, per region; tests incl. same-month = no update) and `osm update-check`; live on localnet: china/henan and kenya registered 261101, Geofabrik 261101, up to date |
| SC-C.6 download, Geofabrik fallback, offline | NOT LOCKED | `osm download`: newest hosted entry whose bytes match the registered MD5, from Storage; Geofabrik only when unhosted and allowed; junk entries skipped and deleted (tests). Live: node B fetched china/henan from node A with `--no-geofabrik`, MD5 = Geofabrik's 5f3c0413…. Missing: offline test with Geofabrik blocked at the network level |
| SC-C.6b streamed multi-GB, payer balance, wallet password | NOT LOCKED | upload streams from file with Content-Length; MD5 uses a 1 MiB buffer (3 MiB test). Missing: multi-GB run, balance, wallet |
| SC-C.7 CLI covers P-U3 | NOT LOCKED | `osm regions|discover|snapshot|verify-file`, `--json`, exit codes from docs/error-codes.md |
| SC-C.8 docs/sdk.md worked example | LOCKED | docs/sdk.md; the example is `osm_sdk/examples/resolve.rs` (compiled by CI's clippy --all-targets) and was run against the localnet: resolve china/henan → CID, then download via node B, MD5 verified |
| SC-C.9 TxNotIncluded, bounds, fuzz, data dir, cache cap | NOT LOCKED | confirmed submit: `lez_adapter_v03` waits for the change to be visible, else S103 TxNotIncluded; already-hosted entries send nothing (localnet run, commit 37594fe). Data dir modes and validated atomic writes tested. Missing: fuzzing, cache cap, free-disk check |

Evidence: `cargo test --locked` on a fresh clone of 9bee462: 47 passed, 0 failed.
Live check (2 small requests, no PBF download): `osm snapshot malaysia-singapore-brunei --json`
returned snapshot 261108, MD5 9ca902dc90bfb86773189636aa363691, exit 0.

## E2E on local infrastructure (2026-11-09)

`scripts/e2e-local.sh` (local v0.3.1 sequencer, program G4s9EP4n…, Storage nodes A and B):
host china/henan via A → lookup by region and CID → download via B with Geofabrik disabled →
MD5 equals Geofabrik's published MD5 for 261101. Output ends with `E2E OK`, exit 0, 12.7 s with a
warm cache. Not yet in CI (needs the sequencer and program build in the job, SC-E.1).
