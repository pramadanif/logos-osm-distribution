# Self-review (evaluator method on our own repo)

Pinned commit: `b9053ae0a2d5b20aaf362bd5888f72e2a40daf33` (https://github.com/pramadanif/logos-osm-distribution, `main`).
Date: 2026-11-12. Reviewer: the build agent, following plan Appendix E and docs/status/e1-extra.md.
Previous review: 2026-11-09 at `9bee462`.

## Verdict

A working alpha on LEZ v0.3 and a proven step on LEZ 0.4-rc, not yet a submission. The SPEL
registry, SDK, C ABI and CLI run end to end on a local v0.3.1 sequencer with real Storage nodes,
in CI on every push. The program is deployed on the public v0.3 testnet but registers nothing
there yet (no public Storage node). The 0.4 actor registry passes 15 of 15 non-batch negative and
repeat-signer probes on a local v0.4.0-rc1 sequencer. All five Basecamp modules compile and
package on Linux and macOS, but none has been opened in Basecamp. Coverage (25 regions / 15
countries), the 0.4 testnet, adoption and the video are all open. 0 high findings open; 2 medium
(M3, M4 below).

## How I reviewed

Cloned the repository into a temporary directory at `b9053ae`, ran `cargo test --locked`, listed
tracked files for wallets, keys, passwords and private plan files, read the CI runs through the
API, and compared README and docs statements with code and evidence files.

## README says / Observed

| Statement | Observed |
|-----------|----------|
| `cargo test --locked` builds and tests | 76 passed, 0 failed (fresh clone, macOS arm64) |
| e2e on a real sequencer in CI | first green run 42755836260 at `0f9a9f2` |
| Basecamp modules built with logos-module-builder | modules run 42757080576 at `05b1389`: 5 `.lgx` on Linux amd64 and macOS |
| program on the public v0.3 testnet | `networks.json` id and ImageID match README (doc-truth job) |
| 0.4 actor registry | 15/15 non-batch probes, probes/results-v04.txt; CI job `e2e-v04` not green yet |
| cycle counts | v0.3 table and 0.4 table equal the raw JSON in probes/data (doc-truth job) |

Tracked files: no wallet, key, password or evaluator corpus; `docs/plan/` holds only the public
prize text (`prize-LP-0018.md`); the private plan and prompt files are gitignored.

## Findings

**M1, M2, L1 (fixed in the previous review).** Empty README, CI never run, synthetic test CID.

**M3 (open). Basecamp modules never loaded in Basecamp.** They compile and package, and the
manifests are tested, but no screen of the app or the templates has been seen running. Until then
H20 (every feature in the UI), D.6 (states, two profiles, Qt number formatting) and W5 are unproven.
Needs a desktop Basecamp session.

**M4 (open). No public data.** Nothing is registered on any public network, so P-F8 and the
coverage criteria (F.3–F.5) are unproven. Needs the public Storage node (VPS) and testnet 0.4.

**L2 (open). The 0.4 batch is unproven on a sequencer.** `RegisterBatch` is in the program and the
client, and type-checks; the locally deployed guest predates it. CI `e2e-v04` builds the current
guest and runs the batch cases.

**L3 (open). No SPEL IDL for 0.4.** SPEL generates plan/apply code for v0.3 only. The 0.4 program
is a plain actor program; no 0.4 IDL is claimed (H1 partly open).

**L4 (open). App bulk host is sequential.** The Basecamp app hosts several regions as one `host`
call each; one `register_batch` transaction exists only in the CLI path.

## What works (with evidence)

- Registry on v0.3: deploy, register, batch of 16, unhost, append-only history with hosts; 18/18
  negative and repeat-signer probes on a real sequencer (probes/results.txt).
- Registry on 0.4-rc: signer as host actor, clock-checked timestamp, region actor via PDA seed,
  same account layout read by the same SDK; 15/15 probes incl. a direct message to a region actor
  refused (probes/results-v04.txt).
- Storage: upload with retries, download by CID and MD5 check against two real nodes and in CI by
  an independent node; 2 GiB streamed at 2.7 MB peak memory.
- Downloads without Geofabrik (CI blocks it); dated first-of-month snapshots with MD5 from
  Geofabrik.
- C ABI under ASan/UBSan; decoder fuzzed; atomic local writes survive `kill -9`.
- Cycle counts: v0.3 register 49,970 and 0.4 Register 50,836 user cycles (docs/cycles.md).

## Not tested

Basecamp at runtime (M3), any public registration (M4), the 0.4 batch on a sequencer (L2),
testnet 0.4, Android, proving time.

## E.1 rows

Row-by-row status is in docs/criteria-checklist.md and the gate list in
docs/status/gap-audit.md (refreshed 2026-11-10); this review changes no row's status beyond them.

## Environment

macOS 26.0.1 arm64, rustc 1.95.0 (workspace), 1.98.1 (0.4 client). LEZ v0.3.1 `1a003bfb`,
v0.4.0-rc1 `1806e873`; Logos Storage v0.5.3; logos-module-builder `5f74cdb3`.

## Test traffic

Public v0.3 testnet: one faucet claim and one program deploy of 6 transactions (docs/test-traffic.md). Everything
else ran on local sequencers and local Storage nodes. Read-only requests to download.geofabrik.de.
