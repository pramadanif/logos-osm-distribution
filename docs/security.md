# Security

What this project trusts, what it does not, and the bound or check applied to every untrusted
input (H21). Each row names the code and the test that hold it. Status: LEZ v0.3 code; the
Basecamp UI and the LEZ 0.4 port are not built yet, so their rows are marked as planned.

## Trust model

- **Nobody is trusted to write correct registry data.** Anyone can register any CID for any
  region. The program enforces structure (region in the closed set, dated Geofabrik source,
  signer as `registered_by`, bounded history and hosts); it cannot check bytes. Consumers trust a
  file only after its MD5 equals the registered checksum, and coverage counts an entry only when
  the bytes also equal Geofabrik's published MD5 for that snapshot.
- **There is no admin.** The program has no owner, fee, treasury, curator or init step that a
  first caller could capture (ADR 0003).
- **Geofabrik is a source, not an authority at runtime.** Hosted regions download and verify
  without it.

## Untrusted inputs and their bounds

| Input | Who controls it | Bound / check | Code | Test |
|-------|-----------------|---------------|------|------|
| `register` / `register_batch` arguments | any signer | region in the 72-leaf set with matching parent and level; CID base58, 46–96 chars; source URL must be `https://download.geofabrik.de/<path of that region>-YYMMDD.osm.pbf` with a valid date equal to `version`; MD5 32 lowercase hex; size 1 B–64 GiB; timestamp ≥ 2026-11-01 in ms and bound to block time by a ±10 min validity window; batch 1–16 regions, no duplicates, each account the PDA of its region | `osm_core/src/entry.rs`, program `register*` | `osm_core/tests/entry.rs`; `probes/b5_negative_v03.sh` (18/18 on a real sequencer) |
| `registered_by`, `hosts` | taken from the transaction signer by the program, never from arguments | signer declared through SPEL `#[account(signer)]` | program | probes p01, p03 |
| Region account bytes read by the SDK | any writer through the program | ≤ 100 KiB, layout 1, region/parent/level equal to the set, ≤ 8 entries, ≤ 16 hosts each, CID valid; truncation, trailing bytes and huge length prefixes are errors (S170), never panics | `osm_sdk/src/registry.rs` `decode_region` | `tests/registry.rs`, `tests/decoder_mutation.rs` (200k mutations), `fuzz/` (4.55M runs) |
| Sequencer JSON-RPC answers | the configured sequencer | body ≤ 400 KiB, shard values must be bytes, RPC errors surfaced (S100/S170) | `RpcRegistry::fetch`, `account_balance` | `tests/registry.rs`, CLI tests |
| Geofabrik index | download.geofabrik.de or a network attacker | ≤ 16 MiB; entries matched to the set by exact PBF URL; names stripped of control and bidi characters, ≤ 128 bytes, never used as paths | `osm_sdk/src/discovery.rs`, `geofabrik.rs` | `tests/discovery.rs` |
| Geofabrik redirect and `.md5` | same | redirect must point to a dated file of the same region on download.geofabrik.de; `.md5` ≤ 1 KiB and must name the expected file | `geofabrik.rs`, `snapshot.rs` | `tests/geofabrik.rs`, `tests/snapshot_hash.rs` |
| Downloaded PBF bytes (Geofabrik or Storage) | anyone serving them | streamed to disk (1 MiB buffer); MD5 compared with the published or registered value; a mismatching file is deleted; free disk checked first; cache capped | `geofabrik.rs`, `fetch.rs`, `datadir.rs` | `tests/fetch.rs`, `tests/geofabrik.rs`, `examples/transfer.rs` (2 GiB, 2.7 MB RSS) |
| Storage node answers | the configured Storage node | CID response ≤ 256 B and validated; `exists` accepts only the two known shapes; CIDs validated before they appear in a URL path | `rest_storage.rs` | `tests/rest_storage.rs` |
| Local PBF for import | the user | must look like a PBF (`OSMHeader`), MD5 vs Geofabrik's published MD5 for the stated snapshot (S130/S132) | `hash.rs` `verify_import` | `tests/snapshot_hash.rs`; live run in docs/status/phase-C.md |
| Coverage manifest | the user | ≤ 1 MiB JSON, every field required, regions validated before any transaction | `osm_cli` `bulk-register` | live run in docs/status/phase-C.md |
| C ABI arguments | the calling module | non-null, UTF-8, JSON object, ≤ 64 KiB; panics caught at the boundary | `osm_ffi/src/lib.rs` | `osm_ffi/tests/ffi.rs`; C test under ASan/UBSan (CI `ffi-sanitizers`) |

## Local data (H38)

- Data and cache folders are created 0700 and files 0600 (downloads, cache, settings); an
  existing group/world-readable folder is tightened.
- Settings are written atomically: validate, temp file, fsync, rename, folder fsync. A writer
  killed mid-write leaves the old or the new file, never a broken one, and its temp file is
  removed on the next write (`probes/p10_atomic_kill.sh`, 200 rounds).
- The LEZ wallet creates `storage.json` with mode 0644; our setup steps `chmod 600` it
  (docs/BUGS_FILED.md #8). This project never stores a wallet password and never logs keys.

## Processes and network

- `osm-chain` is started with an argument array, never through a shell; remote strings never reach
  a shell or a file path (E53).
- Outbound hosts are limited to the list in ADR 0006; `scripts/preflight-submission.sh` check 9
  greps the code for anything else.
- Local sequencers are bound to 127.0.0.1 (`--listen-address`); LEZ's debug config binds 0.0.0.0
  without auth (docs/BUGS_FILED.md #7).

## Known limits

- Registry flooding: a signer who pays for 8 transactions can rotate a region's history; the good
  entry is restored by re-registering. Documented in docs/limitations.md (W4).
- Hosting exposes the host node's network address to Storage peers (ADR 0005, E71).
- Planned, not built: plain-text rendering of every remote string in the QML app (E8/H21), two
  isolated Basecamp profiles (H38), and the LEZ 0.4 versions of the program-side checks.
