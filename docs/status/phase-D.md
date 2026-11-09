# Phase D: Basecamp modules + catalog

| SC | Status | Evidence / missing |
|----|--------|--------------------|
| SC-D.1 core module (mkLogosModule) exposing the SDK | NOT LOCKED | `basecamp/osm-distribution` built with `nix build ./basecamp/osm-distribution` on macOS arm64, 2026-11-09 (logos-module-builder rev 5f74cdb3, rustc 1.89 from nixpkgs): `lib/pramadani_osm_distribution_plugin.dylib` (8,967,768 bytes) and generated consumer API `include/pramadani_osm_distribution_api.h` with sync and async calls for `version`, `settings`, `configure`, `discover`, `resolve`, `fetch`, `hosted_status`, `check_updates`. Missing: loading it in Basecamp (SC-−1.4), write path through `lez_core`, Linux build in CI |
| SC-D.2 QML app | NOT LOCKED | not started |
| SC-D.2b unique names, side-by-side with rival catalogs | NOT LOCKED | names chosen (ADR 0008); install test missing |
| SC-D.3 both templates inside Basecamp | NOT LOCKED | not started |
| SC-D.4 root module.json | NOT LOCKED | not started |
| SC-D.5 catalog fork + release | NOT LOCKED | not started |
| SC-D.6 UI states, formatter in Basecamp Qt, two profiles | NOT LOCKED | per-instance data folder in the module (`instance_persistence_path`); UI missing |

Toolchain check: the unmodified `rust-with-external-lib` template also built
(`rust_external_lib_plugin.dylib`), so the nix setup on this Mac works.
