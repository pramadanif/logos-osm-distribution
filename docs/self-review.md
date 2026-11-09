# Self-review (evaluator method on our own repo)

Pinned commit: `9bee4623b2a4ed046066e85a199576d8a3757a26` (https://github.com/pramadanif/logos-osm-distribution, `main`).
Date: 2026-11-09. Reviewer: the build agent, following plan Appendix E and docs/status/e1-extra.md.

## Verdict

Early-stage repository: the frozen region set, shared validation, design records, and the
offline half of the SDK and CLI (discovery, dated snapshots, MD5 import check, Storage REST
client with retry, private data folder). Nothing on-chain, no Basecamp module. What exists is
tested: 47 tests pass on a fresh clone and CI is green on Linux, macOS and Windows. It is not yet
usable for hosting or downloading through Logos. 0 high, 0 medium findings open.

## How I reviewed

Cloned https://github.com/pramadanif/logos-osm-distribution into a temporary directory,
ran `cargo test --locked`, listed tracked files for secrets and private files, read CI
check-runs for HEAD through the API, and compared README statements with the code.

## README says / Observed

| README says | Observed |
|-------------|----------|
| \"Nothing is deployed and there is no release yet\" | true: no program, no catalog |
| `cargo test --locked` builds and tests | 47 passed, 0 failed |
| CI (not claimed in README) | 3/3 jobs success on 9bee462 |

## Findings

**M1 (fixed). README was empty.** Now states status and what works. Fixed in b71cce0.

**M2 (fixed). CI never ran.** Green on 3 OSes at 9bee462 (run 42555186573).

**L1. CID validation is structural.** `validate_cid` accepts any base58btc string starting with
`z` of 46–96 characters; the test CID is synthetic. Pinned to the real Storage format after the
first real upload (SC-−1.3).

## What works

- `osm regions`, `osm discover --no-geofabrik`: 72 regions offline; `osm snapshot <region>`
  reads the dated snapshot from Geofabrik's redirect and its MD5 (checked live once).
- Storage REST client: request shape, Content-Length upload, atomic download, error classes,
  CID check before any request (local scripted server).

- `regions/predefined.json` and `osm_core::PREDEFINED` agree, 72 leaves, no overlap (5 tests).
- Entry validation rejects out-of-set regions, wrong parent/level, undated or foreign source
  URLs (including a look-alike host and a non-ASCII digit), bad MD5 forms without normalising,
  sizes 0 and above 64 GiB including `u64::MAX`, timestamps 0, 1 and below the floor (8 tests).

## Not tested

Everything on-chain, a real Storage node, Basecamp, ASan/UBSan, fuzzing, two profiles,
catalogs. The REST client has only met a scripted server, not a real node.

## E.1 rows

All rows 1–55 and E56–E87: open, no code yet in their area, except
- 13 (§2 guards 2–3): off-chain part done in osm_core (closed set, size cap); on-chain open.
- 15 (H37 bounds): size and timestamp bounds in osm_core with edge tests; program, SDK, FFI open.
- 19, 26, E85: N/A (no key derivation, no secret hashing, no own identity).
- 31 (SPEL): open, route recorded in docs/status/spel.md.

## Environment

macOS 26.0.1 arm64, rustc 1.95.0. No network writes.

## Test traffic

No transaction or upload to any public network. Read-only requests to download.geofabrik.de:
the index (twice), a few HEAD requests and `.md5` files, on 2026-11-09.
