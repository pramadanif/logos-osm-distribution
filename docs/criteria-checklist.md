# Criteria checklist

Every success and adoption criterion of LP-0018 with its status and where the evidence is.
Status words: **done** (evidence linked), **partial** (what is missing is named), **open**.
Testnet 0.4 is the only network that counts; v0.3 evidence is rehearsal and is labelled so.

## Functionality

| ID | Criterion | Status | Evidence |
|----|-----------|--------|----------|
| P-F1 | Region discovery from the Geofabrik index, version + level, predefined set only | done (CLI) | `osm discover`, `osm snapshot`; `osm_sdk/tests/discovery.rs`; docs/status/phase-C.md SC-C.1 |
| P-F2 | Host: Geofabrik → Storage → on-chain register | done on v0.3 localnet | `osm host`; scripts/e2e-local.sh; docs/status/phase-C.md |
| P-F3 | Download from Storage by CID, Geofabrik fallback, CID = integrity | done | `osm download`; `osm_sdk/tests/fetch.rs`; e2e step 3 |
| P-F4 | Bulk host with opt-out; only the non-overlapping set is exposed | done (CLI) | `osm host --region …` (one batch), `osm bulk-register --skip`; set-only rejection in program and SDK (probes p04) |
| P-F5 | Local import with MD5 check at import time | done | `osm import`; S130/S132 runs in docs/status/phase-C.md |
| P-F6 | Update check per region path | done | `osm update-check`; `osm_sdk/tests/status.rs` |
| P-F7 | Registry program: fields, unique region, query by region/parent/CID, batch | done on v0.3 | program `programs/osm_registry`; `osm lookup`; probes 18/18 |
| P-F8 | End to end on testnet 0.4 | open | needs 0.4 port (docs/status/lez-04-port.md) |
| P-F9 | SDK module with stable API, embedding example, app uses the full SDK | partial | SDK + C ABI + docs/sdk.md; Basecamp module and app not built (needs nix) |

## Usability

| ID | Criterion | Status | Evidence |
|----|-----------|--------|----------|
| P-U1 | Basecamp app GUI, build docs, downloadable assets | open | needs nix |
| P-U2 | SDK/module + API docs + embedding example running as a Basecamp module | partial | docs/sdk.md, `osm_sdk/examples/resolve.rs`, `osm_ffi`; Basecamp module open |
| P-U3 | CLI: host, batch-register, lookup region/parent/CID, update check, download, import | done | README "Command line"; docs/status/phase-C.md SC-C.7 |
| P-U4 | UX shows hosted status, versions, verification results | partial | CLI shows all three; app open |

## Reliability

| ID | Criterion | Status | Evidence |
|----|-----------|--------|----------|
| P-R1 | Upload retries with back-off, clear error after exhaustion | done | `osm_sdk/src/storage.rs`; `tests/storage_retry.rs` (S111) |
| P-R2 | Import checksum failure is graceful | done | S130 message names both MD5s and the snapshot |
| P-R3 | Entries ordered by timestamp | done | `registry::sort_entries`; `tests/registry.rs` |
| P-R4 | No mandatory central service; hosted regions work without Geofabrik | done (CI) | ADR 0006; e2e job blocks download.geofabrik.de and still discovers + downloads |

## Performance

| ID | Criterion | Status | Evidence |
|----|-----------|--------|----------|
| P-P1 | Cycle counts for register and batch-register | done on v0.3 | docs/cycles.md |

## Supportability

| ID | Criterion | Status | Evidence |
|----|-----------|--------|----------|
| P-S1 | Deployed and tested on testnet 0.4 | open | v0.3 public testnet deployment in README is rehearsal |
| P-S2 | Linux, macOS (and Windows) | partial | CI builds and tests on Linux, macOS, Windows; app not built |
| P-S3 | Modules built with logos-module-builder | open | needs nix |
| P-S4 | Catalog fork + release action + logos-repo.json | open | needs nix and the modules |
| P-S5 | IDL generated via SPEL | done for v0.3 | `idl/osm_registry.json`; CI `idl` job regenerates and diffs |
| P-S6 | E2E host → query → download against a standalone sequencer in CI | partial | `.github/workflows/e2e.yml` written; first green run pending |
| P-S7 | README: setup, program id, region list, CLI and app usage | partial | README (app usage open) |
| P-S8 | SDK docs with region → CID worked example | done | docs/sdk.md |

## Adoption (required)

| ID | Criterion | Status | Evidence |
|----|-----------|--------|----------|
| P-A1 | ≥15 countries, ≥25 verified entries on testnet 0.4 | open | tool ready: scripts/verify-onchain.sh; needs 0.4 and the public Storage node |
| P-A2 | ≥5 independent modules (≥3 UI apps) using the distribution module | open | human-led adoption track |
| P-A3 | macOS and Linux among adopters | open | human-led |
