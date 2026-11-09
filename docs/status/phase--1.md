# Phase −1: environment + rival preflight

Session 1, 2026-11-09. Status per SC is binary.

| SC | Status | Evidence / reason |
|----|--------|-------------------|
| SC-−1.1 nix, cargo, rzup, r0vm, gh | HUMAN_BLOCKED | cargo 1.95.0, rzup, r0vm 3.0.6, gh present (`command -v`, docs/status/env.md). nix missing; the installer needs sudo |
| SC-−1.2 LEZ v0.4.0-rc1 standalone sequencer runs | LOCKED | `cargo build --release --features standalone -p sequencer_service` at `1806e873` exit 0 (7m37s, log .deps/build-lez-v04rc1.log). `sequencer_service setup ... --config lez/sequencer/service/networks/local.json --create-channel` then `start`: log "Starting RPC Server on 127.0.0.1:3040", "Block production scheduler started". JSON-RPC: `checkHealth` → null result (ok), `getLastBlockId` 2 then 3 after 20 s, `getProgramIds` → `privacy_preserving_circuit`. 2026-11-09T03:28Z |
| SC-−1.2b 0.4 core/node module split notes | NOT LOCKED | 0.4 breaking changes recorded from the release notes (env.md); Basecamp module split not yet read |
| SC-−1.3 Storage node upload/download by CID | LOCKED | `logos-storage-darwin-arm64-v0.5.3.zip` from the v0.5.3 release, sha256 `7f32f62a…f6ab` equal to the GitHub asset digest. Two nodes on one Mac: A `--api-port=8081 --listen-port=8071 --no-bootstrap-node`, B `--api-port=8082 --listen-port=8072 --bootstrap-node=<A's spr>`. 5 MiB random file uploaded to A gave CID `zDvZRwzmDhJHVcQyvkFBDXtck1bhACLTjt94z6puHN3dsfm5U7wb`; B `exists` false, then `GET /data/<cid>/network/stream` HTTP 200 in 0.76 s, `cmp` identical, MD5 `02ac44ed13bb2399de5a43568a3d2fd7` both sides, B `exists` true. `osm_sdk/tests/real_storage.rs` with OSM_STORAGE_A/B set: 1 passed. Findings: `/spr` returns JSON `{"spr": ...}`; `/exists` returns `{"<cid>": bool}`, not the documented `{"has": bool}`; `--disc-port` does not exist in v0.5.3; `--api-port` works but is missing from `--help` |
| SC-−1.4 Basecamp + hello mkLogosModule | HUMAN_BLOCKED | needs nix |
| SC-−1.5 SPEL rev for the LEZ in use | NOT LOCKED | v0.3: `vpavlin/spel` @ `e43b59be` chosen (docs/status/spel.md), not yet built. v0.4: human decision |
| SC-−1.5b Geofabrik dated snapshots + policy | LOCKED | docs/status/geofabrik.md; commands `curl -sI .../indonesia-latest.osm.pbf` → `Location: ...indonesia-261108.osm.pbf`; directory listing shows daily/monthly/yearly retention |
| SC-−1.5c testnet 0.4 funding route | HUMAN_BLOCKED | lez-faucet 0.5.0 serves legacy + v0.3 only; no 0.4 statement in the repo. Human must ask Logos |
| SC-−1.6 rivals snapshot | LOCKED | docs/status/rivals.md (2026-11-09 snapshot, three commands, exit 0) |

Phase −1: NOT LOCKED.
