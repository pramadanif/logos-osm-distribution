# Versions

Single source of truth for every pinned upstream version. README, scripts, the nix flake and CI
read their pins from this file (H35). Changing a pin means changing it here first.

| Key | Value | Used by |
|-----|-------|---------|
| `LEZ_V04` | `v0.4.0-rc1` (`1806e873ea2ac0e1b608aa7cf2f9b6e1fb12db21`) | lez-adapter `lez-v04`, localnet e2e |
| `LEZ_V03` | `v0.3.2` (`ee6ed4470059d0ac113e2e358dbe25aad3e8b0a9`) | lez-adapter `lez-v03`, localnet e2e; public testnet version to be confirmed |
| `SPEL_V03` | `vpavlin/spel` `feat/lez-0.3-fixes` @ `e43b59be3a80a63e3c0aa5e80a4c020dbf6d65a1` | program + IDL for the v0.3 backend (depends on LEZ `v0.3.0`) |
| `SPEL_V04` | not chosen (human decision, docs/status/spel.md) | |
| `RUST` | `1.95.0` (rust-toolchain.toml) | workspace build, CI lint |
| `RISC0` | `risc0-zkvm 3.0.5` (LEZ pin); local r0vm 3.0.6 | guest build |
| `GEOFABRIK_SNAPSHOT_RULE` | first-of-month dated files `<path>-YYMM01.osm.pbf` | coverage manifest |
