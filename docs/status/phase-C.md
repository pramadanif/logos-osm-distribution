# Phase C: SDK + CLI (started early, offline parts only)

Started 2026-11-09 while Phase −1, A and B wait on human gates. Nothing here touches LEZ.
Statuses stay NOT LOCKED until each SC is complete, including its on-chain part.

| SC | Status | What exists / what is missing |
|----|--------|-------------------------------|
| SC-C.1 discovery from index, filtered, version + level | LOCKED | `osm discover` (index or bundled set, level, parent, Geofabrik name; with --rpc/--program the hosted snapshot and host count from the registry), `osm snapshot` for the current dated snapshot. Fixture from the live index; live run on localnet |
| SC-C.2 host workflow | NOT LOCKED | `osm host`: dated first-of-month snapshot, resumable download with free-disk check, MD5 vs Geofabrik, Storage upload with retry, confirmed register through `osm-chain`; several `--region` = one batch. Run on localnet (china/henan 261101, CID zDvZRwzmA5LrY5Be…). Missing: bulk host with opt-out in the UI, payer-balance check |
| SC-C.3 local import, graceful checksum failure | LOCKED | `osm import <file> --region --snapshot`: MD5 vs Geofabrik's published MD5 for that snapshot, then store + confirmed register. Live (fixture Geofabrik + localnet): good file registered; one byte changed → `S130 ImportChecksumMismatch: MD5 04f39f01… does not match Geofabrik's 52f559d0… for china/jiangsu-261101`, exit 7, nothing stored; non-PBF → S132 |
| SC-C.4 query by region / parent / CID | LOCKED | `osm_sdk::registry` (PDA derivation equal to `spel pda` on 4 real vectors, bounded decoder, queries sorted by timestamp) tested on bytes captured from the v0.3.1 localnet; `osm lookup --region/--parent/--cid` run against it: parent `china` returned 4 leaves, unreachable RPC exit 3 |
| SC-C.5 update check per region path | LOCKED | `status::check_updates` (registered vs Geofabrik first-of-month, per region; tests incl. same-month = no update) and `osm update-check`; live on localnet: china/henan and kenya registered 261101, Geofabrik 261101, up to date |
| SC-C.6 download, Geofabrik fallback, offline | NOT LOCKED | `osm download`: newest hosted entry whose bytes match the registered MD5, from Storage; Geofabrik only when unhosted and allowed; junk entries skipped and deleted (tests). Live: node B fetched china/henan from node A with `--no-geofabrik`, MD5 = Geofabrik's 5f3c0413…. Missing: offline test with Geofabrik blocked at the network level |
| SC-C.6b streamed multi-GB, payer balance, wallet password | NOT LOCKED | 2 GiB through upload + network download with 2.7 MB peak RSS (probes/results.txt); fee-reserve check before every write (S105, live). Missing: wallet password via env/stdin (0.4 wallet encrypts storage, H28) |
| SC-C.7 CLI covers P-U3 | LOCKED | host (store + register; several regions = one batch), bulk-register (manifest, batches of 16, per-region `--skip`), lookup by region/parent/CID, update-check, download, import; all with `--json` and the exit codes of docs/error-codes.md. Each run against the v0.3.1 localnet on 2026-11-09 (this file and phase-B.md) |
| SC-C.8 docs/sdk.md worked example | LOCKED | docs/sdk.md; the example is `osm_sdk/examples/resolve.rs` (compiled by CI's clippy --all-targets) and was run against the localnet: resolve china/henan → CID, then download via node B, MD5 verified |
| SC-C.9 TxNotIncluded, bounds, fuzz, data dir, cache cap | NOT LOCKED | confirmed submit (S103), bounds and timestamps, 200k-mutation decoder test, data dir 0700/0600, atomic writes, PBF cache cap and free-disk check, C ABI under ASan/UBSan. Missing: open-ended `cargo fuzz` run |

Evidence: `cargo test --locked` on a fresh clone of 9bee462: 47 passed, 0 failed.
Live check (2 small requests, no PBF download): `osm snapshot malaysia-singapore-brunei --json`
returned snapshot 261108, MD5 9ca902dc90bfb86773189636aa363691, exit 0.

## E2E on local infrastructure (2026-11-09)

`scripts/e2e-local.sh` (local v0.3.1 sequencer, program G4s9EP4n…, Storage nodes A and B):
host china/henan via A → lookup by region and CID → download via B with Geofabrik disabled →
MD5 equals Geofabrik's published MD5 for 261101. Output ends with `E2E OK`, exit 0, 12.7 s with a
warm cache. Not yet in CI (needs the sequencer and program build in the job, SC-E.1).
