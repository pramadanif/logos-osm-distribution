# Phase D: Basecamp modules + catalog

| SC | Status | Evidence / missing |
|----|--------|--------------------|
| SC-D.1 core module (mkLogosModule) exposing the SDK | NOT LOCKED | `basecamp/osm-distribution` built with `nix build ./basecamp/osm-distribution` on macOS arm64, 2026-11-09 (logos-module-builder rev 5f74cdb3, rustc 1.89 from nixpkgs): `lib/pramadani_osm_distribution_plugin.dylib` (8,967,768 bytes) and generated consumer API `include/pramadani_osm_distribution_api.h` with sync and async calls for `version`, `settings`, `configure`, `discover`, `resolve`, `fetch`, `hosted_status`, `check_updates`. Since then: `host`, `import_file`, `register_entry`, `unhost` added; writes go through `lez_core.send_generic_public_transaction` (flake input locked at lez_core fdd6a75) and are confirmed by read-back. That version is **not built yet**: building third-party code locally was refused by the session's safety check, so it is built in CI (`.github/workflows/modules.yml`, Linux + macOS). Missing: a green modules CI run, loading it in Basecamp (SC-−1.4) |
| SC-D.2 QML app | NOT LOCKED | `basecamp/osm-app` (`pramadani_osm_app`, ui_qml, C++ backend as module, `.rep` view contract): regions with hosted status, details (CID, MD5, snapshot, size, hosts), download, update check, single and bulk host (one region at a time), import, settings, activity list with submitting/confirmed/failed; all chain and Storage text rendered as plain text; size formatter without regular expressions. QML passes `qmllint`; C++ not compiled yet (CI). Bulk host is sequential `host` calls, not one `register_batch` transaction |
| SC-D.2b unique names, side-by-side with rival catalogs | NOT LOCKED | names chosen (ADR 0008); install test missing |
| SC-D.3 both templates inside Basecamp | NOT LOCKED | `examples/consumer-module` (`pramadani_osm_consumer_module`, C++ core module on `pramadani_osm_sdk`) and `examples/distribution-consumer-app` (`pramadani_osm_consumer_app`, app on `pramadani_osm_distribution`); written, not built or loaded yet |
| SC-D.4 root module.json | NOT LOCKED | `module.json` indexes the 5 manifests; `tests/module_manifests.py` checks paths, names (prefix, rival list), versions, `main`, dependencies vs flake inputs, ui_qml icon 256x256/view/rep, per-module data folder; passes locally (5 manifests, 0 failures), runs in CI `scripts` and `modules` |
| SC-D.5 catalog fork + release | NOT LOCKED | not started |
| SC-D.6 UI states, formatter in Basecamp Qt, two profiles | NOT LOCKED | per-instance data folder in the module (`instance_persistence_path`); UI missing |

Toolchain check: the unmodified `rust-with-external-lib` template also built
(`rust_external_lib_plugin.dylib`), so the nix setup on this Mac works.
