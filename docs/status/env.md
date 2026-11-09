# Environment (Phase −1)

Host: macOS 26.0.1, arm64 (Apple silicon), 16 GB RAM, 50 GB free disk on 2026-11-09.
Recorded 2026-11-09. Every line below is an observed command result, not an expectation.

## Tools

| Tool | Observed | Needed for |
|------|----------|------------|
| rustc / cargo | 1.95.0 | our workspace; LEZ v0.4.0-rc1 pins 1.98.1 in rust-toolchain.toml |
| rzup | present; r0vm 3.0.6, cargo-risczero 3.0.6, risc0 rust 1.97.0 | guest build (LEZ pins risc0-zkvm 3.0.5) |
| gh | logged in as pramadanif | API checks, releases |
| docker | 29.1.2 (Docker Desktop started this session) | LEZ docker-all-in-one localnet |
| nix | **missing** | logos-module-builder, Basecamp, `nix build .#lgx-portable` (SC-−1.1, −1.4, Phase D) |
| just | missing (brew install possible) | LEZ Justfile recipes |
| cmake, qmake6, go, python3 | present | misc |

Disk warning: 50 GB free. A LEZ release build, a risc0 toolchain, a nix store with Qt and
Basecamp, and test PBFs can each take 10–30 GB. Coverage PBFs never go on this Mac (VPS only).

## Upstream versions (gh api, 2026-11-09)

| Component | Version / commit | Notes |
|-----------|------------------|-------|
| LEZ | `v0.4.0-rc1` = `1806e873ea2ac0e1b608aa7cf2f9b6e1fb12db21` (2026-10-29) | newest tag; also `v0.3.2` = `ee6ed447…` (2026-10-29), `v0.3.1` = `1a003bfb…` |
| SPEL upstream | `main` @ `c6f73ce21e` (2026-10-13, LEZ 0.2.5-rc2 compat); release `v0.7.0-rc.3` (2026-10-19) | no LEZ 0.3/0.4 support; `release/v0.4.0` branch is SPEL's own v0.4.0 from May, not LEZ 0.4 |
| SPEL v0.3 port | `vpavlin/spel` `feat/lez-0.3-fixes` @ `e43b59be3a80a63e3c0aa5e80a4c020dbf6d65a1` (2026-10-30) | depends on LEZ tag `v0.3.0` (`lee`, `lee_core` aliased as `nssa`, `nssa_core`) |
| lez-faucet | release `lez_faucet_ui-v0.5.0` (2026-10-26) | supports legacy (Piñata) and v0.3 (hosted claimd); no 0.4 mention in repo |
| logos-basecamp | pre-release `pre-release-4cea4bb-1433` (2026-11-06), default branch `master` | |
| logos-module-builder | `0.3.3` (2026-10-06), pushed 2026-11-01 | |
| logos-modules-release-base | `index` release (2026-05-19) | fork target for our catalog |
| logos-modules-release-action | `v1.12` (2026-10-30) | |
| logos-storage-module | `v3.0.2` (2026-10-03) | |
| logos-storage-nim | `v0.5.3` (2026-10-03) | |

## LEZ v0.4.0-rc1 facts read from the tag (README, Justfile, release notes)

- Standalone sequencer: `just run-sequencer-standalone` = `cargo run --features standalone
  --release -p sequencer_service -- start lez/sequencer/service/sequencer_home` after a one-off
  `setup ... --config lez/sequencer/service/networks/local.json --funding-public-key <key>
  --create-channel`. The `standalone` feature mocks Bedrock (`sequencer_bedrock_actor/mock`).
- `docker compose up` runs Bedrock + a four-sequencer devnet + indexer + explorer. README warns
  the image build needs about 14 GB for Docker; on this 16 GB Mac prefer the native standalone
  build or the published per-tag images.
- Breaking changes relevant to us (release notes):
  `feat!: transactional actors with side-effects` (#935) → programming model, read before Phase B;
  `feat!(fees): pptx fees with an in-proof reserve and refund, tx dry-runs` (#985) → payer-balance
  maths in H25 must be redone for 0.4, and dry-runs may give us the fee estimate;
  `feat(lez/wallet)!: password encrypts storage with argon2` (#987) → H28;
  `feat!(ffi): FFI primitives unification` (#993), `feat!(core_ffi): Sequencer-Wallet FFI` (#996).
- `lez/testnet_initial_state` still exists in the tag (fallback funding route (a), keys read at
  runtime only, never copied).

## Status of SC-−1.x

See docs/status/phase--1.md.
