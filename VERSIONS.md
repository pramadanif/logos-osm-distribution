# Versions

Single source of truth for every pinned upstream version. README, scripts, the nix flake and CI
read their pins from this file (H35). Changing a pin means changing it here first.

| Key | Value | Used by |
|-----|-------|---------|
| `LEZ_V04` | `v0.4.0-rc1` (`1806e873ea2ac0e1b608aa7cf2f9b6e1fb12db21`) | lez-adapter `lez-v04`, localnet e2e |
| `LEZ_V03` | `v0.3.1` (`1a003bfbfb89fcf987f62975cd08a401169e1b8c`) | public v0.3 testnet `https://testnet.lez.logos.co` runs this tag (lez-faucet `docs/testnet.md`); lez-adapter `lez-v03`, localnet e2e |
| `SPEL_V03` | `vpavlin/spel` `feat/lez-0.3-fixes` @ `e43b59be3a80a63e3c0aa5e80a4c020dbf6d65a1` | program + IDL for the v0.3 backend (depends on LEZ `v0.3.0`) |
| `SPEL_V04` | our port of `SPEL_V03` to LEZ `v0.4.0-rc1` (route b, chosen 2026-11-09); not started | program + IDL for the v0.4 backend |
| `STORAGE` | `logos-storage-nim` `v0.5.3`; linux-amd64 zip sha256 `1086fae4f283cab882f133abe4d5a88cfae8d61edba9f0094c7f75b368ec9d56`; darwin-arm64 zip sha256 `7f32f62a14e555399ee003f11d1e29bba9b6ff842d973f38b5b280f018c2f6ab` | local nodes, CI storage-retrieval |
| `RUST` | `1.95.0` (rust-toolchain.toml) | workspace build, CI lint |
| `RISC0` | `risc0-zkvm 3.0.5` (LEZ pin); local r0vm 3.0.6 | guest build |
| `GEOFABRIK_SNAPSHOT_RULE` | first-of-month dated files `<path>-YYMM01.osm.pbf` | coverage manifest |
