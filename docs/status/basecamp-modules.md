# Basecamp modules on LEZ 0.3 and 0.4 (SC-−1.2b)

Read from GitHub on 2026-11-09.

## What exists

| Module | Repo | Version | Notes |
|--------|------|---------|-------|
| `lez_core` ("LEZ Core Module", type core) | logos-blockchain/logos-execution-zone-module | 0.5.1, flake pins LEZ `v0.3.1` | wraps the LEZ wallet (`wallet_ffi`); holds the user's keys |
| `storage_module` | logos-co/logos-storage-module | v3.0.2 | upload/download sessions, `exists` (ADR 0004) |
| blockchain node module | logos-blockchain/logos-blockchain-module | pushed 2026-11-04 | node side; the 0.4 roadmap splits node and wallet APIs (logos-co/ecosystem #214, #219, #236) |
| package manager | logos-co/logos-package-manager-module | pushed 2026-10-30 | catalog installs |

## The write path inside Basecamp

`lez_core` exposes `send_generic_public_transaction(account_ids, signing_requirements,
instruction, program_id_hex, payer_account_id_hex, shard_program_account_ids_hex)`
(`src/lez_core_module.h`). The instruction is the borsh bytes our IDL describes. So:

- our distribution module never holds keys or links LEZ wallet libraries; it builds the
  instruction, asks `lez_core` to sign and send, then confirms by reading the region account back
  through `osm_ffi` (H33);
- the CLI keeps `osm-chain` (`lez_adapter_v03`) for headless use.

## What the 0.4 split changes for us

The 0.4 roadmap separates the node API and the wallet API (ecosystem #214/#219/#236), and LEZ
0.4 itself moves to actors (docs/status/lez-04-port.md). `lez_core` 0.5.1 is still v0.3.1. For our
module this means one adapter seam: on 0.4 the same call goes to whichever wallet module the 0.4
Basecamp ships, with an actor-shaped message instead of a v0.3 instruction. The consumer API of
our module (`discover`, `resolve`, `fetch`, `checkUpdates`, `hostedStatus`) does not change, so
module authors are not affected (plan §2 "Frozen consumer API").
